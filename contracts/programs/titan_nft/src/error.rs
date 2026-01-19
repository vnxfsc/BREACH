//! Custom program errors

use pinocchio::program_error::ProgramError;

/// Titan NFT program errors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TitanError {
    // ═══════════ Authorization (6000-6099) ═══════════
    
    /// Not authorized to perform this action
    Unauthorized = 6000,
    
    /// Invalid capture authority signer
    InvalidCaptureAuthority = 6001,
    
    /// Not the owner of this Titan
    NotOwner = 6002,
    
    /// Invalid admin authority
    InvalidAuthority = 6003,
    
    // ═══════════ Program State (6100-6199) ═══════════
    
    /// Program is currently paused
    ProgramPaused = 6100,
    
    /// Account already initialized
    AlreadyInitialized = 6101,
    
    /// Account not initialized
    NotInitialized = 6102,
    
    // ═══════════ Capture (6200-6299) ═══════════
    
    /// Capture cooldown not elapsed
    CaptureCooldown = 6200,
    
    /// Maximum Titans per wallet reached
    MaxTitansReached = 6201,
    
    /// Invalid capture proof/signature
    InvalidCaptureProof = 6202,
    
    /// Invalid capture location
    InvalidLocation = 6203,
    
    // ═══════════ Titan Validation (6300-6399) ═══════════
    
    /// Invalid threat class (must be 1-5)
    InvalidThreatClass = 6300,
    
    /// Invalid element type (must be 0-5)
    InvalidElementType = 6301,
    
    /// Maximum level reached
    MaxLevelReached = 6302,
    
    /// Insufficient experience for level up
    InsufficientExperience = 6303,
    
    /// Cannot evolve this Titan
    CannotEvolve = 6304,
    
    /// Invalid species ID
    InvalidSpeciesId = 6305,
    
    // ═══════════ Fusion (6400-6499) ═══════════
    
    /// Cannot fuse Titan with itself
    CannotFuseWithSelf = 6400,
    
    /// Titan level too low for fusion
    LevelTooLowForFusion = 6401,
    
    /// Element type mismatch for fusion
    ElementMismatch = 6402,
    
    /// Both Titans must be owned by same player
    FusionOwnerMismatch = 6403,
    
    // ═══════════ Token (6500-6599) ═══════════
    
    /// Insufficient $BREACH balance
    InsufficientBalance = 6500,
    
    /// Token transfer failed
    TransferFailed = 6501,
    
    /// Invalid token mint
    InvalidMint = 6502,
    
    // ═══════════ Account (6600-6699) ═══════════
    
    /// Invalid account data
    InvalidAccountData = 6600,
    
    /// Account data too small
    AccountDataTooSmall = 6601,
    
    /// Invalid PDA seeds
    InvalidSeeds = 6602,
    
    /// Invalid program ID
    InvalidProgramId = 6603,
}

impl From<TitanError> for ProgramError {
    fn from(e: TitanError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl TitanError {
    /// Get error message
    pub fn message(&self) -> &'static str {
        match self {
            Self::Unauthorized => "Not authorized to perform this action",
            Self::InvalidCaptureAuthority => "Invalid capture authority signer",
            Self::NotOwner => "Not the owner of this Titan",
            Self::InvalidAuthority => "Invalid admin authority",
            Self::ProgramPaused => "Program is currently paused",
            Self::AlreadyInitialized => "Account already initialized",
            Self::NotInitialized => "Account not initialized",
            Self::CaptureCooldown => "Capture cooldown not elapsed",
            Self::MaxTitansReached => "Maximum Titans per wallet reached",
            Self::InvalidCaptureProof => "Invalid capture proof/signature",
            Self::InvalidLocation => "Invalid capture location",
            Self::InvalidThreatClass => "Invalid threat class (must be 1-5)",
            Self::InvalidElementType => "Invalid element type (must be 0-5)",
            Self::MaxLevelReached => "Maximum level reached",
            Self::InsufficientExperience => "Insufficient experience for level up",
            Self::CannotEvolve => "Cannot evolve this Titan",
            Self::InvalidSpeciesId => "Invalid species ID",
            Self::CannotFuseWithSelf => "Cannot fuse Titan with itself",
            Self::LevelTooLowForFusion => "Titan level too low for fusion",
            Self::ElementMismatch => "Element type mismatch for fusion",
            Self::FusionOwnerMismatch => "Both Titans must be owned by same player",
            Self::InsufficientBalance => "Insufficient $BREACH balance",
            Self::TransferFailed => "Token transfer failed",
            Self::InvalidMint => "Invalid token mint",
            Self::InvalidAccountData => "Invalid account data",
            Self::AccountDataTooSmall => "Account data too small",
            Self::InvalidSeeds => "Invalid PDA seeds",
            Self::InvalidProgramId => "Invalid program ID",
        }
    }
}
