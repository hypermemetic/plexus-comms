# Plexus Communications

A comprehensive multi-platform communications integration library for Plexus RPC, providing unified APIs for email, Discord, SMS, push notifications, Telegram, WhatsApp, and Slack.

## Features

- **Multi-Account Architecture** - Register and manage multiple accounts per platform at runtime
- **Streaming APIs** - Event-driven architecture with real-time message streaming
- **SQLite Storage** - Persistent account configuration storage
- **Modular Design** - Each platform is an independent activation
- **Real-time Events** - Discord Gateway support for live message streams

## Supported Platforms

### ✅ Email (v2.0.0)
- **SMTP Sending** - Support for standard SMTP, SendGrid, AWS SES, Postmark
- **IMAP Reading** - Read inbox, search messages, mark as read/unread
- **Multi-Account** - Manage multiple email accounts with different providers
- **Database**: `email_accounts.db`

**Methods:**
- `register_account(name, smtp?, imap?)` - Register email account
- `send_from(account, to, subject, body, ...)` - Send email
- `read_inbox(account, limit?)` - Read inbox messages
- `search_messages(account, query, limit?)` - Search emails
- `mark_as_read(account, message_id)` - Mark message as read
- `mark_as_unread(account, message_id)` - Mark as unread
- `list_accounts()` - List all registered accounts
- `remove_account(name)` - Remove account

### ✅ Discord (v2.0.0)
- **Message Sending** - Send messages with embeds to any channel
- **Gateway Support** - Real-time event streaming (messages, joins, reactions)
- **Guild Management** - Manage servers, channels, roles, members
- **Moderation** - Kick, ban, timeout, role assignment
- **Multi-Account** - Run multiple Discord bots simultaneously
- **Database**: `discord_accounts.db`

**Account Management:**
- `register_account(name, bot_token)` - Register Discord bot
- `list_accounts()` - List registered bots
- `remove_account(name)` - Remove bot account

**Messaging:**
- `send_message(account, channel_id, content, embed?)` - Send message
- `edit_message(account, channel_id, message_id, content, embed?)` - Edit message
- `delete_message(account, channel_id, message_id)` - Delete message
- `get_messages(account, channel_id, limit)` - Get message history
- `add_reaction(account, channel_id, message_id, emoji)` - Add reaction
- `pin_message(account, channel_id, message_id)` - Pin message
- `unpin_message(account, channel_id, message_id)` - Unpin message

**Guild/Server Management:**
- `list_guilds(account)` - List all guilds bot is in
- `get_guild(account, guild_id)` - Get detailed guild info
- `list_channels(account, guild_id)` - List all channels
- `list_members(account, guild_id, limit)` - List members
- `list_roles(account, guild_id)` - List roles

**Channel Management:**
- `get_channel(account, channel_id)` - Get channel info
- `create_channel(account, guild_id, name, channel_type, parent_id?)` - Create channel
- `modify_channel(account, channel_id, name?, topic?, position?)` - Modify channel
- `delete_channel(account, channel_id)` - Delete channel

**Member Management:**
- `get_member(account, guild_id, user_id)` - Get member info
- `modify_member(account, guild_id, user_id, nick?, roles?)` - Modify member
- `kick_member(account, guild_id, user_id, reason?)` - Kick member
- `ban_member(account, guild_id, user_id, reason?, delete_message_days?)` - Ban member
- `unban_member(account, guild_id, user_id)` - Unban member
- `list_bans(account, guild_id)` - List all bans

**Role Management:**
- `create_role(account, guild_id, name, permissions?, color?)` - Create role
- `modify_role(account, guild_id, role_id, name?, permissions?, color?)` - Modify role
- `delete_role(account, guild_id, role_id)` - Delete role
- `add_role_to_member(account, guild_id, user_id, role_id)` - Add role to member
- `remove_role_from_member(account, guild_id, user_id, role_id)` - Remove role from member

**Thread Management:**
- `create_thread(account, channel_id, name, message_id?)` - Create thread
- `join_thread(account, thread_id)` - Join thread
- `leave_thread(account, thread_id)` - Leave thread

**Webhooks:**
- `create_webhook(account, channel_id, name)` - Create webhook

**Gateway (Real-time Events):**
- `start_listening(account)` - Start Gateway connection, stream live events
- `stop_listening(account)` - Stop Gateway connection
- `list_active_listeners()` - List bots currently listening

**Gateway Events:**
- `MessageReceived` - New message posted (includes content, author, channel_id, guild_id)
- `MessageUpdated` - Message edited
- `MessageDeleted` - Message deleted
- `MemberJoined` - New member joined server
- `Connected` - Bot connected to Gateway
- `Disconnected` - Connection lost

