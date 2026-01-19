//! PvP API endpoints

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
    ActionResultResponse, JoinQueueRequest, MatchHistoryEntry, MatchStateResponse,
    PvpLeaderboardEntry, PvpSeason, PvpStatsResponse, QueueStatusResponse, SubmitActionRequest,
};
use crate::AppState;

/// Get current season
async fn get_season(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<PvpSeason>> {
    let season = state.services.pvp.get_current_season().await?;
    Ok(Json(season))
}

/// Get my PvP stats
async fn get_my_stats(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<PvpStatsResponse>> {
    let stats = state.services.pvp.get_stats_response(player.player_id).await?;
    Ok(Json(stats))
}

/// Join matchmaking queue
async fn join_queue(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<JoinQueueRequest>,
) -> ApiResult<Json<QueueStatusResponse>> {
    let status = state.services.pvp.join_queue(player.player_id, req).await?;
    Ok(Json(status))
}

/// Leave matchmaking queue
async fn leave_queue(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<&'static str>> {
    state.services.pvp.leave_queue(player.player_id).await?;
    Ok(Json("Left queue"))
}

/// Get queue status
async fn get_queue_status(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<QueueStatusResponse>> {
    let status = state.services.pvp.get_queue_status(player.player_id).await?;
    Ok(Json(status))
}

/// Select titan request
#[derive(Debug, Deserialize)]
pub struct SelectTitanRequest {
    pub titan_id: Uuid,
}

/// Select titan for match
async fn select_titan(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(match_id): Path<Uuid>,
    Json(req): Json<SelectTitanRequest>,
) -> ApiResult<Json<MatchStateResponse>> {
    let state_response = state
        .services
        .pvp
        .select_titan(player.player_id, match_id, req.titan_id)
        .await?;
    Ok(Json(state_response))
}

/// Get match state
async fn get_match_state(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(match_id): Path<Uuid>,
) -> ApiResult<Json<MatchStateResponse>> {
    let state_response = state
        .services
        .pvp
        .get_match_state(player.player_id, match_id)
        .await?;
    Ok(Json(state_response))
}

/// Submit action
async fn submit_action(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<SubmitActionRequest>,
) -> ApiResult<Json<ActionResultResponse>> {
    let result = state.services.pvp.submit_action(player.player_id, req).await?;
    Ok(Json(result))
}

/// Surrender match
async fn surrender(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(match_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state.services.pvp.surrender(player.player_id, match_id).await?;
    Ok(Json("Surrendered"))
}

/// Leaderboard query
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// Get PvP leaderboard
async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LeaderboardQuery>,
) -> ApiResult<Json<Vec<PvpLeaderboardEntry>>> {
    let entries = state
        .services
        .pvp
        .get_leaderboard(query.limit.min(100), query.offset)
        .await?;
    Ok(Json(entries))
}

/// History query
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    pub limit: i64,
}

fn default_history_limit() -> i64 {
    20
}

/// Get match history
async fn get_history(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Query(query): Query<HistoryQuery>,
) -> ApiResult<Json<Vec<MatchHistoryEntry>>> {
    let matches = state
        .services
        .pvp
        .get_match_history(player.player_id, query.limit.min(50))
        .await?;
    Ok(Json(matches))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Season & stats
        .route("/pvp/season", get(get_season))
        .route("/pvp/stats", get(get_my_stats))
        // Matchmaking
        .route("/pvp/queue", post(join_queue).get(get_queue_status).delete(leave_queue))
        // Match
        .route("/pvp/match/:match_id", get(get_match_state))
        .route("/pvp/match/:match_id/titan", post(select_titan))
        .route("/pvp/match/:match_id/surrender", post(surrender))
        .route("/pvp/action", post(submit_action))
        // Leaderboard & history
        .route("/pvp/leaderboard", get(get_leaderboard))
        .route("/pvp/history", get(get_history))
        .with_state(state)
}
