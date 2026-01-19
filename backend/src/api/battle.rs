//! Battle API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{
    Battle, BattleAction, BattleActionRequest, BattleResultResponse, BattleSummary,
    StartWildBattleRequest,
};
use crate::AppState;

/// Start a wild battle
async fn start_wild_battle(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<StartWildBattleRequest>,
) -> ApiResult<Json<Battle>> {
    let battle = state
        .services
        .battle
        .start_wild_battle(player.player_id, req.titan_id, req.player_titan_id, req.location)
        .await?;

    Ok(Json(battle))
}

/// Get active battle
async fn get_active_battle(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Option<Battle>>> {
    let battle = state.services.battle.get_active(player.player_id).await?;

    Ok(Json(battle))
}

/// Process battle action
async fn process_action(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<BattleActionRequest>,
) -> ApiResult<Json<BattleAction>> {
    let action = state
        .services
        .battle
        .process_action(player.player_id, req.battle_id, &req.action_type)
        .await?;

    Ok(Json(action))
}

/// End battle request
#[derive(Debug, Deserialize)]
pub struct EndBattleRequest {
    #[serde(default)]
    pub skip_blockchain: bool, // For testing without blockchain
}

/// Extended battle result with blockchain info
#[derive(Debug, Serialize)]
pub struct ExtendedBattleResultResponse {
    // Core battle result fields
    pub battle_id: Uuid,
    pub winner_id: Option<Uuid>,
    pub is_winner: bool,
    pub total_rounds: i32,
    pub xp_earned: i32,
    pub breach_earned: i64,
    pub new_total_xp: i64,
    pub new_level: i32,
    // Blockchain fields
    pub tx_signature: Option<String>,
    pub breach_reward: Option<u64>,
    pub breach_tx_signature: Option<String>,
    pub xp_reward: Option<u64>,
}

/// End battle
async fn end_battle(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(battle_id): Path<Uuid>,
    Query(query): Query<EndBattleRequest>,
) -> ApiResult<Json<ExtendedBattleResultResponse>> {
    // End battle in database
    let result = state
        .services
        .battle
        .end_battle(battle_id, player.player_id)
        .await?;

    let mut tx_signature = None;
    let mut breach_reward = None;
    let mut breach_tx_signature = None;
    let xp_reward;

    // Calculate rewards based on battle result
    let (battle_type, battle_result_code) = if result.is_winner {
        (0u8, 1u8) // wild battle, win
    } else {
        (0u8, 0u8) // wild battle, loss
    };

    // Calculate XP and BREACH rewards based on xp_earned from database
    xp_reward = Some(result.xp_earned as u64);

    // Record on blockchain (if enabled)
    if !query.skip_blockchain {
        if let Some(solana) = &state.services.solana {
            // Use battle_id as placeholder for titan mint (in production, would look up actual mint)
            let titan_mint = result.battle_id.to_string();
            
            // Record battle on-chain
            match solana.record_battle(
                &player.wallet_address,
                &titan_mint,
                battle_type,
                battle_result_code,
            ).await {
                Ok(sig) => {
                    tx_signature = Some(sig.clone());
                    tracing::info!("Battle recorded on-chain: sig={}", sig);
                }
                Err(e) => {
                    tracing::warn!("Failed to record battle on-chain: {}", e);
                }
            }

            // Distribute BREACH reward for victory
            if result.is_winner {
                let reward = calculate_battle_breach_reward(result.xp_earned);
                if reward > 0 {
                    match solana.transfer_breach_tokens(&player.wallet_address, reward).await {
                        Ok(transfer_result) => {
                            breach_reward = Some(transfer_result.amount);
                            breach_tx_signature = Some(transfer_result.signature);
                            tracing::info!(
                                "Battle reward distributed: player={}, amount={}",
                                player.wallet_address,
                                reward
                            );
                        }
                        Err(e) => {
                            tracing::warn!("Failed to distribute battle reward: {}", e);
                        }
                    }
                }
            }

            // Add XP to Titan on-chain
            if let Some(xp) = xp_reward {
                if let Err(e) = solana.add_titan_experience(&titan_mint, xp).await {
                    tracing::warn!("Failed to add Titan XP on-chain: {}", e);
                }
            }
        }
    }

    Ok(Json(ExtendedBattleResultResponse {
        battle_id: result.battle_id,
        winner_id: result.winner_id,
        is_winner: result.is_winner,
        total_rounds: result.total_rounds,
        xp_earned: result.xp_earned,
        breach_earned: result.breach_earned,
        new_total_xp: result.new_total_xp,
        new_level: result.new_level,
        tx_signature,
        breach_reward,
        breach_tx_signature,
        xp_reward,
    }))
}

/// Calculate BREACH reward based on XP earned
fn calculate_battle_breach_reward(xp_earned: i32) -> u64 {
    // Base reward in smallest unit (9 decimals)
    const BASE_REWARD: u64 = 10_000_000; // 0.01 BREACH
    
    // Scale with XP earned
    BASE_REWARD + ((xp_earned as u64) * 100_000)
}

/// Battle history query
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

/// Get battle history
async fn get_history(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Query(query): Query<HistoryQuery>,
) -> ApiResult<Json<Vec<BattleSummary>>> {
    let history = state
        .services
        .battle
        .get_history(player.player_id, query.limit)
        .await?;

    Ok(Json(history))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/battle/wild", post(start_wild_battle))
        .route("/battle/active", get(get_active_battle))
        .route("/battle/action", post(process_action))
        .route("/battle/:battle_id/end", post(end_battle))
        .route("/battle/history", get(get_history))
        .with_state(state)
}
