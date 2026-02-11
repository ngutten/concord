-- Migration 008: User Experience (Phase 4)
-- Presence, custom status, profiles, server nicknames, notification settings

-- ============================================================
-- 1. User Presence & Custom Status
-- ============================================================

CREATE TABLE IF NOT EXISTS user_presence (
    user_id       TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    status        TEXT NOT NULL DEFAULT 'online' CHECK(status IN ('online', 'idle', 'dnd', 'invisible', 'offline')),
    custom_status TEXT,
    status_emoji  TEXT,
    last_seen_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 2. User Profiles (bio, pronouns, banner)
-- ============================================================

CREATE TABLE IF NOT EXISTS user_profiles (
    user_id    TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    bio        TEXT,
    pronouns   TEXT,
    banner_url TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================
-- 4. Notification Settings
-- ============================================================

CREATE TABLE IF NOT EXISTS notification_settings (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    server_id       TEXT REFERENCES servers(id) ON DELETE CASCADE,
    channel_id      TEXT REFERENCES channels(id) ON DELETE CASCADE,
    level           TEXT NOT NULL DEFAULT 'default' CHECK(level IN ('all', 'mentions', 'none', 'default')),
    suppress_everyone INTEGER NOT NULL DEFAULT 0,
    suppress_roles  INTEGER NOT NULL DEFAULT 0,
    muted           INTEGER NOT NULL DEFAULT 0,
    mute_until      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_notification_user ON notification_settings(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_notification_scope ON notification_settings(user_id, server_id, channel_id);

-- ============================================================
-- 5. Full-Text Search for Messages
-- ============================================================

CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    content,
    content='messages',
    content_rowid='rowid',
    tokenize='unicode61'
);

-- Populate FTS index from existing messages
INSERT INTO messages_fts(rowid, content)
    SELECT rowid, content FROM messages WHERE deleted_at IS NULL;

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS messages_fts_insert AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;

CREATE TRIGGER IF NOT EXISTS messages_fts_delete AFTER UPDATE OF deleted_at ON messages
    WHEN new.deleted_at IS NOT NULL AND old.deleted_at IS NULL BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', old.rowid, old.content);
END;

CREATE TRIGGER IF NOT EXISTS messages_fts_update AFTER UPDATE OF content ON messages
    WHEN new.deleted_at IS NULL BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', old.rowid, old.content);
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;
