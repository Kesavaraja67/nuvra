// nuvra-app/src/app_state.rs
// Application state, OS polling (GSMTC, WASAPI FFT, Call/Cam, Network, DND, Notifications), and dispatch.
// R-13: Pure safe Rust (crate root enforces #![forbid(unsafe_code)]).

use nuvra_core_shared::i18n::{keys, FluentCatalog};
use nuvra_mod_clipboard::exclusions::is_excluded;
use nuvra_mod_clipboard::history::ClipboardHistoryStore;
use nuvra_mod_media::gsmtc::GsmtcController;
use nuvra_mod_media::lyrics::LyricsCache;
use nuvra_mod_media::visualizer::{AudioVisualizer, NUM_BANDS};
use nuvra_mod_mic::call_detector::CallDetector;
use nuvra_mod_notifications::queue::NotificationQueue;
use nuvra_mod_shelf::store::ShelfStore;
use nuvra_platform_win::audio::{
    get_master_volume, get_microphone_mute, set_master_volume, toggle_microphone_mute,
};
use nuvra_platform_win::audio_loopback::LoopbackCapture;
use nuvra_platform_win::camera::CameraMonitor;
use nuvra_platform_win::clipboard::{
    get_clipboard_sequence, try_read_clipboard_text, write_clipboard_files, write_clipboard_text,
    ClipboardReadResult,
};
use nuvra_platform_win::draw::RendererAction;
use nuvra_platform_win::foreground::{
    get_foreground_window_class, is_dnd_active, list_visible_window_titles,
};
use nuvra_platform_win::network::{query_active_network, NetworkKind};
use nuvra_platform_win::notifications::{NotifPermission, OsNotificationReader};
use nuvra_platform_win::shell_ops::{compress_to_zip, reveal_in_explorer};
use nuvra_platform_win::theme_sync::{
    is_high_contrast, is_startup_enabled, is_system_dark_mode, set_startup_enabled,
};
use nuvra_render::engine::{GlassStyle, NotchEngine, Theme};
use nuvra_render::tokens;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::HWND;

pub(crate) struct AppData {
    pub gsmtc: GsmtcController,
    pub loopback: LoopbackCapture,
    pub visualizer: AudioVisualizer,
    pub lyrics_cache: LyricsCache,
    pub pcm_scratch: Vec<f32>,
    pub fft_bands: [f32; 8],
    pub lyric_line: Option<String>,
    pub media_playing: bool,
    pub track_title: String,
    pub track_artist: String,
    pub media_progress: f32,
    pub media_position_s: f32,
    pub media_duration_s: f32,
    pub media_time_label: String,
    pub has_os_media: bool,
    pub mic_active: bool,
    pub call_active: bool,
    pub camera_active: bool,
    pub call_detector: CallDetector,
    pub camera: Option<CameraMonitor>,
    pub volume_norm: f32,
    pub network: NetworkKind,
    pub dnd_active: bool,
    pub os_notifs: OsNotificationReader,
    pub glass_style: GlassStyle,
    pub is_dark_theme: bool,
    pub follow_system_theme: bool,
    pub startup_enabled: bool,
    pub clip_store: ClipboardHistoryStore,
    pub last_clip_seq: u32,
    pub clip_retry_seq: u32,
    pub clip_retry_count: u8,
    pub owner_hwnd: HWND,
    pub shelf_store: ShelfStore,
    pub zip_tx: flume::Sender<Result<PathBuf, String>>,
    pub zip_rx: flume::Receiver<Result<PathBuf, String>>,
    pub zip_in_flight: bool,
    pub notif_queue: NotificationQueue,
    pub banner_until: Option<Instant>,
    pub copied_flash_until: Option<Instant>,
    pub is_pinned: bool,
    pub last_poll: Instant,
}

