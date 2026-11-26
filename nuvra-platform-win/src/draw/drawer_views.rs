// nuvra-platform-win/src/draw/drawer_views.rs
// Interactive drawer views (Media + FFT/Seek, Search, Shelf + Actions, Clipboard + Pin, Notifs, Settings).
// R-13: Unsafe code confined here with // SAFETY: comments.
// R-16: Clean modular helpers, <= 60 lines per function, <= 500 lines per file.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows::Win32::Graphics::Direct2D::{ID2D1RenderTarget, D2D1_ROUNDED_RECT};

use super::text::draw_text;
use super::{DrawBrushes, RenderViewModel, RendererAction, TextFormats};
use nuvra_render::engine::GlassStyle;

#[derive(Clone, Copy)]
pub struct DrawerBox {
    pub l: f32,
    pub t: f32,
    pub r: f32,
    pub s: f32,
}

pub fn in_rect(hover: Option<(f32, f32)>, r: &D2D_RECT_F) -> bool {
    let Some((x, y)) = hover else { return false };
    x >= r.left && x <= r.right && y >= r.top && y <= r.bottom
}

#[allow(clippy::too_many_arguments)]
pub fn draw_pill_btn(
    rt: &ID2D1RenderTarget,
    brushes: &DrawBrushes,
    formats: &TextFormats,
    rect: D2D_RECT_F,
    label: &str,
    active: bool,
    hover: Option<(f32, f32)>,
    scale: f32,
) {
    let hov = in_rect(hover, &rect);
    let ins = 0.5 * scale;
    let rrect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: rect.left + ins, top: rect.top + ins, right: rect.right - ins, bottom: rect.bottom - ins },
        radiusX: (8.0 * scale - ins).max(1.0),
        radiusY: (8.0 * scale - ins).max(1.0),
    };
    // SAFETY: Valid D2D render target drawing inset rounded glass pill button and clipped text.
    unsafe {
        let fill = if active { &brushes.plate_active } else if hov { &brushes.plate_hover } else { &brushes.plate_rest };
        rt.FillRoundedRectangle(&rrect, fill);
        let rim = if active || hov { &brushes.edge } else { &brushes.divider };
        rt.DrawRoundedRectangle(&rrect, rim, if hov || active { 0.9 } else { 0.7 } * scale, None);
    }
    draw_text(rt, label, &formats.caption_center, &brushes.text_primary, &D2D_RECT_F {
        left: rect.left + 6.0 * scale,
        top: rect.top,
        right: rect.right - 6.0 * scale,
        bottom: rect.bottom,
    });
}

fn draw_header(rt: &ID2D1RenderTarget, brushes: &DrawBrushes, formats: &TextFormats, bx: DrawerBox, title: &str, hover: Option<(f32, f32)>) {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    draw_text(rt, title, &formats.title, &brushes.text_primary, &D2D_RECT_F {
        left: l + 18.0 * s, top: t + 10.0 * s, right: r - 88.0 * s, bottom: t + 30.0 * s,
    });
    let close_r = D2D_RECT_F { left: r - 76.0 * s, top: t + 10.0 * s, right: r - 14.0 * s, bottom: t + 32.0 * s };
    draw_pill_btn(rt, brushes, formats, close_r, "Close", false, hover, s);
}

