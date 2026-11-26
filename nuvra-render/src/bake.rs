// nuvra-render/src/bake.rs
// Closed-form analytical spring solver & Hermite cubic segment baker — Document UI-1 Section 9.3.
// Converts physics curves into cubic polynomials for DirectComposition IDCompositionAnimation::AddCubic.
// R-13: No unsafe.
#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicSegment {
    pub t_start: f32,
    pub t_end: f32,
    /// Polynomial: y(t) = c0 + c1*(t-t0) + c2*(t-t0)^2 + c3*(t-t0)^3
    pub c0: f32,
    pub c1: f32,
    pub c2: f32,
    pub c3: f32,
}

impl CubicSegment {
    pub fn evaluate(&self, t: f32) -> f32 {
        let dt = (t - self.t_start).clamp(0.0, self.t_end - self.t_start);
        self.c0 + self.c1 * dt + self.c2 * dt * dt + self.c3 * dt * dt * dt
    }
}

#[derive(Debug, Clone)]
pub struct BakedAnimation {
    pub duration: f32,
    pub segments: Vec<CubicSegment>,
}

impl BakedAnimation {
    pub fn evaluate(&self, t: f32) -> f32 {
        if self.segments.is_empty() {
            return 0.0;
        }
        if t <= self.segments[0].t_start {
            return self.segments[0].c0;
        }
        for seg in &self.segments {
            if t <= seg.t_end {
                return seg.evaluate(t);
            }
        }
        let last = &self.segments[self.segments.len() - 1];
        last.evaluate(last.t_end)
    }
}

/// Evaluates underdamped spring response analytically:
/// Returns (position, velocity) at time t.
pub fn evaluate_spring(
    stiffness: f32,
    damping: f32,
    mass: f32,
    x0: f32,
    xt: f32,
    v0: f32,
    t: f32,
) -> (f32, f32) {
    if t <= 0.0 {
        return (x0, v0);
    }
    let omega_0 = (stiffness / mass).sqrt();
    let zeta = damping / (2.0 * (stiffness * mass).sqrt());

    if zeta < 1.0 {
        let omega_d = omega_0 * (1.0 - zeta * zeta).sqrt();
        let a = x0 - xt;
        let b = (v0 + zeta * omega_0 * a) / omega_d;
        let decay = (-zeta * omega_0 * t).exp();
        let cos_val = (omega_d * t).cos();
        let sin_val = (omega_d * t).sin();

        let pos = xt + decay * (a * cos_val + b * sin_val);
        let vel = -zeta * omega_0 * decay * (a * cos_val + b * sin_val)
            + decay * (-a * omega_d * sin_val + b * omega_d * cos_val);
        (pos, vel)
    } else {
        (xt, 0.0)
    }
}

/// Bake closed-form spring transition into a sequence of cubic segments (max dt <= 20ms <= 24ms limit).
pub fn bake_spring(
    stiffness: f32,
    damping: f32,
    mass: f32,
    from: f32,
    to: f32,
    initial_vel: f32,
    duration: f32,
) -> BakedAnimation {
    let segment_dt = 0.020; // 20 ms segments
    let mut segments = Vec::new();

    let mut t = 0.0;
    while t < duration {
        let t_next = (t + segment_dt).min(duration);
        let dt = t_next - t;
        if dt <= 0.0001 {
            break;
        }

        let (p0, v0) = evaluate_spring(stiffness, damping, mass, from, to, initial_vel, t);
        let (p1, v1) = evaluate_spring(stiffness, damping, mass, from, to, initial_vel, t_next);

        // Convert Hermite boundary conditions (p0, v0, p1, v1) to polynomial coefficients:
        // P(u) = c0 + c1*u + c2*u^2 + c3*u^3 where u in [0, dt]
        let c0 = p0;
        let c1 = v0;
        let c2 = (3.0 * (p1 - p0) - (2.0 * v0 + v1) * dt) / (dt * dt);
        let c3 = (2.0 * (p0 - p1) + (v0 + v1) * dt) / (dt * dt * dt);

        segments.push(CubicSegment {
            t_start: t,
            t_end: t_next,
            c0,
            c1,
            c2,
            c3,
        });

        t = t_next;
    }

    BakedAnimation { duration, segments }
}

