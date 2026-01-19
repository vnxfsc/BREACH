-- BREACH Backend PvP System Migration
-- Adds: PvP Matchmaking, ELO Rating, Seasons, Ranked Battles

-- ==========================================
-- PVP SEASONS
-- ==========================================

CREATE TABLE pvp_seasons (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT false,
    rewards JSONB,                           -- Season rewards config
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert initial season
INSERT INTO pvp_seasons (name, start_date, end_date, is_active, rewards) VALUES (
    'Season 1: Genesis',
    NOW(),
    NOW() + INTERVAL '90 days',
    true,
    '{
        "champion": {"breach": 50000, "xp": 100000, "title": "Genesis Champion"},
        "master": {"breach": 25000, "xp": 50000, "title": "Genesis Master"},
        "diamond": {"breach": 10000, "xp": 25000},
        "platinum": {"breach": 5000, "xp": 10000},
        "gold": {"breach": 2500, "xp": 5000},
        "silver": {"breach": 1000, "xp": 2500},
        "bronze": {"breach": 500, "xp": 1000}
    }'
);

-- ==========================================
-- PLAYER PVP STATS
-- ==========================================

CREATE TABLE player_pvp_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    season_id INT NOT NULL REFERENCES pvp_seasons(id),
    
    -- ELO Rating
    elo_rating INT NOT NULL DEFAULT 1000,
    peak_rating INT NOT NULL DEFAULT 1000,
    
    -- Match stats
    matches_played INT NOT NULL DEFAULT 0,
    matches_won INT NOT NULL DEFAULT 0,
    matches_lost INT NOT NULL DEFAULT 0,
    win_streak INT NOT NULL DEFAULT 0,
    max_win_streak INT NOT NULL DEFAULT 0,
    
    -- Rank
    rank_tier VARCHAR(20) NOT NULL DEFAULT 'bronze',  -- bronze, silver, gold, platinum, diamond, master, champion
    rank_division INT NOT NULL DEFAULT 5,              -- 5, 4, 3, 2, 1 (1 is highest in tier)
    rank_points INT NOT NULL DEFAULT 0,                -- Points within division
    
    -- Timestamps
    last_match_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(player_id, season_id)
);

CREATE INDEX idx_pvp_stats_player ON player_pvp_stats(player_id);
CREATE INDEX idx_pvp_stats_season ON player_pvp_stats(season_id);
CREATE INDEX idx_pvp_stats_elo ON player_pvp_stats(season_id, elo_rating DESC);
CREATE INDEX idx_pvp_stats_rank ON player_pvp_stats(season_id, rank_tier, rank_division, rank_points DESC);

-- ==========================================
-- MATCHMAKING QUEUE
-- ==========================================

CREATE TYPE queue_status AS ENUM (
    'searching',
    'matched',
    'cancelled',
    'expired'
);

CREATE TABLE matchmaking_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    titan_id UUID NOT NULL REFERENCES player_titans(id),
    
    -- Matchmaking params
    elo_rating INT NOT NULL,
    elo_range INT NOT NULL DEFAULT 100,              -- Initial search range
    search_start_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Status
    status queue_status NOT NULL DEFAULT 'searching',
    matched_with UUID REFERENCES players(id),
    match_id UUID,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(player_id)  -- One queue entry per player
);

CREATE INDEX idx_queue_searching ON matchmaking_queue(status, elo_rating) WHERE status = 'searching';

-- ==========================================
-- PVP MATCHES
-- ==========================================

CREATE TYPE pvp_match_status AS ENUM (
    'preparing',      -- Players confirming
    'titan_select',   -- Titan selection phase
    'active',         -- Battle in progress
    'completed',      -- Battle finished
    'abandoned'       -- Player disconnected
);

CREATE TABLE pvp_matches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    season_id INT NOT NULL REFERENCES pvp_seasons(id),
    
    -- Players
    player1_id UUID NOT NULL REFERENCES players(id),
    player2_id UUID NOT NULL REFERENCES players(id),
    
    -- Pre-match ratings
    player1_elo INT NOT NULL,
    player2_elo INT NOT NULL,
    
    -- Selected titans
    player1_titan_id UUID REFERENCES player_titans(id),
    player2_titan_id UUID REFERENCES player_titans(id),
    
    -- Status
    status pvp_match_status NOT NULL DEFAULT 'preparing',
    
    -- Battle state
    player1_hp INT NOT NULL DEFAULT 100,
    player2_hp INT NOT NULL DEFAULT 100,
    current_turn UUID,                       -- Who's turn
    turn_number INT NOT NULL DEFAULT 0,
    turn_deadline TIMESTAMPTZ,               -- Auto-lose if exceeded
    
    -- Results
    winner_id UUID REFERENCES players(id),
    loser_id UUID REFERENCES players(id),
    win_reason VARCHAR(50),                  -- ko, surrender, timeout, disconnect
    
    -- ELO changes
    winner_elo_change INT,
    loser_elo_change INT,
    
    -- Rewards
    winner_breach_reward BIGINT,
    winner_xp_reward INT,
    
    -- Timestamps
    ready_deadline TIMESTAMPTZ,              -- Must confirm by this time
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pvp_matches_player1 ON pvp_matches(player1_id);
CREATE INDEX idx_pvp_matches_player2 ON pvp_matches(player2_id);
CREATE INDEX idx_pvp_matches_season ON pvp_matches(season_id);
CREATE INDEX idx_pvp_matches_active ON pvp_matches(status) WHERE status IN ('preparing', 'titan_select', 'active');

