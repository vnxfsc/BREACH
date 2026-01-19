//! Global configuration account

use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Global configuration account
/// PDA: ["config"]
/// Size: 182 bytes (packed, no padding)
#[repr(C, packed)]
pub struct GlobalConfig {
    /// Account discriminator
    pub discriminator: [u8; 8],
    
    /// Program authority (can update config, pause)
    pub authority: Pubkey,
    
    /// Treasury wallet for collecting fees
    pub treasury: Pubkey,
    
    /// $BREACH token mint address
    pub breach_mint: Pubkey,
    
    /// Backend signer (validates captures)
    pub capture_authority: Pubkey,
    
    /// Capture fee in basis points (100 = 1%)
    pub capture_fee_bps: u16,
    
    /// Marketplace fee in basis points
    pub marketplace_fee_bps: u16,
    
    /// Fusion fee in basis points
    pub fusion_fee_bps: u16,
    
    /// Maximum Titans per wallet
    pub max_titans_per_wallet: u16,
    
    /// Capture cooldown in seconds
    pub capture_cooldown_seconds: u32,
    
    /// Program paused flag
    pub paused: bool,
    
    /// PDA bump seed
    pub bump: u8,
    
    /// Total Titans minted
    pub total_titans_minted: u64,
    
    /// Total battles recorded
    pub total_battles: u64,
    
    /// Total fusions performed
    pub total_fusions: u64,
    
    /// Total fees collected (in lamports)
    pub total_fees_collected: u64,
}

impl GlobalConfig {
    /// Account size in bytes (packed, no padding)
    pub const SIZE: usize = 182;
    
    /// Account discriminator
    pub const DISCRIMINATOR: [u8; 8] = [0x42, 0x52, 0x45, 0x41, 0x43, 0x48, 0x43, 0x46]; // "BREACHCF"
    
    /// PDA seed
    pub const SEED: &'static [u8] = b"config";
    
    /// Zero-copy read from account data
    #[inline]
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
    
    /// Zero-copy mutable read from account data
    #[inline]
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
    
    /// Initialize new config (for uninitialized accounts)
    #[inline]
    pub fn init_from_account_data(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let config = unsafe { &mut *(data.as_mut_ptr() as *mut GlobalConfig) };
        config.discriminator = Self::DISCRIMINATOR;
        
        Ok(config)
    }
}
