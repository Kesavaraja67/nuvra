// nuvra-platform-win/src/draw/overlay.rs
// Move mode rim, magnetic snap flash & tooltip overlay — Document UI-1 Sections 8.3 & 9.6.
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows::Win32::Graphics::Direct2D::{ID2D1RenderTarget, D2D1_ROUNDED_RECT};

use super::DrawBrushes;
use nuvra_render::layout::Layout;
use nuvra_render::tokens;

pub fn draw_move_rim(
    rt: &ID2D1RenderTarget,
    brushes: &DrawBrushes,
    layout: &Layout,
    is_snap_flash: bool,
) {
    let scale = layout.scale;
    let r = tokens::radius::BOTTOM * scale;

    let body_rect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: layout.body.left,
            top: layout.body.top,
            right: layout.body.right,
            bottom: layout.body.bottom,
        },
        radiusX: r,
        radiusY: r,
    };

    let rim_brush = if is_snap_flash {
        &brushes.ok
    } else {
        &brushes.accent
    };

    unsafe {
        rt.DrawRoundedRectangle(&body_rect, rim_brush, 1.5 * scale, None);
    }
}
