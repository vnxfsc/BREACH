-- Marketplace & Trading System Migration
-- Version: 0.7.0

-- ============================================
-- 1. Listing Status Enum
-- ============================================
CREATE TYPE listing_status AS ENUM ('active', 'sold', 'cancelled', 'expired');
CREATE TYPE listing_type AS ENUM ('fixed_price', 'auction');
CREATE TYPE transaction_type AS ENUM ('purchase', 'auction_win', 'offer_accepted');

-- ============================================
-- 2. Marketplace Listings Table
-- ============================================
CREATE TABLE marketplace_listings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    titan_id UUID NOT NULL REFERENCES player_titans(id) ON DELETE CASCADE,
    
    -- Listing details
    listing_type LISTING_TYPE NOT NULL DEFAULT 'fixed_price',
    price BIGINT NOT NULL CHECK (price > 0),  -- In BREACH tokens (with decimals)
    min_price BIGINT,  -- For auctions: minimum/reserve price
    
    -- Status tracking
    status LISTING_STATUS NOT NULL DEFAULT 'active',
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    sold_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    
    -- Buyer info (filled when sold)
    buyer_id UUID REFERENCES players(id) ON DELETE SET NULL,
    final_price BIGINT,
    
    -- Metadata
    views INTEGER NOT NULL DEFAULT 0,
    favorites INTEGER NOT NULL DEFAULT 0
);

-- Indexes for listings
CREATE INDEX idx_listings_seller ON marketplace_listings(seller_id);
CREATE INDEX idx_listings_titan ON marketplace_listings(titan_id);
CREATE INDEX idx_listings_status ON marketplace_listings(status);
CREATE INDEX idx_listings_type ON marketplace_listings(listing_type);
CREATE INDEX idx_listings_price ON marketplace_listings(price) WHERE status = 'active';
CREATE INDEX idx_listings_created ON marketplace_listings(created_at DESC);
CREATE INDEX idx_listings_expires ON marketplace_listings(expires_at) WHERE status = 'active';

-- ============================================
-- 3. Auction Bids Table
-- ============================================
CREATE TABLE auction_bids (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID NOT NULL REFERENCES marketplace_listings(id) ON DELETE CASCADE,
    bidder_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    
    -- Bid details
    amount BIGINT NOT NULL CHECK (amount > 0),
    is_winning BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Prevent duplicate bids at same amount
    UNIQUE (listing_id, bidder_id, amount)
);

-- Indexes for bids
CREATE INDEX idx_bids_listing ON auction_bids(listing_id);
CREATE INDEX idx_bids_bidder ON auction_bids(bidder_id);
CREATE INDEX idx_bids_amount ON auction_bids(listing_id, amount DESC);
CREATE INDEX idx_bids_winning ON auction_bids(listing_id) WHERE is_winning = TRUE;

-- ============================================
-- 4. Marketplace Transactions Table
-- ============================================
CREATE TABLE marketplace_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    listing_id UUID NOT NULL REFERENCES marketplace_listings(id) ON DELETE CASCADE,
    
    -- Parties
    seller_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    buyer_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    titan_id UUID NOT NULL,  -- Keep reference even if titan deleted
    
    -- Transaction details
    transaction_type TRANSACTION_TYPE NOT NULL,
    price BIGINT NOT NULL,
    fee BIGINT NOT NULL DEFAULT 0,  -- Platform fee
    seller_receives BIGINT NOT NULL,  -- price - fee
    
    -- Blockchain transaction
    tx_signature VARCHAR(128),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for transactions
CREATE INDEX idx_tx_listing ON marketplace_transactions(listing_id);
CREATE INDEX idx_tx_seller ON marketplace_transactions(seller_id);
CREATE INDEX idx_tx_buyer ON marketplace_transactions(buyer_id);
CREATE INDEX idx_tx_created ON marketplace_transactions(created_at DESC);

-- ============================================
-- 5. Price Offers (Buy Offers)
-- ============================================
CREATE TABLE price_offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    titan_id UUID NOT NULL REFERENCES player_titans(id) ON DELETE CASCADE,
    offerer_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    
    -- Offer details
    amount BIGINT NOT NULL CHECK (amount > 0),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'cancelled', 'expired')),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    responded_at TIMESTAMPTZ,
    
    -- Message from offerer
    message TEXT
);

-- Indexes for offers
CREATE INDEX idx_offers_titan ON price_offers(titan_id);
CREATE INDEX idx_offers_offerer ON price_offers(offerer_id);
CREATE INDEX idx_offers_owner ON price_offers(owner_id);
CREATE INDEX idx_offers_status ON price_offers(status) WHERE status = 'pending';

-- ============================================
-- 6. Listing Favorites (Watchlist)
-- ============================================
CREATE TABLE listing_favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    listing_id UUID NOT NULL REFERENCES marketplace_listings(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE (player_id, listing_id)
);

-- Indexes for favorites
CREATE INDEX idx_favorites_player ON listing_favorites(player_id);
CREATE INDEX idx_favorites_listing ON listing_favorites(listing_id);

