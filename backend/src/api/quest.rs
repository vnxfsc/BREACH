//! Quest API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{QuestRewardResponse, QuestWithDetails};
use crate::AppState;

/// Get player's daily quests
async fn get_quests(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<QuestWithDetails>>> {
    let quests = state
        .services
        .quest
        .get_player_quests(player.player_id)
        .await?;

    Ok(Json(quests))
}

/// Claim quest reward
async fn claim_reward(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(quest_id): Path<Uuid>,
) -> ApiResult<Json<QuestRewardResponse>> {
    let response = state
        .services
        .quest
        .claim_reward(player.player_id, quest_id)
        .await?;

    Ok(Json(response))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/quests", get(get_quests))
        .route("/quests/:quest_id/claim", post(claim_reward))
        .with_state(state)
}
