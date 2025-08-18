## Milestone M3 — Hardening, Multi-Message, and UX Polish

Scope: Focus on production hardening, multi-message capabilities, improved UX, and operational polish. Build on M2 by adding batching, attachments, richer inbox actions, stronger key lifecycle management, and end-to-end demo scripts.

### Epics and Tasks

- Reliability & Throughput
  - M3-300: Batch send in `send_loop` (coalesce due messages, single session/multiplex)
  - M3-301: Chunked attachments with integrity (blake2) and reassembly
  - M3-302: Adaptive backoff caps based on recent success rate

- Security & Key Management
  - M3-310: Passphrase rotation workflow with seamless rekey of at-rest data
  - M3-311: Key backup/restore commands with checksum and dry-run verify
  - M3-312: Optional message-level forward secrecy (ephemeral X25519 per message)

- Inbox & Search UX
  - M3-320: Inbox labels and simple rules (e.g., sender → label)
  - M3-321: Full-text search index (minimal tokenizer) with AND/OR filters
  - M3-322: `inbox redact <id>` (scrub body, keep envelope metadata)

- Networking & Discovery
  - M3-330: Bootstrap peer list + auto-dial on startup (feature-gated)
  - M3-331: NAT hole punching experiment (if supported by libp2p version)
  - M3-332: Peer health cache and prefer-healthy dialing

- Observability & Ops
  - M3-340: Histograms for send latencies and retry counts
  - M3-341: Structured logs option (`--log-format json`)
  - M3-342: `ops check` to validate config, keys, and permissions

- CLI & DX Polish
  - M3-350: `pigeon demo local-two-peers` script (spin up listener + sender)
  - M3-351: Shell completions generation for common shells
  - M3-352: `--yes` non-interactive for all destructive ops

### Exit Criteria

- Batch send implemented and covered by tests (unit + integration).
- Attachments up to 10 MB supported with chunking, verified by tests.
- Passphrase rotation rekeys data; cold restore works; tests included.
- Inbox labels and redact command available and tested.
- Metrics extended with histograms; `/metrics` reflects new series.
- Demo script runs end-to-end locally; README documents usage.


