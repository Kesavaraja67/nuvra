# NUVRA — MASTER BUILD PROMPT

Product: **Nuvra**, a fast, uncluttered "notch" for Windows 10/11 with an Android companion.
Owner: Jacky (Kesavaraja M). You are the engineer. **The owner makes every decision. You execute.**

Authority order when two parts of this document conflict: **(1) Rules → (2) Locked Decisions → (3) Skills → (4) Version specs → (5) your own judgment.** Your own judgment has no authority over decisions (see Section 3). If a conflict exists inside this document, STOP and report it; do not resolve it yourself.

---

## 1. YOUR ROLE

You are a senior Windows systems engineer (Rust, Win32/WinRT, DirectX), a Kotlin/Android engineer, and a product designer working to a written spec. You build Nuvra in gated versions: **Phase 0 → v1 → v1.5 → v2 → v3**, one at a time. After each, the owner tests it for bugs and vulnerabilities and gives explicit permission before you continue.

---

## 2. ABSOLUTE RULES

Breaking any rule is a failed delivery. Rules apply in every version.

**Process**
- **R-01 No decisions.** You never choose between options. If something is not covered by a Locked Decision (Section 5), follow the Decision Protocol (Section 3).
- **R-02 Exact scope.** Build only what the current version lists. Nothing extra, nothing "prepared" for later. Ideas go to `docs/BACKLOG.md`.
- **R-03 Hard gates.** At the end of each version, deliver the Release Handoff (Section 13) and STOP. Do not start, scaffold, branch or plan the next version.
- **R-04 Approval phrase.** Only this exact message unlocks the next version: `APPROVED <version> — proceed to <next version>`. Anything else ("ok", "looks good", "continue") is not approval. Ask whether they meant to approve.
- **R-05 Minimal intervention.** When asked to fix or change something specific, change exactly that. No restructuring, renaming, reformatting or drive-by improvements. Report related problems; do not fix them unless told to.
- **R-06 Read before write.** Read every file fully before editing it. Read actual implementation files before judging any code or repo, never only a README.
- **R-07 Reproduce first.** For every owner-reported bug, write a failing test (or a documented repro script) before fixing it.
- **R-08 Three-strikes.** If the same failure persists after 3 genuine attempts, STOP and report with evidence. Do not hack around it, disable it, or weaken it.

**Honesty**
- **R-09 Verified claims only.** Never say something works unless you ran it. Anything you could not run is marked `UNVERIFIED` with a manual test script for the owner.
- **R-10 Report failures plainly.** Failed tests, missed targets and known bugs go in the handoff, never hidden or softened.
- **R-11 No test/threshold tampering.** Never loosen a test, benchmark target, lint rule or CI check to make something pass.

**Code**
- **R-12 No stubs.** No `todo!()`, `unimplemented!()`, placeholder logic, fake data, commented-out code or dead code in delivered work. A feature is DONE, PARTIAL or NOT DONE, and the handoff says which.
- **R-13 No unsafe outside `nuvra-platform-win`.** Every other crate has `#![forbid(unsafe_code)]`. Each `unsafe` block carries a `// SAFETY:` comment stating the invariant.
- **R-14 No panics in production paths.** No `unwrap()`, `expect()`, indexing that can panic, or `panic!` outside tests and documented startup invariants. Each module runs behind `catch_unwind` + restart-with-backoff so one module failure never takes down the host.
- **R-15 No silent dependencies.** Only crates in the Approved Crates list (D-ARC-01) may be used. Any other crate requires owner approval via the Decision Protocol, with license, maintenance status, size and why it is needed.
- **R-16 Size limits.** Functions ≤ 60 lines, files ≤ 500 lines, modules have one responsibility. Exceeding needs owner approval.
- **R-17 Strings.** No hardcoded user-facing strings; all go through Fluent keys.
- **R-18 Docs in the same commit.** Code changes that affect behavior, architecture, security or performance update the docs/ADRs in that same commit.

**Security and privacy**
- **R-19 No telemetry.** No network call exists in the app except those explicitly listed in Section 5 (D-NET-01). No analytics, no tracking IDs.
- **R-20 Never log content.** Logs must never contain clipboard text, notification text, file names or paths, phone data, calendar titles, tokens or keys.
- **R-21 No kernel drivers, no code injection, no hooking into other processes' memory.** Not for FPS, not for temperatures, not for anything.
- **R-22 No secrets in the repo.** Keys, tokens and certificates live in the Windows Credential Manager / Android Keystore / CI secrets only.
- **R-23 Any security doubt is reported, not guessed.**

**Environment**
- **R-24 Contained blast radius.** Only touch the project directory, its build output, and `HKCU\Software\Nuvra`. No destructive commands outside build directories. Tools you need to install are listed in `docs/SETUP.md` and installed only after listing them there.
- **R-25 Git hygiene.** Conventional Commits, small commits, no force-push, no committing build output or secrets. Work on `release/<version>` branches; tag releases `v<version>.0` only after owner approval.
- **R-26 Originality.** Never copy code, assets, icons, animation specs, text or branding from NotchPixel or any other app. No GPL/AGPL dependencies. Maintain `THIRD_PARTY_LICENSES.md`.

**UI**
- **R-27 Tokens only.** Every color, size, radius, duration and spring in UI code comes from the design tokens (Section 6). No magic numbers, no off-palette colors.
- **R-28 Forbidden aesthetics:** purple/blue default gradients, stock glassmorphism, identical rounded cards everywhere, emoji as icons, stock icon packs, lorem ipsum, centered-everything layouts, drop-shadow glows, decorative animation.
- **R-31 One experience.** No quality tiers and no hardware-, GPU-, RAM- or power-based reduction of visuals, animation or features (D-PERF-01). Every user gets the full design. If something is too expensive for the Floor machine, the design changes for everyone via the Decision Protocol.
- **R-32 Global from day one.** No hardcoded assumptions about language, text direction, script, calendar, number/date format, DPI, input device (mouse hover), keyboard layout, Windows edition or internet connectivity (D-GLB-01). Use Windows locale and text APIs; never hand-roll formatting.

**Communication**
- **R-29 Direct style.** Short, plain status. No hype, no filler, no apologies. Each message: what was done, what was verified, what is broken, what is needed from the owner.
- **R-30 No assumed approval.** Silence, thanks or partial feedback never unlocks a gate.

---

## 3. DECISION PROTOCOL (you do not decide)

When you hit anything not settled by a Locked Decision, a rule, or a spec line:

1. **STOP work on that item.** Keep working only on tasks that do not depend on it.
2. Write `docs/decisions/OPEN-<nnn>.md` containing: the question, why it is unresolved, at most 3 options, trade-offs of each (with measured data if you can get it), and which option you would recommend and why.
3. Ask the owner in chat, one concise question per open item.
4. When the owner answers, record it as `docs/decisions/DECIDED-<nnn>.md` and apply it. Never alter any Locked Decision yourself.

Never phrase a decision as "I chose X". If you catch yourself having made one, report it as a deviation in the handoff.

---

## 4. SKILLS PROTOCOL (strict)

**S-01 Inventory first.** Before any other work in Phase 0, list every skill and plugin available in your environment (names exactly as shown). Write the list to `docs/SKILLS_INVENTORY.md`.

**S-02 Missing = stop.** Compare the inventory with the REQUIRED list below. Match by function if names carry a plugin prefix (for example `engineering:code-review` satisfies `code-review`). **If any REQUIRED skill is missing, STOP and report which ones. Do not substitute, imitate or silently skip.**

**S-03 Read fully, then apply.** Before first using a skill in a version, read its entire `SKILL.md`. Follow it.

