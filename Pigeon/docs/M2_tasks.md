## M2 – Throughput, UX Polish, and Hardening

### Epic: Bandwidth & Throughput
- [x] M2-200: Priority lanes (small/urgent vs bulk) in queue and send loop
  - Acceptance: Small messages are not starved by bulk transfers; tests assert scheduling fairness
- [x] M2-201: Chunked large-message transfer with integrity (blake2 checks) and reassembly
  - Acceptance: 10MB payload sent in chunks; reassembly verified; retry on missing chunks
- [x] M2-202: Rate limiting per peer and global caps
  - Acceptance: Configurable tokens/sec; tests verify shaping under load

### Epic: UX & CLI Improvements
- [x] M2-210: `send-net --contact <name>` default; remove need for pubkey/addr flags
  - Acceptance: Clean help/error messages; examples in README
- [x] M2-211: `inbox export <id> --out <file>` and `inbox search <term>`
  - Acceptance: Round-trip export/import; simple substring search
- [x] M2-212: Rich `contacts show <id>` incl. last seen, notes
  - Acceptance: CLI prints structured details

### Epic: Security & Privacy
- [x] M2-220: Optional padding for envelopes (configurable fixed sizes)
  - Acceptance: Padded on-the-wire sizes; tests ensure padding toggles correctly
- [x] M2-221: Passphrase-protected at-rest key (argon2id + salt) with `identity unlock`
  - Acceptance: Locked startup flow; unlock via CLI/env; negative tests included
- [x] M2-222: Key rotation procedure for sodium and libp2p keys
  - Acceptance: Rolling rotation with backward decrypt capability for grace period

### Epic: Networking Resilience
- [x] M2-230: Auto-redial/backoff for libp2p connections
  - Acceptance: Under intermittent connectivity, reconnects occur and queue drains
- [x] M2-231: Optional QUIC transport (feature-gated)
  - Acceptance: Smoke test using QUIC dials; fallback to TCP when disabled

### Epic: Observability & Ops
- [x] M2-240: Structured logging with `tracing` and `RUST_LOG` bridges
  - Acceptance: JSON logs optional; spans for send/receive flows
- [x] M2-241: Metrics (prometheus exporter) behind feature flag
  - Acceptance: `/metrics` exposes queue depth, send successes/failures

### Epic: CI/CD & Quality
- [x] M2-250: Windows/macOS/Linux release builds via CI and GitHub Releases
  - Acceptance: Tagged builds upload artifacts; checksum list published
- [x] M2-251: Long-running soak test simulating churn
  - Acceptance: 1-hour soak without panics; bounded memory/cpu

### Exit Criteria (M2)
- [x] Bulk transfers don’t starve small messages; fairness verified
- [x] CLI is contact-first and adds export/search ergonomics
- [x] Optional padding and passphrase lock for at-rest key
- [x] Auto-reconnect drains queue under unstable networks
- [x] Release artifacts published across platforms

