use std::sync::Arc;

use chrono::Utc;
use dashmap::DashMap;
use sqlx::SqlitePool;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::channel::ChannelState;
use super::events::{
    ChannelInfo, ChatEvent, HistoryMessage, MemberInfo, ReactionGroup, ReplyInfo, ServerInfo,
    SessionId,
};
use super::permissions::ServerRole;
use super::rate_limiter::RateLimiter;
use super::server::ServerState;
use super::user_session::{Protocol, UserSession};
use super::validation;

/// The default server ID used as a fallback for IRC clients
/// that don't specify a server. No server with this ID is pre-created;
/// IRC bare-channel operations will fail unless one is created by a user.
pub const DEFAULT_SERVER_ID: &str = "default";

/// The central hub that manages all chat state. Protocol-agnostic —
/// both IRC and WebSocket adapters call into this.
pub struct ChatEngine {
    /// All currently connected sessions, keyed by session ID.
    sessions: DashMap<SessionId, Arc<UserSession>>,
    /// All servers (guilds), keyed by server ID.
    servers: DashMap<String, ServerState>,
    /// All channels, keyed by channel UUID.
    channels: DashMap<String, ChannelState>,
    /// Index: (server_id, channel_name) -> channel_id for name-based lookups.
    channel_name_index: DashMap<(String, String), String>,
    /// Reverse lookup: nickname -> session ID (for DMs and WHOIS).
    nick_to_session: DashMap<String, SessionId>,
    /// Optional database pool. When present, messages and channels are persisted.
    db: Option<SqlitePool>,
    /// Per-user message rate limiter (burst of 10, refill 1 per second).
    message_limiter: RateLimiter,
    /// HTTP client for outbound requests (link embed unfurling).
    http_client: reqwest::Client,
}

impl ChatEngine {
    pub fn new(db: Option<SqlitePool>) -> Self {
        Self {
            sessions: DashMap::new(),
            servers: DashMap::new(),
            channels: DashMap::new(),
            channel_name_index: DashMap::new(),
            nick_to_session: DashMap::new(),
            db,
            message_limiter: RateLimiter::new(10, 1.0),
            http_client: reqwest::Client::new(),
        }
    }

    // ── Startup loading ─────────────────────────────────────────────

    /// Load servers from the database into memory on startup.
    pub async fn load_servers_from_db(&self) -> Result<(), String> {
        let Some(pool) = &self.db else {
            return Ok(());
        };

        let rows = crate::db::queries::servers::list_all_servers(pool)
            .await
            .map_err(|e| format!("Failed to load servers: {e}"))?;

        for row in rows {
            let mut state = ServerState::new(row.id.clone(), row.name, row.owner_id, row.icon_url);

            let members = crate::db::queries::servers::get_server_members(pool, &row.id)
                .await
                .map_err(|e| format!("Failed to load server members: {e}"))?;
            for m in members {
                state.member_user_ids.insert(m.user_id);
            }

            self.servers.insert(row.id, state);
        }

        info!(count = self.servers.len(), "loaded servers from database");
        Ok(())
    }

    /// Load channels from the database into memory on startup.
    pub async fn load_channels_from_db(&self) -> Result<(), String> {
        let Some(pool) = &self.db else {
            return Ok(());
        };

        // Collect server IDs first to avoid holding a read lock on self.servers
        // while later acquiring a write lock via get_mut (DashMap deadlock).
        let server_ids: Vec<String> = self.servers.iter().map(|s| s.id.clone()).collect();

        for server_id in &server_ids {
            let rows = crate::db::queries::channels::list_channels(pool, server_id)
                .await
                .map_err(|e| format!("Failed to load channels: {e}"))?;

            for row in rows {
                let mut ch =
                    ChannelState::new(row.id.clone(), row.server_id.clone(), row.name.clone());
                ch.topic = row.topic;
                ch.topic_set_by = row.topic_set_by;

                self.channel_name_index
                    .insert((row.server_id.clone(), row.name), row.id.clone());

                if let Some(mut srv) = self.servers.get_mut(&row.server_id) {
                    srv.channel_ids.insert(row.id.clone());
                }

                self.channels.insert(row.id, ch);
            }
        }

        info!(count = self.channels.len(), "loaded channels from database");
        Ok(())
    }

    // ── Session management ──────────────────────────────────────────

    /// Register a new session. Returns the session ID and an event receiver.
    pub fn connect(
        &self,
        user_id: Option<String>,
        nickname: String,
        protocol: Protocol,
        avatar_url: Option<String>,
    ) -> Result<(SessionId, mpsc::UnboundedReceiver<ChatEvent>), String> {
        validation::validate_nickname(&nickname)?;

        // If nickname is already in use, disconnect the stale session.
        if let Some(old_session_id) = self.nick_to_session.get(&nickname).map(|r| *r) {
            info!(%nickname, "replacing stale session for reconnecting user");
            self.disconnect(old_session_id);
        }

        let session_id = Uuid::new_v4();
        let (tx, rx) = mpsc::unbounded_channel();

        let session = Arc::new(UserSession::new(
            session_id,
            user_id,
            nickname.clone(),
            protocol,
            tx,
            avatar_url,
        ));

        self.sessions.insert(session_id, session);
        self.nick_to_session.insert(nickname.clone(), session_id);

        info!(%session_id, %nickname, ?protocol, "session connected");

        Ok((session_id, rx))
    }

