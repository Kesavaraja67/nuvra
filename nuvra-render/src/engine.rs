// nuvra-render/src/engine.rs
// Notch presentation engine and state coordinator (D-WIN-01..09, Section 6).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::spring::{NotchGeometryAnimator, NotchState};
use crate::tokens;
use nuvra_core_shared::module::ModuleState;

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
    pub edge_lit: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_tertiary: u32,
    pub accent: u32,
    pub danger: u32,
    pub ok: u32,
    pub warning: u32,
}

impl Theme {
    pub fn colors(&self) -> ColorScheme {
        match self {
            Theme::Obsidian => ColorScheme {
                surface: tokens::obsidian::SURFACE,
                surface_raised: tokens::obsidian::SURFACE_RAISED,
                surface_high: tokens::obsidian::SURFACE_HIGH,
                edge: tokens::obsidian::EDGE,
                edge_lit: tokens::obsidian::EDGE_LIT,
                text_primary: tokens::obsidian::TEXT_PRIMARY,
                text_secondary: tokens::obsidian::TEXT_SECONDARY,
                text_tertiary: tokens::obsidian::TEXT_TERTIARY,
                accent: tokens::obsidian::ACCENT,
                danger: tokens::obsidian::DANGER,
                ok: tokens::obsidian::OK,
                warning: tokens::obsidian::WARNING,
            },
            Theme::Paper => ColorScheme {
                surface: tokens::paper::SURFACE,
                surface_raised: tokens::paper::SURFACE_RAISED,
                surface_high: tokens::paper::SURFACE_HIGH,
                edge: tokens::paper::EDGE,
                edge_lit: tokens::paper::EDGE_LIT,
                text_primary: tokens::paper::TEXT_PRIMARY,
                text_secondary: tokens::paper::TEXT_SECONDARY,
                text_tertiary: tokens::paper::TEXT_TERTIARY,
                accent: tokens::paper::ACCENT,
                danger: tokens::paper::DANGER,
                ok: tokens::paper::OK,
                warning: tokens::paper::WARNING,
            },
            Theme::Signal => ColorScheme {
                surface: tokens::signal::SURFACE,
                surface_raised: tokens::signal::SURFACE_RAISED,
                surface_high: tokens::signal::SURFACE_HIGH,
                edge: tokens::signal::EDGE,
                edge_lit: tokens::signal::EDGE_LIT,
                text_primary: tokens::signal::TEXT_PRIMARY,
                text_secondary: tokens::signal::TEXT_SECONDARY,
                text_tertiary: tokens::signal::TEXT_TERTIARY,
                accent: tokens::signal::ACCENT,
                danger: tokens::signal::DANGER,
                ok: tokens::signal::OK,
                warning: tokens::signal::WARNING,
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
    pub colors: ColorScheme,
    pub is_settled: bool,
    pub active_state: NotchState,
}

pub struct NotchEngine {
    theme: Theme,
    animator: NotchGeometryAnimator,
    is_hovered: bool,
    is_pinned: bool,
    is_expanded: bool,
    active_drawer: Option<usize>,
}

impl NotchEngine {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            animator: NotchGeometryAnimator::new(),
            is_hovered: false,
            is_pinned: true, // Default to pinned ambient so notch is visible on launch (D-WIN-07)
            is_expanded: false,
            active_drawer: None,
        }
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_pinned(&mut self, pinned: bool) {
        self.is_pinned = pinned;
    }

    pub fn is_pinned(&self) -> bool {
        self.is_pinned
    }

    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded;
        if !self.is_expanded {
            self.active_drawer = None;
        }
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.is_expanded = expanded;
        if !expanded {
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
    }

    pub fn toggle_drawer(&mut self, drawer: usize) {
        if self.active_drawer == Some(drawer) {
            self.active_drawer = None;
        } else {
            self.active_drawer = Some(drawer);
            self.is_expanded = true;
        }
    }

    /// Advance the simulation and compute the current visual frame.
    pub fn update(&mut self, active_module: &ModuleState, dt: f32) -> FrameDescriptor {
        // Resolve target state based on priority & user interactions
        if self.is_expanded {
            self.animator.set_state(NotchState::Expanded);
            if self.active_drawer.is_some() {
                self.animator
                    .set_target_height(tokens::window::EXPANDED_DRAWER_HEIGHT);
            } else {
                self.animator
                    .set_target_height(tokens::window::EXPANDED_DEFAULT_HEIGHT);
            }
        } else if self.is_hovered {
            self.animator.set_state(NotchState::Glance);
        } else if self.is_pinned {
            // When pinned and not expanded, stay at Ambient visible pill (D-WIN-07)
            self.animator.set_state(NotchState::Ambient);
        } else {
            // Unhovered and unpinned: inspect module state
            match active_module {
                ModuleState::Idle => {
                    self.animator.set_state(NotchState::Idle);
                }
                ModuleState::Media(m) if m.is_playing => {
                    self.animator.set_state(NotchState::Ambient);
                }
                ModuleState::Mic(m) if m.call_active || !m.is_muted => {
                    self.animator.set_state(NotchState::Ambient);
                }
                ModuleState::Notification(n) if n.queued > 0 => {
                    self.animator.set_state(NotchState::Ambient);
                }
                _ => {
                    self.animator.set_state(NotchState::Idle);
                }
            }
        }

        self.animator.step(dt);

        FrameDescriptor {
            width: self.animator.width.current,
            height: self.animator.height.current,
            radius: self.animator.radius.current,
            colors: self.theme.colors(),
            is_settled: self.animator.is_settled(),
            active_state: self.animator.current_state,
        }
    }
}

impl Default for NotchEngine {
    fn default() -> Self {
        Self::new(Theme::Obsidian)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuvra_core_shared::event::MediaSnapshot;

    #[test]
    fn test_engine_media_active_state() {
        let mut engine = NotchEngine::new(Theme::Obsidian);
        engine.set_pinned(false);
        let playing_media = ModuleState::Media(MediaSnapshot {
            is_playing: true,
            title: "Test Track".into(),
            artist: "Artist".into(),
            ..Default::default()
        });

        let frame = engine.update(&playing_media, 0.016);
        assert_eq!(frame.active_state, NotchState::Ambient);

        let idle_state = ModuleState::Idle;
        let idle_frame = engine.update(&idle_state, 0.016);
        assert_eq!(idle_frame.active_state, NotchState::Idle);
    }

    #[test]
    fn test_theme_color_selection() {
        let obsidian = Theme::Obsidian.colors();
        assert_eq!(obsidian.surface, tokens::obsidian::SURFACE);

        let paper = Theme::Paper.colors();
        assert_eq!(paper.surface, tokens::paper::SURFACE);

        let signal = Theme::Signal.colors();
        assert_eq!(signal.surface, tokens::signal::SURFACE);
    }
}
