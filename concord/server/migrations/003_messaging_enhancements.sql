-- Phase 1: Core Messaging Enhancements
-- Adds support for message editing, deletion, replies, reactions, typing, and read state.

-- Message editing and soft deletion
ALTER TABLE messages ADD COLUMN edited_at TEXT;
ALTER TABLE messages ADD COLUMN deleted_at TEXT;

-- Reply threading
ALTER TABLE messages ADD COLUMN reply_to_id TEXT;

-- Emoji reactions
CREATE TABLE IF NOT EXISTS reactions (
    message_id  TEXT NOT NULL,
    user_id     TEXT NOT NULL,
    emoji       TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (message_id, user_id, emoji)
);

CREATE INDEX IF NOT EXISTS idx_reactions_message ON reactions(message_id);

-- Read state tracking
CREATE TABLE IF NOT EXISTS read_states (
    user_id             TEXT NOT NULL,
    channel_id          TEXT NOT NULL,
    last_read_message_id TEXT,
    last_read_at        TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, channel_id)
);

-- Record migration version
INSERT INTO schema_version (version) VALUES (3);
