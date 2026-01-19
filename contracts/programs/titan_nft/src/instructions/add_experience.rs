//! Add Experience instruction
//!
//! Adds experience points to a Titan. Can only be called by authorized programs
//! (Game Logic Program) via CPI.

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::state::{GlobalConfig, TitanData};

/// Add experience instruction data
#[repr(C, packed)]
pub struct AddExperienceData {
    /// Amount of experience to add
    pub exp_amount: u32,
}

/// Process add_experience instruction
/// 
/// Accounts:
/// 0. `[signer]` Backend authority (verified against config)
/// 1. `[]` Global config PDA
/// 2. `[writable]` Titan data PDA
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        backend_authority, // [0] Signer - authorized backend
        config_account,    // [1] Global config PDA
        titan_account,     // [2] Titan data PDA (writable)
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate backend authority is signer
    if !backend_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load and validate config
    let config_data = config_account.try_borrow_data()?;
    let config = GlobalConfig::from_account_data(&config_data)?;

    // Check if program is paused
    if config.paused {
        return Err(ProgramError::Custom(1)); // Program paused
    }

    // Verify backend authority matches config
    if backend_authority.key() != &config.authority {
        return Err(ProgramError::Custom(2)); // Unauthorized
    }

    // Parse instruction data
    if data.len() < std::mem::size_of::<AddExperienceData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let exp_data = unsafe { &*(data.as_ptr() as *const AddExperienceData) };

    // Validate experience amount
    if exp_data.exp_amount == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Load and update Titan
    let mut titan_data = titan_account.try_borrow_mut_data()?;
    let titan = TitanData::from_account_data_mut(&mut titan_data)?;

    // Add experience with overflow protection
    titan.experience = titan.experience.saturating_add(exp_data.exp_amount);

    // Update config stats
    drop(config_data);
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GlobalConfig::from_account_data_mut(&mut config_data)?;
    config.total_titans_minted = config.total_titans_minted; // Just to mark as used
    // Note: Could add total_exp_distributed counter to GlobalConfig if needed

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_experience_data_size() {
        assert_eq!(std::mem::size_of::<AddExperienceData>(), 4);
    }
}
