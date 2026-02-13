use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Re-export from storage
pub use super::storage::{DiscordAccount, DiscordAccountConfig};

// Request types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendMessageParams {
    pub channel_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWebhookParams {
    pub channel_id: String,
    pub name: String,
}

// Response/Event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendMessageEvent {
    Sent {
        message_id: String,
        channel_id: String,
    },
    Error {
        message: String,
        code: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebhookEvent {
    Created {
        webhook_id: String,
        webhook_url: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiscordEvent {
    Message {
        message_id: String,
        channel_id: String,
        author: String,
        content: String,
        timestamp: i64,
    },
    Reaction {
        message_id: String,
        user_id: String,
        emoji: String,
    },
    Error {
        message: String,
    },
}

// Account management events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RegisterAccountEvent {
    Registered {
        account_name: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListAccountsEvent {
    Account {
        name: String,
        created_at: i64,
    },
    Complete {
        total: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemoveAccountEvent {
    Removed {
        account_name: String,
    },
    NotFound {
        account_name: String,
    },
    Error {
        message: String,
    },
}

// Guild/Server events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListGuildsEvent {
    Guild {
        id: String,
        name: String,
        icon: Option<String>,
        owner_id: String,
        member_count: Option<i32>,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GetGuildEvent {
    GuildInfo {
        id: String,
        name: String,
        icon: Option<String>,
        owner_id: String,
        member_count: Option<i32>,
        description: Option<String>,
        role_count: usize,
        channel_count: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListChannelsEvent {
    Channel {
        id: String,
        name: Option<String>,
        channel_type: i32,
        position: Option<i32>,
        parent_id: Option<String>,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListMembersEvent {
    Member {
        user_id: String,
        username: String,
        discriminator: String,
        nick: Option<String>,
        roles: Vec<String>,
        joined_at: String,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListRolesEvent {
    Role {
        id: String,
        name: String,
        color: i32,
        permissions: String,
        position: i32,
        hoist: bool,
        mentionable: bool,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

// Channel events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GetChannelEvent {
    ChannelInfo {
        id: String,
        name: Option<String>,
        channel_type: i32,
        guild_id: Option<String>,
        position: Option<i32>,
        topic: Option<String>,
        parent_id: Option<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateChannelEvent {
    Created {
        channel_id: String,
        channel_name: Option<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModifyChannelEvent {
    Modified {
        channel_id: String,
        channel_name: Option<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeleteChannelEvent {
    Deleted {
        channel_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GetMessagesEvent {
    Message {
        message_id: String,
        channel_id: String,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

// Member events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GetMemberEvent {
    MemberInfo {
        user_id: String,
        username: String,
        discriminator: String,
        nick: Option<String>,
        roles: Vec<String>,
        joined_at: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModifyMemberEvent {
    Modified {
        user_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KickMemberEvent {
    Kicked {
        user_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BanMemberEvent {
    Banned {
        user_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnbanMemberEvent {
    Unbanned {
        user_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListBansEvent {
    Ban {
        user_id: String,
        username: String,
        discriminator: String,
        reason: Option<String>,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

// Role events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateRoleEvent {
    Created {
        role_id: String,
        role_name: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModifyRoleEvent {
    Modified {
        role_id: String,
        role_name: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeleteRoleEvent {
    Deleted {
        role_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AddRoleToMemberEvent {
    Added {
        user_id: String,
        role_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemoveRoleFromMemberEvent {
    Removed {
        user_id: String,
        role_id: String,
    },
    Error {
        message: String,
    },
}

// Message events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditMessageEvent {
    Edited {
        message_id: String,
        channel_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeleteMessageEvent {
    Deleted {
        message_id: String,
        channel_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AddReactionEvent {
    Added {
        message_id: String,
        channel_id: String,
        emoji: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PinMessageEvent {
    Pinned {
        message_id: String,
        channel_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnpinMessageEvent {
    Unpinned {
        message_id: String,
        channel_id: String,
    },
    Error {
        message: String,
    },
}

// Thread events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateThreadEvent {
    Created {
        thread_id: String,
        thread_name: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JoinThreadEvent {
    Joined {
        thread_id: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LeaveThreadEvent {
    Left {
        thread_id: String,
    },
    Error {
        message: String,
    },
}

// Gateway listener events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayListenerEvent {
    Starting {
        account_name: String,
    },
    Connected {
        account_name: String,
        session_id: String,
    },
    MessageReceived {
        message_id: String,
        channel_id: String,
        guild_id: Option<String>,
        author_id: String,
        author_username: String,
        content: String,
        timestamp: String,
        is_bot: bool,
    },
    MessageUpdated {
        message_id: String,
        channel_id: String,
        guild_id: Option<String>,
        author_id: String,
        author_username: String,
        content: String,
        edited_timestamp: Option<String>,
    },
    MessageDeleted {
        message_id: String,
        channel_id: String,
        guild_id: Option<String>,
    },
    MemberJoined {
        user_id: String,
        username: String,
        guild_id: String,
        joined_at: String,
    },
    Error {
        message: String,
    },
    Disconnected {
        account_name: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StartListeningEvent {
    Starting {
        account_name: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StopListeningEvent {
    Stopped {
        account_name: String,
    },
    NotListening {
        account_name: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListActiveListenersEvent {
    Listener {
        account_name: String,
    },
    Complete {
        total: usize,
    },
}
