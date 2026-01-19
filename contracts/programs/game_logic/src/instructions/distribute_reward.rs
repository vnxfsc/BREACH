//! Distribute Reward instruction
//! 
//! Distributes $BREACH token rewards to players

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};
use pinocchio_token::instructions::Transfer;

use crate::error::GameError;
use crate::state::GameConfig;

/// Distribute reward instruction data
#[repr(C, packed)]
pub struct DistributeRewardData {
    /// Reward type (0 = capture, 1 = battle_win, 2 = daily_bonus)
    pub reward_type: u8,
    /// Base amount (will be multiplied by multipliers)
    pub amount: u64,
}

/// Reward types
#[repr(u8)]
pub enum RewardType {
    Capture = 0,
    BattleWin = 1,
    DailyBonus = 2,
}

/// Process distribute reward instruction
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Parse accounts
    let [
        backend_authority,     // [0] Signer, backend authority
        config_account,        // [1] Config PDA
        reward_pool,           // [2] Reward pool token account
        player_token_account,  // [3] Player's token account
        token_program,         // [4] SPL Token Program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify backend authority is signer
    if !backend_authority.is_signer() {
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

    // Verify reward pool
    if reward_pool.key() != &config.reward_pool {
        return Err(GameError::InvalidAccountData.into());
    }

    // Parse instruction data
    if data.len() < std::mem::size_of::<DistributeRewardData>() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let reward_data = unsafe { &*(data.as_ptr() as *const DistributeRewardData) };

    // Validate reward amount
    if reward_data.amount == 0 {
        return Err(GameError::InvalidRewardAmount.into());
    }

    // Calculate final reward based on type
    let final_amount = match reward_data.reward_type {
        0 => reward_data.amount, // Capture reward
        1 => reward_data.amount * 2, // Battle win reward (2x)
        2 => reward_data.amount * 5, // Daily bonus (5x)
        _ => return Err(GameError::InvalidRewardAmount.into()),
    };

    // Transfer tokens from reward pool to player
    // Note: This requires the config PDA to have authority over the reward pool
    Transfer {
        from: reward_pool,
        to: player_token_account,
        authority: config_account,
        amount: final_amount,
    }
    .invoke()?;

    // Suppress unused variable warning
    let _ = token_program;

    // Update config stats
    drop(config_data);
    let mut config_data = config_account.try_borrow_mut_data()?;
    let config = GameConfig::from_account_data_mut(&mut config_data)?;
    config.total_rewards_distributed += final_amount;

    Ok(())
}
