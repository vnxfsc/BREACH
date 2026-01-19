# 🦖 BREACH - Hunt. Capture. Dominate.

<div align="center">

**A Solana-powered AR monster hunting game**

[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?style=flat&logo=solana)](https://solana.com)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat&logo=rust)](https://rust-lang.org)
[![Flutter](https://img.shields.io/badge/Flutter-3.16+-blue?style=flat&logo=flutter)](https://flutter.dev)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat)](LICENSE)

</div>

---

## 🌟 Overview

**BREACH** is a Web3 AR mobile game where players hunt and capture massive creatures called **Titans** that emerge from dimensional rifts. Inspired by Pacific Rim's colossal monsters, BREACH brings the thrill of capturing giant beasts to the blockchain.

### Key Features

- 🗺️ **AR Capture** - Hunt Titans in the real world using augmented reality
- 🧬 **Gene System** - Each Titan has unique DNA determining its potential
- ⚔️ **Strategic Combat** - PvE auto-battles and PvP turn-based strategy
- 💎 **True Ownership** - Titans are fully on-chain NFTs on Solana
- 💰 **Play-to-Earn** - Earn $BREACH tokens through gameplay

---

## 🎮 Gameplay

### The Titans

Titans are classified by threat level (Class I-V) and type:

| Class | Name | Size | Rarity |
|-------|------|------|--------|
| I | Pioneer | 15-30m | Common (60%) |
| II | Hunter | 30-60m | Uncommon (25%) |
| III | Destroyer | 60-100m | Rare (10%) |
| IV | Calamity | 100-200m | Epic (4%) |
| V | Apex | 200m+ | Legendary (1%) |

### Types & Elements

```
🌊 Abyssal    → 🌋 Volcanic   → ⚡ Storm
    ↑                              ↓
💀 Ossified  ← 🧬 Parasitic  ← 🦴 Void
```

### Neural Link Capture

Establish a mental connection with wild Titans through a rhythm-based minigame. The higher the Class, the more challenging the capture!

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Mobile App (Flutter)                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Backend Services (Rust)                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  API Server │  │ Game Server │  │  Realtime   │         │
│  │   (Axum)    │  │   (Logic)   │  │ (WebSocket) │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Solana Blockchain                         │
│                                                              │
│  Custom Programs:                                            │
│  ┌─────────────┐  ┌─────────────┐                           │
│  │ Titan NFT   │  │ Game Logic  │                           │
│  │  Program    │  │   Program   │                           │
│  └─────────────┘  └─────────────┘                           │
│                                                              │
│  External Infrastructure:                                    │
│  $BREACH Token → Standard SPL Token                         │
│  Token Trading → Raydium / Orca / Jupiter                   │
│  NFT Trading   → Magic Eden / Tensor                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 📁 Project Structure

```
breach/
├── apps/
│   ├── mobile/              # Flutter mobile app
│   ├── web/                 # Web app
│   └── admin/               # Admin dashboard
│
├── backend/
│   ├── api/                 # REST API (Axum)
│   ├── game-server/         # Game logic service
│   ├── realtime/            # WebSocket service
│   └── worker/              # Background jobs
│
├── contracts/               # Solana programs (Pinocchio)
│   ├── programs/
│   │   ├── titan_nft/       # Titan NFT program
│   │   └── game_logic/      # Game logic program
│   └── tests/
│   # Note: $BREACH uses standard SPL Token (no custom program)
│
├── packages/                # Shared libraries
│   ├── common/              # Common types/utils
│   ├── game-logic/          # Core game logic
│   └── solana-client/       # Solana interaction
│
├── infrastructure/          # Deployment configs
│   ├── docker/
│   ├── k8s/
│   └── terraform/
│
└── docs/                    # Documentation
```

---

## 🚀 Deployment Status

| Network | Program | Program ID | Status |
|---------|---------|------------|--------|
| **Devnet** | Titan NFT | `3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7` | ✅ Live |
| Devnet | Game Logic | TBD | 🔜 Planned |
| Mainnet | All | TBD | 🔜 Planned |

**Explorer**: [View Titan NFT Program](https://explorer.solana.com/address/3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7?cluster=devnet)

---

## 🛠️ Getting Started

### Prerequisites

- Rust 1.75+
- Flutter 3.16+
- Solana CLI 2.0+
- Pinocchio 0.8+
- Docker & Docker Compose
- PostgreSQL 16+
- Redis 7+

### Local Development

1. **Clone the repository**

```bash
git clone https://github.com/your-org/breach.git
cd breach
```

2. **Start infrastructure**

```bash
docker-compose up -d postgres redis meilisearch
```

3. **Setup database**

```bash
cd backend/api
cargo sqlx database create
cargo sqlx migrate run
```

4. **Deploy Solana programs (devnet)**

```bash
cd contracts

# Build
cargo build-sbf

# Deploy (Titan NFT already deployed)
solana config set --url devnet
solana airdrop 2
solana program deploy target/deploy/titan_nft.so
```

5. **Run backend services**

```bash
# Terminal 1 - API
cd backend/api
cargo run

# Terminal 2 - Game Server
cd backend/game-server
cargo run

# Terminal 3 - Realtime
cd backend/realtime
cargo run
```

6. **Run mobile app**

```bash
cd apps/mobile
flutter pub get
flutter run
```

---

## 💰 Token Economics

### $BREACH Token

| Property | Value |
|----------|-------|
| Total Supply | 1,000,000,000 |
| Initial Circulation | 50,000,000 (5%) |
| Decimals | 9 |
| Chain | Solana |

### Distribution

| Allocation | Percentage | Amount |
|------------|------------|--------|
| Game Rewards | 40% | 400M |
| Team | 20% | 200M |
| Ecosystem | 15% | 150M |
| Investors | 15% | 150M |
| Liquidity | 10% | 100M |

### Use Cases

- 🎯 Capture costs
- ⬆️ Upgrade/evolution
- 🧬 Fusion
- 🏪 Marketplace fees
- 🗳️ Governance voting

---

## 📖 Documentation

- [Design Document](docs/BREACH_DESIGN_DOCUMENT.md) - Complete game design
- [Technical Specification](docs/TECHNICAL_SPECIFICATION.md) - Technical details
- [API Reference](docs/API_REFERENCE.md) - API documentation
- [Smart Contract Docs](docs/CONTRACTS.md) - Solana program docs

---

## 🗺️ Roadmap

### Phase 1: Foundation (Q1 2026)
- [ ] Project scaffolding
- [ ] Core smart contracts
- [ ] Basic API framework
- [ ] Flutter app setup

### Phase 2: MVP (Q2 2026)
- [ ] Titan NFT minting
- [ ] Map & breach system
- [ ] AR capture
- [ ] Basic PvE combat

### Phase 3: Full Release (Q3 2026)
- [ ] PvP battles
- [ ] Ranking system
- [ ] Marketplace
- [ ] Quest system

### Phase 4: Expansion (Q4 2026)
- [ ] Guild system
- [ ] Boss raids
- [ ] Cross-chain support
- [ ] Mobile app stores release

---

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guide](CONTRIBUTING.md) before submitting PRs.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Built with ❤️ on Solana**

*The Titans have awakened. Will you answer the call?*

</div>
