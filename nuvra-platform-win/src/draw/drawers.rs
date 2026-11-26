// nuvra-platform-win/src/draw/drawers.rs
// Adaptive asymmetric workspace (Media stage appears ONLY when playing) & focused drawers.
// R-13: Unsafe code confined here with // SAFETY: comments.
// R-16: Function line count <= 60, file line count <= 500.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::{D2D_POINT_2F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{ID2D1RenderTarget, D2D1_ELLIPSE, D2D1_ROUNDED_RECT};

use super::drawer_views::{self, draw_pill_btn, in_rect, DrawerBox};
use super::glass::{draw_glass_card, draw_recessed_well};
use super::icons::draw_icon;
use super::text::draw_text;
use super::{DrawBrushes, RenderViewModel, RendererAction, TextFormats};
use nuvra_render::icons::IconKind;
use nuvra_render::layout::Layout;

pub fn draw_drawer(rt: &ID2D1RenderTarget, brushes: &DrawBrushes, formats: &TextFormats, layout: &Layout, drawer_idx: Option<usize>, vm: &RenderViewModel<'_>) {
    let s = layout.scale;
    if layout.body.bottom - 10.0 * s <= 52.0 * s + 24.0 * s {
        return;
    }
    match drawer_idx {
        None => draw_workspace(rt, brushes, formats, layout, vm),
        Some(idx) => draw_focused_drawer(rt, brushes, formats, layout, idx, vm),
    }
}

fn draw_media_stage(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, r: D2D_RECT_F, vm: &RenderViewModel<'_>, s: f32) {
    draw_glass_card(rt, r, 12.0 * s, in_rect(vm.hover_pos, &r), vm.is_dark_theme, s);
    let art = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: r.left + 14.0 * s, top: r.top + 14.0 * s, right: r.left + 60.0 * s, bottom: r.top + 60.0 * s }, radiusX: 10.0 * s, radiusY: 10.0 * s };
    let has_fft = vm.fft_bands.iter().any(|&v| v > 0.02);
    unsafe {
        rt.FillRoundedRectangle(&art, &b.plate_rest);
        rt.DrawRoundedRectangle(&art, &b.ok, 0.9 * s, None);
        for i in 0..5 {
            let amp = if has_fft {
                0.22 + 0.78 * vm.fft_bands[i].clamp(0.0, 1.0)
            } else if vm.media_playing {
                0.30 + 0.70 * ((vm.eq_phase * 5.4 + i as f32 * 1.15).sin().abs())
            } else {
                0.25
            };
            let bh = 22.0 * s * amp;
            let bx = r.left + 23.0 * s + (i as f32) * 6.0 * s;
            let cy = r.top + 37.0 * s;
            let bar = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: bx, top: cy - bh / 2.0, right: bx + 3.0 * s, bottom: cy + bh / 2.0 }, radiusX: 1.5 * s, radiusY: 1.5 * s };
            rt.FillRoundedRectangle(&bar, if i == 2 { &b.ok } else { &b.text_primary });
        }
    }

    draw_text(rt, vm.track_title, &f.title, &b.text_primary, &D2D_RECT_F { left: r.left + 72.0 * s, top: r.top + 14.0 * s, right: r.right - 14.0 * s, bottom: r.top + 34.0 * s });
    draw_text(rt, vm.track_artist, &f.body, &b.text_secondary, &D2D_RECT_F { left: r.left + 72.0 * s, top: r.top + 35.0 * s, right: r.right - 14.0 * s, bottom: r.top + 54.0 * s });

    let tw = (r.right - r.left - 28.0 * s).max(20.0);
    let frac = vm.media_progress.clamp(0.04, 1.0);
    let tr = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: r.left + 14.0 * s, top: r.top + 72.0 * s, right: r.right - 14.0 * s, bottom: r.top + 76.0 * s }, radiusX: 2.0 * s, radiusY: 2.0 * s };
    let pr = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: r.left + 14.0 * s, top: r.top + 72.0 * s, right: r.left + 14.0 * s + tw * frac, bottom: r.top + 76.0 * s }, radiusX: 2.0 * s, radiusY: 2.0 * s };
    let tip = D2D1_ELLIPSE { point: D2D_POINT_2F { x: r.left + 14.0 * s + tw * frac, y: r.top + 74.0 * s }, radiusX: 4.0 * s, radiusY: 4.0 * s };
    // SAFETY: D2D render target drawing media progress bar and scrub indicator.
    unsafe {
        rt.FillRoundedRectangle(&tr, &b.plate_rest);
        rt.FillRoundedRectangle(&pr, &b.ok);
        rt.FillEllipse(&tip, &b.text_primary);
    }
    draw_text(rt, vm.media_time_label, &f.caption, &b.text_tertiary, &D2D_RECT_F { left: r.left + 14.0 * s, top: r.top + 81.0 * s, right: r.right - 14.0 * s, bottom: r.top + 97.0 * s });

    let by = r.bottom - 38.0 * s;
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r.left + 14.0 * s, top: by, right: r.left + 96.0 * s, bottom: by + 26.0 * s }, "Prev", false, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r.left + 104.0 * s, top: by, right: r.left + 216.0 * s, bottom: by + 26.0 * s }, if vm.media_playing { "Pause" } else { "Play" }, vm.media_playing, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r.left + 224.0 * s, top: by, right: r.right - 14.0 * s, bottom: by + 26.0 * s }, "Next", false, vm.hover_pos, s);
}

