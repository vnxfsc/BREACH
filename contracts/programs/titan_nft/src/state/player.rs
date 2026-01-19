//! Player account data

use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Player profile account
/// PDA: ["player", wallet_pubkey]
/// Size: 152 bytes
#[repr(C)]
pub struct PlayerAccount {
    /// Account discriminator
    pub discriminator: [u8; 8],
    
    /// Player wallet address
    pub wallet: Pubkey,
    
    /// Username (32 bytes, null-padded)
    pub username: [u8; 32],
    
    // ═══════════ Statistics ═══════════
    
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
    
    /// Peak Elo rating achieved
    pub peak_elo: u16,
    
    // ═══════════ Economy ═══════════
    
    /// Total $BREACH spent
    pub total_breach_spent: u64,
    
    /// Total $BREACH earned
    pub total_breach_earned: u64,
    
    // ═══════════ Timestamps ═══════════
    
    /// Last capture timestamp
    pub last_capture_at: i64,
    
    /// Account creation timestamp
    pub created_at: i64,
    
    /// PDA bump seed
    pub bump: u8,
    
    /// Reserved for future use
    pub _reserved: [u8; 7],
}

impl PlayerAccount {
    /// Account size in bytes
    pub const SIZE: usize = 152;
    
    /// Account discriminator
    pub const DISCRIMINATOR: [u8; 8] = [0x50, 0x4C, 0x41, 0x59, 0x45, 0x52, 0x41, 0x43]; // "PLAYERAC"
    
    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"player";
    
    /// Initial Elo rating
    pub const INITIAL_ELO: u16 = 1000;
    
    /// Zero-copy read from account data
    #[inline]
    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let player = unsafe { &*(data.as_ptr() as *const PlayerAccount) };
        
        if player.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        
        Ok(player)
    }
    
    /// Zero-copy mutable read from account data
    #[inline]
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let player = unsafe { &mut *(data.as_mut_ptr() as *mut PlayerAccount) };
        
        if player.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        
        Ok(player)
    }
    
    /// Initialize new player (for uninitialized accounts)
    #[inline]
    pub fn init_from_account_data(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let player = unsafe { &mut *(data.as_mut_ptr() as *mut PlayerAccount) };
        player.discriminator = Self::DISCRIMINATOR;
        
        Ok(player)
    }
    
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
