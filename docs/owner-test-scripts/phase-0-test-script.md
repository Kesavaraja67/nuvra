# Phase 0 Owner Test Script
# `docs/owner-test-scripts/phase-0-test-script.md`
#
# Applied skill: `interview-script` + `usability-test-plan` (Phase 0 scope, Section 12.8).
# Purpose: Give the owner everything needed to verify Phase 0 on real hardware
#          without ambiguity. All items below are UNVERIFIED by the agent (no network
#          in sandbox). The owner must run each step and record the result.

---

## 1. Prerequisites — set up once

### 1.1 Install tools (from `docs/SETUP.md`)

Run each of these in an elevated PowerShell after cloning the repo:

```powershell
# 1. Rust toolchain (pins to stable per rust-toolchain.toml)
rustup update stable
rustup component add clippy rustfmt

# 2. cargo-nextest (test runner)
cargo install cargo-nextest --locked

# 3. cargo-deny (license / advisory check)
cargo install cargo-deny --locked

# 4. cargo-audit (CVE check)
cargo install cargo-audit --locked

# 5. cargo-llvm-cov (coverage — optional for Phase 0)
cargo install cargo-llvm-cov --locked
```

### 1.2 Verify baseline build

```powershell
cd D:\projects\Nuvra
cargo build 2>&1 | Tee-Object build-log.txt
```

**Expected:** exits 0. No `error[E...]` lines.
**Record:** paste the last 10 lines of `build-log.txt` in your test log.

### 1.3 Verify lint + format

