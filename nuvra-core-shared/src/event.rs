// nuvra-core-shared/src/event.rs
// OS event and module state snapshot types — D-ARCH-01/02/03.
// R-13: No unsafe.
#![forbid(unsafe_code)]

/// Events produced by OS subsystems and dispatched to modules.
/// D-ARCH-01: flows from OS → module via bounded flume channels.
#[derive(Debug, Clone)]
pub enum OsEvent {
    /// Media session changed (GSMTC).
    MediaSessionChanged,
    /// Clipboard content changed.
    ClipboardChanged,
    /// Default capture device mute state changed.
    MicMuteChanged { muted: bool },
    /// Foreground window changed (for fullscreen detection, call detection).
    ForegroundWindowChanged { process_id: u32 },
    /// Display configuration changed (DPI, monitor add/remove).
    DisplayChanged,
    /// System going to sleep.
    SystemSuspend,
    /// System resumed from sleep.
    SystemResume,
    /// Hot key registered via RegisterHotKey was pressed.
    HotKey { id: u32 },
    /// A registered timer tick (D-ARCH-04 — periodic, stops when hidden).
    TimerTick { timer_id: u32 },
}

/// Media module state snapshot — rendered by the notch.
#[derive(Debug, Clone, Default)]
pub struct MediaSnapshot {
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub has_art: bool,
    // Art is handled via GPU texture cache (D-DAT-05); only a cache key here.
    pub art_key: Option<u64>,
    pub position_s: f32,
    pub duration_s: f32,
    pub volume_norm: f32,
}

/// Mic module state snapshot.
#[derive(Debug, Clone, Default)]
pub struct MicSnapshot {
    pub is_muted: bool,
    pub call_active: bool,
    pub app_name: Option<String>,
}

/// Notification snapshot (current queued banner, if any).
#[derive(Debug, Clone, Default)]
pub struct NotificationSnapshot {
    pub app_name: String,
    pub summary: String,
    // Content is never logged (R-20). Only a display-ready summary is kept.
    pub queued: u8,
}

/// Clipboard snapshot (search results / pinned count, not content).
#[derive(Debug, Clone, Default)]
pub struct ClipboardSnapshot {
    pub item_count: usize,
    pub pinned_count: usize,
}

/// Shelf snapshot.
#[derive(Debug, Clone, Default)]
pub struct ShelfSnapshot {
    pub item_count: usize,
}