-- ============================================
-- 7. Price History (for analytics)
-- ============================================
CREATE TABLE price_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Titan attributes for aggregation
    element ELEMENT_TYPE NOT NULL,
    threat_class THREAT_CLASS_TYPE NOT NULL,
    species_id INTEGER,
    
    -- Price data
    price BIGINT NOT NULL,
    transaction_type TRANSACTION_TYPE NOT NULL,
    
    -- Timestamp
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for price history
CREATE INDEX idx_price_element ON price_history(element, recorded_at DESC);
CREATE INDEX idx_price_threat ON price_history(threat_class, recorded_at DESC);
CREATE INDEX idx_price_species ON price_history(species_id, recorded_at DESC) WHERE species_id IS NOT NULL;

-- ============================================
-- 8. Marketplace Statistics Cache
-- ============================================
CREATE TABLE marketplace_stats (
    id SERIAL PRIMARY KEY,
    stat_date DATE NOT NULL UNIQUE DEFAULT CURRENT_DATE,
    
    -- Volume stats
    total_listings INTEGER NOT NULL DEFAULT 0,
    active_listings INTEGER NOT NULL DEFAULT 0,
    total_sales INTEGER NOT NULL DEFAULT 0,
    total_volume BIGINT NOT NULL DEFAULT 0,  -- Total BREACH traded
    
    -- Price stats
    avg_price BIGINT,
    min_price BIGINT,
    max_price BIGINT,
    
    -- By element
    stats_by_element JSONB DEFAULT '{}',
    
    -- By threat class
    stats_by_threat_class JSONB DEFAULT '{}',
    
    -- Timestamps
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================
-- 9. Helper Functions
-- ============================================

-- Function to get highest bid for a listing
CREATE OR REPLACE FUNCTION get_highest_bid(p_listing_id UUID)
RETURNS BIGINT AS $$
BEGIN
    RETURN (
        SELECT COALESCE(MAX(amount), 0)
        FROM auction_bids
        WHERE listing_id = p_listing_id
    );
END;
$$ LANGUAGE plpgsql;

-- Function to update listing favorite count
CREATE OR REPLACE FUNCTION update_listing_favorites()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE marketplace_listings
        SET favorites = favorites + 1
        WHERE id = NEW.listing_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE marketplace_listings
        SET favorites = favorites - 1
        WHERE id = OLD.listing_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger for favorite count
CREATE TRIGGER trg_listing_favorites
AFTER INSERT OR DELETE ON listing_favorites
FOR EACH ROW EXECUTE FUNCTION update_listing_favorites();

-- Function to expire listings
CREATE OR REPLACE FUNCTION expire_old_listings()
RETURNS INTEGER AS $$
DECLARE
    expired_count INTEGER;
BEGIN
    UPDATE marketplace_listings
    SET status = 'expired'
    WHERE status = 'active'
      AND expires_at < NOW();
    
    GET DIAGNOSTICS expired_count = ROW_COUNT;
    RETURN expired_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 10. Views for Common Queries
-- ============================================

-- Active listings with Titan details
CREATE OR REPLACE VIEW v_active_listings AS
SELECT 
    l.id,
    l.seller_id,
    p.username as seller_username,
    l.titan_id,
    pt.element,
    pt.threat_class,
    pt.species_id,
    pt.level,
    pt.nickname,
    l.listing_type,
    l.price,
    l.min_price,
    get_highest_bid(l.id) as current_bid,
    l.expires_at,
    l.views,
    l.favorites,
    l.created_at
FROM marketplace_listings l
JOIN players p ON l.seller_id = p.id
JOIN player_titans pt ON l.titan_id = pt.id
WHERE l.status = 'active';

-- Recent sales
CREATE OR REPLACE VIEW v_recent_sales AS
SELECT 
    t.id,
    t.seller_id,
    ps.username as seller_username,
    t.buyer_id,
    pb.username as buyer_username,
    t.titan_id,
    t.price,
    t.fee,
    t.transaction_type,
    t.created_at
FROM marketplace_transactions t
JOIN players ps ON t.seller_id = ps.id
JOIN players pb ON t.buyer_id = pb.id
ORDER BY t.created_at DESC;

-- ============================================
-- 11. Insert Default Marketplace Config
-- ============================================

-- Platform fee percentage (stored as basis points, 250 = 2.5%)
INSERT INTO marketplace_stats (stat_date, total_listings, active_listings)
VALUES (CURRENT_DATE, 0, 0)
ON CONFLICT (stat_date) DO NOTHING;

COMMENT ON TABLE marketplace_listings IS 'NFT marketplace listings for Titan trading';
COMMENT ON TABLE auction_bids IS 'Bids on auction-type listings';
COMMENT ON TABLE marketplace_transactions IS 'Completed marketplace transactions';
COMMENT ON TABLE price_offers IS 'Direct offers to buy Titans not listed';
COMMENT ON TABLE listing_favorites IS 'Player watchlist for listings';
COMMENT ON TABLE price_history IS 'Historical price data for analytics';
