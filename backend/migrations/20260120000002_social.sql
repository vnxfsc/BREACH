-- BREACH Backend Social Features Migration
-- Adds: Friends System, Guild System, Social Messages

-- ==========================================
-- FRIENDS SYSTEM
-- ==========================================

-- Friend request status
CREATE TYPE friend_request_status AS ENUM (
    'pending',
    'accepted',
    'rejected',
    'cancelled'
);

-- Friend requests
CREATE TABLE friend_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    status friend_request_status NOT NULL DEFAULT 'pending',
    message VARCHAR(200),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    
    UNIQUE(sender_id, receiver_id),
    CHECK(sender_id != receiver_id)
);

CREATE INDEX idx_friend_requests_sender ON friend_requests(sender_id, status);
CREATE INDEX idx_friend_requests_receiver ON friend_requests(receiver_id, status);

-- Friends (bidirectional friendship)
CREATE TABLE friendships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player1_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    player2_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure player1_id < player2_id to prevent duplicates
    UNIQUE(player1_id, player2_id),
    CHECK(player1_id < player2_id)
);

CREATE INDEX idx_friendships_player1 ON friendships(player1_id);
CREATE INDEX idx_friendships_player2 ON friendships(player2_id);

-- Daily gift tracking
CREATE TABLE friend_gifts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    receiver_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    gift_date DATE NOT NULL DEFAULT CURRENT_DATE,
    breach_amount BIGINT NOT NULL DEFAULT 5,
    opened BOOLEAN NOT NULL DEFAULT false,
    opened_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(sender_id, receiver_id, gift_date)
);

CREATE INDEX idx_friend_gifts_receiver ON friend_gifts(receiver_id, opened);
CREATE INDEX idx_friend_gifts_date ON friend_gifts(gift_date);

-- ==========================================
-- GUILD SYSTEM
-- ==========================================

-- Guild roles
CREATE TYPE guild_role AS ENUM (
    'leader',
    'co_leader',
    'elder',
    'member'
);

-- Guilds
CREATE TABLE guilds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL UNIQUE,
    tag VARCHAR(5) NOT NULL UNIQUE,      -- Short tag like [ABC]
    description TEXT,
    icon VARCHAR(100),
    banner VARCHAR(100),
    leader_id UUID NOT NULL REFERENCES players(id),
    
    -- Settings
    min_level INT NOT NULL DEFAULT 1,
    is_public BOOLEAN NOT NULL DEFAULT true,
    max_members INT NOT NULL DEFAULT 50,
    
    -- Stats
    total_captures INT NOT NULL DEFAULT 0,
    total_battles INT NOT NULL DEFAULT 0,
    total_breach BIGINT NOT NULL DEFAULT 0,
    weekly_xp BIGINT NOT NULL DEFAULT 0,
    
    -- Season tracking
    season_rank INT,
    season_points BIGINT NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_guilds_name ON guilds(name);
CREATE INDEX idx_guilds_tag ON guilds(tag);
CREATE INDEX idx_guilds_leader ON guilds(leader_id);
CREATE INDEX idx_guilds_season_points ON guilds(season_points DESC);

-- Guild members
CREATE TABLE guild_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    role guild_role NOT NULL DEFAULT 'member',
    contribution_xp BIGINT NOT NULL DEFAULT 0,
    contribution_captures INT NOT NULL DEFAULT 0,
    contribution_battles INT NOT NULL DEFAULT 0,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(player_id)  -- Player can only be in one guild
);

CREATE INDEX idx_guild_members_guild ON guild_members(guild_id);
CREATE INDEX idx_guild_members_player ON guild_members(player_id);
CREATE INDEX idx_guild_members_contribution ON guild_members(guild_id, contribution_xp DESC);

-- Guild join requests
CREATE TABLE guild_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    message VARCHAR(200),
    status friend_request_status NOT NULL DEFAULT 'pending',
    reviewed_by UUID REFERENCES players(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ,
    
    UNIQUE(guild_id, player_id)
);

CREATE INDEX idx_guild_requests_guild ON guild_requests(guild_id, status);
CREATE INDEX idx_guild_requests_player ON guild_requests(player_id);

-- Guild activity log
CREATE TABLE guild_activity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id UUID NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    player_id UUID REFERENCES players(id) ON DELETE SET NULL,
    activity_type VARCHAR(50) NOT NULL,  -- join, leave, promote, demote, capture, battle, etc.
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_guild_activity_guild ON guild_activity(guild_id, created_at DESC);

