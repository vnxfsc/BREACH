# BREACH Technical Specification

## Technical Specification v1.0

---

## 1. System Architecture Detailed Design

### 1.1 Service Decomposition

```
┌─────────────────────────────────────────────────────────────────┐
│                         Load Balancer                           │
│                      (AWS ALB / Nginx)                          │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│   API Gateway │   │  Game Server  │   │   Realtime    │
│   (REST API)  │   │  (Game Logic) │   │  (WebSocket)  │
│   Port: 8080  │   │   Port: 8081  │   │   Port: 8082  │
└───────────────┘   └───────────────┘   └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│  PostgreSQL   │   │     Redis     │   │    Solana     │
│   (Primary)   │   │    (Cache)    │   │   (Devnet)    │
└───────────────┘   └───────────────┘   └───────────────┘
```

### 1.2 API Gateway Service

**Responsibilities**:
- User authentication/authorization
- Request routing
- Rate limiting/circuit breaking
- Logging

**Main Endpoints**:

```
/api/v1/
├── auth/
│   ├── POST   /wallet-login      # Wallet login
│   ├── POST   /refresh           # Refresh token
│   └── POST   /logout            # Logout
│
├── users/
│   ├── GET    /me                # Get current user
│   ├── PATCH  /me                # Update user info
│   └── GET    /:id               # Get user profile
│
├── titans/
│   ├── GET    /                  # Get user's Titan list
│   ├── GET    /:id               # Get Titan details
│   ├── POST   /mint              # Mint Titan (after capture)
│   ├── POST   /:id/level-up      # Level up
│   ├── POST   /:id/learn-skill   # Learn skill
│   └── PATCH  /:id/equip-skills  # Equip skills
│
├── map/
│   ├── GET    /breaches          # Get nearby breaches
│   ├── POST   /breaches/:id/enter # Enter breach
│   └── POST   /capture/complete  # Complete capture
│
├── battles/
│   ├── POST   /pve/start         # Start PvE battle
│   ├── POST   /pve/complete      # Complete PvE battle
│   ├── POST   /pvp/match         # PvP matchmaking
│   ├── POST   /pvp/action        # Submit PvP action
│   └── GET    /history           # Battle history
│
├── market/
│   ├── GET    /listings          # Get market listings
│   ├── POST   /list              # List Titan for sale
│   ├── DELETE /list/:id          # Delist
│   └── POST   /buy               # Buy
│
└── rankings/
    ├── GET    /pvp               # PvP leaderboard
    └── GET    /titans            # Titan power leaderboard
```

### 1.3 Game Server Service

**Responsibilities**:
- Game logic calculations
- Breach generation algorithm
- Battle resolution
- Attribute calculations

**Core Modules**:

```rust
// Module structure
game_server/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── breach_generator.rs   // Breach generation
│   │   ├── titan_generator.rs    // Titan attribute generation
│   │   ├── battle_engine.rs      // Battle engine
│   │   ├── capture_validator.rs  // Capture validation
│   │   └── reward_calculator.rs  // Reward calculation
│   ├── models/
│   │   ├── mod.rs
│   │   ├── titan.rs
│   │   ├── battle.rs
│   │   └── breach.rs
│   └── utils/
│       ├── mod.rs
│       ├── random.rs
│       └── geo.rs
└── Cargo.toml
```

### 1.4 Realtime Service

**Responsibilities**:
- WebSocket connection management
- Real-time battle sync
- Push notifications
- Online status

**Message Protocol**:

```rust
// WebSocket message format
#[derive(Serialize, Deserialize)]
pub struct WsMessage {
    pub msg_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    // Client -> Server
    PvpAction,          // PvP action
    Heartbeat,          // Heartbeat
    
    // Server -> Client
    MatchFound,         // Match found
    BattleUpdate,       // Battle state update
    OpponentAction,     // Opponent action
    BattleResult,       // Battle result
    BreachSpawned,      // New breach appeared
    Notification,       // Notification
}
```

---

## 2. Database Design

### 2.1 PostgreSQL Schema

