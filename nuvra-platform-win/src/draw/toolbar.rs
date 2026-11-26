// nuvra-platform-win/src/draw/toolbar.rs
// Smoked-quartz horizontal toolbar strip (48px peek mode & top header).
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::{D2D_POINT_2F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{ID2D1RenderTarget, D2D1_ELLIPSE, D2D1_ROUNDED_RECT};

use super::collapsed::draw_network_indicator;
use super::icons::draw_icon;
use super::text::{draw_text, get_current_date_time_string};
use super::{DrawBrushes, RenderViewModel, TextFormats};
use nuvra_render::icons::IconKind;
use nuvra_render::layout::{ItemId, Layout, LayoutItem};

fn is_hovered(item: &LayoutItem, hover: Option<(f32, f32)>) -> bool {
    let Some((hx, hy)) = hover else { return false };
    hx >= item.hit_rect.left && hx <= item.hit_rect.right && hy >= item.hit_rect.top && hy <= item.hit_rect.bottom
}

fn draw_album_tile(rt: &ID2D1RenderTarget, brushes: &DrawBrushes, item: &LayoutItem, playing: bool, eq_phase: f32, hovered: bool, scale: f32) {
    let rrect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: item.visual_rect.left, top: item.visual_rect.top, right: item.visual_rect.right, bottom: item.visual_rect.bottom },
        radiusX: 8.0 * scale,
        radiusY: 8.0 * scale,
    };
    let cx = (item.visual_rect.left + item.visual_rect.right) / 2.0;
    let cy = (item.visual_rect.top + item.visual_rect.bottom) / 2.0;
    // SAFETY: D2D render target drawing smoked-glass audio disc & live equalizer bars.
    unsafe {
        rt.FillRoundedRectangle(&rrect, if hovered { &brushes.plate_hover } else { &brushes.plate_rest });
        rt.DrawRoundedRectangle(&rrect, if playing { &brushes.ok } else { &brushes.edge }, 0.9 * scale, None);

        for i in 0..3 {
            let amp = if playing { 0.32 + 0.68 * ((eq_phase * 5.2 + i as f32 * 1.6).sin().abs()) } else { 0.25 };
            let bar_h = 14.0 * scale * amp;
            let bx = cx - 5.2 * scale + (i as f32) * 4.2 * scale;
            let bar = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F { left: bx, top: cy - bar_h / 2.0, right: bx + 2.2 * scale, bottom: cy + bar_h / 2.0 },
                radiusX: 1.1 * scale,
                radiusY: 1.1 * scale,
            };
            rt.FillRoundedRectangle(&bar, if playing && i == 1 { &brushes.ok } else { &brushes.text_primary });
        }
    }
}

fn draw_transport_btn(rt: &ID2D1RenderTarget, brushes: &DrawBrushes, item: &LayoutItem, kind: IconKind, hovered: bool, highlight: bool, scale: f32) {
    let cx = (item.visual_rect.left + item.visual_rect.right) / 2.0;
    let cy = (item.visual_rect.top + item.visual_rect.bottom) / 2.0;
    // SAFETY: D2D circular glass plate for transport button hover/rest.
    unsafe {
        if hovered || highlight {
            let plate = D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx, y: cy }, radiusX: 14.5 * scale, radiusY: 14.5 * scale };
            rt.FillEllipse(&plate, if hovered { &brushes.plate_hover } else { &brushes.plate_rest });
            if hovered {
                rt.DrawEllipse(&plate, &brushes.edge, 0.8 * scale, None);
            }
        }
    }
    draw_icon(rt, kind, cx, cy, &brushes.text_primary, scale);
}

#[allow(clippy::too_many_arguments)]
fn draw_tool_circle(rt: &ID2D1RenderTarget, brushes: &DrawBrushes, item: &LayoutItem, kind: IconKind, active: bool, hovered: bool, alert: bool, scale: f32) {
    let cx = (item.visual_rect.left + item.visual_rect.right) / 2.0;
    let cy = (item.visual_rect.top + item.visual_rect.bottom) / 2.0;
    let plate = D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx, y: cy }, radiusX: 15.5 * scale, radiusY: 15.5 * scale };
    // SAFETY: D2D circular frosted glass button fill + subtle specular rim.
    unsafe {
        let fill = if active { &brushes.plate_active } else if hovered { &brushes.plate_hover } else { &brushes.plate_rest };
        rt.FillEllipse(&plate, fill);
        let rim = if alert { &brushes.danger } else { &brushes.edge };
        if active || hovered || alert {
            rt.DrawEllipse(&plate, rim, 0.9 * scale, None);
        }
    }
    let icon_brush = if alert { &brushes.danger } else { &brushes.text_primary };
    draw_icon(rt, kind, cx, cy, icon_brush, 0.92 * scale);
}