pub fn draw_media(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, bx: DrawerBox, vm: &RenderViewModel<'_>) {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    if !vm.media_active {
        draw_header(rt, b, f, bx, "MEDIA  •  Nothing playing", vm.hover_pos);
        draw_text(rt, "Play audio or video in Windows or a browser to control and visualize here.", &f.body, &b.text_secondary, &D2D_RECT_F {
            left: l + 18.0 * s, top: t + 46.0 * s, right: r - 18.0 * s, bottom: t + 84.0 * s,
        });
        return;
    }
    draw_header(rt, b, f, bx, &format!("NOW PLAYING  •  {} — {}", vm.track_title, vm.track_artist), vm.hover_pos);
    let track_w = (r - l - 140.0 * s).max(40.0);
    let track_r = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: l + 18.0 * s, top: t + 44.0 * s, right: l + 18.0 * s + track_w, bottom: t + 49.0 * s }, radiusX: 2.5 * s, radiusY: 2.5 * s };
    let prog_r = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: l + 18.0 * s, top: t + 44.0 * s, right: l + 18.0 * s + track_w * vm.media_progress.clamp(0.03, 1.0), bottom: t + 49.0 * s }, radiusX: 2.5 * s, radiusY: 2.5 * s };
    // SAFETY: D2D render target drawing clickable media progress bar & 8-band WASAPI FFT spectrum.
    unsafe {
        rt.FillRoundedRectangle(&track_r, &b.plate_rest);
        rt.FillRoundedRectangle(&prog_r, &b.ok);
        for (i, &band) in vm.fft_bands.iter().enumerate() {
            let amp = (band.max(if vm.media_playing { 0.15 } else { 0.08 })).clamp(0.08, 1.0);
            let bx_pos = r - 96.0 * s + (i as f32) * 10.0 * s;
            let bar_h = 22.0 * s * amp;
            let bar = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: bx_pos, top: t + 64.0 * s - bar_h, right: bx_pos + 6.0 * s, bottom: t + 64.0 * s }, radiusX: 2.0 * s, radiusY: 2.0 * s };
            rt.FillRoundedRectangle(&bar, if i % 3 == 0 { &b.ok } else { &b.text_secondary });
        }
    }
    let meta = vm.lyric_line.map(|ly| format!("{}  •  {ly}", vm.media_time_label)).unwrap_or_else(|| format!("{}  •  Click bar to seek", vm.media_time_label));
    draw_text(rt, &meta, &f.caption, &b.text_secondary, &D2D_RECT_F { left: l + 18.0 * s, top: t + 56.0 * s, right: r - 110.0 * s, bottom: t + 74.0 * s });
    let btn_y = t + 86.0 * s;
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: btn_y, right: l + 114.0 * s, bottom: btn_y + 28.0 * s }, "Previous", false, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 124.0 * s, top: btn_y, right: l + 240.0 * s, bottom: btn_y + 28.0 * s }, if vm.media_playing { "Pause" } else { "Play" }, vm.media_playing, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 250.0 * s, top: btn_y, right: l + 346.0 * s, bottom: btn_y + 28.0 * s }, "Next Track", false, vm.hover_pos, s);
}

pub fn draw_search(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, bx: DrawerBox, vm: &RenderViewModel<'_>) {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    draw_header(rt, b, f, bx, "QUICK FIND  •  Click any item to copy", vm.hover_pos);
    let mut y = t + 40.0 * s;
    for (idx, snippet) in vm.clipboard_items.iter().take(2).enumerate() {
        let row = D2D_RECT_F { left: l + 18.0 * s, top: y, right: r - 18.0 * s, bottom: y + 26.0 * s };
        let clean: String = snippet.chars().take(72).collect();
        draw_pill_btn(rt, b, f, row, &format!("Clip {}: {}", idx + 1, clean.replace('\n', " ")), false, vm.hover_pos, s);
        y += 32.0 * s;
    }
    if let Some(file) = vm.shelf_items.first() {
        let row = D2D_RECT_F { left: l + 18.0 * s, top: y, right: r - 18.0 * s, bottom: y + 26.0 * s };
        draw_pill_btn(rt, b, f, row, &format!("Staged File: {file}"), false, vm.hover_pos, s);
    } else if vm.clipboard_items.is_empty() {
        draw_text(rt, "Copy text or drop files onto the notch to index items here.", &f.body, &b.text_secondary, &D2D_RECT_F { left: l + 18.0 * s, top: y, right: r - 18.0 * s, bottom: y + 28.0 * s });
    }
}

