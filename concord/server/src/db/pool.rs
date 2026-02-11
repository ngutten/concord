use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::str::FromStr;
use tracing::info;

/// Create and initialize a SQLite connection pool with WAL mode.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    info!("database connected: {}", database_url);
    Ok(pool)
}

/// Split SQL text into statements, respecting BEGIN...END blocks (triggers).
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_begin = false;

    for line in sql.lines() {
        let trimmed = line.trim();
        // Skip pure comment lines outside of a statement
        if trimmed.starts_with("--") && current.trim().is_empty() {
            continue;
        }

        current.push_str(line);
        current.push('\n');

        let upper = trimmed.to_uppercase();
        if upper.starts_with("BEGIN") || upper.ends_with(" BEGIN") {
            in_begin = true;
        }

        if in_begin {
            if upper.starts_with("END;") || upper == "END" {
                in_begin = false;
                let stmt = current.trim().to_string();
                // Remove trailing semicolon for consistency
                let stmt = stmt.strip_suffix(';').unwrap_or(&stmt).trim().to_string();
                if !stmt.is_empty() {
                    statements.push(stmt);
                }
                current.clear();
            }
        } else {
            // Outside BEGIN..END: split on semicolons
            while let Some(pos) = current.find(';') {
                let stmt = current[..pos].trim().to_string();
                if !stmt.is_empty() && !stmt.starts_with("--") {
                    statements.push(stmt);
                }
                current = current[pos + 1..].to_string();
            }
        }
    }

    // Any remaining text
    let remaining = current.trim().to_string();
    if !remaining.is_empty() && !remaining.starts_with("--") {
        let remaining = remaining.strip_suffix(';').unwrap_or(&remaining).trim().to_string();
        if !remaining.is_empty() {
            statements.push(remaining);
        }
    }

    statements
}

/// Run all pending migration SQL files against the database.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Ensure schema_version table exists for tracking
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_version (\
            version     INTEGER PRIMARY KEY, \
            applied_at  TEXT NOT NULL DEFAULT (datetime('now'))\
        )",
    )
    .execute(pool)
    .await?;

    let current_version: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM schema_version")
            .fetch_one(pool)
            .await?;

    let migrations: &[(i64, &str)] = &[
        (1, include_str!("../../migrations/001_initial.sql")),
        (2, include_str!("../../migrations/002_servers.sql")),
        (
            3,
            include_str!("../../migrations/003_messaging_enhancements.sql"),
        ),
        (
            4,
            include_str!("../../migrations/004_media_files.sql"),
        ),
        (
            5,
            include_str!("../../migrations/005_atproto_blob_storage.sql"),
        ),
        (
            6,
            include_str!("../../migrations/006_server_config.sql"),
        ),
        (
            7,
            include_str!("../../migrations/007_organization_permissions.sql"),
        ),
        (
            8,
            include_str!("../../migrations/008_user_experience.sql"),
        ),
        (
            9,
            include_str!("../../migrations/009_threads_pinning.sql"),
        ),
        (
            10,
            include_str!("../../migrations/010_moderation.sql"),
        ),
        (
            11,
            include_str!("../../migrations/011_community.sql"),
        ),
    ];

    for &(version, sql) in migrations {
        if version <= current_version {
            continue;
        }
        info!("applying migration {version}...");
        // Use a single connection so PRAGMAs persist across statements
        let mut conn = pool.acquire().await?;
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *conn)
            .await?;
        for statement in split_sql_statements(sql) {
            if !statement.is_empty() {
                sqlx::query(&statement).execute(&mut *conn).await?;
            }
        }
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *conn)
            .await?;
        // Record the migration version (some older migrations do this themselves,
        // but we always do it here to ensure it's tracked)
        sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (?)")
            .bind(version)
            .execute(&mut *conn)
            .await?;
    }

    // Rebuild FTS index to fix any duplicates from prior migration re-runs
    let fts_exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='messages_fts'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if fts_exists {
        let mut conn = pool.acquire().await?;
        let _ = sqlx::query("INSERT INTO messages_fts(messages_fts) VALUES('rebuild')")
            .execute(&mut *conn)
            .await;
    }

    let final_version = migrations.last().map(|m| m.0).unwrap_or(0);
    info!("database migrations applied (version: {final_version})");
    Ok(())
}
