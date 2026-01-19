//! Notification API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{Notification, NotificationCount};
use crate::AppState;

/// Notification query
#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    #[serde(default)]
    pub unread_only: bool,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// Get notifications
async fn get_notifications(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Query(query): Query<NotificationQuery>,
) -> ApiResult<Json<Vec<Notification>>> {
    let notifications = state
        .services
        .notification
        .get_notifications(
            player.player_id,
            query.unread_only,
            query.limit.min(100),
            query.offset,
        )
        .await?;
    Ok(Json(notifications))
}

/// Get notification counts
async fn get_counts(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<NotificationCount>> {
    let counts = state
        .services
        .notification
        .get_counts(player.player_id)
        .await?;
    Ok(Json(counts))
}

/// Mark notification as read
async fn mark_read(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(notification_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .notification
        .mark_read(player.player_id, notification_id)
        .await?;
    Ok(Json("Marked as read"))
}

/// Mark all response
#[derive(serde::Serialize)]
pub struct MarkAllResponse {
    pub marked_count: i64,
}

/// Mark all notifications as read
async fn mark_all_read(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<MarkAllResponse>> {
    let count = state
        .services
        .notification
        .mark_all_read(player.player_id)
        .await?;
    Ok(Json(MarkAllResponse { marked_count: count }))
}

/// Delete notification
async fn delete_notification(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(notification_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .notification
        .delete(player.player_id, notification_id)
        .await?;
    Ok(Json("Deleted"))
}

/// Delete response
#[derive(serde::Serialize)]
pub struct DeleteResponse {
    pub deleted_count: i64,
}

/// Delete all read notifications
async fn delete_read(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<DeleteResponse>> {
    let count = state
        .services
        .notification
        .delete_read(player.player_id)
        .await?;
    Ok(Json(DeleteResponse { deleted_count: count }))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/notifications", get(get_notifications))
        .route("/notifications/count", get(get_counts))
        .route("/notifications/read-all", post(mark_all_read))
        .route("/notifications/delete-read", delete(delete_read))
        .route("/notifications/:notification_id/read", post(mark_read))
        .route("/notifications/:notification_id", delete(delete_notification))
        .with_state(state)
}
