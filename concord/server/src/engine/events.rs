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

    /// List of roles in a server.
    RoleList {
        server_id: String,
        roles: Vec<RoleInfo>,
    },

    /// A role was created or updated.
    RoleUpdate {
        server_id: String,
        role: RoleInfo,
    },

    /// A role was deleted.
    RoleDelete {
        server_id: String,
        role_id: String,
    },

    /// A member's role assignments changed.
    MemberRoleUpdate {
        server_id: String,
        user_id: String,
        role_ids: Vec<String>,
    },

    /// List of categories in a server.
    CategoryList {
        server_id: String,
        categories: Vec<CategoryInfo>,
    },

    /// A category was created or updated.
    CategoryUpdate {
        server_id: String,
        category: CategoryInfo,
    },

    /// A category was deleted.
    CategoryDelete {
        server_id: String,
        category_id: String,
    },

    /// Channel positions/categories were reordered.
    ChannelReorder {
        server_id: String,
        channels: Vec<ChannelPositionInfo>,
    },

    /// Presence update for a user (broadcast to shared server members).
    PresenceUpdate {
        server_id: String,
        presence: PresenceInfo,
    },

    /// Bulk presence list for a server (sent on connect/join).
    PresenceList {
        server_id: String,
        presences: Vec<PresenceInfo>,
    },

    /// A user's profile was fetched or updated.
    UserProfile {
        profile: UserProfileInfo,
    },

    /// A member's server nickname changed.
    ServerNicknameUpdate {
        server_id: String,
        user_id: String,
        nickname: Option<String>,
    },

    /// Notification settings response.
    NotificationSettings {
        server_id: String,
        settings: Vec<NotificationSettingInfo>,
    },

    /// Search results response.
    SearchResults {
        server_id: String,
        query: String,
        results: Vec<SearchResultMessage>,
        total_count: i64,
        offset: i64,
    },

    /// Message was pinned in a channel.
    MessagePin {
        server_id: String,
        channel: String,
        pin: PinnedMessageInfo,
    },

    /// Message was unpinned from a channel.
    MessageUnpin {
        server_id: String,
        channel: String,
        message_id: String,
    },

    /// List of all pinned messages in a channel.
    PinnedMessages {
        server_id: String,
        channel: String,
        pins: Vec<PinnedMessageInfo>,
    },

    /// A thread was created.
    ThreadCreate {
        server_id: String,
        parent_channel: String,
        thread: ThreadInfo,
    },

    /// A thread was archived or unarchived.
    ThreadUpdate {
        server_id: String,
        thread: ThreadInfo,
    },

    /// List of threads for a channel.
    ThreadList {
        server_id: String,
        channel: String,
        threads: Vec<ThreadInfo>,
    },

    /// Forum tags list.
    ForumTagList {
        server_id: String,
        channel: String,
        tags: Vec<ForumTagInfo>,
    },

    /// Forum tag created/updated.
    ForumTagUpdate {
        server_id: String,
        channel: String,
        tag: ForumTagInfo,
    },

    /// Forum tag deleted.
    ForumTagDelete {
        server_id: String,
        channel: String,
        tag_id: String,
    },

    /// Bookmarks list response.
    BookmarkList {
        bookmarks: Vec<BookmarkInfo>,
    },

    /// Bookmark added.
    BookmarkAdd {
        bookmark: BookmarkInfo,
    },

    /// Bookmark removed.
    BookmarkRemove {
        message_id: String,
    },

    /// A member was kicked from the server.
    MemberKick {
        server_id: String,
        user_id: String,
        kicked_by: String,
        reason: Option<String>,
    },

    /// A member was banned from the server.
    MemberBan {
        server_id: String,
        user_id: String,
        banned_by: String,
        reason: Option<String>,
    },

    /// A ban was removed from the server.
    MemberUnban {
        server_id: String,
        user_id: String,
    },

    /// A member was timed out.
    MemberTimeout {
        server_id: String,
        user_id: String,
        timeout_until: Option<String>,
    },

    /// Channel slow mode was updated.
    SlowModeUpdate {
        server_id: String,
        channel: String,
        seconds: i32,
    },

    /// Channel NSFW flag was updated.
    NsfwUpdate {
        server_id: String,
        channel: String,
        is_nsfw: bool,
    },

    /// Bulk messages were deleted.
    BulkMessageDelete {
        server_id: String,
        channel: String,
        message_ids: Vec<String>,
    },

    /// Audit log entries response.
    AuditLogEntries {
        server_id: String,
        entries: Vec<AuditLogEntry>,
    },

    /// Ban list response.
    BanList {
        server_id: String,
        bans: Vec<BanInfo>,
    },

    /// AutoMod rules list response.
    AutomodRuleList {
        server_id: String,
        rules: Vec<AutomodRuleInfo>,
    },

    /// AutoMod rule created/updated.
    AutomodRuleUpdate {
        server_id: String,
        rule: AutomodRuleInfo,
    },

    /// AutoMod rule deleted.
    AutomodRuleDelete {
        server_id: String,
        rule_id: String,
    },

    // ── Phase 7: Community & Discovery ──

    /// Invite list response.
    InviteList {
        server_id: String,
        invites: Vec<InviteInfo>,
    },

    /// Invite created.
    InviteCreate {
        server_id: String,
        invite: InviteInfo,
    },

    /// Invite deleted.
    InviteDelete {
        server_id: String,
        invite_id: String,
    },

    /// Server events list.
    EventList {
        server_id: String,
        events: Vec<EventInfo>,
    },

    /// Event created or updated.
    EventUpdate {
        server_id: String,
        event: EventInfo,
    },

    /// Event deleted.
    EventDelete {
        server_id: String,
        event_id: String,
    },

    /// Event RSVP list.
    EventRsvpList {
        event_id: String,
        rsvps: Vec<RsvpInfo>,
    },

    /// Server community settings.
    ServerCommunity {
        community: ServerCommunityInfo,
    },

    /// Discoverable servers list.
    DiscoverServers {
        servers: Vec<ServerCommunityInfo>,
    },

    /// Channel follows list.
    ChannelFollowList {
        channel_id: String,
        follows: Vec<ChannelFollowInfo>,
    },

    /// Channel follow created.
    ChannelFollowCreate {
        follow: ChannelFollowInfo,
    },

    /// Channel follow deleted.
    ChannelFollowDelete {
        follow_id: String,
    },

    /// Server templates list.
    TemplateList {
        server_id: String,
        templates: Vec<TemplateInfo>,
    },

    /// Template created/updated.
    TemplateUpdate {
        server_id: String,
        template: TemplateInfo,
    },

    /// Template deleted.
    TemplateDelete {
        server_id: String,
        template_id: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    pub position: i32,
    pub is_private: bool,
    pub channel_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_parent_message_id: Option<String>,
    pub archived: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberInfo {
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_emoji: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
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

/// Role metadata sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub id: String,
    pub server_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub position: i32,
    pub permissions: i64,
    pub is_default: bool,
}

/// Channel category metadata sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub position: i32,
}

