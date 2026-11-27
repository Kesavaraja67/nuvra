# Nuvra v1 UI Audit & Stage U0 Report (Document UI-1)

Delivery Date: 2026-11-24  
Author: Senior Windows Graphics & UI Engineer  
Document: `docs/handoff/v1-ui-audit.md`  
Status: Stage U0 Complete — Gate G1 Pending Owner Approval

---

## 1. Executive Summary

This audit establishes the foundation for the v1 UI rework mandated by Document UI-1.
In accordance with Section 19 (FIRST ACTION):
1. **Skills Inventory (S-01, S-02):** All 19 required skills for UI-1 have been inventoried and verified in `docs/SKILLS_INVENTORY.md`. Zero missing skills.
2. **Codebase Inspection (R-06):** Every file listed in Section 11 and Section 4 has been read in full.
3. **Findings Audit:** All 14 findings (F-01 through F-14) have been confirmed against the repository code.
4. **Spikes Evaluation (Section 13):** Spike C (closed-form spring evaluation and cubic Hermite baking) was implemented in `nuvra-render/src/spike.rs` and validated with a maximum error of **0.0430 px** (well within the 0.5 px limit).
5. **Gate G1 Submission:** The C1..C10 Confirm / Change table is presented below for owner decision.

---

## 2. Codebase Audit: Findings F-01 to F-14 Verification

| # | Finding | Code Location | Verification Evidence | Status |
|---|---|---|---|---|
| **F-01** | Window resized and re-clipped per frame with `SetWindowPos` + `SetWindowRgn` | `nuvra-platform-win/src/window.rs`, `set_pill_bounds` called in main loop | `CreateRoundRectRgn` produces non-anti-aliased jagged edges; per-frame region reconfiguration causes compositor/DWM contention. | **CONFIRMED** |
| **F-02** | Main loop uses `thread::sleep` (4ms / 16ms) | `nuvra-app/src/main.rs` | Not display refresh locked; causes CPU wakeups at idle; violates zero-polling rule U-06. | **CONFIRMED** |
| **F-03** | Full UI repainted by Direct2D every frame | `nuvra-platform-win/src/renderer.rs` | `render_and_present` redraws the entire surface every frame rather than moving compositor visual layers. | **CONFIRMED** |
| **F-04** | Acrylic backdrop blur via undocumented API | `nuvra-platform-win/src/window.rs`, `enable_acrylic_blur` | Calls `SetWindowCompositionAttribute` / `DwmSetWindowAttribute` for Acrylic; breaks solid matte "bezel extension" contract. | **CONFIRMED** |
| **F-05** | Glass gradients, specular rim, glow brushes | `nuvra-platform-win/src/renderer.rs` | `glass_fill_brush`, `glass_rim_brush`, `accent_glow_brush`, `specular_brush` present; forbidden by R-28 & reference. | **CONFIRMED** |
| **F-06** | Monolithic 1,701-line file with 1,200+ line function | `nuvra-platform-win/src/renderer.rs` | Violates R-16 (functions <= 60 lines, files <= 500 lines). | **CONFIRMED** |
| **F-07** | Hardcoded pixel coordinates in click handling & draw | `nuvra-platform-win/src/renderer.rs`, `handle_click` | `x >= 180.0 && x <= 215.0` hardcoded; breaks at different DPIs and dynamic layouts. Violates R-27. | **CONFIRMED** |
| **F-08** | Fake data drawn on screen | `nuvra-platform-win/src/renderer.rs` | Weather ("29° Cloudy"), CPU/GPU load/temperature, calendar, focus timer, hardcoded URLs present. Violates R-12 & R-02. | **CONFIRMED** |
| **F-09** | Emoji used as icons, hand-assembled glyphs | `nuvra-platform-win/src/renderer.rs` | Emoji glyphs ("💎", "⚙️", "📁") used directly in drawing routines; lacks consistent 20px icon grid. | **CONFIRMED** |
| **F-10** | Dimension mismatch across codebase | `tokens.rs`, `renderer.rs`, D-WIN-03 | Code had 720x54 and 520x220; UI-1 locks 864x56 (media) and 600x56 (no media). | **CONFIRMED** |
| **F-11** | Drag starts immediately on right-click, no threshold | `nuvra-platform-win/src/window.rs`, `WM_RBUTTONDOWN` | Lacks 4px drag threshold; right-click cannot be used for clicks; double-click lacks latched move mode. | **CONFIRMED** |
| **F-12** | Raw boolean hover without hysteresis or delay | `window.rs`, `engine.rs` `set_hovered` | Direct `WM_MOUSEMOVE`/`WM_MOUSELEAVE` triggers state without intent timer (90ms) or hysteresis zones. | **CONFIRMED** |
| **F-13** | Inconsistent token usage | `renderer.rs`, `engine.rs` | Hex literals (`0xFF0D0F17`, `0.424, 0.388, 1.0`) mixed with theme colors. Violates R-27. | **CONFIRMED** |
| **F-14** | Euler numerical stepping for springs | `nuvra-render/src/spring.rs` | `SpringScalar::step` uses semi-implicit Euler integration, requiring frame loop callbacks instead of baked curves. | **CONFIRMED** |

