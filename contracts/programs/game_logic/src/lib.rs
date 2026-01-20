//! BREACH Game Logic Program
//! 
//! Handles capture validation, battle records, experience distribution, and rewards.
//! Interacts with the Titan NFT Program via CPI.

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

// Program ID (Devnet)
pub const PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("DLk2GnDu9AYn7PeLprEDHDYH9UWKENX47UqqfeiQBaSX");

// Titan NFT Program ID (for CPI)
pub const TITAN_NFT_PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7");

entrypoint!(process_instruction);

/// Program entry point
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse instruction discriminator (first byte)
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        // === Game Logic Instructions ===
        0 => initialize::process(program_id, accounts, data),
        1 => record_capture::process(program_id, accounts, data),
        2 => record_battle::process(program_id, accounts, data),
        3 => add_experience::process(program_id, accounts, data),
        4 => distribute_reward::process(program_id, accounts, data),
        5 => update_config::process(program_id, accounts, data),
        6 => set_paused::process(program_id, accounts, data),
        7 => force_update_authority::process(program_id, accounts, data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
