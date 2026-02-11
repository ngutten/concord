use sqlx::SqlitePool;

use crate::db::models::InviteRow;

#[allow(clippy::too_many_arguments)]
pub async fn create_invite(
    pool: &SqlitePool,
    id: &str,
    server_id: &str,
    code: &str,
    created_by: &str,
    max_uses: Option<i32>,
    expires_at: Option<&str>,
    channel_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO invites (id, server_id, code, created_by, max_uses, expires_at, channel_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(server_id)
    .bind(code)
    .bind(created_by)
    .bind(max_uses)
    .bind(expires_at)
    .bind(channel_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_invite_by_code(
    pool: &SqlitePool,
    code: &str,
) -> Result<Option<InviteRow>, sqlx::Error> {
    sqlx::query_as::<_, InviteRow>("SELECT * FROM invites WHERE code = ?")
        .bind(code)
        .fetch_optional(pool)
        .await
}

pub async fn list_server_invites(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<InviteRow>, sqlx::Error> {
    sqlx::query_as::<_, InviteRow>(
        "SELECT * FROM invites WHERE server_id = ? ORDER BY created_at DESC",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

pub async fn increment_use_count(
    pool: &SqlitePool,
    invite_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE invites SET use_count = use_count + 1 WHERE id = ?")
        .bind(invite_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_invite(pool: &SqlitePool, invite_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM invites WHERE id = ?")
        .bind(invite_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_expired_invites(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM invites WHERE expires_at IS NOT NULL AND expires_at < datetime('now')",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
