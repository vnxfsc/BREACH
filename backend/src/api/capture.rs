//! Capture endpoints

use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};
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

/// Confirm capture request - now mints NFT on-chain
#[derive(Debug, Deserialize)]
pub struct ConfirmCaptureRequest {
    pub titan_id: uuid::Uuid,
    #[serde(default)]
    pub skip_blockchain: bool, // For testing without blockchain
}

#[derive(Debug, Serialize)]
pub struct ConfirmCaptureResponse {
    pub success: bool,
    pub titan_id: String,
    pub remaining_captures: i32,
    // Blockchain details
    pub mint_address: Option<String>,
    pub token_account: Option<String>,
    pub tx_signature: Option<String>,
    pub breach_reward: Option<u64>,
    pub breach_tx_signature: Option<String>,
}

async fn confirm_capture(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<ConfirmCaptureRequest>,
) -> ApiResult<Json<ConfirmCaptureResponse>> {
    // Get the titan before confirming (to get data for minting)
    let titan = state.services.map.get_titan(request.titan_id).await?
        .ok_or(AppError::TitanNotFound)?;

    // Verify titan is capturable
    if titan.captured_by.is_some() && titan.capture_count >= titan.max_captures {
        return Err(AppError::TitanAlreadyCaptured);
    }

    // Initialize response fields
    let mut mint_address = None;
    let mut token_account = None;
    let mut tx_signature = None;
    let mut breach_reward = None;
    let mut breach_tx_signature = None;

    // Mint NFT on Solana (if blockchain enabled)
    if !request.skip_blockchain {
        if let Some(solana) = &state.services.solana {
            // Convert genes from Vec<u8> to [u8; 32]
            let mut genes_array = [0u8; 32];
            let len = titan.genes.len().min(32);
            genes_array[..len].copy_from_slice(&titan.genes[..len]);

            // Mint the Titan NFT
            match solana.mint_titan_nft(
                &player.wallet_address,
                titan.element,
                titan.threat_class as u8,
                titan.species_id as u32,
                genes_array,
            ).await {
                Ok(result) => {
                    mint_address = Some(result.mint_address.clone());
                    token_account = Some(result.token_account.clone());
                    tx_signature = Some(result.signature.clone());

                    tracing::info!(
                        "NFT minted: player={}, mint={}, sig={}",
                        player.wallet_address,
                        result.mint_address,
                        result.signature
                    );

                    // Record capture on Game Logic contract
                    if let Err(e) = solana.record_capture(
                        &player.wallet_address,
                        &result.mint_address,
                        &titan.geohash,
                    ).await {
                        tracing::warn!("Failed to record capture on-chain: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to mint NFT: {}", e);
                    // Continue without minting - don't fail the capture
                }
            }

            // Calculate and distribute $BREACH reward based on threat class
            let reward_amount = calculate_breach_reward(titan.threat_class);
            if reward_amount > 0 {
                match solana.transfer_breach_tokens(&player.wallet_address, reward_amount).await {
                    Ok(result) => {
                        breach_reward = Some(result.amount);
                        breach_tx_signature = Some(result.signature);
                        
                        tracing::info!(
                            "BREACH reward distributed: player={}, amount={}",
                            player.wallet_address,
                            reward_amount
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Failed to distribute BREACH reward: {}", e);
                    }
                }
            }
        }
    }

    // Confirm the capture in database
    state
        .services
        .capture
        .confirm_capture(request.titan_id, player.player_id)
        .await?;

    // Calculate remaining captures
    let remaining_captures = titan.max_captures - titan.capture_count - 1;

    // Broadcast capture event via WebSocket
    let message = WsMessage::TitanCaptured {
        titan_id: request.titan_id.to_string(),
        captured_by: player.wallet_address.clone(),
        remaining_captures,
    };

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

    Ok(Json(ConfirmCaptureResponse {
        success: true,
        titan_id: request.titan_id.to_string(),
        remaining_captures,
        mint_address,
        token_account,
        tx_signature,
        breach_reward,
        breach_tx_signature,
    }))
}

/// Calculate BREACH reward based on threat class
/// Higher threat class = higher reward
fn calculate_breach_reward(threat_class: i16) -> u64 {
    // Base reward in smallest unit (9 decimals)
    // 1 BREACH = 1_000_000_000
    const BASE_REWARD: u64 = 100_000_000; // 0.1 BREACH
    
    match threat_class {
        1 => BASE_REWARD * 1,      // 0.1 BREACH
        2 => BASE_REWARD * 3,      // 0.3 BREACH
        3 => BASE_REWARD * 10,     // 1 BREACH
        4 => BASE_REWARD * 50,     // 5 BREACH
        5 => BASE_REWARD * 200,    // 20 BREACH (Legendary)
        _ => BASE_REWARD,
    }
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/capture/request", post(request_capture))
        .route("/capture/confirm", post(confirm_capture))
        .with_state(state)
}
