//! Social data models (Friends, Guilds, Notifications)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ==========================================
// FRIEND SYSTEM
// ==========================================

/// Friend request status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "friend_request_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum FriendRequestStatus {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
}

/// Friend request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FriendRequest {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub status: FriendRequestStatus,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

/// Friendship record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Friendship {
    pub id: Uuid,
    pub player1_id: Uuid,
    pub player2_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Friend with profile info (for API response)
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FriendInfo {
    pub player_id: Uuid,
    pub username: Option<String>,
    pub wallet_address: String,
    pub level: i32,
    pub titans_captured: i32,
    pub is_online: bool,
    pub last_active_at: Option<DateTime<Utc>>,
    pub friendship_date: DateTime<Utc>,
}

/// Friend request with sender info
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct FriendRequestWithSender {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_username: Option<String>,
    pub sender_level: i32,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Send friend request input
#[derive(Debug, Deserialize)]
pub struct SendFriendRequest {
    pub friend_code: Option<String>,
    pub player_id: Option<Uuid>,
    pub message: Option<String>,
}

/// Friend gift
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FriendGift {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub gift_date: NaiveDate,
    pub breach_amount: i64,
    pub opened: bool,
    pub opened_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Gift with sender info
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct GiftWithSender {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub sender_username: Option<String>,
    pub breach_amount: i64,
    pub created_at: DateTime<Utc>,
}

// ==========================================
// GUILD SYSTEM
// ==========================================

/// Guild role (ordered from highest to lowest rank)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "guild_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum GuildRole {
    Leader = 0,     // Highest rank
    CoLeader = 1,
    Elder = 2,
    Member = 3,     // Lowest rank
}

impl GuildRole {
    pub fn can_manage(&self) -> bool {
        matches!(self, GuildRole::Leader | GuildRole::CoLeader)
    }

    pub fn can_kick(&self, target: &GuildRole) -> bool {
        match self {
            GuildRole::Leader => true,
            GuildRole::CoLeader => matches!(target, GuildRole::Elder | GuildRole::Member),
            GuildRole::Elder => matches!(target, GuildRole::Member),
            GuildRole::Member => false,
        }
    }
}

/// Guild
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Guild {
    pub id: Uuid,
    pub name: String,
    pub tag: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub leader_id: Uuid,
    pub min_level: i32,
    pub is_public: bool,
    pub max_members: i32,
    pub total_captures: i32,
    pub total_battles: i32,
    pub total_breach: i64,
    pub weekly_xp: i64,
    pub season_rank: Option<i32>,
    pub season_points: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Guild summary (for listings)
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct GuildSummary {
    pub id: Uuid,
    pub name: String,
    pub tag: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub leader_username: Option<String>,
    pub member_count: i64,
    pub max_members: i32,
    pub min_level: i32,
    pub is_public: bool,
    pub weekly_xp: i64,
    pub season_rank: Option<i32>,
}

/// Guild member
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GuildMember {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub player_id: Uuid,
    pub role: GuildRole,
    pub contribution_xp: i64,
    pub contribution_captures: i32,
    pub contribution_battles: i32,
    pub joined_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

/// Guild member with profile
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct GuildMemberInfo {
    pub player_id: Uuid,
    pub username: Option<String>,
    pub level: i32,
    pub role: GuildRole,
    pub contribution_xp: i64,
    pub contribution_captures: i32,
    pub is_online: bool,
    pub joined_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

/// Create guild input
#[derive(Debug, Deserialize)]
pub struct CreateGuildRequest {
    pub name: String,
    pub tag: String,
    pub description: Option<String>,
    pub min_level: Option<i32>,
    pub is_public: Option<bool>,
}

/// Update guild input
#[derive(Debug, Deserialize)]
pub struct UpdateGuildRequest {
    pub description: Option<String>,
    pub icon: Option<String>,
    pub banner: Option<String>,
    pub min_level: Option<i32>,
    pub is_public: Option<bool>,
}

/// Guild join request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GuildRequest {
    pub id: Uuid,
    pub guild_id: Uuid,
    pub player_id: Uuid,
    pub message: Option<String>,
    pub status: FriendRequestStatus,
    pub reviewed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

/// Guild request with player info
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct GuildRequestWithPlayer {
    pub id: Uuid,
    pub player_id: Uuid,
    pub username: Option<String>,
    pub level: i32,
    pub titans_captured: i32,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ==========================================
// NOTIFICATIONS
// ==========================================

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    FriendRequest,
    FriendAccepted,
    GiftReceived,
    GuildInvite,
    GuildRequest,
    GuildAccepted,
    GuildPromoted,
    GuildDemoted,
    GuildKicked,
    AchievementUnlocked,
    LevelUp,
    RareCapture,
    System,
}

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub player_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Notification count response
#[derive(Debug, Serialize)]
pub struct NotificationCount {
    pub total: i64,
    pub unread: i64,
}
