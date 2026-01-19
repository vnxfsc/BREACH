//! Player data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Player account
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub id: Uuid,
    pub wallet_address: String,
    pub username: Option<String>,
    pub level: i32,
    pub experience: i64,
    pub titans_captured: i32,
    pub battles_won: i32,
    pub breach_earned: i64,
    pub last_capture_at: Option<DateTime<Utc>>,
    pub last_location_lat: Option<f64>,
    pub last_location_lng: Option<f64>,
    pub last_location_at: Option<DateTime<Utc>>,
    pub is_banned: bool,
    pub ban_reason: Option<String>,
    pub offense_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Player creation input
#[derive(Debug, Deserialize)]
pub struct CreatePlayer {
    pub wallet_address: String,
    pub username: Option<String>,
}

/// Player update input
#[derive(Debug, Deserialize)]
pub struct UpdatePlayer {
    pub username: Option<String>,
}

/// Player stats response
#[derive(Debug, Serialize)]
pub struct PlayerStats {
    pub level: i32,
    pub experience: i64,
    pub experience_to_next_level: i64,
    pub titans_captured: i32,
    pub battles_won: i32,
    pub breach_earned: i64,
    pub rank: Option<i32>,
}

/// Player session data (stored in JWT)
#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerSession {
    pub player_id: Uuid,
    pub wallet_address: String,
    pub exp: i64, // Expiration timestamp
}
