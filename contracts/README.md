# BREACH Smart Contracts

Solana programs for the BREACH game, built with [Pinocchio](https://github.com/febo/pinocchio) framework.

## 🚀 Deployment Status

| Network | Program | Program ID | Status |
|---------|---------|------------|--------|
| **Devnet** | Titan NFT | `3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7` | ✅ Live |
| **Devnet** | Game Logic | `DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX` | ✅ Live |
| Mainnet | All | TBD | 🔜 Planned |

**Explorer**:
- [Titan NFT Program](https://explorer.solana.com/address/3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7?cluster=devnet)
- [Game Logic Program](https://explorer.solana.com/address/DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX?cluster=devnet)

## Programs

### Titan NFT Program (`titan_nft`)

Core NFT program for Titan management.

**Instructions:**
| ID | Name | Description |
|----|------|-------------|
| 0 | `initialize` | Initialize program config |
| 1 | `mint_titan` | Mint new Titan NFT |
| 2 | `level_up` | Level up a Titan |
| 3 | `evolve` | Evolve a Titan |
| 4 | `fuse` | Fuse two Titans |
| 5 | `transfer` | Transfer Titan ownership |
| 6 | `update_config` | Update program config (admin) |
| 7 | `set_paused` | Pause/unpause program (admin) |

**Accounts:**
| Account | Size | Description |
|---------|------|-------------|
| `GlobalConfig` | 182 bytes | Program configuration (packed) |
| `TitanData` | 118 bytes | Titan NFT data (packed) |
| `PlayerAccount` | 152 bytes | Player profile |

---

### Game Logic Program (`game_logic`)

Battle records, capture validation, experience/rewards distribution.

**Instructions:**
| ID | Name | Description |
|----|------|-------------|
| 0 | `initialize` | Initialize game config |
| 1 | `record_capture` | Record a Titan capture |
| 2 | `record_battle` | Record a battle result |
| 3 | `add_experience` | Add experience to a Titan |
| 4 | `distribute_reward` | Distribute $BREACH rewards |
| 5 | `update_config` | Update game config (admin) |
| 6 | `set_paused` | Pause/unpause program (admin) |

**Accounts:**
| Account | Size | Description |
|---------|------|-------------|
| `GameConfig` | 228 bytes | Game configuration (packed) |
| `BattleRecord` | 122 bytes | Battle record (packed) |
| `CaptureRecord` | 83 bytes | Capture record (packed) |

## Building

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v2.0.0/install)"

# Build
cargo build-sbf

# Test (Rust unit tests)
cargo test
```

## Testing

### TypeScript Integration Tests

```bash
cd tests
pnpm install
pnpm test
```

### Titan NFT Tests (22/22 passing)

📦 Basic Functionality:
- ✅ Initialize / Update Config
- ✅ Mint Titan (multiple elements)
- ✅ Read Player / Read Multiple Titans
- ✅ Level Up (requires EXP)
- ✅ Evolve (requires Lv30+)
- ✅ Fuse (requires Lv20+ & same element)
- ✅ Set Paused / Mint While Paused

🔒 Edge Cases:
- ✅ Invalid Element Type (rejected)
- ✅ Invalid Threat Class (rejected)
- ✅ Fuse With Self (rejected)
- ✅ Max Titans Per Wallet Check

🛡️ Authorization:
- ✅ Unauthorized Set Paused (rejected)
- ✅ Unauthorized Update Config (rejected)
- ✅ Not Owner Transfer (rejected)

### Game Logic Tests (15/15 passing)

📦 Basic Functionality:
- ✅ Initialize
- ✅ Update Backend Authority
- ✅ Read Game Config
- ✅ Record Capture (x3: different threat/element)
- ✅ Record Battle (x2: different outcomes)

🔒 Edge Cases:
- ✅ Expired Capture Signature (rejected)
- ✅ Battle Self (rejected)

🛡️ Authorization & Pause:
- ✅ Invalid Backend Authority (rejected)
- ✅ Unauthorized Set Paused (rejected)
- ✅ Set Paused True/False
- ✅ Record While Paused (rejected)

---

**Total: 37/37 tests passing** ✅

## Deployment

```bash
# Generate keypair (first time only)
solana-keygen new -o target/deploy/titan_nft-keypair.json

# Get program ID
solana address -k target/deploy/titan_nft-keypair.json

# Update program ID in lib.rs
# pub const PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("YOUR_PROGRAM_ID");

# Build
cargo build-sbf

# Deploy to devnet
solana config set --url devnet
solana airdrop 2
solana program deploy target/deploy/titan_nft.so --program-id target/deploy/titan_nft-keypair.json
```

## Project Structure

```
contracts/
├── Cargo.toml              # Workspace config
├── README.md
├── programs/
│   ├── titan_nft/          # Titan NFT Program
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Entry point
│   │       ├── error.rs        # Error definitions
│   │       ├── state/          # Account structures
│   │       │   ├── config.rs   # GlobalConfig (182 bytes)
│   │       │   ├── titan.rs    # TitanData (118 bytes)
│   │       │   └── player.rs   # PlayerAccount (152 bytes)
│   │       ├── instructions/   # Instruction handlers
│   │       └── utils/          # Gene calculations
│   │
│   └── game_logic/         # Game Logic Program
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs          # Entry point
│           ├── error.rs        # Error definitions
│           ├── state/          # Account structures
│           │   ├── config.rs   # GameConfig (228 bytes)
│           │   ├── battle.rs   # BattleRecord (122 bytes)
│           │   └── capture.rs  # CaptureRecord (83 bytes)
│           └── instructions/   # Instruction handlers
│
└── tests/
    ├── package.json
    ├── tsconfig.json
    ├── test-titan.ts           # Titan NFT tests (22)
    └── test-game-logic.ts      # Game Logic tests (15)
```

## Dependencies

| Package | Version | Description |
|---------|---------|-------------|
| `pinocchio` | 0.8 | Lightweight Solana program framework |
| `pinocchio-token` | 0.3 | SPL Token interactions |
| `pinocchio-system` | 0.2 | System program interactions |
| `pinocchio-pubkey` | 0.2 | Compile-time pubkey generation |

## Error Codes

| Range | Category |
|-------|----------|
| 6000-6099 | Authorization errors |
| 6100-6199 | Program state errors |
| 6200-6299 | Capture errors |
| 6300-6399 | Titan validation errors |
| 6400-6499 | Fusion errors |
| 6500-6599 | Token errors |
| 6600-6699 | Account errors |

## License

MIT
