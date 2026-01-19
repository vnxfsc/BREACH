//! Marketplace API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::AppState;
use crate::middleware::auth::AuthPlayer;
use crate::models::{
    AuctionBid, BidResponse, CreateListingRequest, ListingResponse, MakeOfferRequest,
    MarketplaceListing, MarketplaceSearchQuery, MarketplaceStatsResponse, MarketplaceTransaction,
    OfferResponse, PlaceBidRequest, PriceChartResponse, PriceOffer, SearchResultsResponse,
    TransactionHistoryEntry,
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

/// Buy a fixed-price listing
async fn buy_listing(
    State(state): State<Arc<AppState>>,
    AuthPlayer(player): AuthPlayer,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<MarketplaceTransaction>> {
    let tx = state.services.marketplace.buy_listing(player.player_id, id).await?;
    Ok(Json(tx))
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
