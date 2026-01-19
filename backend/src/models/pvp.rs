//! PvP data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ==========================================
// SEASONS
// ==========================================

/// PvP Season
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PvpSeason {
    pub id: i32,
    pub name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub is_active: bool,
    pub rewards: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ==========================================
// PLAYER STATS
// ==========================================

/// Rank tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RankTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Master,
    Champion,
}

impl RankTier {
    pub fn from_elo(elo: i32) -> Self {
        match elo {
            e if e >= 2400 => RankTier::Champion,
            e if e >= 2200 => RankTier::Master,
            e if e >= 2000 => RankTier::Diamond,
            e if e >= 1800 => RankTier::Platinum,
            e if e >= 1600 => RankTier::Gold,
            e if e >= 1400 => RankTier::Silver,
            _ => RankTier::Bronze,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            RankTier::Bronze => "bronze",
            RankTier::Silver => "silver",
            RankTier::Gold => "gold",
            RankTier::Platinum => "platinum",
            RankTier::Diamond => "diamond",
            RankTier::Master => "master",
            RankTier::Champion => "champion",
        }
    }
}

/// Player PvP stats
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerPvpStats {
    pub id: Uuid,
    pub player_id: Uuid,
    pub season_id: i32,
    pub elo_rating: i32,
    pub peak_rating: i32,
    pub matches_played: i32,
    pub matches_won: i32,
    pub matches_lost: i32,
    pub win_streak: i32,
    pub max_win_streak: i32,
    pub rank_tier: String,
    pub rank_division: i32,
    pub rank_points: i32,
    pub last_match_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Player PvP stats response (with computed fields)
#[derive(Debug, Clone, Serialize)]
pub struct PvpStatsResponse {
    pub player_id: Uuid,
    pub season_id: i32,
    pub elo_rating: i32,
    pub peak_rating: i32,
    pub matches_played: i32,
    pub matches_won: i32,
    pub matches_lost: i32,
    pub win_rate: f64,
    pub win_streak: i32,
    pub max_win_streak: i32,
    pub rank_tier: String,
    pub rank_division: i32,
    pub rank_display: String,  // e.g. "Gold III"
    pub global_rank: Option<i64>,
}

// ==========================================
// MATCHMAKING
// ==========================================

/// Queue status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "queue_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum QueueStatus {
    Searching,
    Matched,
    Cancelled,
    Expired,
}

/// Matchmaking queue entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueueEntry {
    pub id: Uuid,
    pub player_id: Uuid,
    pub titan_id: Uuid,
    pub elo_rating: i32,
    pub elo_range: i32,
    pub search_start_time: DateTime<Utc>,
    pub status: QueueStatus,
    pub matched_with: Option<Uuid>,
    pub match_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Join queue request
#[derive(Debug, Deserialize)]
pub struct JoinQueueRequest {
    pub titan_id: Uuid,
}

/// Queue status response
#[derive(Debug, Serialize)]
pub struct QueueStatusResponse {
    pub in_queue: bool,
    pub status: Option<QueueStatus>,
    pub wait_time_seconds: Option<i64>,
    pub estimated_wait: Option<String>,
    pub match_found: bool,
    pub match_id: Option<Uuid>,
    pub opponent_id: Option<Uuid>,
}

// ==========================================
// PVP MATCHES
// ==========================================

/// Match status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "pvp_match_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PvpMatchStatus {
    Preparing,
    TitanSelect,
    Active,
    Completed,
    Abandoned,
}

