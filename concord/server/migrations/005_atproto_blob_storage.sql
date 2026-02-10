-- AT Protocol blob storage: store PDS credentials and blob references

-- Add AT Protocol session fields to oauth_accounts
ALTER TABLE oauth_accounts ADD COLUMN pds_url TEXT;
ALTER TABLE oauth_accounts ADD COLUMN dpop_private_key TEXT;
ALTER TABLE oauth_accounts ADD COLUMN token_expires_at TEXT;

-- Add blob reference fields to attachments
ALTER TABLE attachments ADD COLUMN blob_cid TEXT;
ALTER TABLE attachments ADD COLUMN blob_url TEXT;

INSERT OR IGNORE INTO schema_version (version) VALUES (5);