```sql
-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

-- ============================================
-- Users Table
-- ============================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wallet_address VARCHAR(44) UNIQUE NOT NULL,
    username VARCHAR(32) UNIQUE,
    avatar_url TEXT,
    
    -- Game data
    level INT DEFAULT 1,
    exp BIGINT DEFAULT 0,
    breach_tokens BIGINT DEFAULT 0,
    
    -- Statistics
    total_captures INT DEFAULT 0,
    pvp_wins INT DEFAULT 0,
    pvp_losses INT DEFAULT 0,
    pvp_rating INT DEFAULT 1000,
    
    -- Staking tier
    staked_amount BIGINT DEFAULT 0,
    stake_tier VARCHAR(16) DEFAULT 'none',
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_users_wallet ON users(wallet_address);
CREATE INDEX idx_users_pvp_rating ON users(pvp_rating DESC);

-- ============================================
-- Titans Table (Off-chain cache, on-chain is source of truth)
-- ============================================
CREATE TABLE titans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- On-chain identifiers
    mint_address VARCHAR(44) UNIQUE NOT NULL,
    owner_wallet VARCHAR(44) NOT NULL,
    
    -- Immutable attributes (synced from chain)
    species_id INT NOT NULL,
    class SMALLINT NOT NULL,
    titan_type SMALLINT NOT NULL,
    genes BYTEA NOT NULL,  -- 6 bytes
    catch_timestamp BIGINT NOT NULL,
    catch_location GEOGRAPHY(POINT, 4326),
    generation SMALLINT DEFAULT 0,
    origin SMALLINT DEFAULT 0,
    
    -- Mutable attributes (synced from chain)
    level SMALLINT DEFAULT 1,
    exp INT DEFAULT 0,
    learned_skills BIGINT DEFAULT 0,
    equipped_skills SMALLINT[] DEFAULT '{}',
    kills INT DEFAULT 0,
    wins SMALLINT DEFAULT 0,
    losses SMALLINT DEFAULT 0,
    
    -- Calculated attributes (cached)
    current_atk INT,
    current_def INT,
    current_spd INT,
    current_hp INT,
    power_score INT,
    
    -- Off-chain personalization
    nickname VARCHAR(32),
    is_favorite BOOLEAN DEFAULT false,
    
    -- Sync status
    last_synced_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_titans_owner ON titans(owner_wallet);
CREATE INDEX idx_titans_mint ON titans(mint_address);
CREATE INDEX idx_titans_species ON titans(species_id);
CREATE INDEX idx_titans_class ON titans(class);
CREATE INDEX idx_titans_power ON titans(power_score DESC);

-- ============================================
-- Titan Species Table
-- ============================================
CREATE TABLE titan_species (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    description TEXT,
    titan_type SMALLINT NOT NULL,
    
    -- Base attributes
    base_atk INT NOT NULL,
    base_def INT NOT NULL,
    base_spd INT NOT NULL,
    base_hp INT NOT NULL,
    
    -- Growth rates
    growth_atk DECIMAL(4,3) DEFAULT 1.0,
    growth_def DECIMAL(4,3) DEFAULT 1.0,
    growth_spd DECIMAL(4,3) DEFAULT 1.0,
    growth_hp DECIMAL(4,3) DEFAULT 1.0,
    
    -- Learnable skills
    learnable_skills INT[] DEFAULT '{}',
    
    -- Rarity weights (spawn probability per Class)
    rarity_weights JSONB,
    
    -- Art assets
    model_url TEXT,
    icon_url TEXT,
    
    -- Status
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================
-- Skills Table
-- ============================================
CREATE TABLE skills (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    description TEXT,
    
    -- Skill type
    skill_type VARCHAR(16) NOT NULL,  -- attack, defense, control, support
    element SMALLINT,  -- corresponds to titan_type
    
    -- Effect values
    power INT DEFAULT 0,
    accuracy INT DEFAULT 100,
    cooldown SMALLINT DEFAULT 0,
    
    -- Additional effects
    effects JSONB,
    
    -- Learning requirements
    required_level SMALLINT DEFAULT 1,
    required_species INT[],  -- species restriction
    
    -- Art assets
    icon_url TEXT,
    animation_id VARCHAR(32),
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================
-- Breaches Table (Dynamically generated)
-- ============================================
CREATE TABLE breaches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Location
    location GEOGRAPHY(POINT, 4326) NOT NULL,
    geohash VARCHAR(12) NOT NULL,
    
    -- Breach attributes
    class SMALLINT NOT NULL,
    titan_type SMALLINT NOT NULL,
    species_id INT NOT NULL,
    
    -- Time window
    spawned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    
    -- Status
    is_captured BOOLEAN DEFAULT false,
    captured_by UUID REFERENCES users(id),
    captured_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_breaches_location ON breaches USING GIST(location);
CREATE INDEX idx_breaches_geohash ON breaches(geohash);
CREATE INDEX idx_breaches_expires ON breaches(expires_at);
CREATE INDEX idx_breaches_active ON breaches(is_captured, expires_at);

-- ============================================
-- Battle Records Table
-- ============================================
CREATE TABLE battle_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Battle type
    battle_type VARCHAR(16) NOT NULL,  -- pve, pvp, boss
    
    -- Participants
    player1_id UUID REFERENCES users(id),
    player2_id UUID REFERENCES users(id),  -- NULL for PvE
    
    -- Teams
    team1_titans UUID[] NOT NULL,
    team2_titans UUID[],
    
    -- Result
    winner_id UUID REFERENCES users(id),
    battle_data JSONB,  -- Detailed battle process
    
    -- Rewards
    exp_gained INT DEFAULT 0,
    tokens_gained BIGINT DEFAULT 0,
    
    -- On-chain record
    tx_signature VARCHAR(88),
    
    -- Time
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ended_at TIMESTAMP WITH TIME ZONE,
    duration_seconds INT
);

CREATE INDEX idx_battles_player1 ON battle_records(player1_id);
CREATE INDEX idx_battles_player2 ON battle_records(player2_id);
CREATE INDEX idx_battles_type ON battle_records(battle_type);
CREATE INDEX idx_battles_time ON battle_records(started_at DESC);

-- ============================================
-- Market Listings Table
-- ============================================
CREATE TABLE market_listings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Titan
    titan_id UUID REFERENCES titans(id),
    mint_address VARCHAR(44) NOT NULL,
    
    -- Seller
    seller_id UUID REFERENCES users(id),
    seller_wallet VARCHAR(44) NOT NULL,
    
    -- Price
    price BIGINT NOT NULL,  -- Unit: lamports of $BREACH
    
    -- Status
    status VARCHAR(16) DEFAULT 'active',  -- active, sold, cancelled
    
    -- Buyer (after sale)
    buyer_id UUID REFERENCES users(id),
    buyer_wallet VARCHAR(44),
    
    -- On-chain transactions
    list_tx VARCHAR(88),
    sale_tx VARCHAR(88),
    
    -- Time
    listed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    sold_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_listings_status ON market_listings(status);
CREATE INDEX idx_listings_seller ON market_listings(seller_id);
CREATE INDEX idx_listings_price ON market_listings(price) WHERE status = 'active';

-- ============================================
-- Quests Table
-- ============================================
CREATE TABLE quests (
    id SERIAL PRIMARY KEY,
    
    -- Quest info
    name VARCHAR(64) NOT NULL,
    description TEXT,
    quest_type VARCHAR(16) NOT NULL,  -- daily, weekly, achievement
    
    -- Objective
    target_type VARCHAR(32) NOT NULL,
    target_count INT NOT NULL,
    target_params JSONB,
    
    -- Rewards
    reward_exp INT DEFAULT 0,
    reward_tokens BIGINT DEFAULT 0,
    reward_items JSONB,
    
    -- Requirements
    required_level SMALLINT DEFAULT 1,
    
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ============================================
-- User Quest Progress Table
-- ============================================
CREATE TABLE user_quests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    quest_id INT REFERENCES quests(id),
    
    -- Progress
    current_count INT DEFAULT 0,
    is_completed BOOLEAN DEFAULT false,
    is_claimed BOOLEAN DEFAULT false,
    
    -- Time
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    claimed_at TIMESTAMP WITH TIME ZONE,
    
    -- Daily/weekly quest reset
    reset_at TIMESTAMP WITH TIME ZONE,
    
    UNIQUE(user_id, quest_id, reset_at)
);

CREATE INDEX idx_user_quests_user ON user_quests(user_id);
CREATE INDEX idx_user_quests_active ON user_quests(user_id, is_completed, is_claimed);
```

