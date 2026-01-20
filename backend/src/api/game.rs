//! Game Logic on-chain operations API
//!
//! Provides capture records, battle records, and experience distribution operations.

use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};
use crate::middleware::auth::AuthPlayer;
use crate::AppState;

// ═══════════════════════════════════════════════════════════════════════════════
// Record Capture API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct RecordCaptureRequest {
    /// On-chain Titan ID
    pub titan_id: u64,
    /// Capture latitude (*1e6)
    pub location_lat: i32,
    /// Capture longitude (*1e6)
    pub location_lng: i32,
    /// Threat class (1-5)
    pub threat_class: u8,
    /// Element type (0-5)
    pub element_type: u8,
}

#[derive(Debug, Serialize)]
pub struct RecordCaptureResponse {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    pub capture_id: u64,
    pub capture_record_pda: String,
}

/// Build Record Capture transaction
async fn build_record_capture(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<RecordCaptureRequest>,
) -> ApiResult<Json<RecordCaptureResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.record_capture_onchain(
        &player.wallet_address,
        request.titan_id,
        request.location_lat,
        request.location_lng,
        request.threat_class,
        request.element_type,
    ).await?;

    Ok(Json(RecordCaptureResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
        capture_id: result.capture_id,
        capture_record_pda: result.capture_record_pda,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Record Battle API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct RecordBattleRequest {
    /// Opponent wallet address
    pub opponent_wallet: String,
    /// Player's Titan on-chain ID
    pub titan_id: u64,
    /// Opponent's Titan on-chain ID
    pub opponent_titan_id: u64,
    /// Winner (0 = player, 1 = opponent, 2 = draw)
    pub winner: u8,
    /// Experience gained by player's Titan
    pub exp_gained: u32,
    /// Experience gained by opponent's Titan
    pub opponent_exp_gained: u32,
    /// Battle latitude (*1e6)
    pub location_lat: i32,
    /// Battle longitude (*1e6)
    pub location_lng: i32,
}

#[derive(Debug, Serialize)]
pub struct RecordBattleResponse {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    pub battle_id: u64,
    pub battle_record_pda: String,
}

/// Build Record Battle transaction
async fn build_record_battle(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<RecordBattleRequest>,
) -> ApiResult<Json<RecordBattleResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.record_battle_onchain(
        &player.wallet_address,
        &request.opponent_wallet,
        request.titan_id,
        request.opponent_titan_id,
        request.winner,
        request.exp_gained,
        request.opponent_exp_gained,
        request.location_lat,
        request.location_lng,
    ).await?;

    Ok(Json(RecordBattleResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
        battle_id: result.battle_id,
        battle_record_pda: result.battle_record_pda,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Add Experience API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct AddExperienceRequest {
    /// On-chain Titan ID
    pub titan_id: u64,
    /// Amount of experience to add
    pub exp_amount: u32,
}

#[derive(Debug, Serialize)]
pub struct AddExperienceResponse {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    pub titan_id: u64,
    pub exp_amount: u32,
}

/// Build Add Experience transaction
async fn build_add_experience(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<AddExperienceRequest>,
) -> ApiResult<Json<AddExperienceResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.add_experience_onchain(
        &player.wallet_address,
        request.titan_id,
        request.exp_amount,
    ).await?;

    Ok(Json(AddExperienceResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
        titan_id: result.titan_id,
        exp_amount: result.exp_amount,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Distribute Reward API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct DistributeRewardRequest {
    /// Reward type (0=capture, 1=battle_win, 2=daily_bonus)
    pub reward_type: u8,
    /// Base amount (in lamports)
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct DistributeRewardResponse {
    pub success: bool,
    pub tx_signature: String,
    pub amount: u64,
    pub reward_type: u8,
}

/// Distribute BREACH token rewards (backend direct transfer).
///
/// Note: currently uses direct backend transfers instead of the on-chain `game_logic` contract
/// because the reward pool account is not yet initialized.
async fn distribute_reward(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<DistributeRewardRequest>,
) -> ApiResult<Json<DistributeRewardResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    // Validate reward type
    if request.reward_type > 2 {
        return Err(AppError::BadRequest("Invalid reward type".to_string()));
    }

    // Validate amount
    if request.amount == 0 {
        return Err(AppError::BadRequest("Amount must be greater than 0".to_string()));
    }

    // Calculate reward multiplier (must match on-chain logic)
    let multiplier = match request.reward_type {
        0 => 1,  // Capture
        1 => 2,  // Battle Win (2x)
        2 => 5,  // Daily Bonus (5x)
        _ => 1,
    };
    let final_amount = request.amount * multiplier;

    // Use transfer_breach_tokens directly (bypassing on-chain `distribute_reward`)
    let result = solana.transfer_breach_tokens(
        &player.wallet_address,
        final_amount,
    ).await?;

    Ok(Json(DistributeRewardResponse {
        success: true,
        tx_signature: result.signature,
        amount: final_amount,  // Actual amount distributed (after applying multiplier)
        reward_type: request.reward_type,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Submit dual-signed transaction API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct SubmitDualSignedRequest {
    /// Base64-encoded serialized transaction
    pub serialized_transaction: String,
    /// Base64-encoded player signature
    pub player_signature: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitDualSignedResponse {
    pub success: bool,
    pub tx_signature: String,
}

/// Submit dual-signed transaction (player + backend)
async fn submit_dual_signed(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<SubmitDualSignedRequest>,
) -> ApiResult<Json<SubmitDualSignedResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.submit_dual_signed_transaction(
        &request.serialized_transaction,
        &request.player_signature,
        &player.wallet_address,
    ).await?;

    Ok(Json(SubmitDualSignedResponse {
        success: true,
        tx_signature: result.signature,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Routes
// ═══════════════════════════════════════════════════════════════════════════════

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Build transaction endpoints
        .route("/game/capture/build", post(build_record_capture))
        .route("/game/battle/build", post(build_record_battle))
        .route("/game/experience/build", post(build_add_experience))
        // Submit transaction endpoint
        .route("/game/submit", post(submit_dual_signed))
        // Reward distribution endpoint
        .route("/game/reward/distribute", post(distribute_reward))
        .with_state(state)
}
