use sqlx::SqlitePool;

use crate::db::models::PinnedMessageRow;

/// Pin a message in a channel. Ignores if already pinned.
pub async fn pin_message(
    pool: &SqlitePool,
    id: &str,
    channel_id: &str,
    message_id: &str,
    pinned_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO pinned_messages (id, channel_id, message_id, pinned_by) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(channel_id)
    .bind(message_id)
    .bind(pinned_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// Unpin a message from a channel.
pub async fn unpin_message(
    pool: &SqlitePool,
    channel_id: &str,
    message_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM pinned_messages WHERE channel_id = ? AND message_id = ?")
        .bind(channel_id)
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get all pinned messages in a channel, ordered by most recently pinned first.
pub async fn get_pinned_messages(
    pool: &SqlitePool,
    channel_id: &str,
) -> Result<Vec<PinnedMessageRow>, sqlx::Error> {
    sqlx::query_as::<_, PinnedMessageRow>(
        "SELECT * FROM pinned_messages WHERE channel_id = ? ORDER BY pinned_at DESC",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

/// Count the number of pinned messages in a channel.
pub async fn count_pins(pool: &SqlitePool, channel_id: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM pinned_messages WHERE channel_id = ?")
        .bind(channel_id)
        .fetch_one(pool)
        .await
}

/// Check if a specific message is pinned in a channel.
pub async fn is_pinned(
    pool: &SqlitePool,
    channel_id: &str,
    message_id: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM pinned_messages WHERE channel_id = ? AND message_id = ?")
            .bind(channel_id)
            .bind(message_id)
            .fetch_one(pool)
            .await?;
    Ok(count > 0)
}
