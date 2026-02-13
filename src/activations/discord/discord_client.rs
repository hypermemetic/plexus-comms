use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

#[derive(Debug, Clone)]
pub struct DiscordClient {
    bot_token: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct CreateMessagePayload {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
    pub channel_id: String,
}

#[derive(Debug, Serialize)]
struct CreateWebhookPayload {
    name: String,
}

#[derive(Debug, Deserialize)]
struct DiscordWebhook {
    id: String,
    token: String,
}

#[derive(Debug, Deserialize)]
struct DiscordError {
    message: String,
    code: i32,
}

// Guild/Server types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner_id: String,
    pub member_count: Option<i32>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GuildDetailed {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner_id: String,
    pub member_count: Option<i32>,
    pub description: Option<String>,
    pub roles: Vec<Role>,
    pub channels: Vec<Channel>,
}

// Channel types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub guild_id: Option<String>,
    pub position: Option<i32>,
    pub topic: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateChannelPayload {
    name: String,
    #[serde(rename = "type")]
    channel_type: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ModifyChannelPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<i32>,
}

// Member types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Member {
    pub user: Option<User>,
    pub nick: Option<String>,
    pub roles: Vec<String>,
    pub joined_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ModifyMemberPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    nick: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Ban {
    pub user: User,
    pub reason: Option<String>,
}

// Role types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub color: i32,
    pub permissions: String,
    pub position: i32,
    pub hoist: bool,
    pub mentionable: bool,
}

#[derive(Debug, Serialize)]
struct CreateRolePayload {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<i32>,
}

#[derive(Debug, Serialize)]
struct ModifyRolePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<i32>,
}

// Thread types
#[derive(Debug, Serialize)]
struct CreateThreadPayload {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_archive_duration: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Thread {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: i32,
    pub guild_id: Option<String>,
}

// Message edit payload
#[derive(Debug, Serialize)]
struct EditMessagePayload {
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<serde_json::Value>>,
}

impl DiscordClient {
    pub fn new(bot_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { bot_token, client }
    }