### 2.2 Redis Data Structures

```
# User session
session:{wallet_address} -> JSON{user_id, token, expires_at}
TTL: 24h

# User online status
online:users -> SORTED_SET {user_id: last_heartbeat_timestamp}

# PvP matchmaking queue
pvp:queue:{rating_bracket} -> LIST [user_id, ...]

# PvP battle room
pvp:room:{room_id} -> HASH {
    player1: user_id,
    player2: user_id,
    state: "selecting" | "resolving" | "finished",
    round: 1,
    p1_action: JSON,
    p2_action: JSON,
    created_at: timestamp
}
TTL: 30m

# Breach cache (by geohash)
breaches:{geohash} -> SET [breach_id, ...]
TTL: based on nearest expiry time

# User Titans cache
user:titans:{user_id} -> JSON [titan_summary, ...]
TTL: 5m

# Leaderboards
rankings:pvp -> SORTED_SET {user_id: rating}
rankings:power -> SORTED_SET {titan_id: power_score}

# Rate limiting
ratelimit:{user_id}:{endpoint} -> INT count
TTL: window time
```

---

## 3. Solana Smart Contract Design

### 3.1 Contract Architecture

```
contracts/programs/
├── titan/           # Titan NFT contract
│   ├── src/
│   │   ├── lib.rs
│   │   ├── state.rs        # Account state definitions
│   │   ├── instructions/   # Instruction handlers
│   │   │   ├── mod.rs
│   │   │   ├── mint.rs
│   │   │   ├── level_up.rs
│   │   │   ├── learn_skill.rs
│   │   │   ├── equip_skills.rs
│   │   │   └── record_battle.rs
│   │   ├── errors.rs       # Error definitions
│   │   └── utils.rs        # Utility functions
│   └── Cargo.toml
│
└── game_logic/      # Game Logic Program
    ├── src/
    │   ├── lib.rs
    │   ├── instructions/
    │   │   ├── record_capture.rs
    │   │   ├── record_battle.rs
    │   │   └── distribute_reward.rs
    │   └── errors.rs
    └── Cargo.toml

# External Infrastructure (No custom development needed)
# $BREACH Token    → Standard SPL Token
# Token Trading    → Raydium / Orca / Jupiter
# NFT Trading      → Magic Eden / Tensor
```

### 3.2 Titan Program Detailed Design

