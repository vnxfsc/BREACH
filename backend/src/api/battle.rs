//! Battle API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
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

/// End battle
async fn end_battle(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(battle_id): Path<Uuid>,
) -> ApiResult<Json<BattleResultResponse>> {
    let result = state
        .services
        .battle
        .end_battle(battle_id, player.player_id)
        .await?;

    Ok(Json(result))
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
