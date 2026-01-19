//! Mint Titan NFT instruction

use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    ProgramResult,
};

use crate::{
    error::TitanError,
    state::{GlobalConfig, PlayerAccount, TitanData},
};

/// Mint Titan instruction data
#[repr(C, packed)]
pub struct MintTitanData {
    /// Species ID
    pub species_id: u16,
    /// Threat class (1-5)
    pub threat_class: u8,
    /// Element type (0-5)
    pub element_type: u8,
    /// Power attribute
    pub power: u8,
    /// Fortitude attribute
    pub fortitude: u8,
    /// Velocity attribute
    pub velocity: u8,
    /// Resonance attribute
    pub resonance: u8,
    /// Gene sequence
    pub genes: [u8; 6],
    /// Capture latitude (scaled by 10^6)
    pub capture_lat: i32,
    /// Capture longitude (scaled by 10^6)
    pub capture_lng: i32,
    /// Nonce for signature verification
    pub nonce: u64,
    /// Backend signature (64 bytes)
    pub signature: [u8; 64],
}

/// Process mint_titan instruction
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        payer,              // [0] Signer, player
        config_account,     // [1] Config PDA
        player_account,     // [2] Player PDA
        titan_account,      // [3] New Titan PDA
        capture_authority,  // [4] Backend signer
        _system_program,    // [5] System program
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate signers
    if !payer.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !capture_authority.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load config
    let config_data = config_account.try_borrow_data()?;
    let config = GlobalConfig::from_account_data(&config_data)?;

    // Check program not paused
    if config.paused {
        return Err(TitanError::ProgramPaused.into());
    }

    // Validate capture authority
    if capture_authority.key().as_ref() != config.capture_authority.as_ref() {
        return Err(TitanError::InvalidCaptureAuthority.into());
    }

    // Parse instruction data
    if data.len() < core::mem::size_of::<MintTitanData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mint_data = unsafe { &*(data.as_ptr() as *const MintTitanData) };

    // Validate threat class (1-5)
    if mint_data.threat_class < 1 || mint_data.threat_class > 5 {
        return Err(TitanError::InvalidThreatClass.into());
    }

    // Validate element type (0-5)
    if mint_data.element_type > 5 {
        return Err(TitanError::InvalidElementType.into());
    }

    let clock = Clock::get()?;
    let rent = Rent::get()?;
    let total_minted = config.total_titans_minted;
    let cooldown = config.capture_cooldown_seconds;
    let max_titans = config.max_titans_per_wallet;
    drop(config_data);

    // Derive Player PDA
    let (player_pda, player_bump) = pinocchio::pubkey::find_program_address(
        &[PlayerAccount::SEED_PREFIX, payer.key().as_ref()],
        program_id,
    );
    
    if player_account.key() != &player_pda {
        return Err(TitanError::InvalidSeeds.into());
    }

    // Create player account if needed
    if player_account.lamports() == 0 {
        let required_lamports = rent.minimum_balance(PlayerAccount::SIZE);
        let bump_slice = [player_bump];
        let seeds: [Seed; 3] = [
            Seed::from(PlayerAccount::SEED_PREFIX),
            Seed::from(payer.key().as_ref()),
            Seed::from(&bump_slice),
        ];
        let signer = Signer::from(&seeds);
        
        pinocchio_system::instructions::CreateAccount {
            from: payer,
            to: player_account,
            lamports: required_lamports,
            space: PlayerAccount::SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[signer])?;
        
        // Initialize player
        let mut player_data = player_account.try_borrow_mut_data()?;
        let player = PlayerAccount::init_from_account_data(&mut player_data)?;
        player.wallet = *payer.key();
        player.elo_rating = PlayerAccount::INITIAL_ELO;
        player.peak_elo = PlayerAccount::INITIAL_ELO;
        player.created_at = clock.unix_timestamp;
        drop(player_data);
    }

    // Load player
    let mut player_data = player_account.try_borrow_mut_data()?;
    let player = PlayerAccount::from_account_data_mut(&mut player_data)?;

    // Check capture cooldown
    if !player.can_capture(clock.unix_timestamp, cooldown) {
        return Err(TitanError::CaptureCooldown.into());
    }

    // Check max titans
    if player.titans_owned >= max_titans as u32 {
        return Err(TitanError::MaxTitansReached.into());
    }

    // Get new titan ID
    let titan_id = total_minted + 1;

    // Derive Titan PDA
    let titan_id_bytes = titan_id.to_le_bytes();
    let (titan_pda, titan_bump) = pinocchio::pubkey::find_program_address(
        &[TitanData::SEED_PREFIX, &titan_id_bytes],
        program_id,
    );

    if titan_account.key() != &titan_pda {
        return Err(TitanError::InvalidSeeds.into());
    }

    // Create Titan account
    if titan_account.lamports() == 0 {
        let required_lamports = rent.minimum_balance(TitanData::SIZE);
        let bump_slice = [titan_bump];
        let seeds: [Seed; 3] = [
            Seed::from(TitanData::SEED_PREFIX),
            Seed::from(&titan_id_bytes[..]),
            Seed::from(&bump_slice),
        ];
        let signer = Signer::from(&seeds);
        
        pinocchio_system::instructions::CreateAccount {
            from: payer,
            to: titan_account,
            lamports: required_lamports,
            space: TitanData::SIZE as u64,
            owner: program_id,
        }
        .invoke_signed(&[signer])?;
    }

    // Initialize Titan data
    let mut titan_data_mut = titan_account.try_borrow_mut_data()?;
    let titan = TitanData::init_from_account_data(&mut titan_data_mut)?;

    titan.titan_id = titan_id;
    titan.species_id = mint_data.species_id;
    titan.threat_class = mint_data.threat_class;
    titan.element_type = mint_data.element_type;
    titan.power = mint_data.power;
    titan.fortitude = mint_data.fortitude;
    titan.velocity = mint_data.velocity;
    titan.resonance = mint_data.resonance;
    titan.genes = mint_data.genes;
    titan.level = 1;
    titan.experience = 0;
    titan.link_strength = 10; // Initial link strength
    titan.captured_at = clock.unix_timestamp;
    titan.original_owner = *payer.key();
    titan.owner = *payer.key();  // Current owner is the initial capturer
    titan.capture_location = encode_location(mint_data.capture_lat, mint_data.capture_lng);
    titan.generation = 0;
    titan.parent_a = 0;
    titan.parent_b = 0;
    titan.bump = titan_bump;

    drop(titan_data_mut);

    // Update player stats
    player.titans_captured += 1;
    player.titans_owned += 1;
    player.last_capture_at = clock.unix_timestamp;
    drop(player_data);

    // Update config stats
    let mut config_data_mut = config_account.try_borrow_mut_data()?;
    let config = GlobalConfig::from_account_data_mut(&mut config_data_mut)?;
    config.total_titans_minted = titan_id;

    Ok(())
}

/// Encode latitude and longitude into a single u64
fn encode_location(lat: i32, lng: i32) -> u64 {
    ((lat as u64) << 32) | (lng as u64 & 0xFFFFFFFF)
}
