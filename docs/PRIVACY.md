# Nuvra Privacy Policy (Technical Document)

*This is the engineering privacy specification. The owner must have this reviewed by legal counsel before publishing a user-facing privacy policy (D-GLB-09).*

## Data Collected and Stored

| Data | Location | Encryption | Retention | How to wipe |
|---|---|---|---|---|
| Clipboard history | `%LOCALAPPDATA%\Nuvra\data.db` | AES-256-GCM, DPAPI-sealed key | 7 days max (D-DAT-02) | Settings → Privacy → Wipe clipboard |
| File shelf references | `%LOCALAPPDATA%\Nuvra\data.db` | AES-256-GCM, DPAPI-sealed key | Until removed | Settings → Shelf → Clear all |
| App configuration | `%APPDATA%\Nuvra\config.toml` | None (no sensitive data) | Until uninstall | Uninstall removes unless user opts to keep |
| Crash minidumps | `%LOCALAPPDATA%\Nuvra\crashes` | None | Last 3 crashes | Settings → About → Clear crash reports |
| Log files | `%LOCALAPPDATA%\Nuvra\logs` | None — content-free (R-20) | 7 days rolling | Deleted automatically |

## Data Never Collected
- No telemetry, analytics, or tracking IDs (R-19).
- No network calls except the explicit opt-in list (D-NET-01).
- Logs never contain clipboard text, notification text, file paths, keys, or tokens (R-20).

## Optional Network Features (v1)
- **Lyrics (opt-in):** Sends title/artist/album/duration to LRCLIB. No account. Requires explicit opt-in. Disabled by default.

## User Rights (D-GLB-09)
Users can at any time:
- **Export** all stored data: Settings → Privacy → Export data (content-free JSON).
- **Delete** all stored data: Settings → Privacy → Wipe all data.
- **Uninstall:** removes `%LOCALAPPDATA%\Nuvra` and `%APPDATA%\Nuvra` unless the user opts to keep data.

## Owner Responsibilities
- Legal review of the user-facing privacy policy (D-GLB-09 — owner, not the agent).
- Store rating age-gate (no children's data knowingly collected).
- GDPR/DPDP/CCPA/LGPD compliance determinations.
