## M1 – Peer-to-Peer Messaging (Stabilize + UX) – Task List

### Epic: Contacts & Identity
- [ ] M1-100: Persistent local identity (save/load libp2p keypair + sodium keys)
  - Acceptance: Deterministic peer ID across runs; keys securely stored (permissions)
- [ ] M1-101: Contacts CRUD with validation (name, multiaddr, pubkey)
  - Acceptance: CLI `contacts add/list/remove`; sled-backed with tests

### Epic: Networking Protocols
- [ ] M1-110: Request-response message envelope over libp2p (with versioning)
  - Acceptance: CLI `send-net` transmits encrypted envelope; listener decrypts, ACKs
- [ ] M1-111: NAT-friendly listen and dial (mdns optional, configurable listen addr)
  - Acceptance: Discovery or manual dial on LAN; config file + env override for listen

### Epic: Messaging Pipeline
- [ ] M1-120: Queue-driven send loop with backoff; resume on reconnect
  - Acceptance: Background task drains queue; retries respect `max_retries`
- [ ] M1-121: Inbox store and CLI browsing (filters, preview, export)
  - Acceptance: `inbox list --limit N` and `inbox show <id>` commands

### Epic: Security Hardening
- [ ] M1-130: Encrypt-at-rest for sled values (envelope AEAD key from KDF)
  - Acceptance: On-disk records are ciphertext; migration path clear
- [ ] M1-131: Message authentication (sign/verify) and replay protection (nonce store)
  - Acceptance: Invalid or replayed messages rejected; tests in place

### Epic: Config & UX
- [ ] M1-140: Config file sections for `network`, `storage`, `security`; env overrides
  - Acceptance: Default file written on first run; doc in README
- [ ] M1-141: CLI ergonomics (global flags: `--data-dir`, `--log-level`; helpful errors)
  - Acceptance: Consistent errors; integration tests cover common misuses

### Epic: CI/CD & Quality
- [ ] M1-150: CI matrix adds macOS; cache tuning; test artifacts upload
  - Acceptance: Green across Windows/Linux/macOS; stable cache hits
- [ ] M1-151: Code coverage job (optional) and badge; flaky-test guard
  - Acceptance: Coverage report published; no flaky tests on main

### Exit Criteria (M1)
- [ ] Send/receive encrypted envelopes between two peers by name (via contacts)
- [ ] Queue drains reliably under intermittent connectivity
- [ ] Configurable listen/dial and basic discovery on LAN
- [ ] Encrypted-at-rest storage for messages and contacts
- [ ] Usable CLI with helpful commands and docs