```rust
// ============================================
// state.rs - Account State Definitions
// ============================================

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Titan NFT core data account
/// Using #[repr(C)] for zero-copy deserialization
#[repr(C)]
pub struct Titan {
    /// Account discriminator (8 bytes)
    pub discriminator: [u8; 8],
    // ═══════════ Immutable Core Data (60 bytes) ═══════════
    
    /// NFT Mint address
    pub mint: Pubkey,                    // 32 bytes
    
    /// Species ID (0-65535)
    pub species_id: u16,                 // 2 bytes
    
    /// Threat Class (1-5)
    pub class: u8,                       // 1 byte
    
    /// Type (0-5: Abyssal, Volcanic, Storm, Void, Parasitic, Ossified)
    pub titan_type: u8,                  // 1 byte
    
    /// Gene sequence [ATK, SPD, DEF, GRW, SKL, MUT]
    pub genes: [u8; 6],                  // 6 bytes
    
    /// Capture timestamp
    pub catch_timestamp: i64,            // 8 bytes
    
    /// Capture location (latitude * 10^6)
    pub catch_lat: i32,                  // 4 bytes
    
    /// Capture location (longitude * 10^6)
    pub catch_lng: i32,                  // 4 bytes
    
    /// Generation (0=wild, 1+=fusion)
    pub generation: u8,                  // 1 byte
    
    /// Origin (0=Wild, 1=Hatch, 2=Fusion, 3=Event)
    pub origin: u8,                      // 1 byte
    
    // ═══════════ Mutable Growth Data (24 bytes) ═══════════
    
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
    
    // ═══════════ Metadata (24 bytes) ═══════════
    
    /// Last battle timestamp
    pub last_battle: i64,                // 8 bytes
    
    /// Reserved for expansion
    pub reserved: [u8; 15],              // 15 bytes
    
    /// Account bump
    pub bump: u8,                        // 1 byte
}

impl Titan {
    pub const SIZE: usize = 8 + 32 + 2 + 1 + 1 + 6 + 8 + 4 + 4 + 1 + 1 
                          + 1 + 4 + 8 + 4 + 4 + 2 + 2 + 8 + 15 + 1;
    
    /// Calculate experience needed for next level
    pub fn exp_to_next_level(&self) -> u32 {
        // Experience curve: level^2 * 100
        ((self.level as u32 + 1).pow(2)) * 100
    }
    
    /// Check if can level up
    pub fn can_level_up(&self) -> bool {
        self.level < 100 && self.exp >= self.exp_to_next_level()
    }
}

/// Global config account
#[repr(C)]
pub struct GlobalConfig {
    pub discriminator: [u8; 8],
    pub authority: Pubkey,
    pub backend_signer: Pubkey,
    pub treasury: Pubkey,
    pub total_minted: u64,
    pub is_paused: bool,
    pub bump: u8,
}

impl GlobalConfig {
    pub const SIZE: usize = 8 + 32 + 32 + 32 + 8 + 1 + 1;
    pub const DISCRIMINATOR: [u8; 8] = [0x43, 0x4F, 0x4E, 0x46, 0x49, 0x47, 0x00, 0x00];
    
    /// Zero-copy read from account data
    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        let config = unsafe { &*(data.as_ptr() as *const GlobalConfig) };
        if config.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(config)
    }
}

// ============================================
// instructions/mint.rs - Mint Titan
// ============================================

use pinocchio::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
};
use pinocchio_token::instructions::MintTo;
use pinocchio_system::instructions::CreateAccount;

/// Mint instruction data layout
#[repr(C, packed)]
pub struct MintTitanData {
    pub species_id: u16,
    pub class: u8,
    pub titan_type: u8,
    pub genes: [u8; 6],
    pub catch_lat: i32,
    pub catch_lng: i32,
}

pub fn mint_titan(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts using Pinocchio slice pattern
    let [
        user,                    // [0] Signer, payer
        config,                  // [1] Config PDA
        backend_signer,          // [2] Backend signer
        mint,                    // [3] NFT mint account
        titan_data,              // [4] Titan PDA
        user_token_account,      // [5] User's token account
        system_program,          // [6] System program
        token_program,           // [7] Token program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    // Validate signers
    if !user.is_signer() || !backend_signer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Load config and validate backend signer
    let config_data = config.try_borrow_data()?;
    let config_account = GlobalConfig::from_account_data(&config_data)?;
    
    if *backend_signer.key() != config_account.backend_signer {
        return Err(TitanError::InvalidBackendSigner.into());
    }
    
    // Parse instruction data
    if data.len() < std::mem::size_of::<MintTitanData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mint_data = unsafe { &*(data.as_ptr() as *const MintTitanData) };
    
    // Validate parameters
    if mint_data.class < 1 || mint_data.class > 5 {
        return Err(TitanError::InvalidClass.into());
    }
    if mint_data.titan_type > 5 {
        return Err(TitanError::InvalidType.into());
    }
    
    // Derive Titan PDA
    let (titan_pda, titan_bump) = Pubkey::find_program_address(
        &[b"titan", mint.key().as_ref()],
        program_id,
    );
    
    if *titan_data.key() != titan_pda {
        return Err(ProgramError::InvalidSeeds);
    }
    
    let clock = Clock::get()?;
    
    // Initialize Titan data using zero-copy write
    let mut titan_account_data = titan_data.try_borrow_mut_data()?;
    let titan = unsafe { &mut *(titan_account_data.as_mut_ptr() as *mut Titan) };
    
    titan.discriminator = Titan::DISCRIMINATOR;
    titan.mint = *mint.key();
    titan.species_id = mint_data.species_id;
    titan.class = mint_data.class;
    titan.titan_type = mint_data.titan_type;
    titan.genes = mint_data.genes;
    titan.catch_timestamp = clock.unix_timestamp;
    titan.catch_lat = mint_data.catch_lat;
    titan.catch_lng = mint_data.catch_lng;
    titan.generation = 0;
    titan.origin = 0; // Wild
    titan.level = 1;
    titan.exp = 0;
    titan.learned_skills = 0;
    titan.equipped_skills = [0; 4];
    titan.kills = 0;
    titan.wins = 0;
    titan.losses = 0;
    titan.last_battle = 0;
    titan.bump = titan_bump;
    
    // Mint NFT token via CPI
    MintTo {
        mint,
        token_account: user_token_account,
        mint_authority: titan_data,
        amount: 1,
    }.invoke_signed(&[&[b"titan", mint.key().as_ref(), &[titan_bump]]])?;
    
    Ok(())
}

// ============================================
// instructions/level_up.rs - Level Up
// ============================================

pub fn level_up(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let [
        user,               // [0] Signer, owner
        titan_account,      // [1] Titan PDA
        mint,               // [2] NFT mint
        user_token_account, // [3] User's token account (verify ownership)
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    if !user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Load and modify titan data
    let mut titan_data = titan_account.try_borrow_mut_data()?;
    let titan = Titan::from_account_data_mut(&mut titan_data)?;
    
    if !titan.can_level_up() {
        return Err(TitanError::CannotLevelUp.into());
    }
    
    let exp_needed = titan.exp_to_next_level();
    titan.exp -= exp_needed;
    titan.level += 1;
    
    Ok(())
}

// ============================================
// instructions/record_battle.rs - Record Battle
// ============================================

/// Record battle instruction data
#[repr(C, packed)]
pub struct RecordBattleData {
    pub won: bool,
    pub kills: u32,
    pub exp_gained: u32,
}

pub fn record_battle(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [
        user,            // [0] Signer
        config,          // [1] Config PDA
        backend_signer,  // [2] Backend signer
        titan_account,   // [3] Titan PDA
        mint,            // [4] NFT mint
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    if !user.is_signer() || !backend_signer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate backend signer
    let config_data = config.try_borrow_data()?;
    let config_account = GlobalConfig::from_account_data(&config_data)?;
    
    if *backend_signer.key() != config_account.backend_signer {
        return Err(TitanError::InvalidBackendSigner.into());
    }
    
    // Parse instruction data
    if data.len() < std::mem::size_of::<RecordBattleData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let battle_data = unsafe { &*(data.as_ptr() as *const RecordBattleData) };
    
    let clock = Clock::get()?;
    
    // Update titan stats
    let mut titan_data = titan_account.try_borrow_mut_data()?;
    let titan = Titan::from_account_data_mut(&mut titan_data)?;
    
    titan.exp = titan.exp.saturating_add(battle_data.exp_gained);
    titan.kills = titan.kills.saturating_add(battle_data.kills);
    titan.last_battle = clock.unix_timestamp;
    
    if battle_data.won {
        titan.wins = titan.wins.saturating_add(1);
    } else {
        titan.losses = titan.losses.saturating_add(1);
    }
    
    Ok(())
}

// ============================================
// errors.rs - Error Definitions
// ============================================

use pinocchio::program_error::ProgramError;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum TitanError {
    InvalidBackendSigner = 6000,
    InvalidClass = 6001,
    InvalidType = 6002,
    NotOwner = 6003,
    CannotLevelUp = 6004,
    MaxLevelReached = 6005,
    SkillAlreadyLearned = 6006,
    SkillNotLearned = 6007,
    InvalidSkillSlot = 6008,
}

impl From<TitanError> for ProgramError {
    fn from(e: TitanError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ============================================
// Events (Pinocchio logs instead of Anchor events)
// ============================================

// In Pinocchio, we use msg!() macro or sol_log for events
// Example logging:
// pinocchio::msg!("TitanMinted: mint={}, owner={}", mint, owner);
```

