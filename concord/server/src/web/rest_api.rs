use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;
use tracing::{error, info};
use uuid::Uuid;

use crate::auth::token::{generate_irc_token, hash_irc_token};
use crate::db::queries::{attachments, emoji, servers, users};
use crate::engine::events::HistoryMessage;

use super::app_state::AppState;
use super::auth_middleware::AuthUser;

// ── Channel endpoints (public, require server_id query param) ──

#[derive(Deserialize)]
pub struct HistoryParams {
    pub server_id: Option<String>,
    pub before: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct HistoryResponse {
    pub channel: String,
    pub messages: Vec<HistoryMessage>,
    pub has_more: bool,
}

#[derive(Deserialize)]
pub struct ChannelListParams {
    pub server_id: Option<String>,
}

pub async fn get_channel_history(
    State(state): State<Arc<AppState>>,
    Path(channel_name): Path<String>,
    Query(params): Query<HistoryParams>,
) -> impl IntoResponse {
    let Some(server_id) = params.server_id else {
        return (
            StatusCode::BAD_REQUEST,
            "server_id query parameter is required",
        )
            .into_response();
    };

    let channel = if channel_name.starts_with('#') {
        channel_name
    } else {
        format!("#{}", channel_name)
    };

    let limit = params.limit.unwrap_or(50).min(200);

    match state
        .engine
        .fetch_history(&server_id, &channel, params.before.as_deref(), limit)
        .await
    {
        Ok((messages, has_more)) => Json(HistoryResponse {
            channel,
            messages,
            has_more,
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn get_channels(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ChannelListParams>,
) -> impl IntoResponse {
    let Some(server_id) = params.server_id else {
        return (
            StatusCode::BAD_REQUEST,
            "server_id query parameter is required",
        )
            .into_response();
    };
    Json(state.engine.list_channels(&server_id)).into_response()
}

// ── Server endpoints (authenticated) ────────────────────

/// GET /api/servers — list the current user's servers.
pub async fn list_servers(State(state): State<Arc<AppState>>, auth: AuthUser) -> impl IntoResponse {
    Json(state.engine.list_servers_for_user(&auth.user_id))
}

#[derive(Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub icon_url: Option<String>,
}

/// POST /api/servers — create a new server.
pub async fn create_server(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateServerRequest>,
) -> impl IntoResponse {
    match state
        .engine
        .create_server(body.name, auth.user_id, body.icon_url)
        .await
    {
        Ok(server_id) => {
            let server = state
                .engine
                .list_all_servers()
                .into_iter()
                .find(|s| s.id == server_id);
            (StatusCode::CREATED, Json(server)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

/// GET /api/servers/:id — get server info.
pub async fn get_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    match state
        .engine
        .list_all_servers()
        .into_iter()
        .find(|s| s.id == server_id)
    {
        Some(server) => Json(server).into_response(),
        None => (StatusCode::NOT_FOUND, "Server not found").into_response(),
    }
}

/// DELETE /api/servers/:id — delete a server (owner only).
pub async fn delete_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    if !state.engine.is_server_owner(&server_id, &auth.user_id) {
        return (StatusCode::FORBIDDEN, "Only the server owner can delete it").into_response();
    }
    match state.engine.delete_server(&server_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

/// GET /api/servers/:id/channels — list channels in a server.
pub async fn list_server_channels(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    Json(state.engine.list_channels(&server_id))
}

/// GET /api/servers/:id/channels/:name/messages — channel history within a server.
pub async fn get_server_channel_history(
    State(state): State<Arc<AppState>>,
    Path((server_id, channel_name)): Path<(String, String)>,
    Query(params): Query<HistoryParams>,
) -> impl IntoResponse {
    let channel = if channel_name.starts_with('#') {
        channel_name
    } else {
        format!("#{}", channel_name)
    };

    let limit = params.limit.unwrap_or(50).min(200);

    match state
        .engine
        .fetch_history(&server_id, &channel, params.before.as_deref(), limit)
        .await
    {
        Ok((messages, has_more)) => Json(HistoryResponse {
            channel,
            messages,
            has_more,
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

/// GET /api/servers/:id/members — list server members.
pub async fn list_server_members(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    match servers::get_server_members(&state.db, &server_id).await {
        Ok(rows) => {
            let members: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|m| {
                    serde_json::json!({
                        "user_id": m.user_id,
                        "role": m.role,
                        "joined_at": m.joined_at,
                    })
                })
                .collect();
            Json(members).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list server members");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

// ── Admin endpoints (system admin only) ─────────────────

/// GET /api/admin/servers — list all servers (system admin).
pub async fn admin_list_servers(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> impl IntoResponse {
    match servers::is_system_admin(&state.db, &auth.user_id).await {
        Ok(true) => Json(state.engine.list_all_servers()).into_response(),
        Ok(false) => (StatusCode::FORBIDDEN, "Not a system admin").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to check admin status");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// DELETE /api/admin/servers/:id — delete any server (system admin).
pub async fn admin_delete_server(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    auth: AuthUser,
) -> impl IntoResponse {
    match servers::is_system_admin(&state.db, &auth.user_id).await {
        Ok(true) => match state.engine.delete_server(&server_id).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        },
        Ok(false) => (StatusCode::FORBIDDEN, "Not a system admin").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to check admin status");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SetAdminRequest {
    pub is_admin: bool,
}

/// PUT /api/admin/users/:id/admin — toggle system admin flag.
pub async fn admin_set_admin(
    State(state): State<Arc<AppState>>,
    Path(target_user_id): Path<String>,
    auth: AuthUser,
    Json(body): Json<SetAdminRequest>,
) -> impl IntoResponse {
    match servers::is_system_admin(&state.db, &auth.user_id).await {
        Ok(true) => {
            match servers::set_system_admin(&state.db, &target_user_id, body.is_admin).await {
                Ok(()) => StatusCode::NO_CONTENT.into_response(),
                Err(e) => {
                    error!(error = %e, "Failed to set admin flag");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
                }
            }
        }
        Ok(false) => (StatusCode::FORBIDDEN, "Not a system admin").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to check admin status");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

// ── Auth status (public) ────────────────────────────────

#[derive(Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
    pub providers: Vec<String>,
}

/// GET /api/auth/status — returns available providers and auth state.
pub async fn auth_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut providers = vec!["atproto".to_string()];
    if state.auth_config.github.is_some() {
        providers.push("github".to_string());
    }
    if state.auth_config.google.is_some() {
        providers.push("google".to_string());
    }

    Json(AuthStatusResponse {
        authenticated: false, // caller can check /api/me instead
        providers,
    })
}

// ── User profile (authenticated) ────────────────────────

#[derive(Serialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// GET /api/me — return the current user's profile.
pub async fn get_me(State(state): State<Arc<AppState>>, auth: AuthUser) -> impl IntoResponse {
    match users::get_user(&state.db, &auth.user_id).await {
        Ok(Some((id, username, email, avatar_url))) => Json(UserProfile {
            id,
            username,
            email,
            avatar_url,
        })
        .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to fetch user profile");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

// ── User profile lookup (public) ──────────────────────────

#[derive(Serialize)]
pub struct PublicUserProfile {
    pub username: String,
    pub avatar_url: Option<String>,
    pub provider: Option<String>,
    pub provider_id: Option<String>,
}

/// GET /api/users/:nickname — look up a user's public profile by nickname.
pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    Path(nickname): Path<String>,
) -> impl IntoResponse {
    match users::get_user_by_nickname(&state.db, &nickname).await {
        Ok(Some((_id, username, _email, avatar_url, provider, provider_id))) => {
            Json(PublicUserProfile {
                username,
                avatar_url,
                provider,
                provider_id,
            })
            .into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to fetch user profile");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

// ── IRC token management (authenticated) ─────────────────

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub label: Option<String>,
}

#[derive(Serialize)]
pub struct CreateTokenResponse {
    pub id: String,
    pub token: String, // plaintext, shown only once
    pub label: Option<String>,
}

#[derive(Serialize)]
pub struct IrcTokenInfo {
    pub id: String,
    pub label: Option<String>,
    pub last_used: Option<String>,
    pub created_at: String,
}

/// POST /api/tokens — generate a new IRC access token.
pub async fn create_irc_token(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let token = generate_irc_token();
    let hash = match hash_irc_token(&token) {
        Ok(h) => h,
        Err(e) => {
            error!(error = %e, "Failed to hash IRC token");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed").into_response();
        }
    };

    let token_id = Uuid::new_v4().to_string();

    if let Err(e) = users::create_irc_token(
        &state.db,
        &token_id,
        &auth.user_id,
        &hash,
        body.label.as_deref(),
    )
    .await
    {
        error!(error = %e, "Failed to store IRC token");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed").into_response();
    }

    Json(CreateTokenResponse {
        id: token_id,
        token, // shown only once
        label: body.label,
    })
    .into_response()
}

/// GET /api/tokens — list the current user's IRC tokens (no secrets).
pub async fn list_irc_tokens(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> impl IntoResponse {
    match users::list_irc_tokens(&state.db, &auth.user_id).await {
        Ok(rows) => {
            let tokens: Vec<IrcTokenInfo> = rows
                .into_iter()
                .map(|(id, label, last_used, created_at)| IrcTokenInfo {
                    id,
                    label,
                    last_used,
                    created_at,
                })
                .collect();
            Json(tokens).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list IRC tokens");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

/// DELETE /api/tokens/:id — revoke an IRC token.
pub async fn delete_irc_token(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(token_id): Path<String>,
) -> impl IntoResponse {
    match users::delete_irc_token(&state.db, &token_id, &auth.user_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Token not found").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to delete IRC token");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

// ── File upload endpoints ─────────────────────────────────

#[derive(Serialize)]
pub struct UploadResponse {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub file_size: i64,
    pub url: String,
}

/// POST /api/uploads — upload a file (multipart form data).
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut file_data: Option<(String, String, Vec<u8>)> = None; // (filename, content_type, bytes)

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            let filename = field
                .file_name()
                .unwrap_or("unnamed")
                .to_string();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            match field.bytes().await {
                Ok(bytes) => {
                    if bytes.len() as u64 > state.max_file_size {
                        return (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            format!(
                                "File too large. Max size is {} MB",
                                state.max_file_size / (1024 * 1024)
                            ),
                        )
                            .into_response();
                    }
                    file_data = Some((filename, content_type, bytes.to_vec()));
                }
                Err(e) => {
                    error!(error = %e, "Failed to read upload data");
                    return (StatusCode::BAD_REQUEST, "Failed to read file data").into_response();
                }
            }
            break;
        }
    }

    let Some((original_filename, content_type, bytes)) = file_data else {
        return (StatusCode::BAD_REQUEST, "No file field in upload").into_response();
    };

    let file_size = bytes.len() as i64;
    let attachment_id = Uuid::new_v4().to_string();

    // Sanitize filename: keep only the last path component, replace unsafe chars
    let safe_filename = original_filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("file");

    // Try uploading to the user's PDS (AT Protocol blob storage) first
    let public_url = &state.auth_config.public_url;
    let client_id = format!("{}/api/auth/atproto/v2/client-metadata.json", public_url);
    let redirect_uri = format!("{}/api/auth/atproto/callback", public_url);
    match super::pds_client::upload_blob_to_pds(
        &state.db,
        &auth.user_id,
        bytes.clone(),
        &content_type,
        &state.atproto.signing_key,
        &client_id,
        &redirect_uri,
    )
    .await
    {
        Ok(blob_ref) => {
            info!(
                user_id = %auth.user_id,
                cid = %blob_ref.cid,
                "Uploaded blob to PDS"
            );
            if let Err(e) = attachments::insert_attachment_with_blob(
                &state.db,
                &attachments::InsertBlobAttachmentParams {
                    id: &attachment_id,
                    uploader_id: &auth.user_id,
                    filename: &attachment_id,
                    original_filename: safe_filename,
                    content_type: &content_type,
                    file_size,
                    blob_cid: &blob_ref.cid,
                    blob_url: &blob_ref.url,
                },
            )
            .await
            {
                error!(error = %e, "Failed to insert blob attachment record");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
            }

            let url = format!("/api/uploads/{}", attachment_id);
            return (
                StatusCode::CREATED,
                Json(UploadResponse {
                    id: attachment_id,
                    filename: safe_filename.to_string(),
                    content_type,
                    file_size,
                    url,
                }),
            )
                .into_response();
        }
        Err(e) => {
            // No AT Protocol credentials or PDS upload failed — fall back to local storage
            info!(error = %e, "PDS blob upload unavailable, using local storage");
        }
    }

    // Fallback: store on disk as {upload_dir}/{id}
    let file_path = state.upload_dir.join(&attachment_id);
    if let Err(e) = tokio::fs::write(&file_path, &bytes).await {
        error!(error = %e, "Failed to write upload to disk");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to store file").into_response();
    }

    // Insert DB record (local storage, no blob fields)
    if let Err(e) = attachments::insert_attachment(
        &state.db,
        &attachment_id,
        &auth.user_id,
        &attachment_id, // storage filename = id
        safe_filename,
        &content_type,
        file_size,
    )
    .await
    {
        error!(error = %e, "Failed to insert attachment record");
        let _ = tokio::fs::remove_file(&file_path).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
    }

    let url = format!("/api/uploads/{}", attachment_id);

    (
        StatusCode::CREATED,
        Json(UploadResponse {
            id: attachment_id,
            filename: safe_filename.to_string(),
            content_type,
            file_size,
            url,
        }),
    )
        .into_response()
}

/// GET /api/uploads/:id — serve an uploaded file.
pub async fn get_upload(
    State(state): State<Arc<AppState>>,
    Path(attachment_id): Path<String>,
) -> impl IntoResponse {
    // Look up attachment metadata
    let attachment = match attachments::get_attachment(&state.db, &attachment_id).await {
        Ok(Some(a)) => a,
        Ok(None) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to look up attachment");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // If the attachment has a PDS blob URL, redirect to it
    if let Some(blob_url) = &attachment.blob_url {
        return axum::response::Redirect::temporary(blob_url).into_response();
    }

    // Otherwise serve from local disk
    let file_path = state.upload_dir.join(&attachment_id);
    let file = match tokio::fs::File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found on disk").into_response(),
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let content_disposition = format!(
        "inline; filename=\"{}\"",
        attachment.original_filename.replace('"', "\\\"")
    );

    (
        [
            (header::CONTENT_TYPE, attachment.content_type),
            (header::CONTENT_DISPOSITION, content_disposition),
            (
                header::CACHE_CONTROL,
                "public, max-age=31536000, immutable".to_string(),
            ),
        ],
        body,
    )
        .into_response()
}

// ── Custom emoji endpoints ──────────────────────────────────────

#[derive(Serialize)]
pub struct EmojiResponse {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub image_url: String,
}

pub async fn list_server_emoji(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
) -> impl IntoResponse {
    match emoji::list_emoji(&state.db, &server_id).await {
        Ok(rows) => {
            let list: Vec<EmojiResponse> = rows
                .into_iter()
                .map(|r| EmojiResponse {
                    id: r.id,
                    server_id: r.server_id,
                    name: r.name,
                    image_url: r.image_url,
                })
                .collect();
            Json(list).into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list emoji");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct CreateEmojiRequest {
    pub name: String,
    pub image_url: String,
}

pub async fn create_server_emoji(
    State(state): State<Arc<AppState>>,
    Path(server_id): Path<String>,
    user: AuthUser,
    Json(body): Json<CreateEmojiRequest>,
) -> impl IntoResponse {
    // Validate emoji name: alphanumeric + underscores, 2-32 chars
    let name = body.name.trim().to_lowercase();
    if name.len() < 2 || name.len() > 32 || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return (
            StatusCode::BAD_REQUEST,
            "Emoji name must be 2-32 alphanumeric/underscore characters",
        )
            .into_response();
    }

    let id = Uuid::new_v4().to_string();
    match emoji::insert_emoji(&state.db, &id, &server_id, &name, &body.image_url, &user.user_id)
        .await
    {
        Ok(()) => Json(EmojiResponse {
            id,
            server_id,
            name,
            image_url: body.image_url,
        })
        .into_response(),
        Err(e) => {
            if e.to_string().contains("UNIQUE") {
                (StatusCode::CONFLICT, "An emoji with that name already exists").into_response()
            } else {
                error!(error = %e, "Failed to create emoji");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
        }
    }
}

pub async fn delete_server_emoji(
    State(state): State<Arc<AppState>>,
    Path((_server_id, emoji_id)): Path<(String, String)>,
    _user: AuthUser,
) -> impl IntoResponse {
    match emoji::delete_emoji(&state.db, &emoji_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Emoji not found").into_response(),
        Err(e) => {
            error!(error = %e, "Failed to delete emoji");
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}
