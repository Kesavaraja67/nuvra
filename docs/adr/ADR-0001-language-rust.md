# ADR-0001 — Implementation Language: Rust

**Status:** Accepted  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context

Nuvra is an always-on, low-latency Windows notch bar. It hooks OS events (clipboard, mic, notifications), maintains persistent background threads, and must never leak memory or crash. The language choice determines safety, performance, and long-term maintenance cost.

## Decision

**Use Rust (stable toolchain, pinned in `rust-toolchain.toml`).**

## Alternatives Considered and Rejected

| Language | Rejection Reason |
|---|---|
| Go | GC pauses incompatible with ≤ 8 ms p95 frame latency target (Section 8). Weak Windows UI/API story — no idiomatic `windows-rs` equivalent. |
| C++ | Memory safety requires manual discipline; for a process that hooks clipboard and runs always-on, CVE risk is unacceptable. Higher maintenance cost per line. |
| C# / .NET | GC pauses (same concern as Go). JIT warm-up would miss the ≤ 150 ms cold-start target. |

## Consequences

- `windows-rs` crate provides zero-cost Windows API bindings (D-STK-02).
- `#![forbid(unsafe_code)]` on all crates except `nuvra-platform-win` (R-13).
- Toolchain pinned: reproducible builds, no silent upgrades.
- ARM64 / x86 cross-compilation straightforward (D-PRD-02, v3).

## Implementation Plan

- `rust-toolchain.toml` pins the channel.
- All crates declare `rust-version = "1.82"` in `Cargo.toml`.
- CI enforces `cargo fmt --check` and `cargo clippy -- -D warnings`.
