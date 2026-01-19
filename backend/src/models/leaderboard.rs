//! Leaderboard data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Leaderboard types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "leaderboard_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardType {
    Experience,
    Captures,
    Battles,
    Breach,
    WeeklyXp,
    WeeklyCaptures,
    WeeklyBattles,
}

/// Leaderboard cache entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaderboardEntry {
    pub id: i32,
    pub leaderboard_type: LeaderboardType,
    pub player_id: Uuid,
    pub rank: i32,
    pub score: i64,
    pub region: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Leaderboard query parameters
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    #[serde(default)]
    pub lb_type: Option<LeaderboardType>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}

/// Leaderboard response entry (with player details)
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LeaderboardResponseEntry {
    pub rank: i64,  // ROW_NUMBER returns i64
    pub player_id: Uuid,
    pub username: Option<String>,
    pub wallet_address: String,
    pub score: i64,
    pub level: i32,
}

/// Full leaderboard response
#[derive(Debug, Serialize)]
pub struct LeaderboardResponse {
    pub leaderboard_type: LeaderboardType,
    pub region: Option<String>,
    pub entries: Vec<LeaderboardResponseEntry>,
    pub total_count: i64,
    pub my_rank: Option<i32>,
    pub my_score: Option<i64>,
}
