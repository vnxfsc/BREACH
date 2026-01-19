//! Quest data models

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::Element;

/// Quest types enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quest_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum QuestType {
    Capture,
    CaptureElement,
    CaptureRare,
    Battle,
    Walk,
    VisitPoi,
    Streak,
}

/// Quest template definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuestTemplate {
    pub id: i32,
    pub quest_type: QuestType,
    pub title: String,
    pub description: String,
    pub target_count: i32,
    pub element: Option<Element>,
    pub xp_reward: i32,
    pub breach_reward: i64,
    pub is_daily: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Player quest progress
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlayerQuest {
    pub id: Uuid,
    pub player_id: Uuid,
    pub template_id: i32,
    pub progress: i32,
    pub is_completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub reward_claimed: bool,
    pub assigned_date: NaiveDate,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Quest with template details (for API response)
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct QuestWithDetails {
    pub id: Uuid,
    pub quest_type: QuestType,
    pub title: String,
    pub description: String,
    pub target_count: i32,
    pub element: Option<Element>,
    pub progress: i32,
    pub is_completed: bool,
    pub reward_claimed: bool,
    pub xp_reward: i32,
    pub breach_reward: i64,
    pub expires_at: DateTime<Utc>,
}

/// Claim quest reward response
#[derive(Debug, Serialize)]
pub struct QuestRewardResponse {
    pub success: bool,
    pub quest_id: Uuid,
    pub xp_earned: i32,
    pub breach_earned: i64,
    pub new_total_xp: i64,
    pub new_level: i32,
}
