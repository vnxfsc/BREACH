//! Marketplace service - NFT trading functionality

use chrono::{Duration, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::db::Database;
use crate::error::{ApiResult, AppError};
use crate::models::{
    AuctionBid, BidResponse, CreateListingRequest, Element, ListingResponse, ListingStatus,
    ListingType, MakeOfferRequest, MarketplaceListing, MarketplaceSearchQuery,
    MarketplaceStatsResponse, MarketplaceTransaction, OfferResponse, PriceChartResponse,
    PriceHistoryEntry, PriceOffer, SearchResultsResponse, TitanListingInfo,
    TransactionHistoryEntry,
};

/// Platform fee in basis points (250 = 2.5%)
const PLATFORM_FEE_BPS: i64 = 250;

/// Marketplace service
#[derive(Clone)]
pub struct MarketplaceService {
    db: Database,
}

impl MarketplaceService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // ============================================
    // Listings
    // ============================================

    /// Create a new listing
    pub async fn create_listing(
        &self,
        seller_id: Uuid,
        req: CreateListingRequest,
    ) -> ApiResult<MarketplaceListing> {
        // 验证 Titan 所有权
        let titan = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM player_titans WHERE id = $1 AND player_id = $2"
        )
        .bind(req.titan_id)
        .bind(seller_id)
        .fetch_optional(&self.db.pg)
        .await?;

        if titan.is_none() {
            return Err(AppError::NotFound("Titan not found or not owned by you".into()));
        }

        // 检查是否已有活跃挂单
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM marketplace_listings WHERE titan_id = $1 AND status = 'active'"
        )
        .bind(req.titan_id)
        .fetch_one(&self.db.pg)
        .await?;

        if existing > 0 {
            return Err(AppError::BadRequest("Titan is already listed".into()));
        }

        // 验证拍卖参数
        if req.listing_type == ListingType::Auction && req.min_price.is_none() {
            return Err(AppError::BadRequest("Auction requires min_price".into()));
        }

        // 创建挂单
        let expires_at = Utc::now() + Duration::hours(req.duration_hours);
        
        let listing = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            INSERT INTO marketplace_listings 
            (seller_id, titan_id, listing_type, price, min_price, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(seller_id)
        .bind(req.titan_id)
        .bind(req.listing_type)
        .bind(req.price)
        .bind(req.min_price)
        .bind(expires_at)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(listing)
    }

    /// Get listing by ID
    pub async fn get_listing(
        &self,
        listing_id: Uuid,
        viewer_id: Option<Uuid>,
    ) -> ApiResult<ListingResponse> {
        // 增加浏览量
        sqlx::query("UPDATE marketplace_listings SET views = views + 1 WHERE id = $1")
            .bind(listing_id)
            .execute(&self.db.pg)
            .await?;

        // 获取挂单详情
        let row = sqlx::query(
            r#"
            SELECT 
                l.id, l.seller_id, l.titan_id, l.listing_type, l.price, l.min_price,
                l.status, l.expires_at, l.views, l.favorites, l.created_at,
                p.username as seller_username,
                pt.element, pt.threat_class, pt.species_id, pt.level, pt.nickname, pt.genes,
                COALESCE((SELECT MAX(amount) FROM auction_bids WHERE listing_id = l.id), 0) as current_bid,
                (SELECT COUNT(*) FROM auction_bids WHERE listing_id = l.id)::INT as bid_count,
                CASE WHEN $2::UUID IS NOT NULL THEN
                    EXISTS(SELECT 1 FROM listing_favorites WHERE listing_id = l.id AND player_id = $2)
                ELSE FALSE END as is_favorited
            FROM marketplace_listings l
            JOIN players p ON l.seller_id = p.id
            JOIN player_titans pt ON l.titan_id = pt.id
            WHERE l.id = $1
            "#
        )
        .bind(listing_id)
        .bind(viewer_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let row = row.ok_or_else(|| AppError::NotFound("Listing not found".into()))?;

        Ok(ListingResponse {
            id: row.get("id"),
            seller_id: row.get("seller_id"),
            seller_username: row.get("seller_username"),
            titan_id: row.get("titan_id"),
            titan: TitanListingInfo {
                id: row.get("titan_id"),
                element: row.get("element"),
                threat_class: row.get("threat_class"),
                species_id: row.get("species_id"),
                level: row.get("level"),
                nickname: row.get("nickname"),
                genes: row.get("genes"),
            },
            listing_type: row.get("listing_type"),
            price: row.get("price"),
            min_price: row.get("min_price"),
            current_bid: {
                let bid: i64 = row.get("current_bid");
                if bid > 0 { Some(bid) } else { None }
            },
            bid_count: row.get("bid_count"),
            status: row.get("status"),
            expires_at: row.get("expires_at"),
            views: row.get("views"),
            favorites: row.get("favorites"),
            is_favorited: row.get("is_favorited"),
            created_at: row.get("created_at"),
        })
    }

    /// Search listings
    pub async fn search_listings(
        &self,
        query: MarketplaceSearchQuery,
        viewer_id: Option<Uuid>,
    ) -> ApiResult<SearchResultsResponse> {
        // 构建动态查询
        let mut sql = String::from(
            r#"
            SELECT 
                l.id, l.seller_id, l.titan_id, l.listing_type, l.price, l.min_price,
                l.status, l.expires_at, l.views, l.favorites, l.created_at,
                p.username as seller_username,
                pt.element, pt.threat_class, pt.species_id, pt.level, pt.nickname, pt.genes,
                COALESCE((SELECT MAX(amount) FROM auction_bids WHERE listing_id = l.id), 0) as current_bid,
                (SELECT COUNT(*) FROM auction_bids WHERE listing_id = l.id)::INT as bid_count,
                CASE WHEN $1::UUID IS NOT NULL THEN
                    EXISTS(SELECT 1 FROM listing_favorites WHERE listing_id = l.id AND player_id = $1)
                ELSE FALSE END as is_favorited
            FROM marketplace_listings l
            JOIN players p ON l.seller_id = p.id
            JOIN player_titans pt ON l.titan_id = pt.id
            WHERE l.status = 'active'
            "#
        );

        let mut conditions = Vec::new();

        if query.element.is_some() {
            conditions.push("pt.element = $2".to_string());
        }
        if query.min_threat_class.is_some() {
            conditions.push(format!("pt.threat_class >= ${}", conditions.len() + 2));
        }
        if query.max_threat_class.is_some() {
            conditions.push(format!("pt.threat_class <= ${}", conditions.len() + 2));
        }
        if query.min_price.is_some() {
            conditions.push(format!("l.price >= ${}", conditions.len() + 2));
        }
        if query.max_price.is_some() {
            conditions.push(format!("l.price <= ${}", conditions.len() + 2));
        }
        if query.min_level.is_some() {
            conditions.push(format!("pt.level >= ${}", conditions.len() + 2));
        }
        if query.listing_type.is_some() {
            conditions.push(format!("l.listing_type = ${}", conditions.len() + 2));
        }

        for cond in &conditions {
            sql.push_str(" AND ");
            sql.push_str(cond);
        }

        // 排序
        let order = match query.sort_by.as_deref() {
            Some("price_asc") => "l.price ASC",
            Some("price_desc") => "l.price DESC",
            Some("ending_soon") => "l.expires_at ASC",
            _ => "l.created_at DESC",
        };
        sql.push_str(&format!(" ORDER BY {} LIMIT {} OFFSET {}", order, query.limit + 1, query.offset));

        // 执行查询（简化版本，实际应使用参数化查询）
        let rows = sqlx::query(&sql)
            .bind(viewer_id)
            .bind(query.element)
            .bind(query.min_threat_class)
            .bind(query.max_threat_class)
            .bind(query.min_price)
            .bind(query.max_price)
            .bind(query.min_level)
            .bind(query.listing_type)
            .fetch_all(&self.db.pg)
            .await?;

        let has_more = rows.len() as i64 > query.limit;
        let listings: Vec<ListingResponse> = rows
            .into_iter()
            .take(query.limit as usize)
            .map(|row| ListingResponse {
                id: row.get("id"),
                seller_id: row.get("seller_id"),
                seller_username: row.get("seller_username"),
                titan_id: row.get("titan_id"),
                titan: TitanListingInfo {
                    id: row.get("titan_id"),
                    element: row.get("element"),
                    threat_class: row.get("threat_class"),
                    species_id: row.get("species_id"),
                    level: row.get("level"),
                    nickname: row.get("nickname"),
                    genes: row.get("genes"),
                },
                listing_type: row.get("listing_type"),
                price: row.get("price"),
                min_price: row.get("min_price"),
                current_bid: {
                    let bid: i64 = row.get("current_bid");
                    if bid > 0 { Some(bid) } else { None }
                },
                bid_count: row.get("bid_count"),
                status: row.get("status"),
                expires_at: row.get("expires_at"),
                views: row.get("views"),
                favorites: row.get("favorites"),
                is_favorited: row.get("is_favorited"),
                created_at: row.get("created_at"),
            })
            .collect();

        // 获取总数
        let total_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM marketplace_listings WHERE status = 'active'"
        )
        .fetch_one(&self.db.pg)
        .await?;

        Ok(SearchResultsResponse {
            listings,
            total_count,
            has_more,
        })
    }

    /// Cancel listing
    pub async fn cancel_listing(&self, seller_id: Uuid, listing_id: Uuid) -> ApiResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE marketplace_listings
            SET status = 'cancelled', cancelled_at = NOW()
            WHERE id = $1 AND seller_id = $2 AND status = 'active'
            "#
        )
        .bind(listing_id)
        .bind(seller_id)
        .execute(&self.db.pg)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Listing not found or already sold".into()));
        }

        Ok(())
    }

    // ============================================
    // Purchases
    // ============================================

    /// Buy a fixed-price listing
    pub async fn buy_listing(&self, buyer_id: Uuid, listing_id: Uuid) -> ApiResult<MarketplaceTransaction> {
        let mut tx = self.db.pg.begin().await?;

        // 获取并锁定挂单
        let listing = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            SELECT * FROM marketplace_listings
            WHERE id = $1 AND status = 'active' AND listing_type = 'fixed_price'
            FOR UPDATE
            "#
        )
        .bind(listing_id)
        .fetch_optional(&mut *tx)
        .await?;

        let listing = listing.ok_or_else(|| AppError::NotFound("Listing not found or not available".into()))?;

        // 不能购买自己的
        if listing.seller_id == buyer_id {
            return Err(AppError::BadRequest("Cannot buy your own listing".into()));
        }

        // 计算费用
        let fee = (listing.price * PLATFORM_FEE_BPS) / 10000;
        let seller_receives = listing.price - fee;

        // 更新挂单状态
        sqlx::query(
            r#"
            UPDATE marketplace_listings
            SET status = 'sold', sold_at = NOW(), buyer_id = $1, final_price = $2
            WHERE id = $3
            "#
        )
        .bind(buyer_id)
        .bind(listing.price)
        .bind(listing_id)
        .execute(&mut *tx)
        .await?;

        // 转移 Titan 所有权
        sqlx::query(
            "UPDATE player_titans SET player_id = $1 WHERE id = $2"
        )
        .bind(buyer_id)
        .bind(listing.titan_id)
        .execute(&mut *tx)
        .await?;

        // 创建交易记录
        let transaction = sqlx::query_as::<_, MarketplaceTransaction>(
            r#"
            INSERT INTO marketplace_transactions
            (listing_id, seller_id, buyer_id, titan_id, transaction_type, price, fee, seller_receives)
            VALUES ($1, $2, $3, $4, 'purchase', $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(listing_id)
        .bind(listing.seller_id)
        .bind(buyer_id)
        .bind(listing.titan_id)
        .bind(listing.price)
        .bind(fee)
        .bind(seller_receives)
        .fetch_one(&mut *tx)
        .await?;

        // 记录价格历史
        sqlx::query(
            r#"
            INSERT INTO price_history (element, threat_class, species_id, price, transaction_type)
            SELECT pt.element, pt.threat_class, pt.species_id, $1, 'purchase'
            FROM player_titans pt WHERE pt.id = $2
            "#
        )
        .bind(listing.price)
        .bind(listing.titan_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(transaction)
    }

    // ============================================
    // Auctions
    // ============================================

    /// Place a bid on an auction
    pub async fn place_bid(
        &self,
        bidder_id: Uuid,
        listing_id: Uuid,
        amount: i64,
    ) -> ApiResult<AuctionBid> {
        let mut tx = self.db.pg.begin().await?;

        // 获取并锁定挂单
        let listing = sqlx::query_as::<_, MarketplaceListing>(
            r#"
            SELECT * FROM marketplace_listings
            WHERE id = $1 AND status = 'active' AND listing_type = 'auction'
            FOR UPDATE
            "#
        )
        .bind(listing_id)
        .fetch_optional(&mut *tx)
        .await?;

        let listing = listing.ok_or_else(|| AppError::NotFound("Auction not found".into()))?;

        // 验证
        if listing.seller_id == bidder_id {
            return Err(AppError::BadRequest("Cannot bid on your own auction".into()));
        }

        if listing.expires_at < Utc::now() {
            return Err(AppError::BadRequest("Auction has ended".into()));
        }

        // 检查最低出价
        let min_price = listing.min_price.unwrap_or(listing.price);
        if amount < min_price {
            return Err(AppError::BadRequest(format!("Bid must be at least {}", min_price)));
        }

        // 获取当前最高出价
        let current_highest: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(amount) FROM auction_bids WHERE listing_id = $1"
        )
        .bind(listing_id)
        .fetch_one(&mut *tx)
        .await?;

        if let Some(highest) = current_highest {
            if amount <= highest {
                return Err(AppError::BadRequest(format!("Bid must be higher than {}", highest)));
            }
        }

        // 取消之前的最高出价标记
        sqlx::query(
            "UPDATE auction_bids SET is_winning = FALSE WHERE listing_id = $1 AND is_winning = TRUE"
        )
        .bind(listing_id)
        .execute(&mut *tx)
        .await?;

        // 创建出价
        let bid = sqlx::query_as::<_, AuctionBid>(
            r#"
            INSERT INTO auction_bids (listing_id, bidder_id, amount, is_winning)
            VALUES ($1, $2, $3, TRUE)
            RETURNING *
            "#
        )
        .bind(listing_id)
        .bind(bidder_id)
        .bind(amount)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(bid)
    }

    /// Get bids for a listing
    pub async fn get_listing_bids(&self, listing_id: Uuid) -> ApiResult<Vec<BidResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT b.*, p.username as bidder_username
            FROM auction_bids b
            JOIN players p ON b.bidder_id = p.id
            WHERE b.listing_id = $1
            ORDER BY b.amount DESC
            "#
        )
        .bind(listing_id)
        .fetch_all(&self.db.pg)
        .await?;

        let bids = rows.into_iter().map(|row| BidResponse {
            id: row.get("id"),
            listing_id: row.get("listing_id"),
            bidder_id: row.get("bidder_id"),
            bidder_username: row.get("bidder_username"),
            amount: row.get("amount"),
            is_winning: row.get("is_winning"),
            created_at: row.get("created_at"),
        }).collect();

        Ok(bids)
    }

    /// End auction and complete sale
    pub async fn end_auction(&self, listing_id: Uuid) -> ApiResult<Option<MarketplaceTransaction>> {
        let mut tx = self.db.pg.begin().await?;

        // 获取挂单
        let listing = sqlx::query_as::<_, MarketplaceListing>(
            "SELECT * FROM marketplace_listings WHERE id = $1 AND listing_type = 'auction' FOR UPDATE"
        )
        .bind(listing_id)
        .fetch_optional(&mut *tx)
        .await?;

        let listing = listing.ok_or_else(|| AppError::NotFound("Auction not found".into()))?;

        if listing.status != ListingStatus::Active {
            return Err(AppError::BadRequest("Auction is not active".into()));
        }

        // 获取最高出价
        let winning_bid = sqlx::query_as::<_, AuctionBid>(
            "SELECT * FROM auction_bids WHERE listing_id = $1 AND is_winning = TRUE"
        )
        .bind(listing_id)
        .fetch_optional(&mut *tx)
        .await?;

        match winning_bid {
            Some(bid) => {
                // 有中标者 - 完成交易
                let fee = (bid.amount * PLATFORM_FEE_BPS) / 10000;
                let seller_receives = bid.amount - fee;

                // 更新挂单
                sqlx::query(
                    r#"
                    UPDATE marketplace_listings
                    SET status = 'sold', sold_at = NOW(), buyer_id = $1, final_price = $2
                    WHERE id = $3
                    "#
                )
                .bind(bid.bidder_id)
                .bind(bid.amount)
                .bind(listing_id)
                .execute(&mut *tx)
                .await?;

                // 转移所有权
                sqlx::query("UPDATE player_titans SET player_id = $1 WHERE id = $2")
                    .bind(bid.bidder_id)
                    .bind(listing.titan_id)
                    .execute(&mut *tx)
                    .await?;

                // 创建交易记录
                let transaction = sqlx::query_as::<_, MarketplaceTransaction>(
                    r#"
                    INSERT INTO marketplace_transactions
                    (listing_id, seller_id, buyer_id, titan_id, transaction_type, price, fee, seller_receives)
                    VALUES ($1, $2, $3, $4, 'auction_win', $5, $6, $7)
                    RETURNING *
                    "#
                )
                .bind(listing_id)
                .bind(listing.seller_id)
                .bind(bid.bidder_id)
                .bind(listing.titan_id)
                .bind(bid.amount)
                .bind(fee)
                .bind(seller_receives)
                .fetch_one(&mut *tx)
                .await?;

                tx.commit().await?;
                Ok(Some(transaction))
            }
            None => {
                // 无出价 - 标记为过期
                sqlx::query("UPDATE marketplace_listings SET status = 'expired' WHERE id = $1")
                    .bind(listing_id)
                    .execute(&mut *tx)
                    .await?;

                tx.commit().await?;
                Ok(None)
            }
        }
    }

    // ============================================
    // Offers
    // ============================================

    /// Make an offer on a Titan (not listed)
    pub async fn make_offer(&self, offerer_id: Uuid, req: MakeOfferRequest) -> ApiResult<PriceOffer> {
        // 验证 Titan 存在且不属于出价者
        let owner_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT player_id FROM player_titans WHERE id = $1"
        )
        .bind(req.titan_id)
        .fetch_optional(&self.db.pg)
        .await?;

        let owner_id = owner_id.ok_or_else(|| AppError::NotFound("Titan not found".into()))?;

        if owner_id == offerer_id {
            return Err(AppError::BadRequest("Cannot make offer on your own Titan".into()));
        }

        let expires_at = Utc::now() + Duration::hours(req.expires_in_hours);

        let offer = sqlx::query_as::<_, PriceOffer>(
            r#"
            INSERT INTO price_offers (titan_id, offerer_id, owner_id, amount, expires_at, message)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#
        )
        .bind(req.titan_id)
        .bind(offerer_id)
        .bind(owner_id)
        .bind(req.amount)
        .bind(expires_at)
        .bind(req.message)
        .fetch_one(&self.db.pg)
        .await?;

        Ok(offer)
    }

    /// Accept an offer
    pub async fn accept_offer(&self, owner_id: Uuid, offer_id: Uuid) -> ApiResult<MarketplaceTransaction> {
        let mut tx = self.db.pg.begin().await?;

        // 获取并验证 offer
        let offer = sqlx::query_as::<_, PriceOffer>(
            "SELECT * FROM price_offers WHERE id = $1 AND owner_id = $2 AND status = 'pending' FOR UPDATE"
        )
        .bind(offer_id)
        .bind(owner_id)
        .fetch_optional(&mut *tx)
        .await?;

        let offer = offer.ok_or_else(|| AppError::NotFound("Offer not found".into()))?;

        if offer.expires_at < Utc::now() {
            sqlx::query("UPDATE price_offers SET status = 'expired' WHERE id = $1")
                .bind(offer_id)
                .execute(&mut *tx)
                .await?;
            return Err(AppError::BadRequest("Offer has expired".into()));
        }

        // 计算费用
        let fee = (offer.amount * PLATFORM_FEE_BPS) / 10000;
        let seller_receives = offer.amount - fee;

        // 更新 offer 状态
        sqlx::query("UPDATE price_offers SET status = 'accepted', responded_at = NOW() WHERE id = $1")
            .bind(offer_id)
            .execute(&mut *tx)
            .await?;

        // 转移所有权
        sqlx::query("UPDATE player_titans SET player_id = $1 WHERE id = $2")
            .bind(offer.offerer_id)
            .bind(offer.titan_id)
            .execute(&mut *tx)
            .await?;

        // 创建虚拟挂单（用于交易记录）
        let listing_id = Uuid::new_v4();

        // 创建交易记录
        let transaction = sqlx::query_as::<_, MarketplaceTransaction>(
            r#"
            INSERT INTO marketplace_transactions
            (id, listing_id, seller_id, buyer_id, titan_id, transaction_type, price, fee, seller_receives)
            VALUES ($1, $1, $2, $3, $4, 'offer_accepted', $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(listing_id)
        .bind(owner_id)
        .bind(offer.offerer_id)
        .bind(offer.titan_id)
        .bind(offer.amount)
        .bind(fee)
        .bind(seller_receives)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(transaction)
    }

    /// Reject an offer
    pub async fn reject_offer(&self, owner_id: Uuid, offer_id: Uuid) -> ApiResult<()> {
        let result = sqlx::query(
            "UPDATE price_offers SET status = 'rejected', responded_at = NOW() WHERE id = $1 AND owner_id = $2 AND status = 'pending'"
        )
        .bind(offer_id)
        .bind(owner_id)
        .execute(&self.db.pg)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Offer not found".into()));
        }

        Ok(())
    }

    /// Get offers received
    pub async fn get_received_offers(&self, owner_id: Uuid) -> ApiResult<Vec<OfferResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT o.*, p.username as offerer_username
            FROM price_offers o
            JOIN players p ON o.offerer_id = p.id
            WHERE o.owner_id = $1 AND o.status = 'pending' AND o.expires_at > NOW()
            ORDER BY o.created_at DESC
            "#
        )
        .bind(owner_id)
        .fetch_all(&self.db.pg)
        .await?;

        let offers = rows.into_iter().map(|row| OfferResponse {
            id: row.get("id"),
            titan_id: row.get("titan_id"),
            offerer_id: row.get("offerer_id"),
            offerer_username: row.get("offerer_username"),
            owner_id: row.get("owner_id"),
            amount: row.get("amount"),
            status: row.get("status"),
            message: row.get("message"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
        }).collect();

        Ok(offers)
    }

    /// Get offers sent
    pub async fn get_sent_offers(&self, offerer_id: Uuid) -> ApiResult<Vec<OfferResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT o.*, NULL as offerer_username
            FROM price_offers o
            WHERE o.offerer_id = $1
            ORDER BY o.created_at DESC
            "#
        )
        .bind(offerer_id)
        .fetch_all(&self.db.pg)
        .await?;

        let offers = rows.into_iter().map(|row| OfferResponse {
            id: row.get("id"),
            titan_id: row.get("titan_id"),
            offerer_id: row.get("offerer_id"),
            offerer_username: None,
            owner_id: row.get("owner_id"),
            amount: row.get("amount"),
            status: row.get("status"),
            message: row.get("message"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
        }).collect();

        Ok(offers)
    }

    // ============================================
    // Favorites
    // ============================================

    /// Add listing to favorites
    pub async fn add_favorite(&self, player_id: Uuid, listing_id: Uuid) -> ApiResult<()> {
        sqlx::query(
            "INSERT INTO listing_favorites (player_id, listing_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(player_id)
        .bind(listing_id)
        .execute(&self.db.pg)
        .await?;

        Ok(())
    }

    /// Remove listing from favorites
    pub async fn remove_favorite(&self, player_id: Uuid, listing_id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM listing_favorites WHERE player_id = $1 AND listing_id = $2")
            .bind(player_id)
            .bind(listing_id)
            .execute(&self.db.pg)
            .await?;

        Ok(())
    }

    /// Get player's favorite listings
    pub async fn get_favorites(&self, player_id: Uuid) -> ApiResult<Vec<ListingResponse>> {
        // 简化版本，复用 search_listings 的返回格式
        let rows = sqlx::query(
            r#"
            SELECT 
                l.id, l.seller_id, l.titan_id, l.listing_type, l.price, l.min_price,
                l.status, l.expires_at, l.views, l.favorites, l.created_at,
                p.username as seller_username,
                pt.element, pt.threat_class, pt.species_id, pt.level, pt.nickname, pt.genes,
                COALESCE((SELECT MAX(amount) FROM auction_bids WHERE listing_id = l.id), 0) as current_bid,
                (SELECT COUNT(*) FROM auction_bids WHERE listing_id = l.id)::INT as bid_count,
                TRUE as is_favorited
            FROM listing_favorites f
            JOIN marketplace_listings l ON f.listing_id = l.id
            JOIN players p ON l.seller_id = p.id
            JOIN player_titans pt ON l.titan_id = pt.id
            WHERE f.player_id = $1
            ORDER BY f.created_at DESC
            "#
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        let listings = rows.into_iter().map(|row| ListingResponse {
            id: row.get("id"),
            seller_id: row.get("seller_id"),
            seller_username: row.get("seller_username"),
            titan_id: row.get("titan_id"),
            titan: TitanListingInfo {
                id: row.get("titan_id"),
                element: row.get("element"),
                threat_class: row.get("threat_class"),
                species_id: row.get("species_id"),
                level: row.get("level"),
                nickname: row.get("nickname"),
                genes: row.get("genes"),
            },
            listing_type: row.get("listing_type"),
            price: row.get("price"),
            min_price: row.get("min_price"),
            current_bid: {
                let bid: i64 = row.get("current_bid");
                if bid > 0 { Some(bid) } else { None }
            },
            bid_count: row.get("bid_count"),
            status: row.get("status"),
            expires_at: row.get("expires_at"),
            views: row.get("views"),
            favorites: row.get("favorites"),
            is_favorited: true,
            created_at: row.get("created_at"),
        }).collect();

        Ok(listings)
    }

    // ============================================
    // Stats & History
    // ============================================

    /// Get marketplace stats
    pub async fn get_stats(&self) -> ApiResult<MarketplaceStatsResponse> {
        let row = sqlx::query(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM marketplace_listings)::INT as total_listings,
                (SELECT COUNT(*) FROM marketplace_listings WHERE status = 'active')::INT as active_listings,
                COALESCE((SELECT SUM(price) FROM marketplace_transactions WHERE created_at > NOW() - INTERVAL '24 hours'), 0)::BIGINT as total_volume_24h,
                (SELECT COUNT(*) FROM marketplace_transactions WHERE created_at > NOW() - INTERVAL '24 hours')::INT as total_sales_24h,
                (SELECT MIN(price) FROM marketplace_listings WHERE status = 'active') as floor_price,
                (SELECT AVG(price)::BIGINT FROM marketplace_transactions WHERE created_at > NOW() - INTERVAL '7 days') as avg_price
            "#
        )
        .fetch_one(&self.db.pg)
        .await?;

        Ok(MarketplaceStatsResponse {
            total_listings: row.get("total_listings"),
            active_listings: row.get("active_listings"),
            total_volume_24h: row.get("total_volume_24h"),
            total_sales_24h: row.get("total_sales_24h"),
            floor_price: row.get("floor_price"),
            avg_price: row.get("avg_price"),
        })
    }

    /// Get transaction history for a player
    pub async fn get_transaction_history(&self, player_id: Uuid) -> ApiResult<Vec<TransactionHistoryEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                t.id, t.listing_id, t.transaction_type, t.price, t.fee,
                CASE WHEN t.seller_id = $1 THEN t.buyer_id ELSE t.seller_id END as counterparty_id,
                CASE WHEN t.seller_id = $1 THEN pb.username ELSE ps.username END as counterparty_username,
                pt.element as titan_element,
                pt.level as titan_level,
                t.seller_id = $1 as is_seller,
                t.created_at
            FROM marketplace_transactions t
            JOIN players ps ON t.seller_id = ps.id
            JOIN players pb ON t.buyer_id = pb.id
            LEFT JOIN player_titans pt ON t.titan_id = pt.id
            WHERE t.seller_id = $1 OR t.buyer_id = $1
            ORDER BY t.created_at DESC
            LIMIT 100
            "#
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        let history = rows.into_iter().map(|row| TransactionHistoryEntry {
            id: row.get("id"),
            listing_id: row.get("listing_id"),
            transaction_type: row.get("transaction_type"),
            price: row.get("price"),
            fee: row.get("fee"),
            counterparty_id: row.get("counterparty_id"),
            counterparty_username: row.get("counterparty_username"),
            titan_element: row.get("titan_element"),
            titan_level: row.get("titan_level"),
            is_seller: row.get("is_seller"),
            created_at: row.get("created_at"),
        }).collect();

        Ok(history)
    }

    /// Get price chart data
    pub async fn get_price_chart(
        &self,
        element: Option<Element>,
        threat_class: Option<i16>,
        days: i32,
    ) -> ApiResult<PriceChartResponse> {
        let mut sql = String::from(
            "SELECT price, recorded_at FROM price_history WHERE recorded_at > NOW() - $1::INTERVAL"
        );

        if element.is_some() {
            sql.push_str(" AND element = $2");
        }
        if threat_class.is_some() {
            sql.push_str(&format!(" AND threat_class = ${}", if element.is_some() { 3 } else { 2 }));
        }

        sql.push_str(" ORDER BY recorded_at ASC");

        let interval = format!("{} days", days);

        let data_points = sqlx::query_as::<_, PriceHistoryEntry>(&sql)
            .bind(&interval)
            .bind(element)
            .bind(threat_class)
            .fetch_all(&self.db.pg)
            .await?;

        let prices: Vec<i64> = data_points.iter().map(|p| p.price).collect();
        
        let (avg_price, min_price, max_price) = if prices.is_empty() {
            (0, 0, 0)
        } else {
            let sum: i64 = prices.iter().sum();
            let min = *prices.iter().min().unwrap();
            let max = *prices.iter().max().unwrap();
            (sum / prices.len() as i64, min, max)
        };

        Ok(PriceChartResponse {
            element,
            threat_class,
            period: format!("{}d", days),
            data_points,
            avg_price,
            min_price,
            max_price,
        })
    }

    /// Get player's active listings
    pub async fn get_my_listings(&self, player_id: Uuid) -> ApiResult<Vec<ListingResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                l.id, l.seller_id, l.titan_id, l.listing_type, l.price, l.min_price,
                l.status, l.expires_at, l.views, l.favorites, l.created_at,
                NULL as seller_username,
                pt.element, pt.threat_class, pt.species_id, pt.level, pt.nickname, pt.genes,
                COALESCE((SELECT MAX(amount) FROM auction_bids WHERE listing_id = l.id), 0) as current_bid,
                (SELECT COUNT(*) FROM auction_bids WHERE listing_id = l.id)::INT as bid_count,
                FALSE as is_favorited
            FROM marketplace_listings l
            JOIN player_titans pt ON l.titan_id = pt.id
            WHERE l.seller_id = $1
            ORDER BY l.created_at DESC
            "#
        )
        .bind(player_id)
        .fetch_all(&self.db.pg)
        .await?;

        let listings = rows.into_iter().map(|row| ListingResponse {
            id: row.get("id"),
            seller_id: row.get("seller_id"),
            seller_username: None,
            titan_id: row.get("titan_id"),
            titan: TitanListingInfo {
                id: row.get("titan_id"),
                element: row.get("element"),
                threat_class: row.get("threat_class"),
                species_id: row.get("species_id"),
                level: row.get("level"),
                nickname: row.get("nickname"),
                genes: row.get("genes"),
            },
            listing_type: row.get("listing_type"),
            price: row.get("price"),
            min_price: row.get("min_price"),
            current_bid: {
                let bid: i64 = row.get("current_bid");
                if bid > 0 { Some(bid) } else { None }
            },
            bid_count: row.get("bid_count"),
            status: row.get("status"),
            expires_at: row.get("expires_at"),
            views: row.get("views"),
            favorites: row.get("favorites"),
            is_favorited: false,
            created_at: row.get("created_at"),
        }).collect();

        Ok(listings)
    }
}
