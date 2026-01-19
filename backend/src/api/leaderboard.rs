//! Leaderboard API endpoints

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{LeaderboardQuery, LeaderboardResponse, LeaderboardResponseEntry, LeaderboardType};
use crate::AppState;

/// Get leaderboard by type
async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Query(query): Query<LeaderboardQuery>,
) -> ApiResult<Json<LeaderboardResponse>> {
    let lb_type = query.lb_type.unwrap_or(LeaderboardType::Experience);
    let limit = query.limit.min(100);
    let offset = query.offset.max(0);

    let response = state
        .services
        .leaderboard
        .get_leaderboard(
            lb_type,
            query.region.as_deref(),
            limit,
            offset,
            Some(player.player_id),
        )
        .await?;

    Ok(Json(response))
}

/// My ranks response
#[derive(Debug, serde::Serialize)]
pub struct MyRanksResponse {
    pub ranks: Vec<RankEntry>,
}

#[derive(Debug, serde::Serialize)]
pub struct RankEntry {
    pub leaderboard_type: LeaderboardType,
    pub rank: i32,
    pub score: i64,
}

/// Get my ranks across all leaderboards
async fn get_my_ranks(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<MyRanksResponse>> {
    let ranks = state
        .services
        .leaderboard
        .get_player_ranks(player.player_id)
        .await?;

    let rank_entries: Vec<RankEntry> = ranks
        .into_iter()
        .map(|(lb_type, rank, score)| RankEntry {
            leaderboard_type: lb_type,
            rank,
            score,
        })
        .collect();

    Ok(Json(MyRanksResponse { ranks: rank_entries }))
}

/// Top stat query
#[derive(Debug, serde::Deserialize)]
pub struct TopStatQuery {
    pub stat: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    10
}

/// Get top players by a specific stat
async fn get_top_by_stat(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopStatQuery>,
) -> ApiResult<Json<Vec<LeaderboardResponseEntry>>> {
    let entries = state
        .services
        .leaderboard
        .get_top_by_stat(&query.stat, query.limit.min(100))
        .await?;

    Ok(Json(entries))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/leaderboard", get(get_leaderboard))
        .route("/leaderboard/me", get(get_my_ranks))
        .route("/leaderboard/top", get(get_top_by_stat))
        .with_state(state)
}