impl AppData {
    pub fn new(owner_hwnd: HWND) -> Self {
        let fluent = FluentCatalog::en_us();
        let mut clip_store = ClipboardHistoryStore::new(50);
        let last_clip_seq = get_clipboard_sequence();
        let fg_class = get_foreground_window_class();
        if !fg_class.as_deref().is_some_and(is_excluded) {
            if let ClipboardReadResult::Text(txt) = try_read_clipboard_text() {
                clip_store.add(txt);
            }
        }
        let mut notif_queue = NotificationQueue::new(10);
        let os_notifs = OsNotificationReader::new();
        if os_notifs.permission_status() == NotifPermission::NeedsPackagedIdentityOrPermission {
            notif_queue.push(
                fluent.get(keys::APP_NAME),
                fluent.get(keys::NOTIFICATIONS_DENIED_HELP),
            );
        } else {
            notif_queue.push(
                fluent.get(keys::APP_NAME),
                "Double-tap or Ctrl+Alt+N to expand workspace".into(),
            );
        }
        let (zip_tx, zip_rx) = flume::bounded(4);

        let mut me = Self {
            gsmtc: GsmtcController::new(),
            loopback: LoopbackCapture::new(),
            visualizer: AudioVisualizer::new(),
            lyrics_cache: LyricsCache::new(),
            pcm_scratch: Vec::with_capacity(1024),
            fft_bands: [0.0; 8],
            lyric_line: None,
            media_playing: false,
            track_title: String::new(),
            track_artist: String::new(),
            media_progress: 0.0,
            media_position_s: 0.0,
            media_duration_s: 0.0,
            media_time_label: String::new(),
            has_os_media: false,
            mic_active: !get_microphone_mute().unwrap_or(false),
            call_active: false,
            camera_active: false,
            call_detector: CallDetector::new(),
            camera: CameraMonitor::new(),
            volume_norm: get_master_volume().unwrap_or(0.65).clamp(0.0, 1.0),
            network: query_active_network(),
            dnd_active: is_dnd_active(),
            os_notifs,
            glass_style: GlassStyle::Frosted,
            is_dark_theme: is_system_dark_mode(),
            follow_system_theme: false,
            startup_enabled: is_startup_enabled(),
            clip_store,
            last_clip_seq,
            clip_retry_seq: 0,
            clip_retry_count: 0,
            owner_hwnd,
            shelf_store: ShelfStore::new(),
            zip_tx,
            zip_rx,
            zip_in_flight: false,
            notif_queue,
            banner_until: None,
            copied_flash_until: None,
            is_pinned: false,
            last_poll: Instant::now() - Duration::from_secs(2),
        };
        me.poll_os_state();
        me
    }

    pub fn poll_os_state(&mut self) {
        self.poll_zip_results();
        self.poll_clipboard();
        if self.has_os_media && self.media_playing {
            self.loopback.copy_latest_samples(&mut self.pcm_scratch);
            let bands32 = self.visualizer.process_samples(&self.pcm_scratch);
            for (i, slot) in self.fft_bands.iter_mut().enumerate() {
                let idx = (i * NUM_BANDS / 8).min(NUM_BANDS - 1);
                *slot = bands32[idx];
            }
        }
        if self.last_poll.elapsed().as_millis() < 450 {
            return;
        }
        self.last_poll = Instant::now();
        if let Ok(muted) = get_microphone_mute() {
            self.mic_active = !muted;
        }
        if let Ok(vol) = get_master_volume() {
            self.volume_norm = vol.clamp(0.0, 1.0);
        }
        self.network = query_active_network();
        self.dnd_active = is_dnd_active();
        self.camera_active = self
            .camera
            .as_ref()
            .is_some_and(CameraMonitor::is_camera_active);
        self.call_active = self.mic_active
            && list_visible_window_titles()
                .iter()
                .any(|t| self.call_detector.is_meeting_window_title(t));
        self.poll_notifications();
        self.poll_media_state();
    }

    fn poll_zip_results(&mut self) {
        while let Ok(res) = self.zip_rx.try_recv() {
            self.zip_in_flight = false;
            match res {
                Ok(zip) => {
                    let _ = self.shelf_store.add_path(zip);
                    self.copied_flash_until = Some(Instant::now() + Duration::from_millis(1400));
                }
                Err(err_msg) => {
                    if self.notif_queue.push("Nuvra Shelf".into(), err_msg) {
                        self.banner_until = Some(
                            Instant::now()
                                + Duration::from_millis(tokens::timing::BANNER_DURATION_MS),
                        );
                    }
                }
            }
        }
    }

