# ADR-0007 — Phone Transport: QUIC + Mutual TLS 1.3

**Status:** Accepted  
**Date:** 2026-11-21  
**Deciders:** Kesavaraja M (owner)

## Context (v2 planning, recorded in Phase 0)

A secure, low-latency transport is needed for the PC↔Android connection. The transport must be encrypted end-to-end, resistant to MITM on untrusted LANs, and work over the owner's optional relay (D-PHN-05).

## Decision

**QUIC (`quinn`) with mutual TLS 1.3 (`rustls`) using per-device self-signed Ed25519 certificates, pinned at pairing time (D-PHN-01). No Noise layer on top.**

## Alternatives Considered and Rejected

| Transport | Rejection Reason |
|---|---|
| TCP + TLS | Head-of-line blocking for multiplexed streams (file transfer + notifications simultaneously); QUIC solves this natively. |
| Noise Protocol | Additional layer on top of TLS 1.3 is redundant; mTLS already provides mutual authentication. |
| WebSocket / HTTP | No QUIC multiplexing; higher overhead for real-time media/battery updates. |

## Consequences

- Pairing: QR code (fingerprint + one-time 128-bit token + LAN address, 120 s expiry). 6-digit confirmation code on both screens (D-PHN-02).
- Discovery: mDNS `_nuvra._udp.local`. BLE RSSI for proximity (D-PHN-03).
- Wire schema: Protobuf (`prost`), backward-compatible (D-PHN-04).
- Relay: opaque packet forwarder; end-to-end TLS runs inside; relay cannot read content (D-PHN-05).
- File transfer: 256 KiB chunks, BLAKE3 verification, resumable, 2 GB cap (D-PHN-06).

## Implementation Plan

- `nuvra-core-shared/src/`: Protobuf schema (Phase 0 ADR only; code in v2).
- `nuvra-relay/`: self-hostable Rust relay service (v2).
- `android/`: Kotlin + Compose companion (v2).
- Run `system-design` skill before v2 protocol implementation.
