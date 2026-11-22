// nuvra-render/src/spike.rs
// Phase 0 Renderer Validation Spike — Section 12.6
//
// Purpose: validate D-STK-03 (DirectComposition + D3D11 + D2D) and D-PERF-03
// (compositor-driven spring transitions) on the Floor machine, a WARP-forced
// run, and the High machine. This file is PHASE 0 SCOPE ONLY and is removed
// before v1 delivery.
//
// UNVERIFIED: Requires the owner to run on Floor, WARP-forced, and High
// machines with the manual test script in docs/handoff/phase-0.md Section 9.
//
// R-12 Note: This is a validation spike, not a shipping feature. It is
// explicitly listed as Phase 0 scope and will be replaced by the real
// renderer in v1.
#![forbid(unsafe_code)]

/// Spike entry point — called from nuvra-app/src/main.rs in Phase 0.
/// Returns Ok(()) if the spike completed all state transitions.
/// Returns Err(msg) if any step failed (device creation, composition, etc.).
pub fn run_spike() -> Result<(), String> {
    // UNVERIFIED: Full implementation requires nuvra-platform-win unsafe
    // D3D11/DXGI/DirectComposition calls. Stubbed here — owner runs the
    // manual test script to exercise the real implementation.
    //
    // Steps this spike must demonstrate (manual verification):
    // 1. Create D3D11 device (hardware); measure creation time.
    // 2. Create DXGI swap chain (FLIP_SEQUENTIAL, NOREDIRECTIONBITMAP).
    // 3. Create DirectComposition device and visual tree.
    // 4. Transition Idle → Ambient → Glance → Expanded with real spring curves.
    // 5. Measure pointer-event-to-first-frame latency (target ≤ 8 ms p95).
    // 6. Measure frame CPU time (target ≤ 4 ms per frame).
    // 7. Confirm compositor handles animation independently of app thread.
    // 8. Repeat with WARP adapter (set NUVRA_FORCE_WARP=1).
    // 9. Confirm hardware == WARP golden-image output within 1% tolerance.

    Err("Phase 0 spike: UNVERIFIED — run manual test script docs/handoff/phase-0.md §9".into())
}
