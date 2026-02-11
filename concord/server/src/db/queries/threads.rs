use sqlx::SqlitePool;

use crate::db::models::ChannelRow;

/// Create a thread (stored as a channel row with thread-specific fields).
pub async fn create_thread(
    pool: &SqlitePool,
    channel_id: &str,
    server_id: &str,
    name: &str,
    channel_type: &str,
    parent_message_id: &str,
    auto_archive_minutes: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO channels (id, server_id, name, channel_type, thread_parent_message_id, \
         thread_auto_archive_minutes) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(channel_id)
    .bind(server_id)
    .bind(name)
    .bind(channel_type)
    .bind(parent_message_id)
    .bind(auto_archive_minutes)
    .execute(pool)
    .await?;
    Ok(())
}

/// Archive a thread.
pub async fn archive_thread(pool: &SqlitePool, channel_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE channels SET archived = 1 WHERE id = ?")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Unarchive a thread.
pub async fn unarchive_thread(pool: &SqlitePool, channel_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE channels SET archived = 0 WHERE id = ?")
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get all threads whose parent message lives in the given channel.
pub async fn get_threads_for_channel(
    pool: &SqlitePool,
    parent_channel_id: &str,
    server_id: &str,
) -> Result<Vec<ChannelRow>, sqlx::Error> {
    sqlx::query_as::<_, ChannelRow>(
        "SELECT c.* FROM channels c \
         JOIN messages m ON c.thread_parent_message_id = m.id \
         WHERE m.channel_id = ? AND c.server_id = ? \
         AND c.channel_type IN ('public_thread', 'private_thread')",
    )
    .bind(parent_channel_id)
    .bind(server_id)
    .fetch_all(pool)
    .await
}
