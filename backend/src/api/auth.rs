//! Authentication endpoints

use std::sync::Arc;

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::error::{ApiResult, AppError};
use crate::services::auth::{AuthChallenge, AuthRequest, AuthResponse};
use crate::AppState;

/// Request a challenge for wallet authentication
#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    pub wallet_address: String,
}

async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChallengeRequest>,
) -> ApiResult<Json<AuthChallenge>> {
    let challenge = state.services.auth.generate_challenge(&req.wallet_address);
    Ok(Json(challenge))
}

/// Authenticate with signed message
async fn authenticate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Verify signature
    state
        .services
        .auth
        .verify_signature(&req.wallet_address, &req.message, &req.signature)?;

    // Get or create player
    let player = state
        .services
        .player
        .get_or_create(&req.wallet_address)
        .await?;

    // Check if banned
    if player.is_banned {
        return Err(AppError::Unauthorized);
    }

    // Generate token
    let token = state
        .services
        .auth
        .generate_token(player.id, &req.wallet_address)?;

    let expires_at = chrono::Utc::now().timestamp() + (state.config.auth.jwt_expiry_hours as i64 * 3600);

    Ok(Json(AuthResponse {
        token,
        expires_at,
        player_id: player.id.to_string(),
    }))
}

/// Refresh token (requires valid token in header)
async fn refresh_token(
    State(_state): State<Arc<AppState>>,
    // Would need to extract from header
) -> ApiResult<Json<AuthResponse>> {
    // TODO: Implement token refresh
    Err(AppError::NotFound)
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/auth/challenge", post(get_challenge))
        .route("/auth/authenticate", post(authenticate))
        .route("/auth/refresh", post(refresh_token))
        .with_state(state)
}
