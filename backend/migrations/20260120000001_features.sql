-- BREACH Backend Feature Extension Migration
-- Adds: Daily Quests, Achievements, Battle System, Inventory, Leaderboards

-- ==========================================
-- DAILY QUESTS SYSTEM
-- ==========================================

-- Quest types enum
CREATE TYPE quest_type AS ENUM (
    'capture',           -- Capture X titans
    'capture_element',   -- Capture X titans of specific element
    'capture_rare',      -- Capture X rare titans (class 3+)
    'battle',            -- Win X battles
    'walk',              -- Walk X meters
    'visit_poi',         -- Visit X different POIs
    'streak'             -- Login X days in a row
);

-- Quest templates (defines available quests)
CREATE TABLE quest_templates (
    id SERIAL PRIMARY KEY,
    quest_type quest_type NOT NULL,
    title VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    target_count INT NOT NULL,
    element element_type,          -- For capture_element quests
    xp_reward INT NOT NULL DEFAULT 100,
    breach_reward BIGINT NOT NULL DEFAULT 0,
    is_daily BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Player daily quests (assigned each day)
CREATE TABLE player_quests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    template_id INT NOT NULL REFERENCES quest_templates(id),
    progress INT NOT NULL DEFAULT 0,
    is_completed BOOLEAN NOT NULL DEFAULT false,
    completed_at TIMESTAMPTZ,
    reward_claimed BOOLEAN NOT NULL DEFAULT false,
    assigned_date DATE NOT NULL DEFAULT CURRENT_DATE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(player_id, template_id, assigned_date)
);

CREATE INDEX idx_player_quests_player ON player_quests(player_id);
CREATE INDEX idx_player_quests_date ON player_quests(assigned_date);
CREATE INDEX idx_player_quests_active ON player_quests(player_id, is_completed, expires_at);

-- ==========================================
-- ACHIEVEMENT SYSTEM
-- ==========================================

-- Achievement categories
CREATE TYPE achievement_category AS ENUM (
    'capture',           -- Capture related
    'collection',        -- Collection milestones
    'battle',            -- Battle related
    'exploration',       -- Exploration/travel
    'social',            -- Social features
    'special'            -- Special/event achievements
);

-- Achievement definitions
CREATE TABLE achievements (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    category achievement_category NOT NULL,
    icon VARCHAR(100),
    tier INT NOT NULL DEFAULT 1,           -- 1=Bronze, 2=Silver, 3=Gold
    requirement_type VARCHAR(50) NOT NULL, -- titans_captured, battles_won, etc.
    requirement_value INT NOT NULL,
    xp_reward INT NOT NULL DEFAULT 0,
    breach_reward BIGINT NOT NULL DEFAULT 0,
    is_hidden BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Player unlocked achievements
CREATE TABLE player_achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    achievement_id INT NOT NULL REFERENCES achievements(id),
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(player_id, achievement_id)
);

CREATE INDEX idx_player_achievements_player ON player_achievements(player_id);

-- ==========================================
-- INVENTORY / TITAN COLLECTION
-- ==========================================

-- Player owned titans (on-chain NFTs tracked off-chain)
CREATE TABLE player_titans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    mint_address VARCHAR(64) NOT NULL UNIQUE,      -- Solana NFT mint address
    species_id INT NOT NULL,
    element element_type NOT NULL,
    threat_class SMALLINT NOT NULL,
    genes BYTEA NOT NULL,
    nickname VARCHAR(50),
    is_favorite BOOLEAN NOT NULL DEFAULT false,
    captured_at TIMESTAMPTZ NOT NULL,
    capture_location_lat DOUBLE PRECISION,
    capture_location_lng DOUBLE PRECISION,
    battles_participated INT NOT NULL DEFAULT 0,
    battles_won INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_player_titans_player ON player_titans(player_id);
CREATE INDEX idx_player_titans_element ON player_titans(element);
CREATE INDEX idx_player_titans_species ON player_titans(species_id);

-- ==========================================
-- BATTLE SYSTEM
-- ==========================================

-- Battle types
CREATE TYPE battle_type AS ENUM (
    'wild',              -- Battle against wild titan
    'pvp',               -- Player vs Player
    'raid',              -- Raid boss battle
    'gym'                -- Gym battle
);

-- Battle status
CREATE TYPE battle_status AS ENUM (
    'pending',           -- Waiting for opponent (PvP)
    'active',            -- Battle in progress
    'completed',         -- Battle finished
    'cancelled'          -- Battle cancelled
);

