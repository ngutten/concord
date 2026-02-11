use sqlx::SqlitePool;

use crate::db::models::MessageRow;

/// Full-text search messages within a server, optionally filtered by channel.
pub async fn search_messages(
    pool: &SqlitePool,
    server_id: &str,
    query: &str,
    channel_id: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<MessageRow>, i64), sqlx::Error> {
    // Use FTS5 MATCH for full-text search
    let (rows, total) = if let Some(ch_id) = channel_id {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT m.id, m.server_id, m.channel_id, m.sender_id, m.sender_nick, m.content, \
             m.created_at, m.target_user_id, m.edited_at, m.deleted_at, m.reply_to_id \
             FROM messages m \
             JOIN messages_fts f ON m.rowid = f.rowid \
             WHERE f.content MATCH ? AND m.server_id = ? AND m.channel_id = ? AND m.deleted_at IS NULL \
             ORDER BY m.created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(query)
        .bind(server_id)
        .bind(ch_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM messages m \
             JOIN messages_fts f ON m.rowid = f.rowid \
             WHERE f.content MATCH ? AND m.server_id = ? AND m.channel_id = ? AND m.deleted_at IS NULL",
        )
        .bind(query)
        .bind(server_id)
        .bind(ch_id)
        .fetch_one(pool)
        .await?;

        (rows, total.0)
    } else {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT m.id, m.server_id, m.channel_id, m.sender_id, m.sender_nick, m.content, \
             m.created_at, m.target_user_id, m.edited_at, m.deleted_at, m.reply_to_id \
             FROM messages m \
             JOIN messages_fts f ON m.rowid = f.rowid \
             WHERE f.content MATCH ? AND m.server_id = ? AND m.deleted_at IS NULL \
             ORDER BY m.created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(query)
        .bind(server_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM messages m \
             JOIN messages_fts f ON m.rowid = f.rowid \
             WHERE f.content MATCH ? AND m.server_id = ? AND m.deleted_at IS NULL",
        )
        .bind(query)
        .bind(server_id)
        .fetch_one(pool)
        .await?;

        (rows, total.0)
    };

    Ok((rows, total))
}
