# 🕊️ Pigeon: Secure Peer-to-Peer Messaging

**Pigeon** is a secure, lightweight messaging client that establishes direct encrypted connections between users. Designed with privacy and security at its core, Pigeon provides robust end-to-end encryption without the need for centralized servers—ensuring that your communications remain confidential and resilient.

---

## 🚀 Key Features

- 🔒 **Military-Grade Encryption**  
  XChaCha20-Poly1305 authenticated encryption with X25519 key exchange

- 🌐 **Direct P2P Connections**  
  No central servers—messages travel directly between peers

- ⏱️ **Receiver-Controlled Delivery**  
  Messages are only delivered when recipients actively check their inbox

- 🔁 **Persistent Message Queue**  
  Unsent messages are securely stored and retried automatically

- ⚖️ **Bandwidth Prioritization**  
  Small messages are delivered first—even during large file transfers

- 🔧 **Configurable Ping Intervals**  
  Set check-in frequency per contact (default: every 5 minutes)

---

## 🔐 Security Architecture

| Component          | Technology Used          | Security Features                                  |
|-------------------|--------------------------|----------------------------------------------------|
| **Encryption**     | Libsodium (XChaCha20)     | 256-bit keys, nonce reuse protection               |
| **Key Exchange**   | X25519 ECDH               | Perfect Forward Secrecy                            |
| **Transport**      | libp2p + Noise Protocol   | NAT traversal, encrypted peer-to-peer streams      |
| **Storage**        | Sled (encrypted at rest)  | ACID-compliant, zero-copy I/O                      |
| **Authentication** | HMAC-BLAKE2b              | Truncated 128-bit MACs                             |

### 🛡️ Threat Model Mitigations

| Threat Vector           | Pigeon's Protection                                        |
|-------------------------|------------------------------------------------------------|
| Message Interception    | End-to-end encryption with Perfect Forward Secrecy         |
| Replay Attacks          | Nonces + Timestamps in all messages                        |
| Traffic Analysis        | Fixed-size packet padding (512-byte blocks)                |
| Denial of Service       | Per-peer rate limiting and connection quotas               |
| Key Compromise          | Ephemeral session keys + OS-secured key storage            |

---

## ⚙️ Getting Started

### 🧰 Prerequisites

- Rust 1.65+ ([Install via `rustup`](https://rustup.rs))
- Linux or Windows (WSL recommended for Windows users)

### 📦 Installation

```bash
git clone https://github.com/yourusername/pigeon.git
cd pigeon
cargo build --release
