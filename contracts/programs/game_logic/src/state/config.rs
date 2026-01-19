//! Game Logic configuration account

use pinocchio::pubkey::Pubkey;

/// Game Logic configuration account
/// PDA: ["game_config"]
#[repr(packed)]
pub struct GameConfig {
    /// Account discriminator
    pub discriminator: [u8; 8],
    
    /// Program authority (can update config, pause)
    pub authority: Pubkey,
    
    /// Backend signer (validates captures and battles)
    pub backend_authority: Pubkey,
    
    /// Titan NFT Program ID (for CPI)
    pub titan_program: Pubkey,
    
    /// $BREACH token mint address
    pub breach_mint: Pubkey,
    
    /// Reward pool token account
    pub reward_pool: Pubkey,
    
    /// Experience multiplier (100 = 1x, 200 = 2x)
    pub exp_multiplier: u16,
    
    /// Battle cooldown in seconds
    pub battle_cooldown_seconds: u32,
    
    /// Capture signature validity in seconds
    pub capture_validity_seconds: u32,
    
    /// Base reward per battle win (in lamports)
    pub battle_reward_base: u64,
    
    /// Base reward per capture (in lamports)
    pub capture_reward_base: u64,
    
    /// Program paused flag
    pub paused: bool,
    
    /// PDA bump seed
    pub bump: u8,
    
    /// Total battles recorded
    pub total_battles: u64,
    
    /// Total captures recorded
    pub total_captures: u64,
    
    /// Total experience distributed
    pub total_exp_distributed: u64,
    
    /// Total rewards distributed (in lamports)
    pub total_rewards_distributed: u64,
}

impl GameConfig {
    /// Account size in bytes (packed)
    pub const SIZE: usize = 8 + 32 + 32 + 32 + 32 + 32 + 2 + 4 + 4 + 8 + 8 + 1 + 1 + 8 + 8 + 8 + 8;
    // = 8 + 160 + 2 + 4 + 4 + 8 + 8 + 1 + 1 + 8 + 8 + 8 + 8 = 228 bytes
    
    /// Account discriminator
    pub const DISCRIMINATOR: [u8; 8] = *b"gamecfg_";
    
    /// PDA seed
    pub const SEED: &'static [u8] = b"game_config";
    
    /// Deserialize from account data
    pub fn from_account_data(data: &[u8]) -> Result<&Self, pinocchio::program_error::ProgramError> {
        if data.len() < Self::SIZE {
            return Err(pinocchio::program_error::ProgramError::AccountDataTooSmall);
        }
        
        let config = unsafe { &*(data.as_ptr() as *const Self) };
        
        if config.discriminator != Self::DISCRIMINATOR {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        
        Ok(config)
    }
    
    /// Deserialize mutable from account data
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, pinocchio::program_error::ProgramError> {
        if data.len() < Self::SIZE {
            return Err(pinocchio::program_error::ProgramError::AccountDataTooSmall);
        }
        
        let config = unsafe { &mut *(data.as_mut_ptr() as *mut Self) };
        
        Ok(config)
    }
}