### 3.3 $BREACH Token Program

```rust
// token/src/lib.rs

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_token::instructions::{Transfer, Burn};

pinocchio::declare_id!("BREACHxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    
    match discriminator {
        0 => initialize(program_id, accounts, data),
        1 => distribute_reward(program_id, accounts, data),
        2 => consume(program_id, accounts, data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

/// Initialize token config
pub fn initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let [
        authority,      // [0] Signer
        config_account, // [1] Config PDA
        mint,           // [2] Token mint
        treasury,       // [3] Treasury account
        system_program, // [4] System program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Initialize config data...
    Ok(())
}

/// Distribute game rewards (from treasury)
pub fn distribute_reward(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [
        backend_signer,     // [0] Backend signer
        config,             // [1] Config PDA
        treasury,           // [2] Treasury PDA
        treasury_account,   // [3] Treasury token account
        user_account,       // [4] User token account
        token_program,      // [5] Token program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    if !backend_signer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse amount from data
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    
    // Transfer from treasury using signed CPI
    let treasury_bump = data[8]; // Bump passed in data
    Transfer {
        from: treasury_account,
        to: user_account,
        authority: treasury,
        amount,
    }.invoke_signed(&[&[b"treasury", &[treasury_bump]]])?;
    
    Ok(())
}

/// Consume tokens (capture/level up etc.)
pub fn consume(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [
        user,              // [0] Signer
        user_account,      // [1] User token account
        treasury_account,  // [2] Treasury token account
        mint,              // [3] Token mint
        token_program,     // [4] Token program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    if !user.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Parse data: amount (u64) + burn_rate (u8)
    let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let burn_rate = data[8];
    
    let burn_amount = amount * burn_rate as u64 / 100;
    let treasury_amount = amount - burn_amount;
    
    // Burn portion
    if burn_amount > 0 {
        Burn {
            mint,
            token_account: user_account,
            authority: user,
            amount: burn_amount,
        }.invoke()?;
    }
    
    // Treasury portion
    if treasury_amount > 0 {
        Transfer {
            from: user_account,
            to: treasury_account,
            authority: user,
            amount: treasury_amount,
        }.invoke()?;
        }
        
        emit!(TokenConsumed {
            user: ctx.accounts.user.key(),
            amount,
            burned: burn_amount,
            to_treasury: treasury_amount,
        });
        
        Ok(())
    }
}
```

---

## 4. Core Game Logic Algorithms

### 4.1 Breach Generation Algorithm

```rust
// game-server/src/services/breach_generator.rs

use geo::{Point, HaversineDistance};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct BreachGenerator {
    config: BreachConfig,
}

#[derive(Clone)]
pub struct BreachConfig {
    pub base_spawn_rate: f64,        // Base spawn rate
    pub class_weights: [f64; 5],     // Weight per class
    pub type_weights: [f64; 6],      // Weight per type
    pub min_distance_meters: f64,    // Minimum breach spacing
    pub duration_minutes: [u32; 5],  // Duration per class
}

impl BreachGenerator {
    /// Generate breaches for specified area
    pub fn generate_breaches(
        &self,
        center: Point<f64>,
        radius_km: f64,
        current_time: i64,
    ) -> Vec<Breach> {
        let mut breaches = Vec::new();
        let mut rng = self.create_seeded_rng(center, current_time);
        
        // Calculate spawn count
        let area = std::f64::consts::PI * radius_km * radius_km;
        let spawn_count = (area * self.config.base_spawn_rate) as usize;
        
        for _ in 0..spawn_count {
            if let Some(breach) = self.generate_single_breach(&mut rng, center, radius_km, current_time) {
                // Check distance from existing breaches
                let too_close = breaches.iter().any(|b: &Breach| {
                    let p1 = Point::new(breach.lng, breach.lat);
                    let p2 = Point::new(b.lng, b.lat);
                    p1.haversine_distance(&p2) < self.config.min_distance_meters
                });
                
                if !too_close {
                    breaches.push(breach);
                }
            }
        }
        
        breaches
    }
    
    fn generate_single_breach(
        &self,
        rng: &mut ChaCha8Rng,
        center: Point<f64>,
        radius_km: f64,
        current_time: i64,
    ) -> Option<Breach> {
        // Random location
        let angle = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
        let distance = rng.gen::<f64>().sqrt() * radius_km;
        
        let lat = center.y() + (distance / 111.0) * angle.cos();
        let lng = center.x() + (distance / (111.0 * center.y().to_radians().cos())) * angle.sin();
        
        // Determine class
        let class = self.weighted_random(rng, &self.config.class_weights) + 1;
        
        // Determine type
        let titan_type = self.weighted_random(rng, &self.config.type_weights);
        
        // Determine species (filtered by type)
        let species_id = self.random_species(rng, class as u8, titan_type as u8);
        
        // Duration
        let duration = self.config.duration_minutes[class - 1] as i64 * 60;
        
        Some(Breach {
            id: uuid::Uuid::new_v4(),
            lat,
            lng,
            class: class as u8,
            titan_type: titan_type as u8,
            species_id,
            spawned_at: current_time,
            expires_at: current_time + duration,
        })
    }
    
    fn create_seeded_rng(&self, center: Point<f64>, time: i64) -> ChaCha8Rng {
        // Create deterministic seed based on location and time
        // Same area at same time bucket should produce same breaches
        let time_bucket = time / 300; // 5-minute time bucket
        let lat_bucket = (center.y() * 100.0) as i64;
        let lng_bucket = (center.x() * 100.0) as i64;
        
        let seed = (time_bucket ^ lat_bucket ^ lng_bucket) as u64;
        ChaCha8Rng::seed_from_u64(seed)
    }
    
    fn weighted_random(&self, rng: &mut ChaCha8Rng, weights: &[f64]) -> usize {
        let total: f64 = weights.iter().sum();
        let mut random = rng.gen::<f64>() * total;
        
        for (i, &weight) in weights.iter().enumerate() {
            random -= weight;
            if random <= 0.0 {
                return i;
            }
        }
        
        weights.len() - 1
    }
}
```

