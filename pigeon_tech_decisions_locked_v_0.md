# Pigeon – Tech Decisions (Locked v0)

> To reduce churn and accelerate M0, we lock the core stack and conventions for the Rust implementation.

## Core Choices (Final)
- **Language & Std:** Rust 1.78+ (Edition 2021)
- **Crypto:** `sodiumoxide` (XChaCha20-Poly1305, X25519); consider `libsodium-sys` direct if needed
- **Networking:** `libp2p` (tcp-tokio, mdns, request-response); Noise for transport security
- **Async Runtime:** `tokio` (full features)
- **Persistence:** `sled` (encrypted-at-rest via app-level envelope)
- **Config:** `serde` + `toml`; env overrides via `std::env`
- **CLI:** `clap` 4.x
- **Logging/Tracing:** `log` + `env_logger` (upgrade path to `tracing`)
- **IDs/Time:** `uuid` v4, `chrono` with serde

## Architecture
- **Crate layout:** `lib` exposes domain APIs; `bin` contains thin CLI front-end
- **Modules:** `config`, `crypto`, `messaging`, `network`, `storage`, `ui`
- **Queues:** priority lanes (small/urgent vs bulk) with retry/backoff
- **Protocol:** libp2p Request/Response subprotocol for message exchange

## Security Posture
- Ephemeral session keys per connection for PFS
- Nonce uniqueness guaranteed; random nonces from libsodium
- Replay protection via timestamps + dedupe cache
- Secrets never logged; sensitive buffers zeroized where feasible

## Determinism & Reliability
- Deterministic retry/backoff schedule (configurable)
- Idempotent delivery with ACK semantics and message IDs (UUIDv4)
- Bounded resource use: rate limiting per peer, queue caps

## Build & Tooling
- `cargo fmt`, `cargo clippy -D warnings` required pre-commit
- `cargo-deny` optional in CI for deps/licenses
- Feature flags for experimental components (e.g., padding, alt transport)

## Testing
- Unit tests for crypto wrappers and queue logic
- Integration tests simulating peer sessions via libp2p swarm
- Fuzzing optional for decoders/parsers

## Milestones
- M0: CLI skeleton, config, storage queue, basic libp2p handshake and echo messaging
- M1: Reliable delivery semantics, persistence, backoff, and basic ping
- M2: Bandwidth prioritization and contact-specific configs

---
Notes:
- Changes to the above require an explicit decision and milestone reset if disruptive.


