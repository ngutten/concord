-- Phase 2: Media & Files

-- File attachments for messages
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    uploader_id TEXT NOT NULL,
    message_id TEXT,
    filename TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (uploader_id) REFERENCES users(id),
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_attachments_message ON attachments(message_id);
CREATE INDEX IF NOT EXISTS idx_attachments_uploader ON attachments(uploader_id);

-- Cache for link embed previews (Open Graph metadata)
CREATE TABLE IF NOT EXISTS embed_cache (
    url TEXT PRIMARY KEY,
    title TEXT,
    description TEXT,
    image_url TEXT,
    site_name TEXT,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Custom server emoji
CREATE TABLE IF NOT EXISTS custom_emoji (
    id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    image_url TEXT NOT NULL,
    uploader_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE,
    FOREIGN KEY (uploader_id) REFERENCES users(id),
    UNIQUE(server_id, name)
);

INSERT OR IGNORE INTO schema_version (version) VALUES (4);
