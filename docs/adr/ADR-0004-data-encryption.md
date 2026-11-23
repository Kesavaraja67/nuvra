# ADR-0004 — Data Storage and Encryption

**Status:** Accepted  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context

Sensitive payloads (clipboard text, notes, phone data) must be stored at rest with strong encryption. Keys must not persist in memory beyond their use window.

## Decision

**SQLite (bundled rusqlite) at `%LOCALAPPDATA%\Nuvra\data.db`. Config at `%APPDATA%\Nuvra\config.toml`. Sensitive payloads encrypted AES-256-GCM. Key sealed with Windows DPAPI (`CryptProtectData`, current-user scope). Keys zeroized after use (`zeroize`).**

## Consequences

- `rusqlite` with the `bundled` feature: no external SQLite install required.
- `aes-gcm` for payload encryption; `zeroize` on key material.
- DPAPI seals the AES key to the current Windows user account — data is inaccessible to other users and to the app when running as a different user.
- Clipboard limits: 200 items, 7 days, 1 MB text, 8 MB image, 100 MB total (D-DAT-02). Enforced at write time.
- Exclusion list (D-DAT-03): password managers honored via format flags and process denylist.
- Uninstall wipes `%LOCALAPPDATA%\Nuvra` and keys (D-GLB-09).

## Implementation Plan

- `nuvra-core-shared/src/db.rs`: typed schema, migrations, AES-GCM read/write helpers.
- `nuvra-platform-win/src/dpapi.rs`: `seal_key` / `unseal_key` wrappers with `// SAFETY:` comments.
- Schema version tracked in `config.toml`; migration runs at startup.
