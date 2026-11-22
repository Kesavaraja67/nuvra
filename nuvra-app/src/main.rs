// Nuvra — main entry point
// ADR-0001 (Rust), ADR-0002 (renderer), ADR-0003 (threading/event bus)
// R-13: unsafe is ONLY in nuvra-platform-win; this crate is safe.
#![forbid(unsafe_code)]

fn main() {
    // Phase 0 spike placeholder — real host loop implemented in v1.
    // This file is intentionally minimal; the renderer validation spike
    // lives in nuvra-render/src/spike.rs (Phase 0 scope only).
    println!("Nuvra Phase 0 — workspace scaffold verified.");
}
