//! Solana blockchain API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::{ApiResult, AppError};
use crate::middleware::auth::AuthPlayer;
use crate::models::Element;
use crate::AppState;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Wallet info
        .route("/solana/balance/:address", get(get_sol_balance))
        .route("/solana/breach-balance/:address", get(get_breach_balance))
        // NFT operations
        .route("/solana/mint-titan", post(mint_titan))
        // Token operations
        .route("/solana/transfer-breach", post(transfer_breach))
        // Transaction status
        .route("/solana/transaction/:signature", get(get_transaction_status))
        // Backend wallet info
        .route("/solana/backend-info", get(get_backend_info))
        .with_state(state)
}

// ========================================
// Request/Response Types
// ========================================

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
    pub balance_sol: f64,
}

#[derive(Debug, Serialize)]
pub struct BreachBalanceResponse {
    pub address: String,
    pub balance: u64,
    pub balance_formatted: String,
}

#[derive(Debug, Deserialize)]
pub struct MintTitanRequest {
    pub element: String,
    pub threat_class: u8,
    pub species_id: u32,
    #[serde(default)]
    pub genes: Option<String>, // Base64 encoded, optional (will generate random if not provided)
}

#[derive(Debug, Serialize)]
pub struct MintTitanResponse {
    pub success: bool,
    pub signature: String,
    pub mint_address: String,
    pub token_account: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferBreachRequest {
    pub recipient: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct TransferBreachResponse {
    pub success: bool,
    pub signature: String,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct TransactionStatusResponse {
    pub signature: String,
    pub confirmed: Option<bool>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct BackendInfoResponse {
    pub wallet_address: String,
    pub titan_program_id: String,
    pub game_program_id: String,
    pub breach_token_mint: String,
    pub network: String,
}

// ========================================
// Handlers
// ========================================

/// Get SOL balance for an address
async fn get_sol_balance(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> ApiResult<Json<BalanceResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Solana service not available".into()))?;

    let balance = solana.get_balance(&address).await?;
    
    Ok(Json(BalanceResponse {
        address,
        balance,
        balance_sol: balance as f64 / 1_000_000_000.0, // Convert lamports to SOL
    }))
}

/// Get $BREACH token balance for an address
async fn get_breach_balance(
    State(state): State<Arc<AppState>>,
    Path(address): Path<String>,
) -> ApiResult<Json<BreachBalanceResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Solana service not available".into()))?;

    let balance = solana.get_breach_balance(&address).await?;
    
    // $BREACH has 9 decimals
    let formatted = format!("{:.9} BREACH", balance as f64 / 1_000_000_000.0);
    
    Ok(Json(BreachBalanceResponse {
        address,
        balance,
        balance_formatted: formatted,
    }))
}

/// Mint a new Titan NFT for the authenticated player
async fn mint_titan(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<MintTitanRequest>,
) -> ApiResult<Json<MintTitanResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Solana service not available".into()))?;

    // Parse element
    let element = match req.element.to_lowercase().as_str() {
        "abyssal" => Element::Abyssal,
        "volcanic" => Element::Volcanic,
        "storm" => Element::Storm,
        "void" => Element::Void,
        "parasitic" => Element::Parasitic,
        "ossified" => Element::Ossified,
        _ => return Err(AppError::BadRequest(format!("Invalid element: {}", req.element))),
    };

    // Validate threat class
    if req.threat_class < 1 || req.threat_class > 5 {
        return Err(AppError::BadRequest("Threat class must be 1-5".into()));
    }

    // Parse or generate genes
    let genes: [u8; 32] = if let Some(genes_b64) = req.genes {
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &genes_b64,
        ).map_err(|_| AppError::BadRequest("Invalid genes encoding".into()))?;
        
        if decoded.len() != 32 {
            return Err(AppError::BadRequest("Genes must be 32 bytes".into()));
        }
        
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&decoded);
        arr
    } else {
        // Generate random genes
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut arr = [0u8; 32];
        rng.fill(&mut arr);
        arr
    };

    // Mint the NFT
    let result = solana.mint_titan_nft(
        &player.wallet_address,
        element,
        req.threat_class,
        req.species_id,
        genes,
    ).await?;

    // Log the mint
    tracing::info!(
        "Titan NFT minted: player={}, mint={}, element={:?}",
        player.wallet_address,
        result.mint_address,
        element
    );

    Ok(Json(MintTitanResponse {
        success: true,
        signature: result.signature,
        mint_address: result.mint_address,
        token_account: result.token_account,
    }))
}

/// Transfer $BREACH tokens (admin only for now)
async fn transfer_breach(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<TransferBreachRequest>,
) -> ApiResult<Json<TransferBreachResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Solana service not available".into()))?;

    // For now, only allow transfers from backend to players (rewards)
    // In the future, this could be expanded for player-to-player transfers
    
    if req.amount == 0 {
        return Err(AppError::BadRequest("Amount must be greater than 0".into()));
    }

    let result = solana.transfer_breach_tokens(&req.recipient, req.amount).await?;

    tracing::info!(
        "BREACH tokens transferred: to={}, amount={}, by={}",
        req.recipient,
        req.amount,
        player.wallet_address
    );

    Ok(Json(TransferBreachResponse {
        success: true,
        signature: result.signature,
        amount: result.amount,
    }))
}

/// Get transaction status
async fn get_transaction_status(
    State(state): State<Arc<AppState>>,
    Path(signature): Path<String>,
) -> ApiResult<Json<TransactionStatusResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Solana service not available".into()))?;

    let confirmed = solana.get_transaction_status(&signature).await?;
    
    let status = match confirmed {
        Some(true) => "confirmed".to_string(),
        Some(false) => "failed".to_string(),
        None => "pending".to_string(),
    };

    Ok(Json(TransactionStatusResponse {
        signature,
        confirmed,
        status,
    }))
}

/// Get backend wallet info
async fn get_backend_info(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<BackendInfoResponse>> {
    let solana = state.services.solana.as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Solana service not available".into()))?;

    let network = if state.config.solana.rpc_url.contains("devnet") {
        "devnet"
    } else if state.config.solana.rpc_url.contains("mainnet") {
        "mainnet-beta"
    } else {
        "unknown"
    };

    Ok(Json(BackendInfoResponse {
        wallet_address: solana.backend_pubkey().to_string(),
        titan_program_id: state.config.solana.titan_program_id.clone(),
        game_program_id: state.config.solana.game_program_id.clone(),
        breach_token_mint: state.config.solana.breach_token_mint.clone(),
        network: network.to_string(),
    }))
}
