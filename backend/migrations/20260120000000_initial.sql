-- BREACH Backend Initial Migration
-- Creates all required database tables

-- Enable PostGIS extension
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Create custom enum types
CREATE TYPE element_type AS ENUM ('abyssal', 'volcanic', 'storm', 'void', 'parasitic', 'ossified');
CREATE TYPE poi_category AS ENUM ('landmark', 'touristattraction', 'park', 'publicsquare', 'commercial', 'educational', 'religious', 'sports', 'transportation', 'residential');
CREATE TYPE terrain_type AS ENUM ('water', 'mountain', 'urban', 'forest', 'desert', 'coastal', 'arctic');

-- Regions table
CREATE TABLE regions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    country_code CHAR(2) NOT NULL,
    timezone VARCHAR(50) NOT NULL,
    bounds GEOGRAPHY(POLYGON, 4326),
    population_density INT,
    spawn_quota INT DEFAULT 1000,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_regions_bounds ON regions USING GIST(bounds);

-- Points of Interest
CREATE TABLE pois (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_id UUID REFERENCES regions(id),
    name VARCHAR(500) NOT NULL,
    category poi_category NOT NULL,
    location_lat DOUBLE PRECISION NOT NULL,
    location_lng DOUBLE PRECISION NOT NULL,
    radius DOUBLE PRECISION DEFAULT 50.0,
    spawn_weight DOUBLE PRECISION DEFAULT 1.0,
    terrain_type terrain_type NOT NULL DEFAULT 'urban',
    osm_id VARCHAR(50),
    google_place_id VARCHAR(100),
    opening_hours JSONB,
    is_indoor BOOLEAN DEFAULT false,
    accessibility BOOLEAN DEFAULT true,
    elevation DOUBLE PRECISION,
    is_active BOOLEAN DEFAULT true,
    is_verified BOOLEAN DEFAULT false,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_pois_location ON pois(location_lat, location_lng);
CREATE INDEX idx_pois_region ON pois(region_id);
CREATE INDEX idx_pois_category ON pois(category);
CREATE INDEX idx_pois_name_trgm ON pois USING GIN(name gin_trgm_ops);
CREATE INDEX idx_pois_active ON pois(is_active) WHERE is_active = true;

-- Players table
CREATE TABLE players (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(44) UNIQUE NOT NULL,
    username VARCHAR(32),
    level INT DEFAULT 1,
    experience BIGINT DEFAULT 0,
    titans_captured INT DEFAULT 0,
    battles_won INT DEFAULT 0,
    breach_earned BIGINT DEFAULT 0,
    last_capture_at TIMESTAMPTZ,
    last_location_lat DOUBLE PRECISION,
    last_location_lng DOUBLE PRECISION,
    last_location_at TIMESTAMPTZ,
    is_banned BOOLEAN DEFAULT false,
    ban_reason TEXT,
    offense_count INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_players_wallet ON players(wallet_address);
CREATE INDEX idx_players_experience ON players(experience DESC);
CREATE INDEX idx_players_level ON players(level DESC);

-- Active Titan Spawns
CREATE TABLE titan_spawns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    poi_id UUID REFERENCES pois(id),
    location_lat DOUBLE PRECISION NOT NULL,
    location_lng DOUBLE PRECISION NOT NULL,
    geohash VARCHAR(12) NOT NULL,
    element element_type NOT NULL,
    threat_class SMALLINT NOT NULL CHECK (threat_class BETWEEN 1 AND 5),
    species_id INT NOT NULL,
    genes BYTEA NOT NULL,
    spawned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    captured_by UUID REFERENCES players(id),
    captured_at TIMESTAMPTZ,
    capture_count INT DEFAULT 0,
    max_captures INT DEFAULT 1
);

CREATE INDEX idx_titan_spawns_location ON titan_spawns(location_lat, location_lng);
CREATE INDEX idx_titan_spawns_geohash ON titan_spawns(geohash);
CREATE INDEX idx_titan_spawns_expires ON titan_spawns(expires_at) WHERE captured_by IS NULL;
CREATE INDEX idx_titan_spawns_active ON titan_spawns(expires_at) 
    WHERE captured_by IS NULL AND expires_at > NOW();
CREATE INDEX idx_titan_spawns_poi ON titan_spawns(poi_id);

-- Player location history
CREATE TABLE player_locations (
    id BIGSERIAL PRIMARY KEY,
    player_id UUID NOT NULL REFERENCES players(id),
    location_lat DOUBLE PRECISION NOT NULL,
    location_lng DOUBLE PRECISION NOT NULL,
    accuracy DOUBLE PRECISION,
    speed DOUBLE PRECISION,
    heading DOUBLE PRECISION,
    altitude DOUBLE PRECISION,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    device_id VARCHAR(100),
    is_suspicious BOOLEAN DEFAULT false,
    flags JSONB
);

CREATE INDEX idx_player_locations_player ON player_locations(player_id, timestamp DESC);
CREATE INDEX idx_player_locations_time ON player_locations(timestamp);
CREATE INDEX idx_player_locations_suspicious ON player_locations(player_id) WHERE is_suspicious = true;

-- Offenses (cheating records)
CREATE TABLE offenses (
    id BIGSERIAL PRIMARY KEY,
    player_id UUID NOT NULL REFERENCES players(id),
    offense_type VARCHAR(50) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_offenses_player ON offenses(player_id);
CREATE INDEX idx_offenses_type ON offenses(offense_type);

-- Capture records (for analytics)
CREATE TABLE capture_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id UUID NOT NULL REFERENCES players(id),
    titan_spawn_id UUID NOT NULL REFERENCES titan_spawns(id),
    poi_id UUID REFERENCES pois(id),
    element element_type NOT NULL,
    threat_class SMALLINT NOT NULL,
    species_id INT NOT NULL,
    location_lat DOUBLE PRECISION NOT NULL,
    location_lng DOUBLE PRECISION NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tx_signature VARCHAR(88)
);

CREATE INDEX idx_capture_records_player ON capture_records(player_id);
CREATE INDEX idx_capture_records_time ON capture_records(captured_at);

-- Battle records
CREATE TABLE battle_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    player1_id UUID NOT NULL REFERENCES players(id),
    player2_id UUID REFERENCES players(id), -- NULL for PvE
    winner_id UUID REFERENCES players(id),
    titan1_species INT NOT NULL,
    titan2_species INT NOT NULL,
    battle_type VARCHAR(20) NOT NULL, -- 'pve', 'pvp', 'boss'
    result JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_battle_records_player1 ON battle_records(player1_id);
CREATE INDEX idx_battle_records_player2 ON battle_records(player2_id);

-- Functions

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_players_updated_at
    BEFORE UPDATE ON players
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER update_pois_updated_at
    BEFORE UPDATE ON pois
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
