// nuvra-platform-win/src/renderer.rs
// Direct2D 1.1 + DirectWrite hardware-accelerated renderer
// Implements exact showcase layouts from user specification:
//   - "A quiet notch..." (Ambient): [N badge] WiFi Battery% Date/Time
//   - "...an expandable workspace." (Expanded): Media Card | Tools | Notification Pill | Date/Time | Glass/Opaque Toggle | Settings
//   - Dynamic Frosted Glassmorphism vs Solid Opaque mode toggle
// R-13: All unsafe blocks carry // SAFETY: comments.
#![allow(unsafe_code)]

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
    DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER,
    DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::{IDXGISurface, IDXGISwapChain, DXGI_PRESENT};

/// Frame rendering input parameters.
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererAction {
    ToggleDrawer(usize),
    TogglePlayPause,
    ToggleMute,
    ToggleGlassMode,
    OpenSettings,
}

pub struct NotchRenderer {
    _factory: ID2D1Factory,
    _dwrite_factory: IDWriteFactory,
    render_target: ID2D1RenderTarget,
    // Brushes
    glass_fill_brush: ID2D1LinearGradientBrush,
    glass_rim_brush: ID2D1LinearGradientBrush,
    opaque_fill_brush: ID2D1SolidColorBrush,
    opaque_edge_brush: ID2D1SolidColorBrush,
    purple_pill_brush: ID2D1SolidColorBrush,
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
    _caption_center: IDWriteTextFormat,
    ambient_format: IDWriteTextFormat,
    frame_counter: u32,
    pub is_glass_mode: bool,
    pub active_drawer: Option<usize>,
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
                        r: 0.07,
                        g: 0.09,
                        b: 0.16,
                        a: 0.38, // 38% alpha: background wallpaper clearly blurs through
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 0.50,
                    color: D2D1_COLOR_F {
                        r: 0.05,
                        g: 0.07,
                        b: 0.13,
                        a: 0.44,
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: D2D1_COLOR_F {
                        r: 0.03,
                        g: 0.04,
                        b: 0.09,
                        a: 0.52,
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
                        a: 0.70, // Bright specular glass catchlight
                    },
                },
                D2D1_GRADIENT_STOP {
                    position: 0.35,
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.22,
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

            let purple_col = D2D1_COLOR_F {
                r: 0.424,
                g: 0.388,
                b: 1.0,
                a: 0.95,
            };
            let purple_pill_brush = render_target
                .CreateSolidColorBrush(&purple_col, None)
                .map_err(|e| format!("Purple brush failed: {e}"))?;

            let opaque_col = D2D1_COLOR_F {
                r: 0.05,
                g: 0.06,
                b: 0.09,
                a: 1.0,
            };
            let opaque_fill_brush = render_target
                .CreateSolidColorBrush(&opaque_col, None)
                .map_err(|e| format!("Opaque fill brush failed: {e}"))?;

            let opaque_edge_col = D2D1_COLOR_F {
                r: 0.20,
                g: 0.22,
                b: 0.30,
                a: 1.0,
            };
            let opaque_edge_brush = render_target
                .CreateSolidColorBrush(&opaque_edge_col, None)
                .map_err(|e| format!("Opaque edge brush failed: {e}"))?;

            // 4. DirectWrite Typography Hierarchy
            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|e| format!("DWriteCreateFactory failed: {e}"))?;

