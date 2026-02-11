use sqlx::SqlitePool;

use crate::db::models::ForumTagRow;

/// Create a new forum tag for a channel.
pub async fn create_tag(
    pool: &SqlitePool,
    id: &str,
    channel_id: &str,
    name: &str,
    emoji: Option<&str>,
    moderated: i32,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO forum_tags (id, channel_id, name, emoji, moderated, position) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(channel_id)
    .bind(name)
    .bind(emoji)
    .bind(moderated)
    .bind(position)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update an existing forum tag.
pub async fn update_tag(
    pool: &SqlitePool,
    tag_id: &str,
    name: &str,
    emoji: Option<&str>,
    moderated: i32,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE forum_tags SET name = ?, emoji = ?, moderated = ?, position = ? WHERE id = ?",
    )
    .bind(name)
    .bind(emoji)
    .bind(moderated)
    .bind(position)
    .bind(tag_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete a forum tag by ID.
pub async fn delete_tag(pool: &SqlitePool, tag_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM forum_tags WHERE id = ?")
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// List all tags for a channel, ordered by position.
pub async fn list_tags(
    pool: &SqlitePool,
    channel_id: &str,
) -> Result<Vec<ForumTagRow>, sqlx::Error> {
    sqlx::query_as::<_, ForumTagRow>(
        "SELECT * FROM forum_tags WHERE channel_id = ? ORDER BY position",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await
}

/// Replace all tags on a thread. Deletes existing associations and inserts new ones.
pub async fn set_thread_tags(
    pool: &SqlitePool,
    thread_id: &str,
    tag_ids: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM thread_tags WHERE thread_id = ?")
        .bind(thread_id)
        .execute(pool)
        .await?;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO thread_tags (thread_id, tag_id) VALUES (?, ?)")
            .bind(thread_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Get all forum tags associated with a thread.
pub async fn get_thread_tags(
    pool: &SqlitePool,
    thread_id: &str,
) -> Result<Vec<ForumTagRow>, sqlx::Error> {
    sqlx::query_as::<_, ForumTagRow>(
        "SELECT ft.* FROM forum_tags ft \
         JOIN thread_tags tt ON ft.id = tt.tag_id \
         WHERE tt.thread_id = ? ORDER BY ft.position",
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await
}