/// PvP Match
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PvpMatch {
    pub id: Uuid,
    pub season_id: i32,
    pub player1_id: Uuid,
    pub player2_id: Uuid,
    pub player1_elo: i32,
    pub player2_elo: i32,
    pub player1_titan_id: Option<Uuid>,
    pub player2_titan_id: Option<Uuid>,
    pub status: PvpMatchStatus,
    pub player1_hp: i32,
    pub player2_hp: i32,
    pub current_turn: Option<Uuid>,
    pub turn_number: i32,
    pub turn_deadline: Option<DateTime<Utc>>,
    pub winner_id: Option<Uuid>,
    pub loser_id: Option<Uuid>,
    pub win_reason: Option<String>,
    pub winner_elo_change: Option<i32>,
    pub loser_elo_change: Option<i32>,
    pub winner_breach_reward: Option<i64>,
    pub winner_xp_reward: Option<i32>,
    pub ready_deadline: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Match state for client
#[derive(Debug, Clone, Serialize)]
pub struct MatchStateResponse {
    pub match_id: Uuid,
    pub status: PvpMatchStatus,
    pub opponent_id: Uuid,
    pub opponent_username: Option<String>,
    pub opponent_elo: i32,
    pub my_hp: i32,
    pub opponent_hp: i32,
    pub is_my_turn: bool,
    pub turn_number: i32,
    pub turn_deadline: Option<DateTime<Utc>>,
    pub my_titan: Option<TitanBattleInfo>,
    pub opponent_titan: Option<TitanBattleInfo>,
}

/// Titan info for battle
#[derive(Debug, Clone, Serialize)]
pub struct TitanBattleInfo {
    pub id: Uuid,
    pub species_id: i32,
    pub element: String,
    pub threat_class: i16,
    pub nickname: Option<String>,
    pub stats: TitanBattleStats,
}

/// Computed battle stats
#[derive(Debug, Clone, Serialize)]
pub struct TitanBattleStats {
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub special: i32,
}

// ==========================================
// BATTLE ACTIONS
// ==========================================

/// PvP action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "pvp_action_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PvpActionType {
    Attack,
    Special,
    Defend,
    Item,
}

/// Battle turn record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PvpBattleTurn {
    pub id: Uuid,
    pub match_id: Uuid,
    pub turn_number: i32,
    pub player1_action: Option<PvpActionType>,
    pub player1_damage: Option<i32>,
    pub player2_action: Option<PvpActionType>,
    pub player2_damage: Option<i32>,
    pub player1_hp_after: Option<i32>,
    pub player2_hp_after: Option<i32>,
    pub submitted_at: DateTime<Utc>,
}

/// Submit action request
#[derive(Debug, Deserialize)]
pub struct SubmitActionRequest {
    pub match_id: Uuid,
    pub action: PvpActionType,
}

/// Action result
#[derive(Debug, Serialize)]
pub struct ActionResultResponse {
    pub success: bool,
    pub my_action: PvpActionType,
    pub my_damage: i32,
    pub opponent_action: Option<PvpActionType>,
    pub opponent_damage: Option<i32>,
    pub my_hp_after: i32,
    pub opponent_hp_after: i32,
    pub turn_complete: bool,
    pub match_ended: bool,
    pub winner_id: Option<Uuid>,
}

// ==========================================
// LEADERBOARD
// ==========================================

/// PvP leaderboard entry
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PvpLeaderboardEntry {
    pub player_id: Uuid,
    pub username: Option<String>,
    pub wallet_address: String,
    pub elo_rating: i32,
    pub peak_rating: i32,
    pub rank_tier: String,
    pub rank_division: i32,
    pub matches_played: i32,
    pub matches_won: i32,
    pub matches_lost: i32,
    pub win_rate: f64,
    pub max_win_streak: i32,
    pub rank: i64,
}

/// Match history entry
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MatchHistoryEntry {
    pub id: Uuid,
    pub opponent_id: Uuid,
    pub opponent_username: Option<String>,
    pub my_elo: i32,
    pub opponent_elo: i32,
    pub won: bool,
    pub elo_change: i32,
    pub win_reason: Option<String>,
    pub total_turns: i32,
    pub duration_seconds: Option<f64>,
    pub ended_at: Option<DateTime<Utc>>,
}
