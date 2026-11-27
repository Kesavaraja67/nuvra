// nuvra-platform-win/src/audio_loopback.rs
// Live WASAPI loopback capture for real-time FFT media visualizer (Section 14.2).
// Runs ONLY while media is playing (zero idle CPU when paused/stopped).
// R-13: Unsafe confined here with // SAFETY: comments.

#![allow(unsafe_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, WAVEFORMATEX,
};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};

const RING_CAP: usize = 1024;

pub struct LoopbackCapture {
    active: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    ring: Arc<Mutex<Vec<f32>>>,
}

impl LoopbackCapture {
    pub fn new() -> Self {
        let active = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let ring = Arc::new(Mutex::new(vec![0.0; RING_CAP]));
        let (a, s, r) = (Arc::clone(&active), Arc::clone(&stop), Arc::clone(&ring));
        let _ = std::thread::Builder::new()
            .name("nuvra-loopback".into())
            .spawn(move || loopback_worker(a, s, r));
        Self { active, stop, ring }
    }

    pub fn set_active(&self, playing: bool) {
        self.active.store(playing, Ordering::Release);
    }

    /// Copy up to 1024 most recent mono PCM samples into `out`.
    pub fn copy_latest_samples(&self, out: &mut Vec<f32>) {
        out.clear();
        if let Ok(guard) = self.ring.lock() {
            out.extend_from_slice(&guard);
        }
    }
}

impl Default for LoopbackCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LoopbackCapture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

fn loopback_worker(active: Arc<AtomicBool>, stop: Arc<AtomicBool>, ring: Arc<Mutex<Vec<f32>>>) {
    crate::clipboard::init_com_mta();
    let mut client: Option<(IAudioClient, IAudioCaptureClient, usize, bool)> = None;

    while !stop.load(Ordering::Acquire) {
        if !active.load(Ordering::Acquire) {
            if let Some((ref ac, ..)) = client {
                // SAFETY: Valid started IAudioClient stopped cleanly when playback pauses.
                unsafe {
                    let _ = ac.Stop();
                }
                client = None;
            }
            std::thread::sleep(Duration::from_millis(80));
            continue;
        }
        if client.is_none() {
            client = unsafe { open_loopback_client() };
            if client.is_none() {
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
        }
        if let Some((_, ref cap, ch, is_f32)) = client {
            // SAFETY: Valid initialized loopback IAudioCaptureClient polling available packets.
            let ok = unsafe { drain_packets(cap, ch, is_f32, &ring) };
            if !ok {
                invalidate_ring(&ring);
                client = None;
            }
        }
        std::thread::sleep(Duration::from_millis(16));
    }
}

pub(crate) fn invalidate_ring(ring: &Arc<Mutex<Vec<f32>>>) {
    if let Ok(mut guard) = ring.lock() {
        guard.clear();
    }
}

unsafe fn open_loopback_client() -> Option<(IAudioClient, IAudioCaptureClient, usize, bool)> {
    let enumerator: IMMDeviceEnumerator =
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
    let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
    let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None).ok()?;
    let fmt_ptr: *mut WAVEFORMATEX = audio_client.GetMixFormat().ok()?;
    let channels = (*fmt_ptr).nChannels.max(1) as usize;
    let tag = (*fmt_ptr).wFormatTag;
    let bits = (*fmt_ptr).wBitsPerSample;
    let is_f32 = bits == 32 && (tag == 3 || tag == 0xFFFE);
    let init_res = audio_client.Initialize(
        AUDCLNT_SHAREMODE_SHARED,
        AUDCLNT_STREAMFLAGS_LOOPBACK,
        1_000_000,
        0,
        fmt_ptr,
        None,
    );
    CoTaskMemFree(Some(fmt_ptr.cast()));
    init_res.ok()?;
    let capture: IAudioCaptureClient = audio_client.GetService().ok()?;
    audio_client.Start().ok()?;
    Some((audio_client, capture, channels, is_f32))
}