pub fn draw_toolbar(
    rt: &ID2D1RenderTarget,
    brushes: &DrawBrushes,
    formats: &TextFormats,
    layout: &Layout,
    vm: &RenderViewModel<'_>,
    active_drawer: Option<usize>,
) {
    let scale = layout.scale;
    for item in &layout.items {
        let hov = is_hovered(item, vm.hover_pos);
        match item.id {
            ItemId::Logo => {
                let cx = (item.visual_rect.left + item.visual_rect.right) / 2.0;
                let cy = (item.visual_rect.top + item.visual_rect.bottom) / 2.0;
                draw_network_indicator(rt, brushes, cx, cy, vm.network, scale);
            }
            ItemId::AlbumArt => draw_album_tile(rt, brushes, item, vm.media_playing, vm.eq_phase, hov, scale),
            ItemId::TrackText => {
                if vm.media_active {
                    draw_text(rt, vm.track_title, &formats.title, &brushes.text_primary, &D2D_RECT_F {
                        left: item.visual_rect.left, top: item.visual_rect.top, right: item.visual_rect.right, bottom: item.visual_rect.top + 17.0 * scale,
                    });
                    draw_text(rt, vm.track_artist, &formats.caption, &brushes.text_secondary, &D2D_RECT_F {
                        left: item.visual_rect.left, top: item.visual_rect.top + 16.0 * scale, right: item.visual_rect.right, bottom: item.visual_rect.bottom,
                    });
                } else {
                    let bat = if vm.battery_charging {
                        format!("Audio {}%  •  {}%+", vm.volume_pct, vm.battery_pct)
                    } else {
                        format!("Audio {}%  •  {}%", vm.volume_pct, vm.battery_pct)
                    };
                    draw_text(rt, &bat, &formats.caption, &brushes.text_secondary, &D2D_RECT_F {
                        left: item.visual_rect.left, top: item.visual_rect.top, right: item.visual_rect.right + 140.0 * scale, bottom: item.visual_rect.bottom,
                    });
                }
            }
            ItemId::Prev => draw_transport_btn(rt, brushes, item, IconKind::Previous, hov, false, scale),
            ItemId::PlayPause => draw_transport_btn(rt, brushes, item, if vm.media_playing { IconKind::Pause } else { IconKind::Play }, hov, true, scale),
            ItemId::Next => draw_transport_btn(rt, brushes, item, IconKind::Next, hov, false, scale),
            ItemId::DividerA | ItemId::DividerB | ItemId::DividerC => unsafe {
                rt.DrawLine(D2D_POINT_2F { x: item.visual_rect.left, y: item.visual_rect.top }, D2D_POINT_2F { x: item.visual_rect.left, y: item.visual_rect.bottom }, &brushes.divider, 1.0 * scale, None);
            },
            ItemId::Find => draw_tool_circle(rt, brushes, item, IconKind::Find, active_drawer == Some(1), hov, false, scale),
            ItemId::Shelf => draw_tool_circle(rt, brushes, item, IconKind::Shelf, active_drawer == Some(2), hov, false, scale),
            ItemId::Clipboard => draw_tool_circle(rt, brushes, item, IconKind::Clipboard, active_drawer == Some(3), hov, false, scale),
            ItemId::Mic => draw_tool_circle(rt, brushes, item, if vm.mic_active { IconKind::Mic } else { IconKind::MicMuted }, false, hov, !vm.mic_active, scale),
            ItemId::Bell => draw_tool_circle(rt, brushes, item, IconKind::Bell, active_drawer == Some(4), hov, false, scale),
            ItemId::Settings => draw_tool_circle(rt, brushes, item, IconKind::Gear, active_drawer == Some(5), hov, false, scale),
            ItemId::Badge if vm.unread_notifications > 0 => {
                let cx = (item.visual_rect.left + item.visual_rect.right) / 2.0;
                let cy = (item.visual_rect.top + item.visual_rect.bottom) / 2.0;
                let badge = D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx, y: cy }, radiusX: 6.8 * scale, radiusY: 6.8 * scale };
                unsafe {
                    rt.FillEllipse(&badge, &brushes.danger);
                    rt.DrawEllipse(&badge, &brushes.surface, 1.4 * scale, None);
                }
                let s = if vm.unread_notifications > 9 { "9+".into() } else { vm.unread_notifications.to_string() };
                draw_text(rt, &s, &formats.badge, &brushes.text_primary, &D2D_RECT_F {
                    left: item.visual_rect.left, top: item.visual_rect.top, right: item.visual_rect.right, bottom: item.visual_rect.bottom,
                });
            }
            ItemId::Clock => {
                let date_str = get_current_date_time_string();
                draw_text(rt, &date_str, &formats.clock, &brushes.text_primary, &D2D_RECT_F {
                    left: item.visual_rect.left, top: item.visual_rect.top, right: item.visual_rect.right, bottom: item.visual_rect.bottom,
                });
            }
            _ => {}
        }
    }
}
