# Nuvra — Developer Setup

## Prerequisites

Install these tools in order. All must be on `PATH`.

### 1. Rust stable toolchain
```
rustup toolchain install stable --profile default
rustup target add x86_64-pc-windows-msvc
```

### 2. Cargo tools (requires internet — run once)
```
cargo install cargo-nextest  --locked
cargo install cargo-deny     --locked
cargo install cargo-audit    --locked
cargo install cargo-llvm-cov --locked
```

Verify after install:
```
cargo nextest  --version
cargo deny     --version
cargo audit    --version
cargo llvm-cov --version
```

### 3. Build tools
- **Visual Studio 2022** Build Tools (MSVC v143, Windows 11 SDK 10.0.22621+)
- **git** ≥ 2.40

### 4. WebView2 (for settings window dev)
The Evergreen bootstrapper is bundled in the installer. For local dev, install:
https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### 5. Node.js (for settings-ui build only)
- Node.js ≥ 20 LTS
- `npm install -g esbuild`

---

## Build

```
# Debug build (all crates)
cargo build

# Release build
cargo build --release

# Run tests
cargo nextest run

# Run benchmarks
cargo bench

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check

# Security audit
cargo audit

# Dependency policy check
cargo deny check

# Coverage
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

## One-command full check (CI equivalent)
```
cargo fmt --check && cargo clippy -- -D warnings && cargo nextest run && cargo audit && cargo deny check
```

---

## Floor Machine (Performance Reference — D-PERF-02)

For performance testing on the Floor machine:
- Windows 10 22H2, 4 GB RAM, dual-core, integrated GPU (2015–2018 era)
- Run with WARP software renderer forced:
  ```
  set NUVRA_FORCE_WARP=1
  cargo run --release
  ```

---

## Registry

Nuvra writes only to `HKCU\Software\Nuvra`.
Uninstall removes `%LOCALAPPDATA%\Nuvra` and `%APPDATA%\Nuvra` (unless user opts to keep data).

---

## Owner-provided values needed before release (see docs/handoff/)

- Code-signing certificate
- Microsoft Store developer account
- Update manifest URL and signing key
- See `docs/handoff/phase-0.md` Section 11 for full list.