**S-04 Skills do not override decisions.** If a skill's guidance conflicts with a Rule or Locked Decision, the Rule/Decision wins; log the conflict in the handoff.

**S-05 Evidence.** Every handoff includes a Skills Evidence table: skill name, step where used, output produced (file path or finding count). A skill with no evidence counts as not used.

**S-06 No unassigned skills.** `brutalist-skill`, `soft-skill`, `gpt-tasteskill`, `stitch-skill`, `imagegen-frontend-web`, `imagegen-frontend-mobile` and `output-skill` are NOT used unless the owner names them in a version-approval message.

### REQUIRED skills and where they are mandatory

| Skill | Mandatory use |
|---|---|
| `system-design` | Phase 0 architecture; before v2 protocol design; before v3 SDK/plugin design |
| `architecture` (ADR) | Every ADR in `docs/adr/` (ADRs record Locked Decisions; they never change them) |
| `taste-skill` and `taste-skill-v1` | Design system (Phase 0); every UI screen review in every version |
| `minimalist-skill` | Clutter check on every UI change: nothing added to a surface without removing noise elsewhere |
| `redesign-skill` | The design-review pass at the end of each version, before the handoff |
| `brandkit` | Phase 0: wordmark, logo mark, app icon, brand tokens (assets are original) |
| `design-system` | Phase 0: tokens, component states; each version: audit for hardcoded values |
| `design-critique` | Every version: structured review of every screen touched |
| `accessibility-review` | Every version: audit against WCAG 2.1 AA for touched UI |
| `ux-copy` | Every version: all new strings |
| `design-handoff` | Phase 0 and when specifying motion/component details for implementation |
| `frontend-design` + `image-to-code-skill` | Settings window (WebView2 UI) and the marketing site draft only; never the native notch renderer |
| `modern-web-guidance` | Settings window and marketing site only |
| `code-review` | Every version: full review of all new code before handoff, plus security pass on hooks, clipboard, IPC, network, crypto |
| `testing-strategy` | Phase 0 test plan; start of each version |
| `debug` | Every owner-reported bug and every failed benchmark |
| `tech-debt` | End of v1.5 and v3 only |
| `documentation` | Every version: docs and SDK docs |
| `deploy-checklist` | Every version: packaging and release checks |
| `user-research` | Phase 0: write the owner test scripts and interview guide (the owner runs them) |

The owner has these design skills installed: brandkit, brutalist-skill, gpt-tasteskill, image-to-code-skill, imagegen-frontend-mobile, imagegen-frontend-web, minimalist-skill, output-skill, redesign-skill, soft-skill, stitch-skill, taste-skill-v1, taste-skill (plus Anthropic skills). Use only the ones assigned above.

---

## 5. LOCKED DECISIONS REGISTER

These were decided by the owner. You may implement them, challenge them via the Decision Protocol (with data), but never change them.

### 5.1 Product and platforms
| ID | Decision |
|---|---|
| D-PRD-01 | Product name: **Nuvra**. Repo: `nuvra`. Crate prefix: `nuvra-`. Windows exe: `Nuvra.exe`. |
| D-PRD-02 | Windows 10 22H2 and Windows 11, x64 in v1. ARM64 and x86 (32-bit) builds added in v3. |
| D-PRD-03 | Android minimum API 26 (8.0), target latest stable. |
| D-PRD-04 | iOS: not built. No iPhone app in any version; documented as out of scope in `docs/BACKLOG.md`. |
| D-PRD-05 | Distribution: Microsoft Store (primary) plus signed MSIX direct download. Android: Google Play plus signed APK. |
| D-PRD-06 | App language in v1: English. v3 ships the language list in D-GLB-03. Layout, text rendering and formats are fully global-ready from v1 (D-GLB-01, D-GLB-02). i18n infrastructure (Fluent) exists from v1. |
| D-PRD-07 | License: app proprietary (all rights reserved). SDK and protocol spec: Apache-2.0. |

### 5.2 Stack
| ID | Decision |
|---|---|
| D-STK-01 | Language: **Rust**, stable toolchain pinned in `rust-toolchain.toml`. Rationale recorded in ADR-0001 (not Go: GC latency, weak Windows UI/API story; not C++: memory-safety and maintenance cost for an always-on, hook-using app). |
| D-STK-02 | Windows APIs: the `windows` crate (windows-rs). |
| D-STK-03 | Rendering: **Direct Composition + Direct3D 11 + Direct2D/DirectWrite.** No `wgpu`, no Electron, no WebView for the notch window. |
| D-STK-04 | Settings window: **WebView2** with vanilla TypeScript + CSS (no UI framework), bundled with esbuild, created only when opened and destroyed on close. WebView2 evergreen bootstrapper included in the installer. |
| D-STK-05 | Config: `%APPDATA%\Nuvra\config.toml` (typed, schema-versioned, with migrations). Data: SQLite at `%LOCALAPPDATA%\Nuvra\data.db`. |
| D-STK-06 | Logging: `tracing`, rolling files in `%LOCALAPPDATA%\Nuvra\logs`, 7 days, level INFO by default, content-free (R-20). |
| D-STK-07 | Android: Kotlin + Jetpack Compose, design tokens mirrored from Section 6. Shared Rust core via UniFFI. |
| D-STK-08 | Build/CI: GitHub Actions (`windows-latest`), `cargo nextest`, `criterion`, `proptest`, `cargo-fuzz`, `cargo audit`, `cargo deny`, `cargo llvm-cov`, `clippy -D warnings`, `rustfmt`. |

**D-ARC-01 Approved crates (nothing else without owner approval):** `windows`, `tokio` (rt-multi-thread, net, time, sync, io-util), `tracing`, `tracing-subscriber`, `tracing-appender`, `serde`, `serde_json`, `toml`, `rusqlite` (bundled), `aes-gcm`, `zeroize`, `thiserror`, `anyhow` (bin crates only), `flume`, `arc-swap`, `parking_lot`, `fluent-bundle`, `unic-langid`, `rustfft`, `nucleo`, `quinn`, `rustls`, `rcgen`, `prost`, `prost-build`, `mdns-sd`, `blake3`, `uniffi`, `ed25519-dalek`, `sha2`, `argon2`, `wasmtime` (v3 only), `criterion`, `proptest`, `libfuzzer-sys`.

### 5.3 Architecture
| ID | Decision |
|---|---|
| D-ARCH-01 | Threads: (1) UI/render thread (message pump + composition), (2) module worker threads (one per module, or a small tokio runtime for I/O modules), (3) event bus between them using bounded `flume` channels. The render thread never blocks on I/O, locks or allocation in the frame path. |
| D-ARCH-02 | Modules implement one trait: `id`, `capabilities`, `init`, `on_event`, `state() -> ModuleState` (plain data), `shutdown`. Modules never draw; the renderer draws `ModuleState`. |
| D-ARCH-03 | State flows one way: OS event → module → `ModuleState` → context engine → renderer. |
| D-ARCH-04 | Polling is banned. Allowed periodic timers only: system mini-monitor at 1 Hz while its panel is visible; visualizer at display rate while expanded and audio plays; phone heartbeat every 10 s while paired. Timers stop when hidden. |
| D-ARCH-05 | Crate layout: `nuvra-app`, `nuvra-render`, `nuvra-platform-win`, `nuvra-core-shared`, `nuvra-mod-media`, `nuvra-mod-mic`, `nuvra-mod-notifications`, `nuvra-mod-shelf`, `nuvra-mod-clipboard` (then one `nuvra-mod-*` per later module), `nuvra-sdk` (v3), `nuvra-relay` (v2), plus `/android`, `/settings-ui`, `/docs`, `/tools`. |

