# ADR-0008 — Plugin Sandbox Plan (v3)

**Status:** Accepted (plan recorded; implementation in v3)  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context (v3 planning, recorded in Phase 0)

Third-party plugins must not crash or slow the host, must not access the filesystem arbitrarily, and must not circumvent Nuvra's design system (R-26, R-28).

## Decision

**WASM via `wasmtime` (Component Model, WIT interface `nuvra:plugin`). Capability-declared in a manifest. CPU fuel-metered. Hard kill at 50 ms wall time per event.**

## Declared Capabilities (all off unless granted)

- `ui.render`: declarative widgets only (text, icon, button, list, progress, image ≤ 128 KB). No custom drawing.
- `net.fetch`: host allowlist.
- `storage`: 5 MB KV.
- `clipboard.read`: explicit per use.
- `files.pick`: user-chosen files only.
- `timers`.

## Limits

- 16 MB memory per plugin.
- Fuel-metered CPU: 5 ms per event; hard kill at 50 ms wall time.
- SDK: 10 updates/s and 3 live activities per app.

## Consequences

- Plugin packages: `.nvx` (zip: `manifest.toml`, `plugin.wasm`, assets, Ed25519 signature).
- Unsigned packages require an explicit user warning flow.
- Sandbox-escape tests documented and must fail (D-SEC-05).
- `wasmtime` is on the approved crates list for v3 only (D-ARC-01).
- Run `system-design` and `architecture` skills before v3 SDK/plugin implementation.

## Implementation Plan

- v3: `nuvra-sdk/`, WIT definitions, `wasmtime` integration.
- Phase 0: this ADR records the design decision only.