/// Bake standard cubic-bezier (e.g. (0.2, 0, 0, 1)) easing into cubic segments
pub fn bake_easing(from: f32, to: f32, duration: f32) -> BakedAnimation {
    let segment_dt = 0.020;
    let mut segments = Vec::new();
    let mut t = 0.0;

    while t < duration {
        let t_next = (t + segment_dt).min(duration);
        let dt = t_next - t;
        if dt <= 0.0001 {
            break;
        }

        // Cubic bezier approximation (0.2, 0.0, 0.0, 1.0)
        let progress0 = (t / duration).clamp(0.0, 1.0);
        let progress1 = (t_next / duration).clamp(0.0, 1.0);
        let ease0 = 1.0 - (1.0 - progress0).powi(3);
        let ease1 = 1.0 - (1.0 - progress1).powi(3);

        let p0 = from + (to - from) * ease0;
        let p1 = from + (to - from) * ease1;
        let v0 = if t > 0.0 {
            3.0 * (1.0 - progress0).powi(2) * (to - from) / duration
        } else {
            0.0
        };
        let v1 = 3.0 * (1.0 - progress1).powi(2) * (to - from) / duration;

        let c0 = p0;
        let c1 = v0;
        let c2 = (3.0 * (p1 - p0) - (2.0 * v0 + v1) * dt) / (dt * dt);
        let c3 = (2.0 * (p0 - p1) + (v0 + v1) * dt) / (dt * dt * dt);

        segments.push(CubicSegment {
            t_start: t,
            t_end: t_next,
            c0,
            c1,
            c2,
            c3,
        });

        t = t_next;
    }

    BakedAnimation { duration, segments }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens;

    #[test]
    fn test_baking_error_under_half_pixel() {
        let expand = &tokens::spring::EXPAND;
        let anim = bake_spring(
            expand.stiffness,
            expand.damping,
            expand.mass,
            380.0,
            864.0,
            0.0,
            0.350,
        );

        let mut max_err: f32 = 0.0;
        let mut t = 0.0;
        while t <= 0.350 {
            let baked_val = anim.evaluate(t);
            let (analytical_val, _) = evaluate_spring(
                expand.stiffness,
                expand.damping,
                expand.mass,
                380.0,
                864.0,
                0.0,
                t,
            );
            let err = (baked_val - analytical_val).abs();
            if err > max_err {
                max_err = err;
            }
            t += 0.001; // 1 ms sampling
        }

        assert!(
            max_err <= 0.5,
            "Baking error exceeds 0.5 px tolerance: max_err = {max_err:.4} px"
        );
    }

    #[test]
    fn test_interruption_continuity() {
        // Test interruption at t = 100 ms while expanding
        let expand = &tokens::spring::EXPAND;
        let collapse = &tokens::spring::COLLAPSE;

        let interrupt_t = 0.100;
        let (current_pos, current_vel) = evaluate_spring(
            expand.stiffness,
            expand.damping,
            expand.mass,
            380.0,
            864.0,
            0.0,
            interrupt_t,
        );

        // Reverse back towards 380.0 with collapse spring
        let reversed_anim = bake_spring(
            collapse.stiffness,
            collapse.damping,
            collapse.mass,
            current_pos,
            380.0,
            current_vel,
            0.300,
        );

        let initial_reversed_pos = reversed_anim.evaluate(0.0);
        let pos_delta = (initial_reversed_pos - current_pos).abs();
        assert!(
            pos_delta <= 0.5,
            "Interruption position jump exceeds 0.5 px: delta = {pos_delta:.4} px"
        );
    }
}
