//! Capture endpoints

use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};

use crate::error::ApiResult;
use crate::middleware::auth::AuthPlayer;
use crate::models::{CaptureAuthorization, CaptureRequest};
use crate::websocket::WsMessage;
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

#[derive(Debug, serde::Serialize)]
pub struct ConfirmCaptureResponse {
    pub success: bool,
    pub titan_id: String,
    pub remaining_captures: i32,
}

async fn confirm_capture(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<ConfirmCaptureRequest>,
) -> ApiResult<Json<ConfirmCaptureResponse>> {
    // Get the titan before confirming (to get geohash for broadcast)
    let titan = state.services.map.get_titan(request.titan_id).await?;

    // Confirm the capture
    state
        .services
        .capture
        .confirm_capture(request.titan_id, player.player_id)
        .await?;

    // Calculate remaining captures
    let remaining_captures = if let Some(ref t) = titan {
        t.max_captures - t.capture_count - 1
    } else {
        0
    };

    // Broadcast capture event via WebSocket
    if let Some(titan) = titan {
        let message = WsMessage::TitanCaptured {
            titan_id: request.titan_id.to_string(),
            captured_by: player.wallet_address.clone(),
            remaining_captures,
        };

        // Broadcast to the titan's region and neighbors
        state
            .broadcaster
            .broadcast_to_neighbors(&titan.geohash, message)
            .await;

        tracing::info!(
            "Player {} captured Titan {} ({} captures remaining)",
            player.wallet_address,
            request.titan_id,
            remaining_captures
        );
    }

    Ok(Json(ConfirmCaptureResponse {
        success: true,
        titan_id: request.titan_id.to_string(),
        remaining_captures,
    }))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/capture/request", post(request_capture))
        .route("/capture/confirm", post(confirm_capture))
        .with_state(state)
}