    /// Disconnect a session and clean up all state.
    pub fn disconnect(&self, session_id: SessionId) {
        let Some((_, session)) = self.sessions.remove(&session_id) else {
            return;
        };

        let nickname = session.nickname.clone();
        self.nick_to_session.remove(&nickname);

        // Collect channels this session was in
        let channels_to_leave: Vec<String> = self
            .channels
            .iter()
            .filter(|ch| ch.members.contains(&session_id))
            .map(|ch| ch.key().clone())
            .collect();

        for channel_id in &channels_to_leave {
            if let Some(mut channel) = self.channels.get_mut(channel_id) {
                channel.members.remove(&session_id);
            }
        }

        // Broadcast quit to all channels this user was in
        let quit_event = ChatEvent::Quit {
            nickname: nickname.clone(),
            reason: None,
        };

        for channel_id in &channels_to_leave {
            self.broadcast_to_channel(channel_id, &quit_event, Some(session_id));
        }

        info!(%session_id, %nickname, "session disconnected");
    }

    // ── Server management ───────────────────────────────────────────

    /// Create a new server. Returns the server ID.
    pub async fn create_server(
        &self,
        name: String,
        owner_user_id: String,
        icon_url: Option<String>,
    ) -> Result<String, String> {
        validation::validate_server_name(&name)?;

        let server_id = Uuid::new_v4().to_string();

        if let Some(pool) = &self.db {
            crate::db::queries::servers::create_server(
                pool,
                &server_id,
                &name,
                &owner_user_id,
                icon_url.as_deref(),
            )
            .await
            .map_err(|e| format!("Failed to create server: {e}"))?;
        }

        let mut state = ServerState::new(
            server_id.clone(),
            name.clone(),
            owner_user_id.clone(),
            icon_url,
        );
        state.member_user_ids.insert(owner_user_id);
        self.servers.insert(server_id.clone(), state);

        // Create default #general channel
        let channel_id = Uuid::new_v4().to_string();
        let channel_name = "#general".to_string();
        if let Some(pool) = &self.db {
            let _ = crate::db::queries::channels::ensure_channel(
                pool,
                &channel_id,
                &server_id,
                &channel_name,
            )
            .await;
        }
        let ch = ChannelState::new(channel_id.clone(), server_id.clone(), channel_name.clone());
        self.channel_name_index
            .insert((server_id.clone(), channel_name), channel_id.clone());
        if let Some(mut srv) = self.servers.get_mut(&server_id) {
            srv.channel_ids.insert(channel_id.clone());
        }
        self.channels.insert(channel_id, ch);

        info!(%server_id, %name, "server created");
        Ok(server_id)
    }

    /// Delete a server.
    pub async fn delete_server(&self, server_id: &str) -> Result<(), String> {
        if let Some(server) = self.servers.get(server_id) {
            for ch_id in &server.channel_ids {
                if let Some((_, ch)) = self.channels.remove(ch_id) {
                    self.channel_name_index
                        .remove(&(server_id.to_string(), ch.name));
                }
            }
        }

        self.servers.remove(server_id);

        if let Some(pool) = &self.db {
            crate::db::queries::servers::delete_server(pool, server_id)
                .await
                .map_err(|e| format!("Failed to delete server: {e}"))?;
        }

        info!(%server_id, "server deleted");
        Ok(())
    }

    /// List servers for a user (by their DB user_id).
    pub fn list_servers_for_user(&self, user_id: &str) -> Vec<ServerInfo> {
        self.servers
            .iter()
            .filter(|s| s.member_user_ids.contains(user_id))
            .map(|s| {
                let role = if s.owner_id == user_id {
                    Some("owner".to_string())
                } else {
                    Some("member".to_string())
                };
                ServerInfo {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    icon_url: s.icon_url.clone(),
                    member_count: s.member_user_ids.len(),
                    role,
                }
            })
            .collect()
    }

