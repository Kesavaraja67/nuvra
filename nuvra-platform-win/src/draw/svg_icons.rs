// nuvra-platform-win/src/draw/svg_icons.rs
// High-precision Direct2D PathGeometry SVG-grade icons (Wi-Fi arcs, Ethernet LAN, Bell, Lightning).
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_BEZIER_SEGMENT, D2D1_FIGURE_BEGIN_FILLED, D2D1_FIGURE_BEGIN_HOLLOW,
    D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN, D2D_POINT_2F, D2D_RECT_F, D2D_SIZE_F,
};
use windows::Win32::Graphics::Direct2D::{
    ID2D1Brush, ID2D1RenderTarget, ID2D1SolidColorBrush, ID2D1StrokeStyle, D2D1_ARC_SEGMENT,
    D2D1_ARC_SIZE_SMALL, D2D1_CAP_STYLE_ROUND, D2D1_DASH_STYLE_SOLID, D2D1_ELLIPSE,
    D2D1_LINE_JOIN_ROUND, D2D1_ROUNDED_RECT, D2D1_STROKE_STYLE_PROPERTIES,
    D2D1_SWEEP_DIRECTION_CLOCKWISE, D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
};

unsafe fn round_stroke(rt: &ID2D1RenderTarget) -> Option<ID2D1StrokeStyle> {
    let factory = rt.GetFactory().ok()?;
    let props = D2D1_STROKE_STYLE_PROPERTIES {
        startCap: D2D1_CAP_STYLE_ROUND,
        endCap: D2D1_CAP_STYLE_ROUND,
        dashCap: D2D1_CAP_STYLE_ROUND,
        lineJoin: D2D1_LINE_JOIN_ROUND,
        miterLimit: 1.0,
        dashStyle: D2D1_DASH_STYLE_SOLID,
        dashOffset: 0.0,
    };
    factory.CreateStrokeStyle(&props, None).ok()
}

/// Clean SVG-style Wi-Fi symbol: 3 true concentric circular arcs + solid bottom dot.
pub fn draw_wifi_svg(rt: &ID2D1RenderTarget, cx: f32, cy: f32, brush: &ID2D1SolidColorBrush, offline: bool, s: f32) {
    let ox = cx;
    let oy = cy + 5.0 * s;
    let dot = D2D1_ELLIPSE { point: D2D_POINT_2F { x: ox, y: oy }, radiusX: 1.55 * s, radiusY: 1.55 * s };
    let k = std::f32::consts::FRAC_1_SQRT_2; // 45-degree symmetric arc half-angle

    // SAFETY: Valid D2D factory, PathGeometry, and GeometrySink creating crisp circular Wi-Fi wave arcs.
    unsafe {
        rt.FillEllipse(&dot, brush);
        let Ok(factory) = rt.GetFactory() else { return };
        let Ok(geo) = factory.CreatePathGeometry() else { return };
        let Ok(sink) = geo.Open() else { return };
        for r in [4.5 * s, 7.7 * s, 10.9 * s] {
            let start = D2D_POINT_2F { x: ox - r * k, y: oy - r * k };
            let end = D2D_POINT_2F { x: ox + r * k, y: oy - r * k };
            sink.BeginFigure(start, D2D1_FIGURE_BEGIN_HOLLOW);
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: end,
                size: D2D_SIZE_F { width: r, height: r },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            });
            sink.EndFigure(D2D1_FIGURE_END_OPEN);
        }
        let _ = sink.Close();
        let stroke = round_stroke(rt);
        rt.DrawGeometry(&geo, brush, 1.65 * s, stroke.as_ref());
        if offline {
            rt.DrawLine(D2D_POINT_2F { x: cx - 7.5 * s, y: cy + 5.5 * s }, D2D_POINT_2F { x: cx + 7.5 * s, y: cy - 6.5 * s }, brush, 1.5 * s, stroke.as_ref());
        }
    }
}

/// Clean SVG-style wired Ethernet / LAN topology glyph.
pub fn draw_ethernet_svg(rt: &ID2D1RenderTarget, cx: f32, cy: f32, brush: &ID2D1SolidColorBrush, s: f32) {
    let top_node = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: cx - 3.0 * s, top: cy - 6.5 * s, right: cx + 3.0 * s, bottom: cy - 2.3 * s }, radiusX: 1.0 * s, radiusY: 1.0 * s };
    let left_node = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: cx - 7.0 * s, top: cy + 2.5 * s, right: cx - 2.2 * s, bottom: cy + 6.3 * s }, radiusX: 1.0 * s, radiusY: 1.0 * s };
    let right_node = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: cx + 2.2 * s, top: cy + 2.5 * s, right: cx + 7.0 * s, bottom: cy + 6.3 * s }, radiusX: 1.0 * s, radiusY: 1.0 * s };
    // SAFETY: D2D render target drawing round-capped wired Ethernet bus topology.
    unsafe {
        let stroke = round_stroke(rt);
        rt.DrawRoundedRectangle(&top_node, brush, 1.35 * s, stroke.as_ref());
        rt.DrawLine(D2D_POINT_2F { x: cx, y: cy - 2.3 * s }, D2D_POINT_2F { x: cx, y: cy + 0.4 * s }, brush, 1.35 * s, stroke.as_ref());
        rt.DrawLine(D2D_POINT_2F { x: cx - 4.6 * s, y: cy + 0.4 * s }, D2D_POINT_2F { x: cx + 4.6 * s, y: cy + 0.4 * s }, brush, 1.35 * s, stroke.as_ref());
        rt.DrawLine(D2D_POINT_2F { x: cx - 4.6 * s, y: cy + 0.4 * s }, D2D_POINT_2F { x: cx - 4.6 * s, y: cy + 2.5 * s }, brush, 1.35 * s, stroke.as_ref());
        rt.DrawLine(D2D_POINT_2F { x: cx + 4.6 * s, y: cy + 0.4 * s }, D2D_POINT_2F { x: cx + 4.6 * s, y: cy + 2.5 * s }, brush, 1.35 * s, stroke.as_ref());
        rt.DrawRoundedRectangle(&left_node, brush, 1.35 * s, stroke.as_ref());
        rt.DrawRoundedRectangle(&right_node, brush, 1.35 * s, stroke.as_ref());
    }
}