pub fn draw_shelf(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, bx: DrawerBox, vm: &RenderViewModel<'_>) {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    let hdr_buf;
    let hdr = if vm.copied_flash {
        "FILES SHELF  •  Copied / updated!"
    } else {
        hdr_buf = format!("FILES SHELF  •  {} staged (drag out or act)", vm.shelf_items.len());
        &hdr_buf
    };
    draw_header(rt, b, f, bx, hdr, vm.hover_pos);
    if vm.shelf_items.is_empty() {
        draw_text(rt, "Drag and drop any file or folder onto the notch to stage it locally.", &f.body, &b.text_secondary, &D2D_RECT_F {
            left: l + 18.0 * s, top: t + 46.0 * s, right: r - 18.0 * s, bottom: t + 84.0 * s,
        });
        return;
    }
    let mut y = t + 40.0 * s;
    for (idx, item) in vm.shelf_items.iter().take(2).enumerate() {
        draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: y, right: r - 232.0 * s, bottom: y + 26.0 * s }, &format!("{item}  •  Drag"), false, vm.hover_pos, s);
        draw_pill_btn(rt, b, f, D2D_RECT_F { left: r - 226.0 * s, top: y, right: r - 172.0 * s, bottom: y + 26.0 * s }, "Path", false, vm.hover_pos, s);
        draw_pill_btn(rt, b, f, D2D_RECT_F { left: r - 166.0 * s, top: y, right: r - 102.0 * s, bottom: y + 26.0 * s }, "Folder", false, vm.hover_pos, s);
        draw_pill_btn(rt, b, f, D2D_RECT_F { left: r - 96.0 * s, top: y, right: r - 18.0 * s, bottom: y + 26.0 * s }, &format!("Remove #{}", idx + 1), false, vm.hover_pos, s);
        y += 32.0 * s;
    }
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: t + 106.0 * s, right: l + 150.0 * s, bottom: t + 132.0 * s }, "Compress ZIP", false, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 158.0 * s, top: t + 106.0 * s, right: l + 264.0 * s, bottom: t + 132.0 * s }, "Clear Shelf", false, vm.hover_pos, s);
}

pub fn draw_clipboard(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, bx: DrawerBox, vm: &RenderViewModel<'_>) {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    let hdr = if vm.copied_flash { "CLIPBOARD VAULT  •  Copied to clipboard!" } else { "CLIPBOARD VAULT  •  Click entry to copy or pin" };
    draw_header(rt, b, f, bx, hdr, vm.hover_pos);
    if vm.clipboard_items.is_empty() {
        draw_text(rt, "Copy any text in Windows and it appears here automatically.", &f.body, &b.text_secondary, &D2D_RECT_F {
            left: l + 18.0 * s, top: t + 46.0 * s, right: r - 18.0 * s, bottom: t + 80.0 * s,
        });
        return;
    }
    let mut y = t + 40.0 * s;
    for (idx, clip) in vm.clipboard_items.iter().take(2).enumerate() {
        let preview: String = clip.chars().take(64).collect();
        draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: y, right: r - 92.0 * s, bottom: y + 26.0 * s }, &format!("{}. {}", idx + 1, preview.replace('\n', " ")), false, vm.hover_pos, s);
        let pinned = vm.clip_pinned.get(idx).copied().unwrap_or(false);
        draw_pill_btn(rt, b, f, D2D_RECT_F { left: r - 84.0 * s, top: y, right: r - 18.0 * s, bottom: y + 26.0 * s }, if pinned { "Pinned" } else { "Pin" }, pinned, vm.hover_pos, s);
        y += 32.0 * s;
    }
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: t + 106.0 * s, right: l + 150.0 * s, bottom: t + 132.0 * s }, "Clear Unpinned", false, vm.hover_pos, s);
}

pub fn draw_notifications(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, bx: DrawerBox, vm: &RenderViewModel<'_>) {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    let dnd_note = if vm.dnd_active { "  •  Focus Assist On" } else { "" };
    draw_header(rt, b, f, bx, &format!("NOTIFICATIONS  •  {} unread{dnd_note}", vm.unread_notifications), vm.hover_pos);
    if vm.notifications.is_empty() {
        draw_text(rt, "All caught up. No pending notifications.", &f.body, &b.text_secondary, &D2D_RECT_F {
            left: l + 18.0 * s, top: t + 46.0 * s, right: r - 18.0 * s, bottom: t + 80.0 * s,
        });
        return;
    }
    let mut y = t + 42.0 * s;
    for (app, summary) in vm.notifications.iter().take(2) {
        draw_text(rt, &format!("• {app}: {summary}"), &f.body, &b.text_primary, &D2D_RECT_F { left: l + 20.0 * s, top: y, right: r - 20.0 * s, bottom: y + 22.0 * s });
        y += 26.0 * s;
    }
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: t + 102.0 * s, right: l + 148.0 * s, bottom: t + 128.0 * s }, "Mark All Read", false, vm.hover_pos, s);
}

