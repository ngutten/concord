use sqlx::SqlitePool;

use crate::db::models::BookmarkRow;

/// Add a bookmark on a message for a user. Ignores if already bookmarked.
pub async fn add_bookmark(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    message_id: &str,
    note: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO bookmarks (id, user_id, message_id, note) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(user_id)
    .bind(message_id)
    .bind(note)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a bookmark for a user on a specific message.
pub async fn remove_bookmark(
    pool: &SqlitePool,
    user_id: &str,
    message_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM bookmarks WHERE user_id = ? AND message_id = ?")
        .bind(user_id)
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// List all bookmarks for a user, ordered by most recently created first.
pub async fn list_bookmarks(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<BookmarkRow>, sqlx::Error> {
    sqlx::query_as::<_, BookmarkRow>(
        "SELECT * FROM bookmarks WHERE user_id = ? ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Check if a user has bookmarked a specific message.
pub async fn is_bookmarked(
    pool: &SqlitePool,
    user_id: &str,
    message_id: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks WHERE user_id = ? AND message_id = ?")
            .bind(user_id)
            .bind(message_id)
            .fetch_one(pool)
            .await?;
    Ok(count > 0)
}