-- Battle records
CREATE TABLE battles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    battle_type battle_type NOT NULL,
    status battle_status NOT NULL DEFAULT 'active',
    
    -- Players
    player1_id UUID NOT NULL REFERENCES players(id),
    player2_id UUID REFERENCES players(id),          -- NULL for wild/raid battles
    
    -- Titans used
    player1_titan_id UUID REFERENCES player_titans(id),
    player2_titan_id UUID REFERENCES player_titans(id),
    wild_titan_id UUID REFERENCES titan_spawns(id),
    
    -- Results
    winner_id UUID REFERENCES players(id),
    player1_damage INT NOT NULL DEFAULT 0,
    player2_damage INT NOT NULL DEFAULT 0,
    rounds INT NOT NULL DEFAULT 0,
    
    -- Rewards
    xp_reward INT NOT NULL DEFAULT 0,
    breach_reward BIGINT NOT NULL DEFAULT 0,
    
    -- Location
    location_lat DOUBLE PRECISION,
    location_lng DOUBLE PRECISION,
    
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_battles_player1 ON battles(player1_id);
CREATE INDEX idx_battles_player2 ON battles(player2_id);
CREATE INDEX idx_battles_status ON battles(status);
CREATE INDEX idx_battles_type ON battles(battle_type);

-- Battle actions log (for replay)
CREATE TABLE battle_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    battle_id UUID NOT NULL REFERENCES battles(id) ON DELETE CASCADE,
    round INT NOT NULL,
    actor_id UUID NOT NULL REFERENCES players(id),
    action_type VARCHAR(50) NOT NULL,       -- attack, defend, special, item
    damage INT NOT NULL DEFAULT 0,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_battle_actions_battle ON battle_actions(battle_id);

-- ==========================================
-- LEADERBOARD CACHE (for performance)
-- ==========================================

-- Leaderboard types
CREATE TYPE leaderboard_type AS ENUM (
    'experience',        -- Total experience
    'captures',          -- Titans captured
    'battles',           -- Battles won
    'breach',            -- BREACH earned
    'weekly_xp',         -- Weekly experience
    'weekly_captures',   -- Weekly captures
    'weekly_battles'     -- Weekly battles
);

-- Cached leaderboard entries
CREATE TABLE leaderboard_cache (
    id SERIAL PRIMARY KEY,
    leaderboard_type leaderboard_type NOT NULL,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    rank INT NOT NULL,
    score BIGINT NOT NULL,
    region VARCHAR(50),                     -- For regional leaderboards
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(leaderboard_type, region, player_id)
);

CREATE INDEX idx_leaderboard_type ON leaderboard_cache(leaderboard_type);
CREATE INDEX idx_leaderboard_rank ON leaderboard_cache(leaderboard_type, region, rank);

-- ==========================================
-- LOGIN STREAK
-- ==========================================

-- Player login tracking
CREATE TABLE player_logins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    login_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(player_id, login_date)
);

-- Add streak columns to players
ALTER TABLE players ADD COLUMN IF NOT EXISTS current_streak INT NOT NULL DEFAULT 0;
ALTER TABLE players ADD COLUMN IF NOT EXISTS max_streak INT NOT NULL DEFAULT 0;
ALTER TABLE players ADD COLUMN IF NOT EXISTS last_login_date DATE;

-- ==========================================
-- SEED DATA: QUEST TEMPLATES
-- ==========================================

INSERT INTO quest_templates (quest_type, title, description, target_count, element, xp_reward, breach_reward) VALUES
-- Basic capture quests
('capture', 'Titan Hunter', 'Capture 3 Titans today', 3, NULL, 150, 10),
('capture', 'Dedicated Collector', 'Capture 5 Titans today', 5, NULL, 300, 25),

-- Element-specific quests
('capture_element', 'Fire Seeker', 'Capture 2 Volcanic Titans', 2, 'volcanic', 200, 15),
('capture_element', 'Ocean Explorer', 'Capture 2 Abyssal Titans', 2, 'abyssal', 200, 15),
('capture_element', 'Storm Chaser', 'Capture 2 Storm Titans', 2, 'storm', 200, 15),
('capture_element', 'Void Hunter', 'Capture 2 Void Titans', 2, 'void', 200, 15),
('capture_element', 'Bio Researcher', 'Capture 2 Parasitic Titans', 2, 'parasitic', 200, 15),
('capture_element', 'Fossil Finder', 'Capture 2 Ossified Titans', 2, 'ossified', 200, 15),

-- Rare capture
('capture_rare', 'Rare Find', 'Capture a Class III+ Titan', 1, NULL, 500, 50),

-- Battle quests
('battle', 'Battle Ready', 'Win 2 battles', 2, NULL, 200, 20),
('battle', 'Battle Master', 'Win 5 battles', 5, NULL, 500, 50),

-- Exploration quests
('visit_poi', 'Explorer', 'Visit 3 different POIs', 3, NULL, 150, 10),
('walk', 'Distance Walker', 'Walk 1000 meters', 1000, NULL, 100, 5);

-- ==========================================
-- SEED DATA: ACHIEVEMENTS
-- ==========================================

