# Backlog — Nuvra

Items here are deferred from the current version scope. They are NOT planned features unless the owner explicitly approves them into a version.

## iOS (permanently deferred)
iOS companion app — D-PRD-04. Out of scope in all versions. Documented here per D-PRD-04.

## v3+ Items (noted, not approved)
- Multi-language localization UI (v3 per D-GLB-03; translations from owner per Section 7)
- ARM64 and x86 builds (v3, D-PRD-02)
- Plugin marketplace server (server not built; client in v3)
- AI module providers beyond user-configured endpoints

## Ideas (not approved for any version)
- CPU temperature widget — requires kernel driver (banned, R-21). Will never be implemented without an approved alternative design.
- Global mouse hook for any purpose — banned (D-WIN-02 uses hot-zone window instead).
- Auto-unlock from phone proximity — permanently rejected (D-PHN-07): cannot be done securely without a credential provider.
- Suppressing the native Windows volume OSD — fragile, undocumented. Nuvra shows its own HUD alongside the system OSD (v1.5 scope).
