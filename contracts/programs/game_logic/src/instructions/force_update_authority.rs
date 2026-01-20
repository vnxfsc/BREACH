//! Force Update Authority instruction
//!
//! This instruction allows the program's upgrade authority to force update
//! the config authority and backend authority. This is useful for recovering
//! access when the original authority keypair is lost.

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::error::GameError;
use crate::state::GameConfig;

// BPFLoaderUpgradeable Program ID
// BPFLoaderUpgradeab1e11111111111111111111111
const BPF_LOADER_UPGRADEABLE: Pubkey = [
    0x02, 0xa8, 0xf6, 0x91, 0x4e, 0x88, 0xa1, 0xb0,
    0xe2, 0x10, 0x15, 0x3e, 0xf7, 0x63, 0xae, 0x2b,
    0x00, 0xc2, 0xb9, 0x3d, 0x16, 0xc1, 0x24, 0xd2,
    0xc0, 0x53, 0x7a, 0x10, 0x04, 0x80, 0x00, 0x00,
];

/// Force update authority instruction data
#[repr(C, packed)]
pub struct ForceUpdateAuthorityData {
    /// 新的 config authority
    pub new_authority: Pubkey,
    /// 新的 backend authority
    pub new_backend_authority: Pubkey,
    /// 新的 titan_program
    pub new_titan_program: Pubkey,
    /// 新的 breach_mint
    pub new_breach_mint: Pubkey,
    /// 新的 reward_pool
    pub new_reward_pool: Pubkey,
}

/// Process force update authority instruction
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    // [0] Signer, must be program upgrade authority
    // [1] Config PDA
    // [2] ProgramData account (derived from program ID)
    let [
        upgrade_authority,  // [0] Signer, program upgrade authority
        config_account,     // [1] Config PDA
        program_data,       // [2] ProgramData account
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify upgrade authority is signer
    if !upgrade_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify program_data account is owned by BPF Loader Upgradeable
    // SAFETY: program_data is a valid AccountInfo from the runtime
    if unsafe { program_data.owner() } != &BPF_LOADER_UPGRADEABLE {
        return Err(GameError::InvalidAuthority.into());
    }

    // ProgramData account structure (first 45 bytes):
    // - 4 bytes: account type (must be 3 for ProgramData)
    // - 8 bytes: slot
    // - 1 byte: option flag for upgrade authority
    // - 32 bytes: upgrade authority pubkey (if present)
    let program_data_bytes = program_data.try_borrow_data()?;
    
    if program_data_bytes.len() < 45 {
        return Err(GameError::InvalidAccountData.into());
    }

    // Check account type (4 bytes, little endian)
    let account_type = u32::from_le_bytes([
        program_data_bytes[0],
        program_data_bytes[1],
        program_data_bytes[2],
        program_data_bytes[3],
    ]);
    
    // ProgramData account type is 3
    if account_type != 3 {
        return Err(GameError::InvalidAccountData.into());
    }

    // Check if upgrade authority is present (byte 12)
    if program_data_bytes[12] != 1 {
        // No upgrade authority set, cannot proceed
        return Err(GameError::InvalidAuthority.into());
    }

    // Extract upgrade authority pubkey (bytes 13-44)
    let stored_upgrade_authority = &program_data_bytes[13..45];
    
    // Verify the signer is the upgrade authority
    if upgrade_authority.key().as_ref() != stored_upgrade_authority {
        return Err(GameError::InvalidAuthority.into());
    }

    // Verify ProgramData is for this program
    // ProgramData PDA = find_program_address([""], BPF_LOADER, program_id)
    // We can verify by checking the program_data address matches
    let (expected_program_data, _bump) = pinocchio::pubkey::find_program_address(
        &[program_id.as_ref()],
        &BPF_LOADER_UPGRADEABLE,
    );
    
    if program_data.key() != &expected_program_data {
        return Err(GameError::InvalidSeeds.into());
    }

    drop(program_data_bytes);

    // Parse instruction data
    if data.len() < core::mem::size_of::<ForceUpdateAuthorityData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let update_data = unsafe { &*(data.as_ptr() as *const ForceUpdateAuthorityData) };

    // Load and update config
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;

    // Update all configurable fields
    config.authority = update_data.new_authority;
    config.backend_authority = update_data.new_backend_authority;
    config.titan_program = update_data.new_titan_program;
    config.breach_mint = update_data.new_breach_mint;
    config.reward_pool = update_data.new_reward_pool;

    Ok(())
}
