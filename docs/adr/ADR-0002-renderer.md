# ADR-0002 — Renderer: DirectComposition + D3D11 + D2D/DWrite

**Status:** Accepted  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context

The notch window must achieve ≤ 8 ms p95 pointer-to-frame latency, 0 dropped frames, ≤ 4 ms CPU per frame, and pass on the Floor machine (integrated GPU, 2015–2018 era) with the same full visual design as high-end hardware. The renderer must support compositor-driven spring transitions (D-PERF-03) so animation runs independent of app CPU.

## Decision

**DirectComposition + Direct3D 11 + Direct2D/DirectWrite. No wgpu, no Electron, no WebView for the notch window.**

## Alternatives Considered and Rejected

| Renderer | Rejection Reason |
|---|---|
| `wgpu` | Additional abstraction layer; no DirectComposition integration; compositor-driven animation (D-PERF-03-d) requires DC animation objects, which wgpu does not expose. |
| Electron / CEF | Multi-process overhead; GC pauses; > 100 MB RAM at idle — violates the ≤ 30 MB target. |
| WinUI 3 / MAUI | Cannot achieve `WS_EX_NOREDIRECTIONBITMAP` + `WS_EX_NOACTIVATE` compositing model needed for D-WIN-04. |
| WebView2 (notch) | Same concern as Electron for the notch; WebView2 IS used for the settings window (D-STK-04), never for the notch renderer. |

## Consequences

- `WS_EX_NOREDIRECTIONBITMAP` enables the layered composition model; content is composited by DWM directly.
- `IDCompositionAnimation` objects bake spring curves piecewise-cubic (D-PERF-03-d); motion runs on the compositor thread at display rate with no app CPU involvement.
- Hardware D3D11 primary; WARP fallback when no hardware adapter or in RDP (D-PERF-05).
- Golden-image tests prove hardware == WARP output within tolerance.
- Phase 0 spike validates this approach before v1 (Section 12.6).

## Implementation Plan

- `nuvra-render`: D3D11 device, swap chain (FLIP_SEQUENTIAL), DirectComposition visual tree.
- `nuvra-platform-win`: unsafe Win32 calls, all with `// SAFETY:` comments.
- `nuvra-render/src/spike.rs`: Phase 0 renderer validation — Idle→Ambient→Glance→Expanded with real springs, measured on Floor + WARP + High.
- If the spike misses Section 8 targets, STOP and open a decision (D-PERF-02).
