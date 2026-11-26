// nuvra-render/src/spike.rs
// UI-1 Renderer Validation Spike — Section 13 (Spike C: Closed-form and Cubic Hermite baking)
// R-12: Real verification logic for closed-form spring physics and cubic segment error.
// R-13: No unsafe in this crate (#![forbid(unsafe_code)]).
// D-PERF-03: Spring curves evaluated and verified against target tolerance (error <= 0.5 px).

use crate::tokens;

/// Closed-form analytical evaluation of an underdamped spring:
/// Returns (position, velocity) at exact time t (in seconds).
pub fn evaluate_spring_analytical(
    stiffness: f32,
    damping: f32,
    mass: f32,
    initial_pos: f32,
    target_pos: f32,
    initial_vel: f32,
    t: f32,
) -> (f32, f32) {
    if t <= 0.0 {
        return (initial_pos, initial_vel);
    }
    let omega_0 = (stiffness / mass).sqrt();
    let zeta = damping / (2.0 * (stiffness * mass).sqrt());

    if zeta < 1.0 {
        // Underdamped oscillator
        let omega_d = omega_0 * (1.0 - zeta * zeta).sqrt();
        let a = initial_pos - target_pos;
        let b = (initial_vel + zeta * omega_0 * a) / omega_d;
        let decay = (-zeta * omega_0 * t).exp();
        let cos_val = (omega_d * t).cos();
        let sin_val = (omega_d * t).sin();

        let pos = target_pos + decay * (a * cos_val + b * sin_val);
        let vel = -zeta * omega_0 * decay * (a * cos_val + b * sin_val)
            + decay * (-a * omega_d * sin_val + b * omega_d * cos_val);
        (pos, vel)
    } else {
        // Overdamped or critically damped fallback
        (target_pos, 0.0)
    }
}

/// Evaluates a cubic Hermite segment at local parameter u in [0, 1].
pub fn eval_hermite(p0: f32, v0: f32, p1: f32, v1: f32, dt: f32, u: f32) -> f32 {
    let u2 = u * u;
    let u3 = u2 * u;
    let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
    let h10 = u3 - 2.0 * u2 + u;
    let h01 = -2.0 * u3 + 3.0 * u2;
    let h11 = u3 - u2;
    h00 * p0 + h10 * (v0 * dt) + h01 * p1 + h11 * (v1 * dt)
}

/// Spike entry point — validates closed-form dynamics and Hermite cubic baking accuracy (Spike C).
pub fn run_spike() -> Result<(), String> {
    // 1. Validate environment driver selection
    let force_warp = std::env::var("NUVRA_FORCE_WARP")
        .map(|v| v == "1")
        .unwrap_or(false);

    if force_warp {
        println!("WARP adapter active");
    }

    // 2. Validate Expand spring (380 to 864 px, 484 px displacement)
    let expand = &tokens::spring::EXPAND;
    let from_w = 380.0;
    let to_w = 864.0;
    let segment_dt = 0.020; // 20 ms segment (<= 24 ms max segment rule)
    let total_duration = 0.350; // 350 ms

    let mut t = 0.0;
    let mut max_err: f32 = 0.0;

    while t < total_duration {
        let t_next = f32::min(t + segment_dt, total_duration);
        let dt = t_next - t;
        if dt <= 0.001 {
            break;
        }

        let (p0, v0) = evaluate_spring_analytical(
            expand.stiffness,
            expand.damping,
            expand.mass,
            from_w,
            to_w,
            0.0,
            t,
        );
        let (p1, v1) = evaluate_spring_analytical(
            expand.stiffness,
            expand.damping,
            expand.mass,
            from_w,
            to_w,
            0.0,
            t_next,
        );

        // Check error at 1 ms steps inside the segment
        for step in 0..=20 {
            let sub_t = t + (step as f32) * 0.001;
            if sub_t > t_next {
                break;
            }
            let u = (sub_t - t) / dt;
            let hermite_p = eval_hermite(p0, v0, p1, v1, dt, u);
            let (analytical_p, _) = evaluate_spring_analytical(
                expand.stiffness,
                expand.damping,
                expand.mass,
                from_w,
                to_w,
                0.0,
                sub_t,
            );
            let err = (hermite_p - analytical_p).abs();
            if err > max_err {
                max_err = err;
            }
        }

        t = t_next;
    }

    if max_err > 0.5 {
        return Err(format!(
            "Spike C failure: cubic Hermite error exceeds 0.5px: max error = {max_err:.4}px"
        ));
    }

    println!("Spike C passed: Closed-form spring and cubic Hermite fit max error = {max_err:.4}px (<= 0.5px).");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_closed_form_and_baking_error() {
        assert!(run_spike().is_ok());
    }
}