            // Display title: Segoe UI, Bold 12.0 pt
            let display_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    12.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat display failed: {e}"))?;
            let _ = display_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = display_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Subtitle: Segoe UI, SemiBold 11.0 pt
            let title_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    11.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat title failed: {e}"))?;
            let _ = title_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = title_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Body: Segoe UI, Normal 10.0 pt
            let body_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    10.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat body failed: {e}"))?;
            let _ = body_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = body_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Badge counter: Segoe UI, Bold 9.0 pt
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
            let _ = badge_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = badge_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption leading: Segoe UI, Normal 9.0 pt
            let caption_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    9.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption failed: {e}"))?;
            let _ = caption_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = caption_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption trailing: Segoe UI, SemiBold 9.0 pt
            let caption_trailing = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    9.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption trailing failed: {e}"))?;
            let _ = caption_trailing.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING);
            let _ = caption_trailing.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption center: Segoe UI, Normal 9.0 pt
            let caption_center = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    9.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption center failed: {e}"))?;
            let _ = caption_center.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = caption_center.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Ambient date/time label: Segoe UI, SemiBold 10.5 pt
            let ambient_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    10.5,
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
                opaque_fill_brush,
                opaque_edge_brush,
                purple_pill_brush,
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
                _caption_center: caption_center,
                ambient_format,
                frame_counter: 0,
                is_glass_mode: true,
                active_drawer: None,
            })
        }
    }

    pub fn toggle_glass_mode(&mut self) -> bool {
        self.is_glass_mode = !self.is_glass_mode;
        self.is_glass_mode
    }

    pub fn is_glass_mode(&self) -> bool {
        self.is_glass_mode
    }

    pub fn set_glass_mode(&mut self, glass: bool) {
        self.is_glass_mode = glass;
    }

    pub fn active_drawer(&self) -> Option<usize> {
        self.active_drawer
    }

    pub fn set_active_drawer(&mut self, drawer: Option<usize>) {
        self.active_drawer = drawer;
    }

    /// Handles a click on the notch in window coordinates. Returns the detected action if any.
    pub fn handle_click(&mut self, x: f32, y: f32) -> Option<RendererAction> {
        // Workspace bar hit testing (y: 6.0 to 48.0)
        if (6.0..=48.0).contains(&y) {
            // Media Previous: x 138..158
            if (138.0..=158.0).contains(&x) {
                return Some(RendererAction::TogglePlayPause);
            }
            // Media Play/Pause: x 160..186
            if (160.0..=186.0).contains(&x) {
                return Some(RendererAction::TogglePlayPause);
            }
            // Media Next: x 188..208
            if (188.0..=208.0).contains(&x) {
                return Some(RendererAction::TogglePlayPause);
            }
            // Quick Tool 3: Search (x 230..256)
            if (230.0..=256.0).contains(&x) {
                let d = 3;
                self.active_drawer = if self.active_drawer == Some(d) {
                    None
                } else {
                    Some(d)
                };
                return Some(RendererAction::ToggleDrawer(d));
            }
            // Quick Tool 0: File Shelf (x 270..298)
            if (270.0..=298.0).contains(&x) {
                let d = 0;
                self.active_drawer = if self.active_drawer == Some(d) {
                    None
                } else {
                    Some(d)
                };
                return Some(RendererAction::ToggleDrawer(d));
            }
            // Quick Tool 1: Clipboard History (x 312..340)
            if (312.0..=340.0).contains(&x) {
                let d = 1;
                self.active_drawer = if self.active_drawer == Some(d) {
                    None
                } else {
                    Some(d)
                };
                return Some(RendererAction::ToggleDrawer(d));
            }
            // Quick Tool: Microphone Mute (x 354..382)
            if (354.0..=382.0).contains(&x) {
                return Some(RendererAction::ToggleMute);
            }
            // Section 3: Notification Pill (x 408..464)
            if (408.0..=464.0).contains(&x) {
                let d = 2;
                self.active_drawer = if self.active_drawer == Some(d) {
                    None
                } else {
                    Some(d)
                };
                return Some(RendererAction::ToggleDrawer(d));
            }
            // Section 4: Glass/Opaque Toggle Button 💎 (x 638..668)
            if (638.0..=668.0).contains(&x) {
                self.is_glass_mode = !self.is_glass_mode;
                return Some(RendererAction::ToggleGlassMode);
            }
            // Settings Gear: x 674..706
            if (674.0..=706.0).contains(&x) {
                return Some(RendererAction::OpenSettings);
            }
        }
        None
    }

    /// Render a frame of the notch pill with full glassmorphic or opaque styling and present.
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
                    a: if self.is_glass_mode { 0.06 } else { 0.10 },
                });
                self.card_edge_brush.SetColor(&D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: if self.is_glass_mode { 0.14 } else { 0.18 },
                });
                self.specular_brush.SetColor(&D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.60,
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
                    .SetColor(&to_color(params.accent_color, 0.25));
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

                // ── 1. BASE NOTCH SHELL (Frosted Glass vs Solid Opaque) ──
                if self.is_glass_mode {
                    // Frosted Glass Linear Gradient Fill
                    self.render_target
                        .FillRoundedRectangle(&rounded_rect, &self.glass_fill_brush);

                    // Specular beveled glass rim
                    self.render_target.DrawRoundedRectangle(
                        &rounded_rect,
                        &self.glass_rim_brush,
                        1.2,
                        None,
                    );

                    // Top inset catchlight line
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
                } else {
                    // 100% Solid Matte Obsidian (matches showcase image exactly)
                    self.render_target
                        .FillRoundedRectangle(&rounded_rect, &self.opaque_fill_brush);
                    self.render_target.DrawRoundedRectangle(
                        &rounded_rect,
                        &self.opaque_edge_brush,
                        1.0,
                        None,
                    );
                }

                if params.height >= 45.0 {
                    // ══════════════════════════════════════════════════════════════════════
                    // EXPANDED WORKSPACE VIEW ("...an expandable workspace.")
                    // Exact layout matching showcase image:
                    //   [Album Art] Good Days / SZA  |◀ ⏸ ▶|  |  🔍  📁  📋  🎤  |  [🔔 2]  |  Mon, Nov 24 7:15 PM  💎  ⚙️
                    // ══════════════════════════════════════════════════════════════════════
                    let btn_cy = params.y + 26.0;

                    // ── SECTION 1: MEDIA PLAYER (Left side) ──
                    // Album art thumbnail (36x36 px)
                    let art_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 12.0,
                            top: params.y + 8.0,
                            right: params.x + 48.0,
                            bottom: params.y + 44.0,
                        },
                        radiusX: 7.0,
                        radiusY: 7.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&art_rect, &self.purple_pill_brush);

                    // Album graphic detail (warm glowing sun inside art)
                    let inner_sun = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: params.x + 30.0,
                            y: params.y + 26.0,
                        },
                        radiusX: 7.0,
                        radiusY: 7.0,
                    };
                    self.render_target
                        .FillEllipse(&inner_sun, &self.accent_brush);

                    // Track Title & Artist
                    let title_r = D2D_RECT_F {
                        left: params.x + 56.0,
                        top: params.y + 10.0,
                        right: params.x + 138.0,
                        bottom: params.y + 26.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Good Days",
                        &self.display_format,
                        &self.text_primary_brush,
                        &title_r,
                    );

                    let artist_r = D2D_RECT_F {
                        left: params.x + 56.0,
                        top: params.y + 26.0,
                        right: params.x + 138.0,
                        bottom: params.y + 42.0,
                    };
                    draw_text(
                        &self.render_target,
                        "SZA",
                        &self.caption_format,
                        &self.text_secondary_brush,
                        &artist_r,
                    );

                    // Transport buttons:
                    let prev_cx = params.x + 148.0;
                    let play_cx = params.x + 173.0;
                    let next_cx = params.x + 198.0;

                    // Prev |◀
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: prev_cx - 5.5,
                            y: btn_cy - 5.0,
                        },
                        D2D_POINT_2F {
                            x: prev_cx - 5.5,
                            y: btn_cy + 5.0,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: prev_cx + 4.0,
                            y: btn_cy - 5.0,
                        },
                        D2D_POINT_2F {
                            x: prev_cx - 4.0,
                            y: btn_cy,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: prev_cx + 4.0,
                            y: btn_cy + 5.0,
                        },
                        D2D_POINT_2F {
                            x: prev_cx - 4.0,
                            y: btn_cy,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );

                    // Play/Pause capsule ⏸
                    let pp_cap = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: play_cx - 11.0,
                            top: btn_cy - 12.0,
                            right: play_cx + 11.0,
                            bottom: btn_cy + 12.0,
                        },
                        radiusX: 6.0,
                        radiusY: 6.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&pp_cap, &self.card_fill_brush);
                    // Double pause bars:
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: play_cx - 3.0,
                            y: btn_cy - 5.0,
                        },
                        D2D_POINT_2F {
                            x: play_cx - 3.0,
                            y: btn_cy + 5.0,
                        },
                        &self.text_primary_brush,
                        2.2,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: play_cx + 3.0,
                            y: btn_cy - 5.0,
                        },
                        D2D_POINT_2F {
                            x: play_cx + 3.0,
                            y: btn_cy + 5.0,
                        },
                        &self.text_primary_brush,
                        2.2,
                        None,
                    );

                    // Next ▶|
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: next_cx + 5.5,
                            y: btn_cy - 5.0,
                        },
                        D2D_POINT_2F {
                            x: next_cx + 5.5,
                            y: btn_cy + 5.0,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: next_cx - 4.0,
                            y: btn_cy - 5.0,
                        },
                        D2D_POINT_2F {
                            x: next_cx + 4.0,
                            y: btn_cy,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: next_cx - 4.0,
                            y: btn_cy + 5.0,
                        },
                        D2D_POINT_2F {
                            x: next_cx + 4.0,
                            y: btn_cy,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );

                    // ── DIVIDER 1 ──
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: params.x + 218.0,
                            y: params.y + 14.0,
                        },
                        D2D_POINT_2F {
                            x: params.x + 218.0,
                            y: params.y + 38.0,
                        },
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    // ── SECTION 2: QUICK TOOLS (Center icons) ──
                    // 1. Search 🔍
                    let s_cx = params.x + 243.0;
                    let s_circle = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: s_cx - 2.0,
                            y: btn_cy - 2.0,
                        },
                        radiusX: 4.5,
                        radiusY: 4.5,
                    };
                    self.render_target
                        .DrawEllipse(&s_circle, &self.text_secondary_brush, 1.5, None);
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: s_cx + 1.5,
                            y: btn_cy + 1.5,
                        },
                        D2D_POINT_2F {
                            x: s_cx + 6.0,
                            y: btn_cy + 6.0,
                        },
                        &self.text_secondary_brush,
                        1.8,
                        None,
                    );

                    // 2. File Shelf 📁
                    let f_cx = params.x + 284.0;
                    let f_box = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: f_cx - 8.0,
                            top: btn_cy - 5.0,
                            right: f_cx + 8.0,
                            bottom: btn_cy + 6.0,
                        },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    self.render_target
                        .DrawRoundedRectangle(&f_box, &self.text_secondary_brush, 1.4, None);
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: f_cx - 8.0,
                            y: btn_cy - 6.5,
                        },
                        D2D_POINT_2F {
                            x: f_cx - 2.0,
                            y: btn_cy - 6.5,
                        },
                        &self.text_secondary_brush,
                        1.4,
                        None,
                    );

                    // 3. Clipboard 📋
                    let c_cx = params.x + 326.0;
                    let c_box = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: c_cx - 6.5,
                            top: btn_cy - 6.0,
                            right: c_cx + 6.5,
                            bottom: btn_cy + 7.0,
                        },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    self.render_target
                        .DrawRoundedRectangle(&c_box, &self.text_secondary_brush, 1.4, None);
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: c_cx - 3.0,
                            y: btn_cy - 7.5,
                        },
                        D2D_POINT_2F {
                            x: c_cx + 3.0,
                            y: btn_cy - 7.5,
                        },
                        &self.text_secondary_brush,
                        1.4,
                        None,
                    );

                    // 4. Microphone 🎤
                    let m_cx = params.x + 368.0;
                    let mic_cap = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: m_cx - 3.5,
                            top: btn_cy - 7.0,
                            right: m_cx + 3.5,
                            bottom: btn_cy + 2.0,
                        },
                        radiusX: 2.5,
                        radiusY: 2.5,
                    };
                    self.render_target
                        .DrawRoundedRectangle(&mic_cap, &self.text_secondary_brush, 1.4, None);
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: m_cx,
                            y: btn_cy + 2.0,
                        },
                        D2D_POINT_2F {
                            x: m_cx,
                            y: btn_cy + 6.5,
                        },
                        &self.text_secondary_brush,
                        1.4,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: m_cx - 4.0,
                            y: btn_cy + 6.5,
                        },
                        D2D_POINT_2F {
                            x: m_cx + 4.0,
                            y: btn_cy + 6.5,
                        },
                        &self.text_secondary_brush,
                        1.4,
                        None,
                    );

                    // ── DIVIDER 2 ──
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: params.x + 396.0,
                            y: params.y + 14.0,
                        },
                        D2D_POINT_2F {
                            x: params.x + 396.0,
                            y: params.y + 38.0,
                        },
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    // ── SECTION 3: NOTIFICATION PILL BUTTON [🔔 2] ──
                    let notif_cap = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 412.0,
                            top: params.y + 11.0,
                            right: params.x + 462.0,
                            bottom: params.y + 41.0,
                        },
                        radiusX: 15.0,
                        radiusY: 15.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&notif_cap, &self.purple_pill_brush);

                    // White vector bell glyph:
                    let b_cx = params.x + 430.0;
                    let b_top = D2D_POINT_2F {
                        x: b_cx,
                        y: btn_cy - 5.5,
                    };
                    let b_bl = D2D_POINT_2F {
                        x: b_cx - 4.5,
                        y: btn_cy + 3.0,
                    };
                    let b_br = D2D_POINT_2F {
                        x: b_cx + 4.5,
                        y: btn_cy + 3.0,
                    };
                    self.render_target
                        .DrawLine(b_top, b_bl, &self.text_primary_brush, 1.4, None);
                    self.render_target
                        .DrawLine(b_top, b_br, &self.text_primary_brush, 1.4, None);
                    self.render_target
                        .DrawLine(b_bl, b_br, &self.text_primary_brush, 1.4, None);

                    // Red badge counter circle with "2":
                    let red_dot = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: params.x + 448.0,
                            y: params.y + 18.0,
                        },
                        radiusX: 5.5,
                        radiusY: 5.5,
                    };
                    self.render_target
                        .FillEllipse(&red_dot, &self.danger_brush);
                    let badge_txt_r = D2D_RECT_F {
                        left: params.x + 443.0,
                        top: params.y + 11.0,
                        right: params.x + 453.0,
                        bottom: params.y + 25.0,
                    };
                    draw_text(
                        &self.render_target,
                        "2",
                        &self.badge_format,
                        &self.text_primary_brush,
                        &badge_txt_r,
                    );

                    // ── DIVIDER 3 ──
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: params.x + 476.0,
                            y: params.y + 14.0,
                        },
                        D2D_POINT_2F {
                            x: params.x + 476.0,
                            y: params.y + 38.0,
                        },
                        &self.card_edge_brush,
                        1.0,
                        None,
                    );

                    // ── SECTION 4: SYSTEM INFO, GLASS TOGGLE & SETTINGS ──
                    let date_str = get_current_date_time_string();
                    let dt_r = D2D_RECT_F {
                        left: params.x + 488.0,
                        top: params.y + 10.0,
                        right: params.x + 634.0,
                        bottom: params.y + 42.0,
                    };
                    draw_text(
                        &self.render_target,
                        &date_str,
                        &self.title_format,
                        &self.text_primary_brush,
                        &dt_r,
                    );

                    // Glass / Opaque Mode Toggle Button 💎:
                    let g_cx = params.x + 652.0;
                    let g_brush = if self.is_glass_mode {
                        &self.accent_glow_brush
                    } else {
                        &self.card_fill_brush
                    };
                    let g_cap = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: g_cx - 13.0,
                            top: btn_cy - 12.0,
                            right: g_cx + 13.0,
                            bottom: btn_cy + 12.0,
                        },
                        radiusX: 6.0,
                        radiusY: 6.0,
                    };
                    self.render_target.FillRoundedRectangle(&g_cap, g_brush);
                    let g_border = if self.is_glass_mode {
                        &self.accent_brush
                    } else {
                        &self.card_edge_brush
                    };
                    self.render_target
                        .DrawRoundedRectangle(&g_cap, g_border, 1.0, None);

                    // Diamond prism vector glyph 💎:
                    let prism_brush = if self.is_glass_mode {
                        &self.text_primary_brush
                    } else {
                        &self.text_secondary_brush
                    };
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: g_cx,
                            y: btn_cy - 5.5,
                        },
                        D2D_POINT_2F {
                            x: g_cx + 5.5,
                            y: btn_cy,
                        },
                        prism_brush,
                        1.2,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: g_cx + 5.5,
                            y: btn_cy,
                        },
                        D2D_POINT_2F {
                            x: g_cx,
                            y: btn_cy + 5.5,
                        },
                        prism_brush,
                        1.2,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: g_cx,
                            y: btn_cy + 5.5,
                        },
                        D2D_POINT_2F {
                            x: g_cx - 5.5,
                            y: btn_cy,
                        },
                        prism_brush,
                        1.2,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: g_cx - 5.5,
                            y: btn_cy,
                        },
                        D2D_POINT_2F {
                            x: g_cx,
                            y: btn_cy - 5.5,
                        },
                        prism_brush,
                        1.2,
                        None,
                    );

                    // Settings Gear ⚙️:
                    let gear_cx = params.x + 688.0;
                    let gear_circle = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: gear_cx,
                            y: btn_cy,
                        },
                        radiusX: 4.5,
                        radiusY: 4.5,
                    };
                    self.render_target
                        .DrawEllipse(&gear_circle, &self.text_secondary_brush, 1.2, None);
                    let gear_inner = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: gear_cx,
                            y: btn_cy,
                        },
                        radiusX: 1.8,
                        radiusY: 1.8,
                    };
                    self.render_target
                        .FillEllipse(&gear_inner, &self.text_secondary_brush);

                    // ── DRAWER CONTENT (When Shelf, Clipboard, etc. is expanded) ──
                    if params.height >= 110.0 {
                        // Horizontal divider separating top bar from drawer
                        self.render_target.DrawLine(
                            D2D_POINT_2F {
                                x: params.x + 16.0,
                                y: params.y + 53.0,
                            },
                            D2D_POINT_2F {
                                x: params.x + params.width - 16.0,
                                y: params.y + 53.0,
                            },
                            &self.card_edge_brush,
                            1.0,
                            None,
                        );

                        match self.active_drawer {
                            Some(0) => {
                                // 📁 File Shelf Drawer
                                let shelf_r = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: params.x + 20.0,
                                        top: params.y + 64.0,
                                        right: params.x + params.width - 20.0,
                                        bottom: params.y + params.height - 14.0,
                                    },
                                    radiusX: 10.0,
                                    radiusY: 10.0,
                                };
                                self.render_target
                                    .FillRoundedRectangle(&shelf_r, &self.card_fill_brush);
                                self.render_target.DrawRoundedRectangle(
                                    &shelf_r,
                                    &self.card_edge_brush,
                                    1.0,
                                    None,
                                );

                                let s_title = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 80.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 100.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "File Shelf  •  Drag. Access. Share.",
                                    &self.title_format,
                                    &self.text_primary_brush,
                                    &s_title,
                                );

                                let s_sub = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 104.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 124.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "Drop any files directly onto this zone to stage in memory for instant sharing.",
                                    &self.body_format,
                                    &self.text_secondary_brush,
                                    &s_sub,
                                );
                            }
                            Some(1) => {
                                // 📋 Clipboard History Drawer
                                let clip_r = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: params.x + 20.0,
                                        top: params.y + 64.0,
                                        right: params.x + params.width - 20.0,
                                        bottom: params.y + params.height - 14.0,
                                    },
                                    radiusX: 10.0,
                                    radiusY: 10.0,
                                };
                                self.render_target
                                    .FillRoundedRectangle(&clip_r, &self.card_fill_brush);
                                self.render_target.DrawRoundedRectangle(
                                    &clip_r,
                                    &self.card_edge_brush,
                                    1.0,
                                    None,
                                );

                                let c_title = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 75.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 95.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "Clipboard Vault  •  Encrypted History",
                                    &self.title_format,
                                    &self.text_primary_brush,
                                    &c_title,
                                );

                                // Item 1
                                let it1 = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 100.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 118.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "1. https://github.com/Kesavaraja67/nuvra",
                                    &self.body_format,
                                    &self.text_secondary_brush,
                                    &it1,
                                );
                                // Item 2
                                let it2 = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 120.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 138.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "2. cargo run -p nuvra-app",
                                    &self.body_format,
                                    &self.text_secondary_brush,
                                    &it2,
                                );
                            }
                            Some(2) => {
                                // 🔔 Smart Notifications Drawer
                                let notif_r = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: params.x + 20.0,
                                        top: params.y + 64.0,
                                        right: params.x + params.width - 20.0,
                                        bottom: params.y + params.height - 14.0,
                                    },
                                    radiusX: 10.0,
                                    radiusY: 10.0,
                                };
                                self.render_target
                                    .FillRoundedRectangle(&notif_r, &self.card_fill_brush);
                                self.render_target.DrawRoundedRectangle(
                                    &notif_r,
                                    &self.card_edge_brush,
                                    1.0,
                                    None,
                                );

                                let n_title = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 75.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 95.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "Smart Notifications  •  Stay informed",
                                    &self.title_format,
                                    &self.text_primary_brush,
                                    &n_title,
                                );

                                let n_it1 = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 100.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 118.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "• Calendar: Team Standup meeting in 5 minutes",
                                    &self.body_format,
                                    &self.text_secondary_brush,
                                    &n_it1,
                                );

                                let n_it2 = D2D_RECT_F {
                                    left: params.x + 30.0,
                                    top: params.y + 120.0,
                                    right: params.x + params.width - 30.0,
                                    bottom: params.y + 138.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "• Shelf: Received companion files package",
                                    &self.body_format,
                                    &self.text_secondary_brush,
                                    &n_it2,
                                );
                            }
                            Some(3) => {
                                // 🔍 Quick Search Drawer
                                let s_r = D2D1_ROUNDED_RECT {
                                    rect: D2D_RECT_F {
                                        left: params.x + 20.0,
                                        top: params.y + 64.0,
                                        right: params.x + params.width - 20.0,
                                        bottom: params.y + params.height - 14.0,
                                    },
                                    radiusX: 10.0,
                                    radiusY: 10.0,
                                };
                                self.render_target
                                    .FillRoundedRectangle(&s_r, &self.card_fill_brush);
                                self.render_target.DrawRoundedRectangle(
                                    &s_r,
                                    &self.card_edge_brush,
                                    1.0,
                                    None,
                                );

                                let s_text = D2D_RECT_F {
                                    left: params.x + 34.0,
                                    top: params.y + 80.0,
                                    right: params.x + params.width - 34.0,
                                    bottom: params.y + 104.0,
                                };
                                draw_text(
                                    &self.render_target,
                                    "🔍  Search apps, documents, web... |",
                                    &self.body_format,
                                    &self.text_secondary_brush,
                                    &s_text,
                                );
                            }
                            _ => {}
                        }
                    }
                } else if params.height >= 20.0 {
                    // ══════════════════════════════════════════════════════════════════════
                    // AMBIENT VIEW ("A quiet notch...")
                    // Exact layout matching showcase image:
                    //   [Purple N badge]  📶  🔋 78%   Mon, Nov 24  7:15 PM
                    // ══════════════════════════════════════════════════════════════════════
                    let amb_cy = params.y + 17.0;

                    // 1. Purple Nuvra "N" Badge
                    let badge_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 10.0,
                            top: params.y + 7.0,
                            right: params.x + 34.0,
                            bottom: params.y + 27.0,
                        },
                        radiusX: 5.0,
                        radiusY: 5.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&badge_rect, &self.purple_pill_brush);

                    // Stylized N glyph inside badge:
                    let n_left = params.x + 16.0;
                    let n_top = params.y + 12.0;
                    let n_bottom = params.y + 22.0;
                    let n_right = params.x + 28.0;
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: n_left,
                            y: n_bottom,
                        },
                        D2D_POINT_2F {
                            x: n_left,
                            y: n_top,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: n_left,
                            y: n_top,
                        },
                        D2D_POINT_2F {
                            x: n_right,
                            y: n_bottom,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: n_right,
                            y: n_bottom,
                        },
                        D2D_POINT_2F {
                            x: n_right,
                            y: n_top,
                        },
                        &self.text_primary_brush,
                        1.8,
                        None,
                    );

                    // 2. WiFi Glyph 📶
                    let wifi_cx = params.x + 50.0;
                    let wifi_cy = amb_cy + 2.0;
                    let dot = D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: wifi_cx,
                            y: wifi_cy,
                        },
                        radiusX: 1.5,
                        radiusY: 1.5,
                    };
                    self.render_target
                        .FillEllipse(&dot, &self.text_primary_brush);
                    // 2 concentric arcs
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: wifi_cx - 4.0,
                            y: wifi_cy - 3.5,
                        },
                        D2D_POINT_2F {
                            x: wifi_cx,
                            y: wifi_cy - 5.5,
                        },
                        &self.text_primary_brush,
                        1.2,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: wifi_cx,
                            y: wifi_cy - 5.5,
                        },
                        D2D_POINT_2F {
                            x: wifi_cx + 4.0,
                            y: wifi_cy - 3.5,
                        },
                        &self.text_primary_brush,
                        1.2,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: wifi_cx - 7.0,
                            y: wifi_cy - 6.5,
                        },
                        D2D_POINT_2F {
                            x: wifi_cx,
                            y: wifi_cy - 9.0,
                        },
                        &self.text_primary_brush,
                        1.2,
                        None,
                    );
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: wifi_cx,
                            y: wifi_cy - 9.0,
                        },
                        D2D_POINT_2F {
                            x: wifi_cx + 7.0,
                            y: wifi_cy - 6.5,
                        },
                        &self.text_primary_brush,
                        1.2,
                        None,
                    );

                    // 3. Battery Capsule & Percentage
                    let bat_r = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 68.0,
                            top: amb_cy - 5.5,
                            right: params.x + 84.0,
                            bottom: amb_cy + 5.5,
                        },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    self.render_target.DrawRoundedRectangle(
                        &bat_r,
                        &self.text_secondary_brush,
                        1.0,
                        None,
                    );
                    // Battery tip:
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: params.x + 85.0,
                            y: amb_cy - 2.5,
                        },
                        D2D_POINT_2F {
                            x: params.x + 85.0,
                            y: amb_cy + 2.5,
                        },
                        &self.text_secondary_brush,
                        1.0,
                        None,
                    );
                    // Battery fill (78%):
                    let fill_w = 13.0 * 0.78;
                    let bat_fill = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 69.5,
                            top: amb_cy - 4.0,
                            right: params.x + 69.5 + fill_w,
                            bottom: amb_cy + 4.0,
                        },
                        radiusX: 1.0,
                        radiusY: 1.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&bat_fill, &self.text_primary_brush);

                    let bat_txt_r = D2D_RECT_F {
                        left: params.x + 89.0,
                        top: params.y + 6.0,
                        right: params.x + 118.0,
                        bottom: params.y + 28.0,
                    };
                    draw_text(
                        &self.render_target,
                        "78%",
                        &self.badge_format,
                        &self.text_primary_brush,
                        &bat_txt_r,
                    );

                    // 4. Date & Time
                    let date_str = get_current_date_time_string();
                    let amb_date_r = D2D_RECT_F {
                        left: params.x + 124.0,
                        top: params.y + 6.0,
                        right: params.x + params.width - 10.0,
                        bottom: params.y + 28.0,
                    };
                    draw_text(
                        &self.render_target,
                        &date_str,
                        &self.ambient_format,
                        &self.text_secondary_brush,
                        &amb_date_r,
                    );
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

/// Fetches and formats real Windows local time matching the showcase format: "Mon, Nov 24  7:15 PM"
fn get_current_date_time_string() -> String {
    use windows::Win32::System::SystemInformation::GetLocalTime;
    // SAFETY: GetLocalTime returns valid Win32 SYSTEMTIME for local machine.
    let st = unsafe { GetLocalTime() };
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let day_name = days.get(st.wDayOfWeek as usize).unwrap_or(&"Mon");
    let month_name = months
        .get(st.wMonth.saturating_sub(1) as usize)
        .unwrap_or(&"Nov");
    let (hour_12, am_pm) = match st.wHour {
        0 => (12, "AM"),
        1..=11 => (st.wHour, "AM"),
        12 => (12, "PM"),
        _ => (st.wHour - 12, "PM"),
    };

    format!(
        "{}, {} {}  {}:{:02} {}",
        day_name, month_name, st.wDay, hour_12, st.wMinute, am_pm
    )
}