/// Clean SVG-style notification bell using smooth cubic Bezier contour and clapper arc.
pub fn draw_bell_svg(rt: &ID2D1RenderTarget, cx: f32, cy: f32, brush: &ID2D1SolidColorBrush, s: f32) {
    // SAFETY: Valid D2D factory, PathGeometry, and GeometrySink creating smooth rounded bell SVG.
    unsafe {
        let Ok(factory) = rt.GetFactory() else { return };
        let Ok(geo) = factory.CreatePathGeometry() else { return };
        let Ok(sink) = geo.Open() else { return };

        // Top loop stem
        sink.BeginFigure(D2D_POINT_2F { x: cx, y: cy - 6.8 * s }, D2D1_FIGURE_BEGIN_HOLLOW);
        sink.AddLine(D2D_POINT_2F { x: cx, y: cy - 5.3 * s });
        sink.EndFigure(D2D1_FIGURE_END_OPEN);

        // Smooth bell body contour
        sink.BeginFigure(D2D_POINT_2F { x: cx - 6.0 * s, y: cy + 3.2 * s }, D2D1_FIGURE_BEGIN_HOLLOW);
        sink.AddBezier(&D2D1_BEZIER_SEGMENT {
            point1: D2D_POINT_2F { x: cx - 4.1 * s, y: cy + 1.4 * s },
            point2: D2D_POINT_2F { x: cx - 4.6 * s, y: cy - 5.2 * s },
            point3: D2D_POINT_2F { x: cx, y: cy - 5.2 * s },
        });
        sink.AddBezier(&D2D1_BEZIER_SEGMENT {
            point1: D2D_POINT_2F { x: cx + 4.6 * s, y: cy - 5.2 * s },
            point2: D2D_POINT_2F { x: cx + 4.1 * s, y: cy + 1.4 * s },
            point3: D2D_POINT_2F { x: cx + 6.0 * s, y: cy + 3.2 * s },
        });
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);

        // Bottom clapper arc
        sink.BeginFigure(D2D_POINT_2F { x: cx - 1.9 * s, y: cy + 4.3 * s }, D2D1_FIGURE_BEGIN_HOLLOW);
        sink.AddArc(&D2D1_ARC_SEGMENT {
            point: D2D_POINT_2F { x: cx + 1.9 * s, y: cy + 4.3 * s },
            size: D2D_SIZE_F { width: 2.0 * s, height: 2.0 * s },
            rotationAngle: 0.0,
            sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
            arcSize: D2D1_ARC_SIZE_SMALL,
        });
        sink.EndFigure(D2D1_FIGURE_END_OPEN);

        let _ = sink.Close();
        let stroke = round_stroke(rt);
        rt.DrawGeometry(&geo, brush, 1.45 * s, stroke.as_ref());
    }
}

/// Crisp filled SVG charging lightning bolt (~13px tall).
pub fn draw_bolt_svg(rt: &ID2D1RenderTarget, cx: f32, cy: f32, brush: &ID2D1SolidColorBrush, s: f32) {
    // SAFETY: Valid D2D factory, PathGeometry, and GeometrySink creating filled lightning polygon.
    unsafe {
        let Ok(factory) = rt.GetFactory() else { return };
        let Ok(geo) = factory.CreatePathGeometry() else { return };
        let Ok(sink) = geo.Open() else { return };
        sink.BeginFigure(D2D_POINT_2F { x: cx + 1.4 * s, y: cy - 6.5 * s }, D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLines(&[
            D2D_POINT_2F { x: cx - 3.6 * s, y: cy + 0.6 * s },
            D2D_POINT_2F { x: cx - 0.3 * s, y: cy + 0.6 * s },
            D2D_POINT_2F { x: cx - 1.4 * s, y: cy + 6.5 * s },
            D2D_POINT_2F { x: cx + 3.6 * s, y: cy - 0.6 * s },
            D2D_POINT_2F { x: cx + 0.3 * s, y: cy - 0.6 * s },
        ]);
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        let _ = sink.Close();
        rt.FillGeometry(&geo, brush, None::<&ID2D1Brush>);
    }
}
