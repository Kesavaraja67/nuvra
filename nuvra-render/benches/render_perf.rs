// nuvra-render/benches/render_perf.rs
// Criterion benchmarks for the renderer — D-PERF-02/03.
// Phase 0: harness structure only. Real benchmarks run in v1 against the live renderer.
//
// UNVERIFIED: Requires `cargo bench` on Floor/Mid/High hardware.
// See docs/owner-test-scripts/v1-test-script.md § 3.
//
// Target thresholds (D-PERF-01):
//   - Token access:            < 10 ns
//   - Spring parameter lookup: < 10 ns
//   - Frame CPU time (v1):     ≤ 4 ms p95 on Floor machine

use criterion::{criterion_group, criterion_main, Criterion};
use nuvra_render::tokens;

/// Validate that token lookups are essentially free — no allocation, no indirection.
fn bench_token_access(c: &mut Criterion) {
    c.bench_function("obsidian_accent_token", |b| {
        b.iter(|| criterion::black_box(tokens::obsidian::ACCENT))
    });

    c.bench_function("spacing_m_token", |b| {
        b.iter(|| criterion::black_box(tokens::spacing::M))
    });

    c.bench_function("spring_expand_params", |b| {
        b.iter(|| {
            let s = &tokens::spring::EXPAND;
            criterion::black_box((s.stiffness, s.damping, s.mass))
        })
    });
}

/// Validate that window dimension lookups are free.
fn bench_window_tokens(c: &mut Criterion) {
    c.bench_function("window_notch_dimensions", |b| {
        b.iter(|| {
            criterion::black_box((
                tokens::window::COLLAPSED_WIDTH,
                tokens::window::COLLAPSED_HEIGHT,
                tokens::window::EXPANDED_WIDTH,
                tokens::window::EXPANDED_HEIGHT,
                tokens::window::BANNER_WIDTH,
                tokens::window::BANNER_HEIGHT,
            ))
        })
    });
}

criterion_group!(benches, bench_token_access, bench_window_tokens);
criterion_main!(benches);
