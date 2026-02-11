use sqlx::SqlitePool;

use crate::db::models::{CreateServerEventParams, EventRsvpRow, ServerEventRow};

pub async fn create_event(
    pool: &SqlitePool,
    params: &CreateServerEventParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO server_events (id, server_id, name, description, channel_id, start_time, end_time, image_url, created_by) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(params.id)
    .bind(params.server_id)
    .bind(params.name)
    .bind(params.description)
    .bind(params.channel_id)
    .bind(params.start_time)
    .bind(params.end_time)
    .bind(params.image_url)
    .bind(params.created_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_event(
    pool: &SqlitePool,
    event_id: &str,
) -> Result<Option<ServerEventRow>, sqlx::Error> {
    sqlx::query_as::<_, ServerEventRow>("SELECT * FROM server_events WHERE id = ?")
        .bind(event_id)
        .fetch_optional(pool)
        .await
}

pub async fn list_server_events(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<ServerEventRow>, sqlx::Error> {
    sqlx::query_as::<_, ServerEventRow>(
        "SELECT * FROM server_events WHERE server_id = ? ORDER BY start_time ASC",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

pub async fn update_event_status(
    pool: &SqlitePool,
    event_id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE server_events SET status = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(status)
    .bind(event_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_event(pool: &SqlitePool, event_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM server_events WHERE id = ?")
        .bind(event_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_rsvp(
    pool: &SqlitePool,
    event_id: &str,
    user_id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO event_rsvps (event_id, user_id, status) VALUES (?, ?, ?) \
         ON CONFLICT(event_id, user_id) DO UPDATE SET status = excluded.status",
    )
    .bind(event_id)
    .bind(user_id)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_rsvp(
    pool: &SqlitePool,
    event_id: &str,
    user_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM event_rsvps WHERE event_id = ? AND user_id = ?")
        .bind(event_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_rsvps(
    pool: &SqlitePool,
    event_id: &str,
) -> Result<Vec<EventRsvpRow>, sqlx::Error> {
    sqlx::query_as::<_, EventRsvpRow>(
        "SELECT * FROM event_rsvps WHERE event_id = ? ORDER BY created_at",
    )
    .bind(event_id)
    .fetch_all(pool)
    .await
}

pub async fn get_rsvp_count(pool: &SqlitePool, event_id: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM event_rsvps WHERE event_id = ?")
        .bind(event_id)
        .fetch_one(pool)
        .await
}
