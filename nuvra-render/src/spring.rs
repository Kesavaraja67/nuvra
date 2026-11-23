// nuvra-render/src/spring.rs
// Spring physics animation simulation for notch state transitions (D-WIN-01..09, Section 6.4).
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotchState {
    Idle,
    Ambient,
    Glance,
    Expanded,
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

    /// Advance spring simulation by dt (seconds) using semi-implicit Euler integration.
    pub fn step(&mut self, dt: f32) {
        if self.is_settled() {
            self.current = self.target;
            self.velocity = 0.0;
            return;
        }

        let displacement = self.current - self.target;
        let spring_force = -self.params.stiffness * displacement;
        let damping_force = -self.params.damping * self.velocity;
        let acceleration = (spring_force + damping_force) / self.params.mass;

        self.velocity += acceleration * dt;
        self.current += self.velocity * dt;

        // Convergence check: snap if close enough and slow enough
        if (self.current - self.target).abs() < 0.2 && self.velocity.abs() < 0.1 {
            self.current = self.target;
            self.velocity = 0.0;
        }
    }

    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < 0.001 && self.velocity.abs() < 0.001
    }
}

/// 4-parameter animated layout geometry.
#[derive(Debug, Clone)]
pub struct NotchGeometryAnimator {
    pub width: SpringScalar,
    pub height: SpringScalar,
    pub radius: SpringScalar,
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
            width: SpringScalar::new(tokens::window::IDLE_WIDTH, expand_params),
            height: SpringScalar::new(tokens::window::IDLE_HEIGHT, expand_params),
            radius: SpringScalar::new(0.0, expand_params),
            current_state: NotchState::Idle,
        }
    }

    pub fn set_state(&mut self, state: NotchState) {
        self.current_state = state;
        let (target_w, target_h, target_r) = match state {
            NotchState::Idle => (tokens::window::IDLE_WIDTH, tokens::window::IDLE_HEIGHT, 0.0),
            NotchState::Ambient => (
                tokens::window::AMBIENT_WIDTH,
                tokens::window::AMBIENT_HEIGHT,
                tokens::radius::AMBIENT,
            ),
            NotchState::Glance => (
                tokens::window::GLANCE_WIDTH,
                tokens::window::GLANCE_HEIGHT,
                tokens::radius::GLANCE,
            ),
            NotchState::Expanded => (
                tokens::window::EXPANDED_WIDTH,
                tokens::window::EXPANDED_DEFAULT_HEIGHT,
                tokens::radius::EXPANDED,
            ),
        };

        // Select spring profile based on direction (expand vs collapse)
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

        self.width.params = params;
        self.width.set_target(target_w);

        self.height.params = params;
        self.height.set_target(target_h);

        self.radius.params = params;
        self.radius.set_target(target_r);
    }

    pub fn step(&mut self, dt: f32) {
        self.width.step(dt);
        self.height.step(dt);
        self.radius.step(dt);
    }

    pub fn is_settled(&self) -> bool {
        self.width.is_settled() && self.height.is_settled() && self.radius.is_settled()
    }
}

impl Default for NotchGeometryAnimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_convergence() {
        let mut spring = SpringScalar::new(
            0.0,
            SpringParams {
                stiffness: tokens::spring::EXPAND.stiffness,
                damping: tokens::spring::EXPAND.damping,
                mass: tokens::spring::EXPAND.mass,
            },
        );
        spring.set_target(100.0);

        // Simulate 60 FPS (dt = 0.016)
        for _ in 0..120 {
            spring.step(0.016);
            if spring.is_settled() {
                break;
            }
        }

        assert!(spring.is_settled(), "Spring must settle within 2 seconds");
        assert!(
            (spring.current - 100.0).abs() < 0.1,
            "Final value must converge to target"
        );
    }

    #[test]
    fn test_animator_state_transitions() {
        let mut animator = NotchGeometryAnimator::new();
        assert_eq!(animator.current_state, NotchState::Idle);

        animator.set_state(NotchState::Ambient);
        assert_eq!(animator.current_state, NotchState::Ambient);
        assert_eq!(animator.height.target, tokens::window::AMBIENT_HEIGHT);

        for _ in 0..120 {
            animator.step(0.016);
        }
        assert_eq!(animator.height.current, tokens::window::AMBIENT_HEIGHT);
    }
}
