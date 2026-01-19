//! Custom program errors for Game Logic

use pinocchio::program_error::ProgramError;

/// Game Logic program errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum GameError {
    // ═══════════ Authorization (7000-7099) ═══════════
    
    /// Not authorized to perform this action
    Unauthorized = 7000,
    
    /// Invalid backend authority signer
    InvalidBackendAuthority = 7001,
    
    /// Not the owner of this Titan
    NotOwner = 7002,
    
    /// Invalid admin authority
    InvalidAuthority = 7003,
    
    // ═══════════ Program State (7100-7199) ═══════════
    
    /// Program is currently paused
    ProgramPaused = 7100,
    
    /// Account already initialized
    AlreadyInitialized = 7101,
    
    /// Account not initialized
    NotInitialized = 7102,
    
    /// Invalid config account
    InvalidConfig = 7103,
    
    // ═══════════ Battle (7200-7299) ═══════════
    
    /// Invalid battle signature from backend
    InvalidBattleSignature = 7200,
    
    /// Battle already recorded
    BattleAlreadyRecorded = 7201,
    
    /// Invalid opponent titan
    InvalidOpponent = 7202,
    
    /// Cannot battle own titan
    CannotBattleSelf = 7203,
    
    /// Battle cooldown not elapsed
    BattleCooldown = 7204,
    
    // ═══════════ Capture (7300-7399) ═══════════
    
    /// Invalid capture signature from backend
    InvalidCaptureSignature = 7300,
    
    /// Capture already recorded
    CaptureAlreadyRecorded = 7301,
    
    /// Invalid capture location
    InvalidCaptureLocation = 7302,
    
    /// Capture timestamp too old
    CaptureExpired = 7303,
    
    // ═══════════ Experience (7400-7499) ═══════════
    
    /// Invalid experience amount
    InvalidExperienceAmount = 7400,
    
    /// Experience overflow
    ExperienceOverflow = 7401,
    
    // ═══════════ Rewards (7500-7599) ═══════════
    
    /// Invalid reward amount
    InvalidRewardAmount = 7500,
    
    /// Insufficient reward pool
    InsufficientRewardPool = 7501,
    
    /// Reward already claimed
    RewardAlreadyClaimed = 7502,
    
    // ═══════════ Account (7600-7699) ═══════════
    
    /// Invalid account data
    InvalidAccountData = 7600,
    
    /// Account data too small
    AccountDataTooSmall = 7601,
    
    /// Invalid PDA seeds
    InvalidSeeds = 7602,
    
    /// Invalid program ID
    InvalidProgramId = 7603,
    
    /// CPI call failed
    CpiCallFailed = 7604,
    
    /// Invalid Titan NFT Program ID
    InvalidTitanProgram = 7605,
}

impl From<GameError> for ProgramError {
    fn from(e: GameError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl GameError {
    /// Get error message
    pub fn message(&self) -> &'static str {
        match self {
            Self::Unauthorized => "Not authorized to perform this action",
            Self::InvalidBackendAuthority => "Invalid backend authority signer",
            Self::NotOwner => "Not the owner of this Titan",
            Self::InvalidAuthority => "Invalid admin authority",
            Self::ProgramPaused => "Program is currently paused",
            Self::AlreadyInitialized => "Account already initialized",
            Self::NotInitialized => "Account not initialized",
            Self::InvalidConfig => "Invalid config account",
            Self::InvalidBattleSignature => "Invalid battle signature from backend",
            Self::BattleAlreadyRecorded => "Battle already recorded",
            Self::InvalidOpponent => "Invalid opponent titan",
            Self::CannotBattleSelf => "Cannot battle own titan",
            Self::BattleCooldown => "Battle cooldown not elapsed",
            Self::InvalidCaptureSignature => "Invalid capture signature from backend",
            Self::CaptureAlreadyRecorded => "Capture already recorded",
            Self::InvalidCaptureLocation => "Invalid capture location",
            Self::CaptureExpired => "Capture timestamp too old",
            Self::InvalidExperienceAmount => "Invalid experience amount",
            Self::ExperienceOverflow => "Experience overflow",
            Self::InvalidRewardAmount => "Invalid reward amount",
            Self::InsufficientRewardPool => "Insufficient reward pool",
            Self::RewardAlreadyClaimed => "Reward already claimed",
            Self::InvalidAccountData => "Invalid account data",
            Self::AccountDataTooSmall => "Account data too small",
            Self::InvalidSeeds => "Invalid PDA seeds",
            Self::InvalidProgramId => "Invalid program ID",
            Self::CpiCallFailed => "CPI call failed",
            Self::InvalidTitanProgram => "Invalid Titan NFT Program ID",
        }
    }
}
