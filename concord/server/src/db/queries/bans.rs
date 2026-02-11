use sqlx::SqlitePool;

use crate::db::models::BanRow;

pub async fn create_ban(
    pool: &SqlitePool,
    id: &str,
    server_id: &str,
    user_id: &str,
    banned_by: &str,
    reason: Option<&str>,
    delete_message_days: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO bans (id, server_id, user_id, banned_by, reason, delete_message_days) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(server_id)
    .bind(user_id)
    .bind(banned_by)
    .bind(reason)
    .bind(delete_message_days)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_ban(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM bans WHERE server_id = ? AND user_id = ?")
        .bind(server_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_ban(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<Option<BanRow>, sqlx::Error> {
    sqlx::query_as::<_, BanRow>("SELECT * FROM bans WHERE server_id = ? AND user_id = ?")
        .bind(server_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

pub async fn list_bans(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<BanRow>, sqlx::Error> {
    sqlx::query_as::<_, BanRow>(
        "SELECT * FROM bans WHERE server_id = ? ORDER BY created_at DESC",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

pub async fn is_banned(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bans WHERE server_id = ? AND user_id = ?")
            .bind(server_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(count > 0)
}
