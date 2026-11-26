// nuvra-render/src/engine.rs
// Notch presentation engine and state coordinator — Document UI-1 Sections 5 & 6.
// R-13: No unsafe in this crate.
#![forbid(unsafe_code)]

use crate::spring::{NotchGeometryAnimator, NotchState};
use crate::tokens;
use nuvra_core_shared::module::ModuleState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlassStyle {
    #[default]
    Frosted,
    Clear,
    Matte,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Obsidian,
    Paper,
    Signal,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub surface: u32,
    pub surface_raised: u32,
    pub surface_high: u32,
    pub edge: u32,
    pub plate_rest: u32,
    pub plate_hover: u32,
    pub plate_active: u32,
    pub divider: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_tertiary: u32,
    pub accent: u32,
    pub accent_hi: u32,
    pub danger: u32,
    pub ok: u32,
    pub warning: u32,
    pub logo_a: u32,
    pub logo_b: u32,
}

impl Theme {
    pub fn colors(&self) -> ColorScheme {
        match self {
            Theme::Obsidian => ColorScheme {
                surface: tokens::obsidian::SURFACE,
                surface_raised: tokens::obsidian::SURFACE_RAISED,
                surface_high: tokens::obsidian::SURFACE_HIGH,
                edge: tokens::obsidian::EDGE,
                plate_rest: tokens::obsidian::PLATE_REST,
                plate_hover: tokens::obsidian::PLATE_HOVER,
                plate_active: tokens::obsidian::PLATE_ACTIVE,
                divider: tokens::obsidian::DIVIDER,
                text_primary: tokens::obsidian::TEXT_PRIMARY,
                text_secondary: tokens::obsidian::TEXT_SECONDARY,
                text_tertiary: tokens::obsidian::TEXT_TERTIARY,
                accent: tokens::obsidian::ACCENT,
                accent_hi: tokens::obsidian::ACCENT_HI,
                danger: tokens::obsidian::DANGER,
                ok: tokens::obsidian::OK,
                warning: tokens::obsidian::WARNING,
                logo_a: tokens::obsidian::LOGO_A,
                logo_b: tokens::obsidian::LOGO_B,
            },
            Theme::Paper => ColorScheme {
                surface: tokens::paper::SURFACE,
                surface_raised: tokens::paper::SURFACE_RAISED,
                surface_high: tokens::paper::SURFACE_HIGH,
                edge: tokens::paper::EDGE,
                plate_rest: tokens::paper::PLATE_REST,
                plate_hover: tokens::paper::PLATE_HOVER,
                plate_active: tokens::paper::PLATE_ACTIVE,
                divider: tokens::paper::DIVIDER,
                text_primary: tokens::paper::TEXT_PRIMARY,
                text_secondary: tokens::paper::TEXT_SECONDARY,
                text_tertiary: tokens::paper::TEXT_TERTIARY,
                accent: tokens::paper::ACCENT,
                accent_hi: tokens::paper::ACCENT_HI,
                danger: tokens::paper::DANGER,
                ok: tokens::paper::OK,
                warning: tokens::paper::WARNING,
                logo_a: tokens::paper::LOGO_A,
                logo_b: tokens::paper::LOGO_B,
            },
            Theme::Signal => ColorScheme {
                surface: tokens::signal::SURFACE,
                surface_raised: tokens::signal::SURFACE_RAISED,
                surface_high: tokens::signal::SURFACE_HIGH,
                edge: tokens::signal::EDGE,
                plate_rest: tokens::signal::PLATE_REST,
                plate_hover: tokens::signal::PLATE_HOVER,
                plate_active: tokens::signal::PLATE_ACTIVE,
                divider: tokens::signal::DIVIDER,
                text_primary: tokens::signal::TEXT_PRIMARY,
                text_secondary: tokens::signal::TEXT_SECONDARY,
                text_tertiary: tokens::signal::TEXT_TERTIARY,
                accent: tokens::signal::ACCENT,
                accent_hi: tokens::signal::ACCENT_HI,
                danger: tokens::signal::DANGER,
                ok: tokens::signal::OK,
                warning: tokens::signal::WARNING,
                logo_a: tokens::signal::LOGO_A,
                logo_b: tokens::signal::LOGO_B,
            },
        }
    }
}

