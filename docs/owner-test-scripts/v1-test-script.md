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
   - Sidebar tab navigation (General, Appearance, Modules, Hotkeys, Privacy, About) switches panels smoothly.
   - Theme dropdown toggles `data-theme` between `obsidian`, `paper`, and `signal`.
   - High contrast mode renders clear contrast borders conforming to WCAG AA.