    fn poll_clipboard(&mut self) {
        let seq = get_clipboard_sequence();
        if seq == self.last_clip_seq {
            return;
        }
        if get_foreground_window_class()
            .as_deref()
            .is_some_and(is_excluded)
        {
            self.last_clip_seq = seq;
            self.clip_retry_count = 0;
            return;
        }
        let outcome = try_read_clipboard_text();
        self.apply_clipboard_outcome(seq, outcome);
    }

    pub(crate) fn apply_clipboard_outcome(&mut self, seq: u32, outcome: ClipboardReadResult) {
        match outcome {
            ClipboardReadResult::Text(txt) => {
                self.last_clip_seq = seq;
                self.clip_retry_count = 0;
                self.clip_store.add(txt);
            }
            ClipboardReadResult::NonText => {
                self.last_clip_seq = seq;
                self.clip_retry_count = 0;
            }
            ClipboardReadResult::OpenFailed => {
                if self.clip_retry_seq == seq {
                    self.clip_retry_count = self.clip_retry_count.saturating_add(1);
                } else {
                    self.clip_retry_seq = seq;
                    self.clip_retry_count = 1;
                }
                if self.clip_retry_count >= 5 {
                    self.last_clip_seq = seq;
                    self.clip_retry_count = 0;
                }
            }
        }
    }

    fn poll_notifications(&mut self) {
        self.notif_queue.set_dnd(self.dnd_active);
        for (app, summary) in self.os_notifs.drain_new() {
            if self.notif_queue.push(app, summary) {
                self.banner_until = Some(
                    Instant::now() + Duration::from_millis(tokens::timing::BANNER_DURATION_MS),
                );
            }
        }
    }

    fn poll_media_state(&mut self) {
        if let Some(snap) = self.gsmtc.query_current_session() {
            if !snap.title.is_empty() {
                self.has_os_media = true;
                self.media_playing = snap.is_playing;
                self.loopback.set_active(snap.is_playing);
                if !snap.is_playing {
                    self.fft_bands = [0.0; 8];
                }
                self.track_title = snap.title;
                self.track_artist = if snap.artist.is_empty() {
                    "Windows Media".into()
                } else {
                    snap.artist
                };
                self.media_position_s = snap.position_s;
                self.media_duration_s = snap.duration_s;
                if snap.duration_s > 0.0 {
                    self.media_progress = (snap.position_s / snap.duration_s).clamp(0.0, 1.0);
                    self.media_time_label = format!(
                        "{}:{:02} / {}:{:02}",
                        snap.position_s as u32 / 60,
                        snap.position_s as u32 % 60,
                        snap.duration_s as u32 / 60,
                        snap.duration_s as u32 % 60
                    );
                } else {
                    self.media_progress = 0.0;
                    self.media_time_label = if snap.is_playing {
                        "Playing".into()
                    } else {
                        "Paused".into()
                    };
                }
                self.lyric_line = self
                    .lyrics_cache
                    .get(&self.track_title, &self.track_artist)
                    .and_then(|l| l.current_line(self.media_position_s))
                    .map(|l| l.text.clone());
                return;
            }
        }
        self.has_os_media = false;
        self.media_playing = false;
        self.loopback.set_active(false);
        self.fft_bands = [0.0; 8];
        self.lyric_line = None;
        self.track_title.clear();
        self.track_artist.clear();
        self.media_progress = 0.0;
        self.media_time_label.clear();
    }

    pub fn sync_system_theme(&mut self, engine: &mut NotchEngine) {
        if is_high_contrast() {
            engine.set_theme(Theme::Signal);
            return;
        }
        if self.follow_system_theme {
            self.is_dark_theme = is_system_dark_mode();
            engine.set_theme(if self.is_dark_theme {
                Theme::Obsidian
            } else {
                Theme::Paper
            });
        }
    }

