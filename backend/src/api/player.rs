//! Player endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use crate::error::{ApiResult, AppError};
use crate::middleware::auth::AuthPlayer;
use crate::models::{Player, PlayerStats, UpdatePlayer};
use crate::AppState;

/// Get current player profile
async fn get_me(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Player>> {
    let player = state
        .services
        .player
        .get_by_id(player.player_id)
        .await?
        .ok_or(AppError::PlayerNotFound)?;

    Ok(Json(player))
}

/// Update current player profile
async fn update_me(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(input): Json<UpdatePlayer>,
) -> ApiResult<Json<Player>> {
    let updated = state
        .services
        .player
        .update(player.player_id, input)
        .await?;

    Ok(Json(updated))
}

/// Get current player stats
async fn get_my_stats(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<PlayerStats>> {
    let stats = state.services.player.get_stats(player.player_id).await?;

    Ok(Json(stats))
}

/// Get player by ID (public profile)
async fn get_player(
    State(state): State<Arc<AppState>>,
    Path(player_id): Path<uuid::Uuid>,
) -> ApiResult<Json<Player>> {
    let player = state
        .services
        .player
        .get_by_id(player_id)
        .await?
        .ok_or(AppError::PlayerNotFound)?;

    Ok(Json(player))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/player/me", get(get_me).put(update_me))
        .route("/player/me/stats", get(get_my_stats))
        .route("/player/:player_id", get(get_player))
        // Note: /leaderboard is now handled by leaderboard.rs
        .with_state(state)
}
