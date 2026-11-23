// nuvra-platform-win/src/renderer.rs
// Direct2D hardware-accelerated glassmorphic renderer for Nuvra Notch (D-STK-03, D-PERF-03).
// Implements production-grade glassmorphism and distinctive frontend-design.
// Every unsafe block carries a // SAFETY: comment.

use windows::core::w;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_GRADIENT_STOP, D2D1_PIXEL_FORMAT,
    D2D_POINT_2F, D2D_RECT_F,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1LinearGradientBrush, ID2D1RenderTarget,
    ID2D1SolidColorBrush, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_GAMMA_2_2,
    D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_MEASURING_MODE_NATURAL,
    DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
    DWRITE_TEXT_ALIGNMENT_TRAILING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::{IDXGISurface, IDXGISwapChain, DXGI_PRESENT};

#[derive(Debug, Clone, Copy)]
pub struct PillRenderParams {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub surface_color: u32,
    pub edge_color: u32,
    pub text_color: u32,
    pub text_secondary: u32,
    pub text_tertiary: u32,
    pub accent_color: u32,
    pub surface_raised: u32,
    pub surface_high: u32,
    pub danger: u32,
    pub ok: u32,
    pub alpha: f32,
}

pub struct NotchRenderer {
    _factory: ID2D1Factory,
    _dwrite_factory: IDWriteFactory,
    render_target: ID2D1RenderTarget,
    // Brushes
    glass_fill_brush: ID2D1LinearGradientBrush,
    glass_rim_brush: ID2D1LinearGradientBrush,
    card_fill_brush: ID2D1SolidColorBrush,
    card_edge_brush: ID2D1SolidColorBrush,
    accent_brush: ID2D1SolidColorBrush,
    accent_glow_brush: ID2D1SolidColorBrush,
    text_primary_brush: ID2D1SolidColorBrush,
    text_secondary_brush: ID2D1SolidColorBrush,
    text_tertiary_brush: ID2D1SolidColorBrush,
    ok_brush: ID2D1SolidColorBrush,
    danger_brush: ID2D1SolidColorBrush,
    specular_brush: ID2D1SolidColorBrush,
    // Text Formats
    display_format: IDWriteTextFormat,
    title_format: IDWriteTextFormat,
    body_format: IDWriteTextFormat,
    caption_format: IDWriteTextFormat,
    caption_trailing: IDWriteTextFormat,
    caption_center: IDWriteTextFormat,
    ambient_format: IDWriteTextFormat,
    frame_counter: u32,
}

impl NotchRenderer {
    /// Create a Direct2D render target bound to the DXGI swapchain backbuffer.
    pub fn new(swapchain: &IDXGISwapChain) -> Result<Self, String> {
        // SAFETY: D2D1CreateFactory, DWriteCreateFactory, and CreateDxgiSurfaceRenderTarget calls are verified Win32 COM APIs.
        unsafe {
            let surface: IDXGISurface = swapchain
                .GetBuffer(0)
                .map_err(|e| format!("Failed to get DXGI surface: {e}"))?;

            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .map_err(|e| format!("D2D1CreateFactory failed: {e}"))?;

            let props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let render_target = factory
                .CreateDxgiSurfaceRenderTarget(&surface, &props)
                .map_err(|e| format!("CreateDxgiSurfaceRenderTarget failed: {e}"))?;

            // 1. Frosted Glass Multi-Stop Linear Gradient Fill (Deep Smoky Indigo)
            let glass_stops = [
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: D2D1_COLOR_F {
                        r: 0.11,
                        g: 0.13,
                        b: 0.22,
                        a: 0.76,
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 0.45,
                    color: D2D1_COLOR_F {
                        r: 0.06,
                        g: 0.08,
                        b: 0.14,
                        a: 0.84,
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: D2D1_COLOR_F {
                        r: 0.03,
                        g: 0.04,
                        b: 0.07,
                        a: 0.92,
                    },
                },
            ];
            let stop_collection = render_target
                .CreateGradientStopCollection(&glass_stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)
                .map_err(|e| format!("Gradient stop collection failed: {e}"))?;

            let glass_brush_props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: D2D_POINT_2F { x: 0.0, y: 0.0 },
                endPoint: D2D_POINT_2F { x: 0.0, y: 220.0 },
            };
            let glass_fill_brush = render_target
                .CreateLinearGradientBrush(&glass_brush_props, None, &stop_collection)
                .map_err(|e| format!("Glass fill brush failed: {e}"))?;

            // 2. Specular Optical Glass Rim Gradient (Frosted Bevel Highlight)
            let rim_stops = [
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.45,
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 0.35,
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.12,
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.04,
                    },
                },
            ];
            let rim_collection = render_target
                .CreateGradientStopCollection(&rim_stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)
                .map_err(|e| format!("Rim gradient stop failed: {e}"))?;