### 5.4 Window geometry and behavior
| ID | Decision |
|---|---|
| D-WIN-01 | Four states: **Idle** (nothing live), **Ambient** (live item shown), **Glance** (hover), **Expanded** (click or hotkey). |
| D-WIN-02 | Idle: nothing visible. A transparent 160×2 px hot-zone window at top-center catches the pointer. No global mouse hook. |
| D-WIN-03 | Size (logical px, DPI-scaled): Ambient 220×34; Glance 360×64; Expanded 520×220 default, content-driven height up to 360. Top edge flush with the screen top; top corners square, bottom corners rounded (Ambient 14, Glance 16, Expanded 18). |
| D-WIN-04 | Window styles: topmost, `WS_EX_NOACTIVATE`, `WS_EX_TOOLWINDOW`, `WS_EX_NOREDIRECTIONBITMAP`; never steals focus; never appears in Alt+Tab or the taskbar. |
| D-WIN-05 | Timing: hover intent delay 90 ms (configurable 0–400); collapse 350 ms after pointer leaves; notification banner 4 s, pauses on hover; max 3 queued banners, grouped by app. |
| D-WIN-06 | Default monitor: primary. Options: primary, all, monitor with active window. Mixed DPI, hot-plug, sleep/resume, RDP, taskbar on any edge must work. |
| D-WIN-07 | Fullscreen behavior: Nuvra hides in fullscreen apps except (a) 1.5 s mute/unmute confirmation, (b) the gaming strip (v1.5, if enabled), (c) the owner's hotkey. Detection: `SHQueryUserNotificationState` plus foreground-window rect equals monitor rect, updated by `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)`. |
| D-WIN-08 | Expanded view layout: the primary module for the current context fills the panel; a 5-icon rail on the left in this order: Media, Shelf, Clipboard, Notifications, More. Other modules live in More. Never more than 5 visible rail icons. |
| D-WIN-09 | Default hotkeys (rebindable; registration conflicts are detected and shown): mic mute `Ctrl+Alt+M`; toggle expanded `Ctrl+Alt+N`; clipboard panel `Ctrl+Alt+V`; launcher `Ctrl+Alt+Space`; color picker `Ctrl+Alt+C`; screenshot `Ctrl+Alt+S`. Use `RegisterHotKey`, not low-level hooks. |

### 5.5 Context engine priority (what Ambient shows)
Highest live priority wins the Ambient slot; others show as up to 3 small glyph chips.

1. Transient confirmations (mic muted/unmuted, screenshot saved) — 1.5 s
2. Active call (mic in use + meeting app)
3. Game strip (v1.5, fullscreen game)
4. Notification banner — 4 s
5. Timer / Pomodoro
6. File copy / download progress
7. Media playing
8. Idle

### 5.6 Data and storage
| ID | Decision |
|---|---|
| D-DAT-01 | Sensitive payloads (clipboard, notes, phone data) are stored in SQLite as AES-256-GCM ciphertext; the key is sealed with Windows DPAPI (`CryptProtectData`, current-user scope). Keys zeroized in memory after use. |
| D-DAT-02 | Clipboard limits: 200 items, 7 days, 1 MB per text item, 8 MB per image, 100 MB total. Defaults changeable in settings. |
| D-DAT-03 | Clipboard exclusions: honor `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory=0`, `CanUploadToCloudClipboard=0`, plus a default process denylist: 1Password, Bitwarden, KeePass/KeePassXC, Dashlane, LastPass. One-click wipe. |
| D-DAT-04 | Shelf: max 50 item references (not copies), persist until removed, missing files shown as "missing". |
| D-DAT-05 | Album art: decoded with WIC, GPU texture cache 32 MB LRU. |

### 5.7 Network (the only allowed network calls)
| ID | Decision |
|---|---|
| D-NET-01 | Allowed: (a) lyrics lookup to LRCLIB, opt-in, sending only title/artist/album/duration; (b) calendar via Microsoft Graph and Google Calendar, opt-in, read-only; (c) update-manifest check to the owner's update URL; (d) ICMP ping to the user-configured host (default 1.1.1.1) for the gaming strip; (e) phone LAN transport and optional relay (v2); (f) AI provider endpoints configured by the user (v3); (g) cloud sync and marketplace endpoints (v3). Everything else is forbidden. |
| D-NET-02 | Before enabling LRCLIB, check its current terms. If use in a commercial app is not permitted, STOP and open a decision. |

### 5.8 Security
| ID | Decision |
|---|---|
| D-SEC-01 | Runs as a normal user, never as admin. The only elevation allowed is the one-time optional step for FPS capture (D-GAM-01), requested only when the user enables that feature, and explained first. |
| D-SEC-02 | Updates: signed manifest (Ed25519), HTTPS only, signature verified before install, previous version kept for rollback. Store builds use the Store updater. |
| D-SEC-03 | Crash handling: local minidumps only. Settings has "Export diagnostics" (zip, content-scrubbed, user chooses where to save). No automatic upload. |
| D-SEC-04 | `cargo audit` and `cargo deny` must be clean; every exception documented with a reason and expiry. SBOM (CycloneDX) per release. |
| D-SEC-05 | Parsers (protocol messages, file/clipboard formats, plugin packages, SDK messages) are fuzzed in the version that introduces them. |

### 5.9 Phone (v2)
| ID | Decision |
|---|---|
| D-PHN-01 | Transport: **QUIC (`quinn`) with mutual TLS 1.3 (`rustls`)** using per-device self-signed Ed25519 certificates, pinned at pairing. No Noise layer on top. |
| D-PHN-02 | Pairing: QR on PC (cert fingerprint, one-time 128-bit token, LAN address, expiry 120 s). Both screens show the same 6-digit confirmation code; the user confirms on both. Tokens single-use. |
| D-PHN-03 | Discovery: mDNS service `_nuvra._udp.local`. Proximity: BLE RSSI (presence only, never carries data). |
| D-PHN-04 | Wire schema: Protobuf (`prost`), versioned, backward-compatible, with compatibility tests. |
| D-PHN-05 | Relay: self-hostable Rust service in `/nuvra-relay`. It is an opaque packet forwarder keyed by a rendezvous ID derived from the pairing secret; end-to-end TLS runs inside, so the relay cannot read content. Rate-limited, no content logging. The owner chooses and operates the host. |
| D-PHN-06 | File transfer: 256 KiB chunks, BLAKE3 verification, resumable, 2 GB cap. Files from a paired phone land on the shelf. |
| D-PHN-07 | Proximity: **auto-lock only** (`LockWorkStation` when the phone is out of range for 15 s). Auto-unlock is rejected permanently (cannot be done securely without a credential provider). |
| D-PHN-08 | iPhone is out of scope (D-PRD-04). |

### 5.10 Business
| ID | Decision |
|---|---|
| D-BIZ-01 | Tiers: **Free**, **Pro**, **Cloud**. |
| D-BIZ-02 | Free: all v1 Core modules, v1.5 launcher, quick tools, system mini-monitor, gaming strip, profiles (basic), themes Obsidian, Paper and Signal (high contrast always free), SDK usage. |
| D-BIZ-03 | Pro (one-time purchase, 14-day trial): phone connection over LAN (all of v2 except relay), Pro themes (Graphite Sand, Ink Blue), automation rules, advanced profiles, plugin installs. |
| D-BIZ-04 | Cloud (subscription): relay, settings sync, marketplace publishing. |
| D-BIZ-05 | Licensing: Ed25519-signed license file validated offline (grace period 30 days); Store builds use the Store entitlement API. No DRM that affects performance. |
| D-BIZ-06 | Prices: see Section 7 (owner-provided). Never invent a price. |