fn draw_idle_hub_stage(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, r: D2D_RECT_F, vm: &RenderViewModel<'_>, s: f32) {
    draw_glass_card(rt, r, 12.0 * s, in_rect(vm.hover_pos, &r), vm.is_dark_theme, s);
    draw_text(rt, "System Hub", &f.title, &b.text_primary, &D2D_RECT_F { left: r.left + 16.0 * s, top: r.top + 12.0 * s, right: r.right - 128.0 * s, bottom: r.top + 32.0 * s });
    draw_text(rt, "No active audio or video", &f.caption, &b.text_secondary, &D2D_RECT_F { left: r.left + 16.0 * s, top: r.top + 32.0 * s, right: r.right - 128.0 * s, bottom: r.top + 48.0 * s });

    let pill = D2D_RECT_F { left: r.right - 120.0 * s, top: r.top + 14.0 * s, right: r.right - 14.0 * s, bottom: r.top + 38.0 * s };
    draw_recessed_well(rt, pill, 10.0 * s, false, vm.is_dark_theme, s);
    // SAFETY: D2D render target drawing status dot inside idle stage status pill.
    unsafe {
        let dot = D2D1_ELLIPSE { point: D2D_POINT_2F { x: pill.left + 13.0 * s, y: (pill.top + pill.bottom) / 2.0 }, radiusX: 3.6 * s, radiusY: 3.6 * s };
        rt.FillEllipse(&dot, if vm.mic_active { &b.ok } else { &b.danger });
    }
    let live_lbl = if vm.mic_active { "Audio Ready" } else { "Mic Muted" };
    draw_text(rt, live_lbl, &f.caption, &b.text_secondary, &D2D_RECT_F { left: pill.left + 22.0 * s, top: pill.top, right: pill.right - 6.0 * s, bottom: pill.bottom });

    let bat_lbl = if vm.battery_charging { format!("Power  •  {}% Charging", vm.battery_pct) } else { format!("Battery  •  {}% Remaining", vm.battery_pct) };
    draw_text(rt, &bat_lbl, &f.caption, &b.text_secondary, &D2D_RECT_F { left: r.left + 16.0 * s, top: r.top + 61.0 * s, right: r.right - 16.0 * s, bottom: r.top + 77.0 * s });
    let bw = (r.right - r.left - 32.0 * s).max(20.0);
    let b_fill = bw * (vm.battery_pct as f32 / 100.0).clamp(0.05, 1.0);
    // SAFETY: D2D render target drawing battery percentage gauge track & fill.
    unsafe {
        let tr = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: r.left + 16.0 * s, top: r.top + 81.0 * s, right: r.right - 16.0 * s, bottom: r.top + 86.0 * s }, radiusX: 2.5 * s, radiusY: 2.5 * s };
        let fr = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: r.left + 16.0 * s, top: r.top + 81.0 * s, right: r.left + 16.0 * s + b_fill, bottom: r.top + 86.0 * s }, radiusX: 2.5 * s, radiusY: 2.5 * s };
        rt.FillRoundedRectangle(&tr, &b.plate_rest);
        rt.FillRoundedRectangle(&fr, if vm.battery_pct <= 20 && !vm.battery_charging { &b.danger } else { &b.ok });
    }

    let by = r.bottom - 38.0 * s;
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r.left + 14.0 * s, top: by, right: r.left + 44.0 * s, bottom: by + 26.0 * s }, "-", false, vm.hover_pos, s);
    let vbar = D2D_RECT_F { left: r.left + 50.0 * s, top: by, right: r.right - 50.0 * s, bottom: by + 26.0 * s };
    draw_recessed_well(rt, vbar, 8.0 * s, in_rect(vm.hover_pos, &vbar), vm.is_dark_theme, s);
    draw_text(rt, "Vol", &f.caption, &b.text_secondary, &D2D_RECT_F { left: vbar.left + 10.0 * s, top: vbar.top, right: vbar.left + 38.0 * s, bottom: vbar.bottom });
    let (tx0, tx1, ty) = (vbar.left + 40.0 * s, vbar.right - 46.0 * s, (vbar.top + vbar.bottom) / 2.0);
    let tw = (tx1 - tx0).max(20.0);
    let t_fill = tw * (vm.volume_pct as f32 / 100.0).clamp(0.0, 1.0);
    // SAFETY: D2D render target drawing clean interactive volume track and thumb dot.
    unsafe {
        let track = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: tx0, top: ty - 2.0 * s, right: tx1, bottom: ty + 2.0 * s }, radiusX: 2.0 * s, radiusY: 2.0 * s };
        rt.FillRoundedRectangle(&track, &b.plate_rest);
        if t_fill > 2.0 * s {
            let fill = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: tx0, top: ty - 2.0 * s, right: tx0 + t_fill, bottom: ty + 2.0 * s }, radiusX: 2.0 * s, radiusY: 2.0 * s };
            rt.FillRoundedRectangle(&fill, &b.ok);
        }
        let thumb = D2D1_ELLIPSE { point: D2D_POINT_2F { x: tx0 + t_fill, y: ty }, radiusX: 4.5 * s, radiusY: 4.5 * s };
        rt.FillEllipse(&thumb, &b.text_primary);
    }
    draw_text(rt, &format!("{}%", vm.volume_pct), &f.clock, &b.text_primary, &D2D_RECT_F { left: vbar.right - 44.0 * s, top: vbar.top, right: vbar.right - 9.0 * s, bottom: vbar.bottom });
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r.right - 44.0 * s, top: by, right: r.right - 14.0 * s, bottom: by + 26.0 * s }, "+", false, vm.hover_pos, s);
}

