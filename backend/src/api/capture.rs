//! Capture endpoints
//!
//! Provides two minting modes:
//! 1. Backend-paid mode (testing): POST /capture/confirm
//! 2. Frontend-signed mode (production): POST /capture/build-transaction + POST /capture/submit-transaction

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

// ═══════════════════════════════════════════════════════════════════════════════
// Production API (frontend signing flow)
// ═══════════════════════════════════════════════════════════════════════════════

/// Build mint transaction request.
#[derive(Debug, Deserialize)]
pub struct BuildMintTransactionRequest {
    /// Titan spawn ID (UUID in database).
    pub titan_id: uuid::Uuid,
    /// Capture latitude (degrees).
    pub capture_lat: f64,
    /// Capture longitude (degrees).
    pub capture_lng: f64,
}

/// Build mint transaction response.
#[derive(Debug, Serialize)]
pub struct BuildMintTransactionResponse {
    /// Base64-encoded serialized transaction (with empty signature slots, bincode format).
    pub serialized_transaction: String,
    /// Base64-encoded message bytes (for frontend signing).
    pub message_to_sign: String,
    /// Recent blockhash.
    pub recent_blockhash: String,
    /// Titan PDA address (NFT address).
    pub titan_pda: String,
    /// Player PDA address.
    pub player_pda: String,
    /// On-chain Titan ID.
    pub titan_id: u64,
    /// Titan info (for client display).
    pub titan_info: TitanMintInfo,
}

/// Titan mint info.
#[derive(Debug, Serialize)]
pub struct TitanMintInfo {
    pub species_id: i32,
    pub element: String,
    pub threat_class: i16,
}

/// Build mint transaction.
///
/// Returns an unsigned transaction message for the wallet to sign.
async fn build_mint_transaction(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<BuildMintTransactionRequest>,
) -> ApiResult<Json<BuildMintTransactionResponse>> {
    // Load Titan info.
    let titan = state.services.map.get_titan(request.titan_id).await?
        .ok_or(AppError::TitanNotFound)?;

    // Validate Titan can be captured.
    if titan.captured_by.is_some() && titan.capture_count >= titan.max_captures {
        return Err(AppError::TitanAlreadyCaptured);
    }

    // Require Solana service.
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    // Convert genes.
    let mut genes_array = [0u8; 32];
    let len = titan.genes.len().min(32);
    genes_array[..len].copy_from_slice(&titan.genes[..len]);

    // Convert coordinates (degrees -> 10^6 integer).
    let capture_lat = (request.capture_lat * 1_000_000.0) as i32;
    let capture_lng = (request.capture_lng * 1_000_000.0) as i32;

    // Build transaction.
    let result = solana.build_mint_transaction(
        &player.wallet_address,
        titan.element,
        titan.threat_class as u8,
        titan.species_id as u32,
        genes_array,
        capture_lat,
        capture_lng,
    ).await?;

    tracing::info!(
        "Built mint transaction for player {} to capture Titan {}",
        player.wallet_address, request.titan_id
    );

    Ok(Json(BuildMintTransactionResponse {
        serialized_transaction: result.serialized_transaction,
        message_to_sign: result.message_to_sign,
        recent_blockhash: result.recent_blockhash,
        titan_pda: result.titan_pda,
        player_pda: result.player_pda,
        titan_id: result.titan_id,
        titan_info: TitanMintInfo {
            species_id: titan.species_id,
            element: format!("{:?}", titan.element),
            threat_class: titan.threat_class,
        },
    }))
}

/// Submit signed transaction request.
#[derive(Debug, Deserialize)]
pub struct SubmitSignedTransactionRequest {
    /// Base64-encoded player signature (64 bytes).
    pub player_signature: String,
    /// Base64-encoded original unsigned transaction.
    pub serialized_transaction: String,
    /// Titan spawn ID (UUID in database).
    pub titan_id: uuid::Uuid,
    /// Titan PDA address (returned from `build-transaction`).
    pub titan_pda: String,
}

/// Submit signed transaction response.
#[derive(Debug, Serialize)]
pub struct SubmitSignedTransactionResponse {
    pub success: bool,
    /// Transaction signature.
    pub tx_signature: String,
    /// Titan PDA (NFT address).
    pub mint_address: String,
    /// Remaining captures.
    pub remaining_captures: i32,
    /// BREACH reward amount.
    pub breach_reward: Option<u64>,
    /// BREACH reward transaction signature.
    pub breach_tx_signature: Option<String>,
}

/// Submit a signed mint transaction.
///
/// Accepts the frontend-signed transaction, adds backend signature and broadcasts it.
async fn submit_signed_transaction(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(request): Json<SubmitSignedTransactionRequest>,
) -> ApiResult<Json<SubmitSignedTransactionResponse>> {
    // Load Titan info.
    let titan = state.services.map.get_titan(request.titan_id).await?
        .ok_or(AppError::TitanNotFound)?;

    // Validate Titan can be captured.
    if titan.captured_by.is_some() && titan.capture_count >= titan.max_captures {
        return Err(AppError::TitanAlreadyCaptured);
    }

    // Require Solana service.
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;

    // Submit transaction.
    let result = solana.submit_signed_transaction(
        &request.serialized_transaction,
        &request.player_signature,
        &player.wallet_address,
    ).await?;

    tracing::info!(
        "NFT minted via signed transaction: player={}, sig={}",
        player.wallet_address, result.signature
    );

    // Distribute BREACH rewards.
    let mut breach_reward = None;
    let mut breach_tx_signature = None;
    
    let reward_amount = calculate_breach_reward(titan.threat_class);
    if reward_amount > 0 {
        match solana.transfer_breach_tokens(&player.wallet_address, reward_amount).await {
            Ok(transfer_result) => {
                breach_reward = Some(transfer_result.amount);
                breach_tx_signature = Some(transfer_result.signature);
                tracing::info!(
                    "BREACH reward distributed: player={}, amount={}",
                    player.wallet_address, reward_amount
                );
            }
            Err(e) => {
                tracing::warn!("Failed to distribute BREACH reward: {}", e);
            }
        }
    }

    // Record capture in database.
    state.services.capture
        .confirm_capture(request.titan_id, player.player_id)
        .await?;

    // Compute remaining captures.
    let remaining_captures = titan.max_captures - titan.capture_count - 1;

    // Broadcast WebSocket events.
    let message = WsMessage::TitanCaptured {
        titan_id: request.titan_id.to_string(),
        captured_by: player.wallet_address.clone(),
        remaining_captures,
    };
    state.broadcaster.broadcast_to_neighbors(&titan.geohash, message).await;

    tracing::info!(
        "Player {} captured Titan {} via signed transaction ({} captures remaining)",
        player.wallet_address, request.titan_id, remaining_captures
    );

    Ok(Json(SubmitSignedTransactionResponse {
        success: true,
        tx_signature: result.signature,
        mint_address: request.titan_pda, // From request payload
        remaining_captures,
        breach_reward,
        breach_tx_signature,
    }))
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Core endpoints
        .route("/capture/request", post(request_capture))
        // Testing endpoint (backend-paid)
        .route("/capture/confirm", post(confirm_capture))
        // Production endpoints (frontend-signed)
        .route("/capture/build-transaction", post(build_mint_transaction))
        .route("/capture/submit-transaction", post(submit_signed_transaction))
        .with_state(state)
}
