# ADR-0006 — Settings Window: WebView2 + Vanilla TypeScript

**Status:** Accepted  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context

A settings window is needed for General, Appearance, Modules, Hotkeys, Privacy, and About panels. It must not be built with the notch renderer (D-STK-03 applies only to the notch). It should be lightweight to open and not leave a background process.

## Decision

**WebView2 (Evergreen) with vanilla TypeScript + CSS, bundled with esbuild. Created only when opened; destroyed on close (D-STK-04). No UI framework — no React, no Vue.**

The modern-web-guidance Chrome plugin guides the implementation toward modern platform APIs.

## Consequences

- WebView2 Evergreen bootstrapper included in the MSIX installer.
- If WebView2 is missing and cannot be installed (offline/S-mode), the settings window shows a clear message; the notch and all core modules still work (D-GLB-05).
- esbuild bundles `settings-ui/src/` → `settings-ui/dist/`; no runtime Node dependency.
- Communication: `window.chrome.webview.postMessage` / `WebMessageReceived` for host↔WebView2 IPC.
- Design tokens from Section 6 are mirrored as CSS custom properties.

## Implementation Plan

- `settings-ui/src/`: TypeScript source (vanilla, no framework).
- `settings-ui/dist/`: esbuild output (committed for now; CI regenerates).
- `nuvra-app/src/settings_window.rs` (v1): `WebView2Environment`, creation and destruction.