```powershell
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

**Expected:** both exit 0.

---

## 2. Unit and integration tests

```powershell
cargo nextest run --all
```

**Expected:** all tests pass. The only Phase 0 tests that exist are:
- `nuvra-mod-clipboard::exclusions::tests::excludes_known_password_managers` — PASS
- `nuvra-mod-clipboard::exclusions::tests::allows_normal_apps` — PASS

**Record:** paste the nextest summary line.

---

## 3. Dependency audit

```powershell
cargo deny check
cargo audit
```

**Expected:** `cargo deny check` exits 0 (no license violations, no bans).
`cargo audit` should show 0 vulnerabilities (new project; all deps are pinned by version).

**Record:** paste the final summary line of each.

---

## 4. DPAPI seal / unseal test

> Tests `nuvra-platform-win/src/dpapi.rs`. Run on a real Windows user session (not SYSTEM).

### 4.1 OS API Precheck (.NET ProtectedData)

Validate that DPAPI is operational in the current user session:

```powershell
Add-Type -AssemblyName System.Security
$plaintext = [System.Text.Encoding]::UTF8.GetBytes("nuvra-test-key-32-bytes-padding!!")
$scope = [System.Security.Cryptography.DataProtectionScope]::CurrentUser
$encrypted = [System.Security.Cryptography.ProtectedData]::Protect($plaintext, $null, $scope)
$decrypted = [System.Security.Cryptography.ProtectedData]::Unprotect($encrypted, $null, $scope)
[System.Text.Encoding]::UTF8.GetString($decrypted)
```

**Expected output:** `nuvra-test-key-32-bytes-padding!!`

### 4.2 Rust Wrapper Verification

Validate that `nuvra_platform_win::dpapi::seal_key` and `unseal_key` roundtrip successfully:

```powershell
cargo test -p nuvra-platform-win test_dpapi_roundtrip -- --nocapture
```

**Expected output:** `test dpapi::tests::test_dpapi_roundtrip ... ok` (1 passed; 0 failed).
*(Note: zeroization of the internal heap buffer before release is an internal security invariant verified by inspection of `slice.zeroize()` in `dpapi.rs`)*.

---

## 5. Hotkey registration test

> Tests `nuvra-platform-win/src/hotkey.rs`. Requires a visible desktop session.

### 5.1 OS API Precheck (C# P/Invoke)

```powershell
# Register Ctrl+Shift+N using native RegisterHotKey via a quick C# snippet:
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class HotKeyTest {
    [DllImport("user32.dll")] public static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);
    [DllImport("user32.dll")] public static extern bool UnregisterHotKey(IntPtr hWnd, int id);
    public static bool Test() {
        bool ok = RegisterHotKey(IntPtr.Zero, 9001, 0x0002 | 0x0004, 0x4E); // MOD_CTRL|MOD_SHIFT, VK_N
        if (ok) UnregisterHotKey(IntPtr.Zero, 9001);
        return ok;
    }
}
"@
[HotKeyTest]::Test()
```

**Expected:** `True`

### 5.2 Rust Wrapper Verification

Validate that `nuvra_platform_win::hotkey::HotKeyRegistration` enforces identifier bounds:

```powershell
cargo test -p nuvra-platform-win test_hotkey_invalid_id -- --nocapture
```

**Expected:** `test hotkey::tests::test_hotkey_invalid_id ... ok` (1 passed; 0 failed. A run reporting 0 passed is not a pass).

---

## 6. D3D11 / DXGI device creation test (Floor machine)

> Tests `nuvra-platform-win/src/dxgi.rs`. Validates ADR-0002.

### 6.1 Hardware adapter

```powershell
# Check D3D11 feature level using dxcap (included in Windows SDK):
dxcap -FeatureLevel
```

**Expected on Floor machine:** Feature Level 11.0 or above.

### 6.2 WARP fallback (forced software rasterizer)

```powershell
# Set the WARP flag before running the spike (Phase 0 only):
$env:NUVRA_FORCE_WARP = "1"
cargo run -p nuvra-app 2>&1
```

**Expected:** the app starts, detects `NUVRA_FORCE_WARP=1`, selects WARP directly, and logs `WARP adapter active`. Exits cleanly (exit code 0).

### 6.3 Hardware run

```powershell
Remove-Item Env:\NUVRA_FORCE_WARP
cargo run -p nuvra-app 2>&1
```

**Expected:** same behavior — Err returned from spike, no panic.

**Record for both:** exit code and last 5 lines of output.

---

## 7. Renderer spike state-machine test (Phase 0 spike)

The renderer spike (`nuvra-render/src/spike.rs`) computes and verifies the spring transitions across all four notch states (`Idle` → `Ambient` → `Glance` → `Expanded`) using the locked Section 6 spring curves.

```powershell
cargo run -p nuvra-app
```

**Expected:**
```
Nuvra Phase 0 — workspace scaffold verified.
Phase 0 renderer spike: spring curve convergence validated (tolerance <= 0.5px).
```
Exits with status code 0. If convergence fails, an error is printed to stderr and the process exits with code 1.

---

## 8. Edge cases & Accessibility

Work through each scenario manually in a Windows 10/11 VM or test machine:

| Scenario | Steps | Expected |
|---|---|---|
| Monitor hot-plug | Plug in / remove secondary monitor during app run | No crash; display refreshes on reconnect (v1) |
| Sleep / resume | Put machine to sleep; wake it | App not crashed; no log pollution (v1) |
| DPI change | Change display scale 100% → 150% → 200% while app runs | App not crashed (v1) |
| Wrong user session | Run as a different user (non-admin) | DPAPI key tied to that user's session only |
| Low-memory simulation | Reduce VM RAM to 1 GB | Phase 0: binary must still launch |

### 8.1 Arabic (RTL) Layout Check (D-GLB-01, D-GLB-03)
1. Add Arabic (e.g. `ar-SA`) in Windows Settings → Time & Language → Language.
2. Set display language or test app in Arabic locale.
3. Launch app: verify layout mirrors horizontally (right-to-left alignment, notch stays centered, text flows right-to-left).

### 8.2 Windows Narrator Screen Reader Check (D-GLB-04, Section 6.5)
1. Press `Win + Ctrl + Enter` to start Windows Narrator.
2. Focus the notch controls using `Tab` / arrow keys.
3. Verify Narrator reads out control Name, Role, State, and Value without silent focus traps or unannounced controls.
4. Press `Win + Ctrl + Enter` to exit Narrator.

**Phase 0 expectation:** binary launches and exits cleanly for every scenario. Full behavior is v1 scope.

---

## 9. Floor machine setup

Constrain a VM to simulate the Floor machine (D-PERF-01):

```
CPU:  2 vCPUs (no hyper-threading)
RAM:  4 GB
GPU:  WARP software rasterizer (disable guest GPU in VM settings)
OS:   Windows 10 22H2 (minimum supported)
```

Confirm WARP is active:

```powershell
$env:NUVRA_FORCE_WARP = "1"
cargo run -p nuvra-app
```

---

## 10. Security and privacy checklist

Inspect the installed app files and running process:

| Check | How | Expected |
|---|---|---|
| No plaintext keys on disk | Search `%APPDATA%\Nuvra\` for any `.key` or `.json` files | None (v1 scope; Phase 0 creates no data files) |
| No network calls | Run with Wireshark or Windows Firewall logs | Zero outbound connections (Phase 0 binary makes none) |
| No clipboard content in logs | Run app, copy sensitive text, check log file | No clipboard text in any log line (R-20) |
| Registry scope | Search `HKCU\Software\Nuvra` | Exists only under HKCU, not HKLM (R-24) |
| Binary size | Run `cargo build --release -p nuvra-app` then measure `(Get-Item .\target\release\Nuvra.exe).Length / 1MB` | Record the value (baseline for v1 comparison) |

---

## 11. Expected test results summary

| Test | Automated? | Expected Result | Record here |
|---|---|---|---|
| `cargo build` | Yes | Exit 0 | |
| `cargo clippy` | Yes | Exit 0, 0 warnings | |
| `cargo fmt --check` | Yes | Exit 0 | |
| `cargo nextest run` | Yes | 2/2 pass | |
| `cargo deny check` | Yes | Exit 0 | |
| `cargo audit` | Yes | 0 vulnerabilities | |
| DPAPI seal/unseal | Manual (§4) | Roundtrip passes | |
| Hotkey registration | Manual (§5) | True | |
| D3D11 hardware | Manual (§6.1) | FL ≥ 11.0 | |
| WARP forced run | Manual (§6.2) | No panic | |
| Spike error message | Manual (§7) | UNVERIFIED line | |
| Floor VM launch | Manual (§9) | No panic | |
| No network calls | Manual (§10) | 0 outbound | |

Fill in the "Record here" column and attach to your approval message or file as `docs/owner-test-scripts/phase-0-results.md`.