fn draw_utility_zone(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, r: D2D_RECT_F, vm: &RenderViewModel<'_>, s: f32) {
    let (rx, top, right, bot) = (r.left, r.top, r.right, r.bottom);
    let drop_w = 164.0 * s;
    let feed_right = right - drop_w - 10.0 * s;
    let mic_w = 150.0 * s;

    // 1. Top control strip: Live Mic toggle + Clipboard Vault button (zero text overflow)
    let mic_r = D2D_RECT_F { left: rx, top, right: rx + mic_w, bottom: top + 34.0 * s };
    draw_glass_card(rt, mic_r, 9.0 * s, in_rect(vm.hover_pos, &mic_r), vm.is_dark_theme, s);
    // SAFETY: D2D render target drawing mic status dot.
    unsafe {
        let dot = D2D1_ELLIPSE { point: D2D_POINT_2F { x: mic_r.left + 15.0 * s, y: (mic_r.top + mic_r.bottom) / 2.0 }, radiusX: 4.0 * s, radiusY: 4.0 * s };
        rt.FillEllipse(&dot, if vm.mic_active { &b.ok } else { &b.danger });
    }
    let mic_lbl = if !vm.mic_active {
        "Mic Muted  •  Live"
    } else if vm.call_active {
        "Call Active  •  Mute"
    } else if vm.camera_active {
        "Cam Live  •  Mute"
    } else {
        "Mic Open  •  Mute"
    };
    draw_text(rt, mic_lbl, &f.caption, &b.text_primary, &D2D_RECT_F { left: mic_r.left + 26.0 * s, top: mic_r.top, right: mic_r.right - 8.0 * s, bottom: mic_r.bottom });

    let stat_r = D2D_RECT_F { left: rx + mic_w + 10.0 * s, top, right: feed_right, bottom: top + 34.0 * s };
    let stat_buf;
    let stat_lbl = if vm.copied_flash {
        "Copied ✓"
    } else {
        stat_buf = format!("Vault ({})  •  Open", vm.clipboard_items.len());
        &stat_buf
    };
    draw_pill_btn(rt, b, f, stat_r, stat_lbl, false, vm.hover_pos, s);

    // 2. Clipboard Feed (2 direct-click snippets)
    let feed_r = D2D_RECT_F { left: rx, top: top + 42.0 * s, right: feed_right, bottom: bot };
    draw_glass_card(rt, feed_r, 11.0 * s, false, vm.is_dark_theme, s);
    for idx in 0..2 {
        let ry = feed_r.top + 10.0 * s + (idx as f32) * 44.0 * s;
        let row = D2D_RECT_F { left: feed_r.left + 10.0 * s, top: ry, right: feed_r.right - 10.0 * s, bottom: ry + 36.0 * s };
        draw_recessed_well(rt, row, 8.0 * s, in_rect(vm.hover_pos, &row), vm.is_dark_theme, s);
        let label = vm.clipboard_items.get(idx).map(|c| {
            let p: String = c.chars().take(34).collect();
            format!("{}  {}", idx + 1, p.replace('\n', " "))
        }).unwrap_or_else(|| if idx == 0 { "Copy text anywhere in Windows to pin here".into() } else { "Encrypted in-memory snippet slot".into() });
        let brush = if vm.clipboard_items.get(idx).is_some() { &b.text_primary } else { &b.text_tertiary };
        draw_text(rt, &label, &f.caption, brush, &D2D_RECT_F { left: row.left + 10.0 * s, top: row.top, right: row.right - 10.0 * s, bottom: row.bottom });
    }

    // 3. Recessed File Drop / Drag-Out Pocket (centered icon + centered labels)
    let drop_r = D2D_RECT_F { left: feed_right + 10.0 * s, top: top + 42.0 * s, right, bottom: bot };
    draw_recessed_well(rt, drop_r, 11.0 * s, in_rect(vm.hover_pos, &drop_r), vm.is_dark_theme, s);
    let drop_cy = (drop_r.top + drop_r.bottom) / 2.0;
    draw_icon(rt, IconKind::Shelf, (drop_r.left + drop_r.right) / 2.0, drop_cy - 18.0 * s, &b.text_secondary, s);
    let drop_title = vm.shelf_items.first().map(|n| n.chars().take(18).collect::<String>()).unwrap_or_else(|| "Drop Files".into());
    let drop_sub = if vm.shelf_items.is_empty() { "Stash onto notch".into() } else { format!("Drag out  •  {} staged", vm.shelf_items.len()) };
    draw_text(rt, &drop_title, &f.caption_center, &b.text_primary, &D2D_RECT_F { left: drop_r.left + 8.0 * s, top: drop_cy - 2.0 * s, right: drop_r.right - 8.0 * s, bottom: drop_cy + 16.0 * s });
    draw_text(rt, &drop_sub, &f.caption_center, &b.text_tertiary, &D2D_RECT_F { left: drop_r.left + 8.0 * s, top: drop_cy + 16.0 * s, right: drop_r.right - 8.0 * s, bottom: drop_cy + 34.0 * s });
}

