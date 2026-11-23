// Nuvra — main entry point
// ADR-0001 (Rust), ADR-0002 (renderer), ADR-0003 (threading/event bus)
// R-13: unsafe is ONLY in nuvra-platform-win; this crate is safe.
#![forbid(unsafe_code)]

fn main() {
    println!("Nuvra Phase 0 — workspace scaffold verified.");

    if let Err(err) = nuvra_render::spike::run_spike() {
        eprintln!("Phase 0 spike failed: {err}");
        std::process::exit(1);
    }
}
