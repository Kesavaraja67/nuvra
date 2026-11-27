# Nuvra v1 Manual Test Script (Owner)
**Target:** Nuvra Core v1  
**Author:** Nuvra Engineering Team  
**Governing Standard:** `nuvra-master-prompt.md` Section 14.8  

---

## 1. Prerequisites & Setup
- Windows 10 (22H2) or Windows 11 x64 machine.
- Rust toolchain 1.99.0 installed.
- PowerShell terminal open in the repository root `D:\projects\Nuvra`.

---

## 2. Test Cases

### Test 2.1 — Notch Launch & DirectComposition Lifecycle
1. Run in PowerShell:
   ```powershell
   cargo run -p nuvra-app
   ```
2. **Expected Behavior:**
   - The topmost borderless Notch window appears centered at the top edge of your primary monitor.
   - Clicks outside the notch visual pass through seamlessly to underlying windows (`WM_NCHITTEST` returns `HTTRANSPARENT`).
   - Move mouse over the notch: expands smoothly from `Ambient` to `Glance`.
   - Click the notch: expands smoothly to `Expanded`.
   - Move mouse away: collapses smoothly back after delay.
3. Press `Ctrl + C` in PowerShell to stop.

---

### Test 2.2 — Forced WARP Software Rasterization Fallback
1. Run in PowerShell:
   ```powershell
   $env:NUVRA_FORCE_WARP="1"
   cargo run -p nuvra-app
   ```
2. **Expected Behavior:**
   - Log shows `WARP adapter active`.
   - Notch window displays and animates smoothly using WARP rasterization with no GPU crashes or driver panics.

---

### Test 2.3 — Global Microphone Mute
1. Ensure your microphone is connected and active.
2. Run `cargo run -p nuvra-app`.
3. Press global hotkey: `Ctrl + Alt + M`.
4. **Expected Behavior:**
   - Windows microphone capture icon in taskbar shows muted.
   - Notch briefly displays the mute confirmation banner (1.5 s).
   - Press `Ctrl + Alt + M` again: microphone un-mutes.

---

### Test 2.4 — System Tray Icon & Menu
1. Locate the Nuvra icon in the Windows taskbar notification area (system tray).
2. Right-click the icon:
   - Menu shows: `Show / Hide Notch`, `Settings...`, `Quit Nuvra`.
3. Click `Quit Nuvra`: the application cleanly unhooks tray icon, closes DirectComposition resources, and exits.

---

### Test 2.5 — Settings Window Preview
1. Open `settings-ui/index.html` in Microsoft Edge or Chrome.
2. Verify:
   - Sidebar tab navigation (General, Appearance, Modules, Hotkeys, Privacy, About) switches panels smoothly via click and keyboard (`Tab` + `Enter`/`Space`).
   - Theme dropdown toggles `data-theme` between `obsidian`, `paper`, and `signal` while keeping selected-tab labels legible.
   - High contrast mode renders clear contrast borders conforming to WCAG AA.

---

### Test 2.6 — Physical Multi-Monitor Mixed-DPI Drag (`WM_DPICHANGED`)
1. **Monitor Setup:**
   - Connect two physical displays in Windows Settings → System → Display.
   - Set **Display 1 (Primary)** scaling to `100%` (96 DPI) and **Display 2 (Secondary)** scaling to `150%` or `200%` (144 or 192 DPI).
   - Ensure Nuvra is running via `cargo run -p nuvra-app`.
2. **Drag & DPI Switch Steps:**
   - Right-click and hold inside the notch, then move more than `4 px` to enter 2D move mode and drag horizontally toward Display 2 across the monitor boundary, releasing to drop.
   - Alternately, move/change primary display assignment or trigger a per-monitor DPI change across `100%` → `150%` → `200%` scaling while the notch is running.
   - Also hover and click to expand/collapse the notch after a DPI/display change.