unsafe fn drain_packets(
    cap: &IAudioCaptureClient,
    channels: usize,
    is_f32: bool,
    ring: &Arc<Mutex<Vec<f32>>>,
) -> bool {
    loop {
        let Ok(pkt_len) = cap.GetNextPacketSize() else {
            return false;
        };
        if pkt_len == 0 {
            return true;
        }
        let mut data_ptr: *mut u8 = std::ptr::null_mut();
        let mut frames: u32 = 0;
        let mut flags: u32 = 0;
        if cap
            .GetBuffer(&mut data_ptr, &mut frames, &mut flags, None, None)
            .is_err()
        {
            return false;
        }
        if frames > 0 {
            if (flags & 0x2) != 0 {
                push_silence_frames(ring, frames as usize);
            } else if !data_ptr.is_null() {
                push_mono_frames(data_ptr, frames as usize, channels, is_f32, ring);
            }
        }
        if cap.ReleaseBuffer(frames).is_err() {
            return false;
        }
    }
}

pub(crate) fn push_ring_samples(ring: &Arc<Mutex<Vec<f32>>>, samples: &[f32]) {
    if samples.is_empty() {
        return;
    }
    let tail = &samples[samples.len().saturating_sub(RING_CAP)..];
    if let Ok(mut guard) = ring.lock() {
        let keep = RING_CAP.saturating_sub(tail.len());
        if keep < guard.len() {
            let drain_n = guard.len() - keep;
            guard.drain(..drain_n);
        }
        guard.extend_from_slice(tail);
    }
}

pub(crate) fn push_silence_frames(ring: &Arc<Mutex<Vec<f32>>>, frames: usize) {
    let count = frames.min(RING_CAP);
    if count == 0 {
        return;
    }
    let zeros = vec![0.0f32; count];
    push_ring_samples(ring, &zeros);
}

unsafe fn push_mono_frames(
    ptr: *const u8,
    frames: usize,
    ch: usize,
    is_f32: bool,
    ring: &Arc<Mutex<Vec<f32>>>,
) {
    let mut batch = Vec::with_capacity(frames);
    if is_f32 {
        let floats = std::slice::from_raw_parts(ptr.cast::<f32>(), frames * ch);
        for frame in floats.chunks_exact(ch) {
            let sum: f32 = frame.iter().copied().sum();
            batch.push((sum / ch as f32).clamp(-1.0, 1.0));
        }
    } else {
        let shorts = std::slice::from_raw_parts(ptr.cast::<i16>(), frames * ch);
        for frame in shorts.chunks_exact(ch) {
            let sum: f32 = frame.iter().map(|&v| f32::from(v) / 32768.0).sum();
            batch.push((sum / ch as f32).clamp(-1.0, 1.0));
        }
    }
    push_ring_samples(ring, &batch);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_retains_newest_samples_and_appends_silence() {
        let ring = Arc::new(Mutex::new(vec![0.5f32; RING_CAP]));
        // Packet > RING_CAP retains the newest tail [1024..2048)
        let pkt: Vec<f32> = (0..(RING_CAP * 2)).map(|i| i as f32).collect();
        push_ring_samples(&ring, &pkt);
        {
            let guard = ring.lock().expect("lock");
            assert_eq!(guard.len(), RING_CAP);
            assert_eq!(guard[0], RING_CAP as f32);
            assert_eq!(guard[RING_CAP - 1], (RING_CAP * 2 - 1) as f32);
        }

        // Silent packet >= RING_CAP replaces all samples with zeros
        push_silence_frames(&ring, RING_CAP);
        let guard = ring.lock().expect("lock");
        assert!(guard.iter().all(|&s| s == 0.0));
        drop(guard);

        // Capture failure invalidation empties stale samples until fresh packets arrive
        invalidate_ring(&ring);
        assert!(ring.lock().expect("lock").is_empty());
        push_ring_samples(&ring, &[0.25, -0.25]);
        assert_eq!(ring.lock().expect("lock").as_slice(), &[0.25, -0.25]);
    }
}
