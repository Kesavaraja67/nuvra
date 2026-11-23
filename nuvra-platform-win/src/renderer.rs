// nuvra-platform-win/src/renderer.rs
// Direct2D hardware-accelerated rendering to DXGI swapchain (D-STK-03, D-PERF-03).
// Every unsafe block carries a // SAFETY: comment.

use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1RenderTarget, ID2D1SolidColorBrush,
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::{IDXGISurface, IDXGISwapChain};

#[derive(Debug, Clone, Copy)]
pub struct PillRenderParams {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub surface_color: u32,
    pub edge_color: u32,
}

pub struct NotchRenderer {
    _factory: ID2D1Factory,
    render_target: ID2D1RenderTarget,
    surface_brush: ID2D1SolidColorBrush,
    edge_brush: ID2D1SolidColorBrush,
}

impl NotchRenderer {
    /// Create a Direct2D render target bound to the DXGI swapchain backbuffer.
    pub fn new(swapchain: &IDXGISwapChain) -> Result<Self, String> {
        // SAFETY: D2D1CreateFactory and CreateDxgiSurfaceRenderTarget calls are verified Win32 COM APIs.
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

            Ok(Self {
                _factory: factory,
                render_target,
                surface_brush,
                edge_brush,
            })
        }
    }

    /// Render a frame of the notch pill and present to the DXGI swapchain.
    pub fn render_and_present(
        &mut self,
        swapchain: &IDXGISwapChain,
        params: &PillRenderParams,
    ) -> Result<(), String> {
        // Convert ARGB u32 to D2D1_COLOR_F
        let to_color = |c: u32| {
            let a = ((c >> 24) & 0xFF) as f32 / 255.0;
            let r = ((c >> 16) & 0xFF) as f32 / 255.0;
            let g = ((c >> 8) & 0xFF) as f32 / 255.0;
            let b = (c & 0xFF) as f32 / 255.0;
            D2D1_COLOR_F { r, g, b, a }
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
                self.surface_brush.SetColor(&to_color(params.surface_color));
                self.edge_brush.SetColor(&to_color(params.edge_color));

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

                // Fill pill surface
                self.render_target
                    .FillRoundedRectangle(&rounded_rect, &self.surface_brush);

                // Stroke 1px edge
                self.render_target
                    .DrawRoundedRectangle(&rounded_rect, &self.edge_brush, 1.0, None);
            }

            self.render_target
                .EndDraw(None, None)
                .map_err(|e| format!("D2D EndDraw failed: {e}"))?;

            swapchain
                .Present(1, windows::Win32::Graphics::Dxgi::DXGI_PRESENT(0))
                .ok()
                .map_err(|e| format!("DXGI Present failed: {e}"))?;
        }

        Ok(())
    }
}
