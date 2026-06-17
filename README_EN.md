# 🦀 TaoStorage — Your Data, Your Tao

> *"The Tao that can be told is not the eternal Tao."* — Lao Tzu
>
> *"The storage that can be censored is not the eternal storage."* — TaoStorage

**TaoStorage** is a decentralized, peer-to-peer storage network built with Rust. Inspired by the ancient wisdom of the *Tao Te Ching*, it models data as a living trinity of **Yin** (payload), **Yang** (metadata), and **Qi** (lifecycle) — forming a self-healing, self-organizing storage fabric.

### 🚀 One-line install

```bash
curl -fsSL https://tao.storage/install.sh | bash
```

That's it. You're now a node in the Tao network. 🌊

---

## 2. Why TaoStorage?

| Feature | TaoStorage 🦀 | Google Drive 🟢 | iCloud ☁️ |
|---|---|---|---|
| **Ownership** | You own your keys & data | Google holds the keys | Apple holds the keys |
| **Network Model** | P2P, no central server | Centralized | Centralized |
| **Privacy** | Zero-knowledge + homomorphic encryption | E2EE (metadata logged) | E2EE (metadata logged) |
| **Censorship** | Censorship-resistant by design | Subject to TOS takedowns | Subject to TOS takedowns |
| **Pricing** | Peer economy — you set it | $2.99/mo for 100 GB | $0.99/mo for 50 GB |
| **Smart Contracts** | WASM sandbox (Qi engine) | ❌ | ❌ |
| **Browser Node** | Open tab → you're a node | ❌ | ❌ |
| **Open Source** | ✅ MIT + Apache 2.0 | ❌ | ❌ |
| **Language** | Rust 🦀 | Go/C++ | Swift/ObjC |
| **Self-Healing** | Yes — 6-hexagram lifecycle | ❌ | ❌ |

**Bottom line:** TaoStorage is not just storage — it's a living, breathing data ecosystem that belongs to *you*.

---

## 3. 🧘 Philosophy — The Yin-Yang-Qi Trinity

Every piece of data in TaoStorage is a **DataUnit** — a trinity of three inseparable aspects:

### ☯️ Yin (阴) — The Body

> *"The valley spirit never dies — it is the mysterious female."* — Tao Te Ching, Ch. 6

**Yin** is the passive, receptive, dark vessel. In code terms, it's the **raw data payload** addressed by its SHA256 hash. Yin *holds* — it doesn't describe, it doesn't judge, it just *is*.

- Data payload + content-addressing (SHA256)
- Immutable, append-only
- Encrypted at rest with homomorphic encryption

### ☀️ Yang (阳) — The Form

> *"Naming is the origin of all particular things."* — Tao Te Ching, Ch. 1

**Yang** is the active, naming, structuring principle. It's the **metadata** that gives the raw Yin its meaning in the world:

- Tags, labels, descriptions
- Access control lists
- Access heat maps (how often is this data touched?)
- Provenance chain

### 🌬️ Qi (气) — The Life Force

> *"The ten thousand things carry yin and embrace yang. They achieve harmony by blending these breaths."* — Tao Te Ching, Ch. 42

**Qi** is the **lifecycle state machine** that governs the DataUnit's journey through the network. It breathes life into the Yin-Yang pair, managing when data lives, replicates, heals, or passes away.

The Qi engine follows **6 hexagrams** from the *I Ching* (Book of Changes):

| Hexagram | Name | State | Meaning |
|---|---|---|---|
| ䷂ | **Zhun** (屯) | Initial Struggling | Data is born — first replication, finding peers |
| ䷾ | **Jiji** (既济) | Already Complete | Data is fully replicated across the network |
| ䷊ | **Tai** (泰) | Peace / Prosperity | Data is healthy, frequently accessed, well-replicated |
| ䷋ | **Pi** (否) | Standstill / Decline | Access heat dropping — replication count falling |
| ䷖ | **Bo** (剥) | Stripping / Falling | Data is decaying — only 1–2 peers hold it |
| ䷁ | **Kun** (坤) | The Receptive — End | Data is reclaimed, space released — the end of the cycle |

When a DataUnit enters **Bo**, the Qi engine triggers **self-healing** — it requests re-replication from trusted peers, effectively rebooting the cycle back to Zhun. The network *heals itself* before data is lost.

```
           ┌───────────┐
           ▼           │
        ┌───┐     ┌──────┐
   ┌───▶│Zhun│────▶│Jiji │
   │    └───┘     └──────┘
   │                 │
   │    ┌───┐     ┌───▼─┐
   │    │Kun│◀───│ Tai │
   │    └───┘     └───▲─┘
   │                 │
   │    ┌───┐     ┌───┴─┐
   │    │Bo │◀───│ Pi  │
   │    └───┘     └─────┘
   │      │
   └──────┘ (self-heal)
```

