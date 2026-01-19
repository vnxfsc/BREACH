//! Initialize Game Logic Program

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::error::GameError;
use crate::state::GameConfig;

/// Initialize instruction data
#[repr(C, packed)]
pub struct InitializeData {
    /// Backend authority public key
    pub backend_authority: Pubkey,
    /// Titan NFT Program ID
    pub titan_program: Pubkey,
    /// $BREACH token mint
    pub breach_mint: Pubkey,
    /// Reward pool token account
    pub reward_pool: Pubkey,
}

/// Process initialize instruction
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts using slice pattern
    let [
        authority,      // [0] Signer, program authority
        config_account, // [1] Config PDA (to be created)
        _system_program, // [2] System Program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify authority is signer
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse instruction data
    if data.len() < core::mem::size_of::<InitializeData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let init_data = unsafe { &*(data.as_ptr() as *const InitializeData) };

    // Derive config PDA and verify
    let (expected_config, bump) = pinocchio::pubkey::find_program_address(
        &[GameConfig::SEED],
        program_id,
    );
    if config_account.key() != &expected_config {
        return Err(GameError::InvalidSeeds.into());
    }

    // Check if already initialized
    let config_data_borrowed = config_account.try_borrow_data()?;
    let already_initialized = config_data_borrowed.len() >= 8 
        && config_data_borrowed[0..8] == GameConfig::DISCRIMINATOR;
    drop(config_data_borrowed);
    
    if already_initialized {
        return Err(GameError::AlreadyInitialized.into());
    }

    // Create config account via CPI if it doesn't exist
    let lamports = config_account.lamports();
    if lamports == 0 {
        // Calculate rent
        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(GameConfig::SIZE);

        // Build signer seeds
        let bump_seed = [bump];
        let signer_seeds: [Seed; 2] = [
            Seed::from(GameConfig::SEED),
            Seed::from(&bump_seed),
        ];
        let signer = Signer::from(&signer_seeds);

        // Create config account via CPI
        CreateAccount {
            from: authority,
            to: config_account,
            lamports: rent_lamports,
            space: GameConfig::SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize config data
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;

    config.discriminator = GameConfig::DISCRIMINATOR;
    config.authority = *authority.key();
    config.backend_authority = init_data.backend_authority;
    config.titan_program = init_data.titan_program;
    config.breach_mint = init_data.breach_mint;
    config.reward_pool = init_data.reward_pool;
    config.exp_multiplier = 100; // 1x multiplier
    config.battle_cooldown_seconds = 300; // 5 minutes
    config.capture_validity_seconds = 60; // 1 minute validity
    config.battle_reward_base = 1000; // 0.000001 SOL
    config.capture_reward_base = 5000; // 0.000005 SOL
    config.paused = false;
    config.bump = bump;
    config.total_battles = 0;
    config.total_captures = 0;
    config.total_exp_distributed = 0;
    config.total_rewards_distributed = 0;

    Ok(())
}
