use sqlx::SqlitePool;

use crate::db::models::{AutomodRuleRow, CreateAutomodRuleParams};

pub async fn create_rule(
    pool: &SqlitePool,
    params: &CreateAutomodRuleParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO automod_rules (id, server_id, name, rule_type, config, action_type, timeout_duration_seconds) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(params.id)
    .bind(params.server_id)
    .bind(params.name)
    .bind(params.rule_type)
    .bind(params.config)
    .bind(params.action_type)
    .bind(params.timeout_duration_seconds)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_rule(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    enabled: bool,
    config: &str,
    action_type: &str,
    timeout_duration_seconds: Option<i32>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE automod_rules SET name = ?, enabled = ?, config = ?, action_type = ?, timeout_duration_seconds = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(name)
    .bind(enabled as i32)
    .bind(config)
    .bind(action_type)
    .bind(timeout_duration_seconds)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_rule(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM automod_rules WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_rules(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<AutomodRuleRow>, sqlx::Error> {
    sqlx::query_as::<_, AutomodRuleRow>(
        "SELECT * FROM automod_rules WHERE server_id = ? ORDER BY created_at",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

pub async fn get_enabled_rules(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<AutomodRuleRow>, sqlx::Error> {
    sqlx::query_as::<_, AutomodRuleRow>(
        "SELECT * FROM automod_rules WHERE server_id = ? AND enabled = 1 ORDER BY created_at",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}
