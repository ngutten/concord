use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a message.
pub type MessageId = Uuid;

/// Unique identifier for a connected session (one per connection, not per user).
pub type SessionId = Uuid;

/// Protocol-agnostic event that flows through the chat engine.
/// Both IRC and WebSocket adapters produce and consume these.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    /// A message sent to a channel or as a DM.
    Message {
        id: MessageId,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_id: Option<String>,
        from: String,
        target: String,
        content: String,
        timestamp: DateTime<Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        avatar_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reply_to: Option<ReplyInfo>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attachments: Option<Vec<AttachmentInfo>>,
    },

    /// A message was edited.
    MessageEdit {
        id: MessageId,
        server_id: String,
        channel: String,
        content: String,
        edited_at: DateTime<Utc>,
    },

    /// A message was deleted.
    MessageDelete {
        id: MessageId,
        server_id: String,
        channel: String,
    },

    /// A reaction was added to a message.
    ReactionAdd {
        message_id: MessageId,
        server_id: String,
        channel: String,
        user_id: String,
        nickname: String,
        emoji: String,
    },

    /// A reaction was removed from a message.
    ReactionRemove {
        message_id: MessageId,
        server_id: String,
        channel: String,
        user_id: String,
        nickname: String,
        emoji: String,
    },

    /// A user started typing in a channel.
    TypingStart {
        server_id: String,
        channel: String,
        nickname: String,
    },

    /// User joined a channel.
    Join {
        nickname: String,
        server_id: String,
        channel: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        avatar_url: Option<String>,
    },

    /// User left a channel.
    Part {
        nickname: String,
        server_id: String,
        channel: String,
        reason: Option<String>,
    },

    /// User disconnected from the server.
    Quit {
        nickname: String,
        reason: Option<String>,
    },

    /// Channel topic changed.
    TopicChange {
        server_id: String,
        channel: String,
        set_by: String,
        topic: String,
    },

    /// User changed their nickname.
    NickChange { old_nick: String, new_nick: String },

    /// Server notice directed at a specific session.
    ServerNotice { message: String },

    /// Channel member list (sent on join).
    Names {
        server_id: String,
        channel: String,
        members: Vec<MemberInfo>,
    },

    /// Current topic of a channel (sent on join).
    Topic {
        server_id: String,
        channel: String,
        topic: String,
    },

    /// Response to a channel list request.
    ChannelList {
        server_id: String,
        channels: Vec<ChannelInfo>,
    },

    /// Message history response.
    History {
        server_id: String,
        channel: String,
        messages: Vec<HistoryMessage>,
        has_more: bool,
    },

    /// List of servers the user belongs to.
    ServerList { servers: Vec<ServerInfo> },

    /// Unread message counts for channels in a server.
    UnreadCounts {
        server_id: String,
        counts: Vec<UnreadCount>,
    },

    /// Link embed previews were resolved for a message.
    MessageEmbed {
        message_id: MessageId,
        server_id: String,
        channel: String,
        embeds: Vec<EmbedInfo>,
    },

    /// Error from the server.
    Error { code: String, message: String },
}

/// Info about a replied-to message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyInfo {
    pub id: String,
    pub from: String,
    pub content_preview: String,
}

/// Grouped reactions for a message in history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionGroup {
    pub emoji: String,
    pub count: usize,
    pub user_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub member_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub topic: String,
    pub member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInfo {
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub id: MessageId,
    pub from: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<ReplyInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<ReactionGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<AttachmentInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<EmbedInfo>>,
}

/// Metadata for a file attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub file_size: i64,
    pub url: String,
}

/// Open Graph link embed preview metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedInfo {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreadCount {
    pub channel_name: String,
    pub count: i64,
}