### 🚧 SMS (Planned)
- **Twilio** - SMS sending via Twilio API
- **AWS SNS** - SMS via Amazon SNS
- Multi-account support

### 🚧 Push Notifications (Planned)
- **APNs** - Apple Push Notification service
- **FCM** - Firebase Cloud Messaging (Android)
- Multi-device support

### 🚧 Telegram (Planned)
- **Bot API** - Send messages, inline keyboards, media
- **Webhook Support** - Receive updates
- Multi-bot support

### 🚧 WhatsApp (Planned)
- **Business API** - WhatsApp Business API integration
- **Message Templates** - Template-based messaging

### 🚧 Slack (Planned)
- **Web API** - Channel management, messaging
- **Webhooks** - Incoming webhooks
- Multi-workspace support

## Quick Start

### Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
plexus-comms = { path = "../plexus-comms" }
```

### Basic Usage

```rust
use plexus_comms::builder::build_default_hub;

#[tokio::main]
async fn main() -> Result<(), String> {
    let hub = build_default_hub().await?;
    // Hub is ready with email and Discord activations
    Ok(())
}
```

### Running the Server

```bash
cd plexus-comms
cargo run --release --features email-smtp,email-imap
```

Server starts on `ws://127.0.0.1:4445`

### Using with Synapse CLI

#### Email Example

```bash
# Register email account
synapse -P 4445 comms email register_account \
  --name "my-email" \
  --smtp '{"host":"smtp.gmail.com","port":587,"username":"you@gmail.com","password":"***","use_tls":true}' \
  --imap '{"host":"imap.gmail.com","port":993,"username":"you@gmail.com","password":"***","use_tls":true}'

# Send email
synapse -P 4445 comms email send_from \
  --account "my-email" \
  --to '["recipient@example.com"]' \
  --subject "Hello" \
  --body "Test email"

# Read inbox
synapse -P 4445 comms email read_inbox --account "my-email" --limit 10
```

#### Discord Example

```bash
# Register Discord bot
synapse -P 4445 comms discord register_account \
  --name "my-bot" \
  --bot_token "YOUR_BOT_TOKEN"

# Send message
synapse -P 4445 comms discord send_message \
  --account "my-bot" \
  --channel_id "CHANNEL_ID" \
  --content "Hello from plexus-comms!"

# Create channel
synapse -P 4445 comms discord create_channel \
  --account "my-bot" \
  --guild_id "GUILD_ID" \
  --name "new-channel" \
  --channel_type 0

# Start listening to events (real-time stream)
synapse -P 4445 comms discord start_listening --account "my-bot"

# List active listeners
synapse -P 4445 comms discord list_active_listeners

# Stop listening
synapse -P 4445 comms discord stop_listening --account "my-bot"
```

## Architecture

### Multi-Account Pattern

All activations follow a consistent multi-account pattern:

1. **Runtime Registration** - Accounts are registered via API calls, not config files
2. **SQLite Storage** - Account credentials stored in platform-specific databases
3. **Account-Based Methods** - All methods accept an `account` parameter
4. **Independent Operation** - Each account operates independently

### File Structure

```
plexus-comms/
├── src/
│   ├── activations/
│   │   ├── email/
│   │   │   ├── activation.rs      # Email hub methods
│   │   │   ├── storage.rs         # SQLite account storage
│   │   │   ├── smtp.rs            # SMTP providers
│   │   │   ├── imap.rs            # IMAP client
│   │   │   ├── types.rs           # Email types & events
│   │   │   └── mod.rs
│   │   ├── discord/
│   │   │   ├── activation.rs      # Discord hub methods
│   │   │   ├── storage.rs         # SQLite account storage
│   │   │   ├── discord_client.rs  # Discord API HTTP client
│   │   │   ├── discord_gateway.rs # Discord Gateway WebSocket client
│   │   │   ├── types.rs           # Discord types & events
│   │   │   └── mod.rs
│   │   ├── sms/                   # SMS activation (planned)
│   │   ├── push/                  # Push notifications (planned)
│   │   ├── telegram/              # Telegram activation (planned)
│   │   ├── whatsapp/              # WhatsApp activation (planned)
│   │   ├── slack/                 # Slack activation (planned)
│   │   └── mod.rs
│   ├── builder.rs                 # Hub builder
│   ├── config.rs                  # Configuration types
│   ├── lib.rs                     # Library exports
│   └── main.rs                    # Server entry point
├── Cargo.toml
└── README.md
```

### Discord Gateway Architecture

The Discord Gateway implementation uses a modular, per-bot architecture:

