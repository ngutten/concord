-- Concord initial schema

CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY,
    username    TEXT NOT NULL UNIQUE,
    email       TEXT,
    avatar_url  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS oauth_accounts (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider        TEXT NOT NULL,
    provider_id     TEXT NOT NULL,
    access_token    TEXT,
    refresh_token   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(provider, provider_id)
);

CREATE TABLE IF NOT EXISTS irc_tokens (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    label       TEXT,
    last_used   TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS user_nicknames (
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nickname    TEXT NOT NULL UNIQUE,
    is_primary  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, nickname)
);

CREATE TABLE IF NOT EXISTS channels (
    name        TEXT PRIMARY KEY,
    topic       TEXT NOT NULL DEFAULT '',
    topic_set_by TEXT,
    topic_set_at TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    is_default  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS channel_members (
    channel_name TEXT NOT NULL REFERENCES channels(name) ON DELETE CASCADE,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role         TEXT NOT NULL DEFAULT 'member',
    joined_at    TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (channel_name, user_id)
);

CREATE TABLE IF NOT EXISTS messages (
    id           TEXT PRIMARY KEY,
    channel_name TEXT REFERENCES channels(name) ON DELETE CASCADE,
    sender_id    TEXT NOT NULL,
    sender_nick  TEXT NOT NULL,
    content      TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    target_user_id TEXT
);

-- Indexes for query performance
CREATE INDEX IF NOT EXISTS idx_messages_channel_time ON messages(channel_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_messages_dm ON messages(target_user_id, created_at DESC)
    WHERE target_user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_oauth_user ON oauth_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_nicknames_nick ON user_nicknames(nickname);
CREATE INDEX IF NOT EXISTS idx_tokens_hash ON irc_tokens(token_hash);

-- Insert default channels
INSERT OR IGNORE INTO channels (name, topic, is_default) VALUES
    ('#general', 'Welcome to Concord!', 1),
    ('#random', 'Off-topic chat', 1);