    fn apply_media_or_audio_action(&mut self, action: RendererAction) {
        match action {
            RendererAction::MediaPlayPause => {
                if self.has_os_media && self.gsmtc.toggle_play_pause() {
                    self.media_playing = !self.media_playing;
                    self.loopback.set_active(self.media_playing);
                }
            }
            RendererAction::MediaNext => {
                if !self.has_os_media || !self.gsmtc.next_track() {
                    self.media_progress = 0.05;
                }
            }
            RendererAction::MediaPrev => {
                if !self.has_os_media || !self.gsmtc.previous_track() {
                    self.media_progress = 0.05;
                }
            }
            RendererAction::MediaSeekPct(pct) => {
                if self.has_os_media && self.media_duration_s > 0.0 {
                    let target = self.media_duration_s * (pct as f32 / 100.0);
                    let _ = self.gsmtc.seek_seconds(target);
                    self.media_progress = (pct as f32 / 100.0).clamp(0.0, 1.0);
                }
            }
            RendererAction::ToggleMute => {
                self.mic_active = toggle_microphone_mute()
                    .map(|muted| !muted)
                    .unwrap_or(self.mic_active);
            }
            RendererAction::VolumeDown
            | RendererAction::VolumeUp
            | RendererAction::SetVolume(_) => {
                let target = match action {
                    RendererAction::VolumeDown => self.volume_norm - 0.05,
                    RendererAction::VolumeUp => self.volume_norm + 0.05,
                    RendererAction::SetVolume(pct) => pct as f32 / 100.0,
                    _ => self.volume_norm,
                }
                .clamp(0.0, 1.0);
                if set_master_volume(target).is_ok() {
                    self.volume_norm = target;
                }
            }
            _ => {}
        }
    }

    fn apply_shelf_or_clip_action(&mut self, action: RendererAction) {
        match action {
            RendererAction::CopyClipItem(idx) => {
                if let Some(entry) = self.clip_store.search("").get(idx) {
                    if write_clipboard_text(self.owner_hwnd, &entry.content) {
                        self.last_clip_seq = get_clipboard_sequence();
                        self.copied_flash_until =
                            Some(Instant::now() + Duration::from_millis(1400));
                    }
                }
            }
            RendererAction::ToggleClipPin(idx) => {
                if let Some((id, pinned)) = self
                    .clip_store
                    .search("")
                    .get(idx)
                    .map(|e| (e.id, e.is_pinned))
                {
                    self.clip_store.set_pinned(id, !pinned);
                }
            }
            RendererAction::ClearClipboard => self.clip_store.wipe_unpinned(),
            RendererAction::CopyShelfItem(idx) => {
                if let Some(item) = self.shelf_store.items().get(idx) {
                    if write_clipboard_files(self.owner_hwnd, std::slice::from_ref(&item.path)) {
                        self.last_clip_seq = get_clipboard_sequence();
                        self.copied_flash_until =
                            Some(Instant::now() + Duration::from_millis(1400));
                    }
                }
            }
            RendererAction::CopyShelfPath(idx) => {
                if let Some(item) = self.shelf_store.items().get(idx) {
                    if write_clipboard_text(self.owner_hwnd, &item.path.to_string_lossy()) {
                        self.last_clip_seq = get_clipboard_sequence();
                        self.copied_flash_until =
                            Some(Instant::now() + Duration::from_millis(1400));
                    }
                }
            }
            RendererAction::RevealShelfItem(idx) => {
                if let Some(item) = self.shelf_store.items().get(idx) {
                    let _ = reveal_in_explorer(&item.path);
                }
            }
            RendererAction::ZipShelfItems => {
                if self.zip_in_flight || self.shelf_store.is_empty() {
                    return;
                }
                let paths: Vec<_> = self
                    .shelf_store
                    .items()
                    .iter()
                    .map(|i| i.path.clone())
                    .collect();
                let tx = self.zip_tx.clone();
                self.zip_in_flight = true;
                if std::thread::Builder::new()
                    .name("nuvra-zip".into())
                    .spawn(move || {
                        let _ = tx.send(compress_to_zip(&paths));
                    })
                    .is_err()
                {
                    self.zip_in_flight = false;
                }
            }
            RendererAction::RemoveShelfItem(idx) => {
                let _ = self.shelf_store.remove_at(idx);
            }
            RendererAction::ClearShelf => self.shelf_store.clear(),
            _ => {}
        }
    }

