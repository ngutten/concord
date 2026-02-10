-- Server configuration key-value store (e.g., persistent signing keys)
CREATE TABLE IF NOT EXISTS server_config (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO schema_version (version) VALUES (6);