fn draw_workspace(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, layout: &Layout, vm: &RenderViewModel<'_>) {
    let s = layout.scale;
    let top = 54.0 * s;
    let bot = layout.body.bottom - 18.0 * s;
    let left_r = D2D_RECT_F { left: 16.0 * s, top, right: 344.0 * s, bottom: bot };
    if vm.media_active {
        draw_media_stage(rt, b, f, left_r, vm, s);
    } else {
        draw_idle_hub_stage(rt, b, f, left_r, vm, s);
    }
    draw_utility_zone(rt, b, f, D2D_RECT_F { left: 356.0 * s, top, right: layout.body.right - 16.0 * s, bottom: bot }, vm, s);

    let cx = (layout.body.left + layout.body.right) / 2.0;
    let hov = in_rect(vm.hover_pos, &D2D_RECT_F { left: cx - 32.0 * s, top: layout.body.bottom - 16.0 * s, right: cx + 32.0 * s, bottom: layout.body.bottom });
    let pw = if hov { 48.0 } else { 36.0 } * s;
    let pill = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: cx - pw / 2.0, top: layout.body.bottom - 11.0 * s, right: cx + pw / 2.0, bottom: layout.body.bottom - 7.5 * s }, radiusX: 2.0 * s, radiusY: 2.0 * s };
    // SAFETY: D2D render target drawing bottom collapse grip pill.
    unsafe { rt.FillRoundedRectangle(&pill, if hov { &b.text_secondary } else { &b.divider }); }
}