    /// List all servers (for system admin).
    pub fn list_all_servers(&self) -> Vec<ServerInfo> {
        self.servers
            .iter()
            .map(|s| ServerInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                icon_url: s.icon_url.clone(),
                member_count: s.member_user_ids.len(),
                role: None,
            })
            .collect()
    }

    /// Check if a user is the owner of a server.
    pub fn is_server_owner(&self, server_id: &str, user_id: &str) -> bool {
        self.servers
            .get(server_id)
            .map(|s| s.owner_id == user_id)
            .unwrap_or(false)
    }

    /// Join a server (persistent membership).
    pub async fn join_server(&self, user_id: &str, server_id: &str) -> Result<(), String> {
        if !self.servers.contains_key(server_id) {
            return Err(format!("No such server: {server_id}"));
        }

        if let Some(pool) = &self.db {
            crate::db::queries::servers::add_server_member(pool, server_id, user_id, "member")
                .await
                .map_err(|e| format!("Failed to join server: {e}"))?;
        }

        if let Some(mut server) = self.servers.get_mut(server_id) {
            server.member_user_ids.insert(user_id.to_string());
        }

        Ok(())
    }

    /// Leave a server (remove persistent membership).
    pub async fn leave_server(&self, user_id: &str, server_id: &str) -> Result<(), String> {
        if let Some(pool) = &self.db {
            crate::db::queries::servers::remove_server_member(pool, server_id, user_id)
                .await
                .map_err(|e| format!("Failed to leave server: {e}"))?;
        }

        if let Some(mut server) = self.servers.get_mut(server_id) {
            server.member_user_ids.remove(user_id);
        }

        Ok(())
    }

    /// Get the role of a user in a server.
    pub async fn get_server_role(&self, server_id: &str, user_id: &str) -> Option<ServerRole> {
        let Some(pool) = &self.db else {
            return None;
        };
        let member = crate::db::queries::servers::get_server_member(pool, server_id, user_id)
            .await
            .ok()
            .flatten()?;
        Some(ServerRole::parse(&member.role))
    }

    /// Look up server_id by server name (for IRC).
    pub fn find_server_by_name(&self, name: &str) -> Option<String> {
        let name_lower = name.to_lowercase();
        self.servers
            .iter()
            .find(|s| s.name.to_lowercase() == name_lower)
            .map(|s| s.id.clone())
    }

    /// Get a server's name by ID.
    pub fn get_server_name(&self, server_id: &str) -> Option<String> {
        self.servers.get(server_id).map(|s| s.name.clone())
    }

    // ── Channel management ──────────────────────────────────────────

    /// Create a channel within a server. Returns the channel ID.
    pub async fn create_channel_in_server(
        &self,
        server_id: &str,
        name: &str,
    ) -> Result<String, String> {
        let name = normalize_channel_name(name);
        validation::validate_channel_name(&name)?;

        if !self.servers.contains_key(server_id) {
            return Err(format!("No such server: {server_id}"));
        }

        if self
            .channel_name_index
            .contains_key(&(server_id.to_string(), name.clone()))
        {
            return Err(format!("Channel {name} already exists in this server"));
        }

        let channel_id = Uuid::new_v4().to_string();

        if let Some(pool) = &self.db {
            crate::db::queries::channels::ensure_channel(pool, &channel_id, server_id, &name)
                .await
                .map_err(|e| format!("Failed to create channel: {e}"))?;
        }

        let ch = ChannelState::new(channel_id.clone(), server_id.to_string(), name.clone());
        self.channel_name_index
            .insert((server_id.to_string(), name), channel_id.clone());
        if let Some(mut srv) = self.servers.get_mut(server_id) {
            srv.channel_ids.insert(channel_id.clone());
        }
        self.channels.insert(channel_id.clone(), ch);

        Ok(channel_id)
    }

    /// Delete a channel from a server.
    pub async fn delete_channel_in_server(
        &self,
        server_id: &str,
        channel_name: &str,
    ) -> Result<(), String> {
        let channel_name = normalize_channel_name(channel_name);
        let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

        if let Some(pool) = &self.db {
            crate::db::queries::channels::delete_channel(pool, &channel_id)
                .await
                .map_err(|e| format!("Failed to delete channel: {e}"))?;
        }

        self.channels.remove(&channel_id);
        self.channel_name_index
            .remove(&(server_id.to_string(), channel_name));
        if let Some(mut srv) = self.servers.get_mut(server_id) {
            srv.channel_ids.remove(&channel_id);
        }

        Ok(())
    }

    /// Join a channel within a server.
    pub fn join_channel(
        &self,
        session_id: SessionId,
        server_id: &str,
        channel_name: &str,
    ) -> Result<(), String> {
        let channel_name = normalize_channel_name(channel_name);
        validation::validate_channel_name(&channel_name)?;

        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        // Get or create channel
        let channel_id = if let Some(id) = self
            .channel_name_index
            .get(&(server_id.to_string(), channel_name.clone()))
        {
            id.clone()
        } else {
            // Create channel on-the-fly
            let new_id = Uuid::new_v4().to_string();
            let ch = ChannelState::new(new_id.clone(), server_id.to_string(), channel_name.clone());
            self.channels.insert(new_id.clone(), ch);
            self.channel_name_index.insert(
                (server_id.to_string(), channel_name.clone()),
                new_id.clone(),
            );
            if let Some(mut srv) = self.servers.get_mut(server_id) {
                srv.channel_ids.insert(new_id.clone());
            }

            // Persist channel to database
            if let Some(pool) = &self.db {
                let pool = pool.clone();
                let ch_id = new_id.clone();
                let srv_id = server_id.to_string();
                let ch_name = channel_name.clone();
                tokio::spawn(async move {
                    if let Err(e) = crate::db::queries::channels::ensure_channel(
                        &pool, &ch_id, &srv_id, &ch_name,
                    )
                    .await
                    {
                        error!(error = %e, "failed to persist channel");
                    }
                });
            }

            new_id
        };

        // Add session to channel
        if let Some(mut channel) = self.channels.get_mut(&channel_id) {
            channel.members.insert(session_id);
        }

        // Broadcast join event
        let join_event = ChatEvent::Join {
            nickname: session.nickname.clone(),
            server_id: server_id.to_string(),
            channel: channel_name.clone(),
            avatar_url: session.avatar_url.clone(),
        };
        self.broadcast_to_channel(&channel_id, &join_event, None);

        // Send current topic to the joiner
        if let Some(channel) = self.channels.get(&channel_id) {
            if !channel.topic.is_empty() {
                let _ = session.send(ChatEvent::Topic {
                    server_id: server_id.to_string(),
                    channel: channel_name.clone(),
                    topic: channel.topic.clone(),
                });
            }

            // Send member list to the joiner
            let members: Vec<MemberInfo> = channel
                .members
                .iter()
                .filter_map(|sid| {
                    self.sessions.get(sid).map(|s| MemberInfo {
                        nickname: s.nickname.clone(),
                        avatar_url: s.avatar_url.clone(),
                    })
                })
                .collect();

            let _ = session.send(ChatEvent::Names {
                server_id: server_id.to_string(),
                channel: channel_name.clone(),
                members,
            });
        }

        info!(nickname = %session.nickname, %server_id, %channel_name, "joined channel");
        Ok(())
    }

    /// Leave a channel.
    pub fn part_channel(
        &self,
        session_id: SessionId,
        server_id: &str,
        channel_name: &str,
        reason: Option<String>,
    ) -> Result<(), String> {
        let channel_name = normalize_channel_name(channel_name);
        let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let mut found = false;
        if let Some(mut channel) = self.channels.get_mut(&channel_id) {
            found = channel.members.remove(&session_id);
        }

        if !found {
            return Err(format!("Not in channel {channel_name}"));
        }

        let part_event = ChatEvent::Part {
            nickname: session.nickname.clone(),
            server_id: server_id.to_string(),
            channel: channel_name.clone(),
            reason,
        };
        let _ = session.send(part_event.clone());
        self.broadcast_to_channel(&channel_id, &part_event, Some(session_id));

        // Remove empty channels from memory (but not from DB)
        self.channels
            .remove_if(&channel_id, |_, ch| ch.members.is_empty());

        info!(nickname = %session.nickname, %server_id, %channel_name, "parted channel");
        Ok(())
    }

    /// Send a message to a channel or user (DM), with optional reply and attachments.
    pub fn send_message(
        &self,
        session_id: SessionId,
        server_id: &str,
        target: &str,
        content: &str,
        reply_to_id: Option<&str>,
        attachment_ids: Option<&[String]>,
    ) -> Result<(), String> {
        validation::validate_message(content)?;

        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        if !self.message_limiter.check(&session.nickname) {
            return Err("Rate limit exceeded. Please slow down.".into());
        }

        // Build reply info if replying to a message
        let reply_to: Option<ReplyInfo> = if let Some(ref_id) = reply_to_id {
            if let Some(pool) = &self.db {
                // Synchronous lookup via block_in_place — reply info is needed before broadcast
                let pool = pool.clone();
                let ref_id = ref_id.to_string();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        match crate::db::queries::messages::get_message_by_id(&pool, &ref_id).await
                        {
                            Ok(Some(row)) => Some(ReplyInfo {
                                id: row.id,
                                from: row.sender_nick,
                                content_preview: row.content.chars().take(100).collect::<String>(),
                            }),
                            _ => None,
                        }
                    })
                })
            } else {
                None
            }
        } else {
            None
        };

        // Look up attachment metadata if attachment_ids provided
        let attachments: Option<Vec<super::events::AttachmentInfo>> =
            if let Some(ids) = attachment_ids
                && !ids.is_empty()
            {
                if let Some(pool) = &self.db {
                    let pool = pool.clone();
                    let ids = ids.to_vec();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let infos =
                                crate::db::queries::attachments::get_attachments_by_ids(&pool, &ids)
                                    .await
                                    .unwrap_or_default();
                            if infos.is_empty() {
                                None
                            } else {
                                Some(
                                    infos
                                        .into_iter()
                                        .map(|a| super::events::AttachmentInfo {
                                            id: a.id.clone(),
                                            filename: a.original_filename,
                                            content_type: a.content_type,
                                            file_size: a.file_size,
                                            url: format!("/api/uploads/{}", a.id),
                                        })
                                        .collect(),
                                )
                            }
                        })
                    })
                } else {
                    None
                }
            } else {
                None
            };

        let msg_id = Uuid::new_v4();
        let event = ChatEvent::Message {
            id: msg_id,
            server_id: Some(server_id.to_string()),
            from: session.nickname.clone(),
            target: target.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            avatar_url: session.avatar_url.clone(),
            reply_to: reply_to.clone(),
            attachments: attachments.clone(),
        };

        if target.starts_with('#') {
            let channel_name = normalize_channel_name(target);
            let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

            let channel = self
                .channels
                .get(&channel_id)
                .ok_or(format!("No such channel: {channel_name}"))?;

            if !channel.members.contains(&session_id) {
                return Err(format!("You are not in channel {channel_name}"));
            }

            drop(channel);

            if let Some(pool) = &self.db {
                let pool = pool.clone();
                let id = msg_id.to_string();
                let srv = server_id.to_string();
                let ch = channel_id.clone();
                let sid = session_id.to_string();
                let nick = session.nickname.clone();
                let msg = content.to_string();
                let reply_id = reply_to_id.map(|s| s.to_string());
                let att_ids = attachment_ids.map(|ids| ids.to_vec());
                tokio::spawn(async move {
                    let params = crate::db::queries::messages::InsertMessageParams {
                        id: &id,
                        server_id: &srv,
                        channel_id: &ch,
                        sender_id: &sid,
                        sender_nick: &nick,
                        content: &msg,
                        reply_to_id: reply_id.as_deref(),
                    };
                    if let Err(e) =
                        crate::db::queries::messages::insert_message(&pool, &params).await
                    {
                        error!(error = %e, "failed to persist message");
                    }
                    // Link attachments to the message
                    if let Some(att_ids) = att_ids
                        && let Err(e) =
                            crate::db::queries::attachments::link_attachments_to_message(
                                &pool, &id, &att_ids, &sid,
                            )
                            .await
                    {
                        error!(error = %e, "failed to link attachments");
                    }
                });
            }

            self.broadcast_to_channel(&channel_id, &event, Some(session_id));

            // Async link embed unfurling — extract URLs and resolve OG metadata
            let urls = super::embeds::extract_urls(content);
            if !urls.is_empty()
                && let Some(pool) = &self.db
            {
                    let pool = pool.clone();
                    let client = self.http_client.clone();
                    let server_id_owned = server_id.to_string();
                    let channel_name_owned = channel_name.clone();
                    // Collect senders for channel members before spawning
                    let member_senders: Vec<mpsc::UnboundedSender<ChatEvent>> =
                        if let Some(channel) = self.channels.get(&channel_id) {
                            channel
                                .members
                                .iter()
                                .filter_map(|sid| {
                                    self.sessions.get(sid).map(|s| s.outbound.clone())
                                })
                                .collect()
                        } else {
                            vec![]
                        };
                    tokio::spawn(async move {
                        let mut embeds = Vec::new();
                        for url in urls {
                            // Check cache first
                            if let Ok(Some(cached)) =
                                crate::db::queries::embeds::get_cached_embed(&pool, &url).await
                            {
                                embeds.push(super::events::EmbedInfo {
                                    url: cached.url,
                                    title: cached.title,
                                    description: cached.description,
                                    image_url: cached.image_url,
                                    site_name: cached.site_name,
                                });
                                continue;
                            }
                            // Unfurl
                            if let Some(info) = super::embeds::unfurl_url(&client, &url).await {
                                let _ = crate::db::queries::embeds::upsert_embed(
                                    &pool,
                                    &info.url,
                                    info.title.as_deref(),
                                    info.description.as_deref(),
                                    info.image_url.as_deref(),
                                    info.site_name.as_deref(),
                                )
                                .await;
                                embeds.push(info);
                            }
                        }
                        if !embeds.is_empty() {
                            let embed_event = ChatEvent::MessageEmbed {
                                message_id: msg_id,
                                server_id: server_id_owned,
                                channel: channel_name_owned,
                                embeds,
                            };
                            for sender in &member_senders {
                                let _ = sender.send(embed_event.clone());
                            }
                        }
                    });
            }
        } else {
            // DM
            let target_session_id = self
                .nick_to_session
                .get(target)
                .ok_or(format!("No such user: {target}"))?;

            if let Some(pool) = &self.db {
                let pool = pool.clone();
                let id = msg_id.to_string();
                let sid = session_id.to_string();
                let nick = session.nickname.clone();
                let target_sid = target_session_id.value().to_string();
                let msg = content.to_string();
                tokio::spawn(async move {
                    if let Err(e) = crate::db::queries::messages::insert_dm(
                        &pool,
                        &id,
                        &sid,
                        &nick,
                        &target_sid,
                        &msg,
                    )
                    .await
                    {
                        error!(error = %e, "failed to persist DM");
                    }
                });
            }

            if let Some(target_session) = self.sessions.get(target_session_id.value()) {
                let _ = target_session.send(event);
            }
        }

        Ok(())
    }

    /// Set the topic for a channel.
    pub fn set_topic(
        &self,
        session_id: SessionId,
        server_id: &str,
        channel_name: &str,
        topic: String,
    ) -> Result<(), String> {
        validation::validate_topic(&topic)?;
        let channel_name = normalize_channel_name(channel_name);
        let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let mut channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(format!("No such channel: {channel_name}"))?;

        if !channel.members.contains(&session_id) {
            return Err(format!("You are not in channel {channel_name}"));
        }

        channel.topic.clone_from(&topic);
        channel.topic_set_by = Some(session.nickname.clone());
        channel.topic_set_at = Some(Utc::now());

        drop(channel);

        if let Some(pool) = &self.db {
            let pool = pool.clone();
            let ch = channel_id.clone();
            let t = topic.clone();
            let by = session.nickname.clone();
            tokio::spawn(async move {
                if let Err(e) = crate::db::queries::channels::set_topic(&pool, &ch, &t, &by).await {
                    error!(error = %e, "failed to persist topic");
                }
            });
        }

        let event = ChatEvent::TopicChange {
            server_id: server_id.to_string(),
            channel: channel_name,
            set_by: session.nickname.clone(),
            topic,
        };
        self.broadcast_to_channel(&channel_id, &event, None);

        Ok(())
    }

    /// Fetch message history for a channel, including edits, replies, and reactions.
    pub async fn fetch_history(
        &self,
        server_id: &str,
        channel_name: &str,
        before: Option<&str>,
        limit: i64,
    ) -> Result<(Vec<HistoryMessage>, bool), String> {
        let Some(pool) = &self.db else {
            return Ok((vec![], false));
        };

        let channel_name = normalize_channel_name(channel_name);
        let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

        let rows = crate::db::queries::messages::fetch_channel_history(
            pool,
            &channel_id,
            before,
            limit + 1,
        )
        .await
        .map_err(|e| format!("Failed to fetch history: {e}"))?;

        let has_more = rows.len() as i64 > limit;
        let rows: Vec<_> = rows.into_iter().take(limit as usize).collect();

        // Collect message IDs for batch reaction lookup
        let msg_ids: Vec<String> = rows.iter().map(|r| r.id.clone()).collect();

        // Fetch reactions for all messages in batch
        let reaction_rows =
            crate::db::queries::messages::get_reactions_for_messages(pool, &msg_ids)
                .await
                .unwrap_or_default();

        // Group reactions by message_id -> emoji -> user_ids
        let mut reaction_map: std::collections::HashMap<
            String,
            std::collections::HashMap<String, Vec<String>>,
        > = std::collections::HashMap::new();
        for r in &reaction_rows {
            reaction_map
                .entry(r.message_id.clone())
                .or_default()
                .entry(r.emoji.clone())
                .or_default()
                .push(r.user_id.clone());
        }

        // Collect reply_to_ids for batch lookup
        let reply_ids: Vec<String> = rows.iter().filter_map(|r| r.reply_to_id.clone()).collect();
        let mut reply_map: std::collections::HashMap<String, ReplyInfo> =
            std::collections::HashMap::new();
        if !reply_ids.is_empty() {
            for rid in &reply_ids {
                if let Ok(Some(parent)) =
                    crate::db::queries::messages::get_message_by_id(pool, rid).await
                {
                    reply_map.insert(
                        parent.id.clone(),
                        ReplyInfo {
                            id: parent.id,
                            from: parent.sender_nick,
                            content_preview: parent.content.chars().take(100).collect(),
                        },
                    );
                }
            }
        }

        // Fetch attachments for all messages in batch
        let attachment_rows =
            crate::db::queries::attachments::get_attachments_for_messages(pool, &msg_ids)
                .await
                .unwrap_or_default();

        // Group attachments by message_id
        let mut attachment_map: std::collections::HashMap<String, Vec<super::events::AttachmentInfo>> =
            std::collections::HashMap::new();
        for a in &attachment_rows {
            if let Some(ref mid) = a.message_id {
                attachment_map.entry(mid.clone()).or_default().push(
                    super::events::AttachmentInfo {
                        id: a.id.clone(),
                        filename: a.original_filename.clone(),
                        content_type: a.content_type.clone(),
                        file_size: a.file_size,
                        url: format!("/api/uploads/{}", a.id),
                    },
                );
            }
        }

        let messages: Vec<HistoryMessage> = rows
            .into_iter()
            .map(|row| {
                let reactions = reaction_map.get(&row.id).map(|emoji_map| {
                    emoji_map
                        .iter()
                        .map(|(emoji, user_ids)| ReactionGroup {
                            emoji: emoji.clone(),
                            count: user_ids.len(),
                            user_ids: user_ids.clone(),
                        })
                        .collect()
                });
                let reply_to = row
                    .reply_to_id
                    .as_ref()
                    .and_then(|rid| reply_map.get(rid).cloned());
                let edited_at = row.edited_at.as_ref().and_then(|s| s.parse().ok());
                let attachments = attachment_map.remove(&row.id);

                HistoryMessage {
                    id: row.id.parse().unwrap_or_default(),
                    from: row.sender_nick,
                    content: row.content,
                    timestamp: row.created_at.parse().unwrap_or_else(|_| Utc::now()),
                    edited_at,
                    reply_to,
                    reactions,
                    attachments,
                    embeds: None,
                }
            })
            .collect();

        Ok((messages, has_more))
    }

    /// List all channels in a server.
    pub fn list_channels(&self, server_id: &str) -> Vec<ChannelInfo> {
        self.channels
            .iter()
            .filter(|ch| ch.server_id == server_id)
            .map(|entry| ChannelInfo {
                id: entry.id.clone(),
                server_id: entry.server_id.clone(),
                name: entry.name.clone(),
                topic: entry.topic.clone(),
                member_count: entry.member_count(),
            })
            .collect()
    }

    /// Get members of a channel.
    pub fn get_members(
        &self,
        server_id: &str,
        channel_name: &str,
    ) -> Result<Vec<MemberInfo>, String> {
        let channel_name = normalize_channel_name(channel_name);
        let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

        let channel = self
            .channels
            .get(&channel_id)
            .ok_or(format!("No such channel: {channel_name}"))?;

        Ok(channel
            .members
            .iter()
            .filter_map(|sid| {
                self.sessions.get(sid).map(|s| MemberInfo {
                    nickname: s.nickname.clone(),
                    avatar_url: s.avatar_url.clone(),
                })
            })
            .collect())
    }

    // ── Message editing & deletion ─────────────────────────────────

    /// Edit a message's content. Only the sender or a moderator+ can edit.
    pub async fn edit_message(
        &self,
        session_id: SessionId,
        message_id: &str,
        new_content: &str,
    ) -> Result<(), String> {
        validation::validate_message(new_content)?;

        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let pool = self.db.as_ref().ok_or("No database configured")?;

        let msg = crate::db::queries::messages::get_message_by_id(pool, message_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or("Message not found")?;

        // Only the sender can edit their own messages
        let sender_id = session.user_id.as_deref().unwrap_or("");
        if msg.sender_id != sender_id && msg.sender_nick != session.nickname {
            return Err("You can only edit your own messages".into());
        }

        crate::db::queries::messages::update_message_content(pool, message_id, new_content)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

        let server_id = msg.server_id.ok_or("Message has no server")?;
        let channel_id = msg.channel_id.ok_or("Message has no channel")?;

        // Find the channel name for the event
        let channel_name = self
            .channels
            .get(&channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_default();

        let event = ChatEvent::MessageEdit {
            id: message_id.parse().unwrap_or_default(),
            server_id: server_id.clone(),
            channel: channel_name,
            content: new_content.to_string(),
            edited_at: Utc::now(),
        };

        // Broadcast to the channel (including sender)
        self.broadcast_to_channel(&channel_id, &event, None);

        Ok(())
    }

    /// Delete a message (soft delete). Sender can delete own, moderator+ can delete any.
    pub async fn delete_message(
        &self,
        session_id: SessionId,
        message_id: &str,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let pool = self.db.as_ref().ok_or("No database configured")?;

        let msg = crate::db::queries::messages::get_message_by_id(pool, message_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or("Message not found")?;

        let sender_id = session.user_id.as_deref().unwrap_or("");
        let is_sender = msg.sender_id == sender_id || msg.sender_nick == session.nickname;

        if !is_sender {
            // Check if user has moderator+ role
            let server_id = msg.server_id.as_deref().ok_or("Message has no server")?;
            if let Some(uid) = &session.user_id {
                let role = self.get_server_role(server_id, uid).await;
                if !matches!(
                    role,
                    Some(ServerRole::Owner) | Some(ServerRole::Admin) | Some(ServerRole::Moderator)
                ) {
                    return Err("You can only delete your own messages".into());
                }
            } else {
                return Err("You can only delete your own messages".into());
            }
        }

        crate::db::queries::messages::soft_delete_message(pool, message_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

        let server_id = msg.server_id.ok_or("Message has no server")?;
        let channel_id = msg.channel_id.ok_or("Message has no channel")?;

        let channel_name = self
            .channels
            .get(&channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_default();

        let event = ChatEvent::MessageDelete {
            id: message_id.parse().unwrap_or_default(),
            server_id,
            channel: channel_name,
        };

        self.broadcast_to_channel(&channel_id, &event, None);

        Ok(())
    }

    // ── Reactions ────────────────────────────────────────────────────

    /// Add a reaction to a message.
    pub async fn add_reaction(
        &self,
        session_id: SessionId,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let pool = self.db.as_ref().ok_or("No database configured")?;

        let msg = crate::db::queries::messages::get_message_by_id(pool, message_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or("Message not found")?;

        let user_id = session.user_id.as_deref().unwrap_or(&session.nickname);

        crate::db::queries::messages::add_reaction(pool, message_id, user_id, emoji)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

        let server_id = msg.server_id.ok_or("Message has no server")?;
        let channel_id = msg.channel_id.ok_or("Message has no channel")?;

        let channel_name = self
            .channels
            .get(&channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_default();

        let event = ChatEvent::ReactionAdd {
            message_id: message_id.parse().unwrap_or_default(),
            server_id,
            channel: channel_name,
            user_id: user_id.to_string(),
            nickname: session.nickname.clone(),
            emoji: emoji.to_string(),
        };

        self.broadcast_to_channel(&channel_id, &event, None);

        Ok(())
    }

    /// Remove a reaction from a message.
    pub async fn remove_reaction(
        &self,
        session_id: SessionId,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let pool = self.db.as_ref().ok_or("No database configured")?;

        let msg = crate::db::queries::messages::get_message_by_id(pool, message_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or("Message not found")?;

        let user_id = session.user_id.as_deref().unwrap_or(&session.nickname);

        crate::db::queries::messages::remove_reaction(pool, message_id, user_id, emoji)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

        let server_id = msg.server_id.ok_or("Message has no server")?;
        let channel_id = msg.channel_id.ok_or("Message has no channel")?;

        let channel_name = self
            .channels
            .get(&channel_id)
            .map(|ch| ch.name.clone())
            .unwrap_or_default();

        let event = ChatEvent::ReactionRemove {
            message_id: message_id.parse().unwrap_or_default(),
            server_id,
            channel: channel_name,
            user_id: user_id.to_string(),
            nickname: session.nickname.clone(),
            emoji: emoji.to_string(),
        };

        self.broadcast_to_channel(&channel_id, &event, None);

        Ok(())
    }

    // ── Typing indicators ────────────────────────────────────────────

    /// Broadcast a typing indicator to a channel.
    pub fn send_typing(
        &self,
        session_id: SessionId,
        server_id: &str,
        channel_name: &str,
    ) -> Result<(), String> {
        let channel_name = normalize_channel_name(channel_name);

        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

        let event = ChatEvent::TypingStart {
            server_id: server_id.to_string(),
            channel: channel_name,
            nickname: session.nickname.clone(),
        };

        self.broadcast_to_channel(&channel_id, &event, Some(session_id));

        Ok(())
    }

    // ── Read state ────────────────────────────────────────────────────

    /// Mark a channel as read for a user, up to a specific message ID.
    pub async fn mark_read(
        &self,
        session_id: SessionId,
        server_id: &str,
        channel_name: &str,
        message_id: &str,
    ) -> Result<(), String> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let user_id = session.user_id.as_deref().ok_or("AUTH_REQUIRED")?;
        let pool = self.db.as_ref().ok_or("No database configured")?;

        let channel_name = normalize_channel_name(channel_name);
        let channel_id = self.resolve_channel_id(server_id, &channel_name)?;

        crate::db::queries::messages::mark_channel_read(pool, user_id, &channel_id, message_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

        Ok(())
    }

    /// Get unread counts for all channels in a server for a user.
    pub async fn get_unread_counts(
        &self,
        session_id: SessionId,
        server_id: &str,
    ) -> Result<Vec<super::events::UnreadCount>, String> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or("Session not found")?
            .clone();

        let user_id = session.user_id.as_deref().ok_or("AUTH_REQUIRED")?;
        let pool = self.db.as_ref().ok_or("No database configured")?;

        let rows = crate::db::queries::messages::get_unread_counts(pool, user_id, server_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?;

        // Map channel_id -> channel_name
        Ok(rows
            .into_iter()
            .filter_map(|r| {
                let name = self
                    .channels
                    .get(&r.channel_id)
                    .map(|ch| ch.name.clone())?;
                Some(super::events::UnreadCount {
                    channel_name: name,
                    count: r.unread_count,
                })
            })
            .collect())
    }

    // ── Utility ─────────────────────────────────────────────────────

    /// Get a reference to the database pool (if configured).
    pub fn db(&self) -> Option<&SqlitePool> {
        self.db.as_ref()
    }

    /// Check if a nickname is available.
    pub fn is_nick_available(&self, nickname: &str) -> bool {
        !self.nick_to_session.contains_key(nickname)
    }

    /// Get a session by ID.
    pub fn get_session(&self, session_id: SessionId) -> Option<Arc<UserSession>> {
        self.sessions.get(&session_id).map(|s| s.clone())
    }

    /// Resolve a channel name within a server to its channel ID.
    pub fn resolve_channel_id(
        &self,
        server_id: &str,
        channel_name: &str,
    ) -> Result<String, String> {
        self.channel_name_index
            .get(&(server_id.to_string(), channel_name.to_string()))
            .map(|r| r.clone())
            .ok_or(format!("No such channel: {channel_name}"))
    }

    /// Broadcast an event to all members of a channel, optionally excluding one session.
    fn broadcast_to_channel(
        &self,
        channel_id: &str,
        event: &ChatEvent,
        exclude: Option<SessionId>,
    ) {
        let Some(channel) = self.channels.get(channel_id) else {
            return;
        };

        for member_id in &channel.members {
            if Some(*member_id) == exclude {
                continue;
            }
            if let Some(session) = self.sessions.get(member_id)
                && !session.send(event.clone())
            {
                warn!(%member_id, "failed to send event to session (channel closed)");
            }
        }
    }
}

/// Ensure channel names are lowercase and start with #.
fn normalize_channel_name(name: &str) -> String {
    let name = name.to_lowercase();
    if name.starts_with('#') {
        name
    } else {
        format!("#{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_channel_name() {
        assert_eq!(normalize_channel_name("#General"), "#general");
        assert_eq!(normalize_channel_name("general"), "#general");
        assert_eq!(normalize_channel_name("#rust"), "#rust");
    }

    /// Helper: create engine with a default server in memory (no DB).
    fn setup_engine() -> ChatEngine {
        let engine = ChatEngine::new(None);
        let state = ServerState::new(
            DEFAULT_SERVER_ID.to_string(),
            "Concord".to_string(),
            "system".to_string(),
            None,
        );
        engine.servers.insert(DEFAULT_SERVER_ID.to_string(), state);
        engine
    }

    #[tokio::test]
    async fn test_connect_and_disconnect() {
        let engine = setup_engine();

        let (session_id, _rx) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();
        assert!(!engine.is_nick_available("alice"));

        engine.disconnect(session_id);
        assert!(engine.is_nick_available("alice"));
    }

    #[tokio::test]
    async fn test_duplicate_nick_replaces_old_session() {
        let engine = setup_engine();

        let (sid1, _rx1) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();
        let (sid2, _rx2) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();

        assert!(engine.get_session(sid1).is_none());
        assert!(engine.get_session(sid2).is_some());
    }

    #[tokio::test]
    async fn test_join_and_message() {
        let engine = setup_engine();

        let (sid1, mut rx1) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();
        let (sid2, mut rx2) = engine
            .connect(None, "bob".into(), Protocol::WebSocket, None)
            .unwrap();

        engine
            .join_channel(sid1, DEFAULT_SERVER_ID, "#general")
            .unwrap();
        engine
            .join_channel(sid2, DEFAULT_SERVER_ID, "#general")
            .unwrap();

        while rx1.try_recv().is_ok() {}
        while rx2.try_recv().is_ok() {}

        engine
            .send_message(
                sid1,
                DEFAULT_SERVER_ID,
                "#general",
                "Hello from Alice!",
                None,
                None,
            )
            .unwrap();

        let event = rx2.try_recv().unwrap();
        match event {
            ChatEvent::Message { from, content, .. } => {
                assert_eq!(from, "alice");
                assert_eq!(content, "Hello from Alice!");
            }
            _ => panic!("Expected Message event, got {:?}", event),
        }

        assert!(rx1.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_part_channel() {
        let engine = setup_engine();

        let (sid1, mut rx1) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();
        let (sid2, _rx2) = engine
            .connect(None, "bob".into(), Protocol::WebSocket, None)
            .unwrap();

        engine
            .join_channel(sid1, DEFAULT_SERVER_ID, "#general")
            .unwrap();
        engine
            .join_channel(sid2, DEFAULT_SERVER_ID, "#general")
            .unwrap();

        while rx1.try_recv().is_ok() {}

        engine
            .part_channel(sid2, DEFAULT_SERVER_ID, "#general", None)
            .unwrap();

        let event = rx1.try_recv().unwrap();
        match event {
            ChatEvent::Part { nickname, .. } => assert_eq!(nickname, "bob"),
            _ => panic!("Expected Part event, got {:?}", event),
        }
    }

    #[tokio::test]
    async fn test_set_topic() {
        let engine = setup_engine();

        let (sid, mut rx) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();
        engine
            .join_channel(sid, DEFAULT_SERVER_ID, "#general")
            .unwrap();
        while rx.try_recv().is_ok() {}

        engine
            .set_topic(
                sid,
                DEFAULT_SERVER_ID,
                "#general",
                "Welcome to Concord!".into(),
            )
            .unwrap();

        let event = rx.try_recv().unwrap();
        match event {
            ChatEvent::TopicChange { topic, .. } => {
                assert_eq!(topic, "Welcome to Concord!");
            }
            _ => panic!("Expected TopicChange event, got {:?}", event),
        }
    }

    #[tokio::test]
    async fn test_dm() {
        let engine = setup_engine();

        let (sid1, _rx1) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();
        let (_sid2, mut rx2) = engine
            .connect(None, "bob".into(), Protocol::WebSocket, None)
            .unwrap();

        engine
            .send_message(sid1, DEFAULT_SERVER_ID, "bob", "Hey Bob!", None, None)
            .unwrap();

        let event = rx2.try_recv().unwrap();
        match event {
            ChatEvent::Message {
                from,
                target,
                content,
                ..
            } => {
                assert_eq!(from, "alice");
                assert_eq!(target, "bob");
                assert_eq!(content, "Hey Bob!");
            }
            _ => panic!("Expected Message event, got {:?}", event),
        }
    }

    #[test]
    fn test_list_channels() {
        let engine = setup_engine();

        let (sid, _rx) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();
        engine
            .join_channel(sid, DEFAULT_SERVER_ID, "#general")
            .unwrap();
        engine
            .join_channel(sid, DEFAULT_SERVER_ID, "#rust")
            .unwrap();

        let channels = engine.list_channels(DEFAULT_SERVER_ID);
        assert_eq!(channels.len(), 2);

        let names: Vec<&str> = channels.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"#general"));
        assert!(names.contains(&"#rust"));
    }

    #[tokio::test]
    async fn test_create_server() {
        let engine = setup_engine();

        let server_id = engine
            .create_server("Test Server".into(), "user1".into(), None)
            .await
            .unwrap();

        assert!(engine.servers.contains_key(&server_id));
        let channels = engine.list_channels(&server_id);
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].name, "#general");
    }

    #[tokio::test]
    async fn test_server_isolation() {
        let engine = setup_engine();

        let server_a = engine
            .create_server("Server A".into(), "user1".into(), None)
            .await
            .unwrap();
        let server_b = engine
            .create_server("Server B".into(), "user1".into(), None)
            .await
            .unwrap();

        let (sid, mut rx) = engine
            .connect(None, "alice".into(), Protocol::WebSocket, None)
            .unwrap();

        engine.join_channel(sid, &server_a, "#general").unwrap();
        while rx.try_recv().is_ok() {}

        let (sid2, _rx2) = engine
            .connect(None, "bob".into(), Protocol::WebSocket, None)
            .unwrap();
        engine.join_channel(sid2, &server_b, "#general").unwrap();

        // Alice is not in server_b's #general — should fail
        let result = engine.send_message(sid, &server_b, "#general", "Hello", None, None);
        assert!(result.is_err());
    }
}