### 4.2 Attribute Calculation

```rust
// packages/game-logic/src/stats.rs

/// Species base attributes
#[derive(Clone)]
pub struct SpeciesBase {
    pub atk: u32,
    pub def: u32,
    pub spd: u32,
    pub hp: u32,
    pub growth_atk: f64,
    pub growth_def: f64,
    pub growth_spd: f64,
    pub growth_hp: f64,
}

/// Class multipliers
pub const CLASS_MULTIPLIERS: [f64; 5] = [1.0, 1.5, 2.0, 3.0, 5.0];

/// Gene value to modifier coefficient
pub fn gene_modifier(gene_value: u8) -> f64 {
    0.5 + (gene_value as f64 / 255.0)
}

/// Gene grade
pub fn gene_grade(gene_value: u8) -> char {
    match gene_value {
        250..=255 => 'S',
        230..=249 => 'A',
        190..=229 => 'B',
        140..=189 => 'C',
        80..=139 => 'D',
        30..=79 => 'E',
        _ => 'F',
    }
}

/// Calculate complete Titan stats
pub fn calculate_stats(
    species: &SpeciesBase,
    class: u8,
    genes: &[u8; 6],
    level: u8,
) -> TitanStats {
    let class_mult = CLASS_MULTIPLIERS[(class - 1) as usize];
    
    let atk_gene_mod = gene_modifier(genes[0]);
    let spd_gene_mod = gene_modifier(genes[1]);
    let def_gene_mod = gene_modifier(genes[2]);
    let growth_gene_mod = gene_modifier(genes[3]);
    
    let level_mult = |base_growth: f64| -> f64 {
        1.0 + (level as f64 - 1.0) * base_growth * growth_gene_mod * 0.02
    };
    
    TitanStats {
        atk: (species.atk as f64 * class_mult * atk_gene_mod * level_mult(species.growth_atk)) as u32,
        def: (species.def as f64 * class_mult * def_gene_mod * level_mult(species.growth_def)) as u32,
        spd: (species.spd as f64 * class_mult * spd_gene_mod * level_mult(species.growth_spd)) as u32,
        hp: (species.hp as f64 * class_mult * gene_modifier(genes[3]) * level_mult(species.growth_hp)) as u32,
    }
}

/// Calculate power score
pub fn calculate_power_score(stats: &TitanStats, class: u8, genes: &[u8; 6]) -> u32 {
    let base_power = stats.atk + stats.def + stats.spd + stats.hp / 10;
    
    // Gene grade bonus
    let gene_bonus: u32 = genes.iter().map(|&g| {
        match gene_grade(g) {
            'S' => 100,
            'A' => 50,
            'B' => 25,
            'C' => 10,
            'D' => 5,
            _ => 0,
        }
    }).sum();
    
    // Class bonus
    let class_bonus = (class as u32 - 1) * 500;
    
    base_power + gene_bonus + class_bonus
}
```

### 4.3 Battle Engine

