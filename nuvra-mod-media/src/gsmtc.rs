// nuvra-mod-media/src/gsmtc.rs
// Windows Runtime GSMTC (Global System Media Transport Controls) integration (Section 14.2).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use nuvra_core_shared::event::MediaSnapshot;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

pub struct GsmtcController {
    manager: Option<GlobalSystemMediaTransportControlsSessionManager>,
}

impl GsmtcController {
    pub fn new() -> Self {
        // Asynchronously request the session manager on the worker thread
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .ok()
            .and_then(|op| op.get().ok());

        Self { manager }
    }

    /// Query the current media session and return a display snapshot.
    pub fn query_current_session(&self) -> Option<MediaSnapshot> {
        let manager = self.manager.as_ref()?;
        let session = manager.GetCurrentSession().ok()?;

        let playback_info = session.GetPlaybackInfo().ok()?;
        let status = playback_info.PlaybackStatus().ok()?;
        let is_playing = status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

        let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
        let title = props
            .Title()
            .ok()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        let artist = props
            .Artist()
            .ok()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();

        let (position_s, duration_s) = if let Ok(timeline) = session.GetTimelineProperties() {
            let pos_ticks = timeline.Position().map(|p| p.Duration).unwrap_or(0);
            let end_ticks = timeline.EndTime().map(|e| e.Duration).unwrap_or(0);
            (
                pos_ticks as f32 / 10_000_000.0,
                end_ticks as f32 / 10_000_000.0,
            )
        } else {
            (0.0, 0.0)
        };

        Some(MediaSnapshot {
            title,
            artist,
            is_playing,
            has_art: false, // Art caching via GPU texture cache (D-DAT-05)
            art_key: None,
            position_s,
            duration_s,
            volume_norm: 1.0,
        })
    }

    /// Toggle Play/Pause on the active session.
    pub fn toggle_play_pause(&self) -> bool {
        if let Some(session) = self.current_session() {
            if let Ok(op) = session.TryTogglePlayPauseAsync() {
                return op.get().unwrap_or(false);
            }
        }
        false
    }

    /// Skip to next track.
    pub fn next_track(&self) -> bool {
        if let Some(session) = self.current_session() {
            if let Ok(op) = session.TrySkipNextAsync() {
                return op.get().unwrap_or(false);
            }
        }
        false
    }

    /// Skip to previous track.
    pub fn previous_track(&self) -> bool {
        if let Some(session) = self.current_session() {
            if let Ok(op) = session.TrySkipPreviousAsync() {
                return op.get().unwrap_or(false);
            }
        }
        false
    }

    fn current_session(&self) -> Option<GlobalSystemMediaTransportControlsSession> {
        self.manager
            .as_ref()
            .and_then(|m| m.GetCurrentSession().ok())
    }
}

impl Default for GsmtcController {
    fn default() -> Self {
        Self::new()
    }
}
