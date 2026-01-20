//! Titan 操作 API
//!
//! 提供 Titan 的链上操作：升级、进化、融合、转移

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
    /// 链上 Titan ID
    pub titan_id: u64,
}

#[derive(Debug, Serialize)]
pub struct BuildTransactionResponse {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
}

/// 构建 Level Up 交易
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
    /// 链上 Titan ID
    pub titan_id: u64,
    /// 新的物种 ID
    pub new_species_id: u16,
}

/// 构建 Evolve 交易
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
    /// 第一个 Titan 的链上 ID
    pub titan_a_id: u64,
    /// 第二个 Titan 的链上 ID
    pub titan_b_id: u64,
}

#[derive(Debug, Serialize)]
pub struct FuseTransactionResponse {
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
    /// 新生成的 Titan ID
    pub offspring_id: u64,
    /// 新 Titan 的 PDA 地址
    pub offspring_pda: String,
}

/// 构建 Fuse 交易
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
    /// 链上 Titan ID
    pub titan_id: u64,
    /// 接收者钱包地址
    pub to_wallet: String,
}

/// 构建 Transfer 交易
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
// 提交签名交易 API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    /// Base64 编码的交易
    pub serialized_transaction: String,
    /// Base64 编码的用户签名
    pub user_signature: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitTransactionResponse {
    pub success: bool,
    pub tx_signature: String,
}

/// 提交用户签名的交易
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
// 路由
// ═══════════════════════════════════════════════════════════════════════════════

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // 构建交易端点
        .route("/titan/level-up/build", post(build_level_up))
        .route("/titan/evolve/build", post(build_evolve))
        .route("/titan/fuse/build", post(build_fuse))
        .route("/titan/transfer/build", post(build_transfer))
        // 提交交易端点
        .route("/titan/submit", post(submit_transaction))
        .with_state(state)
}
