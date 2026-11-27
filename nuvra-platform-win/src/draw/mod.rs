// nuvra-platform-win/src/draw/mod.rs
// Modular Direct2D 1.1 + DirectWrite Glassmorphic drawing pipeline.
// R-13: Unsafe code confined here with // SAFETY: comments.
#![allow(unsafe_code)]

pub mod banner;
pub mod collapsed;
pub mod drawer_views;
pub mod drawers;
pub mod glass;
pub mod icons;
pub mod overlay;
pub mod svg_icons;
pub mod text;
pub mod toolbar;

use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1RenderTarget, ID2D1SolidColorBrush,
    D2D1_ANTIALIAS_MODE_PER_PRIMITIVE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_DEFAULT,
    D2D1_RENDER_TARGET_USAGE_NONE,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Dxgi::{IDXGISurface, IDXGISwapChain, DXGI_PRESENT};

use crate::network::NetworkKind;
use nuvra_render::engine::{ColorScheme, GlassStyle};
use nuvra_render::layout::{ItemId, Layout, LayoutState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererAction {
    ToggleDrawer(usize),
    MediaPrev,
    MediaPlayPause,
    MediaNext,
    MediaSeekPct(u32),
    ToggleMute,
    VolumeDown,
    VolumeUp,
    SetVolume(u32),
    CollapseNotch,
    DismissBanner,
    OpenBannerNotif,
    CopyClipItem(usize),
    ToggleClipPin(usize),
    ClearClipboard,
    ClearShelf,
    CopyShelfItem(usize),
    CopyShelfPath(usize),
    RevealShelfItem(usize),
    ZipShelfItems,
    RemoveShelfItem(usize),
    ClearNotifications,
    CycleGlassStyle(usize),
    ToggleTheme(bool),
    ToggleStartup(bool),
    ToggleFollowSystemTheme(bool),
}

pub struct RenderViewModel<'a> {
    pub media_active: bool,
    pub media_playing: bool,
    pub track_title: &'a str,
    pub track_artist: &'a str,
    pub media_progress: f32,
    pub media_time_label: &'a str,
    pub fft_bands: &'a [f32; 8],
    pub lyric_line: Option<&'a str>,
    pub mic_active: bool,
    pub call_active: bool,
    pub camera_active: bool,
    pub dnd_active: bool,
    pub unread_notifications: u32,
    pub battery_pct: u32,
    pub battery_charging: bool,
    pub volume_pct: u32,
    pub network: NetworkKind,
    pub glass_style: GlassStyle,
    pub is_dark_theme: bool,
    pub startup_enabled: bool,
    pub follow_system_theme: bool,
    pub clipboard_items: &'a [String],
    pub clip_pinned: &'a [bool],
    pub shelf_items: &'a [String],
    pub notifications: &'a [(String, String)],
    pub hover_pos: Option<(f32, f32)>,
    pub copied_flash: bool,
    pub eq_phase: f32,
}

pub struct DrawBrushes {
    pub surface: ID2D1SolidColorBrush,
    pub surface_raised: ID2D1SolidColorBrush,
    pub surface_high: ID2D1SolidColorBrush,
    pub edge: ID2D1SolidColorBrush,
    pub plate_rest: ID2D1SolidColorBrush,
    pub plate_hover: ID2D1SolidColorBrush,
    pub plate_active: ID2D1SolidColorBrush,
    pub divider: ID2D1SolidColorBrush,
    pub text_primary: ID2D1SolidColorBrush,
    pub text_secondary: ID2D1SolidColorBrush,
    pub text_tertiary: ID2D1SolidColorBrush,
    pub accent: ID2D1SolidColorBrush,
    pub accent_hi: ID2D1SolidColorBrush,
    pub danger: ID2D1SolidColorBrush,
    pub ok: ID2D1SolidColorBrush,
    pub warning: ID2D1SolidColorBrush,
    pub logo_a: ID2D1SolidColorBrush,
    pub logo_b: ID2D1SolidColorBrush,
}

pub struct TextFormats {
    pub display: IDWriteTextFormat,
    pub title: IDWriteTextFormat,
    pub artist: IDWriteTextFormat,
    pub body: IDWriteTextFormat,
    pub caption: IDWriteTextFormat,
    pub caption_center: IDWriteTextFormat,
    pub badge: IDWriteTextFormat,
    pub clock: IDWriteTextFormat,
}

pub struct ModularRenderer {
    _factory: ID2D1Factory,
    _dwrite_factory: IDWriteFactory,
    render_target: ID2D1RenderTarget,
    brushes: DrawBrushes,
    formats: TextFormats,
    pub active_drawer: Option<usize>,
    pub is_move_mode: bool,
    pub snap_flashed: bool,
}

fn make_brushes(rt: &ID2D1RenderTarget) -> Result<DrawBrushes, String> {
    let c = D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let b = || unsafe {
        rt.CreateSolidColorBrush(&c, None)
            .map_err(|e| e.to_string())
    };
    Ok(DrawBrushes {
        surface: b()?,
        surface_raised: b()?,
        surface_high: b()?,
        edge: b()?,
        plate_rest: b()?,
        plate_hover: b()?,
        plate_active: b()?,
        divider: b()?,
        text_primary: b()?,
        text_secondary: b()?,
        text_tertiary: b()?,
        accent: b()?,
        accent_hi: b()?,
        danger: b()?,
        ok: b()?,
        warning: b()?,
        logo_a: b()?,
        logo_b: b()?,
    })
}

