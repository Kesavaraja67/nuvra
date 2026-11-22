# Third-Party Licenses

This file is updated with every dependency added. It is required by D-ARC-01.

## Rust crates

| Crate | License | Version |
|---|---|---|
| windows | MIT OR Apache-2.0 | 0.58 |
| tokio | MIT | 1.x |
| tracing | MIT | 0.1 |
| tracing-subscriber | MIT | 0.3 |
| tracing-appender | MIT | 0.2 |
| serde | MIT OR Apache-2.0 | 1 |
| serde_json | MIT OR Apache-2.0 | 1 |
| toml | MIT OR Apache-2.0 | 0.8 |
| rusqlite | MIT | 0.31 |
| aes-gcm | MIT OR Apache-2.0 | 0.10 |
| zeroize | MIT OR Apache-2.0 | 1 |
| blake3 | MIT OR Apache-2.0 | 1 |
| sha2 | MIT OR Apache-2.0 | 0.10 |
| argon2 | MIT OR Apache-2.0 | 0.5 |
| ed25519-dalek | MIT OR Apache-2.0 | 2 |
| thiserror | MIT OR Apache-2.0 | 1 |
| anyhow | MIT OR Apache-2.0 | 1 |
| flume | MIT OR Apache-2.0 | 0.11 |
| arc-swap | MIT OR Apache-2.0 | 1 |
| parking_lot | MIT OR Apache-2.0 | 0.12 |
| fluent-bundle | Apache-2.0 | 0.15 |
| unic-langid | MIT OR Apache-2.0 | 0.9 |
| rustfft | MIT OR Apache-2.0 | 6 |
| nucleo | MIT | 0.5 |
| quinn | MIT OR Apache-2.0 | 0.11 |
| rustls | MIT OR Apache-2.0 | 0.23 |
| rcgen | MIT OR Apache-2.0 | 0.13 |
| prost | Apache-2.0 | 0.13 |
| mdns-sd | MIT OR Apache-2.0 | 0.11 |
| uniffi | Mozilla Public License 2.0 | 0.28 |
| criterion | MIT OR Apache-2.0 | 0.5 |
| proptest | MIT OR Apache-2.0 | 1 |

## Fonts

Segoe UI Variable — Microsoft system font, licensed with Windows. Not bundled.

## No GPL/AGPL dependencies (R-26, D-ARC-01)

`cargo deny check` enforces this on every CI run.