### 5.11 Performance: one experience for every machine
| ID | Decision |
|---|---|
| D-PERF-01 | **One experience.** Nuvra has a single visual and motion spec for all hardware. Quality tiers, hardware-based effect toggles and battery/power-based degradation are banned. The only motion variant is the user's own "reduce motion" accessibility choice. |
| D-PERF-02 | **The Floor defines the spec.** The Floor machine is a low-end reference: Windows 10 22H2, 4 GB RAM, dual-core CPU, 5400 rpm HDD or slow eMMC, integrated GPU of roughly the 2015–2018 generation, 1366×768 and 1920×1080 at 125% DPI. A second Floor run forces the WARP software renderer (hardware GPU disabled). Every feature and every animation must meet the Section 8 targets on both Floor runs. Where it cannot, the design is changed for everyone (owner approval via the Decision Protocol), never tiered. |
| D-PERF-03 | **Cost techniques (mandatory).** (a) Retained scene graph with damage-rect tracking: only dirty regions are redrawn and presented (`Present1` dirty rects). (b) Surface size never exceeds the current state's size (D-WIN-03). (c) Static and animated content live in separate DirectComposition visuals, so animation is a compositor transform, not a redraw. (d) Spring transitions (Section 6.4 values) are pre-baked into DirectComposition animation curves (piecewise-cubic segments, approximation error ≤ 0.5 px verified), so motion runs on the compositor independent of app CPU and costs the same at 60 Hz or 240 Hz; on interruption a new curve is baked from the current position and velocity. (e) Glyph runs, icons and text layouts cached; album art scaled once; no per-frame allocation. (f) Visualizer drawn as one batched draw of 32 rectangles, dirty rect only. Phase 0 must validate (c) and (d); if they cannot work as specified, STOP and open a decision. |
| D-PERF-04 | **Budgets per element.** Every visual element and module has a documented per-frame and idle cost on the Floor. Anything exceeding its budget is redesigned (same experience, cheaper construction) for everyone, never removed per hardware. |
| D-PERF-05 | **Backend selection must produce identical output.** Hardware Direct3D 11 is used when available; WARP is used when no hardware adapter exists, in RDP, after a driver reset, or when forced. Golden-image tests prove hardware and WARP output match within a fixed tolerance. On hybrid laptops Nuvra always uses the integrated/low-power adapter (`IDXGIFactory6::EnumAdapterByGpuPreference`, minimum power) and never wakes the discrete GPU. Device-lost recovery completes in ≤ 1 s with no visible glitch. |
| D-PERF-06 | **Startup.** First frame ≤ 150 ms after launch. Modules initialize after the first frame, in priority order, on background threads. At login, heavy initialization waits until CPU use is below 30% for 2 s, with a 20 s cap, so Nuvra never slows boot. |
| D-PERF-07 | **No power-based behavior change.** Nuvra does not alter visuals or frame behavior on battery or battery saver; the design is already cheap enough that it does not need to. |
| D-PERF-08 | **Test hardware set** (owner-provided, Section 7): Floor, Mid, High (144 Hz+, multi-monitor, mixed DPI), plus a WARP-forced run. All Section 8 targets must pass on all four; results per machine appear in every handoff. |
| D-PERF-09 | **Self-test.** Settings → About → "Run performance test" measures frame time (p50/p95/p99), pointer-to-frame latency, memory and startup time on the user's machine, shows pass/fail against Section 8, and exports a content-free JSON the user may share manually. Never uploaded automatically. |
| D-PERF-10 | **Honest claims.** Nuvra's docs and marketing never claim "fast on every machine". They state the contract and the hardware it was verified on. Any machine that misses a target is reported, not hidden. |

### 5.12 Global reach: the same experience for everyone
| ID | Decision |
|---|---|
| D-GLB-01 | **Global from day one.** Nothing in the architecture assumes English, left-to-right text, Latin script, the Gregorian calendar, 96 DPI, a mouse with hover, a US keyboard, a fast disk or an internet connection. The speed, animation and visual quality promised in Sections 6 and 8 apply in every language, region and configuration. |
| D-GLB-02 | **Text and layout (from v1).** DirectWrite with full complex-script shaping and font fallback (Devanagari, Tamil, Bengali, Arabic, CJK, plus emoji inside media titles and notifications). RTL languages mirror the whole layout; the media timeline and numerals follow platform convention. Strings can expand 40% without clipping or layout jump; tall scripts (Tamil, Thai, Arabic) are never clipped vertically; bidirectional text in titles renders correctly. Dates, times, numbers and calendars (including Hijri, Thai Buddhist, Persian, Japanese era), 12/24-hour and first day of week come from Windows locale APIs. Windows text-size scaling is honored. A pseudo-locale (accented, +40% length, RTL) runs in CI and golden-image tests. |
| D-GLB-03 | **Languages shipped in v3 (18):** English, Spanish, Portuguese (Brazil), French, German, Italian, Russian, Turkish, Arabic, Hindi, Tamil, Bengali, Indonesian, Vietnamese, Japanese, Korean, Chinese (Simplified), Chinese (Traditional). The UI follows the Windows display language by default with an in-app override. Translations come from native speakers supplied by the owner (Section 7); machine-translated strings are flagged in the catalogs and no language ships unreviewed. |
| D-GLB-04 | **Input devices.** Mouse, touch, pen, keyboard-only and screen readers are all first-class. Touch and pen (no hover): tap the pill to expand, swipe down on the pill to expand, pen hover supported; no screen-edge swipe hooks. Hit targets ≥ 24×24 px for mouse and ≥ 44×44 px for touch/pen. Tablet-mode and 2-in-1 posture changes are handled. |
| D-GLB-05 | **Windows editions and environments.** Home, Pro, Enterprise, Education, N/KN (if media features are missing, the media module shows a clear "needs Media Feature Pack" state), LTSC, S mode (Store/MSIX), virtual machines, domain-joined and no-admin accounts, Group Policy-restricted machines, corporate proxies (use system proxy settings and proxy authentication; never bypass). Per-user install, no admin needed. If WebView2 is missing and cannot be installed (offline), the settings window shows a clear message with a retry; the notch and all core modules still work. Builds: x64 (v1), ARM64 and x86 32-bit (v3, with a 2 GB RAM Windows 10 test VM); the same Section 8 targets apply. |
| D-GLB-06 | **Displays.** 1024×768 to 8K; 100–500% scaling; ultrawide (32:9); portrait and rotated monitors; mixed DPI and mixed refresh rates (30–360 Hz, VRR); HDR enabled (tokens defined in sRGB and composited correctly with no banding or crushed blacks); touch screens; TVs as external displays. |
| D-GLB-07 | **Connectivity.** All v1 core modules work with zero internet. Networked features (lyrics, calendar, updates, relay, cloud, AI) never block the UI, time out in ≤ 3 s, cache results and show a calm offline state, never an error dialog. Metered connections are honored (no automatic update downloads on metered networks unless the user overrides). Updates are resumable and a full update is ≤ 25 MB. IPv4 and IPv6 both work; captive portals are detected without false errors. The relay client probes available regions and picks the lowest latency; the owner deploys relays in at least 3 regions (Section 7). |
| D-GLB-08 | **Keyboard layouts and hotkeys.** Default hotkeys are tested on US, UK, AZERTY, QWERTZ, Russian, Arabic, Japanese, Korean, Hindi and Tamil layouts; conflicts are detected and rebinding is offered; IME composition never triggers hotkeys; AltGr is handled. |
| D-GLB-09 | **Privacy-law readiness.** Nuvra is built so that obligations under GDPR/UK GDPR, India's DPDP Act, CCPA/CPRA and Brazil's LGPD can be met: in-app export and delete of all stored data (clipboard, notes, shelf, paired phones, tokens), a consent screen for every optional network feature, no knowingly collected children's data. The owner handles legal review of the privacy policy and store ratings (Section 7); you do not give legal advice. |
| D-GLB-10 | **Global-readiness matrix (every version).** Pseudo-locale; Arabic RTL screenshot set; CJK, Tamil and Hindi shaping; 100/150/200/300/500% DPI; 30/60/144/240 Hz; touch and pen; N-edition VM; S mode; no-admin install; offline; proxy; metered network; high contrast; Narrator and NVDA; 10 keyboard layouts. Anything you cannot test is UNVERIFIED with a manual script. |
| D-GLB-11 | **Honest claims.** Nuvra does not claim to work for "everyone in the world." Docs state the supported systems: Windows 10 22H2+ and Windows 11, plus Android 8.0+ for the companion. |

