use sqlx::SqlitePool;

use crate::db::models::{AuditLogRow, CreateAuditLogParams};

pub async fn create_entry(
    pool: &SqlitePool,
    params: &CreateAuditLogParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO audit_log (id, server_id, actor_id, action_type, target_type, target_id, reason, changes) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(params.id)
    .bind(params.server_id)
    .bind(params.actor_id)
    .bind(params.action_type)
    .bind(params.target_type)
    .bind(params.target_id)
    .bind(params.reason)
    .bind(params.changes)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_entries(
    pool: &SqlitePool,
    server_id: &str,
    action_type: Option<&str>,
    limit: i64,
    before: Option<&str>,
) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    match (action_type, before) {
        (Some(at), Some(b)) => {
            sqlx::query_as::<_, AuditLogRow>(
                "SELECT * FROM audit_log WHERE server_id = ? AND action_type = ? AND created_at < ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(server_id)
            .bind(at)
            .bind(b)
            .bind(limit)
            .fetch_all(pool)
            .await
        }
        (Some(at), None) => {
            sqlx::query_as::<_, AuditLogRow>(
                "SELECT * FROM audit_log WHERE server_id = ? AND action_type = ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(server_id)
            .bind(at)
            .bind(limit)
            .fetch_all(pool)
            .await
        }
        (None, Some(b)) => {
            sqlx::query_as::<_, AuditLogRow>(
                "SELECT * FROM audit_log WHERE server_id = ? AND created_at < ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(server_id)
            .bind(b)
            .bind(limit)
            .fetch_all(pool)
            .await
        }
        (None, None) => {
            sqlx::query_as::<_, AuditLogRow>(
                "SELECT * FROM audit_log WHERE server_id = ? ORDER BY created_at DESC LIMIT ?",
            )
            .bind(server_id)
            .bind(limit)
            .fetch_all(pool)
            .await
        }
    }
}