---

## 4. 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (if compiling from source)
- Or just a browser (no install needed!)

### Install via curl (recommended)

```bash
curl -fsSL https://tao.storage/install.sh | bash
```

This downloads the prebuilt binary for your platform (Linux x86_64, ARM64, macOS).

### Verify

```bash
tao --version
# => TaoStorage v0.3.0 🦀
```

### Initialize your node

```bash
tao init
```

This generates your identity keypair and connects you to the Tao P2P network.

### Your first store

```bash
tao store ./my_document.pdf --tag research --public
# => Stored as: 3a7b...f9c2 (DataUnit ID)
# => Qi status: Zhun (replicating across 3 peers...)
```

### Retrieve

```bash
tao get 3a7b...f9c2
```

---

## 5. 💻 CLI Commands

| Command | Description |
|---|---|
| `tao init` | Initialize a new Tao node |
| `tao store <file>` | Store a file (Yin) with metadata (Yang) |
| `tao get <id>` | Retrieve a DataUnit by hash |
| `tao pin <id>` | Pin a DataUnit to prevent garbage collection |
| `tao unpin <id>` | Release a pinned DataUnit |
| `tao status <id>` | Check Qi lifecycle state of a DataUnit |
| `tao peers` | List connected peers |
| `tao info` | Show node identity and network info |
| `tao invite generate <node_id>` | Generate an invite code |
| `tao invite use <CODE> <new_node_id>` | Join via invite code |
| `tao reputation` | Show invite leaderboard & badges |
| `tao heal` | Force a self-heal cycle on degraded DataUnits |
| `tao config` | View/edit configuration |
| `tao daemon` | Start the Tao node daemon |

---

## 6. 🌐 Browser Node

