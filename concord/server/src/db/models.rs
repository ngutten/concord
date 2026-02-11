use serde::{Deserialize, Serialize};

/// A stored server (guild) from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerRow {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: Option<String>,
    pub is_discoverable: i32,
    pub welcome_message: Option<String>,
    pub rules_text: Option<String>,
    pub category: Option<String>,
}

/// A server membership record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerMemberRow {
    pub server_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

/// A stored message from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageRow {
    pub id: String,
    pub server_id: Option<String>,
    pub channel_id: Option<String>,
    pub sender_id: String,
    pub sender_nick: String,
    pub content: String,
    pub created_at: String,
    pub target_user_id: Option<String>,
    pub edited_at: Option<String>,
    pub deleted_at: Option<String>,
    pub reply_to_id: Option<String>,
}

/// A stored channel from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelRow {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub topic: String,
    pub topic_set_by: Option<String>,
    pub topic_set_at: Option<String>,
    pub created_at: String,
    pub is_default: i32,
    pub category_id: Option<String>,
    pub position: i32,
    pub is_private: i32,
    pub channel_type: String,
    pub thread_parent_message_id: Option<String>,
    pub thread_auto_archive_minutes: i32,
    pub archived: i32,
    pub slowmode_seconds: i32,
    pub is_nsfw: i32,
    pub is_announcement: i32,
}

/// A channel membership record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelMemberRow {
    pub channel_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

/// A custom role within a server.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleRow {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub color: Option<String>,
    pub icon_url: Option<String>,
    pub position: i32,
    pub permissions: i64,
    pub is_default: i32,
    pub created_at: String,
}

/// A user-to-role assignment.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRoleRow {
    pub server_id: String,
    pub user_id: String,
    pub role_id: String,
    pub assigned_at: String,
}

/// A channel category (grouping of channels).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelCategoryRow {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub position: i32,
    pub created_at: String,
}

/// A channel permission override.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelPermissionOverrideRow {
    pub id: String,
    pub channel_id: String,
    pub target_type: String,
    pub target_id: String,
    pub allow_bits: i64,
    pub deny_bits: i64,
    pub created_at: String,
}

/// User presence and custom status.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPresenceRow {
    pub user_id: String,
    pub status: String,
    pub custom_status: Option<String>,
    pub status_emoji: Option<String>,
    pub last_seen_at: String,
    pub updated_at: String,
}

/// User profile (bio, pronouns, banner).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfileRow {
    pub user_id: String,
    pub bio: Option<String>,
    pub pronouns: Option<String>,
    pub banner_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for upserting a notification setting (avoids too-many-arguments).
pub struct UpsertNotificationParams<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub server_id: Option<&'a str>,
    pub channel_id: Option<&'a str>,
    pub level: &'a str,
    pub suppress_everyone: bool,
    pub suppress_roles: bool,
    pub muted: bool,
    pub mute_until: Option<&'a str>,
}

/// Per-server/channel notification settings.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationSettingRow {
    pub id: String,
    pub user_id: String,
    pub server_id: Option<String>,
    pub channel_id: Option<String>,
    pub level: String,
    pub suppress_everyone: i32,
    pub suppress_roles: i32,
    pub muted: i32,
    pub mute_until: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A pinned message in a channel.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PinnedMessageRow {
    pub id: String,
    pub channel_id: String,
    pub message_id: String,
    pub pinned_by: String,
    pub pinned_at: String,
}

/// A forum tag for categorizing threads.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ForumTagRow {
    pub id: String,
    pub channel_id: String,
    pub name: String,
    pub emoji: Option<String>,
    pub moderated: i32,
    pub position: i32,
    pub created_at: String,
}

/// A thread-to-tag association.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ThreadTagRow {
    pub thread_id: String,
    pub tag_id: String,
}

/// A personal bookmark on a message.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookmarkRow {
    pub id: String,
    pub user_id: String,
    pub message_id: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// A server ban record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BanRow {
    pub id: String,
    pub server_id: String,
    pub user_id: String,
    pub banned_by: String,
    pub reason: Option<String>,
    pub delete_message_days: i32,
    pub created_at: String,
}

/// An audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: String,
    pub server_id: String,
    pub actor_id: String,
    pub action_type: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub reason: Option<String>,
    pub changes: Option<String>,
    pub created_at: String,
}

/// An automod rule.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AutomodRuleRow {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub enabled: i32,
    pub rule_type: String,
    pub config: String,
    pub action_type: String,
    pub timeout_duration_seconds: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating an audit log entry (avoids too-many-arguments).
pub struct CreateAuditLogParams<'a> {
    pub id: &'a str,
    pub server_id: &'a str,
    pub actor_id: &'a str,
    pub action_type: &'a str,
    pub target_type: Option<&'a str>,
    pub target_id: Option<&'a str>,
    pub reason: Option<&'a str>,
    pub changes: Option<&'a str>,
}

/// Parameters for creating an automod rule (avoids too-many-arguments).
pub struct CreateAutomodRuleParams<'a> {
    pub id: &'a str,
    pub server_id: &'a str,
    pub name: &'a str,
    pub rule_type: &'a str,
    pub config: &'a str,
    pub action_type: &'a str,
    pub timeout_duration_seconds: Option<i32>,
}

/// Parameters for updating an automod rule (avoids too-many-arguments).
pub struct UpdateAutomodRuleParams<'a> {
    pub rule_id: &'a str,
    pub server_id: &'a str,
    pub name: &'a str,
    pub enabled: bool,
    pub config: &'a str,
    pub action_type: &'a str,
    pub timeout_duration_seconds: Option<i32>,
}

/// A server invite.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct InviteRow {
    pub id: String,
    pub server_id: String,
    pub code: String,
    pub created_by: String,
    pub max_uses: Option<i32>,
    pub use_count: i32,
    pub expires_at: Option<String>,
    pub channel_id: Option<String>,
    pub created_at: String,
}

/// A scheduled server event.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerEventRow {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub channel_id: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub image_url: Option<String>,
    pub created_by: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// An event RSVP record.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventRsvpRow {
    pub event_id: String,
    pub user_id: String,
    pub status: String,
    pub created_at: String,
}

/// A channel follow (for announcement cross-posting).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelFollowRow {
    pub id: String,
    pub source_channel_id: String,
    pub target_channel_id: String,
    pub created_by: String,
    pub created_at: String,
}

/// A server template.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerTemplateRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub server_id: String,
    pub created_by: String,
    pub config: String,
    pub use_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Parameters for creating a server event (avoids too-many-arguments).
pub struct CreateServerEventParams<'a> {
    pub id: &'a str,
    pub server_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub channel_id: Option<&'a str>,
    pub start_time: &'a str,
    pub end_time: Option<&'a str>,
    pub image_url: Option<&'a str>,
    pub created_by: &'a str,
}
