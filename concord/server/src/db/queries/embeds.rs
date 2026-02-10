use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmbedRow {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub site_name: Option<String>,
}

pub async fn get_cached_embed(pool: &SqlitePool, url: &str) -> Result<Option<EmbedRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, EmbedRow>(
        "SELECT url, title, description, image_url, site_name \
         FROM embed_cache \
         WHERE url = ? AND datetime(fetched_at) > datetime('now', '-24 hours')",
    )
    .bind(url)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn upsert_embed(
    pool: &SqlitePool,
    url: &str,
    title: Option<&str>,
    description: Option<&str>,
    image_url: Option<&str>,
    site_name: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO embed_cache (url, title, description, image_url, site_name, fetched_at) \
         VALUES (?, ?, ?, ?, ?, datetime('now')) \
         ON CONFLICT(url) DO UPDATE SET \
           title = excluded.title, \
           description = excluded.description, \
           image_url = excluded.image_url, \
           site_name = excluded.site_name, \
           fetched_at = excluded.fetched_at",
    )
    .bind(url)
    .bind(title)
    .bind(description)
    .bind(image_url)
    .bind(site_name)
    .execute(pool)
    .await?;
    Ok(())
}