- **Independent Connections** - Each bot account has its own WebSocket connection
- **Concurrent Listening** - Multiple bots can listen simultaneously
- **Session Management** - Automatic reconnection with session resuming
- **Heartbeat System** - Background task maintains connection health
- **Event Streaming** - Real-time events streamed via tokio channels

**Gateway Manager:**
```rust
pub struct Discord {
    storage: Arc<DiscordStorage>,
    active_gateways: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}
```

Each `start_listening` call:
1. Retrieves bot token from storage
2. Spawns a Gateway task for that bot
3. Maintains WebSocket connection with heartbeats
4. Streams events to the caller
5. Stores task handle for later shutdown

## Configuration

### Default Configuration

When no config file is present, plexus-comms starts with:
- Email activation (multi-account mode)
- Discord activation (multi-account mode)

### Custom Configuration

Create `config.toml`:

```toml
[sms]
provider = "twilio"
account_sid = "ACXXXX"
auth_token = "your-token"
from_number = "+1234567890"

[push]
apns_key_id = "your-key-id"
apns_team_id = "your-team-id"
fcm_server_key = "your-fcm-key"

[telegram]
bot_token = "your-bot-token"

[whatsapp]
account_sid = "ACXXXX"
auth_token = "your-token"
from_number = "whatsapp:+1234567890"

[slack]
bot_token = "xoxb-your-token"
```

Then run:
```bash
cargo run --release -- --config config.toml
```

## Database Schema

### Email Accounts (`email_accounts.db`)

```sql
CREATE TABLE email_accounts (
    name TEXT PRIMARY KEY,
    smtp_config TEXT,
    imap_config TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### Discord Accounts (`discord_accounts.db`)

```sql
CREATE TABLE discord_accounts (
    name TEXT PRIMARY KEY,
    bot_token TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

## Dependencies

### Core
- `plexus-core` - Plexus RPC framework
- `plexus-transport` - WebSocket transport layer
- `tokio` - Async runtime
- `sqlx` - Database access

### Email
- `lettre` - SMTP client
- `async-imap` - IMAP client
- `async-native-tls` - TLS support
- `tokio-util` - Compat layer for async I/O traits

### Discord
- `reqwest` - HTTP client for Discord API
- `tokio-tungstenite` - WebSocket for Discord Gateway
- `serde_json` - JSON serialization

## Building a Command Bot

To build a bot that reacts to Discord messages:

```rust
// Connect to plexus-comms and start listening
let events = discord.start_listening("my-bot").await?;

// Process events
while let Some(event) = events.next().await {
    match event {
        DiscordGatewayEvent::MessageReceived { content, channel_id, author_id, is_bot, ... } => {
            // Ignore bot messages
            if is_bot {
                continue;
            }

            // Parse commands
            if content.starts_with("!ping") {
                discord.send_message("my-bot", &channel_id, "Pong!".to_string(), None).await?;
            } else if content.starts_with("!help") {
                let help_text = "Available commands: !ping, !help, !info";
                discord.send_message("my-bot", &channel_id, help_text.to_string(), None).await?;
            }
        }
        DiscordGatewayEvent::Connected { .. } => {
            println!("Bot connected to Discord Gateway!");
        }
        _ => {}
    }
}
```

## Development

### Building

```bash
cargo build --features email-smtp,email-imap
```

### Running Tests

```bash
cargo test
```

### Features

- `email-smtp` - SMTP email sending (enabled by default)
- `email-imap` - IMAP email reading (enabled by default)

## Discord Channel Types

- `0` - Text channel
- `2` - Voice channel
- `4` - Category
- `5` - Announcement channel
- `13` - Stage channel
- `15` - Forum channel

## Discord Permissions

Common permission values:
- `8` - Administrator
- `2048` - Send Messages
- `3072` - Send Messages + Embed Links
- `37080128` - Read Messages + Send Messages + Read Message History
- `104324673` - Member (read, send, react, add reactions, use external emojis, read history)
- `268435462` - Moderator (manage messages, kick members, ban members, etc.)

## License

MIT

## Contributing

Contributions welcome! Please ensure all tests pass and follow the existing code structure.

## Roadmap

- [x] Email (SMTP + IMAP)
- [x] Discord (API + Gateway)
- [ ] Discord message storage and CRUD
- [ ] SMS (Twilio, AWS SNS)
- [ ] Push Notifications (APNs, FCM)
- [ ] Telegram Bot API
- [ ] WhatsApp Business API
- [ ] Slack Web API
- [ ] Template system for messages
- [ ] Webhook receivers for incoming messages
- [ ] Rate limiting and retry logic
- [ ] Message queuing
- [ ] Analytics and logging