    pub fn apply_action(&mut self, action: RendererAction, engine: &mut NotchEngine) {
        if !matches!(
            action,
            RendererAction::ToggleMute
                | RendererAction::CollapseNotch
                | RendererAction::DismissBanner
        ) {
            self.is_pinned = true;
        }
        match action {
            RendererAction::ToggleDrawer(d) => engine.toggle_drawer(d),
            RendererAction::MediaPlayPause
            | RendererAction::MediaNext
            | RendererAction::MediaPrev
            | RendererAction::MediaSeekPct(_)
            | RendererAction::ToggleMute
            | RendererAction::VolumeDown
            | RendererAction::VolumeUp
            | RendererAction::SetVolume(_) => self.apply_media_or_audio_action(action),
            RendererAction::CopyClipItem(_)
            | RendererAction::ToggleClipPin(_)
            | RendererAction::ClearClipboard
            | RendererAction::CopyShelfItem(_)
            | RendererAction::CopyShelfPath(_)
            | RendererAction::RevealShelfItem(_)
            | RendererAction::ZipShelfItems
            | RendererAction::RemoveShelfItem(_)
            | RendererAction::ClearShelf => self.apply_shelf_or_clip_action(action),
            RendererAction::CollapseNotch => {
                engine.set_active_drawer(None);
                engine.set_vertical_expanded(false);
                engine.set_expanded(false);
                engine.set_hovered(false);
                self.is_pinned = false;
            }
            RendererAction::DismissBanner => {
                self.banner_until = None;
                engine.set_banner(false);
                self.is_pinned = false;
            }
            RendererAction::OpenBannerNotif => {
                self.banner_until = None;
                engine.set_banner(false);
                engine.set_active_drawer(Some(4));
            }
            RendererAction::ClearNotifications => self.notif_queue.clear(),
            RendererAction::CycleGlassStyle(m) => {
                self.glass_style = match m {
                    1 => GlassStyle::Clear,
                    2 => GlassStyle::Matte,
                    _ => GlassStyle::Frosted,
                };
            }
            RendererAction::ToggleTheme(dark) => {
                self.follow_system_theme = false;
                self.is_dark_theme = dark;
                engine.set_theme(if dark { Theme::Obsidian } else { Theme::Paper });
            }
            RendererAction::ToggleStartup(en) => {
                if set_startup_enabled(en) {
                    self.startup_enabled = en;
                }
            }
            RendererAction::ToggleFollowSystemTheme(follow) => {
                self.follow_system_theme = follow;
                self.sync_system_theme(engine);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::HWND;

    #[test]
    fn test_clipboard_retry_vs_permanent_nontext() {
        let mut data = AppData::new(HWND(std::ptr::null_mut()));
        let seq0 = data.last_clip_seq;
        let next = seq0.wrapping_add(1);

        data.apply_clipboard_outcome(next, ClipboardReadResult::OpenFailed);
        assert_eq!(data.last_clip_seq, seq0);
        assert_eq!(data.clip_retry_count, 1);

        data.apply_clipboard_outcome(next, ClipboardReadResult::Text("hello".into()));
        assert_eq!(data.last_clip_seq, next);
        assert_eq!(data.clip_retry_count, 0);

        let img_seq = next.wrapping_add(1);
        data.apply_clipboard_outcome(img_seq, ClipboardReadResult::NonText);
        assert_eq!(data.last_clip_seq, img_seq);
    }

    #[test]
    fn test_toggle_mute_action_preserves_pin_state() {
        let mut data = AppData::new(HWND(std::ptr::null_mut()));
        let mut engine = NotchEngine::new(Theme::Obsidian);
        assert!(!data.is_pinned);
        data.apply_action(RendererAction::ToggleMute, &mut engine);
        assert!(!data.is_pinned);
    }
}
