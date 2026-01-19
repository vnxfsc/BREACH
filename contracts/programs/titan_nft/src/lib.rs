//! BREACH Titan NFT Program
//! 
//! This program handles:
//! - Titan NFT minting
//! - Titan attributes and evolution
//! - Titan fusion mechanics
//! - Level up system

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
pub mod utils;

// Program ID (generated keypair: target/deploy/titan_nft-keypair.json)
pub const PROGRAM_ID: Pubkey = pinocchio_pubkey::pubkey!("3KYPXMcodPCbnWLDX41yWtgxe6ctsPdnT3fYgp8udmd7");

// Entrypoint
entrypoint!(process_instruction);

/// Process incoming instructions
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
        // Initialize program
        0 => instructions::initialize::process(program_id, accounts, data),
        
        // Mint new Titan
        1 => instructions::mint_titan::process(program_id, accounts, data),
        
        // Level up Titan
        2 => instructions::level_up::process(program_id, accounts, data),
        
        // Evolve Titan
        3 => instructions::evolve::process(program_id, accounts, data),
        
        // Fuse two Titans
        4 => instructions::fuse::process(program_id, accounts, data),
        
        // Transfer Titan
        5 => instructions::transfer::process(program_id, accounts, data),
        
        // Update config (admin only)
        6 => instructions::update_config::process(program_id, accounts, data),
        
        // Pause/unpause program (admin only)
        7 => instructions::set_paused::process(program_id, accounts, data),
        
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