Go to **[tao.storage](https://tao.storage)** in any modern browser. You are instantly a P2P node. No install. No account. No tracking.

```
┌─────────────────────────────────────┐
│          tao.storage                │
│  ┌──────────────────────────────┐   │
│  │  🦀 You are a Tao node!     │   │
│  │  Connected to 12 peers       │   │
│  │                              │   │
│  │  [Drag & Drop files here]    │   │
│  │  ┌──────────────────────┐   │   │
│  │  │   your-file.pdf      │   │   │
│  │  │   └─▶ Replicating... │   │   │
│  │  └──────────────────────┘   │   │
│  │  Qi: Zhun  •  3 peers       │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
```

**How it works:** The browser page loads a WASM bundle that runs libp2p over WebRTC. Your browser becomes a first-class citizen in the P2P network — storing, serving, and healing data alongside native nodes.

---

## 7. 🎟️ Invite System

TaoStorage grows through **invitations** — a ranked referral system with real reputation.

### Commands

```bash
# Generate an invitation code for a friend
tao invite generate <your_node_id>

# Friend joins using your code
tao invite use <CODE> <new_node_id>

# Check your rank
tao reputation
```

### Ranks

| Rank | Title | Requirement |
|---|---|---|
| 🎋 | **Fan** (凡 — Mortal) | Default — joined recently |
| 🏔️ | **Shi** (士 — Scholar) | Invited 3+ users |
| 🌊 | **Dao** (道 — Wayfarer) | Invited 10+ users |
| ☯️ | **Xuan** (玄 — Mysterious) | Invited 50+ users |
| 🐉 | **Sheng** (圣 — Sage) | Invited 100+ users |

### Rewards

| Action | Reputation |
|---|---|
| Invite someone who joins | +10 rep |
| Join via invite | +5 rep |
| Host data for 7+ days | +2 rep/day |
| Trigger a successful self-heal | +3 rep |

### Badges

| Badge | Requirement | Emoji |
|---|---|---|
| 🗣️ Preacher | 10 invites | 🥇 |
| 🎓 Master | 100 invites | 👑 |
| ⚡ Lightning | Heal 50 DataUnits | ⚡ |
| 🌐 Global | Connected from 3+ continents | 🌍 |

---

## 8. 🏗️ Architecture

```
┌───────────────────────────────────────────────────────────┐
│                     TaoStorage Node                        │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐   │
│  │   Yin Layer   │  │  Yang Layer  │  │   Qi Engine    │   │
│  │  (Data Store) │  │  (Metadata)  │  │  (Lifecycle)   │   │
│  │               │  │              │  │                │   │
│  │  • sled DB    │  │  • Tag DB    │  │  • Hexagram    │   │
│  │  • SHA256     │  │  • ACL       │  │    State       │   │
│  │  • Content    │  │  • Access    │  │    Machine     │   │
│  │    Addressed  │  │    Heat Map  │  │  • Self-Heal   │   │
│  │  • Encryption │  │  • Provenance│  │  • GC Trigger  │   │
│  └──────┬───────┘  └──────┬───────┘  └───────┬────────┘   │
│         │                 │                   │            │
│         └─────────┬───────┴───────────────────┘            │
│                   │                                        │
│          ┌────────▼────────┐                               │
│          │   libp2p P2P    │                               │
│          │   (Transport)   │                               │
│          │                 │                               │
│          │  • TCP/IP       │                               │
│          │  • WebRTC       │                               │
│          │  • DHT          │                               │
│          │  • GossipSub    │                               │
│          └────────┬────────┘                               │
│                   │                                        │
│          ┌────────▼────────┐                               │
│          │   WASM Sandbox  │                               │
│          │  (Smart Contr.) │                               │
│          └─────────────────┘                               │
│                                                           │
│  ┌──────────────────────────────────────────────────┐     │
│  │  Zero-Knowledge Proof Verifier                    │     │
│  │  Homomorphic Encryption Engine                    │     │
│  └──────────────────────────────────────────────────┘     │
└───────────────────────────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology |
|---|---|
| Language | **Rust** 🦀 |
| P2P Networking | **libp2p** (TCP + WebRTC) |
| Embedded DB | **sled** (Yin payload + Yang metadata) |
| Smart Contracts | **WASM sandbox** (Qi engine) |
| Browser Runtime | **WASM + WebRTC** |
| Privacy | **Zero-Knowledge Proofs** (storage verification) |
| Encryption | **Homomorphic encryption** (compute on encrypted data) |

---

## 9. 🗺️ Roadmap

### ✅ v0.1.0 — Prototype (PoC)
- [x] Basic P2P file store/retrieve
- [x] SHA256 content addressing
- [x] CLI tool

### ✅ v0.2.0 — Yin-Yang-Qi
- [x] Metadata layer (Yang)
- [x] Lifecycle state machine (Qi)
- [x] Self-healing prototype

### ✅ v0.3.0 — Current Release
- [x] Rust rewrite (from previous prototype)
- [x] libp2p networking
- [x] Browser node (WASM + WebRTC)
- [x] Invite system with ranks & badges
- [x] sled embedded database
- [ ] ~~Progressive Web App (PWA)~~
- [ ] ~~Mobile SDK (iOS/Android)~~

### 🔜 v0.4.0 — The Living Network
- [ ] Zero-Knowledge Proof storage verification
- [ ] Homomorphic encryption engine
- [ ] WASM smart contract execution (Qi engine)
- [ ] Encrypted group sharing

### 🔜 v0.5.0 — The Ecosystem
- [ ] TaoFS — FUSE filesystem mount
- [ ] IPFS gateway compatibility
- [ ] WebTorrent seeding bridge
- [ ] Decentralized DNS via ENS integration

### 🔮 v1.0.0 — The Tao
- [ ] Token economics (storage credits)
- [ ] DAO governance
- [ ] Cross-chain bridges (Polkadot, Cosmos)
- [ ] AI agent data layer

---

## 10. 🤝 Contributing

TaoStorage is open source under **MIT + Apache 2.0**.

### We welcome contributions of all kinds

| Type | How |
|---|---|
| 🐛 **Bugs** | Open a GitHub issue |
| 💡 **Ideas** | Start a discussion |
| 🔧 **Code** | Fork + PR |
| 📖 **Docs** | PR to `/docs` |
| 🌍 **Translations** | PR to `/translations` |

### Development setup

```bash
git clone https://github.com/taostorage/tao.git
cd tao
cargo build --release
cargo test
```

### Community

- 🌐 [tao.storage](https://tao.storage) — Web node & landing page
- 🐦 [@TaoStorage](https://x.com/TaoStorage) — Updates & memes
- 💬 [Discord](https://discord.gg/taostorage) — Chat with devs
- 📖 [Wiki](https://github.com/taostorage/tao/wiki) — Deep dives

---

## 📜 License

Dual-licensed under **MIT** or **Apache 2.0** at your option.

```
Copyright 2024–2026 TaoStorage Contributors

SPDX-License-Identifier: MIT OR Apache-2.0
```

---

<p align="center">
  <strong>🦀 The Tao that can be stored is not the eternal Tao.</strong><br>
  <em>But we're getting close.</em> 🌊
</p>
