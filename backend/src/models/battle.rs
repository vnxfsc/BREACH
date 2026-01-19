//! Battle data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Battle types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "battle_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BattleType {
    Wild,
    Pvp,
    Raid,
    Gym,
}

/// Battle status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "battle_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BattleStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

/// Battle record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Battle {
    pub id: Uuid,
    pub battle_type: BattleType,
    pub status: BattleStatus,
    pub player1_id: Uuid,
    pub player2_id: Option<Uuid>,
    pub player1_titan_id: Option<Uuid>,
    pub player2_titan_id: Option<Uuid>,
    pub wild_titan_id: Option<Uuid>,
    pub winner_id: Option<Uuid>,
    pub player1_damage: i32,
    pub player2_damage: i32,
    pub rounds: i32,
    pub xp_reward: i32,
    pub breach_reward: i64,
    pub location_lat: Option<f64>,
    pub location_lng: Option<f64>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Battle action log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BattleAction {
    pub id: Uuid,
    pub battle_id: Uuid,
    pub round: i32,
    pub actor_id: Uuid,
    pub action_type: String,
    pub damage: i32,
    pub timestamp: DateTime<Utc>,
}

/// Start wild battle request
#[derive(Debug, Deserialize)]
pub struct StartWildBattleRequest {
    pub titan_id: Uuid,
    pub player_titan_id: Uuid,
    pub location: LocationInput,
}

/// Start PvP battle request
#[derive(Debug, Deserialize)]
pub struct StartPvpBattleRequest {
    pub opponent_id: Uuid,
    pub player_titan_id: Uuid,
}

/// Battle action request
#[derive(Debug, Deserialize)]
pub struct BattleActionRequest {
    pub battle_id: Uuid,
    pub action_type: String,
}

/// Location input
#[derive(Debug, Deserialize)]
pub struct LocationInput {
    pub lat: f64,
    pub lng: f64,
}

/// Battle result response
#[derive(Debug, Serialize)]
pub struct BattleResultResponse {
    pub battle_id: Uuid,
    pub winner_id: Option<Uuid>,
    pub is_winner: bool,
    pub total_rounds: i32,
    pub xp_earned: i32,
    pub breach_earned: i64,
    pub new_total_xp: i64,
    pub new_level: i32,
}

/// Battle summary for listing
#[derive(Debug, Clone, Serialize)]
pub struct BattleSummary {
    pub id: Uuid,
    pub battle_type: BattleType,
    pub opponent_name: Option<String>,
    pub is_winner: bool,
    pub xp_earned: i32,
    pub breach_earned: i64,
    pub rounds: i32,
    pub ended_at: Option<DateTime<Utc>>,
}
