//! Titan NFT data account

use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

/// Titan NFT on-chain data
/// PDA: ["titan", titan_id.to_le_bytes()]
/// Size: ~118 bytes
#[repr(C, packed)]
pub struct TitanData {
    /// Account discriminator
    pub discriminator: [u8; 8],
    
    /// Unique Titan ID
    pub titan_id: u64,
    
    /// Species ID (determines base stats and appearance)
    pub species_id: u16,
    
    /// Threat class (1-5: Pioneer, Hunter, Destroyer, Calamity, Apex)
    pub threat_class: u8,
    
    /// Elemental type (0-5: Abyssal, Volcanic, Storm, Void, Parasitic, Ossified)
    pub element_type: u8,
    
    // ═══════════ Visible Attributes (4 bytes) ═══════════
    
    /// Power - Attack damage
    pub power: u8,
    
    /// Fortitude - Defense
    pub fortitude: u8,
    
    /// Velocity - Speed/Initiative
    pub velocity: u8,
    
    /// Resonance - Special ability strength
    pub resonance: u8,
    
    // ═══════════ Hidden Genes (6 bytes) ═══════════
    
    /// Gene sequence [ATK, SPD, DEF, GRW, SKL, MUT]
    pub genes: [u8; 6],
    
    // ═══════════ Growth Data ═══════════
    
    /// Current level (1-100)
    pub level: u8,
    
    /// Experience points
    pub experience: u32,
    
    /// Neural link strength with owner (0-100)
    pub link_strength: u8,
    
    // ═══════════ Metadata ═══════════
    
    /// Capture timestamp (Unix)
    pub captured_at: i64,
    
    /// Original capturer address
    pub original_owner: Pubkey,
    
    /// Capture location (encoded lat/lng)
    pub capture_location: u64,
    
    /// Generation (0 = wild, 1+ = fusion offspring)
    pub generation: u8,
    
    /// Parent A titan ID (0 if wild)
    pub parent_a: u64,
    
    /// Parent B titan ID (0 if wild)
    pub parent_b: u64,
    
    /// PDA bump seed
    pub bump: u8,
}

impl TitanData {
    /// Account size in bytes
    pub const SIZE: usize = 118;
    
    /// Account discriminator
    pub const DISCRIMINATOR: [u8; 8] = [0x54, 0x49, 0x54, 0x41, 0x4E, 0x44, 0x41, 0x54]; // "TITANDAT"
    
    /// PDA seed prefix
    pub const SEED_PREFIX: &'static [u8] = b"titan";
    
    /// Maximum level
    pub const MAX_LEVEL: u8 = 100;
    
    /// Zero-copy read from account data
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
    
    /// Zero-copy mutable read from account data
    #[inline]
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let titan = unsafe { &mut *(data.as_mut_ptr() as *mut TitanData) };
        
        if titan.discriminator != Self::DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }
        
        Ok(titan)
    }
    
    /// Initialize new titan (for uninitialized accounts)
    #[inline]
    pub fn init_from_account_data(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidAccountData);
        }
        
        let titan = unsafe { &mut *(data.as_mut_ptr() as *mut TitanData) };
        titan.discriminator = Self::DISCRIMINATOR;
        
        Ok(titan)
    }
    
    /// Calculate combat power
    pub fn combat_power(&self) -> u32 {
        let class_mult = match self.threat_class {
            1 => 100,  // Pioneer
            2 => 115,  // Hunter
            3 => 130,  // Destroyer
            4 => 150,  // Calamity
            5 => 180,  // Apex
            _ => 100,
        };
        
        let base = (self.power as u32
            + self.fortitude as u32
            + self.velocity as u32
            + self.resonance as u32)
            * 2;
        
        let level_bonus = self.level as u32 * 10;
        let gene_bonus: u32 = self.genes.iter().map(|g| *g as u32).sum::<u32>() / 10;
        
        (base + level_bonus + gene_bonus) * class_mult / 100
    }
    
    /// Experience required for next level
    pub fn exp_for_next_level(&self) -> u32 {
        // level^2 * 100
        (self.level as u32 + 1).pow(2) * 100
    }
    
    /// Check if can level up
    pub fn can_level_up(&self) -> bool {
        self.level < Self::MAX_LEVEL && self.experience >= self.exp_for_next_level()
    }
}

/// Element type enumeration
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Abyssal = 0,
    Volcanic = 1,
    Storm = 2,
    Void = 3,
    Parasitic = 4,
    Ossified = 5,
}

impl ElementType {
    /// Get damage multiplier (150 = super effective, 100 = normal, 75 = not effective)
    /// Effectiveness chain: Abyssal > Volcanic > Storm > Void > Parasitic > Ossified > Abyssal
    pub fn get_multiplier(attacker: u8, defender: u8) -> u8 {
        let beats = [1, 2, 3, 4, 5, 0]; // What each type beats
        
        if beats.get(attacker as usize).copied() == Some(defender) {
            150 // Super effective
        } else if beats.get(defender as usize).copied() == Some(attacker) {
            75 // Not very effective
        } else {
            100 // Normal
        }
    }
}

/// Threat class enumeration
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreatClass {
    Pioneer = 1,   // Class I - Common (60%)
    Hunter = 2,    // Class II - Uncommon (25%)
    Destroyer = 3, // Class III - Rare (10%)
    Calamity = 4,  // Class IV - Epic (4%)
    Apex = 5,      // Class V - Legendary (1%)
}
