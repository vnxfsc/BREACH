//! Achievement API endpoints

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{AchievementCategory, AchievementWithStatus};
use crate::AppState;

/// Achievement query params
#[derive(Debug, Deserialize)]
pub struct AchievementQuery {
    pub category: Option<AchievementCategory>,
}

/// Get all achievements with unlock status
async fn get_achievements(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Query(query): Query<AchievementQuery>,
) -> ApiResult<Json<Vec<AchievementWithStatus>>> {
    let achievements = if let Some(category) = query.category {
        state
            .services
            .achievement
            .get_by_category(player.player_id, category)
            .await?
    } else {
        state
            .services
            .achievement
            .get_all_with_status(player.player_id)
            .await?
    };

    Ok(Json(achievements))
}

/// Recent query params
#[derive(Debug, Deserialize)]
pub struct RecentQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    10
}

/// Get recently unlocked achievements
async fn get_recent(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Query(query): Query<RecentQuery>,
) -> ApiResult<Json<Vec<AchievementWithStatus>>> {
    let achievements = state
        .services
        .achievement
        .get_recent(player.player_id, query.limit)
        .await?;

    Ok(Json(achievements))
}

/// Achievement summary response
#[derive(Debug, serde::Serialize)]
pub struct AchievementSummary {
    pub total: i32,
    pub unlocked: i32,
    pub by_category: Vec<CategorySummary>,
}

#[derive(Debug, serde::Serialize)]
pub struct CategorySummary {
    pub category: AchievementCategory,
    pub total: i32,
    pub unlocked: i32,
}

/// Get achievement summary
async fn get_summary(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<AchievementSummary>> {
    let all = state
        .services
        .achievement
        .get_all_with_status(player.player_id)
        .await?;

    let total = all.len() as i32;
    let unlocked = all.iter().filter(|a| a.is_unlocked).count() as i32;

    // Group by category
    let categories = [
        AchievementCategory::Capture,
        AchievementCategory::Collection,
        AchievementCategory::Battle,
        AchievementCategory::Exploration,
        AchievementCategory::Social,
        AchievementCategory::Special,
    ];

    let by_category: Vec<CategorySummary> = categories
        .iter()
        .map(|cat| {
            let in_category: Vec<_> = all.iter().filter(|a| &a.category == cat).collect();
            CategorySummary {
                category: *cat,
                total: in_category.len() as i32,
                unlocked: in_category.iter().filter(|a| a.is_unlocked).count() as i32,
            }
        })
        .collect();

    Ok(Json(AchievementSummary {
        total,
        unlocked,
        by_category,
    }))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/achievements", get(get_achievements))
        .route("/achievements/recent", get(get_recent))
        .route("/achievements/summary", get(get_summary))
        .with_state(state)
}
