use sqlx::SqlitePool;

use crate::db::models::{ChannelMemberRow, ChannelRow};

/// Ensure a channel exists in the database, creating it if needed.
pub async fn ensure_channel(pool: &SqlitePool, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR IGNORE INTO channels (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get a channel by name.
pub async fn get_channel(pool: &SqlitePool, name: &str) -> Result<Option<ChannelRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, ChannelRow>("SELECT * FROM channels WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// List all channels.
pub async fn list_channels(pool: &SqlitePool) -> Result<Vec<ChannelRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ChannelRow>("SELECT * FROM channels ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Get all default channels (auto-join on registration).
pub async fn get_default_channels(pool: &SqlitePool) -> Result<Vec<ChannelRow>, sqlx::Error> {
    let rows =
        sqlx::query_as::<_, ChannelRow>("SELECT * FROM channels WHERE is_default = 1 ORDER BY name")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

/// Update a channel's topic.
pub async fn set_topic(
    pool: &SqlitePool,
    channel_name: &str,
    topic: &str,
    set_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE channels SET topic = ?, topic_set_by = ?, topic_set_at = datetime('now') WHERE name = ?",
    )
    .bind(topic)
    .bind(set_by)
    .bind(channel_name)
    .execute(pool)
    .await?;
    Ok(())
}

/// Add a member to a channel (persistent membership).
pub async fn add_member(
    pool: &SqlitePool,
    channel_name: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO channel_members (channel_name, user_id) VALUES (?, ?)",
    )
    .bind(channel_name)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a member from a channel.
pub async fn remove_member(
    pool: &SqlitePool,
    channel_name: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM channel_members WHERE channel_name = ? AND user_id = ?")
        .bind(channel_name)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get all members of a channel.
pub async fn get_members(
    pool: &SqlitePool,
    channel_name: &str,
) -> Result<Vec<ChannelMemberRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ChannelMemberRow>(
        "SELECT * FROM channel_members WHERE channel_name = ? ORDER BY joined_at",
    )
    .bind(channel_name)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get all channels a user is a member of.
pub async fn get_user_channels(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<ChannelMemberRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ChannelMemberRow>(
        "SELECT * FROM channel_members WHERE user_id = ? ORDER BY channel_name",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
