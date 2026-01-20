//! Marketplace data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::titan::Element;

// ============================================
// Enums
// ============================================

/// Listing status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "listing_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
    Expired,
}

/// Listing type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "listing_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ListingType {
    FixedPrice,
    Auction,
}

/// Transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Purchase,
    AuctionWin,
    OfferAccepted,
}

/// Offer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfferStatus {
    Pending,
    Accepted,
    Rejected,
    Cancelled,
    Expired,
}

// ============================================
// Database Models
// ============================================

/// Marketplace listing
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MarketplaceListing {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub titan_id: Uuid,
    pub listing_type: ListingType,
    pub price: i64,
    pub min_price: Option<i64>,
    pub status: ListingStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub sold_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub buyer_id: Option<Uuid>,
    pub final_price: Option<i64>,
    pub views: i32,
    pub favorites: i32,
}

/// Auction bid
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AuctionBid {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub bidder_id: Uuid,
    pub amount: i64,
    pub is_winning: bool,
    pub created_at: DateTime<Utc>,
}

/// Marketplace transaction
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MarketplaceTransaction {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub seller_id: Uuid,
    pub buyer_id: Uuid,
    pub titan_id: Uuid,
    pub transaction_type: TransactionType,
    pub price: i64,
    pub fee: i64,
    pub seller_receives: i64,
    pub tx_signature: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Price offer
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct PriceOffer {
    pub id: Uuid,
    pub titan_id: Uuid,
    pub offerer_id: Uuid,
    pub owner_id: Uuid,
    pub amount: i64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
}

/// Listing favorite
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ListingFavorite {
    pub id: Uuid,
    pub player_id: Uuid,
    pub listing_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// ============================================
// API Request/Response Models
// ============================================

/// Create listing request
#[derive(Debug, Deserialize)]
pub struct CreateListingRequest {
    pub titan_id: Uuid,
    pub listing_type: ListingType,
    pub price: i64,
    #[serde(default)]
    pub min_price: Option<i64>,  // For auctions
    #[serde(default = "default_duration_hours")]
    pub duration_hours: i64,  // Listing duration
}

fn default_duration_hours() -> i64 {
    72  // 3 days default
}

/// Listing response with Titan details
#[derive(Debug, Serialize)]
pub struct ListingResponse {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub seller_username: Option<String>,
    pub titan_id: Uuid,
    pub titan: TitanListingInfo,
    pub listing_type: ListingType,
    pub price: i64,
    pub min_price: Option<i64>,
    pub current_bid: Option<i64>,
    pub bid_count: i32,
    pub status: ListingStatus,
    pub expires_at: DateTime<Utc>,
    pub views: i32,
    pub favorites: i32,
    pub is_favorited: bool,
    pub created_at: DateTime<Utc>,
}

/// Titan info for listing display
#[derive(Debug, Serialize)]
pub struct TitanListingInfo {
    pub id: Uuid,
    pub element: Element,
    pub threat_class: i16,
    pub species_id: Option<i32>,
    pub level: i32,
    pub nickname: Option<String>,
    pub genes: Vec<u8>,
}

/// Place bid request
#[derive(Debug, Deserialize)]
pub struct PlaceBidRequest {
    pub amount: i64,
}

/// Bid response
#[derive(Debug, Serialize)]
pub struct BidResponse {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub bidder_id: Uuid,
    pub bidder_username: Option<String>,
    pub amount: i64,
    pub is_winning: bool,
    pub created_at: DateTime<Utc>,
}

/// Make offer request
#[derive(Debug, Deserialize)]
pub struct MakeOfferRequest {
    pub titan_id: Uuid,
    pub amount: i64,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default = "default_offer_hours")]
    pub expires_in_hours: i64,
}

fn default_offer_hours() -> i64 {
    24
}

/// Offer response
#[derive(Debug, Serialize)]
pub struct OfferResponse {
    pub id: Uuid,
    pub titan_id: Uuid,
    pub offerer_id: Uuid,
    pub offerer_username: Option<String>,
    pub owner_id: Uuid,
    pub amount: i64,
    pub status: String,
    pub message: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Marketplace search query
#[derive(Debug, Deserialize)]
pub struct MarketplaceSearchQuery {
    #[serde(default)]
    pub element: Option<Element>,
    #[serde(default)]
    pub min_threat_class: Option<i16>,
    #[serde(default)]
    pub max_threat_class: Option<i16>,
    #[serde(default)]
    pub min_price: Option<i64>,
    #[serde(default)]
    pub max_price: Option<i64>,
    #[serde(default)]
    pub min_level: Option<i32>,
    #[serde(default)]
    pub listing_type: Option<ListingType>,
    #[serde(default)]
    pub sort_by: Option<String>,  // price_asc, price_desc, newest, ending_soon
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

/// Search results response
#[derive(Debug, Serialize)]
pub struct SearchResultsResponse {
    pub listings: Vec<ListingResponse>,
    pub total_count: i64,
    pub has_more: bool,
}

/// Transaction history entry
#[derive(Debug, Serialize)]
pub struct TransactionHistoryEntry {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub transaction_type: TransactionType,
    pub price: i64,
    pub fee: i64,
    pub counterparty_id: Uuid,
    pub counterparty_username: Option<String>,
    pub titan_element: Option<Element>,
    pub titan_level: Option<i32>,
    pub is_seller: bool,
    pub created_at: DateTime<Utc>,
}

/// Marketplace stats response
#[derive(Debug, Serialize)]
pub struct MarketplaceStatsResponse {
    pub total_listings: i32,
    pub active_listings: i32,
    pub total_volume_24h: i64,
    pub total_sales_24h: i32,
    pub floor_price: Option<i64>,
    pub avg_price: Option<i64>,
}

/// Price history entry
#[derive(Debug, Serialize, FromRow)]
pub struct PriceHistoryEntry {
    pub price: i64,
    pub recorded_at: DateTime<Utc>,
}

/// Price chart response
#[derive(Debug, Serialize)]
pub struct PriceChartResponse {
    pub element: Option<Element>,
    pub threat_class: Option<i16>,
    pub period: String,
    pub data_points: Vec<PriceHistoryEntry>,
    pub avg_price: i64,
    pub min_price: i64,
    pub max_price: i64,
}

/// Purchase transaction response (for on-chain purchases)
#[derive(Debug, Serialize)]
pub struct PurchaseTransactionResponse {
    pub listing_id: Uuid,
    pub titan_id: Uuid,
    pub titan_onchain_id: Option<u64>,  // On-chain Titan ID
    pub seller_wallet: String,
    pub buyer_wallet: String,
    pub price: i64,
    pub fee: i64,
    pub total: i64,
    pub serialized_transaction: String,
    pub message_to_sign: String,
    pub recent_blockhash: String,
}

/// Purchase completion request
#[derive(Debug, Deserialize)]
pub struct CompletePurchaseRequest {
    pub serialized_transaction: String,
    pub user_signature: String,
}