-- ==========================================
-- SOCIAL MESSAGES / NOTIFICATIONS
-- ==========================================

-- Notification types
CREATE TYPE notification_type AS ENUM (
    'friend_request',
    'friend_accepted',
    'gift_received',
    'guild_invite',
    'guild_request',
    'guild_accepted',
    'guild_promoted',
    'guild_demoted',
    'guild_kicked',
    'achievement_unlocked',
    'level_up',
    'rare_capture',
    'system'
);

-- Player notifications
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    notification_type notification_type NOT NULL,
    title VARCHAR(100) NOT NULL,
    message TEXT NOT NULL,
    data JSONB,                          -- Additional data (e.g., friend_id, guild_id)
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_notifications_player ON notifications(player_id, is_read, created_at DESC);
CREATE INDEX idx_notifications_expires ON notifications(expires_at) WHERE expires_at IS NOT NULL;

-- ==========================================
-- PLAYER PROFILE EXTENSIONS
-- ==========================================

-- Add guild_id to players
ALTER TABLE players ADD COLUMN IF NOT EXISTS guild_id UUID REFERENCES guilds(id) ON DELETE SET NULL;
ALTER TABLE players ADD COLUMN IF NOT EXISTS friend_code VARCHAR(12) UNIQUE;
ALTER TABLE players ADD COLUMN IF NOT EXISTS bio VARCHAR(200);
ALTER TABLE players ADD COLUMN IF NOT EXISTS avatar VARCHAR(100);
ALTER TABLE players ADD COLUMN IF NOT EXISTS show_online_status BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE players ADD COLUMN IF NOT EXISTS allow_friend_requests BOOLEAN NOT NULL DEFAULT true;

-- Generate unique friend codes for existing players
CREATE OR REPLACE FUNCTION generate_friend_code()
RETURNS VARCHAR(12) AS $$
DECLARE
    code VARCHAR(12);
    exists_count INT;
BEGIN
    LOOP
        -- Generate code like: XXXX-XXXX-XXXX
        code := UPPER(
            SUBSTRING(MD5(RANDOM()::TEXT) FROM 1 FOR 4) || '-' ||
            SUBSTRING(MD5(RANDOM()::TEXT) FROM 5 FOR 4) || '-' ||
            SUBSTRING(MD5(RANDOM()::TEXT) FROM 9 FOR 4)
        );
        
        SELECT COUNT(*) INTO exists_count FROM players WHERE friend_code = code;
        IF exists_count = 0 THEN
            RETURN code;
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Update existing players with friend codes
UPDATE players SET friend_code = generate_friend_code() WHERE friend_code IS NULL;

-- ==========================================
-- HELPER FUNCTIONS
-- ==========================================

-- Function to get friend count
CREATE OR REPLACE FUNCTION get_friend_count(p_player_id UUID)
RETURNS INT AS $$
BEGIN
    RETURN (
        SELECT COUNT(*) FROM friendships 
        WHERE player1_id = p_player_id OR player2_id = p_player_id
    );
END;
$$ LANGUAGE plpgsql;

-- Function to check if two players are friends
CREATE OR REPLACE FUNCTION are_friends(p_player1 UUID, p_player2 UUID)
RETURNS BOOLEAN AS $$
DECLARE
    p1 UUID;
    p2 UUID;
BEGIN
    -- Ensure consistent ordering
    IF p_player1 < p_player2 THEN
        p1 := p_player1;
        p2 := p_player2;
    ELSE
        p1 := p_player2;
        p2 := p_player1;
    END IF;
    
    RETURN EXISTS(
        SELECT 1 FROM friendships WHERE player1_id = p1 AND player2_id = p2
    );
END;
$$ LANGUAGE plpgsql;

-- Function to update guild stats
CREATE OR REPLACE FUNCTION update_guild_stats()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_TABLE_NAME = 'guild_members' THEN
        UPDATE guilds SET
            total_captures = (SELECT COALESCE(SUM(contribution_captures), 0) FROM guild_members WHERE guild_id = NEW.guild_id),
            total_battles = (SELECT COALESCE(SUM(contribution_battles), 0) FROM guild_members WHERE guild_id = NEW.guild_id),
            weekly_xp = (SELECT COALESCE(SUM(contribution_xp), 0) FROM guild_members WHERE guild_id = NEW.guild_id),
            updated_at = NOW()
        WHERE id = NEW.guild_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_guild_stats
AFTER INSERT OR UPDATE ON guild_members
FOR EACH ROW EXECUTE FUNCTION update_guild_stats();
