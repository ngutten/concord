-- Migration 007: Organization & Permissions (Phase 3)

-- ============================================================
-- 1. Custom Roles
-- ============================================================

CREATE TABLE IF NOT EXISTS roles (
    id          TEXT PRIMARY KEY,
    server_id   TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    color       TEXT,
    icon_url    TEXT,
    position    INTEGER NOT NULL DEFAULT 0,
    permissions INTEGER NOT NULL DEFAULT 0,
    is_default  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(server_id, name)
);

CREATE INDEX IF NOT EXISTS idx_roles_server ON roles(server_id, position DESC);

-- Join table: users can have multiple roles per server
CREATE TABLE IF NOT EXISTS user_roles (
    server_id   TEXT NOT NULL,
    user_id     TEXT NOT NULL,
    role_id     TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (server_id, user_id, role_id),
    FOREIGN KEY (server_id, user_id) REFERENCES server_members(server_id, user_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user ON user_roles(user_id, server_id);

-- ============================================================
-- 2. Channel Categories
-- ============================================================

CREATE TABLE IF NOT EXISTS channel_categories (
    id          TEXT PRIMARY KEY,
    server_id   TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(server_id, name)
);

CREATE INDEX IF NOT EXISTS idx_categories_server ON channel_categories(server_id, position);

-- Add category, position, and visibility to channels
ALTER TABLE channels ADD COLUMN category_id TEXT REFERENCES channel_categories(id) ON DELETE SET NULL;
ALTER TABLE channels ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
ALTER TABLE channels ADD COLUMN is_private INTEGER NOT NULL DEFAULT 0;

-- ============================================================
-- 3. Channel Permission Overrides
-- ============================================================

-- target_type: 'role' or 'user'
-- allow_bits / deny_bits: same bitfield format as roles.permissions
CREATE TABLE IF NOT EXISTS channel_permission_overrides (
    id          TEXT PRIMARY KEY,
    channel_id  TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    target_type TEXT NOT NULL CHECK(target_type IN ('role', 'user')),
    target_id   TEXT NOT NULL,
    allow_bits  INTEGER NOT NULL DEFAULT 0,
    deny_bits   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(channel_id, target_type, target_id)
);

CREATE INDEX IF NOT EXISTS idx_overrides_channel ON channel_permission_overrides(channel_id);
