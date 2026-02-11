use sqlx::SqlitePool;

use crate::db::models::{NotificationSettingRow, UpsertNotificationParams};

/// Upsert a notification setting for a user (server-level or channel-level).
pub async fn upsert_notification_setting(
    pool: &SqlitePool,
    params: &UpsertNotificationParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO notification_settings (id, user_id, server_id, channel_id, level, \
         suppress_everyone, suppress_roles, muted, mute_until, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now')) \
         ON CONFLICT(user_id, server_id, channel_id) DO UPDATE SET \
         level = excluded.level, suppress_everyone = excluded.suppress_everyone, \
         suppress_roles = excluded.suppress_roles, muted = excluded.muted, \
         mute_until = excluded.mute_until, updated_at = datetime('now')",
    )
    .bind(params.id)
    .bind(params.user_id)
    .bind(params.server_id)
    .bind(params.channel_id)
    .bind(params.level)
    .bind(params.suppress_everyone as i32)
    .bind(params.suppress_roles as i32)
    .bind(params.muted as i32)
    .bind(params.mute_until)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get notification settings for a user in a server.
pub async fn get_notification_settings(
    pool: &SqlitePool,
    user_id: &str,
    server_id: &str,
) -> Result<Vec<NotificationSettingRow>, sqlx::Error> {
    sqlx::query_as::<_, NotificationSettingRow>(
        "SELECT id, user_id, server_id, channel_id, level, suppress_everyone, \
         suppress_roles, muted, mute_until, created_at, updated_at \
         FROM notification_settings WHERE user_id = ? AND (server_id = ? OR server_id IS NULL) \
         ORDER BY channel_id NULLS FIRST",
    )
    .bind(user_id)
    .bind(server_id)
    .fetch_all(pool)
    .await
}