```rust
// game-server/src/services/battle_engine.rs

use std::collections::VecDeque;

pub struct BattleEngine;

#[derive(Clone)]
pub struct BattleTitan {
    pub id: uuid::Uuid,
    pub stats: TitanStats,
    pub titan_type: u8,
    pub skills: Vec<Skill>,
    pub current_hp: i32,
    pub buffs: Vec<Buff>,
    pub cooldowns: [u8; 4],
}

#[derive(Clone)]
pub enum BattleAction {
    UseSkill { titan_idx: usize, skill_idx: usize, target_idx: usize },
    Switch { from_idx: usize, to_idx: usize },
    Pass,
}

#[derive(Clone)]
pub struct BattleResult {
    pub winner: Option<usize>,  // 0 = player1, 1 = player2
    pub rounds: Vec<RoundResult>,
    pub player1_exp: u32,
    pub player2_exp: u32,
}

impl BattleEngine {
    /// PvP turn-based battle round resolution
    pub fn resolve_pvp_round(
        &self,
        team1: &mut [BattleTitan; 3],
        team2: &mut [BattleTitan; 3],
        action1: BattleAction,
        action2: BattleAction,
        active1: usize,
        active2: usize,
    ) -> RoundResult {
        let mut events = Vec::new();
        
        // Determine action order (higher speed goes first)
        let t1_speed = team1[active1].stats.spd;
        let t2_speed = team2[active2].stats.spd;
        
        let (first_action, second_action, first_team, second_team, first_active, second_active) = 
            if t1_speed >= t2_speed {
                (action1, action2, team1, team2, active1, active2)
            } else {
                (action2, action1, team2, team1, active2, active1)
            };
        
        // Execute first action
        events.extend(self.execute_action(
            first_team, second_team, first_active, second_active, &first_action
        ));
        
        // Check if any Titan was knocked out
        if second_team[second_active].current_hp > 0 {
            // Execute second action
            events.extend(self.execute_action(
                second_team, first_team, second_active, first_active, &second_action
            ));
        }
        
        // Process end of round effects (poison, burn, etc.)
        events.extend(self.process_end_of_round(team1, team2));
        
        // Reduce cooldowns
        for titan in team1.iter_mut().chain(team2.iter_mut()) {
            for cd in titan.cooldowns.iter_mut() {
                *cd = cd.saturating_sub(1);
            }
        }
        
        RoundResult { events }
    }
    
    fn execute_action(
        &self,
        attacker_team: &mut [BattleTitan; 3],
        defender_team: &mut [BattleTitan; 3],
        attacker_idx: usize,
        defender_idx: usize,
        action: &BattleAction,
    ) -> Vec<BattleEvent> {
        let mut events = Vec::new();
        
        match action {
            BattleAction::UseSkill { skill_idx, target_idx, .. } => {
                let attacker = &attacker_team[attacker_idx];
                let skill = &attacker.skills[*skill_idx];
                
                // Check cooldown
                if attacker.cooldowns[*skill_idx] > 0 {
                    events.push(BattleEvent::SkillOnCooldown);
                    return events;
                }
                
                // Calculate damage
                let base_damage = self.calculate_damage(
                    &attacker.stats,
                    &defender_team[*target_idx].stats,
                    skill,
                    attacker.titan_type,
                    defender_team[*target_idx].titan_type,
                );
                
                // Apply damage
                defender_team[*target_idx].current_hp -= base_damage as i32;
                
                events.push(BattleEvent::Damage {
                    attacker: attacker_idx,
                    target: *target_idx,
                    damage: base_damage,
                    skill_name: skill.name.clone(),
                });
                
                // Set cooldown
                attacker_team[attacker_idx].cooldowns[*skill_idx] = skill.cooldown;
                
                // Check knockout
                if defender_team[*target_idx].current_hp <= 0 {
                    events.push(BattleEvent::Knockout { target: *target_idx });
                }
            }
            
            BattleAction::Switch { to_idx, .. } => {
                events.push(BattleEvent::Switch {
                    from: attacker_idx,
                    to: *to_idx,
                });
            }
            
            BattleAction::Pass => {
                events.push(BattleEvent::Pass);
            }
        }
        
        events
    }
    
    fn calculate_damage(
        &self,
        attacker: &TitanStats,
        defender: &TitanStats,
        skill: &Skill,
        attacker_type: u8,
        defender_type: u8,
    ) -> u32 {
        // Base damage = (Attack * Skill Power - Defense * 0.5) * Random variance
        let base = (attacker.atk as f64 * skill.power as f64 / 100.0) 
                 - (defender.def as f64 * 0.5);
        
        let base = base.max(1.0);
        
        // Type effectiveness
        let type_mult = self.get_type_effectiveness(attacker_type, defender_type);
        
        // Random variance (0.85 - 1.0)
        let random_mult = 0.85 + rand::random::<f64>() * 0.15;
        
        (base * type_mult * random_mult) as u32
    }
    
    fn get_type_effectiveness(&self, attacker: u8, defender: u8) -> f64 {
        // Effectiveness: Abyssal > Volcanic > Storm > Void > Parasitic > Ossified > Abyssal
        const EFFECTIVENESS: [[f64; 6]; 6] = [
            // Abyssal  Volcanic  Storm  Void  Parasitic  Ossified
            [1.0,      1.5,      1.0,   1.0,  1.0,       0.67],  // Abyssal
            [0.67,     1.0,      1.5,   1.0,  1.0,       1.0],   // Volcanic
            [1.0,      0.67,     1.0,   1.5,  1.0,       1.0],   // Storm
            [1.0,      1.0,      0.67,  1.0,  1.5,       1.0],   // Void
            [1.0,      1.0,      1.0,   0.67, 1.0,       1.5],   // Parasitic
            [1.5,      1.0,      1.0,   1.0,  0.67,      1.0],   // Ossified
        ];
        
        EFFECTIVENESS[attacker as usize][defender as usize]
    }
}
```

---

## 5. API Authentication & Security

### 5.1 Wallet Authentication Flow

```rust
// backend/api/src/auth/wallet_auth.rs

use ed25519_dalek::{PublicKey, Signature, Verifier};
use base58::FromBase58;

pub struct WalletAuthService {
    jwt_secret: String,
}

#[derive(Deserialize)]
pub struct WalletLoginRequest {
    pub wallet_address: String,
    pub message: String,
    pub signature: String,
}

impl WalletAuthService {
    /// Verify wallet signature and generate JWT
    pub async fn wallet_login(
        &self,
        req: WalletLoginRequest,
        db: &PgPool,
    ) -> Result<AuthResponse, AuthError> {
        // 1. Validate message format
        let expected_message = format!(
            "Sign this message to login to BREACH.\nNonce: {}",
            &req.message
        );
        
        // 2. Verify signature
        let pubkey_bytes = req.wallet_address.from_base58()
            .map_err(|_| AuthError::InvalidWallet)?;
        let pubkey = PublicKey::from_bytes(&pubkey_bytes)
            .map_err(|_| AuthError::InvalidWallet)?;
        
        let sig_bytes = req.signature.from_base58()
            .map_err(|_| AuthError::InvalidSignature)?;
        let signature = Signature::from_bytes(&sig_bytes)
            .map_err(|_| AuthError::InvalidSignature)?;
        
        pubkey.verify(expected_message.as_bytes(), &signature)
            .map_err(|_| AuthError::SignatureVerificationFailed)?;
        
        // 3. Get or create user
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (wallet_address) 
            VALUES ($1)
            ON CONFLICT (wallet_address) 
            DO UPDATE SET last_login_at = NOW()
            RETURNING *
            "#,
            req.wallet_address
        )
        .fetch_one(db)
        .await?;
        
        // 4. Generate JWT
        let token = self.generate_jwt(&user)?;
        
        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }
    
    fn generate_jwt(&self, user: &User) -> Result<String, AuthError> {
        let claims = Claims {
            sub: user.id.to_string(),
            wallet: user.wallet_address.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };
        
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| AuthError::TokenGenerationFailed)
    }
}
```

