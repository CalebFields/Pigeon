## M0 – Core Messaging Spike (2–3 weeks) – Task List

### Epic: Repo, Build, Tooling
- [ ] M0-001: Initialize CI (Windows/Linux): build + tests + clippy fmt gates
  - Acceptance: `cargo test` and `clippy -D warnings` pass in CI
- [ ] M0-002: Lint/format tooling docs and `make`/powershell helpers
  - Acceptance: One-liners to run fmt/lint/test locally

### Epic: Project Layout Skeleton
- [ ] M0-010: Crate layout with modules wired (`config`, `crypto`, `messaging`, `network`, `storage`, `ui`)
  - Acceptance: `cargo build` compiles; public façade in `lib.rs`
- [ ] M0-011: Coding standards doc committed and referenced in README
  - Acceptance: CI checks style (fmt/clippy)

### Epic: Configuration
- [ ] M0-020: Load config (TOML) + env overrides
  - Acceptance: Default config loads; env vars override; unit tests cover parsing

### Epic: Crypto
- [ ] M0-030: Sodium init + wrappers for XChaCha20-Poly1305 and X25519
  - Acceptance: Encrypt/decrypt roundtrip; ECDH shared secret derivation test

### Epic: Storage (sled)
- [ ] M0-040: Persistent message queue (enqueue/dequeue, metadata)
  - Acceptance: Messages persist across restarts; simple unit tests

### Epic: Networking (libp2p)
- [ ] M0-050: Basic libp2p swarm with Noise; request-response protocol stub
  - Acceptance: Two peers establish a secure channel locally
- [ ] M0-051: Ping service with configurable interval
  - Acceptance: Interval respected; logs show RTT

### Epic: Messaging Core
- [ ] M0-060: Compose/receive message types and serialization
  - Acceptance: Message structs serialize/deserialize; versioned envelope
- [ ] M0-061: Send flow: encrypt → enqueue → attempt immediate delivery
  - Acceptance: Local integration test succeeds for small payload
- [ ] M0-062: Receive flow: decrypt → ACK → persist
  - Acceptance: Local integration test verifies idempotency and ACK

### Epic: CLI
- [ ] M0-070: `send`, `fetch`, `config` subcommands
  - Acceptance: Manual smoke test works; help text documented

### Epic: Reliability
- [ ] M0-080: Retry with exponential backoff and max retries
  - Acceptance: Simulated failures show retries then dead-lettering

### Epic: Testing & QA
- [ ] M0-090: Integration test simulating two peers exchanging messages
  - Acceptance: Round-trip success; persisted queue drains on reconnect

### Exit Criteria (M0 Done)
- [ ] CLI can send and receive a message between two local peers
- [ ] Persistent queue survives restart and drains on reconnection
- [ ] CI green with fmt + clippy
- [ ] Basic ping works and is configurable


