use sqlx::SqlitePool;

use crate::db::models::UserProfileRow;

/// Upsert a user's profile.
pub async fn upsert_profile(
    pool: &SqlitePool,
    user_id: &str,
    bio: Option<&str>,
    pronouns: Option<&str>,
    banner_url: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_profiles (user_id, bio, pronouns, banner_url, updated_at) \
         VALUES (?, ?, ?, ?, datetime('now')) \
         ON CONFLICT(user_id) DO UPDATE SET \
         bio = excluded.bio, pronouns = excluded.pronouns, \
         banner_url = excluded.banner_url, updated_at = datetime('now')",
    )
    .bind(user_id)
    .bind(bio)
    .bind(pronouns)
    .bind(banner_url)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a user's profile.
pub async fn get_profile(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<UserProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, UserProfileRow>(
        "SELECT user_id, bio, pronouns, banner_url, created_at, updated_at \
         FROM user_profiles WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}
