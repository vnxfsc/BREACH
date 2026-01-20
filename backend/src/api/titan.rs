//! Titan operations API
//!
//! Provides on-chain Titan operations: level up, evolve, fuse, transfer.

use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};
use crate::middleware::auth::AuthPlayer;
use crate::AppState;

// ═══════════════════════════════════════════════════════════════════════════════
// Level Up API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct LevelUpRequest {
    /// On-chain Titan ID
    pub titan_id: u64,
}

#[derive(Debug, Serialize)]
pub struct BuildTransactionResponse {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
}

/// Build Level Up transaction
async fn build_level_up(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<LevelUpRequest>,
) -> ApiResult<Json<BuildTransactionResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.build_level_up_transaction(
        &player.wallet_address,
        request.titan_id,
    ).await?;

    Ok(Json(BuildTransactionResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Evolve API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct EvolveRequest {
    /// On-chain Titan ID
    pub titan_id: u64,
    /// New species ID after evolution
    pub new_species_id: u16,
}

/// Build Evolve transaction
async fn build_evolve(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<EvolveRequest>,
) -> ApiResult<Json<BuildTransactionResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.build_evolve_transaction(
        &player.wallet_address,
        request.titan_id,
        request.new_species_id,
    ).await?;

    Ok(Json(BuildTransactionResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fuse API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct FuseRequest {
    /// First Titan on-chain ID
    pub titan_a_id: u64,
    /// Second Titan on-chain ID
    pub titan_b_id: u64,
}

#[derive(Debug, Serialize)]
pub struct FuseTransactionResponse {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    /// Newly created Titan ID
    pub offspring_id: u64,
    /// New Titan PDA address
    pub offspring_pda: String,
}

/// Build Fuse transaction
async fn build_fuse(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<FuseRequest>,
) -> ApiResult<Json<FuseTransactionResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.build_fuse_transaction(
        &player.wallet_address,
        request.titan_a_id,
        request.titan_b_id,
    ).await?;

    Ok(Json(FuseTransactionResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
        offspring_id: result.offspring_id,
        offspring_pda: result.offspring_pda,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Transfer API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    /// On-chain Titan ID
    pub titan_id: u64,
    /// Recipient wallet address
    pub to_wallet: String,
}

/// Build Transfer transaction
async fn build_transfer(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<TransferRequest>,
) -> ApiResult<Json<BuildTransactionResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.build_transfer_transaction(
        &player.wallet_address,
        &request.to_wallet,
        request.titan_id,
    ).await?;

    Ok(Json(BuildTransactionResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Submit signed transaction API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    /// Base64-encoded serialized transaction
    pub serialized_transaction: String,
    /// Base64-encoded user signature
    pub user_signature: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitTransactionResponse {
    pub success: bool,
    pub tx_signature: String,
}

/// Submit user-signed transaction
async fn submit_transaction(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<SubmitTransactionRequest>,
) -> ApiResult<Json<SubmitTransactionResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    let result = solana.submit_user_signed_transaction(
        &request.serialized_transaction,
        &request.user_signature,
        &player.wallet_address,
    ).await?;

    Ok(Json(SubmitTransactionResponse {
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
        .route("/titan/level-up/build", post(build_level_up))
        .route("/titan/evolve/build", post(build_evolve))
        .route("/titan/fuse/build", post(build_fuse))
        .route("/titan/transfer/build", post(build_transfer))
        // Submit transaction endpoint
        .route("/titan/submit", post(submit_transaction))
        .with_state(state)
}
