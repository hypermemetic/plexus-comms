use super::discord_client::DiscordClient;
use super::discord_gateway::{DiscordGateway, GatewayEvent};
use super::storage::{DiscordAccount, DiscordStorage, DiscordStorageConfig};
use super::types::*;
use async_stream::stream;
use futures::Stream;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Discord {
    storage: Arc<DiscordStorage>,
    active_gateways: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl Discord {
    pub async fn new() -> Result<Self, String> {
        let storage = DiscordStorage::new(DiscordStorageConfig::default()).await?;

        Ok(Self {
            storage: Arc::new(storage),
            active_gateways: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn with_config(config: DiscordStorageConfig) -> Result<Self, String> {
        let storage = DiscordStorage::new(config).await?;

        Ok(Self {
            storage: Arc::new(storage),
            active_gateways: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

#[plexus_macros::hub_methods(
    namespace = "discord",
    version = "2.0.0",
    description = "Multi-account Discord bot integration with message sending and webhooks"
)]
impl Discord {
    // ==================== Account Management ====================

    #[plexus_macros::hub_method(
        description = "Register a new Discord bot account",
        params(
            name = "Account name (identifier for this bot)",
            bot_token = "Discord bot token"
        )
    )]
    async fn register_account(
        &self,
        name: String,
        bot_token: String,
    ) -> impl Stream<Item = RegisterAccountEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let now = chrono::Utc::now().timestamp();
            let account = DiscordAccount {
                name: name.clone(),
                bot_token,
                created_at: now,
                updated_at: now,
            };

            match storage.register_account(account).await {
                Ok(_) => yield RegisterAccountEvent::Registered {
                    account_name: name,
                },
                Err(e) => yield RegisterAccountEvent::Error { message: e },
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "List all registered Discord bot accounts"
    )]
    async fn list_accounts(&self) -> impl Stream<Item = ListAccountsEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            match storage.list_accounts().await {
                Ok(accounts) => {
                    let total = accounts.len();
                    for account in accounts {
                        yield ListAccountsEvent::Account {
                            name: account.name,
                            created_at: account.created_at,
                        };
                    }
                    yield ListAccountsEvent::Complete { total };
                }
                Err(e) => {
                    tracing::error!("Failed to list accounts: {}", e);
                    yield ListAccountsEvent::Complete { total: 0 };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Remove a Discord bot account",
        params(name = "Account name to remove")
    )]
    async fn remove_account(
        &self,
        name: String,
    ) -> impl Stream<Item = RemoveAccountEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            match storage.remove_account(&name).await {
                Ok(true) => yield RemoveAccountEvent::Removed { account_name: name },
                Ok(false) => yield RemoveAccountEvent::NotFound { account_name: name },
                Err(e) => yield RemoveAccountEvent::Error { message: e },
            }
        }
    }

    // ==================== Discord API Operations ====================

    #[plexus_macros::hub_method(
        description = "Send a message to a Discord channel from a registered bot account",
        params(
            account = "Account name to send from",
            channel_id = "Discord channel ID",
            content = "Message content",
            embed = "Rich embed object (optional)"
        )
    )]
    async fn send_message(
        &self,
        account: String,
        channel_id: String,
        content: String,
        embed: Option<serde_json::Value>,
    ) -> impl Stream<Item = SendMessageEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            // Get account
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield SendMessageEvent::Error {
                        message: format!("Account '{}' not found", account),
                        code: Some("ACCOUNT_NOT_FOUND".to_string()),
                    };
                    return;
                }
                Err(e) => {
                    yield SendMessageEvent::Error {
                        message: format!("Failed to load account: {}", e),
                        code: None,
                    };
                    return;
                }
            };

            // Create Discord client
            let client = DiscordClient::new(account_config.bot_token);

            // Send message
            match client.send_message(&channel_id, content, embed).await {
                Ok(message_id) => {
                    yield SendMessageEvent::Sent {
                        message_id,
                        channel_id,
                    };
                }
                Err(e) => {
                    yield SendMessageEvent::Error {
                        message: e,
                        code: None,
                    };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Create a webhook for a Discord channel using a registered bot account",
        params(
            account = "Account name to use",
            channel_id = "Discord channel ID",
            name = "Webhook name"
        )
    )]
    async fn create_webhook(
        &self,
        account: String,
        channel_id: String,
        name: String,
    ) -> impl Stream<Item = WebhookEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            // Get account
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield WebhookEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield WebhookEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            // Create Discord client
            let client = DiscordClient::new(account_config.bot_token);

            // Create webhook
            match client.create_webhook(&channel_id, name).await {
                Ok(webhook_url) => {
                    // Extract webhook ID from URL
                    let webhook_id = webhook_url
                        .split('/')
                        .nth_back(1)
                        .unwrap_or("unknown")
                        .to_string();

                    yield WebhookEvent::Created {
                        webhook_id,
                        webhook_url,
                    };
                }
                Err(e) => {
                    yield WebhookEvent::Error {
                        message: e,
                    };
                }
            }
        }
    }

    // ==================== Gateway Listener ====================

    #[plexus_macros::hub_method(
        streaming,
        description = "Start listening for Discord events via Gateway for a specific bot account",
        params(account = "Account name to start listening for")
    )]
    async fn start_listening(
        &self,
        account: String,
    ) -> impl Stream<Item = GatewayListenerEvent> + Send + 'static {
        let storage = self.storage.clone();
        let active_gateways = self.active_gateways.clone();
        let account_name = account.clone();

        stream! {
            // Check if already listening
            {
                let gateways = active_gateways.lock().await;
                if gateways.contains_key(&account) {
                    yield GatewayListenerEvent::Error {
                        message: format!("Account '{}' is already listening", account),
                    };
                    return;
                }
            }

            // Get account configuration
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield GatewayListenerEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield GatewayListenerEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            yield GatewayListenerEvent::Starting {
                account_name: account.clone(),
            };

            // Create Gateway connection
            let gateway = DiscordGateway::new(account_config.bot_token);

            // Create channel for Gateway events
            let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

            // Spawn Gateway task
            let gateway_task = {
                let gateway = gateway;
                tokio::spawn(async move {
                    if let Err(e) = gateway.run(event_tx).await {
                        tracing::error!("Gateway error for account '{}': {}", account_name, e);
                    }
                })
            };

            // Store the task handle
            {
                let mut gateways = active_gateways.lock().await;
                gateways.insert(account.clone(), gateway_task);
            }

            // Stream events from Gateway
            while let Some(event) = event_rx.recv().await {
                match event {
                    GatewayEvent::Ready { session_id, .. } => {
                        yield GatewayListenerEvent::Connected {
                            account_name: account.clone(),
                            session_id,
                        };
                    }
                    GatewayEvent::MessageCreate(msg) => {
                        yield GatewayListenerEvent::MessageReceived {
                            message_id: msg.id,
                            channel_id: msg.channel_id,
                            guild_id: msg.guild_id,
                            author_id: msg.author.id,
                            author_username: msg.author.username,
                            content: msg.content,
                            timestamp: msg.timestamp,
                            is_bot: msg.author.bot.unwrap_or(false),
                        };
                    }
                    GatewayEvent::MessageUpdate(msg) => {
                        yield GatewayListenerEvent::MessageUpdated {
                            message_id: msg.id,
                            channel_id: msg.channel_id,
                            guild_id: msg.guild_id,
                            author_id: msg.author.id,
                            author_username: msg.author.username,
                            content: msg.content,
                            edited_timestamp: msg.edited_timestamp,
                        };
                    }
                    GatewayEvent::MessageDelete { id, channel_id, guild_id } => {
                        yield GatewayListenerEvent::MessageDeleted {
                            message_id: id,
                            channel_id,
                            guild_id,
                        };
                    }
                    GatewayEvent::GuildMemberAdd(member) => {
                        if let Some(user) = member.user {
                            yield GatewayListenerEvent::MemberJoined {
                                user_id: user.id,
                                username: user.username,
                                guild_id: member.guild_id,
                                joined_at: member.joined_at,
                            };
                        }
                    }
                    GatewayEvent::Error(e) => {
                        yield GatewayListenerEvent::Error {
                            message: e.clone(),
                        };
                    }
                    GatewayEvent::Disconnected => {
                        yield GatewayListenerEvent::Disconnected {
                            account_name: account.clone(),
                            reason: "Gateway connection closed".to_string(),
                        };
                        break;
                    }
                }
            }

            // Clean up on disconnect
            {
                let mut gateways = active_gateways.lock().await;
                gateways.remove(&account);
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Stop listening for Discord events for a specific bot account",
        params(account = "Account name to stop listening for")
    )]
    async fn stop_listening(
        &self,
        account: String,
    ) -> impl Stream<Item = StopListeningEvent> + Send + 'static {
        let active_gateways = self.active_gateways.clone();

        stream! {
            let mut gateways = active_gateways.lock().await;

            if let Some(handle) = gateways.remove(&account) {
                handle.abort();
                yield StopListeningEvent::Stopped {
                    account_name: account,
                };
            } else {
                yield StopListeningEvent::NotListening {
                    account_name: account,
                };
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "List all bot accounts currently listening via Gateway"
    )]
    async fn list_active_listeners(
        &self,
    ) -> impl Stream<Item = ListActiveListenersEvent> + Send + 'static {
        let active_gateways = self.active_gateways.clone();

        stream! {
            let gateways = active_gateways.lock().await;
            let accounts: Vec<String> = gateways.keys().cloned().collect();
            let total = accounts.len();

            for account_name in accounts {
                yield ListActiveListenersEvent::Listener {
                    account_name,
                };
            }

            yield ListActiveListenersEvent::Complete { total };
        }
    }

    // ==================== Guild/Server Management ====================

    #[plexus_macros::hub_method(
        streaming,
        description = "List all guilds the bot is in",
        params(account = "Account name to use")
    )]
    async fn list_guilds(
        &self,
        account: String,
    ) -> impl Stream<Item = ListGuildsEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ListGuildsEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ListGuildsEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.list_guilds().await {
                Ok(guilds) => {
                    let total = guilds.len();
                    for guild in guilds {
                        yield ListGuildsEvent::Guild {
                            id: guild.id,
                            name: guild.name,
                            icon: guild.icon,
                            owner_id: guild.owner_id,
                            member_count: guild.member_count,
                        };
                    }
                    yield ListGuildsEvent::Complete { total };
                }
                Err(e) => {
                    yield ListGuildsEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Get detailed guild information including roles and channels",
        params(
            account = "Account name to use",
            guild_id = "Guild ID"
        )
    )]
    async fn get_guild(
        &self,
        account: String,
        guild_id: String,
    ) -> impl Stream<Item = GetGuildEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield GetGuildEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield GetGuildEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.get_guild(&guild_id).await {
                Ok(guild) => {
                    yield GetGuildEvent::GuildInfo {
                        id: guild.id,
                        name: guild.name,
                        icon: guild.icon,
                        owner_id: guild.owner_id,
                        member_count: guild.member_count,
                        description: guild.description,
                        role_count: guild.roles.len(),
                        channel_count: guild.channels.len(),
                    };
                }
                Err(e) => {
                    yield GetGuildEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "List all channels in a guild",
        params(
            account = "Account name to use",
            guild_id = "Guild ID"
        )
    )]
    async fn list_channels(
        &self,
        account: String,
        guild_id: String,
    ) -> impl Stream<Item = ListChannelsEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ListChannelsEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ListChannelsEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.list_channels(&guild_id).await {
                Ok(channels) => {
                    let total = channels.len();
                    for channel in channels {
                        yield ListChannelsEvent::Channel {
                            id: channel.id,
                            name: channel.name,
                            channel_type: channel.channel_type,
                            position: channel.position,
                            parent_id: channel.parent_id,
                        };
                    }
                    yield ListChannelsEvent::Complete { total };
                }
                Err(e) => {
                    yield ListChannelsEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "List members in a guild (paginated)",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            limit = "Maximum number of members to return (1-1000)"
        )
    )]
    async fn list_members(
        &self,
        account: String,
        guild_id: String,
        limit: i32,
    ) -> impl Stream<Item = ListMembersEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ListMembersEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ListMembersEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.list_members(&guild_id, limit).await {
                Ok(members) => {
                    let total = members.len();
                    for member in members {
                        if let Some(user) = member.user {
                            yield ListMembersEvent::Member {
                                user_id: user.id,
                                username: user.username,
                                discriminator: user.discriminator,
                                nick: member.nick,
                                roles: member.roles,
                                joined_at: member.joined_at,
                            };
                        }
                    }
                    yield ListMembersEvent::Complete { total };
                }
                Err(e) => {
                    yield ListMembersEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "List all roles in a guild",
        params(
            account = "Account name to use",
            guild_id = "Guild ID"
        )
    )]
    async fn list_roles(
        &self,
        account: String,
        guild_id: String,
    ) -> impl Stream<Item = ListRolesEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ListRolesEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ListRolesEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.list_roles(&guild_id).await {
                Ok(roles) => {
                    let total = roles.len();
                    for role in roles {
                        yield ListRolesEvent::Role {
                            id: role.id,
                            name: role.name,
                            color: role.color,
                            permissions: role.permissions,
                            position: role.position,
                            hoist: role.hoist,
                            mentionable: role.mentionable,
                        };
                    }
                    yield ListRolesEvent::Complete { total };
                }
                Err(e) => {
                    yield ListRolesEvent::Error { message: e };
                }
            }
        }
    }

    // ==================== Channel Management ====================

    #[plexus_macros::hub_method(
        description = "Get channel information",
        params(
            account = "Account name to use",
            channel_id = "Channel ID"
        )
    )]
    async fn get_channel(
        &self,
        account: String,
        channel_id: String,
    ) -> impl Stream<Item = GetChannelEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield GetChannelEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield GetChannelEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.get_channel(&channel_id).await {
                Ok(channel) => {
                    yield GetChannelEvent::ChannelInfo {
                        id: channel.id,
                        name: channel.name,
                        channel_type: channel.channel_type,
                        guild_id: channel.guild_id,
                        position: channel.position,
                        topic: channel.topic,
                        parent_id: channel.parent_id,
                    };
                }
                Err(e) => {
                    yield GetChannelEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Create a new channel in a guild (type: 0=text, 2=voice, 4=category)",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            name = "Channel name",
            channel_type = "Channel type (0=text, 2=voice, 4=category)",
            parent_id = "Parent category ID (optional)"
        )
    )]
    async fn create_channel(
        &self,
        account: String,
        guild_id: String,
        name: String,
        channel_type: i32,
        parent_id: Option<String>,
    ) -> impl Stream<Item = CreateChannelEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield CreateChannelEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield CreateChannelEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.create_channel(&guild_id, name, channel_type, parent_id).await {
                Ok(channel) => {
                    yield CreateChannelEvent::Created {
                        channel_id: channel.id,
                        channel_name: channel.name,
                    };
                }
                Err(e) => {
                    yield CreateChannelEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Modify a channel's properties",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            name = "New channel name (optional)",
            topic = "New channel topic (optional)",
            position = "New channel position (optional)"
        )
    )]
    async fn modify_channel(
        &self,
        account: String,
        channel_id: String,
        name: Option<String>,
        topic: Option<String>,
        position: Option<i32>,
    ) -> impl Stream<Item = ModifyChannelEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ModifyChannelEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ModifyChannelEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.modify_channel(&channel_id, name, topic, position).await {
                Ok(channel) => {
                    yield ModifyChannelEvent::Modified {
                        channel_id: channel.id,
                        channel_name: channel.name,
                    };
                }
                Err(e) => {
                    yield ModifyChannelEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Delete a channel",
        params(
            account = "Account name to use",
            channel_id = "Channel ID"
        )
    )]
    async fn delete_channel(
        &self,
        account: String,
        channel_id: String,
    ) -> impl Stream<Item = DeleteChannelEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield DeleteChannelEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield DeleteChannelEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.delete_channel(&channel_id).await {
                Ok(_) => {
                    yield DeleteChannelEvent::Deleted {
                        channel_id,
                    };
                }
                Err(e) => {
                    yield DeleteChannelEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "Get message history from a channel",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            limit = "Number of messages to retrieve (1-100)"
        )
    )]
    async fn get_messages(
        &self,
        account: String,
        channel_id: String,
        limit: i32,
    ) -> impl Stream<Item = GetMessagesEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield GetMessagesEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield GetMessagesEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.get_messages(&channel_id, limit).await {
                Ok(messages) => {
                    let total = messages.len();
                    for message in messages {
                        yield GetMessagesEvent::Message {
                            message_id: message.id,
                            channel_id: message.channel_id.clone(),
                        };
                    }
                    yield GetMessagesEvent::Complete { total };
                }
                Err(e) => {
                    yield GetMessagesEvent::Error { message: e };
                }
            }
        }
    }

    // ==================== Member Management ====================

    #[plexus_macros::hub_method(
        description = "Get member information",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            user_id = "User ID"
        )
    )]
    async fn get_member(
        &self,
        account: String,
        guild_id: String,
        user_id: String,
    ) -> impl Stream<Item = GetMemberEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield GetMemberEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield GetMemberEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.get_member(&guild_id, &user_id).await {
                Ok(member) => {
                    if let Some(user) = member.user {
                        yield GetMemberEvent::MemberInfo {
                            user_id: user.id,
                            username: user.username,
                            discriminator: user.discriminator,
                            nick: member.nick,
                            roles: member.roles,
                            joined_at: member.joined_at,
                        };
                    } else {
                        yield GetMemberEvent::Error {
                            message: "Member user information not available".to_string(),
                        };
                    }
                }
                Err(e) => {
                    yield GetMemberEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Modify a guild member's nickname or roles",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            user_id = "User ID",
            nick = "New nickname (optional)",
            roles = "Array of role IDs (optional)"
        )
    )]
    async fn modify_member(
        &self,
        account: String,
        guild_id: String,
        user_id: String,
        nick: Option<String>,
        roles: Option<Vec<String>>,
    ) -> impl Stream<Item = ModifyMemberEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ModifyMemberEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ModifyMemberEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.modify_member(&guild_id, &user_id, nick, roles).await {
                Ok(_) => {
                    yield ModifyMemberEvent::Modified {
                        user_id,
                    };
                }
                Err(e) => {
                    yield ModifyMemberEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Kick a member from a guild",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            user_id = "User ID",
            reason = "Reason for kick (optional, appears in audit log)"
        )
    )]
    async fn kick_member(
        &self,
        account: String,
        guild_id: String,
        user_id: String,
        reason: Option<String>,
    ) -> impl Stream<Item = KickMemberEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield KickMemberEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield KickMemberEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.kick_member(&guild_id, &user_id, reason).await {
                Ok(_) => {
                    yield KickMemberEvent::Kicked {
                        user_id,
                    };
                }
                Err(e) => {
                    yield KickMemberEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Ban a member from a guild",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            user_id = "User ID",
            reason = "Reason for ban (optional, appears in audit log)",
            delete_message_days = "Number of days of message history to delete (0-7, optional)"
        )
    )]
    async fn ban_member(
        &self,
        account: String,
        guild_id: String,
        user_id: String,
        reason: Option<String>,
        delete_message_days: Option<i32>,
    ) -> impl Stream<Item = BanMemberEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield BanMemberEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield BanMemberEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.ban_member(&guild_id, &user_id, reason, delete_message_days).await {
                Ok(_) => {
                    yield BanMemberEvent::Banned {
                        user_id,
                    };
                }
                Err(e) => {
                    yield BanMemberEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Unban a member from a guild",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            user_id = "User ID"
        )
    )]
    async fn unban_member(
        &self,
        account: String,
        guild_id: String,
        user_id: String,
    ) -> impl Stream<Item = UnbanMemberEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield UnbanMemberEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield UnbanMemberEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.unban_member(&guild_id, &user_id).await {
                Ok(_) => {
                    yield UnbanMemberEvent::Unbanned {
                        user_id,
                    };
                }
                Err(e) => {
                    yield UnbanMemberEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "Get list of bans for a guild",
        params(
            account = "Account name to use",
            guild_id = "Guild ID"
        )
    )]
    async fn list_bans(
        &self,
        account: String,
        guild_id: String,
    ) -> impl Stream<Item = ListBansEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ListBansEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ListBansEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.list_bans(&guild_id).await {
                Ok(bans) => {
                    let total = bans.len();
                    for ban in bans {
                        yield ListBansEvent::Ban {
                            user_id: ban.user.id,
                            username: ban.user.username,
                            discriminator: ban.user.discriminator,
                            reason: ban.reason,
                        };
                    }
                    yield ListBansEvent::Complete { total };
                }
                Err(e) => {
                    yield ListBansEvent::Error { message: e };
                }
            }
        }
    }

    // ==================== Role Management ====================

    #[plexus_macros::hub_method(
        description = "Create a new role in a guild",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            name = "Role name",
            permissions = "Permission bit string (optional)",
            color = "Role color as integer (optional)"
        )
    )]
    async fn create_role(
        &self,
        account: String,
        guild_id: String,
        name: String,
        permissions: Option<String>,
        color: Option<i32>,
    ) -> impl Stream<Item = CreateRoleEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield CreateRoleEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield CreateRoleEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.create_role(&guild_id, name.clone(), permissions, color).await {
                Ok(role) => {
                    yield CreateRoleEvent::Created {
                        role_id: role.id,
                        role_name: role.name,
                    };
                }
                Err(e) => {
                    yield CreateRoleEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Modify a role's properties",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            role_id = "Role ID",
            name = "New role name (optional)",
            permissions = "New permission bit string (optional)",
            color = "New role color as integer (optional)"
        )
    )]
    async fn modify_role(
        &self,
        account: String,
        guild_id: String,
        role_id: String,
        name: Option<String>,
        permissions: Option<String>,
        color: Option<i32>,
    ) -> impl Stream<Item = ModifyRoleEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ModifyRoleEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ModifyRoleEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.modify_role(&guild_id, &role_id, name, permissions, color).await {
                Ok(role) => {
                    yield ModifyRoleEvent::Modified {
                        role_id: role.id,
                        role_name: role.name,
                    };
                }
                Err(e) => {
                    yield ModifyRoleEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Delete a role",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            role_id = "Role ID"
        )
    )]
    async fn delete_role(
        &self,
        account: String,
        guild_id: String,
        role_id: String,
    ) -> impl Stream<Item = DeleteRoleEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield DeleteRoleEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield DeleteRoleEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.delete_role(&guild_id, &role_id).await {
                Ok(_) => {
                    yield DeleteRoleEvent::Deleted {
                        role_id,
                    };
                }
                Err(e) => {
                    yield DeleteRoleEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Add a role to a member",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            user_id = "User ID",
            role_id = "Role ID"
        )
    )]
    async fn add_role_to_member(
        &self,
        account: String,
        guild_id: String,
        user_id: String,
        role_id: String,
    ) -> impl Stream<Item = AddRoleToMemberEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield AddRoleToMemberEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield AddRoleToMemberEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.add_role_to_member(&guild_id, &user_id, &role_id).await {
                Ok(_) => {
                    yield AddRoleToMemberEvent::Added {
                        user_id,
                        role_id,
                    };
                }
                Err(e) => {
                    yield AddRoleToMemberEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Remove a role from a member",
        params(
            account = "Account name to use",
            guild_id = "Guild ID",
            user_id = "User ID",
            role_id = "Role ID"
        )
    )]
    async fn remove_role_from_member(
        &self,
        account: String,
        guild_id: String,
        user_id: String,
        role_id: String,
    ) -> impl Stream<Item = RemoveRoleFromMemberEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield RemoveRoleFromMemberEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield RemoveRoleFromMemberEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.remove_role_from_member(&guild_id, &user_id, &role_id).await {
                Ok(_) => {
                    yield RemoveRoleFromMemberEvent::Removed {
                        user_id,
                        role_id,
                    };
                }
                Err(e) => {
                    yield RemoveRoleFromMemberEvent::Error { message: e };
                }
            }
        }
    }

    // ==================== Message Management ====================

    #[plexus_macros::hub_method(
        description = "Edit an existing message",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            message_id = "Message ID",
            content = "New message content",
            embed = "Rich embed object (optional)"
        )
    )]
    async fn edit_message(
        &self,
        account: String,
        channel_id: String,
        message_id: String,
        content: String,
        embed: Option<serde_json::Value>,
    ) -> impl Stream<Item = EditMessageEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield EditMessageEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield EditMessageEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.edit_message(&channel_id, &message_id, content, embed).await {
                Ok(msg) => {
                    yield EditMessageEvent::Edited {
                        message_id: msg.id,
                        channel_id: msg.channel_id,
                    };
                }
                Err(e) => {
                    yield EditMessageEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Delete a message",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            message_id = "Message ID"
        )
    )]
    async fn delete_message(
        &self,
        account: String,
        channel_id: String,
        message_id: String,
    ) -> impl Stream<Item = DeleteMessageEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield DeleteMessageEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield DeleteMessageEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.delete_message(&channel_id, &message_id).await {
                Ok(_) => {
                    yield DeleteMessageEvent::Deleted {
                        message_id,
                        channel_id,
                    };
                }
                Err(e) => {
                    yield DeleteMessageEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Add a reaction to a message (emoji can be unicode or custom emoji format)",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            message_id = "Message ID",
            emoji = "Emoji (unicode or custom format: name:id)"
        )
    )]
    async fn add_reaction(
        &self,
        account: String,
        channel_id: String,
        message_id: String,
        emoji: String,
    ) -> impl Stream<Item = AddReactionEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield AddReactionEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield AddReactionEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.add_reaction(&channel_id, &message_id, &emoji).await {
                Ok(_) => {
                    yield AddReactionEvent::Added {
                        message_id,
                        channel_id,
                        emoji,
                    };
                }
                Err(e) => {
                    yield AddReactionEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Pin a message in a channel",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            message_id = "Message ID"
        )
    )]
    async fn pin_message(
        &self,
        account: String,
        channel_id: String,
        message_id: String,
    ) -> impl Stream<Item = PinMessageEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield PinMessageEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield PinMessageEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.pin_message(&channel_id, &message_id).await {
                Ok(_) => {
                    yield PinMessageEvent::Pinned {
                        message_id,
                        channel_id,
                    };
                }
                Err(e) => {
                    yield PinMessageEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Unpin a message from a channel",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            message_id = "Message ID"
        )
    )]
    async fn unpin_message(
        &self,
        account: String,
        channel_id: String,
        message_id: String,
    ) -> impl Stream<Item = UnpinMessageEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield UnpinMessageEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield UnpinMessageEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.unpin_message(&channel_id, &message_id).await {
                Ok(_) => {
                    yield UnpinMessageEvent::Unpinned {
                        message_id,
                        channel_id,
                    };
                }
                Err(e) => {
                    yield UnpinMessageEvent::Error { message: e };
                }
            }
        }
    }

    // ==================== Thread Management ====================

    #[plexus_macros::hub_method(
        description = "Create a thread from a message or in a channel",
        params(
            account = "Account name to use",
            channel_id = "Channel ID",
            name = "Thread name",
            message_id = "Message ID to create thread from (optional, creates standalone thread if omitted)"
        )
    )]
    async fn create_thread(
        &self,
        account: String,
        channel_id: String,
        name: String,
        message_id: Option<String>,
    ) -> impl Stream<Item = CreateThreadEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield CreateThreadEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield CreateThreadEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.create_thread(&channel_id, name.clone(), message_id).await {
                Ok(thread) => {
                    yield CreateThreadEvent::Created {
                        thread_id: thread.id,
                        thread_name: thread.name,
                    };
                }
                Err(e) => {
                    yield CreateThreadEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Join a thread",
        params(
            account = "Account name to use",
            thread_id = "Thread ID"
        )
    )]
    async fn join_thread(
        &self,
        account: String,
        thread_id: String,
    ) -> impl Stream<Item = JoinThreadEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield JoinThreadEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield JoinThreadEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.join_thread(&thread_id).await {
                Ok(_) => {
                    yield JoinThreadEvent::Joined {
                        thread_id,
                    };
                }
                Err(e) => {
                    yield JoinThreadEvent::Error { message: e };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Leave a thread",
        params(
            account = "Account name to use",
            thread_id = "Thread ID"
        )
    )]
    async fn leave_thread(
        &self,
        account: String,
        thread_id: String,
    ) -> impl Stream<Item = LeaveThreadEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield LeaveThreadEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield LeaveThreadEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let client = DiscordClient::new(account_config.bot_token);

            match client.leave_thread(&thread_id).await {
                Ok(_) => {
                    yield LeaveThreadEvent::Left {
                        thread_id,
                    };
                }
                Err(e) => {
                    yield LeaveThreadEvent::Error { message: e };
                }
            }
        }
    }
}
