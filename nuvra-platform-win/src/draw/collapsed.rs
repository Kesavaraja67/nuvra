// nuvra-platform-win/src/draw/collapsed.rs
// Collapsed notch HUD: Charging/Battery + %, clean Bell notification SVG, live Wi-Fi/Ethernet SVG & Clock.
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::{D2D_POINT_2F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{ID2D1RenderTarget, D2D1_ELLIPSE, D2D1_ROUNDED_RECT};

use super::icons::draw_icon;
use super::svg_icons::{draw_bell_svg, draw_bolt_svg, draw_ethernet_svg, draw_wifi_svg};
use super::text::{draw_text, get_date_and_time_parts};
use super::{DrawBrushes, RenderViewModel, TextFormats};
use crate::network::NetworkKind;
use nuvra_render::icons::IconKind;
use nuvra_render::layout::{ItemId, Layout};

fn draw_battery_capsule(
    rt: &ID2D1RenderTarget,
    b: &DrawBrushes,
    f: &TextFormats,
    x: f32,
    cy: f32,
    vm: &RenderViewModel<'_>,
    s: f32,
) {
    let bat_x = if vm.battery_charging {
        draw_bolt_svg(rt, x + 4.5 * s, cy, &b.ok, s);
        x + 13.0 * s
    } else {
        x
    };
    let (bat_w, bat_h) = (22.0 * s, 11.0 * s);
    let top = cy - bat_h / 2.0;
    let outline = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: bat_x,
            top,
            right: bat_x + bat_w,
            bottom: top + bat_h,
        },
        radiusX: 2.8 * s,
        radiusY: 2.8 * s,
    };
    let nub = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: bat_x + bat_w + 0.9 * s,
            top: top + 3.2 * s,
            right: bat_x + bat_w + 2.7 * s,
            bottom: top + bat_h - 3.2 * s,
        },
        radiusX: 0.9 * s,
        radiusY: 0.9 * s,
    };
    let pct = vm.battery_pct.clamp(0, 100) as f32 / 100.0;
    let fill_w = (bat_w - 4.0 * s).max(2.0) * pct.max(0.10);
    let fill_r = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: bat_x + 2.0 * s,
            top: top + 2.0 * s,
            right: bat_x + 2.0 * s + fill_w,
            bottom: top + bat_h - 2.0 * s,
        },
        radiusX: 1.2 * s,
        radiusY: 1.2 * s,
    };
    let semantic = if vm.battery_charging {
        &b.ok
    } else if vm.battery_pct <= 20 {
        &b.danger
    } else if vm.battery_pct <= 35 {
        &b.warning
    } else {
        &b.text_primary
    };
    // SAFETY: D2D render target drawing crisp battery capsule and semantic charge fill.
    unsafe {
        rt.DrawRoundedRectangle(&outline, &b.text_secondary, 1.05 * s, None);
        rt.FillRoundedRectangle(&nub, &b.text_secondary);
        rt.FillRoundedRectangle(&fill_r, semantic);
    }
    let text_x = bat_x + bat_w + 6.0 * s;
    let text_col = if vm.battery_charging || vm.battery_pct <= 35 {
        semantic
    } else {
        &b.text_primary
    };
    draw_text(
        rt,
        &format!("{}%", vm.battery_pct),
        &f.caption,
        text_col,
        &D2D_RECT_F {
            left: text_x,
            top: 0.0,
            right: text_x + 40.0 * s,
            bottom: 38.0 * s,
        },
    );
}

fn draw_notification_status(
    rt: &ID2D1RenderTarget,
    b: &DrawBrushes,
    cx: f32,
    cy: f32,
    unread: u32,
    s: f32,
) {
    let bell_col = if unread > 0 {
        &b.text_primary
    } else {
        &b.text_secondary
    };
    draw_bell_svg(rt, cx, cy, bell_col, 0.95 * s);
    if unread > 0 {
        let dot = D2D1_ELLIPSE {
            point: D2D_POINT_2F {
                x: cx + 5.2 * s,
                y: cy - 4.8 * s,
            },
            radiusX: 2.5 * s,
            radiusY: 2.5 * s,
        };
        // SAFETY: Valid D2D render target drawing unread notification dot on bell shoulder.
        unsafe {
            rt.FillEllipse(&dot, &b.danger);
        }
    }
}

