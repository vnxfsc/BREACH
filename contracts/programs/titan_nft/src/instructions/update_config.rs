//! Update program config instruction (admin only)

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{error::TitanError, state::GlobalConfig};

/// Update config instruction data
#[repr(C, packed)]
pub struct UpdateConfigData {
    /// New treasury address (or zero to keep current)
    pub treasury: Pubkey,
    /// New capture authority (or zero to keep current)
    pub capture_authority: Pubkey,
    /// New capture fee in basis points
    pub capture_fee_bps: u16,
    /// New marketplace fee in basis points
    pub marketplace_fee_bps: u16,
    /// New fusion fee in basis points
    pub fusion_fee_bps: u16,
    /// New max titans per wallet
    pub max_titans_per_wallet: u16,
    /// New capture cooldown in seconds
    pub capture_cooldown_seconds: u32,
}

/// Process update_config instruction
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

    // Parse instruction data
    if data.len() < core::mem::size_of::<UpdateConfigData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let update_data = unsafe { &*(data.as_ptr() as *const UpdateConfigData) };

    // Update fields (only if non-zero provided)
    if update_data.treasury != Pubkey::default() {
        config.treasury = update_data.treasury;
    }
    if update_data.capture_authority != Pubkey::default() {
        config.capture_authority = update_data.capture_authority;
    }
    
    config.capture_fee_bps = update_data.capture_fee_bps;
    config.marketplace_fee_bps = update_data.marketplace_fee_bps;
    config.fusion_fee_bps = update_data.fusion_fee_bps;
    config.max_titans_per_wallet = update_data.max_titans_per_wallet;
    config.capture_cooldown_seconds = update_data.capture_cooldown_seconds;

    Ok(())
}
