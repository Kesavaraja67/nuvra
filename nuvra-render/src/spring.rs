// nuvra-render/src/spring.rs
// Spring animation physics for notch state transitions — Document UI-1 Sections 3.2, 6.2, 9.3.
// Fixed radius (20px bottom, 10px fillet); only width and height spring.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotchState {
    Collapsed,
    Expanded,
    Banner,
}

#[derive(Debug, Clone, Copy)]
pub struct SpringParams {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SpringScalar {
    pub current: f32,
    pub target: f32,
    pub velocity: f32,
    pub params: SpringParams,
}

impl SpringScalar {
    pub fn new(initial: f32, params: SpringParams) -> Self {
        Self {
            current: initial,
            target: initial,
            velocity: 0.0,
            params,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Advance spring simulation by dt (seconds) using semi-implicit Euler integration
    /// with fixed micro-stepping for stable, buttery-smooth frame-rate independence.
    pub fn step(&mut self, dt: f32) {
        if self.is_settled() {
            self.current = self.target;
            self.velocity = 0.0;
            return;
        }

        let mut remaining = dt.clamp(0.0001, 0.05);
        let sub_dt = 0.002;
        while remaining > 0.0 {
            let step = remaining.min(sub_dt);
            let displacement = self.current - self.target;
            let spring_force = -self.params.stiffness * displacement;
            let damping_force = -self.params.damping * self.velocity;
            let acceleration = (spring_force + damping_force) / self.params.mass;

            self.velocity += acceleration * step;
            self.current += self.velocity * step;
            remaining -= step;
        }

        // Clean convergence threshold
        if (self.current - self.target).abs() < 0.1 && self.velocity.abs() < 0.05 {
            self.current = self.target;
            self.velocity = 0.0;
        }
    }

    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < 0.001 && self.velocity.abs() < 0.001
    }
}

/// 2-parameter animated layout geometry (Width, Height). Radius is constant at 20px (Section 3.2).
#[derive(Debug, Clone)]
pub struct NotchGeometryAnimator {
    pub width: SpringScalar,
    pub height: SpringScalar,
    pub current_state: NotchState,
}

impl NotchGeometryAnimator {
    pub fn new() -> Self {
        let expand_params = SpringParams {
            stiffness: tokens::spring::EXPAND.stiffness,
            damping: tokens::spring::EXPAND.damping,
            mass: tokens::spring::EXPAND.mass,
        };

        Self {
            width: SpringScalar::new(tokens::window::COLLAPSED_WIDTH, expand_params),
            height: SpringScalar::new(tokens::window::COLLAPSED_HEIGHT, expand_params),
            current_state: NotchState::Collapsed,
        }
    }

    /// Set dynamic target width and height without resetting params if target is unchanged
    pub fn set_targets(&mut self, target_w: f32, target_h: f32) {
        if (self.width.target - target_w).abs() > 0.5 {
            let params = if target_w > self.width.current {
                SpringParams {
                    stiffness: tokens::spring::EXPAND.stiffness,
                    damping: tokens::spring::EXPAND.damping,
                    mass: tokens::spring::EXPAND.mass,
                }
            } else {
                SpringParams {
                    stiffness: tokens::spring::COLLAPSE.stiffness,
                    damping: tokens::spring::COLLAPSE.damping,
                    mass: tokens::spring::COLLAPSE.mass,
                }
            };
            self.width.params = params;
            self.width.set_target(target_w);
        }

        if (self.height.target - target_h).abs() > 0.5 {
            let params = if target_h > self.height.current {
                SpringParams {
                    stiffness: tokens::spring::EXPAND.stiffness,
                    damping: tokens::spring::EXPAND.damping,
                    mass: tokens::spring::EXPAND.mass,
                }
            } else {
                SpringParams {
                    stiffness: tokens::spring::COLLAPSE.stiffness,
                    damping: tokens::spring::COLLAPSE.damping,
                    mass: tokens::spring::COLLAPSE.mass,
                }
            };
            self.height.params = params;
            self.height.set_target(target_h);
        }
    }

    pub fn set_state(&mut self, state: NotchState, _with_media: bool) {
        self.current_state = state;
        let (target_w, target_h) = match state {
            NotchState::Collapsed => (
                tokens::window::COLLAPSED_WIDTH,
                tokens::window::COLLAPSED_HEIGHT,
            ),
            NotchState::Expanded => (
                tokens::window::EXPANDED_WIDTH,
                tokens::window::EXPANDED_HEIGHT,
            ),
            NotchState::Banner => (tokens::window::BANNER_WIDTH, tokens::window::BANNER_HEIGHT),
        };
        self.set_targets(target_w, target_h);
    }

    /// Set custom target height for active tool drawers
    pub fn set_target_height(&mut self, target_h: f32) {
        self.set_targets(self.width.target, target_h);
    }

    pub fn step(&mut self, dt: f32) {
        self.width.step(dt);
        self.height.step(dt);
    }

    pub fn is_settled(&self) -> bool {
        self.width.is_settled() && self.height.is_settled()
    }
}

impl Default for NotchGeometryAnimator {
    fn default() -> Self {
        Self::new()
    }
}
