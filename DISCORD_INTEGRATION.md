# Discord Integration

This document describes the Discord integration in plexus-comms, which follows the same multi-account pattern as email activation.

## Features

- Multi-account support (register multiple Discord bots)
- SQLite-based account storage
- Real Discord HTTP API integration (no heavy dependencies)
- Send messages to channels
- Create webhooks
- Proper rate limiting handling

## Architecture

The Discord integration follows the same pattern as email:

1. **storage.rs** - SQLite database for storing bot accounts
2. **discord_client.rs** - HTTP client for Discord API v10
3. **activation.rs** - Hub methods for account management and operations
4. **types.rs** - Event types and schemas

## Database Schema

```sql
CREATE TABLE discord_accounts (
    name TEXT PRIMARY KEY,
    bot_token TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

## API Methods

### Account Management

#### `register_account(name, bot_token)`
Register a Discord bot account for use.

**Parameters:**
- `name` (string) - Account identifier (e.g., "my-bot")
- `bot_token` (string) - Discord bot token

**Returns:** Stream of `RegisterAccountEvent`

#### `list_accounts()`
List all registered Discord bot accounts.

**Returns:** Stream of `ListAccountsEvent` (streaming)

#### `remove_account(name)`
Remove a Discord bot account.

**Parameters:**
- `name` (string) - Account identifier to remove

**Returns:** Stream of `RemoveAccountEvent`

### Discord Operations

#### `send_message(account, channel_id, content, embed?)`
Send a message to a Discord channel.

**Parameters:**
- `account` (string) - Account name to send from
- `channel_id` (string) - Discord channel ID
- `content` (string) - Message content
- `embed` (object, optional) - Discord embed object

**Returns:** Stream of `SendMessageEvent`

#### `create_webhook(account, channel_id, name)`
Create a webhook for a Discord channel.

**Parameters:**
- `account` (string) - Account name to use
- `channel_id` (string) - Discord channel ID
- `name` (string) - Webhook name

**Returns:** Stream of `WebhookEvent`

#### `listen_messages()`
Listen for Discord events (stub implementation).

**Returns:** Stream of `DiscordEvent` (streaming)

**Note:** Full Gateway implementation is not included. Use webhooks or polling for now.

## Usage Example (TypeScript)

```typescript
import { PlexusClient } from '@plexus/client';

const client = new PlexusClient('ws://localhost:4445');

// Register a Discord bot account
const registerEvents = client.call('discord', 'register_account', {
  name: 'my-bot',
  bot_token: 'REDACTED_DISCORD_BOT_TOKEN'
});

for await (const event of registerEvents) {
  console.log('Register:', event);
  // { type: 'registered', account_name: 'my-bot' }
}

// List all registered accounts
const accounts = client.call('discord', 'list_accounts', {});

for await (const event of accounts) {
  console.log('Account:', event);
  // { type: 'account', name: 'my-bot', created_at: 1234567890 }
  // { type: 'complete', total: 1 }
}

// Send a message
const sendEvents = client.call('discord', 'send_message', {
  account: 'my-bot',
  channel_id: '1234567890',
  content: 'Hello Discord!',
  embed: null
});

for await (const event of sendEvents) {
  console.log('Send:', event);
  // { type: 'sent', message_id: '...', channel_id: '...' }
}

// Create a webhook
const webhookEvents = client.call('discord', 'create_webhook', {
  account: 'my-bot',
  channel_id: '1234567890',
  name: 'Plexus Webhook'
});

for await (const event of webhookEvents) {
  console.log('Webhook:', event);
  // { type: 'created', webhook_id: '...', webhook_url: 'https://...' }
}

// Remove account
const removeEvents = client.call('discord', 'remove_account', {
  name: 'my-bot'
});

for await (const event of removeEvents) {
  console.log('Remove:', event);
  // { type: 'removed', account_name: 'my-bot' }
}
```

## Discord API Integration

The integration uses the Discord HTTP API v10 directly via `reqwest`:

- **Base URL:** `https://discord.com/api/v10`
- **Authentication:** `Authorization: Bot <token>` header
- **Rate Limiting:** Automatic detection with `retry-after` header handling
- **Error Handling:** Proper Discord error response parsing

### Endpoints Used

1. **POST** `/channels/{channel_id}/messages` - Send message
2. **POST** `/channels/{channel_id}/webhooks` - Create webhook

## Error Handling

All methods return proper error events:

```typescript
{
  type: 'error',
  message: 'Error description',
  code?: 'ERROR_CODE'  // Optional error code
}
```

### Common Error Codes

- `ACCOUNT_NOT_FOUND` - The specified account doesn't exist
- Discord API errors include the Discord error code and message

## Rate Limiting

The Discord client automatically detects rate limits (HTTP 429) and returns an error with retry information:

```typescript
{
  type: 'error',
  message: 'Rate limited. Retry after 1.5 seconds',
  code: null
}
```

## Storage

Account data is stored in `discord_accounts.db` by default. You can customize the database path:

```rust
use plexus_comms::activations::discord::{Discord, DiscordStorageConfig};
use std::path::PathBuf;

let config = DiscordStorageConfig {
    db_path: PathBuf::from("custom_path.db"),
};

let discord = Discord::with_config(config).await?;
```

## Registration in Builder

Discord is always registered in the hub (like email), regardless of configuration:

```rust
// From builder.rs
let discord = Discord::new().await?;
hub = hub.register(discord);
```

This allows runtime account registration without requiring upfront configuration.

## Comparison with Email Pattern

| Feature | Email | Discord |
|---------|-------|---------|
| Multi-account | Yes (SMTP + IMAP) | Yes (bot tokens) |
| Storage | `email_accounts.db` | `discord_accounts.db` |
| Account Config | SmtpConfig + ImapConfig | DiscordAccountConfig |
| Always Registered | Yes | Yes |
| Streaming Methods | `list_accounts`, `read_inbox` | `list_accounts`, `listen_messages` |
| API Client | lettre, async-imap | reqwest (HTTP) |

## Implementation Notes

1. **No serenity dependency** - Uses direct HTTP API calls instead of heavy Discord library
2. **Minimal surface area** - Only implements essential operations (send, webhook)
3. **Future extensibility** - Gateway listener is stubbed for future WebSocket implementation
4. **Account-based** - All operations require a registered account, preventing accidental usage
5. **Storage isolation** - Each activation manages its own database file

## Testing

Start the server:
```bash
cargo run
```

Use the TypeScript example above to test the integration.

## Future Enhancements

Potential additions (not currently implemented):

1. Gateway WebSocket listener for real-time events
2. Reaction management (add/remove reactions)
3. Guild/server management
4. Role management
5. Message editing/deletion
6. Thread support
7. Slash command registration
8. Interaction handling