pub fn draw_settings(rt: &ID2D1RenderTarget, b: &DrawBrushes, f: &TextFormats, bx: DrawerBox, vm: &RenderViewModel<'_>) {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    draw_header(rt, b, f, bx, "SETTINGS  •  Appearance, Startup & Sync", vm.hover_pos);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: t + 42.0 * s, right: l + 122.0 * s, bottom: t + 70.0 * s }, "Frosted Glass", vm.glass_style == GlassStyle::Frosted, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 130.0 * s, top: t + 42.0 * s, right: l + 226.0 * s, bottom: t + 70.0 * s }, "Clear Glass", vm.glass_style == GlassStyle::Clear, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 234.0 * s, top: t + 42.0 * s, right: l + 324.0 * s, bottom: t + 70.0 * s }, "Matte Shell", vm.glass_style == GlassStyle::Matte, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r - 236.0 * s, top: t + 42.0 * s, right: r - 130.0 * s, bottom: t + 70.0 * s }, "Dark Glass", vm.is_dark_theme, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r - 122.0 * s, top: t + 42.0 * s, right: r - 18.0 * s, bottom: t + 70.0 * s }, "Light Glass", !vm.is_dark_theme, vm.hover_pos, s);

    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 18.0 * s, top: t + 86.0 * s, right: l + 190.0 * s, bottom: t + 114.0 * s }, if vm.startup_enabled { "Launch at Login: On" } else { "Launch at Login: Off" }, vm.startup_enabled, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: l + 200.0 * s, top: t + 86.0 * s, right: l + 382.0 * s, bottom: t + 114.0 * s }, if vm.follow_system_theme { "Follow Windows: On" } else { "Follow Windows: Off" }, vm.follow_system_theme, vm.hover_pos, s);
    draw_pill_btn(rt, b, f, D2D_RECT_F { left: r - 144.0 * s, top: t + 86.0 * s, right: r - 18.0 * s, bottom: t + 114.0 * s }, "Collapse Notch", false, vm.hover_pos, s);
}

fn hit_drawer_media(bx: DrawerBox, pt: Option<(f32, f32)>, x: f32) -> Option<RendererAction> {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    let track_w = (r - l - 140.0 * s).max(40.0);
    let seek_r = D2D_RECT_F { left: l + 18.0 * s, top: t + 38.0 * s, right: l + 18.0 * s + track_w, bottom: t + 56.0 * s };
    if in_rect(pt, &seek_r) {
        let pct = (((x - seek_r.left) / (seek_r.right - seek_r.left)).clamp(0.0, 1.0) * 100.0).round() as u32;
        return Some(RendererAction::MediaSeekPct(pct));
    }
    let by = t + 86.0 * s;
    if in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: by, right: l + 114.0 * s, bottom: by + 28.0 * s }) { return Some(RendererAction::MediaPrev); }
    if in_rect(pt, &D2D_RECT_F { left: l + 124.0 * s, top: by, right: l + 240.0 * s, bottom: by + 28.0 * s }) { return Some(RendererAction::MediaPlayPause); }
    if in_rect(pt, &D2D_RECT_F { left: l + 250.0 * s, top: by, right: l + 346.0 * s, bottom: by + 28.0 * s }) { return Some(RendererAction::MediaNext); }
    None
}

fn hit_drawer_shelf(bx: DrawerBox, pt: Option<(f32, f32)>, len: usize) -> Option<RendererAction> {
    if len == 0 { return None; }
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    let mut ry = t + 40.0 * s;
    for idx in 0..len.min(2) {
        if in_rect(pt, &D2D_RECT_F { left: r - 96.0 * s, top: ry, right: r - 18.0 * s, bottom: ry + 26.0 * s }) { return Some(RendererAction::RemoveShelfItem(idx)); }
        if in_rect(pt, &D2D_RECT_F { left: r - 166.0 * s, top: ry, right: r - 102.0 * s, bottom: ry + 26.0 * s }) { return Some(RendererAction::RevealShelfItem(idx)); }
        if in_rect(pt, &D2D_RECT_F { left: r - 226.0 * s, top: ry, right: r - 172.0 * s, bottom: ry + 26.0 * s }) { return Some(RendererAction::CopyShelfPath(idx)); }
        if in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: ry, right: r - 232.0 * s, bottom: ry + 26.0 * s }) { return Some(RendererAction::CopyShelfItem(idx)); }
        ry += 32.0 * s;
    }
    if in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: t + 106.0 * s, right: l + 150.0 * s, bottom: t + 132.0 * s }) { return Some(RendererAction::ZipShelfItems); }
    if in_rect(pt, &D2D_RECT_F { left: l + 158.0 * s, top: t + 106.0 * s, right: l + 264.0 * s, bottom: t + 132.0 * s }) { return Some(RendererAction::ClearShelf); }
    None
}

