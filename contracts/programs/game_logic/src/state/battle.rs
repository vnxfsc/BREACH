//! Battle record account

use pinocchio::pubkey::Pubkey;

/// Battle record account
/// PDA: ["battle", battle_id (u64)]
#[repr(packed)]
pub struct BattleRecord {
    /// Account discriminator
    pub discriminator: [u8; 8],
    
    /// Unique battle ID
    pub battle_id: u64,
    
    /// Player A (initiator)
    pub player_a: Pubkey,
    
    /// Player A's Titan ID
    pub titan_a_id: u64,
    
    /// Player B (opponent)
    pub player_b: Pubkey,
    
    /// Player B's Titan ID
    pub titan_b_id: u64,
    
    /// Winner (0 = Player A, 1 = Player B, 2 = Draw)
    pub winner: u8,
    
    /// Experience gained by Player A's Titan
    pub exp_gained_a: u32,
    
    /// Experience gained by Player B's Titan
    pub exp_gained_b: u32,
    
    /// Battle timestamp (Unix timestamp)
    pub timestamp: i64,
    
    /// Battle location (lat * 1e6)
    pub location_lat: i32,
    
    /// Battle location (lng * 1e6)
    pub location_lng: i32,
    
    /// PDA bump seed
    pub bump: u8,
}

impl BattleRecord {
    /// Account size in bytes (packed)
    pub const SIZE: usize = 8 + 8 + 32 + 8 + 32 + 8 + 1 + 4 + 4 + 8 + 4 + 4 + 1;
    // = 8 + 8 + 32 + 8 + 32 + 8 + 1 + 4 + 4 + 8 + 4 + 4 + 1 = 122 bytes
    
    /// Account discriminator
    pub const DISCRIMINATOR: [u8; 8] = *b"battle__";
    
    /// PDA seed prefix
    pub const SEED: &'static [u8] = b"battle";
    
    /// Deserialize from account data
    pub fn from_account_data(data: &[u8]) -> Result<&Self, pinocchio::program_error::ProgramError> {
        if data.len() < Self::SIZE {
            return Err(pinocchio::program_error::ProgramError::AccountDataTooSmall);
        }
        
        let record = unsafe { &*(data.as_ptr() as *const Self) };
        
        if record.discriminator != Self::DISCRIMINATOR {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        
        Ok(record)
    }
    
    /// Deserialize mutable from account data
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, pinocchio::program_error::ProgramError> {
        if data.len() < Self::SIZE {
            return Err(pinocchio::program_error::ProgramError::AccountDataTooSmall);
        }
        
        let record = unsafe { &mut *(data.as_mut_ptr() as *mut Self) };
        
        Ok(record)
    }
}

/// Battle result enum
#[repr(u8)]
pub enum BattleResult {
    PlayerAWins = 0,
    PlayerBWins = 1,
    Draw = 2,
}
