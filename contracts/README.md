# BREACH Smart Contracts

Solana programs for the BREACH game, built with [Pinocchio](https://github.com/febo/pinocchio) framework.

## 🚀 Deployment Status

| Network | Program ID | Status |
|---------|------------|--------|
| **Devnet** | `3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7` | ✅ Live |
| Mainnet | TBD | 🔜 Planned |

**Explorer**: [View on Solana Explorer](https://explorer.solana.com/address/3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7?cluster=devnet)

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

**Test Coverage (14/14 passing):**
- ✅ Initialize
- ✅ Update Config
- ✅ Mint Titan (multiple elements)
- ✅ Read Player
- ✅ Level Up (requires EXP)
- ✅ Evolve (requires Lv30+)
- ✅ Fuse (requires Lv20+ & same element)
- ✅ Set Paused
- ✅ Mint While Paused (rejected)

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
│   └── titan_nft/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs          # Entry point
│           ├── error.rs        # Error definitions (25+ error codes)
│           ├── state/          # Account structures
│           │   ├── mod.rs
│           │   ├── config.rs   # GlobalConfig (182 bytes)
│           │   ├── titan.rs    # TitanData (118 bytes)
│           │   └── player.rs   # PlayerAccount (152 bytes)
│           ├── instructions/   # Instruction handlers
│           │   ├── mod.rs
│           │   ├── initialize.rs
│           │   ├── mint_titan.rs
│           │   ├── level_up.rs
│           │   ├── evolve.rs
│           │   ├── fuse.rs
│           │   ├── transfer.rs
│           │   ├── update_config.rs
│           │   └── set_paused.rs
│           └── utils/          # Utilities
│               ├── mod.rs
│               └── genes.rs    # Gene calculations + tests
└── tests/
    ├── package.json
    ├── tsconfig.json
    └── test-titan.ts           # Integration tests
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
