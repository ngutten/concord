use sqlx::SqlitePool;

/// Remove a member from a server (kick -- they can rejoin).
pub async fn kick_member(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    // Remove from server_members
    let result = sqlx::query("DELETE FROM server_members WHERE server_id = ? AND user_id = ?")
        .bind(server_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    // Also remove their role assignments
    let _ = sqlx::query("DELETE FROM user_roles WHERE server_id = ? AND user_id = ?")
        .bind(server_id)
        .bind(user_id)
        .execute(pool)
        .await;
    Ok(result.rows_affected() > 0)
}

/// Set timeout on a member. Pass None to clear timeout.
pub async fn set_member_timeout(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
    timeout_until: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE server_members SET timeout_until = ? WHERE server_id = ? AND user_id = ?",
    )
    .bind(timeout_until)
    .bind(server_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Get the timeout_until value for a member.
pub async fn get_member_timeout(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let result: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT timeout_until FROM server_members WHERE server_id = ? AND user_id = ?",
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(result.and_then(|r| r.0))
}

/// Set slow mode seconds on a channel.
pub async fn set_slowmode(
    pool: &SqlitePool,
    channel_id: &str,
    seconds: i32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE channels SET slowmode_seconds = ? WHERE id = ?")
        .bind(seconds)
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Set NSFW flag on a channel.
pub async fn set_nsfw(
    pool: &SqlitePool,
    channel_id: &str,
    is_nsfw: bool,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE channels SET is_nsfw = ? WHERE id = ?")
        .bind(is_nsfw as i32)
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Bulk delete messages by IDs (soft delete). Returns number of messages deleted.
pub async fn bulk_delete_messages(
    pool: &SqlitePool,
    message_ids: &[String],
) -> Result<u64, sqlx::Error> {
    if message_ids.is_empty() {
        return Ok(0);
    }
    // Build parameterized IN clause
    let placeholders: Vec<&str> = message_ids.iter().map(|_| "?").collect();
    let sql = format!(
        "UPDATE messages SET deleted_at = datetime('now') WHERE id IN ({}) AND deleted_at IS NULL",
        placeholders.join(",")
    );
    let mut query = sqlx::query(&sql);
    for id in message_ids {
        query = query.bind(id);
    }
    let result = query.execute(pool).await?;
    Ok(result.rows_affected())
}

/// Delete messages from a user in a server within the last N days (for ban purge).
pub async fn delete_user_messages(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
    days: i32,
) -> Result<u64, sqlx::Error> {
    if days <= 0 {
        return Ok(0);
    }
    let result = sqlx::query(
        "UPDATE messages SET deleted_at = datetime('now') WHERE server_id = ? AND sender_id = ? AND deleted_at IS NULL AND created_at >= datetime('now', ?)",
    )
    .bind(server_id)
    .bind(user_id)
    .bind(format!("-{days} days"))
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
