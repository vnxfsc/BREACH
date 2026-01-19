//! Record Battle instruction

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
use crate::state::{BattleRecord, GameConfig};

/// Record battle instruction data
#[repr(C, packed)]
pub struct RecordBattleData {
    /// Player A's Titan ID
    pub titan_a_id: u64,
    /// Player B's Titan ID
    pub titan_b_id: u64,
    /// Winner (0 = Player A, 1 = Player B, 2 = Draw)
    pub winner: u8,
    /// Experience gained by Player A's Titan
    pub exp_gained_a: u32,
    /// Experience gained by Player B's Titan
    pub exp_gained_b: u32,
    /// Battle location (lat * 1e6)
    pub location_lat: i32,
    /// Battle location (lng * 1e6)
    pub location_lng: i32,
}

/// Process record battle instruction
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        player_a,          // [0] Signer, player A wallet
        player_b,          // [1] Player B wallet
        backend_authority, // [2] Signer, backend authority
        config_account,    // [3] Config PDA
        battle_record,     // [4] Battle record PDA (to be created)
        _system_program,   // [5] System Program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify signers
    if !player_a.is_signer() || !backend_authority.is_signer() {
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
    if data.len() < core::mem::size_of::<RecordBattleData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let battle_data = unsafe { &*(data.as_ptr() as *const RecordBattleData) };

    // Validate: cannot battle self
    if player_a.key() == player_b.key() {
        return Err(GameError::CannotBattleSelf.into());
    }

    // Validate: cannot use same titan
    if battle_data.titan_a_id == battle_data.titan_b_id {
        return Err(GameError::CannotBattleSelf.into());
    }

    // Validate winner value
    if battle_data.winner > 2 {
        return Err(GameError::InvalidAccountData.into());
    }

    // Get current timestamp
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    // Get battle ID from config (total_battles + 1)
    let battle_id = config.total_battles + 1;

    // Derive battle record PDA
    let battle_id_bytes = battle_id.to_le_bytes();
    let (expected_battle_pda, bump) = pinocchio::pubkey::find_program_address(
        &[BattleRecord::SEED, &battle_id_bytes],
        program_id,
    );

    if battle_record.key() != &expected_battle_pda {
        return Err(GameError::InvalidSeeds.into());
    }

    // Check if already recorded
    let record_data_borrowed = battle_record.try_borrow_data()?;
    let already_recorded = record_data_borrowed.len() >= 8 
        && record_data_borrowed[0..8] == BattleRecord::DISCRIMINATOR;
    drop(record_data_borrowed);

    if already_recorded {
        return Err(GameError::BattleAlreadyRecorded.into());
    }

    // Create battle record account if needed
    let lamports = battle_record.lamports();
    if lamports == 0 {
        // Calculate rent
        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(BattleRecord::SIZE);

        // Build signer seeds
        let bump_seed = [bump];
        let signer_seeds: [Seed; 3] = [
            Seed::from(BattleRecord::SEED),
            Seed::from(battle_id_bytes.as_slice()),
            Seed::from(&bump_seed),
        ];
        let signer = Signer::from(&signer_seeds);

        // Create battle record account
        CreateAccount {
            from: player_a,
            to: battle_record,
            lamports: rent_lamports,
            space: BattleRecord::SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize battle record
    let mut record_data = battle_record.try_borrow_mut_data()?;
    let record = BattleRecord::from_account_data_mut(&mut record_data)?;

    record.discriminator = BattleRecord::DISCRIMINATOR;
    record.battle_id = battle_id;
    record.player_a = *player_a.key();
    record.titan_a_id = battle_data.titan_a_id;
    record.player_b = *player_b.key();
    record.titan_b_id = battle_data.titan_b_id;
    record.winner = battle_data.winner;
    record.exp_gained_a = battle_data.exp_gained_a;
    record.exp_gained_b = battle_data.exp_gained_b;
    record.timestamp = current_timestamp;
    record.location_lat = battle_data.location_lat;
    record.location_lng = battle_data.location_lng;
    record.bump = bump;

    // Calculate total exp distributed
    let total_exp = (battle_data.exp_gained_a as u64) + (battle_data.exp_gained_b as u64);

    // Update config stats
    drop(config_data);
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;
    config.total_battles += 1;
    config.total_exp_distributed += total_exp;

    Ok(())
}
