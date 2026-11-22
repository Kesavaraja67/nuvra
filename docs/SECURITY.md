# Nuvra Security Policy

## Threat Model (v0 — Phase 0)

**Updated each version per Section 9.**

### Assets
- Clipboard history (encrypted at rest, D-DAT-01)
- File shelf references (encrypted at rest)
- Config and hotkey bindings
- DPAPI-sealed encryption keys
- (v2+) Pairing tokens and device certificates

### Trust Boundaries
1. **Process boundary:** Nuvra.exe runs as the current Windows user, never elevated except for the documented one-time FPS ETW step (D-GAM-01, v1.5+). No admin rights requested at install or at runtime.
2. **IPC boundary:** Settings WebView2 communicates via `WebMessageReceived`; all messages validated.
3. **Network boundary (v1):** No network calls in v1 (lyrics opt-in in v1, requires D-NET-02 check first).
4. **OS event boundary:** Clipboard, mic, notifications received via documented OS APIs only. No global hooks except `RegisterHotKey` (R-21).

### Attackers
- Malicious processes on the same user session (can read unencrypted clipboard via OS — Nuvra encrypts its *stored* history; it cannot prevent the OS clipboard from being readable).
- Malicious dropped files (input validation on all paths — Section 9).
- (v2+) Network adversaries on LAN (mitigated by mTLS certificate pinning).

### Mitigations
- AES-256-GCM + DPAPI for all sensitive stored data.
- `cargo audit` + `cargo deny` clean per release.
- No telemetry, no network calls beyond the explicit allowlist (D-NET-01).
- R-20: Logs never contain clipboard text, notification text, file paths, tokens, or keys.
- R-21: No kernel drivers, no code injection, no hooking into other processes.

### Known Limitations (v0)
- Clipboard content is encrypted in Nuvra's database, but the OS clipboard itself is accessible to any process running as the same user — this is a Windows design constraint, not a Nuvra defect.
- Phase 0 has no network surface; threat model expands in v2.

---

## Responsible Disclosure

Report security issues to: `OWNER_FILL_support_email`

Do not file public issues for security vulnerabilities.
