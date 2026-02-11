use sqlx::SqlitePool;

use crate::db::models::ChannelCategoryRow;

/// Create a new channel category in a server.
pub async fn create_category(
    pool: &SqlitePool,
    id: &str,
    server_id: &str,
    name: &str,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO channel_categories (id, server_id, name, position) VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(server_id)
    .bind(name)
    .bind(position)
    .execute(pool)
    .await?;
    Ok(())
}

/// List all categories in a server, ordered by position.
pub async fn list_categories(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<ChannelCategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, ChannelCategoryRow>(
        "SELECT * FROM channel_categories WHERE server_id = ? ORDER BY position",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

/// Update a category's name.
pub async fn update_category(
    pool: &SqlitePool,
    category_id: &str,
    name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE channel_categories SET name = ? WHERE id = ?")
        .bind(name)
        .bind(category_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Update a category's position.
pub async fn update_category_position(
    pool: &SqlitePool,
    category_id: &str,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE channel_categories SET position = ? WHERE id = ?")
        .bind(position)
        .bind(category_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a category. Channels in this category will have category_id set to NULL.
pub async fn delete_category(
    pool: &SqlitePool,
    category_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM channel_categories WHERE id = ?")
        .bind(category_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get a category by ID.
pub async fn get_category(
    pool: &SqlitePool,
    category_id: &str,
) -> Result<Option<ChannelCategoryRow>, sqlx::Error> {
    sqlx::query_as::<_, ChannelCategoryRow>(
        "SELECT * FROM channel_categories WHERE id = ?",
    )
    .bind(category_id)
    .fetch_optional(pool)
    .await
}
