// nuvra-platform-win/src/draw/banner.rs
// Compact 480x56 toast banner rendering for incoming notifications (Section 14.4, D-WIN-05).
// R-13: Unsafe confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::{D2D_POINT_2F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{ID2D1RenderTarget, D2D1_ELLIPSE, D2D1_ROUNDED_RECT};

use super::svg_icons::draw_bell_svg;
use super::text::draw_text;
use super::{DrawBrushes, RenderViewModel, TextFormats};
use nuvra_render::layout::{ItemId, Layout};

pub fn draw_banner(
    rt: &ID2D1RenderTarget,
    b: &DrawBrushes,
    f: &TextFormats,
    layout: &Layout,
    vm: &RenderViewModel<'_>,
) {
    let s = layout.scale;
    let (app, summary) = vm
        .notifications
        .first()
        .map(|(a, t)| (a.as_str(), t.as_str()))
        .unwrap_or(("Notification", "New event"));

    for item in &layout.items {
        match item.id {
            ItemId::BannerAppIcon => {
                let plate = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: item.visual_rect.left,
                        top: item.visual_rect.top,
                        right: item.visual_rect.right,
                        bottom: item.visual_rect.bottom,
                    },
                    radiusX: 8.0 * s,
                    radiusY: 8.0 * s,
                };
                // SAFETY: D2D render target drawing banner icon plate & bell glyph.
                unsafe {
                    rt.FillRoundedRectangle(&plate, &b.plate_rest);
                    rt.DrawRoundedRectangle(&plate, &b.divider, 0.8 * s, None);
                }
                let cx = (item.visual_rect.left + item.visual_rect.right) / 2.0;
                let cy = (item.visual_rect.top + item.visual_rect.bottom) / 2.0;
                draw_bell_svg(rt, cx, cy, &b.text_primary, 0.9 * s);
                let dot = D2D1_ELLIPSE { point: D2D_POINT_2F { x: cx + 5.5 * s, y: cy - 5.0 * s }, radiusX: 2.3 * s, radiusY: 2.3 * s };
                // SAFETY: Drawing indicator dot on banner bell icon.
                unsafe { rt.FillEllipse(&dot, &b.ok); }
            }
            ItemId::BannerText => {
                let mid = (item.visual_rect.top + item.visual_rect.bottom) / 2.0;
                draw_text(rt, app, &f.title, &b.text_primary, &D2D_RECT_F {
                    left: item.visual_rect.left, top: item.visual_rect.top, right: item.visual_rect.right, bottom: mid + 1.0 * s,
                });
                draw_text(rt, summary, &f.caption, &b.text_secondary, &D2D_RECT_F {
                    left: item.visual_rect.left, top: mid - 1.0 * s, right: item.visual_rect.right, bottom: item.visual_rect.bottom,
                });
            }
            ItemId::BannerClose => {
                let plate = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: item.visual_rect.left,
                        top: item.visual_rect.top,
                        right: item.visual_rect.right,
                        bottom: item.visual_rect.bottom,
                    },
                    radiusX: 6.0 * s,
                    radiusY: 6.0 * s,
                };
                // SAFETY: D2D render target drawing banner dismiss pill.
                unsafe {
                    rt.FillRoundedRectangle(&plate, &b.plate_rest);
                    rt.DrawRoundedRectangle(&plate, &b.divider, 0.8 * s, None);
                }
                draw_text(rt, "×", &f.caption_center, &b.text_secondary, &plate.rect);
            }
            _ => {}
        }
    }
}
