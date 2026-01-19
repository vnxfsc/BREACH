//! Add Experience instruction
//! 
//! Adds experience to a Titan via CPI to Titan NFT Program

use pinocchio::{
    account_info::AccountInfo,
    cpi,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::error::GameError;
use crate::state::GameConfig;

/// Titan NFT Program ID (deployed on Devnet)
const TITAN_PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7");

/// Add experience instruction discriminator in Titan NFT Program
const ADD_EXPERIENCE_DISCRIMINATOR: u8 = 8;

/// Add experience instruction data
#[repr(C, packed)]
pub struct AddExperienceData {
    /// Titan ID to add experience to
    pub titan_id: u64,
    /// Amount of experience to add
    pub exp_amount: u32,
}

/// Process add experience instruction
/// 
/// Accounts:
/// 0. `[signer]` Player (titan owner)
/// 1. `[signer]` Backend authority
/// 2. `[]` Game config PDA
/// 3. `[writable]` Titan data PDA (from Titan NFT Program)
/// 4. `[]` Titan NFT Global config PDA
/// 5. `[]` Titan NFT Program
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        player,              // [0] Signer, titan owner
        backend_authority,   // [1] Signer, backend authority
        game_config_account, // [2] Game config PDA
        titan_account,       // [3] Titan PDA (from Titan NFT Program)
        titan_config,        // [4] Titan NFT Global config PDA
        titan_program,       // [5] Titan NFT Program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signers
    if !player.is_signer() || !backend_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify Titan NFT Program ID
    if titan_program.key() != &TITAN_PROGRAM_ID {
        return Err(GameError::InvalidTitanProgram.into());
    }

    // Load game config
    let config_data = game_config_account.try_borrow_data()?;
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

    // Apply experience multiplier from config
    let multiplied_exp = (exp_data.exp_amount as u64) * (config.exp_multiplier as u64) / 100;
    let final_exp = multiplied_exp.min(u32::MAX as u64) as u32;

    // Prepare CPI instruction data: [discriminator, exp_amount (u32)]
    let mut cpi_data = Vec::with_capacity(5);
    cpi_data.push(ADD_EXPERIENCE_DISCRIMINATOR);
    cpi_data.extend_from_slice(&final_exp.to_le_bytes());

    // CPI to Titan NFT Program's add_experience instruction
    // Account order for add_experience:
    // 0. [signer] Backend authority
    // 1. [] Global config PDA
    // 2. [writable] Titan data PDA
    cpi::invoke(
        &pinocchio::instruction::Instruction {
            program_id: &TITAN_PROGRAM_ID,
            accounts: &[
                pinocchio::instruction::AccountMeta {
                    pubkey: backend_authority.key(),
                    is_signer: true,
                    is_writable: false,
                },
                pinocchio::instruction::AccountMeta {
                    pubkey: titan_config.key(),
                    is_signer: false,
                    is_writable: false,
                },
                pinocchio::instruction::AccountMeta {
                    pubkey: titan_account.key(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: &cpi_data,
        },
        &[backend_authority, titan_config, titan_account],
    )?;

    // Update game config stats
    drop(config_data);
    let mut config_data = game_config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;
    config.total_exp_distributed += final_exp as u64;

    Ok(())
}
