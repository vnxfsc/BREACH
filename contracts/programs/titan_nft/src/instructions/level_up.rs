//! Level up Titan instruction

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{error::TitanError, state::TitanData};

/// Process level_up instruction
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [owner, titan_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate owner is signer
    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load Titan
    let mut titan_data = titan_account.try_borrow_mut_data()?;
    let titan = TitanData::from_account_data_mut(&mut titan_data)?;

    // Check max level
    if titan.level >= TitanData::MAX_LEVEL {
        return Err(TitanError::MaxLevelReached.into());
    }

    // Check experience requirement
    let required_exp = titan.exp_for_next_level();
    if titan.experience < required_exp {
        return Err(TitanError::InsufficientExperience.into());
    }

    // Level up
    titan.experience -= required_exp;
    titan.level += 1;

    // Increase link strength slightly on level up
    if titan.link_strength < 100 {
        titan.link_strength = titan.link_strength.saturating_add(1);
    }

    Ok(())
}
