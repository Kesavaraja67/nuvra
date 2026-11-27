// nuvra-mod-media/src/gsmtc.rs
// Non-blocking WinRT GSMTC (Global System Media Transport Controls) across all active sessions.
// R-13: No unsafe.

#![forbid(unsafe_code)]

use nuvra_core_shared::event::MediaSnapshot;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

enum GsmtcCmd {
    TogglePlayPause(flume::Sender<bool>),
    Next(flume::Sender<bool>),
    Prev(flume::Sender<bool>),
    SeekSeconds(f32, flume::Sender<bool>),
    Stop,
}

pub struct GsmtcController {
    latest: Arc<Mutex<Option<MediaSnapshot>>>,
    cmd_tx: flume::Sender<GsmtcCmd>,
}

impl GsmtcController {
    pub fn new() -> Self {
        let latest = Arc::new(Mutex::new(None));
        let (cmd_tx, cmd_rx) = flume::unbounded();
        let state_ref = Arc::clone(&latest);
        let _ = std::thread::Builder::new()
            .name("nuvra-gsmtc".into())
            .spawn(move || worker_loop(state_ref, cmd_rx));
        Self { latest, cmd_tx }
    }

    /// Return the most recent playing or paused GSMTC media/video snapshot without blocking the UI thread.
    pub fn query_current_session(&mut self) -> Option<MediaSnapshot> {
        self.latest.lock().ok().and_then(|g| g.clone())
    }

    pub fn toggle_play_pause(&self) -> bool {
        self.send_cmd(GsmtcCmd::TogglePlayPause)
    }

    pub fn next_track(&self) -> bool {
        self.send_cmd(GsmtcCmd::Next)
    }

    pub fn previous_track(&self) -> bool {
        self.send_cmd(GsmtcCmd::Prev)
    }

    pub fn seek_seconds(&self, seconds: f32) -> bool {
        let (tx, rx) = flume::bounded(1);
        if self
            .cmd_tx
            .send(GsmtcCmd::SeekSeconds(seconds.max(0.0), tx))
            .is_err()
        {
            return false;
        }
        rx.recv_timeout(Duration::from_millis(600)).unwrap_or(false)
    }

    fn send_cmd<F>(&self, build: F) -> bool
    where
        F: FnOnce(flume::Sender<bool>) -> GsmtcCmd,
    {
        let (tx, rx) = flume::bounded(1);
        if self.cmd_tx.send(build(tx)).is_err() {
            return false;
        }
        rx.recv_timeout(Duration::from_millis(600)).unwrap_or(false)
    }
}

impl Drop for GsmtcController {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(GsmtcCmd::Stop);
    }
}

impl Default for GsmtcController {
    fn default() -> Self {
        Self::new()
    }
}