pub fn hit_test_workspace(layout: &Layout, x: f32, y: f32, vm: &RenderViewModel<'_>) -> Option<RendererAction> {
    let s = layout.scale;
    let pt = Some((x, y));
    let top = 54.0 * s;
    let bot = layout.body.bottom - 18.0 * s;
    let cx = (layout.body.left + layout.body.right) / 2.0;
    if in_rect(pt, &D2D_RECT_F { left: cx - 36.0 * s, top: layout.body.bottom - 18.0 * s, right: cx + 36.0 * s, bottom: layout.body.bottom }) {
        return Some(RendererAction::CollapseNotch);
    }
    let by = bot - 38.0 * s;
    if vm.media_active {
        if in_rect(pt, &D2D_RECT_F { left: 30.0 * s, top: by, right: 112.0 * s, bottom: by + 26.0 * s }) { return Some(RendererAction::MediaPrev); }
        if in_rect(pt, &D2D_RECT_F { left: 120.0 * s, top: by, right: 232.0 * s, bottom: by + 26.0 * s }) { return Some(RendererAction::MediaPlayPause); }
        if in_rect(pt, &D2D_RECT_F { left: 240.0 * s, top: by, right: 330.0 * s, bottom: by + 26.0 * s }) { return Some(RendererAction::MediaNext); }
        let seek_r = D2D_RECT_F { left: 30.0 * s, top: top + 64.0 * s, right: 330.0 * s, bottom: top + 84.0 * s };
        if in_rect(pt, &seek_r) {
            let pct = (((x - seek_r.left) / (seek_r.right - seek_r.left)).clamp(0.0, 1.0) * 100.0).round() as u32;
            return Some(RendererAction::MediaSeekPct(pct));
        }
        if in_rect(pt, &D2D_RECT_F { left: 16.0 * s, top, right: 344.0 * s, bottom: by - 4.0 * s }) { return Some(RendererAction::ToggleDrawer(0)); }
    } else {
        if in_rect(pt, &D2D_RECT_F { left: 30.0 * s, top: by, right: 60.0 * s, bottom: by + 26.0 * s }) { return Some(RendererAction::VolumeDown); }
        if in_rect(pt, &D2D_RECT_F { left: 300.0 * s, top: by, right: 330.0 * s, bottom: by + 26.0 * s }) { return Some(RendererAction::VolumeUp); }
        let vbar = D2D_RECT_F { left: 66.0 * s, top: by, right: 294.0 * s, bottom: by + 26.0 * s };
        if in_rect(pt, &vbar) {
            let (tx0, tx1) = (vbar.left + 40.0 * s, vbar.right - 46.0 * s);
            let pct = (((x - tx0) / (tx1 - tx0)).clamp(0.0, 1.0) * 100.0).round() as u32;
            return Some(RendererAction::SetVolume(pct));
        }
    }

    let rx = 356.0 * s;
    let right = layout.body.right - 16.0 * s;
    let feed_right = right - 164.0 * s - 10.0 * s;
    let mic_w = 150.0 * s;
    if in_rect(pt, &D2D_RECT_F { left: rx, top, right: rx + mic_w, bottom: top + 34.0 * s }) { return Some(RendererAction::ToggleMute); }
    if in_rect(pt, &D2D_RECT_F { left: rx + mic_w + 10.0 * s, top, right: feed_right, bottom: top + 34.0 * s }) { return Some(RendererAction::ToggleDrawer(3)); }
    for idx in 0..vm.clipboard_items.len().min(2) {
        let ry = top + 52.0 * s + (idx as f32) * 44.0 * s;
        if in_rect(pt, &D2D_RECT_F { left: rx + 10.0 * s, top: ry, right: feed_right - 10.0 * s, bottom: ry + 36.0 * s }) { return Some(RendererAction::CopyClipItem(idx)); }
    }
    if in_rect(pt, &D2D_RECT_F { left: feed_right + 10.0 * s, top: top + 42.0 * s, right, bottom: bot }) { return Some(RendererAction::ToggleDrawer(2)); }
    None
}

