-- Migration 009: Threads & Pinning (Phase 5)
-- Message pinning, public/private threads, forum channels, bookmarks

-- ============================================================
-- 1. Pinned Messages (up to 50 per channel)
-- ============================================================

CREATE TABLE IF NOT EXISTS pinned_messages (
    id         TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    pinned_by  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pinned_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(channel_id, message_id)
);

CREATE INDEX IF NOT EXISTS idx_pinned_channel ON pinned_messages(channel_id, pinned_at DESC);

-- ============================================================
-- 2. Thread support columns on channels
-- ============================================================

ALTER TABLE channels ADD COLUMN channel_type TEXT NOT NULL DEFAULT 'text';
ALTER TABLE channels ADD COLUMN thread_parent_message_id TEXT REFERENCES messages(id) ON DELETE CASCADE;
ALTER TABLE channels ADD COLUMN thread_auto_archive_minutes INTEGER NOT NULL DEFAULT 1440;
ALTER TABLE channels ADD COLUMN archived INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_channels_thread_parent ON channels(thread_parent_message_id);
CREATE INDEX IF NOT EXISTS idx_channels_type ON channels(server_id, channel_type);

-- ============================================================
-- 3. Forum Tags
-- ============================================================

CREATE TABLE IF NOT EXISTS forum_tags (
    id         TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    emoji      TEXT,
    moderated  INTEGER NOT NULL DEFAULT 0,
    position   INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(channel_id, name)
);

CREATE INDEX IF NOT EXISTS idx_forum_tags_channel ON forum_tags(channel_id, position);

-- ============================================================
-- 4. Thread-Tag associations
-- ============================================================

CREATE TABLE IF NOT EXISTS thread_tags (
    thread_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    tag_id    TEXT NOT NULL REFERENCES forum_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (thread_id, tag_id)
);

-- ============================================================
-- 5. Personal Bookmarks
-- ============================================================

CREATE TABLE IF NOT EXISTS bookmarks (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    note       TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, message_id)
);

CREATE INDEX IF NOT EXISTS idx_bookmarks_user ON bookmarks(user_id, created_at DESC);

INSERT OR IGNORE INTO schema_version (version) VALUES (9);
