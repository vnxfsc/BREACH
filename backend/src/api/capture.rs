//! Capture endpoints

use std::sync::Arc;

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{CaptureAuthorization, CaptureRequest};
use crate::AppState;

/// Request capture authorization
async fn request_capture(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<CaptureRequest>,
) -> ApiResult<Json<CaptureAuthorization>> {
    let auth = state
        .services
        .capture
        .request_capture(player.player_id, &player.wallet_address, request)
        .await?;

    Ok(Json(auth))
}

/// Confirm capture (after blockchain tx)
#[derive(Debug, serde::Deserialize)]
pub struct ConfirmCaptureRequest {
    pub titan_id: uuid::Uuid,
    pub tx_signature: String,
}

async fn confirm_capture(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<ConfirmCaptureRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // TODO: Verify blockchain transaction
    // For now, just confirm the capture

    state
        .services
        .capture
        .confirm_capture(request.titan_id, player.player_id)
        .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "titan_id": request.titan_id,
    })))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/capture/request", post(request_capture))
        .route("/capture/confirm", post(confirm_capture))
        .with_state(state)
}
