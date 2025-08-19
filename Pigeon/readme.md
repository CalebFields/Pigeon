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

## License

Apache-2.0