//! Record Capture instruction

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::error::GameError;
use crate::state::{CaptureRecord, GameConfig};

/// Record capture instruction data
#[repr(C, packed)]
pub struct RecordCaptureData {
    /// Titan ID that was captured
    pub titan_id: u64,
    /// Capture location (lat * 1e6)
    pub location_lat: i32,
    /// Capture location (lng * 1e6)
    pub location_lng: i32,
    /// Threat class of captured Titan (1-5)
    pub threat_class: u8,
    /// Element type of captured Titan (0-5)
    pub element_type: u8,
    /// Backend signature timestamp
    pub signature_timestamp: i64,
}

/// Process record capture instruction
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        player,            // [0] Signer, player wallet
        backend_authority, // [1] Signer, backend authority
        config_account,    // [2] Config PDA
        capture_record,    // [3] Capture record PDA (to be created)
        _system_program,   // [4] System Program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signers
    if !player.is_signer() || !backend_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load config
    let config_data = config_account.try_borrow_data()?;
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
    if data.len() < core::mem::size_of::<RecordCaptureData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let capture_data = unsafe { &*(data.as_ptr() as *const RecordCaptureData) };

    // Get current timestamp
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // Validate signature timestamp (must be within validity window)
    if current_timestamp - capture_data.signature_timestamp > config.capture_validity_seconds as i64 {
        return Err(GameError::CaptureExpired.into());
    }

    // Validate threat class (1-5)
    if capture_data.threat_class < 1 || capture_data.threat_class > 5 {
        return Err(GameError::InvalidAccountData.into());
    }

    // Validate element type (0-5)
    if capture_data.element_type > 5 {
        return Err(GameError::InvalidAccountData.into());
    }

    // Get capture ID from config (total_captures + 1)
    let capture_id = config.total_captures + 1;

    // Derive capture record PDA
    let capture_id_bytes = capture_id.to_le_bytes();
    let (expected_capture_pda, bump) = pinocchio::pubkey::find_program_address(
        &[CaptureRecord::SEED, &capture_id_bytes],
        program_id,
    );

    if capture_record.key() != &expected_capture_pda {
        return Err(GameError::InvalidSeeds.into());
    }

    // Check if already recorded
    let record_data_borrowed = capture_record.try_borrow_data()?;
    let already_recorded = record_data_borrowed.len() >= 8 
        && record_data_borrowed[0..8] == CaptureRecord::DISCRIMINATOR;
    drop(record_data_borrowed);

    if already_recorded {
        return Err(GameError::CaptureAlreadyRecorded.into());
    }

    // Calculate reward based on threat class
    let reward_multiplier = match capture_data.threat_class {
        1 => 100,  // Pioneer: 1x
        2 => 200,  // Hunter: 2x
        3 => 500,  // Destroyer: 5x
        4 => 1000, // Calamity: 10x
        5 => 2500, // Apex: 25x
        _ => 100,
    };
    let reward_amount = config.capture_reward_base * reward_multiplier / 100;

    // Create capture record account if needed
    let lamports = capture_record.lamports();
    if lamports == 0 {
        // Calculate rent
        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(CaptureRecord::SIZE);

        // Build signer seeds
        let bump_seed = [bump];
        let signer_seeds: [Seed; 3] = [
            Seed::from(CaptureRecord::SEED),
            Seed::from(capture_id_bytes.as_slice()),
            Seed::from(&bump_seed),
        ];
        let signer = Signer::from(&signer_seeds);

        // Create capture record account
        CreateAccount {
            from: player,
            to: capture_record,
            lamports: rent_lamports,
            space: CaptureRecord::SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize capture record
    let mut record_data = capture_record.try_borrow_mut_data()?;
    let record = CaptureRecord::from_account_data_mut(&mut record_data)?;

    record.discriminator = CaptureRecord::DISCRIMINATOR;
    record.capture_id = capture_id;
    record.player = *player.key();
    record.titan_id = capture_data.titan_id;
    record.location_lat = capture_data.location_lat;
    record.location_lng = capture_data.location_lng;
    record.timestamp = current_timestamp;
    record.threat_class = capture_data.threat_class;
    record.element_type = capture_data.element_type;
    record.reward_amount = reward_amount;
    record.bump = bump;

    // Update config stats
    drop(config_data);
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;
    config.total_captures += 1;
    config.total_rewards_distributed += reward_amount;

    Ok(())
}
