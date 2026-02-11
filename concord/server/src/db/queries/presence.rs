use sqlx::SqlitePool;

use crate::db::models::UserPresenceRow;

/// Upsert a user's presence status.
pub async fn upsert_presence(
    pool: &SqlitePool,
    user_id: &str,
    status: &str,
    custom_status: Option<&str>,
    status_emoji: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_presence (user_id, status, custom_status, status_emoji, last_seen_at, updated_at) \
         VALUES (?, ?, ?, ?, datetime('now'), datetime('now')) \
         ON CONFLICT(user_id) DO UPDATE SET status = excluded.status, \
         custom_status = excluded.custom_status, status_emoji = excluded.status_emoji, \
         last_seen_at = datetime('now'), updated_at = datetime('now')",
    )
    .bind(user_id)
    .bind(status)
    .bind(custom_status)
    .bind(status_emoji)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a single user's presence.
pub async fn get_presence(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<UserPresenceRow>, sqlx::Error> {
    sqlx::query_as::<_, UserPresenceRow>(
        "SELECT user_id, status, custom_status, status_emoji, last_seen_at, updated_at \
         FROM user_presence WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Get presence for multiple users (e.g., all members of a server).
pub async fn get_presences_for_users(
    pool: &SqlitePool,
    user_ids: &[String],
) -> Result<Vec<UserPresenceRow>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<&str> = user_ids.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT user_id, status, custom_status, status_emoji, last_seen_at, updated_at \
         FROM user_presence WHERE user_id IN ({})",
        placeholders.join(", ")
    );
    let mut query = sqlx::query_as::<_, UserPresenceRow>(&sql);
    for id in user_ids {
        query = query.bind(id);
    }
    query.fetch_all(pool).await
}

/// Set user offline and record last_seen.
pub async fn set_offline(pool: &SqlitePool, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE user_presence SET status = 'offline', last_seen_at = datetime('now'), \
         updated_at = datetime('now') WHERE user_id = ?",
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}