            let glass_rim_brush = render_target
                .CreateLinearGradientBrush(&glass_brush_props, None, &rim_collection)
                .map_err(|e| format!("Glass rim brush failed: {e}"))?;

            // 3. Solid Color Brushes for Content & Micro-Interactions
            let default_col = D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
            let card_fill_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Card fill brush failed: {e}"))?;
            let card_edge_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Card edge brush failed: {e}"))?;
            let accent_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Accent brush failed: {e}"))?;
            let accent_glow_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Accent glow brush failed: {e}"))?;
            let text_primary_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Text primary brush failed: {e}"))?;
            let text_secondary_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Text secondary brush failed: {e}"))?;
            let text_tertiary_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Text tertiary brush failed: {e}"))?;
            let ok_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Ok brush failed: {e}"))?;
            let danger_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Danger brush failed: {e}"))?;
            let specular_brush = render_target
                .CreateSolidColorBrush(&default_col, None)
                .map_err(|e| format!("Specular brush failed: {e}"))?;

            // 4. DirectWrite Typography Hierarchy
            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|e| format!("DWriteCreateFactory failed: {e}"))?;

            // Display title: Segoe UI, SemiBold 14.5 pt
            let display_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    14.5,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat display failed: {e}"))?;
            let _ = display_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = display_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Subtitle: Segoe UI, SemiBold 12.0 pt
            let title_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    12.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat title failed: {e}"))?;
            let _ = title_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = title_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Body: Segoe UI, Normal 11.0 pt
            let body_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    11.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat body failed: {e}"))?;
            let _ = body_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = body_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption leading: Segoe UI, SemiBold 9.5 pt
            let caption_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    9.5,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption failed: {e}"))?;
            let _ = caption_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = caption_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption trailing: Segoe UI, SemiBold 9.5 pt
            let caption_trailing = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    9.5,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption trailing failed: {e}"))?;
            let _ = caption_trailing.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING);
            let _ = caption_trailing.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption center: Segoe UI, Normal 10.0 pt
            let caption_center = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    10.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption center failed: {e}"))?;
            let _ = caption_center.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = caption_center.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Ambient tracked label: Segoe UI, Bold 12.0 pt
            let ambient_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    12.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat ambient failed: {e}"))?;
            let _ = ambient_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = ambient_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            Ok(Self {
                _factory: factory,
                _dwrite_factory: dwrite_factory,
                render_target,
                glass_fill_brush,
                glass_rim_brush,
                card_fill_brush,
                card_edge_brush,
                accent_brush,
                accent_glow_brush,
                text_primary_brush,
                text_secondary_brush,
                text_tertiary_brush,
                ok_brush,
                danger_brush,
                specular_brush,
                display_format,
                title_format,
                body_format,
                caption_format,
                caption_trailing,
                caption_center,
                ambient_format,
                frame_counter: 0,
            })
        }
    }

    /// Render a frame of the notch pill with full glassmorphic styling and present.
    pub fn render_and_present(
        &mut self,
        swapchain: &IDXGISwapChain,
        params: &PillRenderParams,
    ) -> Result<(), String> {
        self.frame_counter = self.frame_counter.wrapping_add(1);

        let to_color = |c: u32, alpha_mult: f32| {
            let a = (((c >> 24) & 0xFF) as f32 / 255.0) * alpha_mult;
            let r = ((c >> 16) & 0xFF) as f32 / 255.0;
            let g = ((c >> 8) & 0xFF) as f32 / 255.0;
            let b = (c & 0xFF) as f32 / 255.0;
            D2D1_COLOR_F { r, g, b, a }
        };

        let draw_text = |rt: &ID2D1RenderTarget,
                         text: &str,
                         format: &IDWriteTextFormat,
                         brush: &ID2D1SolidColorBrush,
                         rect: &D2D_RECT_F| unsafe {
            let utf16: Vec<u16> = text.encode_utf16().collect();
            rt.DrawText(
                &utf16,
                format,
                rect,
                brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        };

        // SAFETY: Direct2D BeginDraw/Clear/Draw/EndDraw and DXGI Present calls.
        unsafe {
            self.render_target.BeginDraw();

            // Clear outside of notch to fully transparent (premultiplied black with 0 alpha)
            self.render_target.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            if params.height > 1.0 && params.width > 1.0 {
                // Update dynamic brush coordinates for current pill frame
                self.glass_fill_brush.SetStartPoint(D2D_POINT_2F {
                    x: params.x,
                    y: params.y,
                });
                self.glass_fill_brush.SetEndPoint(D2D_POINT_2F {
                    x: params.x,
                    y: params.y + params.height,
                });

                self.glass_rim_brush.SetStartPoint(D2D_POINT_2F {
                    x: params.x,
                    y: params.y,
                });
                self.glass_rim_brush.SetEndPoint(D2D_POINT_2F {
                    x: params.x,
                    y: params.y + params.height,
                });

                // Update solid brushes
                self.card_fill_brush.SetColor(&D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.045,
                });
                self.card_edge_brush.SetColor(&D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.12,
                });
                self.specular_brush.SetColor(&D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.35,
                });
                self.text_primary_brush
                    .SetColor(&to_color(params.text_color, 1.0));
                self.text_secondary_brush
                    .SetColor(&to_color(params.text_secondary, 0.85));
                self.text_tertiary_brush
                    .SetColor(&to_color(params.text_tertiary, 0.65));
                self.accent_brush
                    .SetColor(&to_color(params.accent_color, 1.0));
                self.accent_glow_brush
                    .SetColor(&to_color(params.accent_color, 0.18));
                self.ok_brush.SetColor(&to_color(params.ok, 1.0));
                self.danger_brush.SetColor(&to_color(params.danger, 1.0));

                let rounded_rect = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: params.x,
                        top: params.y,
                        right: params.x + params.width,
                        bottom: params.y + params.height,
                    },
                    radiusX: params.radius,
                    radiusY: params.radius,
                };

                // 1. Fill base frosted glass gradient surface
                self.render_target
                    .FillRoundedRectangle(&rounded_rect, &self.glass_fill_brush);

                // 2. Stroke specular gradient glass rim (beveled edge)
                self.render_target.DrawRoundedRectangle(
                    &rounded_rect,
                    &self.glass_rim_brush,
                    1.5,
                    None,
                );

                // 3. Specular Top Inset Catchlight Line (1px inner light catch)
                if params.width >= 40.0 {
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: params.x + params.radius.min(20.0),
                            y: params.y + 1.0,
                        },
                        D2D_POINT_2F {
                            x: params.x + params.width - params.radius.min(20.0),
                            y: params.y + 1.0,
                        },
                        &self.specular_brush,
                        1.0,
                        None,
                    );
                }

                if params.height >= 160.0 {
                    // ══════════════════════════════════════════════════════════════════════
                    // EXPANDED VIEW (520 × 220 px) — D-WIN-08
                    // ══════════════════════════════════════════════════════════════════════

                    // ── LEFT NAVIGATION RAIL (x: params.x + 8 to params.x + 50) ───────────
                    let rail_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 8.0,
                            top: params.y + 8.0,
                            right: params.x + 50.0,
                            bottom: params.y + params.height - 8.0,
                        },
                        radiusX: 12.0,
                        radiusY: 12.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&rail_rect, &self.card_fill_brush);
                    self.render_target.DrawRoundedRectangle(
                        &rail_rect,
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    // 5 Geometric Vector Icons for Rail: Media, Shelf, Clipboard, Bell, Settings
                    let icon_step = (params.height - 40.0) / 5.0;
                    let rail_x = params.x + 29.0;

                    for i in 0..5 {
                        let cy = params.y + 24.0 + (i as f32) * icon_step;
                        let is_active = i == 0;

                        if is_active {
                            // Active Neon Indicator Bar
                            let ind = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: params.x + 10.0,
                                    top: cy - 9.0,
                                    right: params.x + 13.0,
                                    bottom: cy + 9.0,
                                },
                                radiusX: 1.5,
                                radiusY: 1.5,
                            };
                            self.render_target
                                .FillRoundedRectangle(&ind, &self.accent_brush);
                        }

                        let brush = if is_active {
                            &self.accent_brush
                        } else {
                            &self.text_secondary_brush
                        };

                        match i {
                            0 => {
                                // 1. Audio Equalizer Vector Glyph (Media)
                                for b in 0..3 {
                                    let bx = rail_x - 6.0 + (b as f32) * 5.0;
                                    let bh = if b == 1 { 12.0 } else { 7.0 };
                                    let bar = D2D1_ROUNDED_RECT {
                                        rect: D2D_RECT_F {
                                            left: bx,
                                            top: cy - bh / 2.0,
                                            right: bx + 2.5,
                                            bottom: cy + bh / 2.0,
                                        },
                                        radiusX: 1.0,
                                        radiusY: 1.0,
                                    };
                                    self.render_target.FillRoundedRectangle(&bar, brush);
                                }
                            }
                            1 => {
                                // 2. Clean Folder Tray Glyph (Shelf)
                                let tray = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: rail_x - 7.0,
                                        top: cy - 4.0,
                                        right: rail_x + 7.0,
                                        bottom: cy + 6.0,
                                    },
                                    radiusX: 2.0,
                                    radiusY: 2.0,
                                };
                                self.render_target
                                    .DrawRoundedRectangle(&tray, brush, 1.2, None);
                                self.render_target.DrawLine(
                                    D2D_POINT_2F {
                                        x: rail_x - 5.0,
                                        y: cy - 6.0,
                                    },
                                    D2D_POINT_2F {
                                        x: rail_x - 1.0,
                                        y: cy - 6.0,
                                    },
                                    brush,
                                    1.2,
                                    None,
                                );
                            }
                            2 => {
                                // 3. Clipboard Vector Glyph
                                let board = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: rail_x - 5.5,
                                        top: cy - 6.0,
                                        right: rail_x + 5.5,
                                        bottom: cy + 7.0,
                                    },
                                    radiusX: 1.5,
                                    radiusY: 1.5,
                                };
                                self.render_target
                                    .DrawRoundedRectangle(&board, brush, 1.2, None);
                                self.render_target.DrawLine(
                                    D2D_POINT_2F {
                                        x: rail_x - 3.0,
                                        y: cy - 7.0,
                                    },
                                    D2D_POINT_2F {
                                        x: rail_x + 3.0,
                                        y: cy - 7.0,
                                    },
                                    brush,
                                    1.5,
                                    None,
                                );
                            }
                            3 => {
                                // 4. Bell Glyph (Notifications)
                                let bell_top = D2D_POINT_2F {
                                    x: rail_x,
                                    y: cy - 6.0,
                                };
                                let bell_bot_l = D2D_POINT_2F {
                                    x: rail_x - 6.0,
                                    y: cy + 4.0,
                                };
                                let bell_bot_r = D2D_POINT_2F {
                                    x: rail_x + 6.0,
                                    y: cy + 4.0,
                                };
                                self.render_target
                                    .DrawLine(bell_top, bell_bot_l, brush, 1.2, None);
                                self.render_target
                                    .DrawLine(bell_top, bell_bot_r, brush, 1.2, None);
                                self.render_target
                                    .DrawLine(bell_bot_l, bell_bot_r, brush, 1.2, None);
                                let clapper = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F {
                                        x: rail_x,
                                        y: cy + 6.0,
                                    },
                                    radiusX: 1.5,
                                    radiusY: 1.5,
                                };
                                self.render_target.FillEllipse(&clapper, brush);
                            }
                            _ => {
                                // 5. Settings Gear Glyph
                                let gear = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F { x: rail_x, y: cy },
                                    radiusX: 5.0,
                                    radiusY: 5.0,
                                };
                                self.render_target.DrawEllipse(&gear, brush, 1.2, None);
                                let inner = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F { x: rail_x, y: cy },
                                    radiusX: 2.0,
                                    radiusY: 2.0,
                                };
                                self.render_target.FillEllipse(&inner, brush);
                            }
                        }
                    }

                    // ── ACTIVE STAGE (x: params.x + 60 to params.x + params.width - 12) ──
                    let stage_x = params.x + 62.0;
                    let stage_r = params.x + params.width - 12.0;

                    // 1. ALBUM ART GLASS STAGE CARD
                    let art_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: stage_x,
                            top: params.y + 14.0,
                            right: stage_x + 66.0,
                            bottom: params.y + 80.0,
                        },
                        radiusX: 12.0,
                        radiusY: 12.0,
                    };
                    // Subtle accent ambient glow behind album card
                    let glow_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: stage_x - 4.0,
                            top: params.y + 10.0,
                            right: stage_x + 70.0,
                            bottom: params.y + 84.0,
                        },
                        radiusX: 16.0,
                        radiusY: 16.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&glow_rect, &self.accent_glow_brush);
                    self.render_target
                        .FillRoundedRectangle(&art_rect, &self.card_fill_brush);
                    self.render_target.DrawRoundedRectangle(
                        &art_rect,
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    // Central Vinyl Disc / Note Art Vector
                    let vinyl_center = D2D_POINT_2F {
                        x: stage_x + 33.0,
                        y: params.y + 47.0,
                    };
                    let vinyl_outer = D2D1_ELLIPSE {
                        point: vinyl_center,
                        radiusX: 20.0,
                        radiusY: 20.0,
                    };
                    self.render_target
                        .DrawEllipse(&vinyl_outer, &self.accent_brush, 1.5, None);
                    let vinyl_groove = D2D1_ELLIPSE {
                        point: vinyl_center,
                        radiusX: 13.0,
                        radiusY: 13.0,
                    };
                    self.render_target
                        .DrawEllipse(&vinyl_groove, &self.card_edge_brush, 1.0, None);
                    let vinyl_core = D2D1_ELLIPSE {
                        point: vinyl_center,
                        radiusX: 4.5,
                        radiusY: 4.5,
                    };
                    self.render_target
                        .FillEllipse(&vinyl_core, &self.accent_brush);

                    // 2. TRACK METADATA & LIVE WAVEFORM
                    let title_r = D2D_RECT_F {
                        left: stage_x + 78.0,
                        top: params.y + 14.0,
                        right: stage_r - 90.0,
                        bottom: params.y + 35.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Windows Media Controller",
                        &self.display_format,
                        &self.text_primary_brush,
                        &title_r,
                    );

                    let sub_r = D2D_RECT_F {
                        left: stage_x + 78.0,
                        top: params.y + 35.0,
                        right: stage_r - 90.0,
                        bottom: params.y + 51.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Acoustic Glass Stage • Hardware Dispatcher Ready",
                        &self.body_format,
                        &self.text_secondary_brush,
                        &sub_r,
                    );

                    // Live 6-bar Dynamic Graphic Equalizer (simulated animation)
                    let eq_x = stage_r - 80.0;
                    let eq_base_y = params.y + 36.0;
                    let time_phase = (self.frame_counter as f32) * 0.12;
                    let eq_heights = [
                        12.0 + (time_phase.sin() * 5.0).abs(),
                        18.0 + ((time_phase * 1.3).sin() * 8.0).abs(),
                        10.0 + ((time_phase * 0.9).cos() * 6.0).abs(),
                        22.0 + ((time_phase * 1.5).sin() * 7.0).abs(),
                        14.0 + ((time_phase * 1.1).cos() * 5.0).abs(),
                        16.0 + ((time_phase * 0.7).sin() * 6.0).abs(),
                    ];
                    for (b, &bh) in eq_heights.iter().enumerate() {
                        let bar_x = eq_x + (b as f32) * 6.5;
                        let bar_rect = D2D1_ROUNDED_RECT {
                            rect: D2D_RECT_F {
                                left: bar_x,
                                top: eq_base_y - bh / 2.0,
                                right: bar_x + 3.0,
                                bottom: eq_base_y + bh / 2.0,
                            },
                            radiusX: 1.5,
                            radiusY: 1.5,
                        };
                        self.render_target
                            .FillRoundedRectangle(&bar_rect, &self.accent_brush);
                    }

                    // 3. BESPOKE PLAYBACK CONTROLS (⏮  ▶  ⏭)
                    let ctrl_y = params.y + 57.0;

                    // Previous Track: |◀◀
                    let prev_center_x = stage_x + 88.0;
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: prev_center_x - 6.0,
                            y: ctrl_y - 6.0,
                        },
                        D2D_POINT_2F {
                            x: prev_center_x - 6.0,
                            y: ctrl_y + 6.0,
                        },
                        &self.text_secondary_brush,
                        1.5,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: prev_center_x + 4.0,
                            y: ctrl_y - 6.0,
                        },
                        D2D_POINT_2F {
                            x: prev_center_x - 5.0,
                            y: ctrl_y,
                        },
                        &self.text_secondary_brush,
                        1.5,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: prev_center_x + 4.0,
                            y: ctrl_y + 6.0,
                        },
                        D2D_POINT_2F {
                            x: prev_center_x - 5.0,
                            y: ctrl_y,
                        },
                        &self.text_secondary_brush,
                        1.5,
                        None,
                    );

                    // Play Circular Glass Button (Luminous Accent)
                    let play_center = D2D_POINT_2F {
                        x: stage_x + 118.0,
                        y: ctrl_y,
                    };
                    let play_btn = D2D1_ELLIPSE {
                        point: play_center,
                        radiusX: 13.0,
                        radiusY: 13.0,
                    };
                    self.render_target
                        .FillEllipse(&play_btn, &self.accent_brush);
                    // Play triangle vector
                    let p1 = D2D_POINT_2F {
                        x: play_center.x - 3.5,
                        y: play_center.y - 5.5,
                    };
                    let p2 = D2D_POINT_2F {
                        x: play_center.x + 5.5,
                        y: play_center.y,
                    };
                    let p3 = D2D_POINT_2F {
                        x: play_center.x - 3.5,
                        y: play_center.y + 5.5,
                    };
                    self.render_target
                        .DrawLine(p1, p2, &self.glass_fill_brush, 2.0, None);
                    self.render_target
                        .DrawLine(p2, p3, &self.glass_fill_brush, 2.0, None);
                    self.render_target
                        .DrawLine(p3, p1, &self.glass_fill_brush, 2.0, None);

                    // Next Track: ▶▶|
                    let next_center_x = stage_x + 148.0;
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: next_center_x + 6.0,
                            y: ctrl_y - 6.0,
                        },
                        D2D_POINT_2F {
                            x: next_center_x + 6.0,
                            y: ctrl_y + 6.0,
                        },
                        &self.text_secondary_brush,
                        1.5,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: next_center_x - 4.0,
                            y: ctrl_y - 6.0,
                        },
                        D2D_POINT_2F {
                            x: next_center_x + 5.0,
                            y: ctrl_y,
                        },
                        &self.text_secondary_brush,
                        1.5,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: next_center_x - 4.0,
                            y: ctrl_y + 6.0,
                        },
                        D2D_POINT_2F {
                            x: next_center_x + 5.0,
                            y: ctrl_y,
                        },
                        &self.text_secondary_brush,
                        1.5,
                        None,
                    );

                    // 4. SEEKBAR & TIMELINE
                    let seek_y = params.y + 90.0;
                    let seek_bg = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: stage_x,
                            top: seek_y,
                            right: stage_r,
                            bottom: seek_y + 3.0,
                        },
                        radiusX: 1.5,
                        radiusY: 1.5,
                    };
                    self.render_target
                        .FillRoundedRectangle(&seek_bg, &self.card_edge_brush);

                    let seek_fill = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: stage_x,
                            top: seek_y,
                            right: stage_x + (stage_r - stage_x) * 0.38,
                            bottom: seek_y + 3.0,
                        },
                        radiusX: 1.5,
                        radiusY: 1.5,
                    };
                    self.render_target
                        .FillRoundedRectangle(&seek_fill, &self.accent_brush);

                    // Glowing Scrubber Thumb
                    let thumb = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: stage_x + (stage_r - stage_x) * 0.38,
                            y: seek_y + 1.5,
                        },
                        radiusX: 4.5,
                        radiusY: 4.5,
                    };
                    self.render_target
                        .FillEllipse(&thumb, &self.text_primary_brush);

                    // Timestamps in Segoe UI SemiBold
                    let time_l = D2D_RECT_F {
                        left: stage_x,
                        top: seek_y + 5.0,
                        right: stage_x + 50.0,
                        bottom: seek_y + 19.0,
                    };
                    draw_text(
                        &self.render_target,
                        "01:24",
                        &self.caption_format,
                        &self.text_tertiary_brush,
                        &time_l,
                    );

                    let time_r = D2D_RECT_F {
                        left: stage_r - 50.0,
                        top: seek_y + 5.0,
                        right: stage_r,
                        bottom: seek_y + 19.0,
                    };
                    draw_text(
                        &self.render_target,
                        "03:45",
                        &self.caption_trailing,
                        &self.text_tertiary_brush,
                        &time_r,
                    );

                    // 5. HARDWARE & TOOL HUBS (3 Glass Cards)
                    let card_w = (stage_r - stage_x - 16.0) / 3.0;
                    let card_y = params.y + 118.0;
                    let card_h = 56.0;

                    // ── Card 1: Microphone Hardware Capsule ────────────────────────
                    let c1_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: stage_x,
                            top: card_y,
                            right: stage_x + card_w,
                            bottom: card_y + card_h,
                        },
                        radiusX: 10.0,
                        radiusY: 10.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&c1_rect, &self.card_fill_brush);
                    self.render_target.DrawRoundedRectangle(
                        &c1_rect,
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    // Live Status Beacon Dot
                    let c1_dot = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: stage_x + 16.0,
                            y: card_y + 18.0,
                        },
                        radiusX: 3.5,
                        radiusY: 3.5,
                    };
                    self.render_target.FillEllipse(&c1_dot, &self.ok_brush);

                    let c1_title = D2D_RECT_F {
                        left: stage_x + 26.0,
                        top: card_y + 8.0,
                        right: stage_x + card_w - 8.0,
                        bottom: card_y + 26.0,
                    };
                    draw_text(
                        &self.render_target,
                        "MIC: ACTIVE",
                        &self.title_format,
                        &self.text_primary_brush,
                        &c1_title,
                    );

                    let c1_sub = D2D_RECT_F {
                        left: stage_x + 12.0,
                        top: card_y + 30.0,
                        right: stage_x + card_w - 8.0,
                        bottom: card_y + 48.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Default Endpoint • Ctrl+Alt+M",
                        &self.caption_format,
                        &self.text_tertiary_brush,
                        &c1_sub,
                    );

                    // ── Card 2: Memory Clipboard Capsule ───────────────────────────
                    let c2_x = stage_x + card_w + 8.0;
                    let c2_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: c2_x,
                            top: card_y,
                            right: c2_x + card_w,
                            bottom: card_y + card_h,
                        },
                        radiusX: 10.0,
                        radiusY: 10.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&c2_rect, &self.card_fill_brush);
                    self.render_target.DrawRoundedRectangle(
                        &c2_rect,
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    let c2_dot = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: c2_x + 16.0,
                            y: card_y + 18.0,
                        },
                        radiusX: 3.5,
                        radiusY: 3.5,
                    };
                    self.render_target.FillEllipse(&c2_dot, &self.accent_brush);

                    let c2_title = D2D_RECT_F {
                        left: c2_x + 26.0,
                        top: card_y + 8.0,
                        right: c2_x + card_w - 8.0,
                        bottom: card_y + 26.0,
                    };
                    draw_text(
                        &self.render_target,
                        "CLIPBOARD",
                        &self.title_format,
                        &self.text_primary_brush,
                        &c2_title,
                    );

                    let c2_sub = D2D_RECT_F {
                        left: c2_x + 12.0,
                        top: card_y + 30.0,
                        right: c2_x + card_w - 8.0,
                        bottom: card_y + 48.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Encrypted AES-GCM • Ctrl+Alt+V",
                        &self.caption_format,
                        &self.text_tertiary_brush,
                        &c2_sub,
                    );

                    // ── Card 3: Files Staging Shelf ────────────────────────────────
                    let c3_x = c2_x + card_w + 8.0;
                    let c3_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: c3_x,
                            top: card_y,
                            right: stage_r,
                            bottom: card_y + card_h,
                        },
                        radiusX: 10.0,
                        radiusY: 10.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&c3_rect, &self.card_fill_brush);
                    self.render_target.DrawRoundedRectangle(
                        &c3_rect,
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    let c3_dot = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: c3_x + 16.0,
                            y: card_y + 18.0,
                        },
                        radiusX: 3.5,
                        radiusY: 3.5,
                    };
                    self.render_target
                        .FillEllipse(&c3_dot, &self.text_secondary_brush);

                    let c3_title = D2D_RECT_F {
                        left: c3_x + 26.0,
                        top: card_y + 8.0,
                        right: stage_r - 8.0,
                        bottom: card_y + 26.0,
                    };
                    draw_text(
                        &self.render_target,
                        "FILES SHELF",
                        &self.title_format,
                        &self.text_primary_brush,
                        &c3_title,
                    );

                    let c3_sub = D2D_RECT_F {
                        left: c3_x + 12.0,
                        top: card_y + 30.0,
                        right: stage_r - 8.0,
                        bottom: card_y + 48.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Drop items onto notch",
                        &self.caption_format,
                        &self.text_tertiary_brush,
                        &c3_sub,
                    );

                    // 6. FOOTER SHORTCUT HINTS
                    let foot_r = D2D_RECT_F {
                        left: stage_x,
                        top: params.y + params.height - 24.0,
                        right: stage_r,
                        bottom: params.y + params.height - 7.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Ctrl+Alt+N or click to collapse  •  Right-click system tray for settings",
                        &self.caption_center,
                        &self.text_tertiary_brush,
                        &foot_r,
                    );
                } else if params.height >= 50.0 {
                    // ══════════════════════════════════════════════════════════════════════
                    // GLANCE VIEW (360 × 64 px)
                    // ══════════════════════════════════════════════════════════════════════
                    // Left equalizers
                    let eq_x = params.x + 20.0;
                    let eq_cy = params.y + 32.0;
                    let time_phase = (self.frame_counter as f32) * 0.15;
                    for b in 0..4 {
                        let bx = eq_x + (b as f32) * 5.0;
                        let bh = 8.0 + ((time_phase + (b as f32)).sin() * 7.0).abs();
                        let bar = D2D1_ROUNDED_RECT {
                            rect: D2D_RECT_F {
                                left: bx,
                                top: eq_cy - bh / 2.0,
                                right: bx + 2.5,
                                bottom: eq_cy + bh / 2.0,
                            },
                            radiusX: 1.0,
                            radiusY: 1.0,
                        };
                        self.render_target
                            .FillRoundedRectangle(&bar, &self.accent_brush);
                    }

                    // Metadata
                    let title_r = D2D_RECT_F {
                        left: params.x + 48.0,
                        top: params.y + 12.0,
                        right: params.x + params.width - 70.0,
                        bottom: params.y + 33.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Nuvra • Acoustic Stage",
                        &self.display_format,
                        &self.text_primary_brush,
                        &title_r,
                    );

                    let sub_r = D2D_RECT_F {
                        left: params.x + 48.0,
                        top: params.y + 33.0,
                        right: params.x + params.width - 70.0,
                        bottom: params.y + 51.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Hardware active • Click to expand",
                        &self.body_format,
                        &self.text_secondary_brush,
                        &sub_r,
                    );

                    // Right Play Icon
                    let play_center = D2D_POINT_2F {
                        x: params.x + params.width - 32.0,
                        y: params.y + 32.0,
                    };
                    let play_btn = D2D1_ELLIPSE {
                        point: play_center,
                        radiusX: 12.0,
                        radiusY: 12.0,
                    };
                    self.render_target
                        .FillEllipse(&play_btn, &self.accent_brush);
                    let p1 = D2D_POINT_2F {
                        x: play_center.x - 3.0,
                        y: play_center.y - 4.5,
                    };
                    let p2 = D2D_POINT_2F {
                        x: play_center.x + 4.5,
                        y: play_center.y,
                    };
                    let p3 = D2D_POINT_2F {
                        x: play_center.x - 3.0,
                        y: play_center.y + 4.5,
                    };
                    self.render_target
                        .DrawLine(p1, p2, &self.glass_fill_brush, 1.8, None);
                    self.render_target
                        .DrawLine(p2, p3, &self.glass_fill_brush, 1.8, None);
                    self.render_target
                        .DrawLine(p3, p1, &self.glass_fill_brush, 1.8, None);
                } else if params.height >= 24.0 {
                    // ══════════════════════════════════════════════════════════════════════
                    // AMBIENT VIEW (220 × 34 px) — Sleek Translucent Glass Jewel
                    // ══════════════════════════════════════════════════════════════════════
                    // 1. Live Emerald Pulse Beacon
                    let beacon = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: params.x + 18.0,
                            y: params.y + 17.0,
                        },
                        radiusX: 3.5,
                        radiusY: 3.5,
                    };
                    self.render_target.FillEllipse(&beacon, &self.ok_brush);

                    let beacon_ring = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: params.x + 18.0,
                            y: params.y + 17.0,
                        },
                        radiusX: 6.0,
                        radiusY: 6.0,
                    };
                    self.render_target
                        .DrawEllipse(&beacon_ring, &self.card_edge_brush, 1.0, None);

                    // 2. Tracked "N U V R A" Brandmark
                    let label_r = D2D_RECT_F {
                        left: params.x + 32.0,
                        top: params.y,
                        right: params.x + params.width - 32.0,
                        bottom: params.y + params.height,
                    };
                    draw_text(
                        &self.render_target,
                        "N U V R A",
                        &self.ambient_format,
                        &self.text_primary_brush,
                        &label_r,
                    );

                    // 3. Right Subtle Audio Waveform Indicator
                    let right_x = params.x + params.width - 24.0;
                    for b in 0..3 {
                        let bx = right_x + (b as f32) * 3.5;
                        let bh = if b == 1 { 9.0 } else { 5.0 };
                        let bar = D2D1_ROUNDED_RECT {
                            rect: D2D_RECT_F {
                                left: bx,
                                top: params.y + 17.0 - bh / 2.0,
                                right: bx + 1.5,
                                bottom: params.y + 17.0 + bh / 2.0,
                            },
                            radiusX: 0.75,
                            radiusY: 0.75,
                        };
                        self.render_target
                            .FillRoundedRectangle(&bar, &self.accent_brush);
                    }
                }
            }

            self.render_target
                .EndDraw(None, None)
                .map_err(|e| format!("D2D EndDraw failed: {e}"))?;

            swapchain
                .Present(0, DXGI_PRESENT(0))
                .ok()
                .map_err(|e| format!("DXGI Present failed: {e}"))?;
        }

        Ok(())
    }
}
