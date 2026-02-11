use sqlx::SqlitePool;

use crate::db::models::{ChannelFollowRow, ServerRow, ServerTemplateRow};

/// List discoverable servers with optional category filter.
pub async fn list_discoverable_servers(
    pool: &SqlitePool,
    category: Option<&str>,
) -> Result<Vec<ServerRow>, sqlx::Error> {
    if let Some(cat) = category {
        sqlx::query_as::<_, ServerRow>(
            "SELECT * FROM servers WHERE is_discoverable = 1 AND category = ? ORDER BY name",
        )
        .bind(cat)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ServerRow>(
            "SELECT * FROM servers WHERE is_discoverable = 1 ORDER BY name",
        )
        .fetch_all(pool)
        .await
    }
}

/// Update server community settings.
pub async fn update_server_community(
    pool: &SqlitePool,
    server_id: &str,
    description: Option<&str>,
    is_discoverable: bool,
    welcome_message: Option<&str>,
    rules_text: Option<&str>,
    category: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE servers SET description = ?, is_discoverable = ?, welcome_message = ?, \
         rules_text = ?, category = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(description)
    .bind(if is_discoverable { 1 } else { 0 })
    .bind(welcome_message)
    .bind(rules_text)
    .bind(category)
    .bind(server_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Accept server rules for a member.
pub async fn accept_rules(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE server_members SET rules_accepted = 1 WHERE server_id = ? AND user_id = ?",
    )
    .bind(server_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Check if a member has accepted rules.
pub async fn has_accepted_rules(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let val: i32 = sqlx::query_scalar(
        "SELECT rules_accepted FROM server_members WHERE server_id = ? AND user_id = ?",
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or(0);
    Ok(val != 0)
}

/// Set channel as announcement channel.
pub async fn set_announcement_channel(
    pool: &SqlitePool,
    channel_id: &str,
    is_announcement: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE channels SET is_announcement = ? WHERE id = ?")
        .bind(if is_announcement { 1 } else { 0 })
        .bind(channel_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Create a channel follow (cross-posting).
pub async fn create_channel_follow(
    pool: &SqlitePool,
    id: &str,
    source_channel_id: &str,
    target_channel_id: &str,
    created_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO channel_follows (id, source_channel_id, target_channel_id, created_by) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(id)
    .bind(source_channel_id)
    .bind(target_channel_id)
    .bind(created_by)
    .execute(pool)
    .await?;
    Ok(())
}

/// List followers of an announcement channel.
pub async fn list_channel_follows(
    pool: &SqlitePool,
    source_channel_id: &str,
) -> Result<Vec<ChannelFollowRow>, sqlx::Error> {
    sqlx::query_as::<_, ChannelFollowRow>(
        "SELECT * FROM channel_follows WHERE source_channel_id = ?",
    )
    .bind(source_channel_id)
    .fetch_all(pool)
    .await
}

/// Delete a channel follow.
pub async fn delete_channel_follow(
    pool: &SqlitePool,
    follow_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM channel_follows WHERE id = ?")
        .bind(follow_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Create a server template.
pub async fn create_template(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    description: Option<&str>,
    server_id: &str,
    created_by: &str,
    config: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO server_templates (id, name, description, server_id, created_by, config) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(server_id)
    .bind(created_by)
    .bind(config)
    .execute(pool)
    .await?;
    Ok(())
}

/// List templates for a server.
pub async fn list_templates(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<ServerTemplateRow>, sqlx::Error> {
    sqlx::query_as::<_, ServerTemplateRow>(
        "SELECT * FROM server_templates WHERE server_id = ? ORDER BY created_at DESC",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

/// Get a specific template.
pub async fn get_template(
    pool: &SqlitePool,
    template_id: &str,
) -> Result<Option<ServerTemplateRow>, sqlx::Error> {
    sqlx::query_as::<_, ServerTemplateRow>("SELECT * FROM server_templates WHERE id = ?")
        .bind(template_id)
        .fetch_optional(pool)
        .await
}

/// Delete a template.
pub async fn delete_template(pool: &SqlitePool, template_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM server_templates WHERE id = ?")
        .bind(template_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Increment template use count.
pub async fn increment_template_use(
    pool: &SqlitePool,
    template_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE server_templates SET use_count = use_count + 1 WHERE id = ?")
        .bind(template_id)
        .execute(pool)
        .await?;
    Ok(())
}
