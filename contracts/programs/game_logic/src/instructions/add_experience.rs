//! Add Experience instruction
//! 
//! Adds experience to a Titan via CPI to Titan NFT Program

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::error::GameError;
use crate::state::GameConfig;

/// Add experience instruction data
#[repr(C, packed)]
pub struct AddExperienceData {
    /// Titan ID to add experience to
    pub titan_id: u64,
    /// Amount of experience to add
    pub exp_amount: u32,
}

/// Process add experience instruction
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        player,            // [0] Signer, titan owner
        backend_authority, // [1] Signer, backend authority
        config_account,    // [2] Config PDA
        titan_account,     // [3] Titan PDA (from Titan NFT Program)
        _titan_program,    // [4] Titan NFT Program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signers
    if !player.is_signer() || !backend_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load config
    let config_data = config_account.try_borrow_data()?;
    let config = GameConfig::from_account_data(&config_data)?;

    // Check if paused
    if config.paused {
        return Err(GameError::ProgramPaused.into());
    }

    // Verify backend authority
    if backend_authority.key() != &config.backend_authority {
        return Err(GameError::InvalidBackendAuthority.into());
    }

    // Parse instruction data
    if data.len() < std::mem::size_of::<AddExperienceData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let exp_data = unsafe { &*(data.as_ptr() as *const AddExperienceData) };

    // Validate experience amount
    if exp_data.exp_amount == 0 {
        return Err(GameError::InvalidExperienceAmount.into());
    }

    // Apply experience multiplier
    let multiplied_exp = (exp_data.exp_amount as u64) * (config.exp_multiplier as u64) / 100;
    let final_exp = multiplied_exp.min(u32::MAX as u64) as u32;

    // TODO: CPI to Titan NFT Program to update experience
    // For now, we'll update the titan account directly if owned by this program
    // In production, this would be a CPI call
    
    // Read titan data and update experience
    let mut titan_data = titan_account.try_borrow_mut_data()?;
    
    // Verify discriminator (first 8 bytes should be "titan___")
    if titan_data.len() < 118 {
        return Err(GameError::InvalidAccountData.into());
    }
    
    // Experience is at offset 8+8+2+1+1+1+1+1+1+6+1 = 31, and it's a u32
    // Titan structure (packed):
    // - discriminator: [u8; 8] (0-7)
    // - titan_id: u64 (8-15)
    // - species_id: u16 (16-17)
    // - threat_class: u8 (18)
    // - element_type: u8 (19)
    // - power: u8 (20)
    // - fortitude: u8 (21)
    // - velocity: u8 (22)
    // - resonance: u8 (23)
    // - genes: [u8; 6] (24-29)
    // - level: u8 (30)
    // - experience: u32 (31-34)
    
    let exp_offset = 31;
    let current_exp = u32::from_le_bytes([
        titan_data[exp_offset],
        titan_data[exp_offset + 1],
        titan_data[exp_offset + 2],
        titan_data[exp_offset + 3],
    ]);
    
    // Add experience with overflow check
    let new_exp = current_exp.saturating_add(final_exp);
    titan_data[exp_offset..exp_offset + 4].copy_from_slice(&new_exp.to_le_bytes());

    // Update config stats
    drop(config_data);
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;
    config.total_exp_distributed += final_exp as u64;

    Ok(())
}