fn hit_drawer_clip(bx: DrawerBox, pt: Option<(f32, f32)>, len: usize) -> Option<RendererAction> {
    if len == 0 { return None; }
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    let mut ry = t + 40.0 * s;
    for idx in 0..len.min(2) {
        if in_rect(pt, &D2D_RECT_F { left: r - 84.0 * s, top: ry, right: r - 18.0 * s, bottom: ry + 26.0 * s }) { return Some(RendererAction::ToggleClipPin(idx)); }
        if in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: ry, right: r - 92.0 * s, bottom: ry + 26.0 * s }) { return Some(RendererAction::CopyClipItem(idx)); }
        ry += 32.0 * s;
    }
    if in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: t + 106.0 * s, right: l + 150.0 * s, bottom: t + 132.0 * s }) { return Some(RendererAction::ClearClipboard); }
    None
}

fn hit_drawer_settings(bx: DrawerBox, pt: Option<(f32, f32)>, vm: &RenderViewModel<'_>) -> Option<RendererAction> {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    if in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: t + 42.0 * s, right: l + 122.0 * s, bottom: t + 70.0 * s }) { return Some(RendererAction::CycleGlassStyle(0)); }
    if in_rect(pt, &D2D_RECT_F { left: l + 130.0 * s, top: t + 42.0 * s, right: l + 226.0 * s, bottom: t + 70.0 * s }) { return Some(RendererAction::CycleGlassStyle(1)); }
    if in_rect(pt, &D2D_RECT_F { left: l + 234.0 * s, top: t + 42.0 * s, right: l + 324.0 * s, bottom: t + 70.0 * s }) { return Some(RendererAction::CycleGlassStyle(2)); }
    if in_rect(pt, &D2D_RECT_F { left: r - 236.0 * s, top: t + 42.0 * s, right: r - 130.0 * s, bottom: t + 70.0 * s }) { return Some(RendererAction::ToggleTheme(true)); }
    if in_rect(pt, &D2D_RECT_F { left: r - 122.0 * s, top: t + 42.0 * s, right: r - 18.0 * s, bottom: t + 70.0 * s }) { return Some(RendererAction::ToggleTheme(false)); }
    if in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: t + 86.0 * s, right: l + 190.0 * s, bottom: t + 114.0 * s }) { return Some(RendererAction::ToggleStartup(!vm.startup_enabled)); }
    if in_rect(pt, &D2D_RECT_F { left: l + 200.0 * s, top: t + 86.0 * s, right: l + 382.0 * s, bottom: t + 114.0 * s }) { return Some(RendererAction::ToggleFollowSystemTheme(!vm.follow_system_theme)); }
    if in_rect(pt, &D2D_RECT_F { left: r - 144.0 * s, top: t + 86.0 * s, right: r - 18.0 * s, bottom: t + 114.0 * s }) { return Some(RendererAction::CollapseNotch); }
    None
}

pub fn hit_test_drawer(
    drawer_idx: usize,
    bx: DrawerBox,
    x: f32,
    y: f32,
    vm: &RenderViewModel<'_>,
) -> Option<RendererAction> {
    let (l, t, r, s) = (bx.l, bx.t, bx.r, bx.s);
    let pt = Some((x, y));
    if in_rect(pt, &D2D_RECT_F { left: r - 76.0 * s, top: t + 10.0 * s, right: r - 14.0 * s, bottom: t + 32.0 * s }) {
        return Some(RendererAction::ToggleDrawer(drawer_idx));
    }
    match drawer_idx {
        0 => hit_drawer_media(bx, pt, x),
        1 => (0..vm.clipboard_items.len().min(2)).find_map(|idx| {
            let ry = t + 40.0 * s + (idx as f32) * 32.0 * s;
            in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: ry, right: r - 18.0 * s, bottom: ry + 26.0 * s })
                .then_some(RendererAction::CopyClipItem(idx))
        }),
        2 => hit_drawer_shelf(bx, pt, vm.shelf_items.len()),
        3 => hit_drawer_clip(bx, pt, vm.clipboard_items.len()),
        4 if !vm.notifications.is_empty() && in_rect(pt, &D2D_RECT_F { left: l + 18.0 * s, top: t + 102.0 * s, right: l + 148.0 * s, bottom: t + 128.0 * s }) => Some(RendererAction::ClearNotifications),
        5 => hit_drawer_settings(bx, pt, vm),
        _ => None,
    }
}
