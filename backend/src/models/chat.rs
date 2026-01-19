//! Chat system data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================
// Enums
// ============================================

/// Chat channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "chat_channel_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChatChannelType {
    World,
    Guild,
    Private,
    Trade,
    Help,
}

// ============================================
// Database Models
// ============================================

/// Chat channel
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ChatChannel {
    pub id: Uuid,
    pub channel_type: ChatChannelType,
    pub name: Option<String>,
    pub participant1_id: Option<Uuid>,
    pub participant2_id: Option<Uuid>,
    pub guild_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ChatMessage {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub is_system: bool,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub reply_to_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

/// Chat read status
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ChatReadStatus {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub player_id: Uuid,
    pub last_read_message_id: Option<Uuid>,
    pub last_read_at: DateTime<Utc>,
    pub muted: bool,
    pub muted_until: Option<DateTime<Utc>>,
}

/// Blocked user
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ChatBlockedUser {
    pub id: Uuid,
    pub blocker_id: Uuid,
    pub blocked_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Chat report
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ChatReport {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub reported_id: Uuid,
    pub message_id: Option<Uuid>,
    pub reason: String,
    pub description: Option<String>,
    pub status: String,
    pub admin_notes: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// API Request/Response Models
// ============================================

/// Send message request
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(default)]
    pub reply_to_id: Option<Uuid>,
}

/// Message response with sender info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_id: Uuid,
    pub sender_username: Option<String>,
    pub sender_level: Option<i32>,
    pub content: String,
    pub is_system: bool,
    pub is_edited: bool,
    pub reply_to: Option<ReplyInfo>,
    pub created_at: DateTime<Utc>,
}

/// Reply info for nested display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyInfo {
    pub id: Uuid,
    pub sender_username: Option<String>,
    pub content_preview: String,
}

/// Channel info response
#[derive(Debug, Serialize)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub channel_type: ChatChannelType,
    pub name: Option<String>,
    pub guild_id: Option<Uuid>,
    pub guild_name: Option<String>,
    pub participant: Option<ParticipantInfo>,
    pub unread_count: i32,
    pub last_message: Option<LastMessageInfo>,
    pub is_muted: bool,
    pub created_at: DateTime<Utc>,
}

/// Participant info for private channels
#[derive(Debug, Serialize)]
pub struct ParticipantInfo {
    pub id: Uuid,
    pub username: Option<String>,
    pub level: i32,
    pub is_online: bool,
}

/// Last message preview
#[derive(Debug, Serialize)]
pub struct LastMessageInfo {
    pub sender_username: Option<String>,
    pub content_preview: String,
    pub sent_at: DateTime<Utc>,
}

/// Start private chat request
#[derive(Debug, Deserialize)]
pub struct StartPrivateChatRequest {
    pub player_id: Uuid,
}

/// Edit message request
#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

/// Report message request
#[derive(Debug, Deserialize)]
pub struct ReportMessageRequest {
    pub reason: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Block user request
#[derive(Debug, Deserialize)]
pub struct BlockUserRequest {
    pub player_id: Uuid,
}

/// Mute channel request
#[derive(Debug, Deserialize)]
pub struct MuteChannelRequest {
    #[serde(default)]
    pub duration_hours: Option<i64>,  // None = permanent
}

/// Messages pagination query
#[derive(Debug, Deserialize)]
pub struct MessagesQuery {
    #[serde(default)]
    pub before_id: Option<Uuid>,
    #[serde(default = "default_message_limit")]
    pub limit: i64,
}

fn default_message_limit() -> i64 {
    50
}

/// Online status response
#[derive(Debug, Serialize)]
pub struct OnlineStatusResponse {
    pub online_count: i32,
    pub channel_online: i32,
}

// ============================================
// WebSocket Messages
// ============================================

/// WebSocket chat message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatWsMessage {
    /// New message received
    NewMessage {
        channel_id: Uuid,
        message: MessageResponse,
    },
    /// Message edited
    MessageEdited {
        channel_id: Uuid,
        message_id: Uuid,
        new_content: String,
    },
    /// Message deleted
    MessageDeleted {
        channel_id: Uuid,
        message_id: Uuid,
    },
    /// User typing indicator
    Typing {
        channel_id: Uuid,
        user_id: Uuid,
        username: String,
    },
    /// User stopped typing
    StopTyping {
        channel_id: Uuid,
        user_id: Uuid,
    },
    /// User joined channel
    UserJoined {
        channel_id: Uuid,
        user_id: Uuid,
        username: String,
    },
    /// User left channel
    UserLeft {
        channel_id: Uuid,
        user_id: Uuid,
        username: String,
    },
    /// Online status update
    OnlineStatus {
        channel_id: Uuid,
        online_users: Vec<Uuid>,
    },
}

/// System channel IDs
pub mod system_channels {
    use uuid::Uuid;

    pub const WORLD_CHAT: Uuid = Uuid::from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
    ]);
    
    pub const TRADE_CHAT: Uuid = Uuid::from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2
    ]);
    
    pub const HELP_CHAT: Uuid = Uuid::from_bytes([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3
    ]);
}