3. **Expected `WM_DPICHANGED` Behavior:**
   - When `WM_DPICHANGED` fires, `NotchWindow` updates its tracked `current_dpi` and applies the OS-suggested `RECT` via `SetWindowPos` without visual clipping, inverted clamping, stale hit-rects, white corner halos, or panics.
   - On `WindowEvent::DpiChanged` / `WindowEvent::DisplayChanged`, the app recentering handler recalculates `user_center_x` and resets `user_offset_y` to `0` against the current primary monitor bounds.
   - Dragged repositioning near center X or top edge snaps magnetically within `12 px` (`SNAP_RADIUS_PX`) without cursor misalignment.

---

## 3. Floor / Mid / High Benchmark & Coverage Procedure
> **Status:** `UNVERIFIED` until run by the owner on target hardware. Refer to `nuvra-master-prompt.md` §§8 & 10 and `docs/TESTING.md`.

### 3.1 Setup & Prerequisites
- Install `cargo-llvm-cov`: `cargo install cargo-llvm-cov --locked`
- Verify clean Release/Bench environment with Rust `1.99.0` (and `cargo-nextest` installed per `docs/owner-test-scripts/phase-0-test-script.md` §1.1).

### 3.2 Machine Profiles (`D-PERF-08`)
Run the full benchmark and verification commands below on each required hardware tier:
- **Floor (Hardware & Forced WARP):** Windows 10 22H2 (or constrained VM: `2` vCPUs, `4 GB` RAM); run once on native adapter and once with `$env:NUVRA_FORCE_WARP="1"`.
- **Mid:** Standard Windows 10/11 desktop or laptop integrated GPU (`60 Hz` display).
- **High:** Discrete GPU system, `144 Hz+` display, physical multi-monitor mixed-DPI layout.

### 3.3 Benchmark & Coverage Commands
```powershell
# 1. Criterion microbenchmarks (token & spring lookup latency)
cargo bench -p nuvra-render --bench render_perf

# 2. Renderer spike convergence validation (Hardware, then WARP)
Remove-Item Env:\NUVRA_FORCE_WARP -ErrorAction SilentlyContinue
$env:NUVRA_RUN_SPIKE="1"; cargo run -p nuvra-app
$env:NUVRA_FORCE_WARP="1"; cargo run -p nuvra-app
Remove-Item Env:\NUVRA_FORCE_WARP, Env:\NUVRA_RUN_SPIKE -ErrorAction SilentlyContinue

# 3. Oneshot frame smoke verification
$env:NUVRA_TEST_ONESHOT="1"; cargo run -p nuvra-app
$env:NUVRA_FORCE_WARP="1"; cargo run -p nuvra-app
Remove-Item Env:\NUVRA_FORCE_WARP, Env:\NUVRA_TEST_ONESHOT -ErrorAction SilentlyContinue

# 4. Line coverage on workspace crates
cargo llvm-cov --workspace --lcov --output-path lcov.info
```
*(Note: The ETW / `QueryPerformanceCounter` end-to-end frame timing script `tools/etw_timing.ps1` referenced in `docs/TESTING.md` is not yet present in the repository and remains unverified until added and run by the owner.)*

### 3.4 Acceptance Criteria (Supported Project Requirements)
- **Criterion (`render_perf`):** Completes with zero errors and no regression `> 10%`; token and spring parameter lookups align with the sub-`10 ns` target (`D-PERF-01`).
- **Renderer Spike & WARP Parity (`D-PERF-05`):** Spring curve convergence error `<= 0.5 px`; Hardware and WARP outputs remain within `1%` pixel tolerance.
- **Section 8 Runtime Contract (measured on Floor, WARP-forced Floor, Mid, High):**
  - Pointer-enter / hotkey event to first frame submitted: `<= 8 ms` p95
  - Animation frame CPU budget: `<= 4 ms` per frame with `0` dropped frames at refresh rate
  - Idle CPU (`Idle`/`Collapsed`, nothing animating): `≈ 0%`
  - Idle RAM working set (v1): `<= 30 MB` (`<= +10 MB` per enabled opt-in module)
  - Cold start to usable: `<= 400 ms`
- **Line Coverage (`D-STK-08` / §10):** `>= 80%` line coverage across non-`unsafe` logic crates in `lcov.info`.
