//! Guild API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{
    CreateGuildRequest, Guild, GuildMember, GuildMemberInfo, GuildRequestWithPlayer, GuildRole,
    GuildSummary, UpdateGuildRequest,
};
use crate::AppState;

/// Search query
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

/// Create a new guild
async fn create_guild(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<CreateGuildRequest>,
) -> ApiResult<Json<Guild>> {
    let guild = state
        .services
        .guild
        .create_guild(player.player_id, req)
        .await?;
    Ok(Json(guild))
}

/// Search guilds
async fn search_guilds(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Vec<GuildSummary>>> {
    let guilds = state
        .services
        .guild
        .search_guilds(query.q.as_deref(), query.limit.min(50), query.offset)
        .await?;
    Ok(Json(guilds))
}

/// Get guild by ID
async fn get_guild(
    State(state): State<Arc<AppState>>,
    Path(guild_id): Path<Uuid>,
) -> ApiResult<Json<Guild>> {
    let guild = state
        .services
        .guild
        .get_guild(guild_id)
        .await?
        .ok_or(crate::error::AppError::NotFound("Guild not found".into()))?;
    Ok(Json(guild))
}

/// Get guild members
async fn get_members(
    State(state): State<Arc<AppState>>,
    Path(guild_id): Path<Uuid>,
) -> ApiResult<Json<Vec<GuildMemberInfo>>> {
    let members = state.services.guild.get_members(guild_id).await?;
    Ok(Json(members))
}

/// Get my guild
async fn get_my_guild(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Option<GuildMember>>> {
    let member = state.services.guild.get_membership(player.player_id).await?;
    Ok(Json(member))
}

/// Update guild
async fn update_guild(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(guild_id): Path<Uuid>,
    Json(req): Json<UpdateGuildRequest>,
) -> ApiResult<Json<Guild>> {
    let guild = state
        .services
        .guild
        .update_guild(player.player_id, guild_id, req)
        .await?;
    Ok(Json(guild))
}

/// Join request input
#[derive(Debug, Deserialize)]
pub struct JoinRequestInput {
    pub message: Option<String>,
}

/// Request to join guild
async fn request_join(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(guild_id): Path<Uuid>,
    Json(req): Json<JoinRequestInput>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .guild
        .request_join(player.player_id, guild_id, req.message)
        .await?;
    Ok(Json("Request sent"))
}

/// Get pending join requests
async fn get_pending_requests(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(guild_id): Path<Uuid>,
) -> ApiResult<Json<Vec<GuildRequestWithPlayer>>> {
    // Verify membership is checked in service
    let requests = state.services.guild.get_pending_requests(guild_id).await?;
    Ok(Json(requests))
}

/// Accept join request
async fn accept_request(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(request_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .guild
        .review_request(player.player_id, request_id, true)
        .await?;
    Ok(Json("Accepted"))
}

/// Reject join request
async fn reject_request(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(request_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .guild
        .review_request(player.player_id, request_id, false)
        .await?;
    Ok(Json("Rejected"))
}

/// Leave guild
async fn leave_guild(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<&'static str>> {
    state.services.guild.leave_guild(player.player_id).await?;
    Ok(Json("Left guild"))
}

/// Kick member
async fn kick_member(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(member_id): Path<Uuid>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .guild
        .kick_member(player.player_id, member_id)
        .await?;
    Ok(Json("Kicked"))
}

/// Change role input
#[derive(Debug, Deserialize)]
pub struct ChangeRoleInput {
    pub role: GuildRole,
}

/// Change member role
async fn change_role(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(member_id): Path<Uuid>,
    Json(req): Json<ChangeRoleInput>,
) -> ApiResult<Json<&'static str>> {
    state
        .services
        .guild
        .change_role(player.player_id, member_id, req.role)
        .await?;
    Ok(Json("Role changed"))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/guild", post(create_guild))
        .route("/guild/me", get(get_my_guild))
        .route("/guild/leave", post(leave_guild))
        .route("/guilds", get(search_guilds))
        .route("/guilds/:guild_id", get(get_guild).put(update_guild))
        .route("/guilds/:guild_id/members", get(get_members))
        .route("/guilds/:guild_id/join", post(request_join))
        .route("/guilds/:guild_id/requests", get(get_pending_requests))
        .route("/guild/requests/:request_id/accept", post(accept_request))
        .route("/guild/requests/:request_id/reject", post(reject_request))
        .route("/guild/members/:member_id/kick", delete(kick_member))
        .route("/guild/members/:member_id/role", put(change_role))
        .with_state(state)
}
