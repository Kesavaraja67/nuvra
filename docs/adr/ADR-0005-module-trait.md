# ADR-0005 — Module Trait

**Status:** Accepted  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context

Modules must be independently implementable, restartable on failure, and forbidden from drawing. The host must be able to query their state without coupling to their internal logic.

## Decision

**All modules implement one `Module` trait (D-ARCH-02):**

```rust
pub trait Module: Send + 'static {
    fn id(&self) -> &'static str;
    fn capabilities(&self) -> &[Capability];
    fn init(&mut self, tx: flume::Sender<ModuleEvent>) -> Result<(), ModuleError>;
    fn on_event(&mut self, event: OsEvent) -> Result<(), ModuleError>;
    fn state(&self) -> ModuleState;  // Returns cheap plain-data snapshot — never blocks.
    fn shutdown(&mut self);
}
```

Modules never draw. The renderer reads `ModuleState` (plain data). State flows one way (D-ARCH-03).

## Consequences

- `ModuleState` is a data-only enum, cheaply cloneable.
- `state()` is called from the render path via `arc-swap` snapshot — zero blocking.
- Modules wrapped in `catch_unwind` + restart-with-backoff by the host supervisor.
- New modules add a crate (`nuvra-mod-*`) + implement `Module` — no changes to `nuvra-render`.

## Implementation Plan

- `nuvra-core-shared/src/module.rs`: `Module` trait, `ModuleState` enum, `ModuleEvent`, `Capability`, `ModuleError`.
- `nuvra-core-shared/src/event.rs`: `OsEvent` enum.
