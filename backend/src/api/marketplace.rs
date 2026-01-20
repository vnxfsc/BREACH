//! Marketplace API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use uuid::Uuid;
use sqlx;
use anyhow;

use crate::error::{ApiResult, AppError};
use crate::AppState;
use crate::middleware::auth::AuthPlayer;
use crate::models::{
    AuctionBid, BidResponse, CreateListingRequest, ListingResponse, ListingStatus, ListingType,
    MakeOfferRequest, MarketplaceListing, MarketplaceSearchQuery, MarketplaceStatsResponse,
    MarketplaceTransaction, OfferResponse, PlaceBidRequest, PriceChartResponse, PriceOffer,
    SearchResultsResponse, TransactionHistoryEntry,
};

/// Build marketplace routes
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // Listings
        .route("/marketplace", get(search_listings))
        .route("/marketplace/listings", post(create_listing))
        .route("/marketplace/listings/:id", get(get_listing))
        .route("/marketplace/listings/:id", delete(cancel_listing))
        .route("/marketplace/listings/:id/buy", post(buy_listing))
        .route("/marketplace/listings/:id/purchase/build", post(build_purchase_transaction))
        .route("/marketplace/listings/:id/purchase/complete", post(complete_purchase))
        // Auctions
        .route("/marketplace/listings/:id/bids", get(get_bids))
        .route("/marketplace/listings/:id/bids", post(place_bid))
        // Offers
        .route("/marketplace/offers", post(make_offer))
        .route("/marketplace/offers/received", get(get_received_offers))
        .route("/marketplace/offers/sent", get(get_sent_offers))
        .route("/marketplace/offers/:id/accept", post(accept_offer))
        .route("/marketplace/offers/:id/reject", post(reject_offer))
        // Favorites
        .route("/marketplace/favorites", get(get_favorites))
        .route("/marketplace/favorites/:listing_id", post(add_favorite))
        .route("/marketplace/favorites/:listing_id", delete(remove_favorite))
        // My listings
        .route("/marketplace/my-listings", get(get_my_listings))
        // Stats & History
        .route("/marketplace/stats", get(get_stats))
        .route("/marketplace/history", get(get_transaction_history))
        .route("/marketplace/price-chart", get(get_price_chart))
        .with_state(state)
}

// ============================================
// Listing Endpoints
// ============================================

/// Search marketplace listings
async fn search_listings(
    State(state): State<Arc<AppState>>,
    auth: Option<AuthPlayer>,
    Query(query): Query<MarketplaceSearchQuery>,
) -> ApiResult<Json<SearchResultsResponse>> {
    let viewer_id = auth.map(|a| a.0.player_id);
    let results = state.services.marketplace.search_listings(query, viewer_id).await?;
    Ok(Json(results))
}

/// Create a new listing
async fn create_listing(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<CreateListingRequest>,
) -> ApiResult<Json<MarketplaceListing>> {
    let listing = state.services.marketplace.create_listing(player.player_id, req).await?;
    Ok(Json(listing))
}

