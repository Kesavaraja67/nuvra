// nuvra-platform-win/src/draw/glass.rs
// Smoked-quartz optical Glassmorphism primitives (frosted translucency, hairline catchlight rim, inset wells).
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, D2D1_GRADIENT_STOP, D2D_POINT_2F, D2D_RECT_F};
use windows::Win32::Graphics::Direct2D::{
    ID2D1LinearGradientBrush, ID2D1RadialGradientBrush, ID2D1RenderTarget, D2D1_EXTEND_MODE_CLAMP,
    D2D1_GAMMA_2_2, D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
    D2D1_ROUNDED_RECT,
};

use nuvra_render::engine::GlassStyle;
use nuvra_render::layout::Layout;
use nuvra_render::tokens;

unsafe fn make_linear_brush(
    rt: &ID2D1RenderTarget,
    start: D2D_POINT_2F,
    end: D2D_POINT_2F,
    stops: &[D2D1_GRADIENT_STOP],
) -> Option<ID2D1LinearGradientBrush> {
    let collection = rt.CreateGradientStopCollection(stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP).ok()?;
    let props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES { startPoint: start, endPoint: end };
    rt.CreateLinearGradientBrush(&props, None, &collection).ok()
}

unsafe fn make_radial_brush(
    rt: &ID2D1RenderTarget,
    center: D2D_POINT_2F,
    rx: f32,
    ry: f32,
    stops: &[D2D1_GRADIENT_STOP],
) -> Option<ID2D1RadialGradientBrush> {
    let collection = rt.CreateGradientStopCollection(stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP).ok()?;
    let props = D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES { center, gradientOriginOffset: D2D_POINT_2F { x: 0.0, y: 0.0 }, radiusX: rx, radiusY: ry };
    rt.CreateRadialGradientBrush(&props, None, &collection).ok()
}

fn shell_gradient_stops(glass_style: GlassStyle, is_dark: bool) -> ([D2D1_GRADIENT_STOP; 3], [D2D1_GRADIENT_STOP; 3]) {
    let (a_top, a_mid, a_bot) = match glass_style {
        GlassStyle::Clear => (0.48, 0.54, 0.62),
        GlassStyle::Frosted => (0.68, 0.74, 0.80),
        GlassStyle::Matte => (0.94, 0.96, 0.98),
    };
    if is_dark {
        (
            [
                D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 0.09, g: 0.095, b: 0.11, a: a_top } },
                D2D1_GRADIENT_STOP { position: 0.55, color: D2D1_COLOR_F { r: 0.05, g: 0.055, b: 0.065, a: a_mid } },
                D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 0.03, g: 0.033, b: 0.04, a: a_bot } },
            ],
            [
                D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.14 } },
                D2D1_GRADIENT_STOP { position: 0.45, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.05 } },
                D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 0.80, g: 0.84, b: 0.90, a: 0.10 } },
            ],
        )
    } else {
        (
            [
                D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 0.97, g: 0.98, b: 1.0, a: a_top + 0.08 } },
                D2D1_GRADIENT_STOP { position: 0.55, color: D2D1_COLOR_F { r: 0.92, g: 0.94, b: 0.97, a: a_mid + 0.08 } },
                D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 0.88, g: 0.90, b: 0.94, a: a_bot.min(0.95) } },
            ],
            [
                D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.88 } },
                D2D1_GRADIENT_STOP { position: 0.5, color: D2D1_COLOR_F { r: 0.78, g: 0.81, b: 0.87, a: 0.45 } },
                D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 0.65, g: 0.68, b: 0.76, a: 0.35 } },
            ],
        )
    }
}

/// Draw smoked-quartz optical glass shell with subtle cool top sheen and specular catchlight border.
pub fn draw_glass_shell(
    rt: &ID2D1RenderTarget,
    layout: &Layout,
    glass_style: GlassStyle,
    is_dark: bool,
    fallback_surface: &windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush,
    fallback_edge: &windows::Win32::Graphics::Direct2D::ID2D1SolidColorBrush,
) {
    let scale = layout.scale;
    let w = layout.body.right - layout.body.left;
    let h = layout.body.bottom - layout.body.top;
    let r = (tokens::radius::BOTTOM * scale).min((h - 1.0 * scale) * 0.5).max(1.0);
    let body_rect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: layout.body.left, top: layout.body.top, right: layout.body.right, bottom: layout.body.bottom },
        radiusX: r,
        radiusY: r,
    };
    let inset = 0.5 * scale;
    let rim_rect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: layout.body.left + inset,
            top: layout.body.top + inset,
            right: layout.body.right - inset,
            bottom: layout.body.bottom - inset,
        },
        radiusX: (r - inset).max(1.0),
        radiusY: (r - inset).max(1.0),
    };
    let (bg_stops, rim_stops) = shell_gradient_stops(glass_style, is_dark);

    // SAFETY: Direct2D gradient fill, specular sheen, and inset hairline rim inside BeginDraw/EndDraw.
    unsafe {
        let start = D2D_POINT_2F { x: layout.body.left, y: layout.body.top };
        let end = D2D_POINT_2F { x: layout.body.right, y: layout.body.bottom };
        if let Some(bg_brush) = make_linear_brush(rt, start, end, &bg_stops) {
            rt.FillRoundedRectangle(&body_rect, &bg_brush);
        } else {
            rt.FillRoundedRectangle(&body_rect, fallback_surface);
        }

        // Cool neutral glass optical sheen inside shell
        let sheen_a = if is_dark { 0.045 } else { 0.28 };
        let sheen_stops = [
            D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: sheen_a } },
            D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.0 } },
        ];
        if let Some(sheen) = make_radial_brush(
            rt,
            D2D_POINT_2F { x: layout.body.left + w * 0.5, y: layout.body.top },
            (w * 0.60).max(100.0),
            (h * 0.90).max(36.0),
            &sheen_stops,
        ) {
            rt.FillRoundedRectangle(&body_rect, &sheen);
        }

        if let Some(rim_brush) = make_linear_brush(rt, start, end, &rim_stops) {
            rt.DrawRoundedRectangle(&rim_rect, &rim_brush, 1.0 * scale, None);
        } else {
            rt.DrawRoundedRectangle(&rim_rect, fallback_edge, 1.0 * scale, None);
        }
    }
}

