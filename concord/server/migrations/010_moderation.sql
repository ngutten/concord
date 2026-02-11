-- Migration 010: Moderation (Phase 6)
-- Kick, ban, timeout, slow mode, audit log, automod, bulk delete, NSFW

-- ============================================================
-- 1. Bans table
-- ============================================================

CREATE TABLE IF NOT EXISTS bans (
    id         TEXT PRIMARY KEY,
    server_id  TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reason     TEXT,
    delete_message_days INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(server_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_bans_server ON bans(server_id);

-- ============================================================
-- 2. Timeout support on server_members
-- ============================================================

ALTER TABLE server_members ADD COLUMN timeout_until TEXT;

-- ============================================================
-- 3. Channel moderation columns
-- ============================================================

ALTER TABLE channels ADD COLUMN slowmode_seconds INTEGER NOT NULL DEFAULT 0;
ALTER TABLE channels ADD COLUMN is_nsfw INTEGER NOT NULL DEFAULT 0;

-- ============================================================
-- 4. Audit log
-- ============================================================

CREATE TABLE IF NOT EXISTS audit_log (
    id          TEXT PRIMARY KEY,
    server_id   TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    actor_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action_type TEXT NOT NULL,
    target_type TEXT,
    target_id   TEXT,
    reason      TEXT,
    changes     TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_log_server ON audit_log(server_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor ON audit_log(actor_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_target ON audit_log(target_type, target_id);

-- ============================================================
-- 5. AutoMod rules
-- ============================================================

CREATE TABLE IF NOT EXISTS automod_rules (
    id          TEXT PRIMARY KEY,
    server_id   TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    enabled     INTEGER NOT NULL DEFAULT 1,
    rule_type   TEXT NOT NULL CHECK(rule_type IN ('keyword', 'mention_spam', 'link_filter')),
    config      TEXT NOT NULL DEFAULT '{}',
    action_type TEXT NOT NULL DEFAULT 'delete' CHECK(action_type IN ('delete', 'timeout', 'flag')),
    timeout_duration_seconds INTEGER,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(server_id, name)
);

CREATE INDEX IF NOT EXISTS idx_automod_server ON automod_rules(server_id, enabled);

INSERT OR IGNORE INTO schema_version (version) VALUES (10);