### 5.2 Backend Signature Verification (On-chain Interaction)

```rust
// backend/api/src/solana/signer.rs

use ed25519_dalek::{Keypair, Signer};

pub struct BackendSigner {
    keypair: Keypair,
}

impl BackendSigner {
    /// Generate signature for minting Titan
    pub fn sign_mint_authorization(
        &self,
        user_wallet: &Pubkey,
        species_id: u16,
        class: u8,
        titan_type: u8,
        genes: &[u8; 6],
        catch_lat: i32,
        catch_lng: i32,
        timestamp: i64,
    ) -> [u8; 64] {
        let message = [
            user_wallet.to_bytes().as_slice(),
            &species_id.to_le_bytes(),
            &[class],
            &[titan_type],
            genes.as_slice(),
            &catch_lat.to_le_bytes(),
            &catch_lng.to_le_bytes(),
            &timestamp.to_le_bytes(),
        ].concat();
        
        self.keypair.sign(&message).to_bytes()
    }
    
    /// Generate signature for battle result
    pub fn sign_battle_result(
        &self,
        titan_mint: &Pubkey,
        won: bool,
        kills: u32,
        exp_gained: u32,
        timestamp: i64,
    ) -> [u8; 64] {
        let message = [
            titan_mint.to_bytes().as_slice(),
            &[won as u8],
            &kills.to_le_bytes(),
            &exp_gained.to_le_bytes(),
            &timestamp.to_le_bytes(),
        ].concat();
        
        self.keypair.sign(&message).to_bytes()
    }
}
```

---

## 6. Deployment Configuration

### 6.1 Docker Compose (Development)

```yaml
# docker-compose.yml

version: '3.8'

services:
  postgres:
    image: postgis/postgis:16-3.4
    environment:
      POSTGRES_USER: breach
      POSTGRES_PASSWORD: breach_dev_password
      POSTGRES_DB: breach
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./infrastructure/docker/init.sql:/docker-entrypoint-initdb.d/init.sql

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  meilisearch:
    image: getmeili/meilisearch:v1.5
    environment:
      MEILI_ENV: development
    ports:
      - "7700:7700"
    volumes:
      - meilisearch_data:/meili_data

  api:
    build:
      context: .
      dockerfile: ./backend/api/Dockerfile
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://breach:breach_dev_password@postgres:5432/breach
      REDIS_URL: redis://redis:6379
      SOLANA_RPC_URL: https://api.devnet.solana.com
      BACKEND_SIGNER_KEYPAIR: ${BACKEND_SIGNER_KEYPAIR}
    depends_on:
      - postgres
      - redis

  game-server:
    build:
      context: .
      dockerfile: ./backend/game-server/Dockerfile
    ports:
      - "8081:8081"
    environment:
      DATABASE_URL: postgres://breach:breach_dev_password@postgres:5432/breach
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis

  realtime:
    build:
      context: .
      dockerfile: ./backend/realtime/Dockerfile
    ports:
      - "8082:8082"
    environment:
      REDIS_URL: redis://redis:6379
    depends_on:
      - redis

volumes:
  postgres_data:
  redis_data:
  meilisearch_data:
```

### 6.2 Kubernetes (Production)

```yaml
# infrastructure/k8s/api-deployment.yaml

apiVersion: apps/v1
kind: Deployment
metadata:
  name: breach-api
  namespace: breach
spec:
  replicas: 3
  selector:
    matchLabels:
      app: breach-api
  template:
    metadata:
      labels:
        app: breach-api
    spec:
      containers:
        - name: api
          image: breach/api:latest
          ports:
            - containerPort: 8080
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: breach-secrets
                  key: database-url
            - name: REDIS_URL
              valueFrom:
                secretKeyRef:
                  name: breach-secrets
                  key: redis-url
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: breach-api
  namespace: breach
spec:
  selector:
    app: breach-api
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: breach-api-ingress
  namespace: breach
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - api.breach.gg
      secretName: breach-api-tls
  rules:
    - host: api.breach.gg
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: breach-api
                port:
                  number: 80
```

---

## 7. Monitoring & Logging

### 7.1 Prometheus Metrics

```rust
// backend/api/src/metrics.rs

use prometheus::{
    Counter, Histogram, IntGauge, Registry,
    histogram_opts, opts,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::with_opts(
        opts!("http_requests_total", "Total HTTP requests")
    ).unwrap();
    
    pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
        histogram_opts!(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        )
    ).unwrap();
    
    pub static ref ACTIVE_WEBSOCKET_CONNECTIONS: IntGauge = IntGauge::new(
        "active_websocket_connections",
        "Number of active WebSocket connections"
    ).unwrap();
    
    pub static ref TITANS_MINTED_TOTAL: Counter = Counter::with_opts(
        opts!("titans_minted_total", "Total titans minted")
    ).unwrap();
    
    pub static ref BATTLES_TOTAL: Counter = Counter::with_opts(
        opts!("battles_total", "Total battles played")
    ).unwrap();
}
```

### 7.2 Logging Configuration

```rust
// backend/api/src/logging.rs

use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().json())
        .init();
}

// Usage example
#[tracing::instrument(skip(db))]
pub async fn mint_titan(
    db: &PgPool,
    user_id: Uuid,
    titan_data: TitanData,
) -> Result<Titan, Error> {
    tracing::info!(
        user_id = %user_id,
        species_id = titan_data.species_id,
        class = titan_data.class,
        "Minting new titan"
    );
    
    // ... implementation
}
```

---

**End of Document**

*BREACH Technical Specification v1.0*
