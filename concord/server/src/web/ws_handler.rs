use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::auth::token::validate_session_token;
use crate::db::queries::users;
use crate::engine::chat_engine::{ChatEngine, DEFAULT_SERVER_ID};
use crate::engine::events::ChatEvent;
use crate::engine::user_session::Protocol;

use super::app_state::AppState;

/// Query parameters for WebSocket upgrade.
/// If authenticated via cookie, nickname is looked up from the user's profile.
/// Falls back to ?nickname= for unauthenticated dev/test usage.
#[derive(Deserialize, Default)]
pub struct WsParams {
    pub nickname: Option<String>,
}

/// Client-to-server WebSocket message types.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    SendMessage {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
        content: String,
        reply_to: Option<String>,
        attachment_ids: Option<Vec<String>>,
    },
    JoinChannel {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
    },
    PartChannel {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
        reason: Option<String>,
    },
    SetTopic {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
        topic: String,
    },
    FetchHistory {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
        before: Option<String>,
        limit: Option<i64>,
    },
    ListChannels {
        #[serde(default = "default_server_id")]
        server_id: String,
    },
    GetMembers {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
    },
    ListServers,
    CreateServer {
        name: String,
        icon_url: Option<String>,
    },
    JoinServer {
        server_id: String,
    },
    LeaveServer {
        server_id: String,
    },
    CreateChannel {
        server_id: String,
        name: String,
    },
    DeleteChannel {
        server_id: String,
        channel: String,
    },
    DeleteServer {
        server_id: String,
    },
    UpdateMemberRole {
        server_id: String,
        user_id: String,
        role: String,
    },
    EditMessage {
        message_id: String,
        content: String,
    },
    DeleteMessage {
        message_id: String,
    },
    AddReaction {
        message_id: String,
        emoji: String,
    },
    RemoveReaction {
        message_id: String,
        emoji: String,
    },
    Typing {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
    },
    MarkRead {
        #[serde(default = "default_server_id")]
        server_id: String,
        channel: String,
        message_id: String,
    },
    GetUnreadCounts {
        #[serde(default = "default_server_id")]
        server_id: String,
    },
    // ── Roles ──
    ListRoles {
        server_id: String,
    },
    CreateRole {
        server_id: String,
        name: String,
        color: Option<String>,
        permissions: Option<i64>,
    },
    UpdateRole {
        server_id: String,
        role_id: String,
        name: String,
        color: Option<String>,
        permissions: i64,
    },
    DeleteRole {
        server_id: String,
        role_id: String,
    },
    AssignRole {
        server_id: String,
        user_id: String,
        role_id: String,
    },
    RemoveRole {
        server_id: String,
        user_id: String,
        role_id: String,
    },
    // ── Categories ──
    ListCategories {
        server_id: String,
    },
    CreateCategory {
        server_id: String,
        name: String,
    },
    UpdateCategory {
        server_id: String,
        category_id: String,
        name: String,
    },
    DeleteCategory {
        server_id: String,
        category_id: String,
    },
    // ── Channel organization ──
    ReorderChannels {
        server_id: String,
        channels: Vec<crate::engine::events::ChannelPositionInfo>,
    },
    // ── Phase 4: Presence ──
    SetPresence {
        status: String,
        custom_status: Option<String>,
        status_emoji: Option<String>,
    },
    GetPresences {
        server_id: String,
    },
    // ── Phase 4: Server Nicknames ──
    SetServerNickname {
        server_id: String,
        nickname: Option<String>,
    },
    // ── Phase 4: Search ──
    SearchMessages {
        server_id: String,
        query: String,
        channel: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    },
    // ── Phase 4: Notifications ──
    UpdateNotificationSettings {
        server_id: String,
        channel_id: Option<String>,
        level: String,
        suppress_everyone: Option<bool>,
        suppress_roles: Option<bool>,
        muted: Option<bool>,
        mute_until: Option<String>,
    },
    GetNotificationSettings {
        server_id: String,
    },
    // ── Phase 4: Profiles ──
    GetUserProfile {
        user_id: String,
    },
    // ── Phase 5: Pinning ──
    PinMessage {
        server_id: String,
        channel: String,
        message_id: String,
    },
    UnpinMessage {
        server_id: String,
        channel: String,
        message_id: String,
    },
    GetPinnedMessages {
        server_id: String,
        channel: String,
    },
    // ── Phase 5: Threads ──
    CreateThread {
        server_id: String,
        parent_channel: String,
        name: String,
        message_id: String,
        #[serde(default)]
        is_private: bool,
    },
    ArchiveThread {
        server_id: String,
        thread_id: String,
    },
    ListThreads {
        server_id: String,
        channel: String,
    },
    // ── Phase 5: Bookmarks ──
    AddBookmark {
        message_id: String,
        note: Option<String>,
    },
    RemoveBookmark {
        message_id: String,
    },
    ListBookmarks,
    // ── Phase 6: Moderation ──
    KickMember {
        server_id: String,
        user_id: String,
        reason: Option<String>,
    },
    BanMember {
        server_id: String,
        user_id: String,
        reason: Option<String>,
        #[serde(default)]
        delete_message_days: i32,
    },
    UnbanMember {
        server_id: String,
        user_id: String,
    },
    ListBans {
        server_id: String,
    },
    TimeoutMember {
        server_id: String,
        user_id: String,
        timeout_until: Option<String>,
        reason: Option<String>,
    },
    SetSlowMode {
        server_id: String,
        channel: String,
        seconds: i32,
    },
    SetNsfw {
        server_id: String,
        channel: String,
        is_nsfw: bool,
    },
    BulkDeleteMessages {
        server_id: String,
        channel: String,
        message_ids: Vec<String>,
    },
    GetAuditLog {
        server_id: String,
        action_type: Option<String>,
        limit: Option<i64>,
        before: Option<String>,
    },
    // ── Phase 6: AutoMod ──
    CreateAutomodRule {
        server_id: String,
        name: String,
        rule_type: String,
        config: String,
        action_type: String,
        timeout_duration_seconds: Option<i32>,
    },
    UpdateAutomodRule {
        server_id: String,
        rule_id: String,
        name: String,
        enabled: bool,
        config: String,
        action_type: String,
        timeout_duration_seconds: Option<i32>,
    },
    DeleteAutomodRule {
        server_id: String,
        rule_id: String,
    },
    ListAutomodRules {
        server_id: String,
    },
    // ── Phase 7: Community & Discovery ──
    CreateInvite {
        server_id: String,
        max_uses: Option<i32>,
        expires_at: Option<String>,
        channel_id: Option<String>,
    },
    ListInvites {
        server_id: String,
    },
    DeleteInvite {
        server_id: String,
        invite_id: String,
    },
    UseInvite {
        code: String,
    },
    CreateEvent {
        server_id: String,
        name: String,
        description: Option<String>,
        channel_id: Option<String>,
        start_time: String,
        end_time: Option<String>,
        image_url: Option<String>,
    },
    ListEvents {
        server_id: String,
    },
    UpdateEventStatus {
        server_id: String,
        event_id: String,
        status: String,
    },
    DeleteEvent {
        server_id: String,
        event_id: String,
    },
    SetRsvp {
        server_id: String,
        event_id: String,
        status: String,
    },
    RemoveRsvp {
        server_id: String,
        event_id: String,
    },
    ListRsvps {
        event_id: String,
    },
    UpdateCommunitySettings {
        server_id: String,
        description: Option<String>,
        is_discoverable: bool,
        welcome_message: Option<String>,
        rules_text: Option<String>,
        category: Option<String>,
    },
    GetCommunitySettings {
        server_id: String,
    },
    DiscoverServers {
        category: Option<String>,
    },
    AcceptRules {
        server_id: String,
    },
    SetAnnouncementChannel {
        server_id: String,
        channel: String,
        is_announcement: bool,
    },
    FollowChannel {
        source_channel_id: String,
        target_channel_id: String,
    },
    UnfollowChannel {
        follow_id: String,
    },
    ListChannelFollows {
        channel_id: String,
    },
    CreateTemplate {
        server_id: String,
        name: String,
        description: Option<String>,
    },
    ListTemplates {
        server_id: String,
    },
    DeleteTemplate {
        server_id: String,
        template_id: String,
    },
}

