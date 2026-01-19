//! Capture record account

use pinocchio::pubkey::Pubkey;

/// Capture record account
/// PDA: ["capture", capture_id (u64)]
#[repr(packed)]
pub struct CaptureRecord {
    /// Account discriminator
    pub discriminator: [u8; 8],
    
    /// Unique capture ID
    pub capture_id: u64,
    
    /// Player who captured
    pub player: Pubkey,
    
    /// Titan ID that was captured
    pub titan_id: u64,
    
    /// Capture location (lat * 1e6)
    pub location_lat: i32,
    
    /// Capture location (lng * 1e6)
    pub location_lng: i32,
    
    /// Capture timestamp (Unix timestamp)
    pub timestamp: i64,
    
    /// Threat class of captured Titan (1-5)
    pub threat_class: u8,
    
    /// Element type of captured Titan (0-5)
    pub element_type: u8,
    
    /// Reward amount distributed (in lamports)
    pub reward_amount: u64,
    
    /// PDA bump seed
    pub bump: u8,
}

impl CaptureRecord {
    /// Account size in bytes (packed)
    pub const SIZE: usize = 8 + 8 + 32 + 8 + 4 + 4 + 8 + 1 + 1 + 8 + 1;
    // = 8 + 8 + 32 + 8 + 4 + 4 + 8 + 1 + 1 + 8 + 1 = 83 bytes
    
    /// Account discriminator
    pub const DISCRIMINATOR: [u8; 8] = *b"capture_";
    
    /// PDA seed prefix
    pub const SEED: &'static [u8] = b"capture";
    
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

/// Daily capture limit tracking per player
/// PDA: ["daily_capture", player, day (u32)]
#[repr(packed)]
pub struct DailyCaptureLimit {
    /// Account discriminator
    pub discriminator: [u8; 8],
    
    /// Player wallet
    pub player: Pubkey,
    
    /// Day number (days since epoch)
    pub day: u32,
    
    /// Number of captures today
    pub capture_count: u16,
    
    /// PDA bump seed
    pub bump: u8,
}

impl DailyCaptureLimit {
    /// Account size in bytes (packed)
    pub const SIZE: usize = 8 + 32 + 4 + 2 + 1;
    // = 47 bytes
    
    /// Account discriminator
    pub const DISCRIMINATOR: [u8; 8] = *b"daily___";
    
    /// PDA seed prefix
    pub const SEED: &'static [u8] = b"daily_capture";
    
    /// Maximum captures per day
    pub const MAX_DAILY_CAPTURES: u16 = 50;
}
