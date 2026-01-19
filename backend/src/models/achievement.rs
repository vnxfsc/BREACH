//! Achievement data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Achievement categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "achievement_category", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AchievementCategory {
    Capture,
    Collection,
    Battle,
    Exploration,
    Social,
    Special,
}

/// Achievement definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub icon: Option<String>,
    pub tier: i32,
    pub requirement_type: String,
    pub requirement_value: i32,
    pub xp_reward: i32,
    pub breach_reward: i64,
    pub is_hidden: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Player achievement unlock record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerAchievement {
    pub id: Uuid,
    pub player_id: Uuid,
    pub achievement_id: i32,
    pub unlocked_at: DateTime<Utc>,
}

/// Achievement with unlock status (for API response)
#[derive(Debug, Clone, Serialize)]
pub struct AchievementWithStatus {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub icon: Option<String>,
    pub tier: i32,
    pub requirement_type: String,
    pub requirement_value: i32,
    pub xp_reward: i32,
    pub breach_reward: i64,
    pub is_hidden: bool,
    pub is_unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub progress: i32,
}

/// Achievement unlock response
#[derive(Debug, Serialize)]
pub struct AchievementUnlockResponse {
    pub achievement: Achievement,
    pub xp_earned: i32,
    pub breach_earned: i64,
    pub new_total_xp: i64,
    pub new_level: i32,
}
