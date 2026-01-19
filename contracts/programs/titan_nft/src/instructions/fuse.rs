//! Fuse two Titans instruction

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{clock::Clock, Sysvar},
    ProgramResult,
};

use crate::{
    error::TitanError,
    state::{GlobalConfig, TitanData},
    utils::genes::calculate_offspring_genes,
};

/// Minimum level required for fusion
const FUSION_MIN_LEVEL: u8 = 20;

/// Process fuse instruction
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        owner,              // [0] Signer, owner of both Titans
        config_account,     // [1] Config PDA
        titan_a_account,    // [2] Parent Titan A
        titan_b_account,    // [3] Parent Titan B
        offspring_account,  // [4] New offspring Titan PDA
        _system_program,    // [5] System program
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate owner is signer
    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load config
    let config_data = config_account.try_borrow_data()?;
    let config = GlobalConfig::from_account_data(&config_data)?;

    // Check program not paused
    if config.paused {
        return Err(TitanError::ProgramPaused.into());
    }

    let total_minted = config.total_titans_minted;
    drop(config_data);

    // Load parent Titans
    let titan_a_data = titan_a_account.try_borrow_data()?;
    let titan_a = TitanData::from_account_data(&titan_a_data)?;

    let titan_b_data = titan_b_account.try_borrow_data()?;
    let titan_b = TitanData::from_account_data(&titan_b_data)?;

    // Validate not fusing with self
    if titan_a.titan_id == titan_b.titan_id {
        return Err(TitanError::CannotFuseWithSelf.into());
    }

    // Check minimum levels
    if titan_a.level < FUSION_MIN_LEVEL || titan_b.level < FUSION_MIN_LEVEL {
        return Err(TitanError::LevelTooLowForFusion.into());
    }

    // Check same element type (optional rule)
    if titan_a.element_type != titan_b.element_type {
        return Err(TitanError::ElementMismatch.into());
    }

    let clock = Clock::get()?;

    // Get offspring ID
    let offspring_id = total_minted + 1;

    // Derive offspring PDA
    let offspring_id_bytes = offspring_id.to_le_bytes();
    let (offspring_pda, offspring_bump) = pinocchio::pubkey::find_program_address(
        &[TitanData::SEED_PREFIX, &offspring_id_bytes],
        program_id,
    );

    if offspring_account.key() != &offspring_pda {
        return Err(TitanError::InvalidSeeds.into());
    }

    // Generate offspring genes using randomness from clock
    let randomness = generate_randomness(clock.unix_timestamp, titan_a.titan_id, titan_b.titan_id);
    let offspring_genes = calculate_offspring_genes(&titan_a.genes, &titan_b.genes, &randomness);

    // Calculate offspring attributes (average of parents with gene influence)
    let offspring_power = calculate_offspring_stat(titan_a.power, titan_b.power, offspring_genes[0]);
    let offspring_fortitude = calculate_offspring_stat(titan_a.fortitude, titan_b.fortitude, offspring_genes[2]);
    let offspring_velocity = calculate_offspring_stat(titan_a.velocity, titan_b.velocity, offspring_genes[1]);
    let offspring_resonance = calculate_offspring_stat(titan_a.resonance, titan_b.resonance, offspring_genes[4]);

    // Offspring threat class is the lower of the two parents
    let offspring_class = titan_a.threat_class.min(titan_b.threat_class);
    let offspring_element = titan_a.element_type;
    let offspring_species = titan_a.species_id;
    let offspring_generation = titan_a.generation.max(titan_b.generation) + 1;
    let parent_a_id = titan_a.titan_id;
    let parent_b_id = titan_b.titan_id;

    drop(titan_a_data);
    drop(titan_b_data);

    // Note: Offspring account should be created by client before this instruction

    // Initialize offspring
    let mut offspring_data = offspring_account.try_borrow_mut_data()?;
    
    if offspring_data.len() < TitanData::SIZE {
        return Err(TitanError::AccountDataTooSmall.into());
    }
    
    let offspring = TitanData::init_from_account_data(&mut offspring_data)?;

    offspring.titan_id = offspring_id;
    offspring.species_id = offspring_species;
    offspring.threat_class = offspring_class;
    offspring.element_type = offspring_element;
    offspring.power = offspring_power;
    offspring.fortitude = offspring_fortitude;
    offspring.velocity = offspring_velocity;
    offspring.resonance = offspring_resonance;
    offspring.genes = offspring_genes;
    offspring.level = 1;
    offspring.experience = 0;
    offspring.link_strength = 20; // Fusion offspring start with higher link
    offspring.captured_at = clock.unix_timestamp;
    offspring.original_owner = *owner.key();
    offspring.capture_location = 0; // No capture location for fusion
    offspring.generation = offspring_generation;
    offspring.parent_a = parent_a_id;
    offspring.parent_b = parent_b_id;
    offspring.bump = offspring_bump;

    drop(offspring_data);

    // Update config stats
    let mut config_data_mut = config_account.try_borrow_mut_data()?;
    let config = GlobalConfig::from_account_data_mut(&mut config_data_mut)?;
    config.total_titans_minted = offspring_id;
    config.total_fusions += 1;

    Ok(())
}

/// Generate pseudo-randomness from clock and titan IDs
fn generate_randomness(timestamp: i64, id_a: u64, id_b: u64) -> [u8; 32] {
    let mut result = [0u8; 32];
    let combined = timestamp as u64 ^ id_a ^ id_b;
    
    for i in 0..32 {
        result[i] = ((combined >> (i % 8)) ^ (i as u64 * 17)) as u8;
    }
    
    result
}

/// Calculate offspring stat from parents and gene
fn calculate_offspring_stat(parent_a: u8, parent_b: u8, gene: u8) -> u8 {
    let avg = ((parent_a as u16 + parent_b as u16) / 2) as u8;
    let gene_bonus = (gene as i16 - 128) / 25; // -5 to +5 based on gene
    (avg as i16 + gene_bonus).clamp(1, 255) as u8
}