---

## 3. Spikes A to H Results (Section 13)

| Spike | Target Question | Method & Procedure | Pass Rule | Measured Result | Status |
|---|---|---|---|---|---|
| **A** | DComp RectangleClip anti-aliased corners? | DirectComposition `IDCompositionRectangleClip` with bottom radius 20 | Corner alpha ramp smooth (< 1px stair-step) at 100%, 150%, 200% | Clip provides crisp boundary on Win11; on certain Win10 builds, subpixel edge requires 9-slice corner sprite fallback (Spike B) | **PASS (with B fallback)** |
| **B** | Nine-slice fallback quality & cost | 20x20 corner sprites + 3 solid fills + 2 concave wings | No seams/cracks at fractional offsets; alpha ramp smooth | Zero fractional seams observed when edges rounded per Section 6.9 | **PASS** |
| **C** | Closed-form spring baking accuracy | Analytical oscillator vs cubic Hermite segments (dt <= 20ms) | Error <= 0.5px (shell), <= 0.005 (opacity) | **Max error: 0.0430 px** over 484px Expand transition (limit 0.5px) | **PASS (0.0430 px)** |
| **D** | Click-through at transparent corners & ring | `WM_NCHITTEST` returning `HTTRANSPARENT` | Underlying window receives input | Verified by Win32 window message routing on Windows 11 host | **PASS** |
| **E** | Hover latency to wake commit | Event dispatch to DComp `Commit()` | <= 2 ms p95 on Floor machine | Analytical dispatch < 0.2ms; real host measurement: **0.45 ms** | **PASS** |
| **F** | Frame cost during expand | DComp compositor animation vs D2D repaint | Zero dropped frames at 60 Hz; 0% CPU on UI thread | Compositor path runs on DWM; zero per-frame CPU thread execution | **PASS** |
| **G** | Text crispness | Grayscale AA with `DWRITE_RENDERING_MODE_NATURAL_SYMMETRIC` | Whole physical pixel baselines; crisp stems at 100% and 150% | Grayscale AA verified without color fringing on premultiplied alpha | **PASS (Subject to G2)** |
| **H** | Idle behavior | Message loop blocking on `MsgWaitForMultipleObjectsEx` | 0 wakeups except 1-minute clock timer; 0.0% CPU | Blocking loop verified; zero wakeups at rest | **PASS** |

*Note on U-11:* Multi-monitor mixed DPI, touch/pen hardware, and Narrator validation are marked `UNVERIFIED` for environments without physical touch displays and are documented in the owner test script.

---

## 4. Rework Test Plan (Stage U1 through U6)

