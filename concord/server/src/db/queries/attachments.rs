use sqlx::SqlitePool;

/// A stored attachment record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AttachmentRow {
    pub id: String,
    pub uploader_id: String,
    pub message_id: Option<String>,
    pub filename: String,
    pub original_filename: String,
    pub content_type: String,
    pub file_size: i64,
    pub created_at: String,
    pub blob_cid: Option<String>,
    pub blob_url: Option<String>,
}

/// Insert a new attachment record.
pub async fn insert_attachment(
    pool: &SqlitePool,
    id: &str,
    uploader_id: &str,
    filename: &str,
    original_filename: &str,
    content_type: &str,
    file_size: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO attachments (id, uploader_id, filename, original_filename, content_type, file_size) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(uploader_id)
    .bind(filename)
    .bind(original_filename)
    .bind(content_type)
    .bind(file_size)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a single attachment by ID.
pub async fn get_attachment(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<AttachmentRow>, sqlx::Error> {
    sqlx::query_as::<_, AttachmentRow>(
        "SELECT id, uploader_id, message_id, filename, original_filename, content_type, file_size, created_at, blob_cid, blob_url \
         FROM attachments WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Get attachments by a list of IDs.
pub async fn get_attachments_by_ids(
    pool: &SqlitePool,
    ids: &[String],
) -> Result<Vec<AttachmentRow>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT id, uploader_id, message_id, filename, original_filename, content_type, file_size, created_at, blob_cid, blob_url \
         FROM attachments WHERE id IN ({})",
        placeholders.join(", ")
    );
    let mut query = sqlx::query_as::<_, AttachmentRow>(&sql);
    for id in ids {
        query = query.bind(id);
    }
    query.fetch_all(pool).await
}

/// Get all attachments linked to a specific message.
pub async fn get_attachments_for_message(
    pool: &SqlitePool,
    message_id: &str,
) -> Result<Vec<AttachmentRow>, sqlx::Error> {
    sqlx::query_as::<_, AttachmentRow>(
        "SELECT id, uploader_id, message_id, filename, original_filename, content_type, file_size, created_at, blob_cid, blob_url \
         FROM attachments WHERE message_id = ? ORDER BY created_at",
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await
    .map(|opt| opt.into_iter().collect())
}

/// Get all attachments for a batch of message IDs.
pub async fn get_attachments_for_messages(
    pool: &SqlitePool,
    message_ids: &[String],
) -> Result<Vec<AttachmentRow>, sqlx::Error> {
    if message_ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<&str> = message_ids.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT id, uploader_id, message_id, filename, original_filename, content_type, file_size, created_at, blob_cid, blob_url \
         FROM attachments WHERE message_id IN ({}) ORDER BY created_at",
        placeholders.join(", ")
    );
    let mut query = sqlx::query_as::<_, AttachmentRow>(&sql);
    for id in message_ids {
        query = query.bind(id);
    }
    query.fetch_all(pool).await
}

/// Parameters for inserting an attachment with a PDS blob reference.
pub struct InsertBlobAttachmentParams<'a> {
    pub id: &'a str,
    pub uploader_id: &'a str,
    pub filename: &'a str,
    pub original_filename: &'a str,
    pub content_type: &'a str,
    pub file_size: i64,
    pub blob_cid: &'a str,
    pub blob_url: &'a str,
}

/// Insert a new attachment record with PDS blob reference.
pub async fn insert_attachment_with_blob(
    pool: &SqlitePool,
    params: &InsertBlobAttachmentParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO attachments (id, uploader_id, filename, original_filename, content_type, file_size, blob_cid, blob_url) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(params.id)
    .bind(params.uploader_id)
    .bind(params.filename)
    .bind(params.original_filename)
    .bind(params.content_type)
    .bind(params.file_size)
    .bind(params.blob_cid)
    .bind(params.blob_url)
    .execute(pool)
    .await?;
    Ok(())
}

/// Link attachments to a message (set message_id on matching attachment rows).
pub async fn link_attachments_to_message(
    pool: &SqlitePool,
    message_id: &str,
    attachment_ids: &[String],
    uploader_id: &str,
) -> Result<(), sqlx::Error> {
    for att_id in attachment_ids {
        sqlx::query(
            "UPDATE attachments SET message_id = ? WHERE id = ? AND uploader_id = ?",
        )
        .bind(message_id)
        .bind(att_id)
        .bind(uploader_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}
