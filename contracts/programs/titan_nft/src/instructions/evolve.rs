//! Evolve Titan instruction

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{error::TitanError, state::TitanData};

/// Evolve instruction data
#[repr(C, packed)]
pub struct EvolveData {
    /// New species ID after evolution
    pub new_species_id: u16,
}

/// Minimum level required for evolution
const EVOLUTION_MIN_LEVEL: u8 = 30;

/// Process evolve instruction
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [owner, titan_account] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate owner is signer
    if !owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Parse instruction data
    if data.len() < core::mem::size_of::<EvolveData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let evolve_data = unsafe { &*(data.as_ptr() as *const EvolveData) };

    // Load Titan
    let mut titan_data = titan_account.try_borrow_mut_data()?;
    let titan = TitanData::from_account_data_mut(&mut titan_data)?;

    // Check minimum level for evolution
    if titan.level < EVOLUTION_MIN_LEVEL {
        return Err(TitanError::CannotEvolve.into());
    }

    // Evolution upgrades the species
    titan.species_id = evolve_data.new_species_id;

    // Boost link strength on evolution
    titan.link_strength = titan.link_strength.saturating_add(10).min(100);

    Ok(())
}
