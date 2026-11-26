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

    /// Return the most recent playing GSMTC media/video snapshot without blocking the UI thread.
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
        if self.cmd_tx.send(GsmtcCmd::SeekSeconds(seconds.max(0.0), tx)).is_err() {
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
        let snap = manager.as_ref().and_then(find_playing_snapshot);
        if let Ok(mut guard) = latest.lock() {
            *guard = snap;
        }
        match cmd_rx.recv_timeout(Duration::from_millis(350)) {
            Ok(GsmtcCmd::Stop) | Err(flume::RecvTimeoutError::Disconnected) => break,
            Ok(GsmtcCmd::TogglePlayPause(reply)) => {
                let ok = manager
                    .as_ref()
                    .and_then(pick_control_session)
                    .and_then(|s| s.TryTogglePlayPauseAsync().ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Ok(GsmtcCmd::Next(reply)) => {
                let ok = manager
                    .as_ref()
                    .and_then(pick_control_session)
                    .and_then(|s| s.TrySkipNextAsync().ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Ok(GsmtcCmd::Prev(reply)) => {
                let ok = manager
                    .as_ref()
                    .and_then(pick_control_session)
                    .and_then(|s| s.TrySkipPreviousAsync().ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Ok(GsmtcCmd::SeekSeconds(secs, reply)) => {
                let ticks = (secs.max(0.0) as f64 * 10_000_000.0) as i64;
                let ok = manager
                    .as_ref()
                    .and_then(pick_control_session)
                    .and_then(|s| s.TryChangePlaybackPositionAsync(ticks).ok())
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false);
                let _ = reply.send(ok);
            }
            Err(flume::RecvTimeoutError::Timeout) => {}
        }
    }
}

fn is_session_playing(session: &GlobalSystemMediaTransportControlsSession) -> bool {
    session
        .GetPlaybackInfo()
        .ok()
        .and_then(|i| i.PlaybackStatus().ok())
        == Some(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
}

fn pick_control_session(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
) -> Option<GlobalSystemMediaTransportControlsSession> {
    if let Ok(cur) = manager.GetCurrentSession() {
        if is_session_playing(&cur) {
            return Some(cur);
        }
    }
    if let Ok(list) = manager.GetSessions() {
        let len = list.Size().unwrap_or(0);
        for i in 0..len {
            if let Ok(s) = list.GetAt(i) {
                if is_session_playing(&s) {
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
        let clean = raw.rsplit(['\\', '/']).next().unwrap_or("").trim_end_matches(".exe");
        if clean.is_empty() { "Active Media".into() } else { clean.into() }
    }
}

fn find_playing_snapshot(
    manager: &GlobalSystemMediaTransportControlsSessionManager,
) -> Option<MediaSnapshot> {
    if let Ok(cur) = manager.GetCurrentSession() {
        if is_session_playing(&cur) {
            if let Some(snap) = snapshot_from_session(&cur) {
                return Some(snap);
            }
        }
    }
    if let Ok(list) = manager.GetSessions() {
        let len = list.Size().unwrap_or(0);
        for i in 0..len {
            if let Ok(s) = list.GetAt(i) {
                if is_session_playing(&s) {
                    if let Some(snap) = snapshot_from_session(&s) {
                        return Some(snap);
                    }
                }
            }
        }
    }
    None
}

fn snapshot_from_session(session: &GlobalSystemMediaTransportControlsSession) -> Option<MediaSnapshot> {
    let app = friendly_source_app(session);
    let props = session.TryGetMediaPropertiesAsync().ok().and_then(|op| op.get().ok());
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

    let title = if raw_title.is_empty() { format!("{app} Playing") } else { raw_title };
    let artist = if raw_artist.is_empty() { app } else { raw_artist };

    let (position_s, duration_s) = if let Ok(timeline) = session.GetTimelineProperties() {
        let pos_ticks = timeline.Position().map(|p| p.Duration).unwrap_or(0).max(0);
        let end_ticks = timeline.EndTime().map(|e| e.Duration).unwrap_or(0).max(0);
        (pos_ticks as f32 / 10_000_000.0, end_ticks as f32 / 10_000_000.0)
    } else {
        (0.0, 0.0)
    };

    Some(MediaSnapshot {
        title,
        artist,
        is_playing: true,
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
}
