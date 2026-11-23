# Testing Guide — Nuvra

## Run All Tests (one command)

```powershell
cargo fmt --check && cargo clippy -- -D warnings && cargo nextest run --workspace && cargo audit && cargo deny check
```

## Individual Commands

| Command | Purpose |
|---|---|
| `cargo nextest run --workspace` | All unit and integration tests |
| `cargo test --doc` | Doc-tests |
| `cargo bench` | Criterion benchmarks (render_perf) |
| `cargo llvm-cov --workspace` | Coverage report |
| `cargo audit` | Known CVE check |
| `cargo deny check` | License + banned crate policy |
| `cargo clippy -- -D warnings` | Lint, zero-warning policy |
| `cargo fmt --check` | Format check |

## Coverage Target

≥ 80% line coverage on non-`unsafe` logic crates (Section 10, D-STK-08). Measured with `cargo llvm-cov`.

## Benchmark Targets (Section 8 — Floor machine)

| Metric | Target |
|---|---|
| Pointer-enter to first frame submitted | ≤ 8 ms p95 |
| CPU per frame | ≤ 4 ms |
| Idle CPU | ≈ 0% |
| Cold start | ≤ 400 ms |

Benchmarks run via `cargo bench`. ETW-based frame timing script `tools/etw_timing.ps1` is not yet available (planned for release benchmarking; UNVERIFIED until available and run by owner on Floor machine).

## Compatibility Matrix (D-GLB-10 — Phase 0)

Items marked UNVERIFIED require manual testing by the owner using the scripts in `docs/owner-test-scripts/phase-0-test-script.md`.

| Test | Status |
|---|---|
| Windows 10 22H2 | UNVERIFIED — requires Floor machine |
| Windows 11 | UNVERIFIED |
| 60 Hz display | UNVERIFIED |
| 144 Hz display | UNVERIFIED |
| 100% DPI | UNVERIFIED |
| 125% DPI | UNVERIFIED |
| 150% DPI | UNVERIFIED |
| 200% DPI | UNVERIFIED |
| WARP software renderer | UNVERIFIED — set `NUVRA_FORCE_WARP=1` |
| RTL layout (Arabic) | UNVERIFIED |
| CJK / Tamil / Hindi shaping | UNVERIFIED |
| Pseudo-locale | UNVERIFIED |
| Touch input | UNVERIFIED |
| Screen reader (Narrator) | UNVERIFIED |
| High contrast mode | UNVERIFIED |
| Offline operation | UNVERIFIED |
| Sleep/resume | UNVERIFIED |

## Golden-Image Tests (Phase 0 spike)

The renderer validation spike (`nuvra-render/src/spike.rs`) produces frames for Idle, Ambient, Glance, and Expanded states. Hardware and WARP outputs must match within 1% pixel tolerance (D-PERF-05).

Golden images are stored in `tools/golden/` (committed after first successful Floor run).
