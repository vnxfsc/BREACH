//! Authentication middleware

use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::error::AppError;
use crate::models::PlayerSession;
use crate::AppState;

/// Extractor for authenticated player
pub struct AuthPlayer(pub PlayerSession);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthPlayer {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Get Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // Parse Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // Verify token
        let session = state.services.auth.verify_token(token)?;

        Ok(AuthPlayer(session))
    }
}

/// Optional auth extractor (for endpoints that work with or without auth)
pub struct OptionalAuthPlayer(pub Option<PlayerSession>);

#[async_trait]
impl FromRequestParts<Arc<AppState>> for OptionalAuthPlayer {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Try to get auth, but don't fail if not present
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        if let Some(header) = auth_header {
            if let Some(token) = header.strip_prefix("Bearer ") {
                if let Ok(session) = state.services.auth.verify_token(token) {
                    return Ok(OptionalAuthPlayer(Some(session)));
                }
            }
        }

        Ok(OptionalAuthPlayer(None))
    }
}