pub fn hit_test_shelf_drag(layout: &Layout, active_drawer: Option<usize>, x: f32, y: f32, shelf_len: usize) -> Option<usize> {
    let s = layout.scale;
    if shelf_len == 0 || layout.body.bottom - 10.0 * s <= 76.0 * s {
        return None;
    }
    let pt = Some((x, y));
    match active_drawer {
        None => {
            let right = layout.body.right - 16.0 * s;
            let feed_right = right - 164.0 * s - 10.0 * s;
            let drop_r = D2D_RECT_F { left: feed_right + 10.0 * s, top: 54.0 * s + 42.0 * s, right, bottom: layout.body.bottom - 18.0 * s };
            if in_rect(pt, &drop_r) { Some(0) } else { None }
        }
        Some(2) => {
            let (l, t, r) = (16.0 * s, 54.0 * s, layout.body.right - 16.0 * s);
            (0..shelf_len.min(2)).find(|&idx| {
                let ry = t + 40.0 * s + (idx as f32) * 32.0 * s;
                in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: ry, right: r - 232.0 * s, bottom: ry + 26.0 * s })
            })
        }
        _ => None,
    }
}

fn draw_focused_drawer(rt: &ID2D1RenderTarget, brushes: &DrawBrushes, formats: &TextFormats, layout: &Layout, drawer_idx: usize, vm: &RenderViewModel<'_>) {
    let s = layout.scale;
    let d_top = 54.0 * s;
    let d_bottom = layout.body.bottom - 14.0 * s;
    let d_left = 16.0 * s;
    let d_right = layout.body.right - 16.0 * s;
    draw_glass_card(rt, D2D_RECT_F { left: d_left, top: d_top, right: d_right, bottom: d_bottom }, 12.0 * s, false, vm.is_dark_theme, s);

    let bx = DrawerBox { l: d_left, t: d_top, r: d_right, s };
    match drawer_idx {
        0 => drawer_views::draw_media(rt, brushes, formats, bx, vm),
        1 => drawer_views::draw_search(rt, brushes, formats, bx, vm),
        2 => drawer_views::draw_shelf(rt, brushes, formats, bx, vm),
        3 => drawer_views::draw_clipboard(rt, brushes, formats, bx, vm),
        4 => drawer_views::draw_notifications(rt, brushes, formats, bx, vm),
        _ => drawer_views::draw_settings(rt, brushes, formats, bx, vm),
    }
}