/// Get listing details
async fn get_listing(
    State(state): State<Arc<AppState>>,
    auth: Option<AuthPlayer>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ListingResponse>> {
    let viewer_id = auth.map(|a| a.0.player_id);
    let listing = state.services.marketplace.get_listing(id, viewer_id).await?;
    Ok(Json(listing))
}

/// Cancel a listing
async fn cancel_listing(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.marketplace.cancel_listing(player.player_id, id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

/// Buy a fixed-price listing (legacy - database only)
async fn buy_listing(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<MarketplaceTransaction>> {
    let tx = state.services.marketplace.buy_listing(player.player_id, id).await?;
    Ok(Json(tx))
}

/// Build on-chain purchase transaction
async fn build_purchase_transaction(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(listing_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    use crate::error::AppError;
    
    // Get listing details
    let listing = state.services.marketplace.get_listing(listing_id, Some(player.player_id)).await?;
    
    // Validate listing
    if listing.status != ListingStatus::Active {
        return Err(AppError::BadRequest("Listing is not active".into()));
    }
    
    if listing.listing_type == ListingType::Auction {
        return Err(AppError::BadRequest("Cannot directly buy auction listings".into()));
    }
    
    if listing.seller_id == player.player_id {
        return Err(AppError::BadRequest("Cannot buy your own listing".into()));
    }
    
    // Get seller wallet address
    let seller_wallet = sqlx::query_scalar::<_, String>(
        "SELECT wallet_address FROM players WHERE id = $1"
    )
    .bind(listing.seller_id)
    .fetch_optional(&state.db.pg)
    .await?
    .ok_or(AppError::NotFound("Seller wallet not found".into()))?;
    
    // Get Titan on-chain ID
    let titan_onchain_id = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT onchain_id FROM player_titans WHERE id = $1"
    )
    .bind(listing.titan_id)
    .fetch_optional(&state.db.pg)
    .await?
    .flatten();
    
    let titan_onchain_id = titan_onchain_id
        .ok_or(AppError::BadRequest("Titan not minted on-chain yet".into()))? as u64;
    
    // Calculate fees
    const PLATFORM_FEE_BPS: i64 = 250; // 2.5%
    let fee = (listing.price * PLATFORM_FEE_BPS) / 10000;
    let total = listing.price;
    
    // Build transfer transaction using Solana service
    let solana = state.services.solana.as_ref()
        .ok_or(AppError::Internal(anyhow::anyhow!("Solana service not available")))?;
    
    let tx_result = solana.build_transfer_transaction(
        &seller_wallet,
        &player.wallet_address,
        titan_onchain_id,
    ).await?;
    
    Ok(Json(serde_json::json!({
        "listing_id": listing_id,
        "titan_id": listing.titan_id,
        "titan_onchain_id": titan_onchain_id,
        "seller_wallet": seller_wallet,
        "buyer_wallet": player.wallet_address,
        "price": listing.price,
        "fee": fee,
        "total": total,
        "serialized_transaction": tx_result.serialized_transaction,
        "message_to_sign": tx_result.message_to_sign,
        "recent_blockhash": tx_result.recent_blockhash,
    })))
}

/// Complete on-chain purchase with signature
/// 
/// **LIMITATION**: Current implementation requires seller signature for transfer.
/// In production, this would need:
/// 1. Escrow contract to hold NFT when listing is created
/// 2. Or program-level marketplace with approval mechanism
/// 3. Or off-chain coordination for seller to sign
/// 
/// For now, this endpoint returns an error guiding users to alternative flows.
async fn complete_purchase(
    State(_state): State<Arc<AppState>>,
    AuthPlayer(_player): AuthPlayer,
    Path(_listing_id): Path<Uuid>,
    Json(_req): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Err(AppError::BadRequest(
        "On-chain marketplace purchases require escrow contract (not yet implemented). \
        Please use off-chain transfer coordination or contact the seller directly.".into()
    ))
    
    // TODO: Implement proper escrow flow:
    // 1. Seller creates listing → NFT transferred to escrow PDA
    // 2. Buyer pays → payment held in escrow
    // 3. On completion → escrow releases NFT to buyer, payment to seller
}

// ============================================
// Auction Endpoints
// ============================================

/// Get bids for a listing
async fn get_bids(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<BidResponse>>> {
    let bids = state.services.marketplace.get_listing_bids(id).await?;
    Ok(Json(bids))
}

/// Place a bid on an auction
async fn place_bid(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(id): Path<Uuid>,
    Json(req): Json<PlaceBidRequest>,
) -> ApiResult<Json<AuctionBid>> {
    let bid = state.services.marketplace.place_bid(player.player_id, id, req.amount).await?;
    Ok(Json(bid))
}

// ============================================
// Offer Endpoints
// ============================================

/// Make an offer on a Titan
async fn make_offer(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Json(req): Json<MakeOfferRequest>,
) -> ApiResult<Json<PriceOffer>> {
    let offer = state.services.marketplace.make_offer(player.player_id, req).await?;
    Ok(Json(offer))
}

/// Get offers received
async fn get_received_offers(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<OfferResponse>>> {
    let offers = state.services.marketplace.get_received_offers(player.player_id).await?;
    Ok(Json(offers))
}

/// Get offers sent
async fn get_sent_offers(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<OfferResponse>>> {
    let offers = state.services.marketplace.get_sent_offers(player.player_id).await?;
    Ok(Json(offers))
}

/// Accept an offer
async fn accept_offer(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<MarketplaceTransaction>> {
    let tx = state.services.marketplace.accept_offer(player.player_id, id).await?;
    Ok(Json(tx))
}

/// Reject an offer
async fn reject_offer(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.marketplace.reject_offer(player.player_id, id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

// ============================================
// Favorites Endpoints
// ============================================

/// Get player's favorites
async fn get_favorites(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<ListingResponse>>> {
    let favorites = state.services.marketplace.get_favorites(player.player_id).await?;
    Ok(Json(favorites))
}

/// Add listing to favorites
async fn add_favorite(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(listing_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.marketplace.add_favorite(player.player_id, listing_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

/// Remove listing from favorites
async fn remove_favorite(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(listing_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.services.marketplace.remove_favorite(player.player_id, listing_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}

// ============================================
// My Listings
// ============================================

/// Get player's listings
async fn get_my_listings(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<ListingResponse>>> {
    let listings = state.services.marketplace.get_my_listings(player.player_id).await?;
    Ok(Json(listings))
}

// ============================================
// Stats & History
// ============================================

/// Get marketplace stats
async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<MarketplaceStatsResponse>> {
    let stats = state.services.marketplace.get_stats().await?;
    Ok(Json(stats))
}

/// Get transaction history
async fn get_transaction_history(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
) -> ApiResult<Json<Vec<TransactionHistoryEntry>>> {
    let history = state.services.marketplace.get_transaction_history(player.player_id).await?;
    Ok(Json(history))
}

/// Price chart query params
#[derive(Debug, serde::Deserialize)]
pub struct PriceChartQuery {
    pub element: Option<crate::models::Element>,
    pub threat_class: Option<i16>,
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}

/// Get price chart data
async fn get_price_chart(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PriceChartQuery>,
) -> ApiResult<Json<PriceChartResponse>> {
    let chart = state.services.marketplace.get_price_chart(
        query.element,
        query.threat_class,
        query.days,
    ).await?;
    Ok(Json(chart))
}
