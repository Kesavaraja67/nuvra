// nuvra-platform-win/src/renderer.rs
// Direct2D hardware-accelerated glassmorphic renderer for Nuvra Notch (D-STK-03, D-PERF-03).
// Implements production-grade glassmorphism with DWM blur integration and non-colliding layout.
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
    badge_format: IDWriteTextFormat,
    caption_format: IDWriteTextFormat,
    _caption_trailing: IDWriteTextFormat,
    caption_center: IDWriteTextFormat,
    ambient_format: IDWriteTextFormat,
    frame_counter: u32,
    pub active_tab: usize,
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

            // 1. Frosted Glass Multi-Stop Linear Gradient Fill (True Translucency with DWM blur bleed)
            let glass_stops = [
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: D2D1_COLOR_F {
                        r: 0.16,
                        g: 0.20,
                        b: 0.36,
                        a: 0.26, // 26% alpha: background wallpaper clearly shines through
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 0.45,
                    color: D2D1_COLOR_F {
                        r: 0.08,
                        g: 0.11,
                        b: 0.22,
                        a: 0.34,
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: D2D1_COLOR_F {
                        r: 0.04,
                        g: 0.06,
                        b: 0.14,
                        a: 0.44,
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
                        a: 0.75, // Bright specular glass catchlight
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 0.35,
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.25,
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.08,
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

            // Display title: Segoe UI, Bold 14.0 pt
            let display_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    14.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat display failed: {e}"))?;
            let _ = display_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = display_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Subtitle: Segoe UI, SemiBold 11.5 pt
            let title_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    11.5,
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

            // Badge format: Segoe UI, Bold 9.0 pt (uppercase tracking)
            let badge_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    9.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat badge failed: {e}"))?;
            let _ = badge_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = badge_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

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

            // Caption center: Segoe UI, Normal 9.5 pt
            let caption_center = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    9.5,
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
                badge_format,
                caption_format,
                _caption_trailing: caption_trailing,
                caption_center,
                ambient_format,
                frame_counter: 0,
                active_tab: 0,
            })
        }
    }

    pub fn active_tab(&self) -> usize {
        self.active_tab
    }

    pub fn set_active_tab(&mut self, tab: usize) {
        self.active_tab = tab.min(4);
    }

    /// Handles a click on the notch in window coordinates. Returns true if an interactive tab was selected.
    pub fn handle_click(&mut self, x: f32, y: f32) -> bool {
        // Top navigation bar icons:
        // [Globe: 180, Bell: 215, Folder: 250, Clipboard: 285, Widgets: 320]
        if (4.0..=34.0).contains(&y) {
            if (x - 180.0).abs() <= 16.0 {
                self.active_tab = 0;
                return true;
            } else if (x - 215.0).abs() <= 16.0 {
                self.active_tab = 1;
                return true;
            } else if (x - 250.0).abs() <= 16.0 {
                self.active_tab = 2;
                return true;
            } else if (x - 285.0).abs() <= 16.0 {
                self.active_tab = 3;
                return true;
            } else if (x - 320.0).abs() <= 16.0 {
                self.active_tab = 4;
                return true;
            }
        }
        false
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
                    a: 0.06,
                });
                self.card_edge_brush.SetColor(&D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.15,
                });
                self.specular_brush.SetColor(&D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.55,
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
                    .SetColor(&to_color(params.accent_color, 0.22));
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

                // 1. Fill base frosted translucent glass gradient surface
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
                    // Modular Frosted Glass Notch (Pure Glassmorphism, Zero Skeuomorphism)
                    // Matches user reference layout with top horizontal icon navigation
                    // ══════════════════════════════════════════════════════════════════════
                    let stage_x = params.x + 20.0;
                    let stage_r = params.x + params.width - 20.0;

                    // ── TOP HORIZONTAL ICON NAVIGATION STRIP (y: params.y + 10.0 to 30.0) ───
                    // Icons: 0: 🌐 Sites/Media, 1: 🔔 Alerts, 2: 📁 Shelf, 3: 📋 Clipboard, 4: 🔲 Widgets, 5: 🎚️ Sound, 6: ⚙️ Settings
                    let nav_icons_x = [180.0, 215.0, 250.0, 285.0, 320.0, 355.0, 395.0];
                    let nav_y = params.y + 16.0;

                    // Subtle divider before settings
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: params.x + 375.0,
                            y: nav_y - 6.0,
                        },
                        D2D_POINT_2F {
                            x: params.x + 375.0,
                            y: nav_y + 6.0,
                        },
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    for (idx, &ix_rel) in nav_icons_x.iter().enumerate() {
                        let ix = params.x + ix_rel;
                        let is_active = idx == self.active_tab;
                        let brush = if is_active {
                            &self.text_primary_brush
                        } else {
                            &self.text_tertiary_brush
                        };

                        match idx {
                            0 => {
                                // 0: Globe 🌐 (Sites & Audio)
                                let globe = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F { x: ix, y: nav_y },
                                    radiusX: 5.5,
                                    radiusY: 5.5,
                                };
                                self.render_target.DrawEllipse(&globe, brush, 1.2, None);
                                self.render_target.DrawLine(
                                    D2D_POINT_2F {
                                        x: ix - 5.5,
                                        y: nav_y,
                                    },
                                    D2D_POINT_2F {
                                        x: ix + 5.5,
                                        y: nav_y,
                                    },
                                    brush,
                                    1.0,
                                    None,
                                );
                                let inner_g = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F { x: ix, y: nav_y },
                                    radiusX: 2.8,
                                    radiusY: 5.5,
                                };
                                self.render_target.DrawEllipse(&inner_g, brush, 1.0, None);
                            }
                            1 => {
                                // 1: Bell 🔔 (Notifications)
                                let b_top = D2D_POINT_2F {
                                    x: ix,
                                    y: nav_y - 5.0,
                                };
                                let b_bl = D2D_POINT_2F {
                                    x: ix - 4.5,
                                    y: nav_y + 3.0,
                                };
                                let b_br = D2D_POINT_2F {
                                    x: ix + 4.5,
                                    y: nav_y + 3.0,
                                };
                                self.render_target.DrawLine(b_top, b_bl, brush, 1.2, None);
                                self.render_target.DrawLine(b_top, b_br, brush, 1.2, None);
                                self.render_target.DrawLine(b_bl, b_br, brush, 1.2, None);
                                let clap = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F {
                                        x: ix,
                                        y: nav_y + 4.5,
                                    },
                                    radiusX: 1.5,
                                    radiusY: 1.5,
                                };
                                self.render_target.FillEllipse(&clap, brush);
                            }
                            2 => {
                                // 2: Folder Tray 📁 (Shelf)
                                let tray = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: ix - 5.5,
                                        top: nav_y - 3.5,
                                        right: ix + 5.5,
                                        bottom: nav_y + 4.5,
                                    },
                                    radiusX: 1.5,
                                    radiusY: 1.5,
                                };
                                self.render_target
                                    .DrawRoundedRectangle(&tray, brush, 1.2, None);
                                self.render_target.DrawLine(
                                    D2D_POINT_2F {
                                        x: ix - 5.5,
                                        y: nav_y - 4.5,
                                    },
                                    D2D_POINT_2F {
                                        x: ix - 1.5,
                                        y: nav_y - 4.5,
                                    },
                                    brush,
                                    1.2,
                                    None,
                                );
                            }
                            3 => {
                                // 3: Clipboard 📋 (Memory Vault)
                                let board = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: ix - 4.5,
                                        top: nav_y - 4.5,
                                        right: ix + 4.5,
                                        bottom: nav_y + 5.0,
                                    },
                                    radiusX: 1.5,
                                    radiusY: 1.5,
                                };
                                self.render_target
                                    .DrawRoundedRectangle(&board, brush, 1.2, None);
                                self.render_target.DrawLine(
                                    D2D_POINT_2F {
                                        x: ix - 2.5,
                                        y: nav_y - 5.5,
                                    },
                                    D2D_POINT_2F {
                                        x: ix + 2.5,
                                        y: nav_y - 5.5,
                                    },
                                    brush,
                                    1.2,
                                    None,
                                );
                            }
                            4 => {
                                // 4: 2x2 Widgets Grid 🔲
                                for row in 0..2 {
                                    for col in 0..2 {
                                        let sq = D2D1_ROUNDED_RECT {
                                            rect: D2D_RECT_F {
                                                left: ix - 4.5 + (col as f32) * 5.0,
                                                top: nav_y - 4.5 + (row as f32) * 5.0,
                                                right: ix - 1.0 + (col as f32) * 5.0,
                                                bottom: nav_y - 1.0 + (row as f32) * 5.0,
                                            },
                                            radiusX: 0.8,
                                            radiusY: 0.8,
                                        };
                                        self.render_target.FillRoundedRectangle(&sq, brush);
                                    }
                                }
                            }
                            5 => {
                                // 5: Sound Sliders 🎚️
                                for s in 0..3 {
                                    let sx = ix - 4.5 + (s as f32) * 4.5;
                                    self.render_target.DrawLine(
                                        D2D_POINT_2F {
                                            x: sx,
                                            y: nav_y - 4.5,
                                        },
                                        D2D_POINT_2F {
                                            x: sx,
                                            y: nav_y + 4.5,
                                        },
                                        brush,
                                        1.0,
                                        None,
                                    );
                                    let dot_y = if s == 1 { nav_y + 1.5 } else { nav_y - 1.5 };
                                    let s_dot = D2D1_ELLIPSE {
                                        point: D2D_POINT_2F { x: sx, y: dot_y },
                                        radiusX: 1.5,
                                        radiusY: 1.5,
                                    };
                                    self.render_target.FillEllipse(&s_dot, brush);
                                }
                            }
                            _ => {
                                // 6: Settings Gear ⚙️
                                let gear = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F { x: ix, y: nav_y },
                                    radiusX: 4.5,
                                    radiusY: 4.5,
                                };
                                self.render_target.DrawEllipse(&gear, brush, 1.2, None);
                                let inner = D2D1_ELLIPSE {
                                    point: D2D_POINT_2F { x: ix, y: nav_y },
                                    radiusX: 1.8,
                                    radiusY: 1.8,
                                };
                                self.render_target.FillEllipse(&inner, brush);
                            }
                        }

                        // Active Indicator Dot beneath active icon (Reference screenshot design)
                        if is_active {
                            let dot = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: ix,
                                    y: nav_y + 10.0,
                                },
                                radiusX: 1.5,
                                radiusY: 1.5,
                            };
                            self.render_target
                                .FillEllipse(&dot, &self.text_primary_brush);
                        }
                    }

                    // ── TAB CONTENT RENDERING (y: params.y + 36.0 to 184.0) ───────────────
                    match self.active_tab {
                        1 => {
                            // ══════════════════════════════════════════════════════════════
                            // TAB 1: NOTIFICATIONS (Matches Reference Image 98121fa1.jpg)
                            // ══════════════════════════════════════════════════════════════
                            let bell_cx = params.x + params.width / 2.0;
                            let bell_cy = params.y + 88.0;

                            // Minimalist vector bell
                            let b_top = D2D_POINT_2F {
                                x: bell_cx,
                                y: bell_cy - 12.0,
                            };
                            let b_bl = D2D_POINT_2F {
                                x: bell_cx - 11.0,
                                y: bell_cy + 8.0,
                            };
                            let b_br = D2D_POINT_2F {
                                x: bell_cx + 11.0,
                                y: bell_cy + 8.0,
                            };
                            self.render_target.DrawLine(
                                b_top,
                                b_bl,
                                &self.text_secondary_brush,
                                1.8,
                                None,
                            );
                            self.render_target.DrawLine(
                                b_top,
                                b_br,
                                &self.text_secondary_brush,
                                1.8,
                                None,
                            );
                            self.render_target.DrawLine(
                                b_bl,
                                b_br,
                                &self.text_secondary_brush,
                                1.8,
                                None,
                            );
                            let clap = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: bell_cx,
                                    y: bell_cy + 12.0,
                                },
                                radiusX: 3.0,
                                radiusY: 3.0,
                            };
                            self.render_target
                                .FillEllipse(&clap, &self.text_secondary_brush);

                            // Empty state text matching reference
                            let title_r = D2D_RECT_F {
                                left: stage_x,
                                top: params.y + 114.0,
                                right: stage_r,
                                bottom: params.y + 134.0,
                            };
                            draw_text(
                                &self.render_target,
                                "Nothing yet",
                                &self.title_format,
                                &self.text_primary_brush,
                                &title_r,
                            );

                            let sub_r = D2D_RECT_F {
                                left: stage_x,
                                top: params.y + 136.0,
                                right: stage_r,
                                bottom: params.y + 154.0,
                            };
                            draw_text(
                                &self.render_target,
                                "What the island says lands here. Click one to open it.",
                                &self.body_format,
                                &self.text_secondary_brush,
                                &sub_r,
                            );
                        }
                        2 => {
                            // ══════════════════════════════════════════════════════════════
                            // TAB 2: FILE SHELF (Staging Drop Zone)
                            // ══════════════════════════════════════════════════════════════
                            let shelf_rect = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x,
                                    top: params.y + 40.0,
                                    right: stage_r,
                                    bottom: params.y + 176.0,
                                },
                                radiusX: 14.0,
                                radiusY: 14.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&shelf_rect, &self.card_fill_brush);
                            self.render_target.DrawRoundedRectangle(
                                &shelf_rect,
                                &self.card_edge_brush,
                                1.0,
                                None,
                            );

                            let cx = params.x + params.width / 2.0;
                            // Clean Folder Glyph
                            let f_body = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: cx - 18.0,
                                    top: params.y + 74.0,
                                    right: cx + 18.0,
                                    bottom: params.y + 98.0,
                                },
                                radiusX: 3.0,
                                radiusY: 3.0,
                            };
                            self.render_target.DrawRoundedRectangle(
                                &f_body,
                                &self.accent_brush,
                                1.5,
                                None,
                            );
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: cx - 18.0,
                                    y: params.y + 70.0,
                                },
                                D2D_POINT_2F {
                                    x: cx - 6.0,
                                    y: params.y + 70.0,
                                },
                                &self.accent_brush,
                                1.5,
                                None,
                            );

                            let title_r = D2D_RECT_F {
                                left: stage_x,
                                top: params.y + 108.0,
                                right: stage_r,
                                bottom: params.y + 128.0,
                            };
                            draw_text(
                                &self.render_target,
                                "Drop files here to stage or transfer",
                                &self.title_format,
                                &self.text_primary_brush,
                                &title_r,
                            );

                            let sub_r = D2D_RECT_F {
                                left: stage_x,
                                top: params.y + 130.0,
                                right: stage_r,
                                bottom: params.y + 148.0,
                            };
                            draw_text(
                                &self.render_target,
                                "Staged items are held in memory for 1-click sharing to companion",
                                &self.body_format,
                                &self.text_secondary_brush,
                                &sub_r,
                            );

                            let status_r = D2D_RECT_F {
                                left: stage_x,
                                top: params.y + 152.0,
                                right: stage_r,
                                bottom: params.y + 168.0,
                            };
                            draw_text(
                                &self.render_target,
                                "● FILE SHELF READY • 0 ITEMS STAGED",
                                &self.badge_format,
                                &self.ok_brush,
                                &status_r,
                            );
                        }
                        3 => {
                            // ══════════════════════════════════════════════════════════════
                            // TAB 3: CLIPBOARD VAULT (Encrypted History)
                            // ══════════════════════════════════════════════════════════════
                            let clip_items = [
                                (
                                    "https://github.com/Kesavaraja67/nuvra",
                                    "Copied from Browser • Click to Copy",
                                    "[ RECENT ]",
                                ),
                                (
                                    "cargo run -p nuvra-app",
                                    "Copied from Terminal • 2m ago",
                                    "[ CODE ]",
                                ),
                                (
                                    "Microphone Endpoint Active (Ctrl+Alt+M)",
                                    "Hardware state snippet",
                                    "[ HARDWARE ]",
                                ),
                            ];

                            for (i, (val, sub, badge)) in clip_items.iter().enumerate() {
                                let cy = params.y + 40.0 + (i as f32) * 44.0;
                                let c_rect = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: stage_x,
                                        top: cy,
                                        right: stage_r,
                                        bottom: cy + 38.0,
                                    },
                                    radiusX: 10.0,
                                    radiusY: 10.0,
                                };
                                self.render_target
                                    .FillRoundedRectangle(&c_rect, &self.card_fill_brush);
                                self.render_target.DrawRoundedRectangle(
                                    &c_rect,
                                    &self.card_edge_brush,
                                    1.0,
                                    None,
                                );

                                let v_rect = D2D_RECT_F {
                                    left: stage_x + 12.0,
                                    top: cy + 4.0,
                                    right: stage_r - 90.0,
                                    bottom: cy + 20.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    val,
                                    &self.body_format,
                                    &self.text_primary_brush,
                                    &v_rect,
                                );

                                let s_rect = D2D_RECT_F {
                                    left: stage_x + 12.0,
                                    top: cy + 20.0,
                                    right: stage_r - 90.0,
                                    bottom: cy + 34.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    sub,
                                    &self.caption_format,
                                    &self.text_secondary_brush,
                                    &s_rect,
                                );

                                let b_rect = D2D_RECT_F {
                                    left: stage_r - 85.0,
                                    top: cy + 10.0,
                                    right: stage_r - 10.0,
                                    bottom: cy + 28.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    badge,
                                    &self.badge_format,
                                    &self.accent_brush,
                                    &b_rect,
                                );
                            }
                        }
                        4 => {
                            // ══════════════════════════════════════════════════════════════
                            // TAB 4: WIDGETS (Matches Reference Image a7d3eca6.jpg 2x2 grid)
                            // ══════════════════════════════════════════════════════════════
                            let qw = (stage_r - stage_x - 12.0) / 2.0;
                            let qh = 66.0;

                            // 1. Top-Left: Date & Calendar
                            let w1 = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x,
                                    top: params.y + 38.0,
                                    right: stage_x + qw,
                                    bottom: params.y + 38.0 + qh,
                                },
                                radiusX: 12.0,
                                radiusY: 12.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&w1, &self.card_fill_brush);
                            self.render_target.DrawRoundedRectangle(
                                &w1,
                                &self.card_edge_brush,
                                1.0,
                                None,
                            );

                            let d_big = D2D_RECT_F {
                                left: stage_x + 12.0,
                                top: params.y + 40.0,
                                right: stage_x + 50.0,
                                bottom: params.y + 70.0,
                            };
                            draw_text(
                                &self.render_target,
                                "23",
                                &self.display_format,
                                &self.text_primary_brush,
                                &d_big,
                            );

                            let d_meta = D2D_RECT_F {
                                left: stage_x + 48.0,
                                top: params.y + 42.0,
                                right: stage_x + qw - 8.0,
                                bottom: params.y + 58.0,
                            };
                            draw_text(
                                &self.render_target,
                                "MON  NOV  •  Week 48",
                                &self.badge_format,
                                &self.text_secondary_brush,
                                &d_meta,
                            );

                            let strip_r = D2D_RECT_F {
                                left: stage_x + 12.0,
                                top: params.y + 76.0,
                                right: stage_x + qw - 12.0,
                                bottom: params.y + 98.0,
                            };
                            draw_text(
                                &self.render_target,
                                "S   M   T   W   T   F   S",
                                &self.caption_format,
                                &self.text_tertiary_brush,
                                &strip_r,
                            );

                            // Highlight active day in capsule
                            let day_cap = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x + 36.0,
                                    top: params.y + 74.0,
                                    right: stage_x + 56.0,
                                    bottom: params.y + 98.0,
                                },
                                radiusX: 6.0,
                                radiusY: 6.0,
                            };
                            self.render_target.DrawRoundedRectangle(
                                &day_cap,
                                &self.accent_brush,
                                1.2,
                                None,
                            );

                            // 2. Top-Right: Focus Timer (25 + 5 min)
                            let w2 = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x + qw + 12.0,
                                    top: params.y + 38.0,
                                    right: stage_r,
                                    bottom: params.y + 38.0 + qh,
                                },
                                radiusX: 12.0,
                                radiusY: 12.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&w2, &self.card_fill_brush);
                            self.render_target.DrawRoundedRectangle(
                                &w2,
                                &self.card_edge_brush,
                                1.0,
                                None,
                            );

                            let timer_dial = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: stage_x + qw + 34.0,
                                    y: params.y + 71.0,
                                },
                                radiusX: 18.0,
                                radiusY: 18.0,
                            };
                            self.render_target.DrawEllipse(
                                &timer_dial,
                                &self.card_edge_brush,
                                2.0,
                                None,
                            );
                            let t_text = D2D_RECT_F {
                                left: stage_x + qw + 16.0,
                                top: params.y + 61.0,
                                right: stage_x + qw + 52.0,
                                bottom: params.y + 81.0,
                            };
                            draw_text(
                                &self.render_target,
                                "25:00",
                                &self.badge_format,
                                &self.text_primary_brush,
                                &t_text,
                            );

                            let t_label = D2D_RECT_F {
                                left: stage_x + qw + 60.0,
                                top: params.y + 44.0,
                                right: stage_r - 8.0,
                                bottom: params.y + 60.0,
                            };
                            draw_text(
                                &self.render_target,
                                "Focus timer",
                                &self.title_format,
                                &self.text_primary_brush,
                                &t_label,
                            );

                            let t_sub = D2D_RECT_F {
                                left: stage_x + qw + 60.0,
                                top: params.y + 60.0,
                                right: stage_r - 8.0,
                                bottom: params.y + 74.0,
                            };
                            draw_text(
                                &self.render_target,
                                "25 + 5 min",
                                &self.caption_format,
                                &self.text_secondary_brush,
                                &t_sub,
                            );

                            // Mini play button
                            let play_btn = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: stage_x + qw + 72.0,
                                    y: params.y + 88.0,
                                },
                                radiusX: 8.0,
                                radiusY: 8.0,
                            };
                            self.render_target
                                .FillEllipse(&play_btn, &self.text_primary_brush);

                            // 3. Bottom-Left: Weather Widget
                            let w3 = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x,
                                    top: params.y + 112.0,
                                    right: stage_x + qw,
                                    bottom: params.y + 112.0 + qh,
                                },
                                radiusX: 12.0,
                                radiusY: 12.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&w3, &self.card_fill_brush);
                            self.render_target.DrawRoundedRectangle(
                                &w3,
                                &self.card_edge_brush,
                                1.0,
                                None,
                            );

                            let temp_r = D2D_RECT_F {
                                left: stage_x + 12.0,
                                top: params.y + 116.0,
                                right: stage_x + 60.0,
                                bottom: params.y + 148.0,
                            };
                            draw_text(
                                &self.render_target,
                                "29°",
                                &self.display_format,
                                &self.text_primary_brush,
                                &temp_r,
                            );

                            let w_sub = D2D_RECT_F {
                                left: stage_x + 12.0,
                                top: params.y + 152.0,
                                right: stage_x + qw - 8.0,
                                bottom: params.y + 170.0,
                            };
                            draw_text(
                                &self.render_target,
                                "Cloudy   •   H 29°  L 21°",
                                &self.body_format,
                                &self.text_secondary_brush,
                                &w_sub,
                            );

                            // Cloud glyph
                            let cloud_cx = stage_x + qw - 30.0;
                            let cloud_cy = params.y + 134.0;
                            let cl_c = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: cloud_cx,
                                    y: cloud_cy,
                                },
                                radiusX: 9.0,
                                radiusY: 6.0,
                            };
                            self.render_target.DrawEllipse(
                                &cl_c,
                                &self.text_secondary_brush,
                                1.2,
                                None,
                            );

                            // 4. Bottom-Right: System Performance Monitor
                            let w4 = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x + qw + 12.0,
                                    top: params.y + 112.0,
                                    right: stage_r,
                                    bottom: params.y + 112.0 + qh,
                                },
                                radiusX: 12.0,
                                radiusY: 12.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&w4, &self.card_fill_brush);
                            self.render_target.DrawRoundedRectangle(
                                &w4,
                                &self.card_edge_brush,
                                1.0,
                                None,
                            );

                            let sys_r1 = D2D_RECT_F {
                                left: stage_x + qw + 20.0,
                                top: params.y + 116.0,
                                right: stage_r - 12.0,
                                bottom: params.y + 132.0,
                            };
                            draw_text(
                                &self.render_target,
                                "CPU   18%   62°",
                                &self.caption_format,
                                &self.text_primary_brush,
                                &sys_r1,
                            );

                            let sys_r2 = D2D_RECT_F {
                                left: stage_x + qw + 20.0,
                                top: params.y + 132.0,
                                right: stage_r - 12.0,
                                bottom: params.y + 148.0,
                            };
                            draw_text(
                                &self.render_target,
                                "GPU   10%   58°",
                                &self.caption_format,
                                &self.text_secondary_brush,
                                &sys_r2,
                            );

                            let sys_r3 = D2D_RECT_F {
                                left: stage_x + qw + 20.0,
                                top: params.y + 148.0,
                                right: stage_r - 12.0,
                                bottom: params.y + 164.0,
                            };
                            draw_text(
                                &self.render_target,
                                "RAM   79%   •   ↓ 522 B/s  ↑ 294 B/s",
                                &self.caption_format,
                                &self.accent_brush,
                                &sys_r3,
                            );

                            // Center Bell Floating Button (Reference image design)
                            let mid_bell = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: stage_x + qw + 6.0,
                                    y: params.y + 109.0,
                                },
                                radiusX: 14.0,
                                radiusY: 14.0,
                            };
                            self.render_target
                                .FillEllipse(&mid_bell, &self.card_fill_brush);
                            self.render_target.DrawEllipse(
                                &mid_bell,
                                &self.card_edge_brush,
                                1.2,
                                None,
                            );
                            let mb_dot = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: stage_x + qw + 6.0,
                                    y: params.y + 109.0,
                                },
                                radiusX: 3.5,
                                radiusY: 3.5,
                            };
                            self.render_target
                                .FillEllipse(&mb_dot, &self.text_primary_brush);
                        }
                        _ => {
                            // ══════════════════════════════════════════════════════════════
                            // TAB 0 (DEFAULT): SITES & AUDIO (Matches Reference Image c61565b9.jpg)
                            // ══════════════════════════════════════════════════════════════

                            // 1. Now Playing Glass Player Tile (y: params.y + 36.0 to 98.0)
                            let p_tile = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x,
                                    top: params.y + 36.0,
                                    right: stage_r,
                                    bottom: params.y + 98.0,
                                },
                                radiusX: 12.0,
                                radiusY: 12.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&p_tile, &self.card_fill_brush);
                            self.render_target.DrawRoundedRectangle(
                                &p_tile,
                                &self.card_edge_brush,
                                1.0,
                                None,
                            );

                            // Left music tile (38x38 px)
                            let art_box = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_x + 10.0,
                                    top: params.y + 44.0,
                                    right: stage_x + 48.0,
                                    bottom: params.y + 82.0,
                                },
                                radiusX: 8.0,
                                radiusY: 8.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&art_box, &self.accent_brush);
                            // White music waveform icon inside art box
                            for b in 0..3 {
                                let bx = stage_x + 22.0 + (b as f32) * 5.0;
                                let bh = if b == 1 { 14.0 } else { 8.0 };
                                self.render_target.DrawLine(
                                    D2D_POINT_2F {
                                        x: bx,
                                        y: params.y + 63.0 - bh / 2.0,
                                    },
                                    D2D_POINT_2F {
                                        x: bx,
                                        y: params.y + 63.0 + bh / 2.0,
                                    },
                                    &self.text_primary_brush,
                                    1.8,
                                    None,
                                );
                            }

                            // Track Info
                            let title_r = D2D_RECT_F {
                                left: stage_x + 58.0,
                                top: params.y + 42.0,
                                right: stage_r - 120.0,
                                bottom: params.y + 60.0,
                            };
                            draw_text(
                                &self.render_target,
                                "Midnight City • M83",
                                &self.title_format,
                                &self.text_primary_brush,
                                &title_r,
                            );

                            let sub_r = D2D_RECT_F {
                                left: stage_x + 58.0,
                                top: params.y + 60.0,
                                right: stage_r - 120.0,
                                bottom: params.y + 76.0,
                            };
                            draw_text(
                                &self.render_target,
                                "Spotify • Lossless Audio Streaming",
                                &self.caption_format,
                                &self.text_secondary_brush,
                                &sub_r,
                            );

                            // Seekbar Line
                            let s_start = stage_x + 58.0;
                            let s_end = stage_r - 130.0;
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: s_start,
                                    y: params.y + 82.0,
                                },
                                D2D_POINT_2F {
                                    x: s_end,
                                    y: params.y + 82.0,
                                },
                                &self.card_edge_brush,
                                2.5,
                                None,
                            );
                            let s_prog = s_start + (s_end - s_start) * 0.42;
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: s_start,
                                    y: params.y + 82.0,
                                },
                                D2D_POINT_2F {
                                    x: s_prog,
                                    y: params.y + 82.0,
                                },
                                &self.accent_brush,
                                2.5,
                                None,
                            );
                            let s_knob = D2D1_ELLIPSE {
                                point: D2D_POINT_2F {
                                    x: s_prog,
                                    y: params.y + 82.0,
                                },
                                radiusX: 4.0,
                                radiusY: 4.0,
                            };
                            self.render_target
                                .FillEllipse(&s_knob, &self.text_primary_brush);

                            // Playback buttons capsule
                            let pb_r = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: stage_r - 110.0,
                                    top: params.y + 50.0,
                                    right: stage_r - 10.0,
                                    bottom: params.y + 84.0,
                                },
                                radiusX: 14.0,
                                radiusY: 14.0,
                            };
                            self.render_target
                                .FillRoundedRectangle(&pb_r, &self.card_fill_brush);
                            self.render_target.DrawRoundedRectangle(
                                &pb_r,
                                &self.card_edge_brush,
                                1.0,
                                None,
                            );

                            let p_cx = stage_r - 60.0;
                            let p_cy = params.y + 67.0;

                            // Prev: |◀
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx - 28.0,
                                    y: p_cy - 4.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx - 28.0,
                                    y: p_cy + 4.5,
                                },
                                &self.text_secondary_brush,
                                1.5,
                                None,
                            );
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx - 22.0,
                                    y: p_cy - 4.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx - 28.0,
                                    y: p_cy,
                                },
                                &self.text_secondary_brush,
                                1.5,
                                None,
                            );
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx - 22.0,
                                    y: p_cy + 4.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx - 28.0,
                                    y: p_cy,
                                },
                                &self.text_secondary_brush,
                                1.5,
                                None,
                            );

                            // Play center circle
                            let play_c = D2D1_ELLIPSE {
                                point: D2D_POINT_2F { x: p_cx, y: p_cy },
                                radiusX: 9.0,
                                radiusY: 9.0,
                            };
                            self.render_target.FillEllipse(&play_c, &self.accent_brush);
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx - 2.5,
                                    y: p_cy - 3.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx + 3.5,
                                    y: p_cy,
                                },
                                &self.text_primary_brush,
                                1.6,
                                None,
                            );
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx + 3.5,
                                    y: p_cy,
                                },
                                D2D_POINT_2F {
                                    x: p_cx - 2.5,
                                    y: p_cy + 3.5,
                                },
                                &self.text_primary_brush,
                                1.6,
                                None,
                            );
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx - 2.5,
                                    y: p_cy + 3.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx - 2.5,
                                    y: p_cy - 3.5,
                                },
                                &self.text_primary_brush,
                                1.6,
                                None,
                            );

                            // Next: ▶|
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx + 28.0,
                                    y: p_cy - 4.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx + 28.0,
                                    y: p_cy + 4.5,
                                },
                                &self.text_secondary_brush,
                                1.5,
                                None,
                            );
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx + 22.0,
                                    y: p_cy - 4.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx + 28.0,
                                    y: p_cy,
                                },
                                &self.text_secondary_brush,
                                1.5,
                                None,
                            );
                            self.render_target.DrawLine(
                                D2D_POINT_2F {
                                    x: p_cx + 22.0,
                                    y: p_cy + 4.5,
                                },
                                D2D_POINT_2F {
                                    x: p_cx + 28.0,
                                    y: p_cy,
                                },
                                &self.text_secondary_brush,
                                1.5,
                                None,
                            );

                            // 2. Four Quick Site Cards (y: params.y + 110.0 to 176.0)
                            let card_w = 112.0;
                            let card_gap = 10.0;
                            let card_y = params.y + 110.0;
                            let card_h = 66.0;

                            let sites = [
                                ("YT Music", "music.you...", true, false),
                                ("YouTube", "youtube.c...", true, false),
                                ("Spotify", "open.spoti...", false, false),
                                ("+ Add a site", "", false, true),
                            ];

                            for (i, (s_name, s_url, is_red, is_dashed)) in sites.iter().enumerate()
                            {
                                let cx_box = stage_x + (i as f32) * (card_w + card_gap);
                                let box_r = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: cx_box,
                                        top: card_y,
                                        right: cx_box + card_w,
                                        bottom: card_y + card_h,
                                    },
                                    radiusX: 12.0,
                                    radiusY: 12.0,
                                };
                                self.render_target
                                    .FillRoundedRectangle(&box_r, &self.card_fill_brush);
                                self.render_target.DrawRoundedRectangle(
                                    &box_r,
                                    &self.card_edge_brush,
                                    1.0,
                                    None,
                                );

                                if *is_dashed {
                                    // Add a site dashed box
                                    let add_r = D2D_RECT_F {
                                        left: cx_box,
                                        top: card_y + 22.0,
                                        right: cx_box + card_w,
                                        bottom: card_y + 44.0,
                                    };
                                    draw_text(
                                        &self.render_target,
                                        "+ Add a site",
                                        &self.title_format,
                                        &self.text_secondary_brush,
                                        &add_r,
                                    );
                                } else {
                                    // Circular Logo
                                    let logo = D2D1_ELLIPSE {
                                        point: D2D_POINT_2F {
                                            x: cx_box + 22.0,
                                            y: card_y + 33.0,
                                        },
                                        radiusX: 11.0,
                                        radiusY: 11.0,
                                    };
                                    let l_brush = if *is_red {
                                        &self.danger_brush
                                    } else {
                                        &self.ok_brush
                                    };
                                    self.render_target.FillEllipse(&logo, l_brush);

                                    // White play/music glyph inside logo
                                    let l_dot = D2D1_ELLIPSE {
                                        point: D2D_POINT_2F {
                                            x: cx_box + 22.0,
                                            y: card_y + 33.0,
                                        },
                                        radiusX: 3.5,
                                        radiusY: 3.5,
                                    };
                                    self.render_target
                                        .FillEllipse(&l_dot, &self.text_primary_brush);

                                    let n_r = D2D_RECT_F {
                                        left: cx_box + 38.0,
                                        top: card_y + 18.0,
                                        right: cx_box + card_w - 4.0,
                                        bottom: card_y + 34.0,
                                    };
                                    draw_text(
                                        &self.render_target,
                                        s_name,
                                        &self.body_format,
                                        &self.text_primary_brush,
                                        &n_r,
                                    );

                                    let u_r = D2D_RECT_F {
                                        left: cx_box + 38.0,
                                        top: card_y + 34.0,
                                        right: cx_box + card_w - 4.0,
                                        bottom: card_y + 50.0,
                                    };
                                    draw_text(
                                        &self.render_target,
                                        s_url,
                                        &self.caption_format,
                                        &self.text_secondary_brush,
                                        &u_r,
                                    );
                                }
                            }
                        }
                    }

                    // ── FOOTER HINT (y: params.y + 192.0 to 212.0) ────────────────────────
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: stage_x + 10.0,
                            y: params.y + params.height - 28.0,
                        },
                        D2D_POINT_2F {
                            x: stage_r - 10.0,
                            y: params.y + params.height - 28.0,
                        },
                        &self.card_edge_brush,
                        0.75,
                        None,
                    );

                    let foot_r = D2D_RECT_F {
                        left: stage_x,
                        top: params.y + params.height - 24.0,
                        right: stage_r,
                        bottom: params.y + params.height - 6.0,
                    };

                    let foot_text = match self.active_tab {
                        0 => "Add a site: any page, signed in, one click away.",
                        1 => "What the island says lands here. Click one to open it.",
                        2 => "Drop any file from File Explorer directly onto the notch.",
                        3 => "Encrypted AES-256 GCM clipboard vault  •  Ctrl+Alt+V",
                        _ => {
                            "Press Esc or Ctrl+Alt+N to collapse  •  Right-click tray for settings"
                        }
                    };
                    draw_text(
                        &self.render_target,
                        foot_text,
                        &self.caption_center,
                        &self.text_tertiary_brush,
                        &foot_r,
                    );
                } else if params.height >= 50.0 {
                    // ══════════════════════════════════════════════════════════════════════
                    // GLANCE VIEW (360 × 64 px)
                    // ══════════════════════════════════════════════════════════════════════
                    let eq_x = params.x + 18.0;
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

                    let title_r = D2D_RECT_F {
                        left: params.x + 46.0,
                        top: params.y + 12.0,
                        right: params.x + params.width - 70.0,
                        bottom: params.y + 32.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Nuvra • Acoustic Stage",
                        &self.display_format,
                        &self.text_primary_brush,
                        &title_r,
                    );

                    let sub_r = D2D_RECT_F {
                        left: params.x + 46.0,
                        top: params.y + 32.0,
                        right: params.x + params.width - 70.0,
                        bottom: params.y + 50.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Media Ready • Click to Expand",
                        &self.body_format,
                        &self.text_secondary_brush,
                        &sub_r,
                    );

                    let play_center = D2D_POINT_2F {
                        x: params.x + params.width - 30.0,
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
