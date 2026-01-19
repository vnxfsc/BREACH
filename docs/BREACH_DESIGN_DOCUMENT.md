# BREACH - Titan Hunter

## Project Design Document v1.0

> Created: 2026-01-20  
> Project Type: Solana Web3 AR Collection & Battle Game  

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [World Setting](#2-world-setting)
3. [Core Decisions Summary](#3-core-decisions-summary)
4. [Technical Architecture](#4-technical-architecture)
5. [Titan System Design](#5-titan-system-design)
6. [NFT Data Structure](#6-nft-data-structure)
7. [Token Economics](#7-token-economics)
8. [Game Mechanics](#8-game-mechanics)
9. [Technology Stack](#9-technology-stack)
10. [Development Roadmap](#10-development-roadmap)

---

## 1. Project Overview

### 1.1 Project Positioning

BREACH is a Solana blockchain-based AR collection and battle game. Players track "Breach" signals in the real world to capture massive creatures called "Titans" from alternate dimensions. Through neural links, players establish connections with Titans and build their own Titan army for battle.

### 1.2 Core Selling Points

| Feature | Description |
|---------|-------------|
| **Giant Beast Aesthetic** | Pacific Rim-style massive monsters, distinct from cute pets |
| **True Ownership** | Titans exist as NFTs on the Solana blockchain |
| **AR Immersive Experience** | Discover and capture Titans in the real world |
| **Deep Strategy** | Gene system + turn-based PvP combat |
| **Decentralized** | Full attributes on-chain, verifiable data |

### 1.3 Target Users

- Web3 game players
- Monster/sci-fi enthusiasts
- Strategy game players

### 1.4 Market Positioning

- Primary Market: Western markets (US/EU)
- Style Tone: Dark, sci-fi, mature

---

## 2. World Setting

### 2.1 Background Story

```
2031. The first "Breach" tore open in the Mariana Trench.

What emerged from the Breach were not invaders, but "Titans" —
massive life forms from a parallel dimension. They are not inherently hostile,
but were driven to our world by dimensional collapse.

Humanity discovered: Titans can be tamed through "Neural Links."
A rare human genetic mutation — "Linkers" —
can establish mental connections with Titans and command them in battle.

You are an awakened Linker.
Track Breach signals in the real world, capture Titans, build your army.

"The Breach has opened. The Titans have awakened."
```

### 2.2 Core Concepts

| Concept | Description |
|---------|-------------|
| **Breach** | Space-time tear connecting two dimensions, where Titans emerge |
| **Titan** | Massive life form from an alternate dimension, can be tamed |
| **Linker** | Human capable of establishing neural links with Titans |
| **Neural Link** | Technology/ability connecting human and Titan consciousness |

### 2.3 Visual Style

- **Primary Colors**: Deep blue, dark gray, black
- **Accent Colors**: Bioluminescent blue, magma orange, void purple
- **Art References**: Pacific Rim, Godzilla, Shadow of the Colossus
- **UI Style**: Tech HUD, holographic projection

---

## 3. Core Decisions Summary

| Decision Item | Choice | Description |
|---------------|--------|-------------|
| Architecture | Backend Generation + On-chain Minting | Game logic runs on backend, NFTs on chain |
| IP/World Setting | BREACH - Titan Hunter | Pacific Rim-style giant monsters |
| Attribute System | Hybrid System | 4 visible attributes + 6-digit gene sequence |
| NFT Structure | Full On-chain | ~110 bytes PDA Account |
| Token Economy | Single Token $BREACH | Total supply 1 billion, 4-year release |
| Capture Mechanism | Neural Link Minigame | Rhythm click sync gameplay |
| Battle System | Hybrid Mode | PvE auto + PvP turn-based + Boss co-op |
| Tech Stack | Rust Backend | Flutter client + Pinocchio contracts |

---

## 4. Technical Architecture

### 4.1 Overall Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Layer (Mobile/Web)                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │
│  │AR Capture│  │Map Explore│  │  Battle  │  │ Market   │        │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Backend Services (Rust)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ API Service  │  │ Game Logic   │  │  Realtime    │          │
│  │ (Axum)       │  │   Service    │  │ (WebSocket)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Solana Blockchain Layer                     │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐                │
│  │ Titan NFT  │  │ $BREACH    │  │  Battle    │                │
│  │  Program   │  │   Token    │  │  Program   │                │
│  └────────────┘  └────────────┘  └────────────┘                │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Data Flow

```
Capture Flow:
User discovers breach → Backend validates location → AR capture minigame → 
Backend generates Titan attributes → Signs authorization → On-chain Mint NFT → 
Update user inventory

Battle Flow (PvP):
Match opponent → Both sides select actions → Backend calculates results → 
On-chain records battle results → Distribute rewards
```

---

## 5. Titan System Design

### 5.1 Threat Level (Class)

| Class | Name | Size | Rarity | Capture Cost |
|-------|------|------|--------|--------------|
| Class I | Pioneer | 15-30m | Common (60%) | 10 $BREACH |
| Class II | Hunter | 30-60m | Uncommon (25%) | 50 $BREACH |
| Class III | Destroyer | 60-100m | Rare (10%) | 200 $BREACH |
| Class IV | Calamity | 100-200m | Epic (4%) | 1,000 $BREACH |
| Class V | Apex | 200m+ | Legendary (1%) | 5,000 $BREACH |

### 5.2 Titan Types

| Type | Origin | Characteristics | Habitat |
|------|--------|-----------------|---------|
| 🌊 Abyssal | Deep sea breach | Bioluminescence, pressure adaptation | Coastal, rivers, lakes |
| 🌋 Volcanic | Earth crust breach | Magma shell, high temperature | Volcanic zones, industrial areas |
| ⚡ Storm | Atmospheric breach | EM interference, flight capability | High altitude, storm zones |
| 🦴 Void | Dimensional gap | Phase shift, space distortion | Ruins, nighttime |
| 🧬 Parasitic | Ecological breach | Rapid evolution, adaptation | Cities, forests |
| 💀 Ossified | Death breach | Bone armor, undead energy | Cemeteries, historic sites |

### 5.3 Type Effectiveness

```
Abyssal ──strong──► Volcanic ──strong──► Storm
   ▲                                      │
   │                                      ▼
Ossified ◄──strong── Parasitic ◄──strong── Void
```

Effectiveness damage bonus: **1.5x**

### 5.4 Attribute System

#### Visible Attributes (Combat)

| Attribute | Abbr | Function |
|-----------|------|----------|
| Attack | ATK | Damage output |
| Defense | DEF | Damage reduction |
| Speed | SPD | Action order, combos |
| Health | HP | Durability |

#### Gene Sequence (NFT Core)

```
DNA: [A7]-[F2]-[3B]-[9C]-[E1]-[D4]
      ATK  SPD  DEF  GRW  SKL  MUT

Each gene position 0-255, mapped to grades:
S: 250-255 (Top 2%)
A: 230-249 (Top 10%)
B: 190-229 (Top 25%)
C: 140-189 (Top 45%)
D: 80-139  (Top 70%)
E: 30-79   (Top 88%)
F: 0-29    (Bottom 12%)
```

#### Attribute Calculation Formula

```
Attribute Value = Species Base × Class Multiplier × Gene Modifier × Level Growth

Gene Modifier = 0.5 + (Gene Value / 255)  // Range 0.5x - 1.5x
Level Growth = 1 + (Level × Growth Rate)
```

---

## 6. NFT Data Structure

### 6.1 On-chain Data (Solana PDA Account)

```rust
#[repr(C)]
pub struct Titan {
    /// Account discriminator (8 bytes)
    pub discriminator: [u8; 8],
    
    // ═══════════ Immutable Core Data ═══════════
    
    /// NFT Mint address
    pub mint: Pubkey,                    // 32 bytes
    
    /// Species ID (0-65535)
    pub species_id: u16,                 // 2 bytes
    
    /// Threat Class (1-5)
    pub class: u8,                       // 1 byte
    
    /// Type (0-5: Abyssal, Volcanic, Storm, Void, Parasitic, Ossified)
    pub titan_type: u8,                  // 1 byte
    
    /// Gene Sequence [ATK, SPD, DEF, GRW, SKL, MUT]
    pub genes: [u8; 6],                  // 6 bytes
    
    /// Capture timestamp
    pub catch_timestamp: i64,            // 8 bytes
    
    /// Capture location (latitude × 10^6)
    pub catch_lat: i32,                  // 4 bytes
    
    /// Capture location (longitude × 10^6)
    pub catch_lng: i32,                  // 4 bytes
    
    /// Generation (0=wild, 1+=fusion)
    pub generation: u8,                  // 1 byte
    
    /// Origin (0=Wild, 1=Hatch, 2=Fusion, 3=Event)
    pub origin: u8,                      // 1 byte
    
    // ═══════════ Mutable Growth Data ═══════════
    
    /// Current level (1-100)
    pub level: u8,                       // 1 byte
    
    /// Current experience
    pub exp: u32,                        // 4 bytes
    
    /// Learned skills bitmap (supports 64 skills)
    pub learned_skills: u64,             // 8 bytes
    
    /// Equipped skills (max 4, 0 = empty slot)
    pub equipped_skills: [u8; 4],        // 4 bytes
    
    /// Total enemies defeated
    pub kills: u32,                      // 4 bytes
    
    /// Total battle wins
    pub wins: u16,                       // 2 bytes
    
    /// Total battle losses
    pub losses: u16,                     // 2 bytes
    
    // ═══════════ Metadata ═══════════
    
    /// Last battle timestamp
    pub last_battle: i64,                // 8 bytes
    
    /// Reserved for expansion
    pub reserved: [u8; 15],              // 15 bytes
    
    /// Account bump
    pub bump: u8,                        // 1 byte
}

// Total: ~118 bytes (including 8-byte discriminator)
// Account rent: ~0.00160 SOL (one-time)
```

### 6.2 On-chain Instructions

| Instruction | Function | Trigger |
|-------------|----------|---------|
| `mint_titan` | Mint Titan NFT | After capture success |
| `add_exp` | Add experience | Battle/quest completion |
| `level_up` | Level up | When experience is sufficient |
| `learn_skill` | Learn skill | When level requirement met |
| `equip_skills` | Equip skills | Player manual change |
| `record_battle` | Record battle result | Battle end |
| `fuse_titans` | Fuse Titans | Two Titans combine |

### 6.3 Metadata (Arweave)

```json
{
  "name": "Abyssal Maw #00001247",
  "symbol": "TITAN",
  "description": "A Class II Abyssal Hunter emerged from the Pacific Breach...",
  "image": "ar://Qm...",
  "animation_url": "ar://Qm...",
  "external_url": "https://breach.gg/titan/1247",
  "attributes": [
    { "trait_type": "Species", "value": "Abyssal Maw" },
    { "trait_type": "Class", "value": "II - Hunter" },
    { "trait_type": "Type", "value": "Abyssal" },
    { "trait_type": "Gene - ATK", "value": "A" },
    { "trait_type": "Gene - SPD", "value": "F" },
    { "trait_type": "Gene - DEF", "value": "C" },
    { "trait_type": "Gene - GRW", "value": "S" },
    { "trait_type": "Gene - SKL", "value": "B" },
    { "trait_type": "Gene - MUT", "value": "D" },
    { "trait_type": "Generation", "value": "0" },
    { "trait_type": "Origin", "value": "Wild Capture" },
    { "trait_type": "Catch Location", "value": "New York, USA" },
    { "trait_type": "Catch Date", "value": "2026-01-20" }
  ]
}
```

---

## 7. Token Economics

### 7.1 $BREACH Basic Information

| Property | Value |
|----------|-------|
| Name | BREACH |
| Symbol | $BREACH |
| Chain | Solana (SPL Token) |
| Decimals | 9 |
| Total Supply | 1,000,000,000 (1 billion) |
| Initial Circulation | 50,000,000 (5%) |

### 7.2 Token Distribution

| Category | Percentage | Amount | Release Schedule |
|----------|------------|--------|------------------|
| Play-to-Earn | 35% | 350M | 4-year linear release |
| Ecosystem | 25% | 250M | DAO vote determines |
| Team (Vested) | 15% | 150M | 2-year cliff + 2-year release |
| Treasury | 10% | 100M | Governance controlled |
| Liquidity | 10% | 100M | TGE for DEX pools |
| Advisors | 5% | 50M | 1-year vesting |

### 7.3 Consumption Scenarios

| Scenario | Amount | Destination |
|----------|--------|-------------|
| Capture Class I | 10 $BREACH | 50% burn, 50% treasury |
| Capture Class II | 50 $BREACH | Same as above |
| Capture Class III | 200 $BREACH | Same as above |
| Capture Class IV | 1,000 $BREACH | Same as above |
| Capture Class V | 5,000 $BREACH | Same as above |
| Level Up (per level) | Level × 5 | 100% burn |
| Fusion | 500+ | 100% burn |
| Skill Learning | 100 | 100% burn |
| Skill Reset | 200 | 100% burn |
| Market Fee | 2.5% | 50% burn, 50% treasury |

### 7.4 Earning Scenarios

| Scenario | Amount | Source |
|----------|--------|--------|
| PvE Battle Victory | 5-50 | Rewards pool |
| PvP Battle Victory | 20-200 | Rewards pool |
| Daily Quests | 10-50 | Rewards pool |
| Weekly Quests | 100-500 | Rewards pool |
| Season Rankings | 100-100,000 | Rewards pool |
| Achievement Unlock | 50-5,000 | Rewards pool |

### 7.5 Staking System

| Tier | Stake Amount | Privileges |
|------|--------------|------------|
| Bronze | 1,000+ | 5% consumption discount |
| Silver | 10,000+ | 15% discount + rare breach priority |
| Gold | 50,000+ | 25% discount + governance rights |
| Diamond | 200,000+ | 30% discount + all privileges |

---

## 8. Game Mechanics

### 8.1 Capture Mechanism: Neural Link Minigame

```
Gameplay Flow:
1. Discover breach signal, approach to trigger AR interface
2. Titan appears in AR
3. Click "Establish Link" to enter minigame
4. Neural link sync interface appears
5. Cursor moves along track
6. Click screen when cursor passes "neural nodes"
7. Accurate click = Link strength +15%
8. Missed node = Link strength -10%
9. Reach 100% before time runs out = Capture success

Difficulty Factors:
• Higher Class → More nodes, faster cursor
• Titan anger → Track shakes/distorts
• Item "Stabilizer" → Slows speed for 5 seconds
```

### 8.2 Battle System

#### PvE - Auto Battle

- Daily grinding, quests, dungeons
- Automatic progression, can speed up/skip
- Focus on cultivation and team building
- Produces $BREACH and experience

#### PvP - Turn-based Strategy

```
Team Composition:
• 3 Titans form a team
• Each Titan carries 4 skills
• No duplicate species

Turn Flow:
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│Select Phase │ ──► │Reveal Phase │ ──► │Resolve Phase│
│    30s      │     │ Simultaneous│     │  Animation  │
└─────────────┘     └─────────────┘     └─────────────┘

Prediction System (Simultaneous Selection):
• "Defense" counters "Attack" (50% damage reduction)
• "Control" counters "Defense" (ignores defense)
• "Attack" counters "Control" (interrupts control)

Victory Condition:
• One side's 3 Titans all defeated = Loss
• 30 rounds without resolution = Judge by remaining HP total
```

#### Boss - Cooperative Realtime

- Multi-player cooperation against Class V Titans
- Simplified real-time operations (tap to cast skills)
- Emphasis on coordination and timing
- Drops rare items/Titan fragments

---

## 9. Technology Stack

### 9.1 Mobile - Flutter

| Category | Technology |
|----------|------------|
| Framework | Flutter 3.16+ (Dart 3) |
| State Management | Riverpod 2.0 |
| Routing | go_router |
| AR | ar_flutter_plugin |
| Maps | flutter_map + Mapbox |
| Wallet | solana_wallet_adapter |
| Animation | flutter_animate |
| 3D | flutter_3d_controller |
| Network | dio + retrofit |
| Local Storage | drift (SQLite) |

### 9.2 Backend - Rust

| Category | Technology |
|----------|------------|
| Web Framework | Axum 0.7 |
| Async Runtime | Tokio 1.35 |
| Serialization | Serde + serde_json |
| Database | SQLx 0.7 (PostgreSQL) |
| Cache | redis-rs |
| WebSocket | tokio-tungstenite |
| Validation | validator |
| Logging | tracing |
| Error Handling | thiserror + anyhow |
| Configuration | config-rs |

### 9.3 Blockchain - Solana

| Category | Technology |
|----------|------------|
| Contract Framework | Pinocchio 0.29 |
| Rust SDK | solana-sdk 1.17 |
| Client | solana-client |
| NFT | mpl-token-metadata |
| Compressed NFT | mpl-bubblegum |

### 9.4 Databases

| Type | Technology | Purpose |
|------|------------|---------|
| Primary DB | PostgreSQL 16 + PostGIS | Users, game data, geolocation |
| Cache | Redis 7 | Sessions, hot data |
| Search | Meilisearch | Titan/player search |
| Time Series | TimescaleDB | Analytics |

### 9.5 Infrastructure

| Category | Technology |
|----------|------------|
| Containers | Docker |
| Orchestration | Kubernetes |
| Cloud | AWS (EKS, RDS, ElastiCache) |
| CDN | CloudFront |
| Storage | Arweave (metadata) + S3 (assets) |
| CI/CD | GitHub Actions |
| Monitoring | Prometheus + Grafana |
| Tracing | Jaeger |

### 9.6 Project Structure

```
breach/
├── apps/
│   ├── mobile/              # Flutter mobile app
│   ├── web/                 # Web app
│   └── admin/               # Admin dashboard
│
├── backend/
│   ├── api/                 # Axum API service
│   ├── game-server/         # Game logic service
│   ├── realtime/            # WebSocket realtime service
│   └── worker/              # Background task processing
│
├── contracts/               # Pinocchio Solana contracts
│   ├── programs/
│   │   ├── titan/           # Titan NFT contract
│   │   ├── battle/          # Battle settlement contract
│   │   └── token/           # $BREACH Token contract
│   └── tests/
│
├── packages/                # Shared libraries
│   ├── common/              # Rust common types/utils
│   ├── game-logic/          # Core game logic
│   └── solana-client/       # Solana interaction wrapper
│
├── infrastructure/          # Deployment configs
│   ├── docker/
│   ├── k8s/
│   └── terraform/
│
└── docs/                    # Documentation
```

---

## 10. Development Roadmap

### Phase 1: Foundation (4-6 weeks)

- [ ] Project scaffolding setup
- [ ] Database schema design
- [ ] Solana contract base version
- [ ] Backend API framework
- [ ] Flutter project initialization
- [ ] Wallet connection functionality

### Phase 2: Core MVP (8-10 weeks)

- [ ] Titan NFT Mint contract
- [ ] Map + Breach generation system
- [ ] AR capture interface
- [ ] Neural link minigame
- [ ] Basic PvE auto battle
- [ ] $BREACH Token contract
- [ ] User inventory/Pokedex

### Phase 3: Feature Completion (6-8 weeks)

- [ ] PvP turn-based battle
- [ ] Ranking system
- [ ] Titan upgrade/evolution
- [ ] Skill system
- [ ] Market trading
- [ ] Quest/achievement system

### Phase 4: Social & Optimization (4-6 weeks)

- [ ] Guild system
- [ ] Boss cooperative battles
- [ ] Friend system
- [ ] Performance optimization
- [ ] Security audit

**Total Estimate: 22-30 weeks (5-7 months)**

---

## Appendix

### A. Glossary

| Term | Definition |
|------|------------|
| Breach | Space-time tear connecting dimensions |
| Titan | Giant creature from alternate dimension |
| Linker | Human capable of connecting with Titans |
| Neural Link | Mental connection between human and Titan |
| Gene Sequence | Core data determining Titan attribute limits |

### B. References

- [Solana Official Documentation](https://docs.solana.com/)
- [Pinocchio Framework Documentation](https://github.com/febo/pinocchio)
- [Metaplex NFT Standard](https://docs.metaplex.com/)
- [Flutter Official Documentation](https://flutter.dev/docs)
- [Axum Web Framework](https://docs.rs/axum/latest/axum/)

### C. Changelog

| Version | Date | Updates |
|---------|------|---------|
| v1.0 | 2026-01-20 | Initial version, all core decisions completed |

---

**End of Document**

*BREACH - Hunt. Capture. Dominate.*