INSERT INTO achievements (name, description, category, tier, requirement_type, requirement_value, xp_reward, breach_reward) VALUES
-- Capture achievements
('First Catch', 'Capture your first Titan', 'capture', 1, 'titans_captured', 1, 100, 10),
('Novice Hunter', 'Capture 10 Titans', 'capture', 1, 'titans_captured', 10, 500, 50),
('Skilled Hunter', 'Capture 50 Titans', 'capture', 2, 'titans_captured', 50, 2000, 200),
('Expert Hunter', 'Capture 100 Titans', 'capture', 2, 'titans_captured', 100, 5000, 500),
('Master Hunter', 'Capture 500 Titans', 'capture', 3, 'titans_captured', 500, 25000, 2500),
('Legendary Hunter', 'Capture 1000 Titans', 'capture', 3, 'titans_captured', 1000, 100000, 10000),

-- Battle achievements
('First Victory', 'Win your first battle', 'battle', 1, 'battles_won', 1, 100, 10),
('Battle Novice', 'Win 10 battles', 'battle', 1, 'battles_won', 10, 500, 50),
('Battle Veteran', 'Win 50 battles', 'battle', 2, 'battles_won', 50, 2000, 200),
('Battle Champion', 'Win 100 battles', 'battle', 2, 'battles_won', 100, 5000, 500),
('Battle Legend', 'Win 500 battles', 'battle', 3, 'battles_won', 500, 25000, 2500),

-- Collection achievements (element mastery)
('Volcanic Initiate', 'Own 5 Volcanic Titans', 'collection', 1, 'volcanic_owned', 5, 500, 50),
('Volcanic Master', 'Own 25 Volcanic Titans', 'collection', 3, 'volcanic_owned', 25, 5000, 500),
('Abyssal Initiate', 'Own 5 Abyssal Titans', 'collection', 1, 'abyssal_owned', 5, 500, 50),
('Abyssal Master', 'Own 25 Abyssal Titans', 'collection', 3, 'abyssal_owned', 25, 5000, 500),

-- Level achievements
('Level 5', 'Reach level 5', 'exploration', 1, 'level', 5, 200, 20),
('Level 10', 'Reach level 10', 'exploration', 1, 'level', 10, 500, 50),
('Level 25', 'Reach level 25', 'exploration', 2, 'level', 25, 2500, 250),
('Level 50', 'Reach level 50', 'exploration', 3, 'level', 50, 10000, 1000),

-- Streak achievements
('Week Warrior', 'Login 7 days in a row', 'special', 1, 'login_streak', 7, 1000, 100),
('Monthly Master', 'Login 30 days in a row', 'special', 2, 'login_streak', 30, 5000, 500),
('Dedicated Player', 'Login 100 days in a row', 'special', 3, 'login_streak', 100, 25000, 2500);

-- ==========================================
-- FUNCTIONS
-- ==========================================

-- Function to update leaderboard cache
CREATE OR REPLACE FUNCTION refresh_leaderboard(lb_type leaderboard_type, region_filter VARCHAR DEFAULT NULL)
RETURNS VOID AS $$
BEGIN
    DELETE FROM leaderboard_cache 
    WHERE leaderboard_type = lb_type 
      AND (region_filter IS NULL OR region = region_filter);
    
    IF lb_type = 'experience' THEN
        INSERT INTO leaderboard_cache (leaderboard_type, player_id, rank, score, region)
        SELECT 'experience', id, ROW_NUMBER() OVER (ORDER BY experience DESC), experience, region_filter
        FROM players WHERE is_banned = false
        ORDER BY experience DESC
        LIMIT 1000;
    ELSIF lb_type = 'captures' THEN
        INSERT INTO leaderboard_cache (leaderboard_type, player_id, rank, score, region)
        SELECT 'captures', id, ROW_NUMBER() OVER (ORDER BY titans_captured DESC), titans_captured, region_filter
        FROM players WHERE is_banned = false
        ORDER BY titans_captured DESC
        LIMIT 1000;
    ELSIF lb_type = 'battles' THEN
        INSERT INTO leaderboard_cache (leaderboard_type, player_id, rank, score, region)
        SELECT 'battles', id, ROW_NUMBER() OVER (ORDER BY battles_won DESC), battles_won, region_filter
        FROM players WHERE is_banned = false
        ORDER BY battles_won DESC
        LIMIT 1000;
    ELSIF lb_type = 'breach' THEN
        INSERT INTO leaderboard_cache (leaderboard_type, player_id, rank, score, region)
        SELECT 'breach', id, ROW_NUMBER() OVER (ORDER BY breach_earned DESC), breach_earned, region_filter
        FROM players WHERE is_banned = false
        ORDER BY breach_earned DESC
        LIMIT 1000;
    END IF;
END;
$$ LANGUAGE plpgsql;