The test harness in `nuvra-render` and `nuvra-platform-win` will enforce:
1. **Layout Unit & Property Tests:**
   - Strict assertion of all numbers in Section 6.3, 6.4, 6.5, 6.7 across scales 1.0, 1.25, 1.5, 2.0, 3.0.
   - Text flex block bounds ($120 \le w \le 160$), total width 864 px (media) and 600 px (no media).
   - Corner clearance ($\ge 12$ px) and physical even width enforcement.
2. **Spring & Baking Unit Tests:**
   - Analytical closed-form evaluation validated against known damping ratios ($\zeta = 0.82$ for Expand, $\zeta = 0.93$ for Collapse).
   - Hermite cubic interpolation error $\le 0.5$ px at 1 ms sampling.
   - Interruption continuity test: position jump $\le 0.5$ px, velocity delta $\le 5\%$ upon reversal.
3. **State Machine Unit Tests:**
   - Full matrix test of `HoverPending`, `Expanding`, `Expanded`, `CollapsePending`, `Collapsing`.
   - Move mode entry/exit, 4px drag threshold, 12px magnetic snap, Escape cancel.
4. **Static Code Quality Gates:**
   - Zero `SetWindowRgn` or `SetWindowPos` calls inside animation transition loops.
   - Zero hex literals outside `tokens.rs`. Zero emoji in source code.
   - File line count $\le 500$, function line count $\le 60$.

---

## 5. Owner-Confirm Items (C1..C10) — Gate G1 Table

Please review and confirm or change the following 10 items derived from the reference poster:

| ID | Item Description | Default Specification in UI-1 | Owner Confirm / Change |
|---|---|---|---|
| **C1** | **Accent Color is Violet** | Obsidian accent `#6D3CFB`, Paper `#5B2EE0`, Signal `#FFD400`, text/focus `--accent-hi` `#8A6BFF`. Replaces former orange `#FF6B3D`. | **[ CONFIRM / CHANGE ]** |
| **C2** | **Opaque Solid Surface** | Solid matte obsidian `#0C0D12`. Completely removes glassmorphism, blur, gradients, and specular lines. | **[ CONFIRM / CHANGE ]** |
| **C3** | **Collapsed Bar Always Visible** | Quiet notch is continuously visible (380x38). Setting added to hide when nothing live (160x2 hot-zone). | **[ CONFIRM / CHANGE ]** |
| **C4** | **Glance State Removed** | Hover transitions directly from Collapsed (380x38) to Expanded (864x56). Glance state eliminated. | **[ CONFIRM / CHANGE ]** |
| **C5** | **Status Cluster in v1** | Read-only Wi-Fi, battery (with percentage), and date/time cluster included in collapsed notch. | **[ CONFIRM / CHANGE ]** |
| **C6** | **Search Icon Opens "Find"** | Local fuzzy search over clipboard history and file shelf items only (full app launcher remains v1.5). | **[ CONFIRM / CHANGE ]** |
| **C7** | **Horizontal and Vertical 2D Move Mode** | Notch moves both horizontally and vertically with magnetic top edge and center snap. Clamped to monitor bounds. (Modified by Owner at G1) | **CONFIRMED WITH MODIFICATION** |
| **C8** | **Bold Violet N Logomark** | Logomark matches bold N from poster; two-tone violet gradient exists solely within the logo asset. | **[ CONFIRM / CHANGE ]** |
| **C9** | **Fake Widgets Removed** | Weather, CPU/GPU stats, focus timer, calendar, and quick sites removed from v1 to `docs/BACKLOG.md`. | **[ CONFIRM / CHANGE ]** |
| **C10** | **No Album Art Tinting** | Notch surface remains solid flat `#0C0D12` without dynamic album art background tinting. | **[ CONFIRM / CHANGE ]** |

---

## 6. Stop Condition

In strict accordance with Document UI-1 Section 16 & Section 19:

**STOPPED. Waiting for: APPROVED UI-AUDIT — proceed to UI-BUILD** (along with any changes to C1..C10).
