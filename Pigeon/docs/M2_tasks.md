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


## M3 – Federation, Groups, and Forward Secrecy

### Epic: Group Messaging & Attachments
- [ ] M3-300: Group chats with membership management and per-group keys
  - Acceptance: Create/invite/leave; messages fan-out to group members; ACL enforced
- [ ] M3-301: File attachments with chunking, integrity and optional deduplication
  - Acceptance: 50MB attachment round-trip; blake2 integrity verified; resumable on reconnect
- [ ] M3-302: Message threading and reactions
  - Acceptance: Thread IDs maintained; simple reaction events appear in `show` output

### Epic: Forward Secrecy & Session Ratchets
- [ ] M3-310: Double Ratchet secure sessions (X25519 KDF chains)
  - Acceptance: Per-contact session established; lost-message tolerance; test vectors pass
- [ ] M3-311: Prekey bundles and session bootstrapping (X3DH-style)
  - Acceptance: New peer can start encrypted session without both being online
- [ ] M3-312: Graceful key rotation and rekey signaling
  - Acceptance: Sessions transparently rekey; old messages remain decryptable for retention window

### Epic: Federation & Relays
- [ ] M3-320: Store-and-forward relay nodes (opt-in)
  - Acceptance: Messages delivered via relay when peer is offline; relay stores only ciphertext
- [ ] M3-321: Rendezvous/Discovery service (beyond mDNS)
  - Acceptance: Peers can register and discover via rendezvous; basic auth and rate-limits
- [ ] M3-322: NAT traversal improvements (AutoNAT/UPnP/PCP)
  - Acceptance: Increased successful direct dials in NAT’d environments; tests simulate topologies

### Epic: Multi-device & Sync
- [ ] M3-330: Device linking with QR-code bootstrap
  - Acceptance: Secondary device links to primary; trust-on-first-use recorded
- [ ] M3-331: Encrypted state sync (inbox, contacts) with CRDT/oplog
  - Acceptance: Edits converge across devices; concurrent changes resolved deterministically
- [ ] M3-332: Replay and conflict hardening for multi-device
  - Acceptance: Nonce/clock protections extended to multi-device scenarios

### Epic: Search & Indexing
- [ ] M3-340: Full-text index (tantivy) encrypted at rest
  - Acceptance: `inbox search` uses index; performance improves on 10k messages corpus
- [ ] M3-341: Advanced filters and tags
  - Acceptance: Filter by contact/date/tag; tags CRUD via CLI

### Epic: Packaging & Distribution
- [ ] M3-350: Installers/Bundles for Windows/macOS/Linux
  - Acceptance: msix/pkg/deb/rpm artifacts produced in releases; basic smoke tests pass
- [ ] M3-351: Package managers (winget/chocolatey/Homebrew)
  - Acceptance: Users can install/update via package managers; version pinning works

### Epic: Observability & Ops (Advanced)
- [ ] M3-360: Histograms and traces
  - Acceptance: Latency/size histograms exposed; optional tracing spans exported
- [ ] M3-361: Health and readiness endpoints
  - Acceptance: `/health` and `/ready` reflect queue/db/network readiness

### Epic: Scale & Performance
- [ ] M3-370: Benchmarks and load generator
  - Acceptance: Reproducible benchmarks for queue, crypto, network; baseline checked into repo
- [ ] M3-371: Storage tuning and compaction
  - Acceptance: Documented sled/DB tunables; long-run compaction keeps disk usage bounded

### Epic: Quality & CI
- [ ] M3-380: Fuzzing critical parsers and protocol handlers (cargo-fuzz)
  - Acceptance: CI runs smoke fuzz; corpus kept; crashes minimized to zero known
- [ ] M3-381: Crypto Known-Answer Tests (KATs)
  - Acceptance: Deterministic vectors for envelope, ratchet steps, and padding
- [ ] M3-382: 24h soak with churn and network partitions
  - Acceptance: No panics; steady resource usage; queue drains post-partition

### Exit Criteria (M3)
- [ ] Group chats and attachments usable via CLI with tests
- [ ] Sessions provide forward secrecy; prekey bootstraps new contacts
- [ ] Delivery resilient via relay/rendezvous when peers are offline
- [ ] Multi-device sync converges without data loss
- [ ] Packaged releases available via installers and package managers
