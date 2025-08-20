# Pigeon

Pigeon is a secure, peer‑to‑peer messaging client and library written in Rust, featuring encrypted messaging, a local queue with retries/backoff, and an optional libp2p networking layer with a desktop GUI.

## Features

- End‑to‑end encryption with sodium (Curve25519 box)
- Message queue with priorities, retries, and dead‑letter handling
- libp2p networking (TCP, Noise, Yamux, Request/Response, mDNS)
- Desktop GUI (egui/eframe) for onboarding, contacts, compose/send, inbox
- At‑rest encryption with passphrase support and rotation
- Metrics/ops hooks and basic observability

## Build

- Core/CLI:
```bash
cargo build
```

- With networking (enables libp2p and GUI networking):
```bash
cargo build --features network
```

## Run

- CLI:
```bash
cargo run --bin secure-p2p-msg
```

- GUI with networking:
```bash
cargo run --features network --bin pigeon-gui -- --listen-addr "/ip4/0.0.0.0/tcp/4001"
```

If `--listen-addr` is omitted, an ephemeral port is used. You can also set `PIGEON_LISTEN_ADDR` or configure `pigeon/config.toml`.

## Contacts: Required Inputs

- Name: Non‑empty label.
- Multiaddr: Must start with `/`. Examples:
  - `/ip4/127.0.0.1/tcp/4001`
  - `/ip4/192.168.1.50/tcp/4001`
  - `/dns4/example.com/tcp/4001`
  - Append `/p2p/<peer-id>` if available
- PubKey (hex): 32‑byte sodium box public key encoded as 64 hex characters.

## Config

Template config is created on first run under your OS config dir (e.g., `%APPDATA%/pigeon/config.toml`). Keys:

```toml
[network]
# listen_addr = "/ip4/0.0.0.0/tcp/4001"
# enable_mdns = false
```

Environment overrides:
`PIGEON_DATA_DIR`, `PIGEON_LOG_LEVEL`, `PIGEON_LISTEN_ADDR`, `PIGEON_ENABLE_MDNS`

## How it works

High‑level flow from compose to receive:

1) Onboarding and identity
- On first run, the app generates an identity containing:
  - A sodium box keypair (Curve25519) for message encryption/decryption
  - A sodium signing keypair (ed25519) for message authenticity
  - A libp2p ed25519 keypair (feature‑gated) for peer networking
- Identity is stored in `identity.bin` under the data dir. At‑rest key material is protected by a sealed key, optionally wrapped by a passphrase.

2) Contacts and addressing
- Each contact stores:
  - `name`
  - `addr` (libp2p multiaddr the peer listens on)
  - `public_key` (their sodium box public key, 32 bytes)
- Contacts are validated and stored in a small embedded database (`sled`), encrypted at rest via `AtRestKey`.

3) Compose and queue
- When composing, the plaintext is enqueued as a `QueuedMessage` with metadata (contact_id, created time, priority, retry counters).
- The queue is a `sled` tree with priority lanes; items are scheduled by `next_attempt_at` and retried with exponential backoff up to a max.

4) Encrypt and send (networking feature)
- The send loop (or explicit send) fetches a pending message, loads the recipient’s sodium box public key, and encrypts the payload using `crypto::box_` with a fresh nonce.
- A message envelope is built containing version, sender/recipient IDs, nonce and ciphertext, and a detached ed25519 signature over these fields.
- The app dials the contact’s multiaddr using libp2p Request/Response and transmits the envelope.
- On failure (connect/send/ack), the message stays queued and is retried per backoff policy; on success, status is updated.

5) Receive and verify (networking feature)
- The listener accepts request‑response messages and attempts to decode them as an envelope.
- The signature is verified with the sender’s signing public key. Replay protection is enforced with a nonce store; duplicate nonces are rejected.
- If verification and decryption succeed, the plaintext is stored in the local inbox (also a `sled` tree), and an ACK is returned to the sender.

6) Inbox and GUI
- The GUI shows Inbox, Compose, Contacts, and a “My Address” tab exposing your dialable multiaddr and PeerId.
- Searching, exporting, and real‑time updates are wired to the queue/inbox APIs and an optional inbox watcher.

7) Settings and config
- Network settings (listen address, mDNS) are loaded from `config.toml` and env overrides; most changes can apply without restart.
- Security settings allow setting a passphrase and rotating the at‑rest key.

Key modules (brief):
- `src/identity.rs`: load/generate identity; persist to disk
- `src/storage/*`: at‑rest encryption, contacts DB, nonce store, message queue
- `src/messaging/*`: compose, send, receive, envelope definition
- `src/network/*`: libp2p setup (transport, behaviours, ping, request/response)
- `src/ui/cli.rs`, `src/bin/pigeon-gui.rs`: CLI and GUI frontends

Data at rest:
- Contacts, queue, and inbox entries are serialized with `bincode` and sealed with a key stored separately, which can be protected by a user passphrase and rotated.

## License

Apache-2.0