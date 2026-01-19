//! Set Paused instruction

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::error::GameError;
use crate::state::GameConfig;

/// Process set paused instruction
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        authority,      // [0] Signer, program authority
        config_account, // [1] Config PDA
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify authority is signer
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse instruction data (1 byte: paused flag)
    if data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let paused = data[0] != 0;

    // Load config
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;

    // Verify authority
    if authority.key() != &config.authority {
        return Err(GameError::InvalidAuthority.into());
    }

    // Update paused state
    config.paused = paused;

    Ok(())
}
