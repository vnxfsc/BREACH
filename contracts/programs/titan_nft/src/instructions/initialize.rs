//! Initialize program instruction

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use crate::{error::TitanError, state::GlobalConfig};

/// Initialize instruction data
#[repr(C, packed)]
pub struct InitializeData {
    /// Treasury wallet address
    pub treasury: Pubkey,
    /// $BREACH token mint address
    pub breach_mint: Pubkey,
    /// Backend capture authority
    pub capture_authority: Pubkey,
    /// Capture fee in basis points
    pub capture_fee_bps: u16,
    /// Marketplace fee in basis points
    pub marketplace_fee_bps: u16,
    /// Fusion fee in basis points
    pub fusion_fee_bps: u16,
    /// Max Titans per wallet
    pub max_titans_per_wallet: u16,
    /// Capture cooldown in seconds
    pub capture_cooldown_seconds: u32,
}

/// Process initialize instruction
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [authority, config_account, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate authority is signer
    if !authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse instruction data
    if data.len() < core::mem::size_of::<InitializeData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let init_data = unsafe { &*(data.as_ptr() as *const InitializeData) };

    // Derive config PDA and get bump
    let (config_pda, bump) = pinocchio::pubkey::find_program_address(
        &[GlobalConfig::SEED],
        program_id,
    );

    if config_account.key() != &config_pda {
        return Err(TitanError::InvalidSeeds.into());
    }

    // Check if already initialized
    let config_data_borrowed = config_account.try_borrow_data()?;
    let already_initialized = config_data_borrowed.len() >= 8 
        && config_data_borrowed[0..8] == GlobalConfig::DISCRIMINATOR;
    drop(config_data_borrowed);
    
    if already_initialized {
        return Err(TitanError::AlreadyInitialized.into());
    }

    // Create config account via CPI if it doesn't exist
    let lamports = config_account.lamports();
    if lamports == 0 {
        // Calculate rent
        let rent = Rent::get()?;
        let required_lamports = rent.minimum_balance(GlobalConfig::SIZE);

        // Create account via CPI using pinocchio-system
        let bump_slice = [bump];
        let seeds: [Seed; 2] = [
            Seed::from(GlobalConfig::SEED),
            Seed::from(&bump_slice),
        ];
        let signer = Signer::from(&seeds);
        
        pinocchio_system::instructions::CreateAccount {
            from: authority,
            to: config_account,
            lamports: required_lamports,
            space: GlobalConfig::SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize config data
    let mut config_data_mut = config_account.try_borrow_mut_data()?;
    
    if config_data_mut.len() < GlobalConfig::SIZE {
        return Err(TitanError::AccountDataTooSmall.into());
    }
    
    let config = GlobalConfig::init_from_account_data(&mut config_data_mut)?;

    config.authority = *authority.key();
    config.treasury = init_data.treasury;
    config.breach_mint = init_data.breach_mint;
    config.capture_authority = init_data.capture_authority;
    config.capture_fee_bps = init_data.capture_fee_bps;
    config.marketplace_fee_bps = init_data.marketplace_fee_bps;
    config.fusion_fee_bps = init_data.fusion_fee_bps;
    config.max_titans_per_wallet = init_data.max_titans_per_wallet;
    config.capture_cooldown_seconds = init_data.capture_cooldown_seconds;
    config.paused = false;
    config.bump = bump;
    config.total_titans_minted = 0;
    config.total_battles = 0;
    config.total_fusions = 0;
    config.total_fees_collected = 0;

    Ok(())
}
