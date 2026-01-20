//! Game Logic 链上操作 API
//!
//! 提供捕获记录、战斗记录、经验值添加等链上操作

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
    /// 链上 Titan ID
    pub titan_id: u64,
    /// 捕获位置纬度 (*1e6)
    pub location_lat: i32,
    /// 捕获位置经度 (*1e6)
    pub location_lng: i32,
    /// 威胁等级 (1-5)
    pub threat_class: u8,
    /// 元素类型 (0-5)
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

/// 构建 Record Capture 交易
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
    /// 对手钱包地址
    pub opponent_wallet: String,
    /// 玩家的 Titan ID
    pub titan_id: u64,
    /// 对手的 Titan ID
    pub opponent_titan_id: u64,
    /// 获胜者 (0 = 玩家, 1 = 对手, 2 = 平局)
    pub winner: u8,
    /// 玩家 Titan 获得的经验
    pub exp_gained: u32,
    /// 对手 Titan 获得的经验
    pub opponent_exp_gained: u32,
    /// 战斗位置纬度 (*1e6)
    pub location_lat: i32,
    /// 战斗位置经度 (*1e6)
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

/// 构建 Record Battle 交易
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
    /// 链上 Titan ID
    pub titan_id: u64,
    /// 要添加的经验值
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

/// 构建 Add Experience 交易
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
    /// 奖励类型 (0=capture, 1=battle_win, 2=daily_bonus)
    pub reward_type: u8,
    /// 基础金额（lamports）
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct DistributeRewardResponse {
    pub success: bool,
    pub tx_signature: String,
    pub amount: u64,
    pub reward_type: u8,
}

/// 分发 BREACH 代币奖励（后端直接转账）
/// 
/// 注意：当前使用后端直接转账而非链上 game_logic 合约
/// 因为 reward pool 尚未初始化
async fn distribute_reward(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<DistributeRewardRequest>,
) -> ApiResult<Json<DistributeRewardResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    // 验证奖励类型
    if request.reward_type > 2 {
        return Err(AppError::BadRequest("Invalid reward type".to_string()));
    }

    // 验证金额
    if request.amount == 0 {
        return Err(AppError::BadRequest("Amount must be greater than 0".to_string()));
    }

    // 计算奖励倍数（与合约逻辑保持一致）
    let multiplier = match request.reward_type {
        0 => 1,  // Capture
        1 => 2,  // Battle Win (2x)
        2 => 5,  // Daily Bonus (5x)
        _ => 1,
    };
    let final_amount = request.amount * multiplier;

    // 直接使用 transfer_breach_tokens（绕过链上 distribute_reward）
    let result = solana.transfer_breach_tokens(
        &player.wallet_address,
        final_amount,
    ).await?;

    Ok(Json(DistributeRewardResponse {
        success: true,
        tx_signature: result.signature,
        amount: final_amount,  // 返回实际分发的金额（已应用倍数）
        reward_type: request.reward_type,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// 提交双签名交易 API
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct SubmitDualSignedRequest {
    /// Base64 编码的交易
    pub serialized_transaction: String,
    /// Base64 编码的玩家签名
    pub player_signature: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitDualSignedResponse {
    pub success: bool,
    pub tx_signature: String,
}

/// 提交双签名交易（玩家 + 后端）
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
// 路由
// ═══════════════════════════════════════════════════════════════════════════════

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // 构建交易端点
        .route("/game/capture/build", post(build_record_capture))
        .route("/game/battle/build", post(build_record_battle))
        .route("/game/experience/build", post(build_add_experience))
        // 提交交易端点
        .route("/game/submit", post(submit_dual_signed))
        // 奖励分发端点
        .route("/game/reward/distribute", post(distribute_reward))
        .with_state(state)
}
