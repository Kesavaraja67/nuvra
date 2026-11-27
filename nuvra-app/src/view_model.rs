// nuvra-app/src/view_model.rs
// View-model projection and live layout builder for the Nuvra renderer.
// R-13: Pure safe Rust (crate root enforces #![forbid(unsafe_code)]).

use crate::app_state::AppData;
use nuvra_core_shared::module::ModuleState;
use nuvra_platform_win::draw::RenderViewModel;
use nuvra_platform_win::power::BatteryInfo;
use nuvra_render::engine::{FrameDescriptor, NotchEngine};
use nuvra_render::layout::{compute_layout, Layout, LayoutState};
use nuvra_render::spring::NotchState;
use nuvra_render::tokens;
use std::time::Instant;

pub(crate) struct UiSnapshots {
    pub clips: Vec<String>,
    pub clip_pinned: Vec<bool>,
    pub shelf: Vec<String>,
    pub notifs: Vec<(String, String)>,
}

pub(crate) fn snap_strings(data: &AppData) -> UiSnapshots {
    let entries = data.clip_store.search("");
    let clips = entries.iter().take(4).map(|e| e.content.clone()).collect();
    let clip_pinned = entries.iter().take(4).map(|e| e.is_pinned).collect();
    let shelf = data
        .shelf_store
        .items()
        .iter()
        .take(4)
        .map(|i| i.name.clone())
        .collect();
    let notifs = data
        .notif_queue
        .peek()
        .map(|t| vec![(t.app_name.clone(), t.summary.clone())])
        .unwrap_or_default();
    UiSnapshots {
        clips,
        clip_pinned,
        shelf,
        notifs,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn make_vm<'a>(
    data: &'a AppData,
    snaps: &'a UiSnapshots,
    battery: BatteryInfo,
    hover_pos: Option<(f32, f32)>,
    eq_phase: f32,
) -> RenderViewModel<'a> {
    RenderViewModel {
        media_active: data.has_os_media,
        media_playing: data.media_playing,
        track_title: &data.track_title,
        track_artist: &data.track_artist,
        media_progress: data.media_progress,
        media_time_label: &data.media_time_label,
        fft_bands: &data.fft_bands,
        lyric_line: data.lyric_line.as_deref(),
        mic_active: data.mic_active,
        call_active: data.call_active,
        camera_active: data.camera_active,
        dnd_active: data.dnd_active,
        unread_notifications: data.notif_queue.len() as u32,
        battery_pct: battery.percentage,
        battery_charging: battery.is_charging,
        volume_pct: (data.volume_norm * 100.0).round().clamp(0.0, 100.0) as u32,
        network: data.network,
        glass_style: data.glass_style,
        is_dark_theme: data.is_dark_theme,
        startup_enabled: data.startup_enabled,
        follow_system_theme: data.follow_system_theme,
        clipboard_items: &snaps.clips,
        clip_pinned: &snaps.clip_pinned,
        shelf_items: &snaps.shelf,
        notifications: &snaps.notifs,
        hover_pos,
        copied_flash: data
            .copied_flash_until
            .map(|t| Instant::now() < t)
            .unwrap_or(false),
        eq_phase,
    }
}

pub(crate) fn build_live_layout(
    data: &AppData,
    engine: &mut NotchEngine,
    module_state: &ModuleState,
    dt: f32,
) -> (FrameDescriptor, Layout) {
    let frame = engine.update(module_state, dt);
    let lstate = match frame.active_state {
        NotchState::Collapsed => LayoutState::Collapsed,
        NotchState::Expanded => LayoutState::Expanded,
        NotchState::Banner => LayoutState::Banner,
    };
    let media_active = data.has_os_media;
    let mut layout = compute_layout(
        lstate,
        1.0,
        149.0,
        media_active,
        data.notif_queue.len() as u32,
        engine
            .active_drawer()
            .map(|_| tokens::window::DRAWER_MIN_HEIGHT),
    );
    layout.body.right = layout.body.left + frame.width;
    layout.body.bottom = layout.body.top + frame.height;
    (frame, layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuvra_render::engine::Theme;
    use nuvra_render::layout::ItemId;
    use windows::Win32::Foundation::HWND;

    #[test]
    fn test_paused_media_keeps_transport_layout_and_preserves_playing_flag() {
        let mut data = AppData::new(HWND(std::ptr::null_mut()));
        data.has_os_media = true;
        data.media_playing = false;
        data.track_title = "Paused Track".into();
        data.track_artist = "Artist".into();

        let snaps = snap_strings(&data);
        let vm = make_vm(
            &data,
            &snaps,
            BatteryInfo {
                percentage: 80,
                is_charging: false,
                has_battery: true,
            },
            None,
            0.0,
        );
        assert!(vm.media_active);
        assert!(!vm.media_playing);

        let mut engine = NotchEngine::new(Theme::Obsidian);
        engine.set_expanded(true);
        for _ in 0..30 {
            let _ = build_live_layout(&data, &mut engine, &ModuleState::Idle, 0.016);
        }
        let (_, layout) = build_live_layout(&data, &mut engine, &ModuleState::Idle, 0.016);
        assert!(layout.items.iter().any(|i| i.id == ItemId::PlayPause));
    }
}
