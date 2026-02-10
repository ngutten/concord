use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmojiRow {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub image_url: String,
    pub uploader_id: String,
    pub created_at: String,
}

pub async fn list_emoji(pool: &SqlitePool, server_id: &str) -> Result<Vec<EmojiRow>, sqlx::Error> {
    sqlx::query_as::<_, EmojiRow>(
        "SELECT id, server_id, name, image_url, uploader_id, created_at \
         FROM custom_emoji WHERE server_id = ? ORDER BY name",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

pub async fn get_emoji_by_name(
    pool: &SqlitePool,
    server_id: &str,
    name: &str,
) -> Result<Option<EmojiRow>, sqlx::Error> {
    sqlx::query_as::<_, EmojiRow>(
        "SELECT id, server_id, name, image_url, uploader_id, created_at \
         FROM custom_emoji WHERE server_id = ? AND name = ?",
    )
    .bind(server_id)
    .bind(name)
    .fetch_optional(pool)
    .await
}

pub async fn insert_emoji(
    pool: &SqlitePool,
    id: &str,
    server_id: &str,
    name: &str,
    image_url: &str,
    uploader_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO custom_emoji (id, server_id, name, image_url, uploader_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(server_id)
    .bind(name)
    .bind(image_url)
    .bind(uploader_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_emoji(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM custom_emoji WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
