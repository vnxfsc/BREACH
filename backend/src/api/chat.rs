//! Chat API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::AppState;
use crate::middleware::auth::AuthPlayer;
use crate::models::{
    BlockUserRequest, ChatChannel, ChatReport, ChannelResponse, EditMessageRequest,
    MessageResponse, MessagesQuery, MuteChannelRequest, ReportMessageRequest,
    SendMessageRequest, StartPrivateChatRequest,
};

/// Build chat routes
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Channels
        .route("/chat/channels", get(get_channels))
        .route("/chat/channels/private", post(start_private_chat))
        .route("/chat/channels/:id/messages", get(get_messages))
        .route("/chat/channels/:id/messages", post(send_message))
        .route("/chat/channels/:id/read", post(mark_as_read))
        .route("/chat/channels/:id/mute", post(mute_channel))
        .route("/chat/channels/:id/unmute", post(unmute_channel))
        // Messages
        .route("/chat/messages/:id", put(edit_message))
        .route("/chat/messages/:id", delete(delete_message))
        .route("/chat/messages/:id/report", post(report_message))
        // Blocking
        .route("/chat/blocked", get(get_blocked_users))
        .route("/chat/blocked", post(block_user))
        .route("/chat/blocked/:player_id", delete(unblock_user))
        .with_state(state)
}

// ============================================
// Channel Endpoints
// ============================================

/// Get player's channels
async fn get_channels(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<ChannelResponse>>> {
    let channels = state.services.chat.get_player_channels(player.player_id).await?;
    Ok(Json(channels))
}

/// Start a private chat
async fn start_private_chat(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<StartPrivateChatRequest>,
) -> ApiResult<Json<ChatChannel>> {
    let channel = state
        .services
        .chat
        .get_or_create_private_channel(player.player_id, req.player_id)
        .await?;
    Ok(Json(channel))
}

/// Get channel messages
async fn get_messages(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<MessagesQuery>,
) -> ApiResult<Json<Vec<MessageResponse>>> {
    let messages = state
        .services
        .chat
        .get_messages(player.player_id, channel_id, query)
        .await?;
    Ok(Json(messages))
}

/// Send a message
async fn send_message(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> ApiResult<Json<MessageResponse>> {
    let message = state
        .services
        .chat
        .send_message(player.player_id, channel_id, req)
        .await?;
    
    // Broadcast message via WebSocket
    let ws_message = crate::websocket::WsMessage::ChatMessage {
        channel_id: channel_id.to_string(),
        message_id: message.id.to_string(),
        sender_id: message.sender_id.to_string(),
        sender_username: message.sender_username.clone(),
        content: message.content.clone(),
        created_at: message.created_at.to_rfc3339(),
    };
    state.broadcaster.broadcast_chat_message(channel_id, ws_message).await;
    
    Ok(Json(message))
}

/// Mark channel as read
async fn mark_as_read(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(channel_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.chat.mark_as_read(player.player_id, channel_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

/// Mute a channel
async fn mute_channel(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<MuteChannelRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    state
        .services
        .chat
        .mute_channel(player.player_id, channel_id, req.duration_hours)
        .await?;
    Ok(Json(serde_json::json!({"success": true})))
}

/// Unmute a channel
async fn unmute_channel(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(channel_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.chat.unmute_channel(player.player_id, channel_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

// ============================================
// Message Endpoints
// ============================================

/// Edit a message
async fn edit_message(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(message_id): Path<Uuid>,
    Json(req): Json<EditMessageRequest>,
) -> ApiResult<Json<MessageResponse>> {
    let message = state
        .services
        .chat
        .edit_message(player.player_id, message_id, req.content.clone())
        .await?;
    
    // Broadcast edit via WebSocket
    let ws_message = crate::websocket::WsMessage::ChatMessageEdited {
        channel_id: message.channel_id.to_string(),
        message_id: message.id.to_string(),
        new_content: message.content.clone(),
        edited_at: chrono::Utc::now().to_rfc3339(),
    };
    state.broadcaster.broadcast_chat_message(message.channel_id, ws_message).await;
    
    Ok(Json(message))
}

/// Delete a message
async fn delete_message(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(message_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Get message info before deleting for broadcast
    let channel_id = state.services.chat.get_message_channel_id(message_id).await?;
    
    state.services.chat.delete_message(player.player_id, message_id).await?;
    
    // Broadcast deletion via WebSocket
    let ws_message = crate::websocket::WsMessage::ChatMessageDeleted {
        channel_id: channel_id.to_string(),
        message_id: message_id.to_string(),
    };
    state.broadcaster.broadcast_chat_message(channel_id, ws_message).await;
    
    Ok(Json(serde_json::json!({"success": true})))
}

/// Report a message
async fn report_message(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(message_id): Path<Uuid>,
    Json(req): Json<ReportMessageRequest>,
) -> ApiResult<Json<ChatReport>> {
    let report = state
        .services
        .chat
        .report_message(player.player_id, message_id, req)
        .await?;
    Ok(Json(report))
}

// ============================================
// Blocking Endpoints
// ============================================

/// Get blocked users
async fn get_blocked_users(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<Uuid>>> {
    let blocked = state.services.chat.get_blocked_users(player.player_id).await?;
    Ok(Json(blocked))
}

/// Block a user
async fn block_user(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<BlockUserRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.chat.block_user(player.player_id, req.player_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

/// Unblock a user
async fn unblock_user(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(player_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.chat.unblock_user(player.player_id, player_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}
