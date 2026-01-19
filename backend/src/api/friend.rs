//! Friend API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{
    FriendGift, FriendInfo, FriendRequest, FriendRequestWithSender, GiftWithSender,
    SendFriendRequest,
};
use crate::AppState;

/// Get all friends
async fn get_friends(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<FriendInfo>>> {
    let friends = state.services.friend.get_friends(player.player_id).await?;
    Ok(Json(friends))
}

/// Get friend count
async fn get_friend_count(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<i64>> {
    let count = state.services.friend.get_friend_count(player.player_id).await?;
    Ok(Json(count))
}

/// Send friend request
async fn send_request(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<SendFriendRequest>,
) -> ApiResult<Json<FriendRequest>> {
    let request = state
        .services
        .friend
        .send_request(player.player_id, req)
        .await?;
    Ok(Json(request))
}

/// Get pending friend requests
async fn get_pending_requests(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<FriendRequestWithSender>>> {
    let requests = state
        .services
        .friend
        .get_pending_requests(player.player_id)
        .await?;
    Ok(Json(requests))
}

/// Accept friend request
async fn accept_request(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(request_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .friend
        .respond_to_request(player.player_id, request_id, true)
        .await?;
    Ok(Json("Accepted"))
}

/// Reject friend request
async fn reject_request(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(request_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .friend
        .respond_to_request(player.player_id, request_id, false)
        .await?;
    Ok(Json("Rejected"))
}

/// Remove friend
async fn remove_friend(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(friend_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .friend
        .remove_friend(player.player_id, friend_id)
        .await?;
    Ok(Json("Removed"))
}

/// Send gift to friend
async fn send_gift(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(friend_id): Path<Uuid>,
) -> ApiResult<Json<FriendGift>> {
    let gift = state
        .services
        .friend
        .send_gift(player.player_id, friend_id)
        .await?;
    Ok(Json(gift))
}

/// Get pending gifts
async fn get_pending_gifts(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<GiftWithSender>>> {
    let gifts = state
        .services
        .friend
        .get_pending_gifts(player.player_id)
        .await?;
    Ok(Json(gifts))
}

/// Gift reward response
#[derive(serde::Serialize)]
pub struct GiftRewardResponse {
    pub breach_amount: i64,
}

/// Open gift
async fn open_gift(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(gift_id): Path<Uuid>,
) -> ApiResult<Json<GiftRewardResponse>> {
    let amount = state
        .services
        .friend
        .open_gift(player.player_id, gift_id)
        .await?;
    Ok(Json(GiftRewardResponse { breach_amount: amount }))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/friends", get(get_friends))
        .route("/friends/count", get(get_friend_count))
        .route("/friends/request", post(send_request))
        .route("/friends/requests", get(get_pending_requests))
        .route("/friends/requests/:request_id/accept", post(accept_request))
        .route("/friends/requests/:request_id/reject", post(reject_request))
        .route("/friends/:friend_id", delete(remove_friend))
        .route("/friends/:friend_id/gift", post(send_gift))
        .route("/friends/gifts", get(get_pending_gifts))
        .route("/friends/gifts/:gift_id/open", post(open_gift))
        .with_state(state)
}