---

## 6. DESIGN SYSTEM (locked)

### 6.1 Visual idea
**"Bezel extension."** The notch looks like a physical continuation of the screen's bezel: matte, flush with the top edge, square top corners, rounded bottom corners, a 1 px inner top-light, and one accent used only when something is live. Solid surfaces only. No backdrop blur (also the cheapest on weak GPUs). Album art may tint the surface with a single solid color at ≤ 10% — never a gradient.

### 6.2 Color tokens
| Token | Obsidian (default, dark) | Paper (light) | Signal (high contrast) |
|---|---|---|---|
| `surface` | #060709 | #F6F5F2 | #000000 |
| `surface-raised` | #0E1014 | #FFFFFF | #0A0A0A |
| `surface-high` | #161920 | #ECEAE4 | #141414 |
| `edge` | #232732 | #DAD8D2 | #FFFFFF |
| `edge-lit` | #FFFFFF @ 6% | #FFFFFF @ 80% | #FFFFFF @ 100% |
| `text-primary` | #F3F4F6 | #14151A | #FFFFFF |
| `text-secondary` | #9AA0AC | #5A5E68 | #E0E0E0 |
| `text-tertiary` | #5E6470 | #8A8D96 | #BDBDBD |
| `accent` (live) | #FF6B3D | #E8501F | #FFD400 |
| `danger` | #FF4258 | #D42A40 | #FF6B6B |
| `ok` (state only) | #46D39A | #1E9E6A | #7CFF9E |
| `warning` | #FFC247 | #B87A00 | #FFD400 |

