-- Chat System Migration
-- Version: 0.7.0

-- ============================================
-- 1. Chat Channel Types
-- ============================================
CREATE TYPE chat_channel_type AS ENUM ('world', 'guild', 'private', 'trade', 'help');

-- ============================================
-- 2. Chat Channels Table
-- ============================================
CREATE TABLE chat_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_type CHAT_CHANNEL_TYPE NOT NULL,
    name VARCHAR(100),
    
    -- For private channels: participants
    participant1_id UUID REFERENCES players(id) ON DELETE CASCADE,
    participant2_id UUID REFERENCES players(id) ON DELETE CASCADE,
    
    -- For guild channels
    guild_id UUID REFERENCES guilds(id) ON DELETE CASCADE,
    
    -- Channel settings
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_message_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT chk_private_participants CHECK (
        channel_type != 'private' OR (participant1_id IS NOT NULL AND participant2_id IS NOT NULL)
    ),
    CONSTRAINT chk_guild_channel CHECK (
        channel_type != 'guild' OR guild_id IS NOT NULL
    )
);

-- Indexes
CREATE INDEX idx_channels_type ON chat_channels(channel_type);
CREATE INDEX idx_channels_guild ON chat_channels(guild_id) WHERE guild_id IS NOT NULL;
CREATE INDEX idx_channels_private ON chat_channels(participant1_id, participant2_id) WHERE channel_type = 'private';

-- Ensure unique private channel between two players
CREATE UNIQUE INDEX idx_unique_private_channel ON chat_channels(
    LEAST(participant1_id, participant2_id),
    GREATEST(participant1_id, participant2_id)
) WHERE channel_type = 'private';

-- ============================================
-- 3. Chat Messages Table
-- ============================================
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    
    -- Message content
    content TEXT NOT NULL,
    
    -- Message metadata
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Reply reference
    reply_to_id UUID REFERENCES chat_messages(id) ON DELETE SET NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ
);

-- Indexes for messages
CREATE INDEX idx_messages_channel ON chat_messages(channel_id, created_at DESC);
CREATE INDEX idx_messages_sender ON chat_messages(sender_id);
CREATE INDEX idx_messages_created ON chat_messages(created_at DESC);

-- ============================================
-- 4. Chat Read Status Table
-- ============================================
CREATE TABLE chat_read_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES chat_channels(id) ON DELETE CASCADE,
    player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    
    -- Last read message
    last_read_message_id UUID REFERENCES chat_messages(id) ON DELETE SET NULL,
    last_read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Notification settings
    muted BOOLEAN NOT NULL DEFAULT FALSE,
    muted_until TIMESTAMPTZ,
    
    UNIQUE (channel_id, player_id)
);

-- Index
CREATE INDEX idx_read_status_player ON chat_read_status(player_id);

-- ============================================
-- 5. Chat Blocked Users Table
-- ============================================
CREATE TABLE chat_blocked_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    blocker_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    blocked_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE (blocker_id, blocked_id)
);

-- Indexes
CREATE INDEX idx_blocked_blocker ON chat_blocked_users(blocker_id);
CREATE INDEX idx_blocked_blocked ON chat_blocked_users(blocked_id);

-- ============================================
-- 6. Chat Reports Table
-- ============================================
CREATE TABLE chat_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    reported_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    message_id UUID REFERENCES chat_messages(id) ON DELETE SET NULL,
    
    -- Report details
    reason VARCHAR(50) NOT NULL,
    description TEXT,
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'reviewed', 'action_taken', 'dismissed')),
    
    -- Admin notes
    admin_notes TEXT,
    reviewed_by UUID REFERENCES players(id) ON DELETE SET NULL,
    reviewed_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_reports_status ON chat_reports(status) WHERE status = 'pending';
CREATE INDEX idx_reports_reported ON chat_reports(reported_id);

-- ============================================
-- 7. System Chat Channels (Pre-created)
-- ============================================

-- World chat channel
INSERT INTO chat_channels (id, channel_type, name, is_active)
VALUES ('00000000-0000-0000-0000-000000000001', 'world', 'World Chat', TRUE);

-- Trade chat channel
INSERT INTO chat_channels (id, channel_type, name, is_active)
VALUES ('00000000-0000-0000-0000-000000000002', 'trade', 'Trade Chat', TRUE);

-- Help chat channel
INSERT INTO chat_channels (id, channel_type, name, is_active)
VALUES ('00000000-0000-0000-0000-000000000003', 'help', 'Help Chat', TRUE);

-- ============================================
-- 8. Helper Functions
-- ============================================

-- Function to get or create private channel
CREATE OR REPLACE FUNCTION get_or_create_private_channel(p1_id UUID, p2_id UUID)
RETURNS UUID AS $$
DECLARE
    channel_id UUID;
BEGIN
    -- Try to find existing channel
    SELECT id INTO channel_id
    FROM chat_channels
    WHERE channel_type = 'private'
      AND ((participant1_id = p1_id AND participant2_id = p2_id)
           OR (participant1_id = p2_id AND participant2_id = p1_id));
    
    -- Create if not exists
    IF channel_id IS NULL THEN
        INSERT INTO chat_channels (channel_type, participant1_id, participant2_id)
        VALUES ('private', LEAST(p1_id, p2_id), GREATEST(p1_id, p2_id))
        RETURNING id INTO channel_id;
    END IF;
    
    RETURN channel_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get unread count
CREATE OR REPLACE FUNCTION get_unread_count(p_player_id UUID, p_channel_id UUID)
RETURNS INTEGER AS $$
DECLARE
    last_read_id UUID;
    unread INTEGER;
BEGIN
    -- Get last read message ID
    SELECT last_read_message_id INTO last_read_id
    FROM chat_read_status
    WHERE player_id = p_player_id AND channel_id = p_channel_id;
    
    IF last_read_id IS NULL THEN
        -- Count all messages
        SELECT COUNT(*) INTO unread
        FROM chat_messages
        WHERE channel_id = p_channel_id
          AND sender_id != p_player_id
          AND is_deleted = FALSE;
    ELSE
        -- Count messages after last read
        SELECT COUNT(*) INTO unread
        FROM chat_messages
        WHERE channel_id = p_channel_id
          AND sender_id != p_player_id
          AND is_deleted = FALSE
          AND created_at > (SELECT created_at FROM chat_messages WHERE id = last_read_id);
    END IF;
    
    RETURN COALESCE(unread, 0);
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- 9. Views
-- ============================================

-- View for channel list with unread counts
CREATE OR REPLACE VIEW v_player_channels AS
SELECT 
    c.id as channel_id,
    c.channel_type,
    c.name,
    c.guild_id,
    c.participant1_id,
    c.participant2_id,
    c.last_message_at,
    c.created_at
FROM chat_channels c
WHERE c.is_active = TRUE;

-- Comments
COMMENT ON TABLE chat_channels IS 'Chat channels for various communication types';
COMMENT ON TABLE chat_messages IS 'Individual chat messages';
COMMENT ON TABLE chat_read_status IS 'Tracks last read position for each player in each channel';
COMMENT ON TABLE chat_blocked_users IS 'Blocked user relationships for chat';
COMMENT ON TABLE chat_reports IS 'Reports for inappropriate messages or behavior';
