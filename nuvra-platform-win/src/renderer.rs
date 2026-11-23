// nuvra-platform-win/src/renderer.rs
// Direct2D hardware-accelerated rendering to DXGI swapchain (D-STK-03, D-PERF-03).
// Every unsafe block carries a // SAFETY: comment.

use windows::core::w;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_POINT_2F, D2D_RECT_F,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1RenderTarget, ID2D1SolidColorBrush,
    D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
    D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_ROUNDED_RECT,
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
    surface_brush: ID2D1SolidColorBrush,
    edge_brush: ID2D1SolidColorBrush,
    text_brush: ID2D1SolidColorBrush,
    secondary_brush: ID2D1SolidColorBrush,
    tertiary_brush: ID2D1SolidColorBrush,
    accent_brush: ID2D1SolidColorBrush,
    card_brush: ID2D1SolidColorBrush,
    ok_brush: ID2D1SolidColorBrush,
    danger_brush: ID2D1SolidColorBrush,
    title_format: IDWriteTextFormat,
    body_format: IDWriteTextFormat,
    caption_format: IDWriteTextFormat,
    caption_center_format: IDWriteTextFormat,
    center_format: IDWriteTextFormat,
    icon_format: IDWriteTextFormat,
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

            let default_color = D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };
            let surface_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush surface failed: {e}"))?;

            let edge_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush edge failed: {e}"))?;

            let text_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush text failed: {e}"))?;

            let secondary_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush secondary failed: {e}"))?;

            let tertiary_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush tertiary failed: {e}"))?;

            let accent_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush accent failed: {e}"))?;

            let card_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush card failed: {e}"))?;

            let ok_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush ok failed: {e}"))?;

            let danger_brush = render_target
                .CreateSolidColorBrush(&default_color, None)
                .map_err(|e| format!("CreateSolidColorBrush danger failed: {e}"))?;

            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|e| format!("DWriteCreateFactory failed: {e}"))?;

            // Centered format for Ambient pill label
            let center_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    13.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat center failed: {e}"))?;
            let _ = center_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = center_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Title format for Expanded and Glance
            let title_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    15.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat title failed: {e}"))?;
            let _ = title_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = title_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Body format for subtitle and track artist
            let body_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    12.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat body failed: {e}"))?;
            let _ = body_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = body_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption format for timestamps and tool cards
            let caption_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    10.5,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption failed: {e}"))?;
            let _ = caption_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);
            let _ = caption_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Caption centered format for hints
            let caption_center_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    10.5,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat caption_center failed: {e}"))?;
            let _ = caption_center_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = caption_center_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            // Icon format for rail and playback buttons
            let icon_format = dwrite_factory
                .CreateTextFormat(
                    w!("Segoe UI Symbol"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    14.0,
                    w!("en-us"),
                )
                .map_err(|e| format!("CreateTextFormat icon failed: {e}"))?;
            let _ = icon_format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
            let _ = icon_format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);

            Ok(Self {
                _factory: factory,
                _dwrite_factory: dwrite_factory,
                render_target,
                surface_brush,
                edge_brush,
                text_brush,
                secondary_brush,
                tertiary_brush,
                accent_brush,
                card_brush,
                ok_brush,
                danger_brush,
                title_format,
                body_format,
                caption_format,
                caption_center_format,
                center_format,
                icon_format,
            })
        }
    }

    /// Render a frame of the notch pill and present to the DXGI swapchain.
    pub fn render_and_present(
        &mut self,
        swapchain: &IDXGISwapChain,
        params: &PillRenderParams,
    ) -> Result<(), String> {
        // Convert ARGB u32 to D2D1_COLOR_F with custom alpha multiplier
        let to_color = |c: u32, alpha_mult: f32| {
            let a = (((c >> 24) & 0xFF) as f32 / 255.0) * alpha_mult;
            let r = ((c >> 16) & 0xFF) as f32 / 255.0;
            let g = ((c >> 8) & 0xFF) as f32 / 255.0;
            let b = (c & 0xFF) as f32 / 255.0;
            D2D1_COLOR_F { r, g, b, a }
        };

        // Helper to draw text within a rectangle
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

            // Only draw pill if visible
            if params.height > 1.0 && params.width > 1.0 {
                // Update brushes
                self.surface_brush
                    .SetColor(&to_color(params.surface_color, params.alpha));
                self.edge_brush.SetColor(&to_color(params.edge_color, 0.6));
                self.text_brush.SetColor(&to_color(params.text_color, 1.0));
                self.secondary_brush
                    .SetColor(&to_color(params.text_secondary, 0.9));
                self.tertiary_brush
                    .SetColor(&to_color(params.text_tertiary, 0.8));
                self.accent_brush
                    .SetColor(&to_color(params.accent_color, 1.0));
                self.card_brush
                    .SetColor(&to_color(params.surface_raised, params.alpha));
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

                // Fill translucent glass pill surface
                self.render_target
                    .FillRoundedRectangle(&rounded_rect, &self.surface_brush);

                // Stroke 1px edge
                self.render_target
                    .DrawRoundedRectangle(&rounded_rect, &self.edge_brush, 1.0, None);

                if params.height >= 160.0 {
                    // ── EXPANDED VIEW (520 × 220 px) — D-WIN-08 ─────────────────────────────
                    // 1. Left 5-icon Navigation Rail (Media, Shelf, Clipboard, Notifications, Settings)
                    let rail_width = 52.0;
                    let rail_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 4.0,
                            top: params.y + 4.0,
                            right: params.x + rail_width,
                            bottom: params.y + params.height - 4.0,
                        },
                        radiusX: 12.0,
                        radiusY: 12.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&rail_rect, &self.card_brush);

                    // Vertical divider line
                    self.render_target.DrawLine(
                        D2D_POINT_2F {
                            x: params.x + rail_width + 4.0,
                            y: params.y + 12.0,
                        },
                        D2D_POINT_2F {
                            x: params.x + rail_width + 4.0,
                            y: params.y + params.height - 12.0,
                        },
                        &self.edge_brush,
                        1.0,
                        None,
                    );

                    // Rail Icons (D-WIN-08: Media, Shelf, Clipboard, Notifications, More)
                    let rail_icons = [
                        ("\u{266B}", true),   // Media (Active)
                        ("\u{1F4C1}", false), // Shelf
                        ("\u{1F4CB}", false), // Clipboard
                        ("\u{1F514}", false), // Notifications
                        ("\u{2699}", false),  // Settings / More
                    ];
                    let icon_step = (params.height - 30.0) / 5.0;
                    for (i, (glyph, is_active)) in rail_icons.iter().enumerate() {
                        let icon_y = params.y + 15.0 + (i as f32) * icon_step;
                        let item_rect = D2D_RECT_F {
                            left: params.x + 6.0,
                            top: icon_y,
                            right: params.x + rail_width - 2.0,
                            bottom: icon_y + 26.0,
                        };

                        if *is_active {
                            // Active accent indicator bar on left
                            let ind_rect = D2D1_ROUNDED_RECT {
                                rect: D2D_RECT_F {
                                    left: params.x + 6.0,
                                    top: icon_y + 4.0,
                                    right: params.x + 9.0,
                                    bottom: icon_y + 22.0,
                                },
                                radiusX: 1.5,
                                radiusY: 1.5,
                            };
                            self.render_target
                                .FillRoundedRectangle(&ind_rect, &self.accent_brush);
                            draw_text(
                                &self.render_target,
                                glyph,
                                &self.icon_format,
                                &self.accent_brush,
                                &item_rect,
                            );
                        } else {
                            draw_text(
                                &self.render_target,
                                glyph,
                                &self.icon_format,
                                &self.secondary_brush,
                                &item_rect,
                            );
                        }
                    }

                    // 2. Active Module Panel (Media & System Controller)
                    let panel_x = params.x + rail_width + 16.0;
                    let panel_right = params.x + params.width - 16.0;

                    // Track Art Card
                    let art_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: panel_x,
                            top: params.y + 16.0,
                            right: panel_x + 58.0,
                            bottom: params.y + 74.0,
                        },
                        radiusX: 10.0,
                        radiusY: 10.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&art_rect, &self.card_brush);
                    self.render_target
                        .DrawRoundedRectangle(&art_rect, &self.edge_brush, 1.0, None);
                    draw_text(
                        &self.render_target,
                        "\u{266B}",
                        &self.icon_format,
                        &self.accent_brush,
                        &art_rect.rect,
                    );

                    // Track Title & Subtitle
                    let title_rect = D2D_RECT_F {
                        left: panel_x + 68.0,
                        top: params.y + 16.0,
                        right: panel_right,
                        bottom: params.y + 36.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Windows Media Controller",
                        &self.title_format,
                        &self.text_brush,
                        &title_rect,
                    );

                    let subtitle_rect = D2D_RECT_F {
                        left: panel_x + 68.0,
                        top: params.y + 36.0,
                        right: panel_right,
                        bottom: params.y + 54.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Ready • Global System Media Transport",
                        &self.body_format,
                        &self.secondary_brush,
                        &subtitle_rect,
                    );

                    // Playback Controls (Previous, Play/Pause circular button, Next)
                    let ctrl_y = params.y + 56.0;
                    let prev_rect = D2D_RECT_F {
                        left: panel_x + 68.0,
                        top: ctrl_y,
                        right: panel_x + 94.0,
                        bottom: ctrl_y + 24.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{23EE}",
                        &self.icon_format,
                        &self.secondary_brush,
                        &prev_rect,
                    );

                    let play_circle = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: panel_x + 102.0,
                            top: ctrl_y - 2.0,
                            right: panel_x + 130.0,
                            bottom: ctrl_y + 26.0,
                        },
                        radiusX: 14.0,
                        radiusY: 14.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&play_circle, &self.accent_brush);
                    let play_glyph_rect = play_circle.rect;
                    draw_text(
                        &self.render_target,
                        "\u{25B6}",
                        &self.icon_format,
                        &self.surface_brush,
                        &play_glyph_rect,
                    );

                    let next_rect = D2D_RECT_F {
                        left: panel_x + 138.0,
                        top: ctrl_y,
                        right: panel_x + 164.0,
                        bottom: ctrl_y + 24.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{23ED}",
                        &self.icon_format,
                        &self.secondary_brush,
                        &next_rect,
                    );

                    // Seekbar
                    let seek_y = params.y + 92.0;
                    let seek_bg = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: panel_x,
                            top: seek_y,
                            right: panel_right,
                            bottom: seek_y + 4.0,
                        },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&seek_bg, &self.card_brush);

                    let seek_fill = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: panel_x,
                            top: seek_y,
                            right: panel_x + 120.0,
                            bottom: seek_y + 4.0,
                        },
                        radiusX: 2.0,
                        radiusY: 2.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&seek_fill, &self.accent_brush);

                    // Timestamps
                    let time_left_rect = D2D_RECT_F {
                        left: panel_x,
                        top: seek_y + 6.0,
                        right: panel_x + 60.0,
                        bottom: seek_y + 22.0,
                    };
                    draw_text(
                        &self.render_target,
                        "0:00",
                        &self.caption_format,
                        &self.tertiary_brush,
                        &time_left_rect,
                    );

                    let time_right_rect = D2D_RECT_F {
                        left: panel_right - 60.0,
                        top: seek_y + 6.0,
                        right: panel_right,
                        bottom: seek_y + 22.0,
                    };
                    let _ = self
                        .caption_format
                        .SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING);
                    draw_text(
                        &self.render_target,
                        "--:--",
                        &self.caption_format,
                        &self.tertiary_brush,
                        &time_right_rect,
                    );
                    let _ = self
                        .caption_format
                        .SetTextAlignment(DWRITE_TEXT_ALIGNMENT_LEADING);

                    // Quick System Cards (Mic, Clipboard, Shelf)
                    let card_w = (panel_right - panel_x - 16.0) / 3.0;
                    let card_y = params.y + 122.0;

                    // 1. Mic Status Card
                    let mic_card = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: panel_x,
                            top: card_y,
                            right: panel_x + card_w,
                            bottom: card_y + 54.0,
                        },
                        radiusX: 8.0,
                        radiusY: 8.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&mic_card, &self.card_brush);
                    self.render_target
                        .DrawRoundedRectangle(&mic_card, &self.edge_brush, 1.0, None);

                    let mic_label_rect = D2D_RECT_F {
                        left: panel_x + 8.0,
                        top: card_y + 6.0,
                        right: panel_x + card_w - 8.0,
                        bottom: card_y + 24.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{1F3A4} Mic: Active",
                        &self.body_format,
                        &self.ok_brush,
                        &mic_label_rect,
                    );

                    let mic_sub_rect = D2D_RECT_F {
                        left: panel_x + 8.0,
                        top: card_y + 28.0,
                        right: panel_x + card_w - 8.0,
                        bottom: card_y + 46.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Ctrl+Alt+M to mute",
                        &self.caption_format,
                        &self.tertiary_brush,
                        &mic_sub_rect,
                    );

                    // 2. Clipboard Card
                    let clip_card = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: panel_x + card_w + 8.0,
                            top: card_y,
                            right: panel_x + card_w * 2.0 + 8.0,
                            bottom: card_y + 54.0,
                        },
                        radiusX: 8.0,
                        radiusY: 8.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&clip_card, &self.card_brush);
                    self.render_target.DrawRoundedRectangle(
                        &clip_card,
                        &self.edge_brush,
                        1.0,
                        None,
                    );

                    let clip_label_rect = D2D_RECT_F {
                        left: clip_card.rect.left + 8.0,
                        top: card_y + 6.0,
                        right: clip_card.rect.right - 8.0,
                        bottom: card_y + 24.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{1F4CB} Clipboard",
                        &self.body_format,
                        &self.text_brush,
                        &clip_label_rect,
                    );

                    let clip_sub_rect = D2D_RECT_F {
                        left: clip_card.rect.left + 8.0,
                        top: card_y + 28.0,
                        right: clip_card.rect.right - 8.0,
                        bottom: card_y + 46.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Listening (Ctrl+Alt+V)",
                        &self.caption_format,
                        &self.tertiary_brush,
                        &clip_sub_rect,
                    );

                    // 3. Shelf Card
                    let shelf_card = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: panel_x + card_w * 2.0 + 16.0,
                            top: card_y,
                            right: panel_right,
                            bottom: card_y + 54.0,
                        },
                        radiusX: 8.0,
                        radiusY: 8.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&shelf_card, &self.card_brush);
                    self.render_target.DrawRoundedRectangle(
                        &shelf_card,
                        &self.edge_brush,
                        1.0,
                        None,
                    );

                    let shelf_label_rect = D2D_RECT_F {
                        left: shelf_card.rect.left + 8.0,
                        top: card_y + 6.0,
                        right: shelf_card.rect.right - 8.0,
                        bottom: card_y + 24.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{1F4C1} Files Shelf",
                        &self.body_format,
                        &self.text_brush,
                        &shelf_label_rect,
                    );

                    let shelf_sub_rect = D2D_RECT_F {
                        left: shelf_card.rect.left + 8.0,
                        top: card_y + 28.0,
                        right: shelf_card.rect.right - 8.0,
                        bottom: card_y + 46.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Drop files onto notch",
                        &self.caption_format,
                        &self.tertiary_brush,
                        &shelf_sub_rect,
                    );

                    // Footer Hint
                    let footer_rect = D2D_RECT_F {
                        left: panel_x,
                        top: params.y + params.height - 24.0,
                        right: panel_right,
                        bottom: params.y + params.height - 6.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Press Ctrl+Alt+N or click to collapse • Right-click tray icon for settings",
                        &self.caption_center_format,
                        &self.tertiary_brush,
                        &footer_rect,
                    );
                } else if params.height >= 50.0 {
                    // ── GLANCE VIEW (360 × 64 px) ──────────────────────────────────────────
                    let left_icon_rect = D2D_RECT_F {
                        left: params.x + 14.0,
                        top: params.y + 12.0,
                        right: params.x + 44.0,
                        bottom: params.y + params.height - 12.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{266B}",
                        &self.icon_format,
                        &self.accent_brush,
                        &left_icon_rect,
                    );

                    let title_rect = D2D_RECT_F {
                        left: params.x + 48.0,
                        top: params.y + 12.0,
                        right: params.x + params.width - 70.0,
                        bottom: params.y + 34.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Nuvra • Media Ready",
                        &self.title_format,
                        &self.text_brush,
                        &title_rect,
                    );

                    let sub_rect = D2D_RECT_F {
                        left: params.x + 48.0,
                        top: params.y + 34.0,
                        right: params.x + params.width - 70.0,
                        bottom: params.y + 52.0,
                    };
                    draw_text(
                        &self.render_target,
                        "Click or Ctrl+Alt+N to expand",
                        &self.caption_format,
                        &self.secondary_brush,
                        &sub_rect,
                    );

                    let right_icon_rect = D2D_RECT_F {
                        left: params.x + params.width - 60.0,
                        top: params.y + 14.0,
                        right: params.x + params.width - 14.0,
                        bottom: params.y + params.height - 14.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{1F3A4}",
                        &self.icon_format,
                        &self.ok_brush,
                        &right_icon_rect,
                    );
                } else if params.height >= 24.0 {
                    // ── AMBIENT VIEW (220 × 34 px) ─────────────────────────────────────────
                    let dot_rect = D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: params.x + 14.0,
                            top: params.y + 13.0,
                            right: params.x + 22.0,
                            bottom: params.y + 21.0,
                        },
                        radiusX: 4.0,
                        radiusY: 4.0,
                    };
                    self.render_target
                        .FillRoundedRectangle(&dot_rect, &self.ok_brush);

                    let label_rect = D2D_RECT_F {
                        left: params.x + 28.0,
                        top: params.y,
                        right: params.x + params.width - 28.0,
                        bottom: params.y + params.height,
                    };
                    draw_text(
                        &self.render_target,
                        "NUVRA",
                        &self.center_format,
                        &self.text_brush,
                        &label_rect,
                    );

                    let mic_rect = D2D_RECT_F {
                        left: params.x + params.width - 24.0,
                        top: params.y + 4.0,
                        right: params.x + params.width - 8.0,
                        bottom: params.y + params.height - 4.0,
                    };
                    draw_text(
                        &self.render_target,
                        "\u{1F3A4}",
                        &self.caption_center_format,
                        &self.ok_brush,
                        &mic_rect,
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
