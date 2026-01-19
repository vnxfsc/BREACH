# BREACH Smart Contract Specification

> Solana Program Development Guide using Pinocchio Framework

---

## Table of Contents

1. [Overview](#1-overview)
2. [Technology Stack](#2-technology-stack)
3. [Program Architecture](#3-program-architecture)
4. [Account Structures](#4-account-structures)
5. [Titan NFT Program](#5-titan-nft-program)
6. [Game Logic Program](#6-game-logic-program)
7. [$BREACH Token (SPL Token)](#7-breach-token-spl-token)
8. [External Integrations](#8-external-integrations)
9. [Utilities](#9-utilities)
10. [Security Considerations](#10-security-considerations)
11. [Deployment Guide](#11-deployment-guide)
12. [Testing Strategy](#12-testing-strategy)

---

## 1. Overview

### 1.1 Purpose

This document specifies the smart contract architecture for BREACH, a Solana-powered AR monster hunting game.

**Custom Programs (2):**
- **Titan NFT Program**: Minting, attributes, evolution, fusion
- **Game Logic Program**: Capture validation, battle records, rewards

**Using Existing Infrastructure:**
- **$BREACH Token**: Standard SPL Token (compatible with all DEXs)
- **Token Trading**: Raydium / Orca / Jupiter
- **NFT Trading**: Magic Eden / Tensor

### 1.2 Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Gas Efficiency** | Use compressed NFTs (cNFTs) for Titans |
| **Minimal Overhead** | Pinocchio's zero-copy deserialization |
| **Security** | Manual authority checks, explicit validation |
| **Upgradability** | Program upgrades via authority |
| **Performance** | Pinocchio's lightweight runtime (~2KB overhead) |

### 1.3 Why Pinocchio?

| Feature | Pinocchio | Anchor |
|---------|-----------|--------|
| Program Size | ~2KB overhead | ~25KB overhead |
| CU Usage | Minimal | Higher due to macros |
| Control | Full manual control | Abstracted |
| Learning Curve | Steeper | Gentler |
| Flexibility | Maximum | Framework constraints |

---

## 2. Technology Stack

### 2.1 Core Technologies

```
┌─────────────────────────────────────────────────────────┐
│                    BREACH Smart Contracts               │
├─────────────────────────────────────────────────────────┤
│  Language:        Rust                                  │
│  Framework:       Pinocchio 0.8+                        │
│  Blockchain:      Solana (Mainnet-beta)                 │
│  NFT Standard:    Metaplex Token Metadata + Bubblegum   │
│  Token Standard:  SPL Token / Token-2022                │
│  Randomness:      Switchboard VRF                       │
│  Indexing:        Helius DAS API                        │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Dependencies

```toml
# Cargo.toml
[package]
name = "breach-titan"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
pinocchio = "0.8"
pinocchio-pubkey = "0.2"
pinocchio-token = "0.3"
pinocchio-system = "0.2"
solana-program = "2.0"

[dev-dependencies]
solana-program-test = "2.0"
solana-sdk = "2.0"
```

---

## 3. Program Architecture

### 3.1 Program Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      BREACH Program Suite                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Custom Programs (Self-developed)             │   │
│  │                                                           │   │
│  │  ┌──────────────┐           ┌──────────────┐             │   │
│  │  │  Titan NFT   │◄─────────►│  Game Logic  │             │   │
│  │  │   Program    │   CPI     │   Program    │             │   │
│  │  │              │           │              │             │   │
│  │  │ - mint       │           │ - capture    │             │   │
│  │  │ - level_up   │           │ - battle     │             │   │
│  │  │ - evolve     │           │ - reward     │             │   │
│  │  │ - fuse       │           │ - validate   │             │   │
│  │  │ - transfer   │           │              │             │   │
│  │  └──────────────┘           └──────────────┘             │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              External Infrastructure (No dev needed)      │   │
│  │                                                           │   │
│  │  $BREACH Token ──► Standard SPL Token                    │   │
│  │  Token Trading ──► Raydium / Orca / Jupiter              │   │
│  │  NFT Trading   ──► Magic Eden / Tensor                   │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Pinocchio Program Structure

```rust
// lib.rs - Program entry point
use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

mod instructions;
mod state;
mod error;

// Program ID
pinocchio::declare_id!("TitanXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

// Entrypoint
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse instruction discriminator (first byte)
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    
    match discriminator {
        0 => instructions::initialize(program_id, accounts, data),
        1 => instructions::mint_titan(program_id, accounts, data),
        2 => instructions::level_up(program_id, accounts, data),
        3 => instructions::evolve(program_id, accounts, data),
        4 => instructions::fuse(program_id, accounts, data),
        5 => instructions::transfer(program_id, accounts, data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
```

---

## 4. Account Structures

### 4.1 Manual Serialization with Pinocchio

```rust
// state/config.rs
use pinocchio::pubkey::Pubkey;

/// Global configuration account
/// PDA: ["config"]
/// Size: 8 + 32 + 32 + 32 + 32 + 32 + 2 + 2 + 2 + 2 + 4 + 1 + 1 + 8 + 8 + 8 + 8 + 1 = 215 bytes
#[repr(C)]
pub struct GlobalConfig {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Program authority (upgrade, pause)
    pub authority: Pubkey,
    /// Treasury wallet for fees
    pub treasury: Pubkey,
    /// $BREACH token mint
    pub breach_mint: Pubkey,
    /// Merkle tree for cNFTs
    pub merkle_tree: Pubkey,
    /// Capture authority (backend signer)
    pub capture_authority: Pubkey,
    /// Fee settings (basis points)
    pub capture_fee_bps: u16,
    pub marketplace_fee_bps: u16,
    pub fusion_fee_bps: u16,
    /// Game parameters
    pub max_titans_per_wallet: u16,
    pub capture_cooldown_seconds: u32,
    /// Pause flag
    pub paused: bool,
    /// Bump seed
    pub bump: u8,
    /// Statistics
    pub total_titans_minted: u64,
    pub total_battles: u64,
    pub total_fusions: u64,
    pub total_fees_collected: u64,
}

impl GlobalConfig {
    pub const SIZE: usize = 215;
    pub const DISCRIMINATOR: [u8; 8] = [0x47, 0x4C, 0x4F, 0x42, 0x43, 0x46, 0x47, 0x00]; // "GLOBCFG\0"
    
    /// Zero-copy read from account data
    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        // Safety: We've verified the length and GlobalConfig is repr(C)
        let config = unsafe { &*(data.as_ptr() as *const GlobalConfig) };
        
        if config.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        
        Ok(config)
    }
    
    /// Zero-copy mutable read
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let config = unsafe { &mut *(data.as_mut_ptr() as *mut GlobalConfig) };
        
        if config.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        
        Ok(config)
    }
}
```

### 4.2 Titan Data Structure

```rust
// state/titan.rs
use pinocchio::pubkey::Pubkey;

/// Titan NFT on-chain data (~110 bytes)
/// PDA: ["titan", mint_pubkey]
#[repr(C, packed)]
pub struct TitanData {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Unique identifier
    pub titan_id: u64,
    /// Species ID (determines base stats)
    pub species_id: u16,
    /// Threat class (1-5)
    pub threat_class: u8,
    /// Elemental type (0-5)
    pub element_type: u8,
    /// Visible attributes (0-255 each)
    pub power: u8,
    pub fortitude: u8,
    pub velocity: u8,
    pub resonance: u8,
    /// Hidden gene sequence (6 bytes)
    pub genes: [u8; 6],
    /// Current level (1-100)
    pub level: u8,
    /// Experience points
    pub experience: u32,
    /// Link strength with owner (0-100)
    pub link_strength: u8,
    /// Capture timestamp
    pub captured_at: i64,
    /// Original capturer
    pub original_owner: Pubkey,
    /// Location captured (encoded)
    pub capture_location: u64,
    /// Generation (0 = wild, 1+ = bred)
    pub generation: u8,
    /// Parent A ID (0 if wild)
    pub parent_a: u64,
    /// Parent B ID (0 if wild)
    pub parent_b: u64,
    /// Bump seed
    pub bump: u8,
}

impl TitanData {
    pub const SIZE: usize = 110;
    pub const DISCRIMINATOR: [u8; 8] = [0x54, 0x49, 0x54, 0x41, 0x4E, 0x44, 0x41, 0x54]; // "TITANDAT"
    
    /// Zero-copy read
    #[inline]
    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        let titan = unsafe { &*(data.as_ptr() as *const TitanData) };
        if titan.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(titan)
    }
    
    /// Calculate combat power
    pub fn combat_power(&self) -> u32 {
        let class_mult = match self.threat_class {
            1 => 100,
            2 => 115,
            3 => 130,
            4 => 150,
            5 => 180,
            _ => 100,
        };
        
        let base = (self.power as u32 + self.fortitude as u32 + 
                   self.velocity as u32 + self.resonance as u32) * 2;
        let level_bonus = self.level as u32 * 10;
        let gene_bonus: u32 = self.genes.iter().map(|g| *g as u32).sum::<u32>() / 10;
        
        (base + level_bonus + gene_bonus) * class_mult / 100
    }
}

/// Element type enumeration
#[repr(u8)]
pub enum ElementType {
    Abyssal = 0,
    Volcanic = 1,
    Storm = 2,
    Void = 3,
    Parasitic = 4,
    Ossified = 5,
}

impl ElementType {
    /// Returns damage multiplier (150 = 1.5x, 100 = 1.0x, 75 = 0.75x)
    pub fn get_multiplier(attacker: u8, defender: u8) -> u8 {
        // Effectiveness: Abyssal > Volcanic > Storm > Void > Parasitic > Ossified > Abyssal
        let beats = [1, 2, 3, 4, 5, 0]; // What each type beats
        
        if beats[attacker as usize % 6] == defender {
            150 // Super effective
        } else if beats[defender as usize % 6] == attacker {
            75  // Not very effective
        } else {
            100 // Normal
        }
    }
}
```

### 4.3 Player Account

```rust
// state/player.rs
use pinocchio::pubkey::Pubkey;

/// Player profile account
/// PDA: ["player", wallet_pubkey]
#[repr(C)]
pub struct PlayerAccount {
    pub discriminator: [u8; 8],
    /// Wallet address
    pub wallet: Pubkey,
    /// Username (32 bytes max, null-padded)
    pub username: [u8; 32],
    /// Total Titans captured
    pub titans_captured: u32,
    /// Current Titans owned
    pub titans_owned: u32,
    /// Total battles won
    pub battles_won: u32,
    /// Total battles lost
    pub battles_lost: u32,
    /// PvP Elo rating
    pub elo_rating: u16,
    /// Peak Elo rating
    pub peak_elo: u16,
    /// Last capture timestamp
    pub last_capture_at: i64,
    /// $BREACH spent total
    pub total_breach_spent: u64,
    /// $BREACH earned total
    pub total_breach_earned: u64,
    /// Account creation timestamp
    pub created_at: i64,
    /// Bump seed
    pub bump: u8,
    /// Reserved for future use
    pub _reserved: [u8; 15],
}

impl PlayerAccount {
    pub const SIZE: usize = 152;
    pub const DISCRIMINATOR: [u8; 8] = [0x50, 0x4C, 0x41, 0x59, 0x45, 0x52, 0x41, 0x43]; // "PLAYERAC"
    pub const INITIAL_ELO: u16 = 1000;
    
    /// Check if capture cooldown has elapsed
    pub fn can_capture(&self, current_time: i64, cooldown_seconds: u32) -> bool {
        current_time - self.last_capture_at >= cooldown_seconds as i64
    }
    
    /// Calculate win rate (0-100)
    pub fn win_rate(&self) -> u8 {
        let total = self.battles_won + self.battles_lost;
        if total == 0 {
            return 50;
        }
        ((self.battles_won as u64 * 100) / total as u64) as u8
    }
}
```

---

## 5. Titan NFT Program

### 5.1 Instruction Definitions

```rust
// instructions/mod.rs
use pinocchio::{
    account_info::AccountInfo,
    pubkey::Pubkey,
    program_error::ProgramError,
    ProgramResult,
};

pub mod initialize;
pub mod mint_titan;
pub mod level_up;
pub mod evolve;
pub mod fuse;
pub mod transfer;

/// Instruction discriminators
pub const INITIALIZE: u8 = 0;
pub const MINT_TITAN: u8 = 1;
pub const LEVEL_UP: u8 = 2;
pub const EVOLVE: u8 = 3;
pub const FUSE: u8 = 4;
pub const TRANSFER: u8 = 5;
pub const UPDATE_CONFIG: u8 = 6;
pub const SET_PAUSED: u8 = 7;
```

### 5.2 Initialize Instruction

```rust
// instructions/initialize.rs
use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;

use crate::state::GlobalConfig;

/// Initialize instruction data layout
#[repr(C, packed)]
pub struct InitializeData {
    pub treasury: Pubkey,
    pub breach_mint: Pubkey,
    pub merkle_tree: Pubkey,
    pub capture_authority: Pubkey,
    pub capture_fee_bps: u16,
    pub marketplace_fee_bps: u16,
    pub fusion_fee_bps: u16,
    pub max_titans_per_wallet: u16,
    pub capture_cooldown_seconds: u32,
}

pub fn initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        authority,          // [0] Signer, payer
        config_account,     // [1] Writable, PDA
        system_program,     // [2] System program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // Validate authority is signer
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse instruction data
    if data.len() < std::mem::size_of::<InitializeData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let init_data = unsafe { &*(data.as_ptr() as *const InitializeData) };
    
    // Derive config PDA
    let (config_pda, bump) = Pubkey::find_program_address(
        &[b"config"],
        program_id,
    );
    
    if config_pda != *config_account.key() {
        return Err(ProgramError::InvalidSeeds);
    }
    
    // Create config account
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(GlobalConfig::SIZE);
    
    CreateAccount {
        from: authority,
        to: config_account,
        lamports,
        space: GlobalConfig::SIZE as u64,
        owner: program_id,
    }.invoke_signed(&[&[b"config", &[bump]]])?;
    
    // Initialize config data
    let config_data = config_account.try_borrow_mut_data()?;
    let config = unsafe { &mut *(config_data.as_mut_ptr() as *mut GlobalConfig) };
    
    config.discriminator = GlobalConfig::DISCRIMINATOR;
    config.authority = *authority.key();
    config.treasury = init_data.treasury;
    config.breach_mint = init_data.breach_mint;
    config.merkle_tree = init_data.merkle_tree;
    config.capture_authority = init_data.capture_authority;
    config.capture_fee_bps = init_data.capture_fee_bps;
    config.marketplace_fee_bps = init_data.marketplace_fee_bps;
    config.fusion_fee_bps = init_data.fusion_fee_bps;
    config.max_titans_per_wallet = init_data.max_titans_per_wallet;
    config.capture_cooldown_seconds = init_data.capture_cooldown_seconds;
    config.paused = false;
    config.bump = bump;
    config.total_titans_minted = 0;
    config.total_battles = 0;
    config.total_fusions = 0;
    config.total_fees_collected = 0;
    
    Ok(())
}
```

### 5.3 Mint Titan Instruction

```rust
// instructions/mint_titan.rs
use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, Sysvar},
};
use pinocchio_token::instructions::Transfer;

use crate::state::{GlobalConfig, TitanData, PlayerAccount};
use crate::error::TitanError;

/// Mint instruction data
#[repr(C, packed)]
pub struct MintTitanData {
    pub species_id: u16,
    pub threat_class: u8,
    pub element_type: u8,
    pub power: u8,
    pub fortitude: u8,
    pub velocity: u8,
    pub resonance: u8,
    pub genes: [u8; 6],
    pub capture_lat: i32,
    pub capture_lng: i32,
    pub nonce: u64,
    pub signature: [u8; 64],
}

pub fn mint_titan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        payer,                  // [0] Signer, player
        config_account,         // [1] Config PDA
        player_account,         // [2] Player PDA (init if needed)
        titan_account,          // [3] New Titan PDA
        capture_authority,      // [4] Backend signer
        payer_token_account,    // [5] Player's $BREACH
        treasury_token_account, // [6] Treasury $BREACH
        token_program,          // [7] SPL Token
        system_program,         // [8] System program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // Validate signers
    if !payer.is_signer() || !capture_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Load config
    let config_data = config_account.try_borrow_data()?;
    let config = GlobalConfig::from_account_data(&config_data)?;
    
    // Check not paused
    if config.paused {
        return Err(TitanError::ProgramPaused.into());
    }
    
    // Validate capture authority
    if *capture_authority.key() != config.capture_authority {
        return Err(TitanError::InvalidCaptureAuthority.into());
    }
    
    // Parse mint data
    if data.len() < std::mem::size_of::<MintTitanData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mint_data = unsafe { &*(data.as_ptr() as *const MintTitanData) };
    
    // Validate threat class (1-5)
    if mint_data.threat_class < 1 || mint_data.threat_class > 5 {
        return Err(TitanError::InvalidThreatClass.into());
    }
    
    // Validate element type (0-5)
    if mint_data.element_type > 5 {
        return Err(TitanError::InvalidElementType.into());
    }
    
    // Get current time
    let clock = Clock::get()?;
    
    // Load or create player account
    let mut player_data = player_account.try_borrow_mut_data()?;
    let player = if player_data[0] == 0 {
        // Initialize new player
        let player = unsafe { &mut *(player_data.as_mut_ptr() as *mut PlayerAccount) };
        player.discriminator = PlayerAccount::DISCRIMINATOR;
        player.wallet = *payer.key();
        player.elo_rating = PlayerAccount::INITIAL_ELO;
        player.peak_elo = PlayerAccount::INITIAL_ELO;
        player.created_at = clock.unix_timestamp;
        player
    } else {
        PlayerAccount::from_account_data_mut(&mut player_data)?
    };
    
    // Check capture cooldown
    if !player.can_capture(clock.unix_timestamp, config.capture_cooldown_seconds) {
        return Err(TitanError::CaptureCooldown.into());
    }
    
    // Check max titans
    if player.titans_owned >= config.max_titans_per_wallet as u32 {
        return Err(TitanError::MaxTitansReached.into());
    }
    
    // Calculate capture fee based on threat class
    let base_fee: u64 = match mint_data.threat_class {
        1 => 10_000_000_000,    // 10 $BREACH
        2 => 50_000_000_000,    // 50 $BREACH
        3 => 200_000_000_000,   // 200 $BREACH
        4 => 1_000_000_000_000, // 1000 $BREACH
        5 => 5_000_000_000_000, // 5000 $BREACH
        _ => 10_000_000_000,
    };
    let fee = (base_fee * config.capture_fee_bps as u64) / 10000;
    
    // Transfer fee to treasury
    if fee > 0 {
        Transfer {
            from: payer_token_account,
            to: treasury_token_account,
            authority: payer,
            amount: fee,
        }.invoke()?;
    }
    
    // Generate titan ID
    let titan_id = config.total_titans_minted + 1;
    
    // Create titan account (PDA)
    let (titan_pda, titan_bump) = Pubkey::find_program_address(
        &[b"titan", &titan_id.to_le_bytes()],
        program_id,
    );
    
    if *titan_account.key() != titan_pda {
        return Err(ProgramError::InvalidSeeds);
    }
    
    // Initialize titan data
    let mut titan_account_data = titan_account.try_borrow_mut_data()?;
    let titan = unsafe { &mut *(titan_account_data.as_mut_ptr() as *mut TitanData) };
    
    titan.discriminator = TitanData::DISCRIMINATOR;
    titan.titan_id = titan_id;
    titan.species_id = mint_data.species_id;
    titan.threat_class = mint_data.threat_class;
    titan.element_type = mint_data.element_type;
    titan.power = mint_data.power;
    titan.fortitude = mint_data.fortitude;
    titan.velocity = mint_data.velocity;
    titan.resonance = mint_data.resonance;
    titan.genes = mint_data.genes;
    titan.level = 1;
    titan.experience = 0;
    titan.link_strength = 10; // Initial link strength
    titan.captured_at = clock.unix_timestamp;
    titan.original_owner = *payer.key();
    titan.capture_location = encode_location(mint_data.capture_lat, mint_data.capture_lng);
    titan.generation = 0;
    titan.parent_a = 0;
    titan.parent_b = 0;
    titan.bump = titan_bump;
    
    // Update player stats
    player.titans_captured += 1;
    player.titans_owned += 1;
    player.last_capture_at = clock.unix_timestamp;
    player.total_breach_spent += fee;
    
    // Update global stats (need mutable borrow)
    drop(config_data);
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GlobalConfig::from_account_data_mut(&mut config_data)?;
    config.total_titans_minted = titan_id;
    config.total_fees_collected += fee;
    
    Ok(())
}

fn encode_location(lat: i32, lng: i32) -> u64 {
    ((lat as u64) << 32) | (lng as u64 & 0xFFFFFFFF)
}
```

### 5.4 Level Up Instruction

```rust
// instructions/level_up.rs
use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::state::TitanData;
use crate::error::TitanError;

/// Experience required for each level
const fn exp_for_level(level: u8) -> u32 {
    // level^2 * 100
    (level as u32) * (level as u32) * 100
}

pub fn level_up(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [
        owner,          // [0] Signer, titan owner
        titan_account,  // [1] Titan PDA
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Load titan
    let mut titan_data = titan_account.try_borrow_mut_data()?;
    let titan = TitanData::from_account_data_mut(&mut titan_data)?;
    
    // Verify ownership (would need additional check via token account)
    
    // Check max level
    if titan.level >= 100 {
        return Err(TitanError::MaxLevelReached.into());
    }
    
    // Check experience requirement
    let required_exp = exp_for_level(titan.level);
    if titan.experience < required_exp {
        return Err(TitanError::InsufficientExperience.into());
    }
    
    // Level up
    titan.experience -= required_exp;
    titan.level += 1;
    
    // Increase link strength slightly
    if titan.link_strength < 100 {
        titan.link_strength = titan.link_strength.saturating_add(1);
    }
    
    Ok(())
}
```

---

## 6. Game Logic Program

### 6.1 Overview

The Game Logic Program handles capture validation, battle records, and reward distribution. It interacts with the Titan NFT Program via CPI.

### 6.2 Instructions

```rust
// instructions/mod.rs
pub const RECORD_CAPTURE: u8 = 0;
pub const RECORD_BATTLE: u8 = 1;
pub const ADD_EXPERIENCE: u8 = 2;
pub const DISTRIBUTE_REWARD: u8 = 3;
```

### 6.3 Record Capture

```rust
/// Capture record instruction data
#[repr(C, packed)]
pub struct RecordCaptureData {
    pub titan_id: u64,
    pub location_lat: i32,
    pub location_lng: i32,
    pub timestamp: i64,
    pub signature: [u8; 64],  // Backend signature
}

pub fn record_capture(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [
        player,           // [0] Signer
        config,           // [1] Config PDA
        backend_signer,   // [2] Backend authority
        player_account,   // [3] Player PDA
        titan_program,    // [4] Titan NFT Program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // Validate backend signature
    if !backend_signer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse and validate capture data
    // ...
    
    // CPI to Titan NFT Program to mint
    // ...
    
    Ok(())
}
```

### 6.4 Record Battle

```rust
/// Battle result data
#[repr(C, packed)]
pub struct RecordBattleData {
    pub titan_id: u64,
    pub opponent_id: u64,
    pub won: bool,
    pub exp_gained: u32,
    pub signature: [u8; 64],
}

pub fn record_battle(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [
        player,
        config,
        backend_signer,
        titan_account,
        player_account,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // Validate and update titan stats
    // ...
    
    Ok(())
}
```

---

## 7. $BREACH Token (SPL Token)

### 7.1 Overview

$BREACH uses **standard SPL Token** - no custom program needed. This provides:

- Automatic compatibility with all Solana DEXs
- Wallet support out of the box
- No additional development or auditing

### 7.2 Token Configuration

```bash
# Create token mint
spl-token create-token --decimals 9

# Create token account
spl-token create-account <MINT_ADDRESS>

# Mint initial supply (1 billion)
spl-token mint <MINT_ADDRESS> 1000000000
```

### 7.3 Token Details

| Property | Value |
|----------|-------|
| Name | BREACH |
| Symbol | $BREACH |
| Decimals | 9 |
| Total Supply | 1,000,000,000 |
| Standard | SPL Token |

### 7.4 Distribution

| Allocation | Percentage | Tokens | Vesting |
|------------|------------|--------|---------|
| Play-to-Earn | 40% | 400M | Released through gameplay |
| Liquidity | 20% | 200M | Raydium/Orca pools |
| Team | 15% | 150M | 2-year vesting |
| Development | 15% | 150M | Milestone-based |
| Marketing | 10% | 100M | Campaign-based |

---

## 8. External Integrations

### 8.1 DEX Integration (Token Trading)

$BREACH can be traded on any Solana DEX:

| DEX | Integration | Notes |
|-----|-------------|-------|
| **Raydium** | Native | Create AMM pool |
| **Orca** | Native | Whirlpool support |
| **Jupiter** | Aggregator | Auto-routing |

**Raydium Pool Setup:**
1. Create pool with SOL/$BREACH pair
2. Add initial liquidity (recommended: $50k+)
3. Token becomes tradeable immediately

### 8.2 NFT Marketplace Integration

Titan NFTs can be traded on existing marketplaces:

| Marketplace | Integration | Notes |
|-------------|-------------|-------|
| **Magic Eden** | Collection listing | Most popular |
| **Tensor** | Collection listing | Pro traders |
| **SolSea** | Collection listing | Alternative |

**Requirements:**
- Standard Metaplex NFT metadata
- Collection verified on marketplace
- Royalties configured (suggested: 5%)

### 8.3 Wallet Integration

All standard Solana wallets supported:
- Phantom
- Solflare
- Backpack
- Glow

---

## 9. Utilities

### 9.1 Error Handling

```rust
// error.rs
use pinocchio::program_error::ProgramError;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum TitanError {
    // Authorization (6000-6099)
    Unauthorized = 6000,
    InvalidCaptureAuthority = 6001,
    NotOwner = 6002,
    
    // Program state (6100-6199)
    ProgramPaused = 6100,
    AlreadyInitialized = 6101,
    
    // Capture (6200-6299)
    CaptureCooldown = 6200,
    MaxTitansReached = 6201,
    InvalidCaptureProof = 6202,
    
    // Titan (6300-6399)
    InvalidThreatClass = 6300,
    InvalidElementType = 6301,
    MaxLevelReached = 6302,
    InsufficientExperience = 6303,
    CannotEvolve = 6304,
    
    // Fusion (6400-6499)
    CannotFuseWithSelf = 6400,
    LevelTooLowForFusion = 6401,
    ElementMismatch = 6402,
    
    // Token (6500-6599)
    InsufficientBalance = 6500,
    TransferFailed = 6501,
}

impl From<TitanError> for ProgramError {
    fn from(e: TitanError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
```

### 9.2 Gene Calculation

```rust
// utils/genes.rs

/// Gene rank classification
pub fn gene_rank(value: u8) -> char {
    match value {
        251..=255 => 'S',
        201..=250 => 'A',
        151..=200 => 'B',
        101..=150 => 'C',
        51..=100 => 'D',
        _ => 'F',
    }
}

/// Calculate offspring genes from two parents
pub fn calculate_offspring_genes(
    parent_a: &[u8; 6],
    parent_b: &[u8; 6],
    randomness: &[u8; 32],
) -> [u8; 6] {
    let mut offspring = [0u8; 6];
    
    for i in 0..6 {
        let roll = randomness[i] % 100;
        
        if roll < 45 {
            // 45% from parent A
            offspring[i] = parent_a[i];
        } else if roll < 90 {
            // 45% from parent B
            offspring[i] = parent_b[i];
        } else {
            // 10% mutation
            let avg = ((parent_a[i] as i16 + parent_b[i] as i16) / 2) as i16;
            let mutation = (randomness[i + 6] as i16 % 64) - 32; // -32 to +31
            offspring[i] = (avg + mutation).clamp(0, 255) as u8;
        }
    }
    
    offspring
}

/// Calculate overall gene quality score (0-1530)
pub fn gene_score(genes: &[u8; 6]) -> u16 {
    genes.iter().map(|g| *g as u16).sum()
}

/// Get gene grade string
pub fn gene_grade(genes: &[u8; 6]) -> &'static str {
    match gene_score(genes) {
        1401..=1530 => "SSS",
        1201..=1400 => "SS",
        1001..=1200 => "S",
        801..=1000 => "A",
        601..=800 => "B",
        401..=600 => "C",
        201..=400 => "D",
        _ => "F",
    }
}
```

---

### 9.3 Battle Math

```rust
// utils/battle.rs

/// Calculate base damage
pub fn calculate_damage(
    attacker_power: u8,
    defender_fortitude: u8,
    skill_power: u8,
    type_multiplier: u8,  // 75, 100, or 150
    randomness: u8,
) -> u16 {
    let attack = attacker_power as u32;
    let defense = (defender_fortitude as u32).max(1);
    let skill = skill_power as u32;
    let type_mult = type_multiplier as u32;
    
    // Random factor: 85-100%
    let random_factor = 85 + (randomness as u32 % 16);
    
    let damage = (attack * skill / defense)
        * type_mult / 100
        * random_factor / 100;
    
    damage.max(1).min(65535) as u16
}

/// Calculate Elo rating change
pub fn calculate_elo_change(winner_rating: u16, loser_rating: u16) -> (u16, u16) {
    const K: i32 = 32;
    
    let expected_winner = 1.0 / (1.0 + 10.0_f64.powf((loser_rating as i32 - winner_rating as i32) as f64 / 400.0));
    let expected_loser = 1.0 - expected_winner;
    
    let delta_winner = (K as f64 * (1.0 - expected_winner)) as i32;
    let delta_loser = (K as f64 * (0.0 - expected_loser)) as i32;
    
    let new_winner = ((winner_rating as i32 + delta_winner) as u16).clamp(100, 3000);
    let new_loser = ((loser_rating as i32 + delta_loser) as u16).clamp(100, 3000);
    
    (new_winner, new_loser)
}
```

---

## 10. Security Considerations

### 10.1 Pinocchio-Specific Security

| Risk | Mitigation |
|------|------------|
| **Buffer Overflows** | Always check data lengths before casting |
| **Integer Overflow** | Use checked arithmetic or explicit wrapping |
| **Account Ownership** | Manually verify program ownership of PDAs |
| **Signer Verification** | Always check `is_signer()` |
| **Rent Exemption** | Ensure accounts have minimum lamports |

### 10.2 Validation Macros

```rust
// utils/validation.rs

/// Require a condition or return error
macro_rules! require {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error.into());
        }
    };
}

/// Require signer
macro_rules! require_signer {
    ($account:expr) => {
        if !$account.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
    };
}

/// Require writable
macro_rules! require_writable {
    ($account:expr) => {
        if !$account.is_writable() {
            return Err(ProgramError::InvalidAccountData);
        }
    };
}

/// Require program owner
macro_rules! require_owned_by {
    ($account:expr, $owner:expr) => {
        if $account.owner() != $owner {
            return Err(ProgramError::IllegalOwner);
        }
    };
}
```

---

## 11. Deployment Guide

### 11.1 Deployment Status

| Network | Program ID | Status |
|---------|------------|--------|
| **Devnet** | `3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7` | ✅ Deployed |
| Mainnet | TBD | 🔜 Planned |

**Explorer**: [View on Solana Explorer](https://explorer.solana.com/address/3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7?cluster=devnet)

### 11.2 Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v2.0.0/install)"

# Configure for devnet
solana config set --url devnet
solana-keygen new
solana airdrop 2
```

### 11.3 Build & Deploy

```bash
# Clone repository
git clone https://github.com/vnxfsc/BREACH.git
cd BREACH/contracts

# Build (Pinocchio uses standard cargo-build-sbf)
cargo build-sbf

# Generate program keypair (first time only)
solana-keygen new -o target/deploy/titan_nft-keypair.json

# Get program ID
solana address -k target/deploy/titan_nft-keypair.json

# Update PROGRAM_ID in lib.rs
# pub const PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("YOUR_PROGRAM_ID");

# Rebuild after updating program ID
cargo build-sbf

# Deploy
solana program deploy target/deploy/titan_nft.so --program-id target/deploy/titan_nft-keypair.json
```

### 11.4 Verification

```bash
# Verify deployment
solana program show 3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7

# Check GlobalConfig PDA
# PDA: seed = "global_config"
solana account <CONFIG_PDA_ADDRESS>
```

---

## 12. Testing Strategy

### 12.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_element_effectiveness() {
        assert_eq!(ElementType::get_multiplier(0, 1), 150); // Abyssal beats Volcanic
        assert_eq!(ElementType::get_multiplier(1, 0), 75);  // Volcanic weak to Abyssal
        assert_eq!(ElementType::get_multiplier(0, 0), 100); // Same type = normal
    }
    
    #[test]
    fn test_gene_calculation() {
        let parent_a = [200, 150, 180, 220, 190, 210];
        let parent_b = [180, 200, 160, 190, 210, 180];
        let randomness = [50u8; 32];
        
        let offspring = calculate_offspring_genes(&parent_a, &parent_b, &randomness);
        
        // All genes should be in valid range
        for gene in offspring.iter() {
            assert!(*gene <= 255);
        }
    }
    
    #[test]
    fn test_combat_power() {
        let titan = TitanData {
            discriminator: TitanData::DISCRIMINATOR,
            titan_id: 1,
            species_id: 1,
            threat_class: 3, // Destroyer
            element_type: 0,
            power: 150,
            fortitude: 120,
            velocity: 130,
            resonance: 140,
            genes: [200, 180, 190, 210, 185, 195],
            level: 50,
            experience: 0,
            link_strength: 75,
            captured_at: 0,
            original_owner: Pubkey::default(),
            capture_location: 0,
            generation: 0,
            parent_a: 0,
            parent_b: 0,
            bump: 0,
        };
        
        let cp = titan.combat_power();
        assert!(cp > 0);
        println!("Combat Power: {}", cp);
    }
}
```

### 12.2 Integration Tests (TypeScript)

Integration tests are implemented in TypeScript using `@solana/web3.js`:

```bash
cd contracts/tests
pnpm install
pnpm test
```

**Test Suite Coverage (14/14 passing):**

| Test | Description | Status |
|------|-------------|--------|
| Initialize | Create GlobalConfig PDA | ✅ Pass |
| Read Config | Verify config data | ✅ Pass |
| Update Config | Modify config settings | ✅ Pass |
| Set Paused (true) | Pause program | ✅ Pass |
| Set Paused (false) | Unpause program | ✅ Pass |
| Mint While Paused | Reject mint when paused | ✅ Pass |
| Mint Titan | Create Titan NFT | ✅ Pass |
| Read Player | Verify player account | ✅ Pass |
| Level Up | Level up Titan | ✅ Pass |
| Evolve | Evolve Titan (Lv30+) | ✅ Pass |
| Fuse | Fuse two Titans | ✅ Pass |

```typescript
// Example: Initialize test
const configPda = PublicKey.findProgramAddressSync(
  [Buffer.from("global_config")],
  PROGRAM_ID
)[0];

const tx = new Transaction().add(
  new TransactionInstruction({
    keys: [
      { pubkey: authority.publicKey, isSigner: true, isWritable: true },
      { pubkey: configPda, isSigner: false, isWritable: true },
      { pubkey: treasury.publicKey, isSigner: false, isWritable: false },
      { pubkey: breachMint.publicKey, isSigner: false, isWritable: false },
      { pubkey: captureAuthority.publicKey, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: Buffer.from([0]), // Instruction ID = 0 (initialize)
  })
);
```

---

## Appendix A: Account Sizes

| Account | Size (bytes) | Rent (SOL) | Notes |
|---------|-------------|------------|-------|
| GlobalConfig | 182 | ~0.00165 | `#[repr(packed)]` for exact size |
| TitanData | 118 | ~0.00157 | `#[repr(packed)]` for exact size |
| PlayerAccount | 152 | ~0.00162 | Player stats and Titan count |
| Listing | 128 | ~0.00159 | NFT marketplace listing |
| Battle | 256 | ~0.00176 | Battle record |

> **Note**: All core accounts use `#[repr(packed)]` attribute to ensure exact size without padding.

---

## Appendix B: Pinocchio vs Anchor Comparison

```rust
// Anchor style (NOT used in BREACH)
#[account]
pub struct MyAccount {
    pub data: u64,
}

#[derive(Accounts)]
pub struct MyInstruction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = 8 + 8)]
    pub my_account: Account<'info, MyAccount>,
    pub system_program: Program<'info, System>,
}

// BREACH style
#[repr(C)]
pub struct MyAccount {
    pub discriminator: [u8; 8],
    pub data: u64,
}

pub fn my_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [payer, my_account, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // Manual validation
    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Manual account creation...
    Ok(())
}
```

---

*Document Version: 2.0*
*Last Updated: January 2026*
*Framework: Pinocchio*