fn worker_loop(latest: Arc<Mutex<Option<MediaSnapshot>>>, cmd_rx: flume::Receiver<GsmtcCmd>) {
    nuvra_platform_win::clipboard::init_com_mta();
    let mut manager: Option<GlobalSystemMediaTransportControlsSessionManager> = None;
    loop {
        if manager.is_none() {
            manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .ok()
                .and_then(|op| op.get().ok());
        }
        let found = manager.as_ref().and_then(find_active_snapshot);
        let (snap, active_session) = match found {
            Some((s, sess)) => (Some(s), Some(sess)),
            None => (None, None),
        };
        if let Ok(mut guard) = latest.lock() {
            *guard = snap;
        }
        match cmd_rx.recv_timeout(Duration::from_millis(350)) {
            Ok(GsmtcCmd::Stop) | Err(flume::RecvTimeoutError::Disconnected) => break,
            Ok(GsmtcCmd::TogglePlayPause(reply)) => {
                let target = active_session
                    .clone()
                    .or_else(|| manager.as_ref().and_then(pick_control_session));
                let ok = target
                    .and_then(|s| s.TryTogglePlayPauseAsync().ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Ok(GsmtcCmd::Next(reply)) => {
                let target = active_session
                    .clone()
                    .or_else(|| manager.as_ref().and_then(pick_control_session));
                let ok = target
                    .and_then(|s| s.TrySkipNextAsync().ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Ok(GsmtcCmd::Prev(reply)) => {
                let target = active_session
                    .clone()
                    .or_else(|| manager.as_ref().and_then(pick_control_session));
                let ok = target
                    .and_then(|s| s.TrySkipPreviousAsync().ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Ok(GsmtcCmd::SeekSeconds(secs, reply)) => {
                let ticks = (secs.max(0.0) as f64 * 10_000_000.0) as i64;
                let target = active_session
                    .clone()
                    .or_else(|| manager.as_ref().and_then(pick_control_session));
                let ok = target
                    .and_then(|s| s.TryChangePlaybackPositionAsync(ticks).ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Err(flume::RecvTimeoutError::Timeout) => {}
        }
    }
}

const WINDOWS_TO_UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;

fn current_windows_ticks() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            WINDOWS_TO_UNIX_EPOCH_TICKS
                .saturating_add((d.as_secs() as i64).saturating_mul(10_000_000))
                .saturating_add((d.subsec_nanos() / 100) as i64)
        })
        .unwrap_or(0)
}

pub(crate) fn compute_timeline_seconds(
    start_ticks: i64,
    end_ticks: i64,
    pos_ticks: i64,
    last_updated_ticks: i64,
    now_win_ticks: i64,
    is_playing: bool,
    playback_rate: f64,
) -> (f32, f32) {
    let min_ticks = start_ticks.max(0);
    let end_nonneg = end_ticks.max(0);
    let max_ticks = end_nonneg.max(min_ticks);
    let rate = if playback_rate.is_finite() && playback_rate >= 0.0 {
        playback_rate
    } else {
        1.0
    };
    let elapsed_ticks =
        if is_playing && last_updated_ticks > 0 && now_win_ticks > last_updated_ticks {
            (now_win_ticks - last_updated_ticks) as f64 * rate
        } else {
            0.0
        };
    let advanced = (pos_ticks.max(0) as f64) + elapsed_ticks;
    let clamped = if end_nonneg > min_ticks {
        advanced.clamp(min_ticks as f64, max_ticks as f64)
    } else {
        advanced.max(min_ticks as f64)
    };
    (
        (clamped / 10_000_000.0) as f32,
        (end_nonneg as f64 / 10_000_000.0) as f32,
    )
}

fn session_playback_info(
    session: &GlobalSystemMediaTransportControlsSession,
) -> (Option<bool>, f64) {
    let Ok(info) = session.GetPlaybackInfo() else {
        return (None, 1.0);
    };
    let status = info.PlaybackStatus().ok();
    let rate = info
        .PlaybackRate()
        .ok()
        .and_then(|r| r.Value().ok())
        .filter(|&v| v.is_finite() && v >= 0.0)
        .unwrap_or(1.0);
    match status {
        Some(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing) => {
            (Some(true), rate)
        }
        Some(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused) => (Some(false), 0.0),
        _ => (None, 0.0),
    }
}

fn pick_control_session(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
) -> Option<GlobalSystemMediaTransportControlsSession> {
    if let Ok(cur) = manager.GetCurrentSession() {
        if session_playback_info(&cur).0 == Some(true) {
            return Some(cur);
        }
    }
    if let Ok(list) = manager.GetSessions() {
        let len = list.Size().unwrap_or(0);
        for i in 0..len {
            if let Ok(s) = list.GetAt(i) {
                if session_playback_info(&s).0 == Some(true) {
                    return Some(s);
                }
            }
        }
        if let Ok(cur) = manager.GetCurrentSession() {
            if session_playback_info(&cur).0 == Some(false) {
                return Some(cur);
            }
        }
        for i in 0..len {
            if let Ok(s) = list.GetAt(i) {
                if session_playback_info(&s).0 == Some(false) {
                    return Some(s);
                }
            }
        }
    }
    manager.GetCurrentSession().ok()
}

fn friendly_source_app(session: &GlobalSystemMediaTransportControlsSession) -> String {
    let raw = session
        .SourceAppUserModelId()
        .ok()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let lower = raw.to_lowercase();
    if lower.contains("chrome") {
        "Chrome".into()
    } else if lower.contains("msedge") || lower.contains("edge") {
        "Edge".into()
    } else if lower.contains("firefox") {
        "Firefox".into()
    } else if lower.contains("spotify") {
        "Spotify".into()
    } else if lower.contains("vlc") {
        "VLC".into()
    } else if lower.contains("zune") || lower.contains("mediaplayer") {
        "Media Player".into()
    } else {
        let clean = raw
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or("")
            .trim_end_matches(".exe");
        if clean.is_empty() {
            "Active Media".into()
        } else {
            clean.into()
        }
    }
}

fn find_active_snapshot(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
) -> Option<(MediaSnapshot, GlobalSystemMediaTransportControlsSession)> {
    // 1. Prefer playing session (current first, then all sessions)
    if let Ok(cur) = manager.GetCurrentSession() {
        let (state, rate) = session_playback_info(&cur);
        if state == Some(true) {
            if let Some(snap) = snapshot_from_session(&cur, true, rate) {
                return Some((snap, cur));
            }
        }
    }
    if let Ok(list) = manager.GetSessions() {
        let len = list.Size().unwrap_or(0);
        for i in 0..len {
            if let Ok(s) = list.GetAt(i) {
                let (state, rate) = session_playback_info(&s);
                if state == Some(true) {
                    if let Some(snap) = snapshot_from_session(&s, true, rate) {
                        return Some((snap, s));
                    }
                }
            }
        }
        // 2. Fall back to paused sessions with metadata so transport controls stay available
        if let Ok(cur) = manager.GetCurrentSession() {
            let (state, _) = session_playback_info(&cur);
            if state == Some(false) {
                if let Some(snap) = snapshot_from_session(&cur, false, 0.0) {
                    return Some((snap, cur));
                }
            }
        }
        for i in 0..len {
            if let Ok(s) = list.GetAt(i) {
                let (state, _) = session_playback_info(&s);
                if state == Some(false) {
                    if let Some(snap) = snapshot_from_session(&s, false, 0.0) {
                        return Some((snap, s));
                    }
                }
            }
        }
    }
    None
}

fn snapshot_from_session(
    session: &GlobalSystemMediaTransportControlsSession,
    is_playing: bool,
    playback_rate: f64,
) -> Option<MediaSnapshot> {
    let app = friendly_source_app(session);
    let props = session
        .TryGetMediaPropertiesAsync()
        .ok()
        .and_then(|op| op.get().ok());
    let raw_title = props
        .as_ref()
        .and_then(|p| p.Title().ok())
        .map(|s| s.to_string_lossy().trim().to_string())
        .unwrap_or_default();
    let raw_artist = props
        .as_ref()
        .and_then(|p| p.Artist().ok())
        .map(|s| s.to_string_lossy().trim().to_string())
        .unwrap_or_default();

    if !is_playing && raw_title.is_empty() && raw_artist.is_empty() {
        return None;
    }
    let title = if raw_title.is_empty() {
        format!("{app} Playing")
    } else {
        raw_title
    };
    let artist = if raw_artist.is_empty() {
        app
    } else {
        raw_artist
    };

    let (position_s, duration_s) = if let Ok(timeline) = session.GetTimelineProperties() {
        let start_ticks = timeline.StartTime().map(|s| s.Duration).unwrap_or(0);
        let end_ticks = timeline.EndTime().map(|e| e.Duration).unwrap_or(0);
        let pos_ticks = timeline.Position().map(|p| p.Duration).unwrap_or(0);
        let updated_ticks = timeline
            .LastUpdatedTime()
            .map(|dt| dt.UniversalTime)
            .unwrap_or(0);
        compute_timeline_seconds(
            start_ticks,
            end_ticks,
            pos_ticks,
            updated_ticks,
            current_windows_ticks(),
            is_playing,
            playback_rate,
        )
    } else {
        (0.0, 0.0)
    };

    Some(MediaSnapshot {
        title,
        artist,
        is_playing,
        has_art: false,
        art_key: None,
        position_s,
        duration_s,
        volume_norm: 1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gsmtc_worker_queries_nonblocking() {
        let mut ctrl = GsmtcController::new();
        std::thread::sleep(Duration::from_millis(200));
        let _snap = ctrl.query_current_session();
    }

    #[test]
    fn test_timeline_position_advances_and_clamps() {
        let sec = 10_000_000i64;
        // 10s recorded 5s ago at 1.0x within [0s, 100s] => 15s
        let (pos, dur) =
            compute_timeline_seconds(0, 100 * sec, 10 * sec, 1_000 * sec, 1_005 * sec, true, 1.0);
        assert!((pos - 15.0).abs() < 0.01);
        assert!((dur - 100.0).abs() < 0.01);

        // 2x rate clamps at duration 20s instead of overflowing to 30s
        let (pos, _) =
            compute_timeline_seconds(0, 20 * sec, 10 * sec, 1_000 * sec, 1_010 * sec, true, 2.0);
        assert!((pos - 20.0).abs() < 0.01);

        // Paused session does not advance
        let (pos, _) =
            compute_timeline_seconds(0, 100 * sec, 10 * sec, 1_000 * sec, 1_010 * sec, false, 1.0);
        assert!((pos - 10.0).abs() < 0.01);
    }
}