impl ModularRenderer {
    pub fn new(swapchain: &IDXGISwapChain) -> Result<Self, String> {
        // SAFETY: D2D1CreateFactory, DWriteCreateFactory, and Direct2D DXGI render target initialization.
        unsafe {
            let surface: IDXGISurface = swapchain
                .GetBuffer(0)
                .map_err(|e| format!("DXGI surface: {e}"))?;
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .map_err(|e| format!("D2D1CreateFactory: {e}"))?;
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
                .map_err(|e| format!("D2D target: {e}"))?;
            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|e| format!("DWrite: {e}"))?;
            let brushes = make_brushes(&render_target)?;
            let formats = text::create_text_formats(&dwrite_factory)?;
            Ok(Self {
                _factory: factory,
                _dwrite_factory: dwrite_factory,
                render_target,
                brushes,
                formats,
                active_drawer: None,
                is_move_mode: false,
                snap_flashed: false,
            })
        }
    }

    pub fn set_active_drawer(&mut self, drawer: Option<usize>) {
        self.active_drawer = drawer;
    }

    /// Render full frame onto DXGI swapchain
    pub fn render_frame(
        &mut self,
        swapchain: &IDXGISwapChain,
        colors: &ColorScheme,
        layout: &Layout,
        vm: &RenderViewModel<'_>,
    ) -> Result<(), String> {
        // SAFETY: D2D BeginDraw, Clear, Draw/Fill routines, and EndDraw/Present.
        unsafe {
            text::update_brushes(&self.brushes, colors);
            self.render_target.BeginDraw();
            self.render_target.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            glass::draw_glass_shell(
                &self.render_target,
                layout,
                vm.glass_style,
                vm.is_dark_theme,
                &self.brushes.surface,
                &self.brushes.edge,
            );

            let clip_rect = D2D_RECT_F {
                left: layout.body.left,
                top: layout.body.top,
                right: layout.body.right,
                bottom: layout.body.bottom,
            };
            self.render_target
                .PushAxisAlignedClip(&clip_rect, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);

            match layout.state {
                LayoutState::Collapsed => {
                    collapsed::draw_collapsed(
                        &self.render_target,
                        &self.brushes,
                        &self.formats,
                        layout,
                        vm,
                        self.is_move_mode,
                    );
                }
                LayoutState::Expanded => {
                    toolbar::draw_toolbar(
                        &self.render_target,
                        &self.brushes,
                        &self.formats,
                        layout,
                        vm,
                        self.active_drawer,
                    );
                    if layout.body.height() >= 96.0 * layout.scale {
                        drawers::draw_drawer(
                            &self.render_target,
                            &self.brushes,
                            &self.formats,
                            layout,
                            self.active_drawer,
                            vm,
                        );
                    }
                }
                LayoutState::Banner => {
                    banner::draw_banner(
                        &self.render_target,
                        &self.brushes,
                        &self.formats,
                        layout,
                        vm,
                    );
                }
            }

            self.render_target.PopAxisAlignedClip();
            if self.is_move_mode || self.snap_flashed {
                overlay::draw_move_rim(
                    &self.render_target,
                    &self.brushes,
                    layout,
                    self.snap_flashed,
                );
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

    /// Hit-test click coordinates against toolbar items, workspace cards, bottom collapse handle, and drawers
    pub fn handle_click(
        &self,
        layout: &Layout,
        x: f32,
        y: f32,
        vm: &RenderViewModel<'_>,
    ) -> Option<RendererAction> {
        for item in &layout.items {
            if x >= item.hit_rect.left
                && x <= item.hit_rect.right
                && y >= item.hit_rect.top
                && y <= item.hit_rect.bottom
            {
                let hit = match item.id {
                    ItemId::AlbumArt | ItemId::TrackText => Some(RendererAction::ToggleDrawer(0)),
                    ItemId::Prev => Some(RendererAction::MediaPrev),
                    ItemId::PlayPause => Some(RendererAction::MediaPlayPause),
                    ItemId::Next => Some(RendererAction::MediaNext),
                    ItemId::Find => Some(RendererAction::ToggleDrawer(1)),
                    ItemId::Shelf => Some(RendererAction::ToggleDrawer(2)),
                    ItemId::Clipboard => Some(RendererAction::ToggleDrawer(3)),
                    ItemId::Mic => Some(RendererAction::ToggleMute),
                    ItemId::Bell => Some(RendererAction::ToggleDrawer(4)),
                    ItemId::Settings => Some(RendererAction::ToggleDrawer(5)),
                    ItemId::BannerClose => Some(RendererAction::DismissBanner),
                    ItemId::BannerAppIcon | ItemId::BannerText => {
                        Some(RendererAction::OpenBannerNotif)
                    }
                    _ => None,
                };
                if hit.is_some() {
                    return hit;
                }
            }
        }

        if layout.state != LayoutState::Expanded || layout.body.height() < 96.0 * layout.scale {
            return None;
        }
        let s = layout.scale;
        if let Some(d_idx) = self.active_drawer {
            let bx = drawer_views::DrawerBox {
                l: 16.0 * s,
                t: 54.0 * s,
                r: layout.body.right - 16.0 * s,
                s,
            };
            return drawer_views::hit_test_drawer(d_idx, bx, x, y, vm);
        }

        drawers::hit_test_workspace(layout, x, y, vm)
    }
}