fn default_server_id() -> String {
    DEFAULT_SERVER_ID.to_string()
}

pub async fn ws_upgrade(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(params): Query<WsParams>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Try cookie-based auth first
    let (nickname, user_id) = if let Some(cookie) = jar.get("concord_session") {
        if let Ok(claims) = validate_session_token(cookie.value(), &state.auth_config.jwt_secret) {
            match users::get_user(&state.db, &claims.sub).await {
                Ok(Some((id, username, _email, _avatar))) => (username, Some(id)),
                _ => {
                    return (axum::http::StatusCode::UNAUTHORIZED, "User not found")
                        .into_response();
                }
            }
        } else {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid session token",
            )
                .into_response();
        }
    } else if let Some(nick) = params.nickname {
        // Fallback: allow ?nickname= for dev/test (no auth required)
        (nick, None)
    } else {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            "Not authenticated. Provide a session cookie or ?nickname= param.",
        )
            .into_response();
    };

    // Look up avatar_url from DB if authenticated via cookie
    let avatar_url = if jar.get("concord_session").is_some() {
        if let Ok(claims) = validate_session_token(
            jar.get("concord_session").unwrap().value(),
            &state.auth_config.jwt_secret,
        ) {
            match users::get_user(&state.db, &claims.sub).await {
                Ok(Some((_id, _username, _email, avatar))) => avatar,
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    let engine = state.engine.clone();
    ws.on_upgrade(move |socket| handle_ws_connection(socket, engine, user_id, nickname, avatar_url))
        .into_response()
}

async fn handle_ws_connection(
    socket: WebSocket,
    engine: Arc<ChatEngine>,
    user_id: Option<String>,
    nickname: String,
    avatar_url: Option<String>,
) {
    let (session_id, mut event_rx) =
        match engine.connect(user_id, nickname.clone(), Protocol::WebSocket, avatar_url) {
            Ok(pair) => pair,
            Err(e) => {
                warn!(%nickname, error = %e, "WebSocket connection rejected");
                return;
            }
        };

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let write_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match serde_json::to_string(&event) {
                Ok(json) => {
                    if ws_sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!(error = %e, "failed to serialize event");
                }
            }
        }
    });

    let engine_ref = engine.clone();
    while let Some(msg_result) = ws_receiver.next().await {
        let msg = match msg_result {
            Ok(msg) => msg,
            Err(e) => {
                warn!(error = %e, "WebSocket read error");
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                handle_client_message(&engine_ref, session_id, &text).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    engine.disconnect(session_id);
    write_handle.abort();
    info!(%session_id, %nickname, "WebSocket connection closed");
}

async fn handle_client_message(
    engine: &ChatEngine,
    session_id: crate::engine::events::SessionId,
    text: &str,
) {
    let msg: ClientMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            warn!(error = %e, "invalid client message");
            return;
        }
    };

    let result = match msg {
        ClientMessage::SendMessage {
            server_id,
            channel,
            content,
            reply_to,
            attachment_ids,
        } => engine.send_message(
            session_id,
            &server_id,
            &channel,
            &content,
            reply_to.as_deref(),
            attachment_ids.as_deref(),
        ),
        ClientMessage::JoinChannel { server_id, channel } => {
            engine.join_channel(session_id, &server_id, &channel)
        }
        ClientMessage::PartChannel {
            server_id,
            channel,
            reason,
        } => engine.part_channel(session_id, &server_id, &channel, reason),
        ClientMessage::SetTopic {
            server_id,
            channel,
            topic,
        } => engine.set_topic(session_id, &server_id, &channel, topic),
        ClientMessage::FetchHistory {
            server_id,
            channel,
            before,
            limit,
        } => {
            let limit = limit.unwrap_or(50).min(200);
            match engine
                .fetch_history(&server_id, &channel, before.as_deref(), limit)
                .await
            {
                Ok((messages, has_more)) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::History {
                            server_id,
                            channel,
                            messages,
                            has_more,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::ListChannels { server_id } => {
            let channels = engine.list_channels(&server_id);
            if let Some(session) = engine.get_session(session_id) {
                let _ = session.send(ChatEvent::ChannelList {
                    server_id,
                    channels,
                });
            }
            Ok(())
        }
        ClientMessage::GetMembers { server_id, channel } => {
            match engine.get_members(&server_id, &channel) {
                Ok(member_infos) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::Names {
                            server_id,
                            channel,
                            members: member_infos,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::ListServers => {
            if let Some(session) = engine.get_session(session_id) {
                let servers = if let Some(ref uid) = session.user_id {
                    engine.list_servers_for_user(uid)
                } else {
                    engine.list_all_servers()
                };
                let _ = session.send(ChatEvent::ServerList { servers });
            }
            Ok(())
        }
        ClientMessage::CreateServer { name, icon_url } => {
            let session = engine.get_session(session_id);
            let user_id = session.as_ref().and_then(|s| s.user_id.clone());
            let Some(uid) = user_id else {
                return send_error(
                    engine,
                    session_id,
                    "AUTH_REQUIRED",
                    "Must be authenticated to create a server",
                );
            };
            match engine.create_server(name, uid, icon_url).await {
                Ok(_server_id) => {
                    if let Some(session) = engine.get_session(session_id)
                        && let Some(ref uid) = session.user_id
                    {
                        let servers = engine.list_servers_for_user(uid);
                        let _ = session.send(ChatEvent::ServerList { servers });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::JoinServer { server_id } => {
            let session = engine.get_session(session_id);
            let user_id = session.as_ref().and_then(|s| s.user_id.clone());
            let Some(uid) = user_id else {
                return send_error(
                    engine,
                    session_id,
                    "AUTH_REQUIRED",
                    "Must be authenticated to join a server",
                );
            };
            match engine.join_server(&uid, &server_id).await {
                Ok(()) => {
                    if let Some(session) = engine.get_session(session_id)
                        && let Some(ref uid) = session.user_id
                    {
                        let servers = engine.list_servers_for_user(uid);
                        let _ = session.send(ChatEvent::ServerList { servers });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::LeaveServer { server_id } => {
            let session = engine.get_session(session_id);
            let user_id = session.as_ref().and_then(|s| s.user_id.clone());
            let Some(uid) = user_id else {
                return send_error(
                    engine,
                    session_id,
                    "AUTH_REQUIRED",
                    "Must be authenticated to leave a server",
                );
            };
            match engine.leave_server(&uid, &server_id).await {
                Ok(()) => {
                    if let Some(session) = engine.get_session(session_id)
                        && let Some(ref uid) = session.user_id
                    {
                        let servers = engine.list_servers_for_user(uid);
                        let _ = session.send(ChatEvent::ServerList { servers });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::CreateChannel { server_id, name } => {
            match engine.create_channel_in_server(&server_id, &name).await {
                Ok(_) => {
                    let channels = engine.list_channels(&server_id);
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::ChannelList {
                            server_id,
                            channels,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::DeleteChannel { server_id, channel } => {
            match engine.delete_channel_in_server(&server_id, &channel).await {
                Ok(()) => {
                    let channels = engine.list_channels(&server_id);
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::ChannelList {
                            server_id,
                            channels,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::DeleteServer { server_id } => {
            let session = engine.get_session(session_id);
            let user_id = session.as_ref().and_then(|s| s.user_id.clone());
            let Some(uid) = user_id else {
                return send_error(
                    engine,
                    session_id,
                    "AUTH_REQUIRED",
                    "Must be authenticated to delete a server",
                );
            };
            if !engine.is_server_owner(&server_id, &uid) {
                return send_error(
                    engine,
                    session_id,
                    "FORBIDDEN",
                    "Only the server owner can delete it",
                );
            }
            match engine.delete_server(&server_id).await {
                Ok(()) => {
                    if let Some(session) = engine.get_session(session_id)
                        && let Some(ref uid) = session.user_id
                    {
                        let servers = engine.list_servers_for_user(uid);
                        let _ = session.send(ChatEvent::ServerList { servers });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::UpdateMemberRole {
            server_id,
            user_id,
            role,
        } => {
            if let Some(pool) = engine.db() {
                crate::db::queries::servers::update_member_role(pool, &server_id, &user_id, &role)
                    .await
                    .map_err(|e| format!("Failed to update role: {e}"))
            } else {
                Err("No database configured".into())
            }
        }
        ClientMessage::EditMessage {
            message_id,
            content,
        } => engine.edit_message(session_id, &message_id, &content).await,
        ClientMessage::DeleteMessage { message_id } => {
            engine.delete_message(session_id, &message_id).await
        }
        ClientMessage::AddReaction { message_id, emoji } => {
            engine.add_reaction(session_id, &message_id, &emoji).await
        }
        ClientMessage::RemoveReaction { message_id, emoji } => {
            engine
                .remove_reaction(session_id, &message_id, &emoji)
                .await
        }
        ClientMessage::Typing { server_id, channel } => {
            engine.send_typing(session_id, &server_id, &channel)
        }
        ClientMessage::MarkRead {
            server_id,
            channel,
            message_id,
        } => {
            engine
                .mark_read(session_id, &server_id, &channel, &message_id)
                .await
        }
        ClientMessage::GetUnreadCounts { server_id } => {
            match engine.get_unread_counts(session_id, &server_id).await {
                Ok(counts) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::UnreadCounts {
                            server_id,
                            counts,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        // ── Roles ──
        ClientMessage::ListRoles { server_id } => {
            match engine.list_roles(&server_id).await {
                Ok(roles) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::RoleList { server_id, roles });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::CreateRole {
            server_id,
            name,
            color,
            permissions,
        } => {
            let perms = permissions.unwrap_or(0);
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_ROLES,
                )
                .await
            {
                Ok(_) => match engine
                    .create_role(&server_id, &name, color.as_deref(), perms)
                    .await
                {
                    Ok(role) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::RoleUpdate {
                                server_id,
                                role,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        ClientMessage::UpdateRole {
            server_id,
            role_id,
            name,
            color,
            permissions,
        } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_ROLES,
                )
                .await
            {
                Ok(_) => match engine
                    .update_role(&role_id, &name, color.as_deref(), permissions)
                    .await
                {
                    Ok(role) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::RoleUpdate {
                                server_id,
                                role,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        ClientMessage::DeleteRole {
            server_id,
            role_id,
        } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_ROLES,
                )
                .await
            {
                Ok(_) => match engine.delete_role(&role_id).await {
                    Ok(()) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::RoleDelete {
                                server_id,
                                role_id,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        ClientMessage::AssignRole {
            server_id,
            user_id,
            role_id,
        } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_ROLES,
                )
                .await
            {
                Ok(_) => match engine.assign_role(&server_id, &user_id, &role_id).await {
                    Ok(role_ids) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::MemberRoleUpdate {
                                server_id,
                                user_id,
                                role_ids,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        ClientMessage::RemoveRole {
            server_id,
            user_id,
            role_id,
        } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_ROLES,
                )
                .await
            {
                Ok(_) => match engine.remove_role(&server_id, &user_id, &role_id).await {
                    Ok(role_ids) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::MemberRoleUpdate {
                                server_id,
                                user_id,
                                role_ids,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        // ── Categories ──
        ClientMessage::ListCategories { server_id } => {
            match engine.list_categories(&server_id).await {
                Ok(categories) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::CategoryList {
                            server_id,
                            categories,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        ClientMessage::CreateCategory { server_id, name } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_CHANNELS,
                )
                .await
            {
                Ok(_) => match engine.create_category(&server_id, &name).await {
                    Ok(category) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::CategoryUpdate {
                                server_id,
                                category,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        ClientMessage::UpdateCategory {
            server_id,
            category_id,
            name,
        } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_CHANNELS,
                )
                .await
            {
                Ok(_) => match engine.update_category(&category_id, &name).await {
                    Ok(category) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::CategoryUpdate {
                                server_id,
                                category,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        ClientMessage::DeleteCategory {
            server_id,
            category_id,
        } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_CHANNELS,
                )
                .await
            {
                Ok(_) => match engine.delete_category(&category_id).await {
                    Ok(()) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::CategoryDelete {
                                server_id,
                                category_id,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        // ── Channel organization ──
        ClientMessage::ReorderChannels {
            server_id,
            channels,
        } => {
            match engine
                .require_permission(
                    session_id,
                    &server_id,
                    None,
                    crate::engine::permissions::Permissions::MANAGE_CHANNELS,
                )
                .await
            {
                Ok(_) => match engine.reorder_channels(&server_id, &channels).await {
                    Ok(()) => {
                        if let Some(session) = engine.get_session(session_id) {
                            let _ = session.send(ChatEvent::ChannelReorder {
                                server_id,
                                channels,
                            });
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
        // ── Phase 4: Presence ──
        ClientMessage::SetPresence {
            status,
            custom_status,
            status_emoji,
        } => {
            engine
                .set_presence(session_id, &status, custom_status.as_deref(), status_emoji.as_deref())
                .await
        }
        ClientMessage::GetPresences { server_id } => {
            match engine.get_server_presences(&server_id).await {
                Ok(presences) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::PresenceList {
                            server_id,
                            presences,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        // ── Phase 4: Server Nicknames ──
        ClientMessage::SetServerNickname { server_id, nickname } => {
            engine
                .set_server_nickname(session_id, &server_id, nickname.as_deref())
                .await
        }
        // ── Phase 4: Search ──
        ClientMessage::SearchMessages {
            server_id,
            query,
            channel,
            limit,
            offset,
        } => {
            let limit = limit.unwrap_or(25).min(50);
            let offset = offset.unwrap_or(0);
            match engine
                .search_messages(&server_id, &query, channel.as_deref(), limit, offset)
                .await
            {
                Ok((results, total_count)) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::SearchResults {
                            server_id,
                            query,
                            results,
                            total_count,
                            offset,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        // ── Phase 4: Notifications ──
        ClientMessage::UpdateNotificationSettings {
            server_id,
            channel_id,
            level,
            suppress_everyone,
            suppress_roles,
            muted,
            mute_until,
        } => {
            let params = crate::engine::chat_engine::UpdateNotificationSettingsParams {
                server_id: &server_id,
                channel_id: channel_id.as_deref(),
                level: &level,
                suppress_everyone: suppress_everyone.unwrap_or(false),
                suppress_roles: suppress_roles.unwrap_or(false),
                muted: muted.unwrap_or(false),
                mute_until: mute_until.as_deref(),
            };
            engine
                .update_notification_settings(session_id, &params)
                .await
        }
        ClientMessage::GetNotificationSettings { server_id } => {
            match engine.get_notification_settings(session_id, &server_id).await {
                Ok(settings) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::NotificationSettings {
                            server_id,
                            settings,
                        });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        // ── Phase 4: Profiles ──
        ClientMessage::GetUserProfile { user_id } => {
            match engine.get_user_profile(&user_id).await {
                Ok(profile) => {
                    if let Some(session) = engine.get_session(session_id) {
                        let _ = session.send(ChatEvent::UserProfile { profile });
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        // ── Phase 5: Pinning ──
        ClientMessage::PinMessage {
            server_id,
            channel,
            message_id,
        } => {
            engine
                .pin_message(session_id, &server_id, &channel, &message_id)
                .await
        }
        ClientMessage::UnpinMessage {
            server_id,
            channel,
            message_id,
        } => {
            engine
                .unpin_message(session_id, &server_id, &channel, &message_id)
                .await
        }
        ClientMessage::GetPinnedMessages {
            server_id,
            channel,
        } => {
            engine
                .get_pinned_messages(session_id, &server_id, &channel)
                .await
        }
        // ── Phase 5: Threads ──
        ClientMessage::CreateThread {
            server_id,
            parent_channel,
            name,
            message_id,
            is_private,
        } => {
            engine
                .create_thread(session_id, &server_id, &parent_channel, &name, &message_id, is_private)
                .await
        }
        ClientMessage::ArchiveThread {
            server_id,
            thread_id,
        } => {
            engine
                .archive_thread(session_id, &server_id, &thread_id)
                .await
        }
        ClientMessage::ListThreads {
            server_id,
            channel,
        } => {
            engine
                .list_threads(session_id, &server_id, &channel)
                .await
        }
        // ── Phase 5: Bookmarks ──
        ClientMessage::AddBookmark { message_id, note } => {
            engine
                .add_bookmark(session_id, &message_id, note.as_deref())
                .await
        }
        ClientMessage::RemoveBookmark { message_id } => {
            engine.remove_bookmark(session_id, &message_id).await
        }
        ClientMessage::ListBookmarks => engine.list_bookmarks(session_id).await,
        // ── Phase 6: Moderation ──
        ClientMessage::KickMember {
            server_id,
            user_id,
            reason,
        } => {
            engine
                .kick_member(session_id, &server_id, &user_id, reason.as_deref())
                .await
        }
        ClientMessage::BanMember {
            server_id,
            user_id,
            reason,
            delete_message_days,
        } => {
            engine
                .ban_member(session_id, &server_id, &user_id, reason.as_deref(), delete_message_days)
                .await
        }
        ClientMessage::UnbanMember {
            server_id,
            user_id,
        } => {
            engine
                .unban_member(session_id, &server_id, &user_id)
                .await
        }
        ClientMessage::ListBans { server_id } => {
            engine.list_bans(session_id, &server_id).await
        }
        ClientMessage::TimeoutMember {
            server_id,
            user_id,
            timeout_until,
            reason,
        } => {
            engine
                .timeout_member(session_id, &server_id, &user_id, timeout_until.as_deref(), reason.as_deref())
                .await
        }
        ClientMessage::SetSlowMode {
            server_id,
            channel,
            seconds,
        } => {
            engine
                .set_slowmode(session_id, &server_id, &channel, seconds)
                .await
        }
        ClientMessage::SetNsfw {
            server_id,
            channel,
            is_nsfw,
        } => {
            engine
                .set_nsfw(session_id, &server_id, &channel, is_nsfw)
                .await
        }
        ClientMessage::BulkDeleteMessages {
            server_id,
            channel,
            message_ids,
        } => {
            engine
                .bulk_delete_messages(session_id, &server_id, &channel, message_ids)
                .await
        }
        ClientMessage::GetAuditLog {
            server_id,
            action_type,
            limit,
            before,
        } => {
            let limit = limit.unwrap_or(50);
            engine
                .get_audit_log(session_id, &server_id, action_type.as_deref(), limit, before.as_deref())
                .await
        }
        // ── Phase 6: AutoMod ──
        ClientMessage::CreateAutomodRule {
            server_id,
            name,
            rule_type,
            config,
            action_type,
            timeout_duration_seconds,
        } => {
            let rule_id_placeholder = ""; // id generated inside engine
            engine
                .create_automod_rule(session_id, &crate::db::models::CreateAutomodRuleParams {
                    id: rule_id_placeholder,
                    server_id: &server_id,
                    name: &name,
                    rule_type: &rule_type,
                    config: &config,
                    action_type: &action_type,
                    timeout_duration_seconds,
                })
                .await
        }
        ClientMessage::UpdateAutomodRule {
            server_id,
            rule_id,
            name,
            enabled,
            config,
            action_type,
            timeout_duration_seconds,
        } => {
            engine
                .update_automod_rule(session_id, &crate::db::models::UpdateAutomodRuleParams {
                    rule_id: &rule_id,
                    server_id: &server_id,
                    name: &name,
                    enabled,
                    config: &config,
                    action_type: &action_type,
                    timeout_duration_seconds,
                })
                .await
        }
        ClientMessage::DeleteAutomodRule {
            server_id,
            rule_id,
        } => {
            engine
                .delete_automod_rule(session_id, &server_id, &rule_id)
                .await
        }
        ClientMessage::ListAutomodRules { server_id } => {
            engine.list_automod_rules(session_id, &server_id).await
        }
        // ── Phase 7: Community & Discovery ──
        ClientMessage::CreateInvite { server_id, max_uses, expires_at, channel_id } => {
            engine.create_invite(session_id, &server_id, max_uses, expires_at.as_deref(), channel_id.as_deref()).await
        }
        ClientMessage::ListInvites { server_id } => {
            engine.list_invites(session_id, &server_id).await
        }
        ClientMessage::DeleteInvite { server_id, invite_id } => {
            engine.delete_invite(session_id, &server_id, &invite_id).await
        }
        ClientMessage::UseInvite { code } => {
            engine.use_invite(session_id, &code).await
        }
        ClientMessage::CreateEvent { server_id, name, description, channel_id, start_time, end_time, image_url } => {
            engine.create_event(session_id, &crate::db::models::CreateServerEventParams {
                id: "",
                server_id: &server_id,
                name: &name,
                description: description.as_deref(),
                channel_id: channel_id.as_deref(),
                start_time: &start_time,
                end_time: end_time.as_deref(),
                image_url: image_url.as_deref(),
                created_by: "",
            }).await
        }
        ClientMessage::ListEvents { server_id } => {
            engine.list_events(session_id, &server_id).await
        }
        ClientMessage::UpdateEventStatus { server_id, event_id, status } => {
            engine.update_event_status(session_id, &server_id, &event_id, &status).await
        }
        ClientMessage::DeleteEvent { server_id, event_id } => {
            engine.delete_event(session_id, &server_id, &event_id).await
        }
        ClientMessage::SetRsvp { server_id, event_id, status } => {
            engine.set_rsvp(session_id, &server_id, &event_id, &status).await
        }
        ClientMessage::RemoveRsvp { server_id, event_id } => {
            engine.remove_rsvp(session_id, &server_id, &event_id).await
        }
        ClientMessage::ListRsvps { event_id } => {
            engine.list_rsvps(session_id, &event_id).await
        }
        ClientMessage::UpdateCommunitySettings { server_id, description, is_discoverable, welcome_message, rules_text, category } => {
            engine.update_community_settings(
                session_id,
                &server_id,
                description.as_deref(),
                is_discoverable,
                welcome_message.as_deref(),
                rules_text.as_deref(),
                category.as_deref(),
            ).await
        }
        ClientMessage::GetCommunitySettings { server_id } => {
            engine.get_community_settings(session_id, &server_id).await
        }
        ClientMessage::DiscoverServers { category } => {
            engine.discover_servers(session_id, category.as_deref()).await
        }
        ClientMessage::AcceptRules { server_id } => {
            engine.accept_rules(session_id, &server_id).await
        }
        ClientMessage::SetAnnouncementChannel { server_id, channel, is_announcement } => {
            engine.set_announcement_channel(session_id, &server_id, &channel, is_announcement).await
        }
        ClientMessage::FollowChannel { source_channel_id, target_channel_id } => {
            engine.follow_channel(session_id, &source_channel_id, &target_channel_id).await
        }
        ClientMessage::UnfollowChannel { follow_id } => {
            engine.unfollow_channel(session_id, &follow_id).await
        }
        ClientMessage::ListChannelFollows { channel_id } => {
            engine.list_channel_follows(session_id, &channel_id).await
        }
        ClientMessage::CreateTemplate { server_id, name, description } => {
            engine.create_template(session_id, &server_id, &name, description.as_deref()).await
        }
        ClientMessage::ListTemplates { server_id } => {
            engine.list_templates(session_id, &server_id).await
        }
        ClientMessage::DeleteTemplate { server_id, template_id } => {
            engine.delete_template(session_id, &server_id, &template_id).await
        }
    };

    if let Err(e) = result {
        send_error(engine, session_id, "COMMAND_FAILED", &e);
    }
}

fn send_error(
    engine: &ChatEngine,
    session_id: crate::engine::events::SessionId,
    code: &str,
    message: &str,
) {
    if let Some(session) = engine.get_session(session_id) {
        let _ = session.send(ChatEvent::Error {
            code: code.into(),
            message: message.into(),
        });
    }
}
