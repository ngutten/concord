-- Migration 011: Community & Discovery
-- Phase 7: invites, welcome screen, rules, discovery, events, announcements, templates, insights

-- 1. Server invites with expiry and use limits
CREATE TABLE IF NOT EXISTS invites (
    id          TEXT PRIMARY KEY,
    server_id   TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    code        TEXT NOT NULL UNIQUE,
    created_by  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    max_uses    INTEGER,
    use_count   INTEGER NOT NULL DEFAULT 0,
    expires_at  TEXT,
    channel_id  TEXT REFERENCES channels(id) ON DELETE SET NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_invites_server ON invites(server_id);
CREATE INDEX IF NOT EXISTS idx_invites_code ON invites(code);

-- 2. Scheduled events with RSVP
CREATE TABLE IF NOT EXISTS server_events (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    description     TEXT,
    channel_id      TEXT REFERENCES channels(id) ON DELETE SET NULL,
    start_time      TEXT NOT NULL,
    end_time        TEXT,
    image_url       TEXT,
    created_by      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status          TEXT NOT NULL DEFAULT 'scheduled' CHECK(status IN ('scheduled', 'active', 'completed', 'cancelled')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_events_server ON server_events(server_id, start_time);

CREATE TABLE IF NOT EXISTS event_rsvps (
    event_id    TEXT NOT NULL REFERENCES server_events(id) ON DELETE CASCADE,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'interested' CHECK(status IN ('interested', 'going')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (event_id, user_id)
);

-- 3. Channel follows for announcement cross-posting
CREATE TABLE IF NOT EXISTS channel_follows (
    id                  TEXT PRIMARY KEY,
    source_channel_id   TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    target_channel_id   TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    created_by          TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_channel_id, target_channel_id)
);
CREATE INDEX IF NOT EXISTS idx_follows_source ON channel_follows(source_channel_id);

-- 4. Server templates
CREATE TABLE IF NOT EXISTS server_templates (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    server_id   TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    created_by  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    config      TEXT NOT NULL DEFAULT '{}',
    use_count   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_templates_server ON server_templates(server_id);

-- 5. Add community fields to servers
ALTER TABLE servers ADD COLUMN description TEXT;
ALTER TABLE servers ADD COLUMN is_discoverable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE servers ADD COLUMN welcome_message TEXT;
ALTER TABLE servers ADD COLUMN rules_text TEXT;
ALTER TABLE servers ADD COLUMN category TEXT;

-- 6. Add rules_accepted to server_members
ALTER TABLE server_members ADD COLUMN rules_accepted INTEGER NOT NULL DEFAULT 0;

-- 7. Add is_announcement to channels
ALTER TABLE channels ADD COLUMN is_announcement INTEGER NOT NULL DEFAULT 0;
