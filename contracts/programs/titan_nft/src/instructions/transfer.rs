//! Transfer Titan ownership instruction

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

use crate::{
    error::TitanError,
    state::{GlobalConfig, PlayerAccount, TitanData},
};

/// Process transfer instruction
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        from_owner,          // [0] Current owner (signer)
        _to_owner,           // [1] New owner
        config_account,      // [2] Config PDA
        titan_account,       // [3] Titan to transfer
        from_player_account, // [4] From player PDA
        to_player_account,   // [5] To player PDA
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate from_owner is signer
    if !from_owner.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Load config
    let config_data = config_account.try_borrow_data()?;
    let config = GlobalConfig::from_account_data(&config_data)?;

    // Check program not paused
    if config.paused {
        return Err(TitanError::ProgramPaused.into());
    }

    // Load to_player to check max titans
    let to_player_data = to_player_account.try_borrow_data()?;
    if to_player_data.len() >= PlayerAccount::SIZE {
        let to_player = PlayerAccount::from_account_data(&to_player_data)?;
        if to_player.titans_owned >= config.max_titans_per_wallet as u32 {
            return Err(TitanError::MaxTitansReached.into());
        }
    }
    drop(to_player_data);
    drop(config_data);

    // Load Titan - just verify it exists
    let titan_data = titan_account.try_borrow_data()?;
    let _titan = TitanData::from_account_data(&titan_data)?;
    drop(titan_data);

    // Update from_player stats
    let mut from_player_data = from_player_account.try_borrow_mut_data()?;
    if from_player_data.len() >= PlayerAccount::SIZE {
        let from_player = PlayerAccount::from_account_data_mut(&mut from_player_data)?;
        from_player.titans_owned = from_player.titans_owned.saturating_sub(1);
    }
    drop(from_player_data);

    // Update to_player stats
    let mut to_player_data = to_player_account.try_borrow_mut_data()?;
    if to_player_data.len() >= PlayerAccount::SIZE {
        let to_player = PlayerAccount::from_account_data_mut(&mut to_player_data)?;
        to_player.titans_owned += 1;
    }

    Ok(())
}