    /// Helper function to handle Discord API responses
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, String> {
        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else if response.status().as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.0);
            Err(format!(
                "Rate limited. Retry after {} seconds",
                retry_after
            ))
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if let Ok(discord_error) = serde_json::from_str::<DiscordError>(&error_text) {
                Err(format!(
                    "Discord API error ({}): {}",
                    discord_error.code, discord_error.message
                ))
            } else {
                Err(format!("HTTP {}: {}", status, error_text))
            }
        }
    }

    /// Helper for successful responses with no content
    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<(), String> {
        if response.status().is_success() {
            Ok(())
        } else if response.status().as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(1.0);
            Err(format!(
                "Rate limited. Retry after {} seconds",
                retry_after
            ))
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if let Ok(discord_error) = serde_json::from_str::<DiscordError>(&error_text) {
                Err(format!(
                    "Discord API error ({}): {}",
                    discord_error.code, discord_error.message
                ))
            } else {
                Err(format!("HTTP {}: {}", status, error_text))
            }
        }
    }

    /// Send a message to a Discord channel
    /// Returns the message ID on success
    pub async fn send_message(
        &self,
        channel_id: &str,
        content: String,
        embed: Option<serde_json::Value>,
    ) -> Result<String, String> {
        let url = format!("{}/channels/{}/messages", DISCORD_API_BASE, channel_id);

        let payload = CreateMessagePayload {
            content,
            embeds: embed.map(|e| vec![e]),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        let message: DiscordMessage = self.handle_response(response).await?;
        Ok(message.id)
    }

    /// Create a webhook for a Discord channel
    /// Returns the webhook URL on success
    pub async fn create_webhook(&self, channel_id: &str, name: String) -> Result<String, String> {
        let url = format!("{}/channels/{}/webhooks", DISCORD_API_BASE, channel_id);

        let payload = CreateWebhookPayload { name };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        let webhook: DiscordWebhook = self.handle_response(response).await?;
        Ok(format!(
            "{}/webhooks/{}/{}",
            DISCORD_API_BASE, webhook.id, webhook.token
        ))
    }

    // ==================== Guild/Server Management ====================

    /// List all guilds the bot is in
    pub async fn list_guilds(&self) -> Result<Vec<Guild>, String> {
        let url = format!("{}/users/@me/guilds", DISCORD_API_BASE);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Get detailed guild information
    pub async fn get_guild(&self, guild_id: &str) -> Result<GuildDetailed, String> {
        let url = format!("{}/guilds/{}?with_counts=true", DISCORD_API_BASE, guild_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// List all channels in a guild
    pub async fn list_channels(&self, guild_id: &str) -> Result<Vec<Channel>, String> {
        let url = format!("{}/guilds/{}/channels", DISCORD_API_BASE, guild_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// List members in a guild (paginated)
    pub async fn list_members(&self, guild_id: &str, limit: i32) -> Result<Vec<Member>, String> {
        let url = format!(
            "{}/guilds/{}/members?limit={}",
            DISCORD_API_BASE, guild_id, limit
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// List all roles in a guild
    pub async fn list_roles(&self, guild_id: &str) -> Result<Vec<Role>, String> {
        let url = format!("{}/guilds/{}/roles", DISCORD_API_BASE, guild_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    // ==================== Channel Management ====================

    /// Get channel information
    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel, String> {
        let url = format!("{}/channels/{}", DISCORD_API_BASE, channel_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Create a new channel in a guild
    /// channel_type: 0 = text, 2 = voice, 4 = category
    pub async fn create_channel(
        &self,
        guild_id: &str,
        name: String,
        channel_type: i32,
        parent_id: Option<String>,
    ) -> Result<Channel, String> {
        let url = format!("{}/guilds/{}/channels", DISCORD_API_BASE, guild_id);

        let payload = CreateChannelPayload {
            name,
            channel_type,
            parent_id,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Modify a channel
    pub async fn modify_channel(
        &self,
        channel_id: &str,
        name: Option<String>,
        topic: Option<String>,
        position: Option<i32>,
    ) -> Result<Channel, String> {
        let url = format!("{}/channels/{}", DISCORD_API_BASE, channel_id);

        let payload = ModifyChannelPayload {
            name,
            topic,
            position,
        };

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Delete a channel
    pub async fn delete_channel(&self, channel_id: &str) -> Result<(), String> {
        let url = format!("{}/channels/{}", DISCORD_API_BASE, channel_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Get message history from a channel
    pub async fn get_messages(
        &self,
        channel_id: &str,
        limit: i32,
    ) -> Result<Vec<DiscordMessage>, String> {
        let url = format!(
            "{}/channels/{}/messages?limit={}",
            DISCORD_API_BASE, channel_id, limit
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    // ==================== Member Management ====================

    /// Get member information
    pub async fn get_member(&self, guild_id: &str, user_id: &str) -> Result<Member, String> {
        let url = format!(
            "{}/guilds/{}/members/{}",
            DISCORD_API_BASE, guild_id, user_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Modify a guild member
    pub async fn modify_member(
        &self,
        guild_id: &str,
        user_id: &str,
        nick: Option<String>,
        roles: Option<Vec<String>>,
    ) -> Result<Member, String> {
        let url = format!(
            "{}/guilds/{}/members/{}",
            DISCORD_API_BASE, guild_id, user_id
        );

        let payload = ModifyMemberPayload { nick, roles };

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Kick a member from a guild
    pub async fn kick_member(
        &self,
        guild_id: &str,
        user_id: &str,
        reason: Option<String>,
    ) -> Result<(), String> {
        let url = format!(
            "{}/guilds/{}/members/{}",
            DISCORD_API_BASE, guild_id, user_id
        );

        let mut request = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token));

        if let Some(r) = reason {
            request = request.header("X-Audit-Log-Reason", r);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Ban a member from a guild
    pub async fn ban_member(
        &self,
        guild_id: &str,
        user_id: &str,
        reason: Option<String>,
        delete_message_days: Option<i32>,
    ) -> Result<(), String> {
        let url = format!("{}/guilds/{}/bans/{}", DISCORD_API_BASE, guild_id, user_id);

        let mut payload = serde_json::Map::new();
        if let Some(days) = delete_message_days {
            payload.insert(
                "delete_message_days".to_string(),
                serde_json::Value::Number(days.into()),
            );
        }

        let mut request = self
            .client
            .put(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload);

        if let Some(r) = reason {
            request = request.header("X-Audit-Log-Reason", r);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Unban a member from a guild
    pub async fn unban_member(&self, guild_id: &str, user_id: &str) -> Result<(), String> {
        let url = format!("{}/guilds/{}/bans/{}", DISCORD_API_BASE, guild_id, user_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Get list of bans for a guild
    pub async fn list_bans(&self, guild_id: &str) -> Result<Vec<Ban>, String> {
        let url = format!("{}/guilds/{}/bans", DISCORD_API_BASE, guild_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    // ==================== Role Management ====================

    /// Create a role in a guild
    pub async fn create_role(
        &self,
        guild_id: &str,
        name: String,
        permissions: Option<String>,
        color: Option<i32>,
    ) -> Result<Role, String> {
        let url = format!("{}/guilds/{}/roles", DISCORD_API_BASE, guild_id);

        let payload = CreateRolePayload {
            name,
            permissions,
            color,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Modify a role
    pub async fn modify_role(
        &self,
        guild_id: &str,
        role_id: &str,
        name: Option<String>,
        permissions: Option<String>,
        color: Option<i32>,
    ) -> Result<Role, String> {
        let url = format!("{}/guilds/{}/roles/{}", DISCORD_API_BASE, guild_id, role_id);

        let payload = ModifyRolePayload {
            name,
            permissions,
            color,
        };

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Delete a role
    pub async fn delete_role(&self, guild_id: &str, role_id: &str) -> Result<(), String> {
        let url = format!("{}/guilds/{}/roles/{}", DISCORD_API_BASE, guild_id, role_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Add a role to a member
    pub async fn add_role_to_member(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<(), String> {
        let url = format!(
            "{}/guilds/{}/members/{}/roles/{}",
            DISCORD_API_BASE, guild_id, user_id, role_id
        );

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Remove a role from a member
    pub async fn remove_role_from_member(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
    ) -> Result<(), String> {
        let url = format!(
            "{}/guilds/{}/members/{}/roles/{}",
            DISCORD_API_BASE, guild_id, user_id, role_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    // ==================== Message Management ====================

    /// Edit a message
    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        content: String,
        embed: Option<Value>,
    ) -> Result<DiscordMessage, String> {
        let url = format!(
            "{}/channels/{}/messages/{}",
            DISCORD_API_BASE, channel_id, message_id
        );

        let payload = EditMessagePayload {
            content,
            embeds: embed.map(|e| vec![e]),
        };

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Delete a message
    pub async fn delete_message(&self, channel_id: &str, message_id: &str) -> Result<(), String> {
        let url = format!(
            "{}/channels/{}/messages/{}",
            DISCORD_API_BASE, channel_id, message_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Add a reaction to a message
    pub async fn add_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), String> {
        let url = format!(
            "{}/channels/{}/messages/{}/reactions/{}/@me",
            DISCORD_API_BASE, channel_id, message_id, emoji
        );

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Pin a message
    pub async fn pin_message(&self, channel_id: &str, message_id: &str) -> Result<(), String> {
        let url = format!(
            "{}/channels/{}/pins/{}",
            DISCORD_API_BASE, channel_id, message_id
        );

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Unpin a message
    pub async fn unpin_message(&self, channel_id: &str, message_id: &str) -> Result<(), String> {
        let url = format!(
            "{}/channels/{}/pins/{}",
            DISCORD_API_BASE, channel_id, message_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    // ==================== Thread Management ====================

    /// Create a thread from a message or in a channel
    pub async fn create_thread(
        &self,
        channel_id: &str,
        name: String,
        message_id: Option<String>,
    ) -> Result<Thread, String> {
        let url = if let Some(msg_id) = message_id {
            format!(
                "{}/channels/{}/messages/{}/threads",
                DISCORD_API_BASE, channel_id, msg_id
            )
        } else {
            format!("{}/channels/{}/threads", DISCORD_API_BASE, channel_id)
        };

        let payload = CreateThreadPayload {
            name,
            auto_archive_duration: Some(60), // 60 minutes default
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_response(response).await
    }

    /// Join a thread
    pub async fn join_thread(&self, thread_id: &str) -> Result<(), String> {
        let url = format!(
            "{}/channels/{}/thread-members/@me",
            DISCORD_API_BASE, thread_id
        );

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }

    /// Leave a thread
    pub async fn leave_thread(&self, thread_id: &str) -> Result<(), String> {
        let url = format!(
            "{}/channels/{}/thread-members/@me",
            DISCORD_API_BASE, thread_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bot {}", self.bot_token))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        self.handle_empty_response(response).await
    }
}