pub fn draw_network_indicator(
    rt: &ID2D1RenderTarget,
    b: &DrawBrushes,
    cx: f32,
    cy: f32,
    net: NetworkKind,
    s: f32,
) {
    match net {
        NetworkKind::Wifi => draw_wifi_svg(rt, cx, cy, &b.text_primary, false, 0.88 * s),
        NetworkKind::Ethernet => draw_ethernet_svg(rt, cx, cy, &b.text_primary, 0.95 * s),
        NetworkKind::Offline => draw_wifi_svg(rt, cx, cy, &b.text_tertiary, true, 0.88 * s),
    }
}

fn draw_collapsed_media_pill(
    rt: &ID2D1RenderTarget,
    b: &DrawBrushes,
    f: &TextFormats,
    cy: f32,
    vm: &RenderViewModel<'_>,
    s: f32,
) {
    // SAFETY: D2D render target drawing miniature live equalizer bars for collapsed playing state.
    unsafe {
        for i in 0..3 {
            let amp = if vm.media_playing {
                0.35 + 0.65 * ((vm.eq_phase * 5.2 + i as f32 * 1.5).sin().abs())
            } else {
                0.30
            };
            let bh = 12.0 * s * amp;
            let bx = 24.0 * s + (i as f32) * 4.2 * s;
            let bar = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: bx,
                    top: cy - bh / 2.0,
                    right: bx + 2.3 * s,
                    bottom: cy + bh / 2.0,
                },
                radiusX: 1.1 * s,
                radiusY: 1.1 * s,
            };
            rt.FillRoundedRectangle(&bar, &b.ok);
        }
    }
    draw_text(
        rt,
        vm.track_title,
        &f.caption,
        &b.text_primary,
        &D2D_RECT_F {
            left: 42.0 * s,
            top: 0.0,
            right: 174.0 * s,
            bottom: 38.0 * s,
        },
    );
}

pub fn draw_collapsed(
    rt: &ID2D1RenderTarget,
    brushes: &DrawBrushes,
    formats: &TextFormats,
    layout: &Layout,
    vm: &RenderViewModel<'_>,
    is_move_mode: bool,
) {
    let s = layout.scale;
    let cy = (layout.body.top + layout.body.bottom) / 2.0;
    let clock_right = layout
        .items
        .iter()
        .find(|i| i.id == ItemId::Clock)
        .map(|i| i.visual_rect.right)
        .unwrap_or(layout.body.right - 16.0 * s);

    if is_move_mode {
        draw_icon(rt, IconKind::Grip, 30.0 * s, cy, &brushes.text_secondary, s);
        draw_text(
            rt,
            "Drag to position",
            &formats.caption,
            &brushes.text_secondary,
            &D2D_RECT_F {
                left: 46.0 * s,
                top: 0.0,
                right: 174.0 * s,
                bottom: 38.0 * s,
            },
        );
    } else if vm.media_active {
        draw_collapsed_media_pill(rt, brushes, formats, cy, vm, s);
    } else {
        draw_battery_capsule(rt, brushes, formats, 19.0 * s, cy, vm, s);
        draw_notification_status(rt, brushes, 112.0 * s, cy, vm.unread_notifications, s);
        draw_network_indicator(rt, brushes, 148.0 * s, cy, vm.network, s);
    }

    // SAFETY: D2D render target drawing hairline separator before date/time cluster.
    unsafe {
        rt.DrawLine(
            D2D_POINT_2F {
                x: 180.0 * s,
                y: 11.0 * s,
            },
            D2D_POINT_2F {
                x: 180.0 * s,
                y: 27.0 * s,
            },
            &brushes.divider,
            1.0 * s,
            None,
        );
    }

    let (date_part, time_part) = get_date_and_time_parts();
    let time_left = clock_right - 60.0 * s;
    draw_text(
        rt,
        &date_part,
        &formats.clock,
        &brushes.text_secondary,
        &D2D_RECT_F {
            left: 188.0 * s,
            top: cy - 10.0 * s,
            right: time_left - 4.0 * s,
            bottom: cy + 10.0 * s,
        },
    );
    draw_text(
        rt,
        &time_part,
        &formats.clock,
        &brushes.text_primary,
        &D2D_RECT_F {
            left: time_left,
            top: cy - 10.0 * s,
            right: clock_right,
            bottom: cy + 10.0 * s,
        },
    );
}
