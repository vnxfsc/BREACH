//! Update Config instruction

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::error::GameError;
use crate::state::GameConfig;

/// Update config instruction data
#[repr(C, packed)]
pub struct UpdateConfigData {
    /// New backend authority
    pub backend_authority: Pubkey,
    /// New experience multiplier (100 = 1x)
    pub exp_multiplier: u16,
    /// New battle cooldown in seconds
    pub battle_cooldown_seconds: u32,
    /// New capture validity in seconds
    pub capture_validity_seconds: u32,
    /// New base battle reward
    pub battle_reward_base: u64,
    /// New base capture reward
    pub capture_reward_base: u64,
}

/// Process update config instruction
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

    // Load config
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;

    // Verify authority
    if authority.key() != &config.authority {
        return Err(GameError::InvalidAuthority.into());
    }

    // Parse instruction data
    if data.len() < std::mem::size_of::<UpdateConfigData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let update_data = unsafe { &*(data.as_ptr() as *const UpdateConfigData) };

    // Update config fields
    config.backend_authority = update_data.backend_authority;
    config.exp_multiplier = update_data.exp_multiplier;
    config.battle_cooldown_seconds = update_data.battle_cooldown_seconds;
    config.capture_validity_seconds = update_data.capture_validity_seconds;
    config.battle_reward_base = update_data.battle_reward_base;
    config.capture_reward_base = update_data.capture_reward_base;

    Ok(())
}