/// Render frame layout descriptor passed to the GPU compositor.
#[derive(Debug, Clone)]
pub struct FrameDescriptor {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub fillet: f32,
    pub colors: ColorScheme,
    pub is_settled: bool,
    pub active_state: NotchState,
}

pub struct NotchEngine {
    theme: Theme,
    animator: NotchGeometryAnimator,
    is_hovered: bool,
    is_expanded: bool,
    is_vertical_expanded: bool,
    is_banner: bool,
    active_drawer: Option<usize>,
}

impl NotchEngine {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            animator: NotchGeometryAnimator::new(),
            is_hovered: false,
            is_expanded: false,
            is_vertical_expanded: false,
            is_banner: false,
            active_drawer: None,
        }
    }

    pub fn set_banner(&mut self, banner: bool) {
        self.is_banner = banner;
    }

    pub fn is_banner(&self) -> bool {
        self.is_banner
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    pub fn is_vertical_expanded(&self) -> bool {
        self.is_vertical_expanded
    }

    pub fn set_vertical_expanded(&mut self, vert: bool) {
        self.is_vertical_expanded = vert;
        if vert {
            self.is_expanded = true;
        } else {
            self.active_drawer = None;
        }
    }

    pub fn toggle_vertical_expand(&mut self) {
        self.set_vertical_expanded(!self.is_vertical_expanded);
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded;
        if !self.is_expanded {
            self.is_vertical_expanded = false;
            self.active_drawer = None;
        }
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.is_expanded = expanded;
        if !expanded {
            self.is_vertical_expanded = false;
            self.active_drawer = None;
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.is_expanded
    }

    pub fn active_drawer(&self) -> Option<usize> {
        self.active_drawer
    }

    pub fn set_active_drawer(&mut self, drawer: Option<usize>) {
        self.active_drawer = drawer;
        if drawer.is_some() {
            self.is_vertical_expanded = true;
            self.is_expanded = true;
        }
    }

    pub fn toggle_drawer(&mut self, drawer: usize) {
        if self.active_drawer == Some(drawer) {
            self.active_drawer = None;
        } else {
            self.active_drawer = Some(drawer);
            self.is_vertical_expanded = true;
            self.is_expanded = true;
        }
    }

    /// Advance the simulation and compute the current visual frame.
    pub fn update(&mut self, _active_module: &ModuleState, dt: f32) -> FrameDescriptor {
        let (target_w, target_h) = if self.is_vertical_expanded || self.active_drawer.is_some() {
            (tokens::window::EXPANDED_WIDTH, tokens::window::EXPANDED_HEIGHT)
        } else if self.is_expanded || self.is_hovered {
            (tokens::window::EXPANDED_WIDTH, tokens::window::EXPANDED_TOOLBAR_HEIGHT)
        } else if self.is_banner {
            (tokens::window::BANNER_WIDTH, tokens::window::BANNER_HEIGHT)
        } else {
            (tokens::window::COLLAPSED_WIDTH, tokens::window::COLLAPSED_HEIGHT)
        };

        self.animator.set_targets(target_w, target_h);
        self.animator.step(dt);

        let active_state =
            if self.animator.width.current > 520.0 || self.animator.height.current > 64.0 {
                NotchState::Expanded
            } else if self.is_banner && self.animator.width.current > 400.0 {
                NotchState::Banner
            } else {
                NotchState::Collapsed
            };

        FrameDescriptor {
            width: self.animator.width.current,
            height: self.animator.height.current,
            radius: tokens::radius::BOTTOM,
            fillet: tokens::radius::FILLET,
            colors: self.theme.colors(),
            is_settled: self.animator.is_settled(),
            active_state,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_state_and_signal_theme() {
        let mut engine = NotchEngine::new(Theme::Signal);
        assert_eq!(engine.update(&ModuleState::Idle, 0.016).active_state, NotchState::Collapsed);
        engine.set_banner(true);
        let mut state = NotchState::Collapsed;
        for _ in 0..30 {
            state = engine.update(&ModuleState::Idle, 0.016).active_state;
        }
        assert_eq!(state, NotchState::Banner);
    }
}