/// Draw elevated glass container with hairline specular rim.
pub fn draw_glass_card(rt: &ID2D1RenderTarget, rect: D2D_RECT_F, radius: f32, hovered: bool, is_dark: bool, scale: f32) {
    let rrect = D2D1_ROUNDED_RECT { rect, radiusX: radius, radiusY: radius };
    let ins = 0.5 * scale;
    let rim_rect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: rect.left + ins, top: rect.top + ins, right: rect.right - ins, bottom: rect.bottom - ins },
        radiusX: (radius - ins).max(1.0),
        radiusY: (radius - ins).max(1.0),
    };
    let (top_a, bot_a, rim_a) = match (is_dark, hovered) {
        (true, true) => (0.10, 0.045, 0.22),
        (true, false) => (0.045, 0.018, 0.10),
        (false, true) => (0.65, 0.45, 0.85),
        (false, false) => (0.44, 0.30, 0.60),
    };
    let fill_stops = [
        D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: top_a } },
        D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: bot_a } },
    ];
    let border_stops = [
        D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: rim_a } },
        D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: rim_a * 0.28 } },
    ];
    // SAFETY: Direct2D gradient fill & hairline stroke within active render target pass.
    unsafe {
        let p0 = D2D_POINT_2F { x: rect.left, y: rect.top };
        let p1 = D2D_POINT_2F { x: rect.right, y: rect.bottom };
        if let Some(fill) = make_linear_brush(rt, p0, p1, &fill_stops) {
            rt.FillRoundedRectangle(&rrect, &fill);
        }
        if let Some(border) = make_linear_brush(rt, p0, p1, &border_stops) {
            rt.DrawRoundedRectangle(&rim_rect, &border, if hovered { 1.0 } else { 0.8 } * scale, None);
        }
    }
}

/// Draw a slightly darker inset glass well (for interactive rows & file drop pocket).
pub fn draw_recessed_well(rt: &ID2D1RenderTarget, rect: D2D_RECT_F, radius: f32, hovered: bool, is_dark: bool, scale: f32) {
    let rrect = D2D1_ROUNDED_RECT { rect, radiusX: radius, radiusY: radius };
    let ins = 0.5 * scale;
    let rim_rect = D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F { left: rect.left + ins, top: rect.top + ins, right: rect.right - ins, bottom: rect.bottom - ins },
        radiusX: (radius - ins).max(1.0),
        radiusY: (radius - ins).max(1.0),
    };
    let (fill_col, rim_a) = match (is_dark, hovered) {
        (true, true) => (D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.09 }, 0.22),
        (true, false) => (D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.26 }, 0.09),
        (false, true) => (D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.08 }, 0.32),
        (false, false) => (D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.045 }, 0.16),
    };
    let stops = [
        D2D1_GRADIENT_STOP { position: 0.0, color: fill_col },
        D2D1_GRADIENT_STOP { position: 1.0, color: fill_col },
    ];
    let rim_stops = [
        D2D1_GRADIENT_STOP { position: 0.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: rim_a } },
        D2D1_GRADIENT_STOP { position: 1.0, color: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: rim_a * 0.4 } },
    ];
    // SAFETY: Direct2D gradient fill & hairline stroke within active draw pass.
    unsafe {
        let p0 = D2D_POINT_2F { x: rect.left, y: rect.top };
        let p1 = D2D_POINT_2F { x: rect.right, y: rect.bottom };
        if let Some(fill) = make_linear_brush(rt, p0, p1, &stops) {
            rt.FillRoundedRectangle(&rrect, &fill);
        }
        if let Some(border) = make_linear_brush(rt, p0, p1, &rim_stops) {
            rt.DrawRoundedRectangle(&rim_rect, &border, 0.8 * scale, None);
        }
    }
}