-- ==========================================
-- PVP BATTLE TURNS
-- ==========================================

CREATE TYPE pvp_action_type AS ENUM (
    'attack',
    'special',
    'defend',
    'item'
);

CREATE TABLE pvp_battle_turns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES pvp_matches(id) ON DELETE CASCADE,
    turn_number INT NOT NULL,
    
    -- Actions
    player1_action pvp_action_type,
    player1_damage INT DEFAULT 0,
    player2_action pvp_action_type,
    player2_damage INT DEFAULT 0,
    
    -- State after turn
    player1_hp_after INT,
    player2_hp_after INT,
    
    -- Timing
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pvp_turns_match ON pvp_battle_turns(match_id);

-- ==========================================
-- ELO CALCULATION FUNCTIONS
-- ==========================================

-- Calculate expected win probability
CREATE OR REPLACE FUNCTION calc_expected_score(player_elo INT, opponent_elo INT)
RETURNS FLOAT AS $$
BEGIN
    RETURN 1.0 / (1.0 + POWER(10, (opponent_elo - player_elo)::FLOAT / 400));
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Calculate ELO change after match
CREATE OR REPLACE FUNCTION calc_elo_change(
    player_elo INT,
    opponent_elo INT,
    won BOOLEAN,
    k_factor INT DEFAULT 32
)
RETURNS INT AS $$
DECLARE
    expected FLOAT;
    actual FLOAT;
BEGIN
    expected := calc_expected_score(player_elo, opponent_elo);
    actual := CASE WHEN won THEN 1.0 ELSE 0.0 END;
    RETURN ROUND(k_factor * (actual - expected));
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Get rank tier from ELO
CREATE OR REPLACE FUNCTION get_rank_tier(elo INT)
RETURNS VARCHAR(20) AS $$
BEGIN
    RETURN CASE
        WHEN elo >= 2400 THEN 'champion'
        WHEN elo >= 2200 THEN 'master'
        WHEN elo >= 2000 THEN 'diamond'
        WHEN elo >= 1800 THEN 'platinum'
        WHEN elo >= 1600 THEN 'gold'
        WHEN elo >= 1400 THEN 'silver'
        ELSE 'bronze'
    END;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Get rank division (5-1, 1 is highest)
CREATE OR REPLACE FUNCTION get_rank_division(elo INT)
RETURNS INT AS $$
DECLARE
    tier_base INT;
BEGIN
    tier_base := CASE
        WHEN elo >= 2400 THEN 2400
        WHEN elo >= 2200 THEN 2200
        WHEN elo >= 2000 THEN 2000
        WHEN elo >= 1800 THEN 1800
        WHEN elo >= 1600 THEN 1600
        WHEN elo >= 1400 THEN 1400
        ELSE 1000
    END;
    
    -- 200 points per tier, 40 points per division
    RETURN 5 - LEAST(4, (elo - tier_base) / 40);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ==========================================
-- MATCH HISTORY VIEW
-- ==========================================

CREATE OR REPLACE VIEW pvp_match_history AS
SELECT 
    m.id,
    m.season_id,
    m.player1_id,
    p1.username as player1_username,
    m.player1_elo,
    m.player2_id,
    p2.username as player2_username,
    m.player2_elo,
    m.winner_id,
    m.win_reason,
    m.winner_elo_change,
    m.loser_elo_change,
    m.turn_number as total_turns,
    m.started_at,
    m.ended_at,
    EXTRACT(EPOCH FROM (m.ended_at - m.started_at)) as duration_seconds
FROM pvp_matches m
JOIN players p1 ON p1.id = m.player1_id
JOIN players p2 ON p2.id = m.player2_id
WHERE m.status = 'completed';

-- ==========================================
-- LEADERBOARD VIEW
-- ==========================================

CREATE OR REPLACE VIEW pvp_leaderboard AS
SELECT 
    s.player_id,
    p.username,
    p.wallet_address,
    s.elo_rating,
    s.peak_rating,
    s.rank_tier,
    s.rank_division,
    s.matches_played,
    s.matches_won,
    s.matches_lost,
    CASE WHEN s.matches_played > 0 
        THEN ROUND(s.matches_won::NUMERIC / s.matches_played * 100, 1)
        ELSE 0 
    END as win_rate,
    s.max_win_streak,
    ROW_NUMBER() OVER (ORDER BY s.elo_rating DESC) as rank
FROM player_pvp_stats s
JOIN players p ON p.id = s.player_id
JOIN pvp_seasons ps ON ps.id = s.season_id AND ps.is_active = true
WHERE p.is_banned = false
ORDER BY s.elo_rating DESC;

-- ==========================================
-- CLEANUP OLD QUEUE ENTRIES
-- ==========================================

-- Function to clean stale queue entries
CREATE OR REPLACE FUNCTION cleanup_matchmaking_queue()
RETURNS INT AS $$
DECLARE
    cleaned INT;
BEGIN
    UPDATE matchmaking_queue
    SET status = 'expired', updated_at = NOW()
    WHERE status = 'searching'
      AND search_start_time < NOW() - INTERVAL '5 minutes';
    
    GET DIAGNOSTICS cleaned = ROW_COUNT;
    RETURN cleaned;
END;
$$ LANGUAGE plpgsql;
