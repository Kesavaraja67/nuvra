# Nuvra

A fast, uncluttered notch bar for Windows 10/11 with an Android companion.

## Overview

Nuvra is built as a native Windows utility (Rust, Win32/WinRT, Direct3D 11, DirectComposition) designed with a "bezel extension" physical aesthetic:
- **Zero clutter**: quiet when nothing is active; expands on intent or live events.
- **Native performance**: hardware compositor-driven animations with software WARP fallback.
- **Privacy-first**: local-only storage (SQLCipher + DPAPI); zero telemetry; content is never logged.

## Project Structure

```
d:\projects\Nuvra
├── nuvra-app              Host executable and application lifecycle
├── nuvra-render           DirectComposition/D3D11 renderer & design tokens
├── nuvra-platform-win     Platform bindings (Win32, DPAPI, DXGI, HotKey)
├── nuvra-core-shared      Shared events, module traits, config, and schema
├── nuvra-mod-media        Media session module (GSMTC)
├── nuvra-mod-mic          Microphone mute and call detection
├── nuvra-mod-notifications Notification banner module (WinRT)
├── nuvra-mod-shelf        File shelf / drop-target module
├── nuvra-mod-clipboard    Clipboard history module (encrypted)
├── locales                Fluent i18n localization catalogs
├── settings-ui            Settings interface
└── docs                   Architecture decision records (ADRs), specs, and testing
```

## Documentation

- Architecture Decision Records: [`docs/adr/`](docs/adr/)
- Design System: [`docs/DESIGN_SYSTEM.md`](docs/DESIGN_SYSTEM.md)
- Security Model: [`docs/SECURITY.md`](docs/SECURITY.md)
- Privacy Policy: [`docs/PRIVACY.md`](docs/PRIVACY.md)
- Setup Guide: [`docs/SETUP.md`](docs/SETUP.md)
- Testing Strategy: [`docs/TESTING.md`](docs/TESTING.md)

## License

Proprietary. All rights reserved.