/// Minimal channel position info for reorder events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPositionInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
    pub position: i32,
}

/// User presence info sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub user_id: String,
    pub nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_emoji: Option<String>,
}

/// Full user profile info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileInfo {
    pub user_id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    pub created_at: String,
}

/// Notification setting info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettingInfo {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    pub level: String,
    pub suppress_everyone: bool,
    pub suppress_roles: bool,
    pub muted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_until: Option<String>,
}

/// A search result message (same as HistoryMessage but with channel info).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultMessage {
    pub id: MessageId,
    pub from: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub channel_id: String,
    pub channel_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<DateTime<Utc>>,
}

/// Info about a pinned message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedMessageInfo {
    pub id: String,
    pub message_id: String,
    pub channel_id: String,
    pub pinned_by: String,
    pub pinned_at: String,
    // Denormalized message content for display
    pub from: String,
    pub content: String,
    pub timestamp: String,
}

/// Info about a thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub id: String,
    pub name: String,
    pub channel_type: String, // "public_thread" | "private_thread"
    pub parent_message_id: Option<String>,
    pub archived: bool,
    pub auto_archive_minutes: i32,
    pub message_count: i64,
    pub created_at: String,
}

/// Forum tag info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumTagInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    pub moderated: bool,
    pub position: i32,
}

/// Bookmark info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkInfo {
    pub id: String,
    pub message_id: String,
    pub channel_id: String,
    pub from: String,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub created_at: String,
}

/// Audit log entry sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub actor_id: String,
    pub action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes: Option<String>,
    pub created_at: String,
}

/// Ban info sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanInfo {
    pub id: String,
    pub user_id: String,
    pub banned_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub created_at: String,
}

/// AutoMod rule info sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomodRuleInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub rule_type: String,
    pub config: String,
    pub action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_duration_seconds: Option<i32>,
}

/// Server invite info sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteInfo {
    pub id: String,
    pub code: String,
    pub server_id: String,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i32>,
    pub use_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    pub created_at: String,
}

/// Scheduled event info sent to clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub id: String,
    pub server_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    pub start_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub created_by: String,
    pub status: String,
    pub interested_count: i64,
    pub created_at: String,
}

/// RSVP info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsvpInfo {
    pub user_id: String,
    pub status: String,
}

/// Channel follow info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelFollowInfo {
    pub id: String,
    pub source_channel_id: String,
    pub target_channel_id: String,
    pub created_by: String,
}

/// Server template info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub server_id: String,
    pub created_by: String,
    pub use_count: i32,
    pub created_at: String,
}

/// Server community/discovery info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCommunityInfo {
    pub server_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_discoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub welcome_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}
