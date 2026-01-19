//! Set program paused state instruction (admin only)

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{error::TitanError, state::GlobalConfig};

/// Process set_paused instruction
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [authority, config_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate authority is signer
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load config and validate authority
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GlobalConfig::from_account_data_mut(&mut config_data)?;

    if *authority.key() != config.authority {
        return Err(TitanError::InvalidAuthority.into());
    }

    // Parse pause flag from data
    if data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let paused = data[0] != 0;

    config.paused = paused;

    Ok(())
}
