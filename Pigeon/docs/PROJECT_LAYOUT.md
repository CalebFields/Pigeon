## Project Layout

Top-level
- `Pigeon/` – Rust workspace crate (library + CLI binary)
- `scripts/` – Dev helpers (PowerShell): `fmt.ps1`, `lint.ps1`, `test.ps1`, `check.ps1`
- `readme.md` – Repo README

Inside `Pigeon/`
- `Cargo.toml` / `Cargo.lock` – crate manifest
- `readme.md` – crate README
- `config/` – default TOML config(s)
- `docs/` – internal docs
  - `M0_tasks.md`, `M1_tasks.md`, `M2_tasks.md`
  - `CODING_STANDARDS.md`, `TECH_DECISIONS_LOCKED_V0.md`
- `src/` – source code
  - `lib.rs` – crate facade (exports modules)
  - `main.rs` – binary entry (CLI)
  - `config.rs` – config loader (file + env overrides)
  - `error.rs` – unified error type
  - `identity.rs` – persistent key management (libp2p + sodium)
  - `crypto.rs` – crypto helpers
  - `ui/` – CLI
    - `mod.rs` – UI helpers (e.g., contact resolution)
    - `cli.rs` – clap CLI commands (contacts, inbox, send, listen, loops)
  - `messaging/` – message pipeline
    - `message.rs` – `EnvelopeV1` (nonce + ciphertext + signature)
    - `compose.rs` – enqueue plaintext for send
    - `send.rs` – immediate encrypt+enqueue
    - `receive.rs` – decrypt, store inbox, ACK
    - `queue.rs` – queue data structures and helpers
    - `send_loop.rs` – background retry/backoff and drain
  - `network/` – libp2p integration
    - `protocol.rs` – protocol aliases
    - `ping.rs` – ping behaviour and events
    - `rr.rs` – request/response codec and types
    - `manager.rs` – orchestration (if used)
  - `storage/` – sled-backed persistence
    - `queue.rs` – message queue, inbox, dead-letter
    - `contacts.rs` – contact store (encrypted at rest)
    - `at_rest.rs` – secretbox at-rest key + encrypt/decrypt
    - `nonce_store.rs` – replay protection store
- `tests/` – integration tests
  - `config_tests.rs`, `crypto_tests.rs`, `messaging_*`, `contact_resolution_tests.rs`, etc.
- `.github/workflows/ci.yml` – CI matrix and coverage jobs

Conventions
- Module dirs mirror domains; public API re-exported via `lib.rs`
- Feature-gated networking (`network`) where applicable
- All sled values encrypted at rest; keys under app data dir
- Tests avoid global state; use temp dirs and drop handles to release sled locks

