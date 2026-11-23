# ADR-0003 — Threading Model and Event Bus

**Status:** Accepted  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context

Nuvra's render thread must never block on I/O, locks, or allocation in the frame path (D-ARCH-01). Modules are independent; a crashing module must not take down the host (R-14).

## Decision

**Three thread domains with bounded `flume` channels as the event bus.**

1. **UI/render thread** — Win32 message pump + DirectComposition presentation. Reads `ModuleState` snapshots only. Never waits on I/O.
2. **Module worker threads** — one per module (or small tokio runtime for I/O modules). Produce `ModuleState`. Never draw.
3. **Event bus** — bounded `flume` channels. State flows one way: OS event → module → `ModuleState` → context engine → renderer (D-ARCH-03).

Polling banned (D-ARCH-04). Every module runs behind `catch_unwind` + restart-with-backoff (R-14).

## Alternatives Considered and Rejected

| Design | Rejection Reason |
|---|---|
| Single-threaded event loop | Audio visualizer at 240 Hz + clipboard events + notification callbacks all on one thread → frame drops. |
| `async` on the render thread | `tokio` runtime requires a Tokio context; Win32 message pump must own the render thread exclusively. |
| Shared mutable state with `Mutex` | Locks in the frame path violate D-ARCH-01. `arc-swap` for snapshots, `flume` for events. |

## Consequences

- `flume::bounded` channels: backpressure prevents unbounded memory growth.
- `arc-swap::ArcSwap<ModuleState>` for lock-free reads in the render path.
- Module trait (D-ARCH-02): `id`, `capabilities`, `init`, `on_event`, `state() -> ModuleState`, `shutdown`. Defined in `nuvra-core-shared`.
- Each module's worker thread holds a `flume::Receiver<OsEvent>` and a `flume::Sender<ModuleState>`.

## Implementation Plan

- `nuvra-core-shared/src/module.rs`: `Module` trait.
- `nuvra-core-shared/src/event.rs`: `OsEvent` enum, `ModuleState` enum.
- `nuvra-app/src/host.rs` (v1): spawns module threads, owns channels, runs restart-with-backoff supervisor.