Pro themes: **Graphite Sand** (surface #121316, raised #1A1C21, accent #D6C29A) and **Ink Blue** (surface #070B14, raised #0D1424, accent #7FA3FF). All text/background pairs must meet 4.5:1 (7:1 in Signal); verify and record.

### 6.3 Type, spacing, radii, icons
- Font: Segoe UI Variable (Text/Display). No bundled fonts. Tabular figures for all numbers.
- Scale (px): caption 11, small 12, body 13, title 15, display 20.
- Spacing grid: 4 px (4, 8, 12, 16, 24). Radii: 8 (controls), 14, 16, 18 (notch states).
- Icons: custom, drawn in-house as vector paths on a 20 px grid, 1.5 px stroke, round caps and joins. No icon packs.
- Sound effects: none by default (optional, off).

### 6.4 Motion
- Expand: spring stiffness 380, damping 32, mass 1. Collapse: stiffness 460, damping 40, mass 1.
- Content crossfade: 120 ms, `cubic-bezier(0.2, 0, 0, 1)`.
- All animations are interruptible (reversible mid-flight).
- Reduced motion (system setting or user toggle): springs replaced by 80 ms fades, visualizer paused.
- Nothing animates for decoration. Animation must communicate a state change.

### 6.5 Accessibility baseline (from v1)
UI Automation tree for the notch (name, role, state, value); full keyboard operation (Tab/Shift+Tab, arrows, Enter, Esc; focus ring uses `accent`); screen-reader names for every control; high-contrast theme (Signal) honoring the Windows contrast setting; reduced motion; minimum 24×24 px hit targets.

### 6.6 Copy rules
Short, specific, human. No "Oops", no exclamation marks, no emoji. Say what happened and what the user can do.

---

## 7. OWNER-PROVIDED VALUES

Do not invent these. Use clearly named config placeholders (`OWNER_FILL_*`) and list unresolved ones in each handoff. Unsigned/dev builds are allowed while they are missing; the handoff states the release candidate is unsigned.

- Pro price and Cloud price (D-BIZ-06)
- Code-signing certificate and Microsoft Store developer account
- Google Play developer account and Android signing key
- Domain name, support email, privacy policy URL
- Update manifest URL and update-signing key (private key never enters the repo)
- Relay host and Cloud sync / marketplace API base URLs
- Microsoft and Google OAuth client IDs (calendar)
- LRCLIB user-agent contact string
- Test hardware for D-PERF-08: a Floor machine (or a VM constrained to 2 vCPU / 4 GB RAM with the software renderer), a Mid machine and a High machine
- For D-GLB: native-speaker translations for each language in D-GLB-03 (the agent supplies English catalogs and the pseudo-locale only); a Windows N-edition VM, an S-mode device or VM, a touch/pen device if available, a 2 GB RAM 32-bit Windows 10 VM (v3); relay hosting in at least 3 regions (D-GLB-07); legal review of the privacy policy (D-GLB-09)

---

## 8. PERFORMANCE CONTRACT

**The Floor machine defines the spec (D-PERF-02).** The same targets, with the same full visuals and animations, must pass on **every** machine in D-PERF-08: Floor, a WARP-forced Floor run, Mid, and High (144 Hz+, multi-monitor, mixed DPI). There are no per-hardware targets and no quality tiers (R-31). Record machine specs in every report.

We do not claim "fast on every machine." We claim "passes this contract on these machines" and report any machine where it does not.

| Metric | Target |
|---|---|
| Pointer-enter / hotkey event to first frame submitted to the compositor (app-side latency; display scan-out adds up to one refresh interval and is outside the app's control) | ≤ 8 ms p95 |
| Animation | 0 dropped frames at refresh rate; ≤ 4 ms CPU per frame |
| Idle CPU (Idle/Ambient, nothing animating) | ≈ 0% (no timer wakeups; verify with ETW) |
| Idle RAM working set, v1 | ≤ 30 MB; each enabled opt-in module ≤ +10 MB |
| Cold start to usable | ≤ 400 ms |
| Installer size | ≤ 25 MB (excluding WebView2 bootstrapper) |
| Impact on games | ≤ 0.5% FPS change with strip enabled |
| Soak (8 h, media + clipboard + sleep/resume) | memory growth ≤ 5%, zero crashes |

Required techniques: pre-warmed hidden window; animate via composition transforms; no layout or re-rasterization in the animation path; no allocation or locks in the frame path; and every technique in D-PERF-03, which make the full experience cheap enough for the Floor instead of degrading it. If a target is missed on any machine, **STOP and report the numbers**; do not change the renderer (D-STK-03), weaken the target (R-11), or tier the experience (R-31). If a feature cannot meet its budget on the Floor, open a decision to redesign it for everyone.

CI/benchmark scripts must enforce this table; a regression > 10% blocks the handoff.

---

## 9. SECURITY SPEC (every version)

- `docs/SECURITY.md` holds the threat model (assets, attackers, trust boundaries, mitigations), updated each version.
- Hotkeys use `RegisterHotKey`; no keystroke capture anywhere.
- Notification access via `UserNotificationListener` (requires MSIX package identity and user permission); explain the permission before requesting it.
- Every input boundary is validated: dropped files and paths, clipboard formats, network messages, SDK/plugin messages. Reject oversized or malformed input without panicking.
- Dependency pinning with a committed `Cargo.lock`; `cargo audit`, `cargo deny`, SBOM per release.
- Privacy: no accounts required, no telemetry, `docs/PRIVACY.md` lists every piece of data stored, where, how it is protected and how to wipe it.
- Unpair, wipe and uninstall must remove keys and data (uninstall removes `%LOCALAPPDATA%\Nuvra` unless the user opts to keep it).

---

## 10. TESTING SPEC (every version)

- Unit tests for logic; integration tests per module against real or faked OS interfaces; golden-image render tests using the Direct2D WARP (software) adapter in CI; property tests (`proptest`) for parsers and protocol; fuzz targets per D-SEC-05.
- Line coverage ≥ 80% on non-`unsafe` logic crates (`cargo llvm-cov`).
- Latency/memory benchmarks (`criterion` plus ETW-based frame timing using `QueryPerformanceCounter`) enforcing Section 8.
- Compatibility matrix, tested where possible and otherwise listed UNVERIFIED with manual scripts: Windows 10 22H2, Windows 11; integrated and discrete GPUs; 60 and 144 Hz; 100/125/150/200% DPI; single and multi-monitor; taskbar bottom/top/side; borderless and exclusive fullscreen; RDP; sleep/resume; monitor hot-plug.
- Hardware set per D-PERF-08 (Floor, WARP-forced Floor, Mid, High): every Section 8 target is measured on each, every version. Add tests for hybrid-GPU adapter selection, device-lost recovery (≤ 1 s), hardware-vs-WARP golden-image parity (D-PERF-05), and slow-disk cold start.
- Global-readiness matrix (D-GLB-10) is run every version and its results go in the handoff.
- `docs/TESTING.md` documents how to run everything with one command.

---

## 11. VERSION GATE PROTOCOL

1. Start of each version: run `testing-strategy`, write the version's test plan, list the Skills it requires from Section 4.
2. Build only that version's scope on `release/<version>`.
3. End: run the review pass (`code-review`, `design-critique`, `accessibility-review`, `redesign-skill`, `design-system` audit, `ux-copy`, `deploy-checklist`), fix what they find (within scope), then produce the handoff.
4. **STOP** with: `STOPPED. Waiting for: APPROVED <version> — proceed to <next version>`.
5. Owner tests. If the owner reports issues: R-07 (failing test first), minimal fix (R-05), re-run all checks, re-issue handoff, STOP again.

---

## 12. PHASE 0 — FOUNDATION

1. Skills inventory and check (Section 4, S-01/S-02). If anything required is missing: STOP.
2. `docs/SETUP.md` listing every tool to install; install only after listing.
3. Workspace scaffold per D-ARCH-05, `rust-toolchain.toml`, CI, lint/format/audit/deny, benchmark harness, `docs/TESTING.md`.
4. ADRs that **record** the locked decisions (ADR-0001 language, 0002 renderer, 0003 threading/event bus, 0004 data/encryption, 0005 module trait, 0006 settings UI, 0007 phone transport, 0008 plugin sandbox plan). ADRs never reopen a decision.
5. `docs/DESIGN_SYSTEM.md` implementing Section 6 exactly: tokens as code (`nuvra-render/tokens`), component states, motion spec, icon grid. Brand assets via `brandkit` (wordmark, mark, app icon), all original.
6. Renderer validation spike: an empty notch that moves Idle → Ambient → Glance → Expanded with the real springs, measured against Section 8 on the Floor machine, a WARP-forced run and the High machine (D-PERF-02, D-PERF-08). It must also validate the compositor-driven animation approach in D-PERF-03 (c) and (d). **Purpose is to validate D-STK-03 and D-PERF-03, not to choose.** If it misses targets, STOP and open a decision with numbers.
7. Threat model v0, `docs/PRIVACY.md` skeleton, `docs/BACKLOG.md`, `THIRD_PARTY_LICENSES.md`.
8. Owner test scripts written with `user-research` (what to click, what to measure, what to try to break).

Then the Release Handoff and STOP.

---

## 13. RELEASE HANDOFF (end of every version)

Write `docs/handoff/<version>.md` and post a short chat summary. Sections, in order:

1. **Scope table:** every scope item → DONE / PARTIAL / NOT DONE with reason.
2. **Build, run, install:** exact commands.
3. **Locked-decision compliance table:** each D-ID touched → followed. **Deviation log must be empty.** If anything deviated, list it and why.
4. **Open decisions:** every `OPEN-nnn` awaiting the owner.
5. **Performance report:** Section 8 table with measured numbers for every machine in D-PERF-08 (Floor, WARP-forced Floor, Mid, High) with specs, plus the self-test export (D-PERF-09). Any machine that misses a target is listed explicitly.
6. **Security report:** threat-model changes, `cargo audit/deny` output, fuzzing results, permissions requested, data stored and where, known risks.
7. **Test report:** passed/failed/UNVERIFIED, coverage numbers, compatibility matrix status, and the global-readiness matrix results (D-GLB-10).
8. **Skills evidence table** (S-05).
9. **Owner test checklist:** functional steps, edge cases, hostile cases (huge clipboard, malformed files, drag of 10,000 files, sleep/resume, monitor hot-plug, network drops, killing the process mid-write), plus a vulnerability checklist (inspect stored files for plaintext, inspect traffic, review permissions, try to crash each module).
10. **Known issues and limitations**, stated plainly.
11. **Owner-provided values still missing** (Section 7).
12. **Backlog additions.**
13. Final line: `STOPPED. Waiting for: APPROVED <version> — proceed to <next version>`

After posting, do nothing until the owner replies.

---

## 14. v1 — CORE

**Scope is exactly the items below.** No calendar, launcher, gaming, themes beyond the three free ones, phone, SDK, AI.

### 14.1 Notch engine
- Four states, geometry, timing and window styles per D-WIN-01…09; Obsidian/Paper/Signal themes, with system-follow option.
- Multi-monitor per D-WIN-06; per-monitor DPI; recompute on `WM_DPICHANGED`, `WM_DISPLAYCHANGE`, taskbar changes, sleep/resume.
- Fullscreen auto-hide per D-WIN-07; manual "pin visible" override in settings.
- Context engine implementing Section 5.5 priority table.
- Settings window (WebView2, D-STK-04): General, Appearance, Modules, Hotkeys, Privacy, About. Built with `frontend-design`, `modern-web-guidance`, `image-to-code-skill`, matching Section 6.

### 14.2 Media module
- Source: `GlobalSystemMediaTransportControlsSessionManager`. Subscribe to session-changed, media-properties-changed, playback-info-changed, timeline-properties-changed.
- Session choice: the currently playing session; if none, the most recent one for 10 minutes, then drop.
- Shows title, artist, art, play/pause/next/previous, seek bar (draggable), volume (system master volume via `IAudioEndpointVolume`).
- Ambient: art thumbnail + title scrolling only when truncated. Glance: title + play/pause. Expanded: full controls.
- Visualizer: WASAPI loopback, 32-band FFT (`rustfft`), only when Expanded and audio playing, at display rate; disabled only by the user's reduced-motion setting (never by hardware or power state, D-PERF-01/07).
- Lyrics: LRCLIB, opt-in (D-NET-01/02), synced LRC display in Expanded, cached locally.
- Edge cases: no art, very long titles, sessions from browsers with changing metadata, ads, paused-for-hours sessions, session crash.

### 14.3 Mic and call module
- Global mic mute: toggles `IAudioEndpointVolume::SetMute` on the default capture endpoint, so it affects every app. Hotkey `Ctrl+Alt+M`. Confirmation per Section 5.5 (1.5 s) with clear muted/live visual (not color alone).
- Mic-in-use: `IAudioSessionManager2` capture sessions on the default capture device. Camera-in-use: `CapabilityAccessManager\ConsentStore\webcam` registry values watched with `RegNotifyChangeKeyValue` (event-driven).
- Call state "Active call" = mic in use by a process in the meeting-app list (Teams, Zoom, Google Meet in a browser window title match, Webex, Discord, Slack huddle, Skype). The list is configurable.
- Handle: default device change, device unplug, multiple capture devices, mic muted by another app, exclusive-mode apps.

### 14.4 Notifications module
- `UserNotificationListener` access with a first-run explanation and permission flow; if denied, the module shows how to enable it and nothing else.
- Compact banner per D-WIN-05; per-app allow/block list; respects Do Not Disturb/Focus Assist (never shows banners while DND is on, unless the user overrides per app).
- Actions: click opens the source app where possible; dismiss clears the toast.
- Content never logged or persisted beyond the current queue.

### 14.5 Files shelf
- `IDropTarget` onto the notch (drag-hover expands to Shelf). Drag out via `DoDragDrop` with `CF_HDROP`.
- Items are references with name, icon (shell icon, cached), size. Actions: copy path, compress (to ZIP in the same folder as the first item, never overwriting), reveal in Explorer, remove.
- D-DAT-04 limits; handle network/removable drives, long paths, files deleted while on the shelf, drops of folders and 10,000+ files (cap and explain).

### 14.6 Clipboard module
- `AddClipboardFormatListener`; capture text, links, images (CF_UNICODETEXT, CF_DIB/PNG). Search (instant, in-memory index), pin, copy back, delete, wipe.
- D-DAT-01/02/03 storage and exclusions. Never capture while the clipboard owner is in the denylist or format flags exclude it.
- Edge cases: delayed rendering, huge items, rapid-fire copies, clipboard locked by another app.

### 14.7 Basics
- First-run setup in ≤ 60 s (permissions explained, hotkeys shown, theme choice).
- Launch at startup (user toggle, via the packaged startup task), tray icon (show/hide, pause Nuvra, settings, quit).
- MSIX packaging, update mechanism per D-SEC-02, uninstall cleanup.
- Fluent string infrastructure (English only).

### 14.8 v1 acceptance
All Section 8 targets met on every machine in D-PERF-08 (Floor and WARP-forced included) with the full visual design and no tiers; the D-PERF-09 self-test is implemented and its export verified; the D-GLB-10 global-readiness matrix passes (RTL mirror, complex-script shaping, touch/pen expand gesture, 100–500% DPI, offline operation, pseudo-locale with no clipping); all Section 9/10 checks complete; accessibility baseline passed on every screen; zero `TODO`/stubs; every module survives forced failure (kill the module thread; the host stays alive and restarts it).

---

## 15. v1.5 — PRODUCTIVITY AND GAMING

Starts only after `APPROVED v1 — proceed to v1.5`. Scope is exactly:

### 15.1 Calendar (opt-in)
- Providers: Microsoft Graph (Calendars.Read) and Google Calendar (calendar.readonly), plus local ICS URL/file. OAuth with PKCE and loopback redirect; tokens in Windows Credential Manager; least-privilege scopes only; offline_access for Microsoft.
- Shows the next event within 12 h; Ambient chip appears 10 min before start; "Join" button parses Teams/Zoom/Meet links.
- Event titles never logged or sent anywhere. Sign-out wipes tokens and cache.

### 15.2 Gaming strip and game mode
- **D-GAM-01** FPS and frame time from ETW present events (DXGI/DxgKrnl) via a documented, user-initiated, one-time step that adds the user to the "Performance Log Users" group (UAC prompt, explained first). No injection into game processes (R-21). If not enabled, the strip hides FPS and says how to enable it.
- Ping: ICMP RTT to the configured host (default 1.1.1.1), 1 Hz while the strip is visible.
- GPU temperature: NVIDIA via NVML (dynamically loaded) and AMD via ADLX if present; otherwise hidden. **CPU temperature is not included** (needs a kernel driver, banned by R-21).
- Game detection: fullscreen/borderless foreground process not in a denylist (browsers, video players, Nuvra itself) or on the user's allowlist; "Add as game" action.
- Game mode profile: silences banners, shows strip, keeps mic-mute confirmation. Auto on game start, auto off on exit.
- Measure and report FPS overhead (target ≤ 0.5%).

### 15.3 Launcher and command palette
- `Ctrl+Alt+Space`. Index: Start Menu shortcuts plus user-chosen folders, built incrementally and stored locally. Fuzzy matching with `nucleo`; frecency ranking.
- Built-ins: calculator (own expression parser), unit converter (own tables), web search (user-configurable engine, default DuckDuckGo), open settings, toggle modules.
- Result in ≤ 16 ms after each keystroke with 50,000 indexed items.

### 15.4 Quick tools (each opt-in)
- Pomodoro (default 25/5/15, configurable), quick notes and tasks (SQLite, encrypted per D-DAT-01), reminders (notch banner plus Windows toast fallback).
- Color picker (`Ctrl+Alt+C`): magnified loupe, copies HEX.
- Screen OCR: `Windows.Media.Ocr`, local only, result copied to clipboard.
- Screenshot (`Ctrl+Alt+S`): `Windows.Graphics.Capture`; region or window; saved to `Pictures\Nuvra`, copied to clipboard, preview in the notch.

### 15.5 System
- Mini-monitor: CPU, RAM, GPU, network via PDH counters, 1 Hz only while visible.
- Battery, Wi-Fi, Bluetooth status chips. Quick toggles for Wi-Fi, Bluetooth, night light (via supported WinRT/Settings APIs; if an API is unavailable on a Windows build, hide that toggle and note it).
- Volume and brightness HUD: Nuvra's own HUD **alongside** the system OSD. **Suppressing the native OSD is not implemented** (fragile, undocumented).
- Download/copy progress indicator via the Windows progress APIs where available.
- Caps/Num lock indicator (state read on key events registered through the existing hotkey mechanism, no keylogging).

### 15.6 Themes and profiles
- Theme engine reading the Section 6 token sets; Obsidian, Paper, Signal ship free. Pro themes (Graphite Sand, Ink Blue) included but entitlement-gated (entitlement stubbed via the license interface until v3 licensing).
- Basic profiles: **Work, Gaming, Meeting, Study.** Each is a module-toggle bundle + DND + chip rules. Defaults: Work (media, clipboard, shelf, notifications), Gaming (strip, mic, DND), Meeting (mic, calendar, DND), Study (Pomodoro, notes, DND). Manual switch plus the game-mode auto-switch.

### 15.7 v1.5 acceptance
All v1 acceptance still met; performance contract holds with all opt-in modules enabled (report per-module cost); `tech-debt` audit delivered; no regressions.

---

## 16. v2 — PHONE CONNECTION

Starts only after `APPROVED v1.5 — proceed to v2`. Principle: the phone is a data source feeding existing modules; there is no cluttered Phone tab. Run `system-design` for the protocol before coding; record it in `docs/PROTOCOL.md`.

### 16.1 Shared core (`nuvra-core-shared`)
Windows-independent: Protobuf schema, QUIC/mTLS session logic, pairing, chunked file transfer, reconnect with exponential backoff (1 s → 60 s), mDNS discovery. UniFFI bindings for Android. Message types: Hello, PairRequest/Confirm, Heartbeat, NotificationPosted/Removed/Reply, CallState (incoming/answered/rejected/ended), MediaState, BatteryState, ClipboardPush, FileOffer/Chunk/Accept/Done, FindPhone, RemoteInput (media, volume, slides, pointer), SmsOtp, ProximityBeacon, Unpair.

### 16.2 PC side
- Pairing UI per D-PHN-02; device list (max 3 paired phones); unpair wipes keys and cached data on both sides.
- Phone events feed existing modules: calls → call module (accept/reject from PC), notifications → notifications module (reply), files → shelf, clipboard → clipboard history (tagged "from phone").
- Phone battery chip; "Find my phone" action; phone-as-remote handled as input into media/volume.
- Proximity auto-lock per D-PHN-07.
- SMS/OTP: detect 4–8 digit codes with keywords; show as a banner with Copy; never persisted; excluded from clipboard history.
- Reconnect seamlessly across Wi-Fi changes and sleep/resume.

### 16.3 Android app
- Kotlin + Compose with Section 6 tokens. Small, battery-efficient, foreground service only while paired and needed.
- Features: notification mirroring and reply (`NotificationListenerService`), call alerts and reject, media and battery state, clipboard sync, file send/receive, find-my-phone, remote control, SMS/OTP, BLE proximity beacon.
- Permissions requested just in time, each with a plain explanation and a per-feature toggle.
- Respect Doze and battery optimization; measure and report battery impact over 24 h.

### 16.4 Relay (`nuvra-relay`)
Per D-PHN-05, with rate limits, connection limits, abuse handling docs, deployment guide for the owner's chosen host, load test report.

### 16.5 v2 acceptance
No plaintext on the wire (verified by packet capture and documented); replayed/forged/malformed messages rejected (fuzz results); pairing token single-use verified; unpair and forget-device wipe verified; reconnection across network changes verified; Pro gating hooks in place (D-BIZ-03); `docs/PROTOCOL.md` complete.

---

## 17. v3 — PLATFORM

Starts only after `APPROVED v2 — proceed to v3`. Run `system-design` and `architecture` for the SDK and plugin runtime first.

### 17.1 Live Activities SDK
- Local API over a named pipe `\\.\pipe\nuvra-sdk-v1`, ACL restricted to the current user. Protobuf messages, versioned.
- Activity schema: `id`, `title`, `subtitle`, `progress` (0–1), `icon` (PNG ≤ 64 KB or built-in glyph), up to 3 `actions` (callbacks via the pipe), `priority` (suggestion only; the host decides via Section 5.5).
- Per-app permission prompt on first connect (identified by executable path and signature); revocable. Limits: 10 updates/s and 3 activities per app. Flooding and spoofing another app's ID must fail.
- SDK packages: Rust (`nuvra-sdk`), C# (NuGet), Python (pip), TypeScript/Node (npm), each a thin pipe wrapper, with samples and docs (Apache-2.0).

### 17.2 Plugin runtime
- WASM via `wasmtime`, Component Model, WIT interface `nuvra:plugin`.
- Capabilities (declared in the manifest, shown to the user, off unless granted): `ui.render` (declarative widgets only: text, icon, button, list, progress, image ≤ 128 KB — no custom drawing, so design stays consistent), `net.fetch` (host allowlist), `storage` (5 MB KV), `clipboard.read` (explicit per use), `files.pick` (user-chosen files only), `timers`.
- Limits: 16 MB memory, fuel-metered CPU (5 ms per event), hard kill at 50 ms wall time per event. A misbehaving plugin cannot crash or slow the host.
- Package `.nvx` (zip: `manifest.toml`, `plugin.wasm`, assets, Ed25519 signature). Client verifies signatures; unsigned packages need an explicit warning flow.
- Sandbox-escape tests documented and failing; plugin-package parser fuzzed.

### 17.3 Marketplace (client only)
Browse/install/update/remove plugins and theme packs inside the settings window. Server API documented in `docs/marketplace-api.md`; the server is not built. Base URL from Section 7.

### 17.4 Automation and profiles
- Rules (TOML/JSON): triggers = app start/stop, fullscreen enter/exit, time of day, Wi-Fi SSID, monitor connect/disconnect, mic in use, phone connected. Actions = switch profile, toggle module, set DND, show chip, launch an app (path confirmed by the user at creation). **No "run arbitrary command" action.**
- Rule editor in settings with a dry-run mode. Advanced profiles (auto-switch by app, time, network).

### 17.5 Cloud sync (opt-in, Cloud tier)
End-to-end encrypted: key from the user's passphrase via Argon2id. Syncs config, profiles, rules and snippets only. **Never** clipboard, files, notes or phone data.

### 17.6 AI module (off by default)
- Providers: any OpenAI-compatible endpoint (user URL + key), Anthropic API (user key), local Ollama endpoint. Keys in Credential Manager.
- Actions only on explicit user click: summarize clipboard, rewrite (tone presets), ask. Show exactly what will be sent before sending. No history persisted by default. No background reading of anything.

### 17.7 Licensing, platform and polish
- License system per D-BIZ-05 (Ed25519 offline validation, 30-day grace; Store entitlement for Store builds); entitlement gating per D-BIZ-02/03/04 replaces the v1.5 stub.
- ARM64 and x86 (32-bit) builds and testing, including the 2 GB RAM Windows 10 VM (D-PRD-02, D-GLB-05).
- Localization: every language in D-GLB-03, using the owner-supplied reviewed translations (Section 7), plus font-fallback, RTL and pseudo-locale checks.
- Final hardening: full security review (`code-review` security pass on every network, IPC, crypto and parser surface), complete compatibility matrix, performance re-baseline, `tech-debt` audit.
- Documents: user guide, SDK docs, plugin authoring guide, `docs/PRIVACY.md` final, marketing site draft (via `frontend-design`, `modern-web-guidance`, `image-to-code-skill`), store listing copy draft (via `ux-copy`).
- Release candidate packaging with `deploy-checklist`; signed builds if Section 7 values exist, otherwise explicitly marked unsigned.

### 17.8 v3 acceptance
Plugin escape attempts, SDK abuse (flooding, spoofing) and sync-tamper tests documented and failing; performance contract met with plugins and SDK clients active; all earlier acceptance still met.

---

## 18. DEFINITION OF DONE (every version)

- Builds with zero warnings; `clippy -D warnings`, `rustfmt`, tests, audit and deny all pass.
- Section 8 performance contract met and recorded.
- Section 9 security and Section 10 testing checks complete; no known high or critical issues.
- Accessibility baseline met; Section 6 followed with zero off-token values.
- No stubs, dead code or unrelated changes in the diff (R-05, R-12).
- Docs and ADRs updated (R-18).
- Skills Evidence table complete (S-05); deviation log empty.
- Release Handoff delivered and the process stopped at the gate.

---

## 19. FIRST ACTION

Do exactly this, in order, and nothing else:

1. Reply with a plan restatement in **10 lines or fewer**.
2. Run the skills inventory (S-01), write `docs/SKILLS_INVENTORY.md`, and report any missing REQUIRED skills (S-02). If any are missing, STOP here.
3. List any blocking open decisions (maximum 3).
4. Begin Phase 0 (Section 12).

Do not begin v1 until the owner sends: `APPROVED Phase 0 — proceed to v1`.