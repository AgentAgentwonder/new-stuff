# Chat Integrations Guide

This guide covers the implementation of Telegram, Slack, and Discord notification integrations for the trading platform.

## Overview

The chat integrations allow users to receive real-time price alerts and notifications through Telegram, Slack, and Discord channels. The system includes:

- Multiple destination support per service
- Rate limiting compliance
- Delivery status logging
- Message formatting (Markdown, embeds, mentions)
- Test send functionality
- Retry logic with error handling

## Architecture

### Frontend Components

- **ChatIntegrations Component** (`/src/pages/Settings/ChatIntegrations.tsx`)
  - UI for managing chat integration configurations
  - Tabs for Telegram, Slack, Discord, and delivery logs
  - Add/Edit/Delete configurations
  - Test message sending
  - View delivery logs

- **Store** (`/src/store/chatIntegrationsStore.ts`)
  - State management for chat integrations
  - API calls to backend commands
  - CRUD operations for each service

- **Types** (`/src/types/chatIntegrations.ts`)
  - TypeScript interfaces for configurations
  - Delivery log types
  - Rate limit status types

### Backend Modules

Located in `/src-tauri/src/notifications/`:

- **types.rs** - Core types and enums
- **telegram.rs** - Telegram Bot API integration
- **slack.rs** - Slack Incoming Webhooks
- **discord.rs** - Discord Webhook integration
- **rate_limiter.rs** - Rate limiting implementation
- **delivery_log.rs** - Delivery status logging
- **router.rs** - Notification routing engine
- **commands.rs** - Tauri commands for frontend
- **integration.rs** - Alert system integration

## Setup Instructions

### Telegram

1. Create a bot using @BotFather on Telegram
2. Get the bot token (format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)
3. Add the bot to your group or channel
4. Get the chat ID using @userinfobot (format: `-1001234567890`)
5. Add configuration in Settings → Chat Integrations → Telegram

### Slack

1. Go to your Slack workspace settings
2. Navigate to Apps → Custom Integrations → Incoming Webhooks
3. Create a new webhook for your desired channel
4. Copy the webhook URL
5. Add configuration in Settings → Chat Integrations → Slack

### Discord

1. Open Server Settings → Integrations → Webhooks
2. Create a new webhook
3. Choose the channel and copy the webhook URL
4. Optionally set a custom username
5. Add configuration in Settings → Chat Integrations → Discord

## Features

### Message Formatting

**Telegram:**
- Supports MarkdownV2 formatting
- Bold: `*text*`
- Italic: `_text_`
- Auto-escaping of special characters

**Slack:**
- Supports mrkdwn formatting
- Bold: `*text*`
- Italic: `_text_`
- Channel mentions: `#channel-name`

**Discord:**
- Supports embeds with color, fields, timestamps
- Role mentions: `<@&role_id>`
- Rich formatting with titles and descriptions

### Rate Limiting

The system implements rate limiting to comply with API restrictions:

- **Telegram:** 30 messages per minute per bot
- **Slack:** 60 messages per minute
- **Discord:** 60 messages per minute

When rate limits are hit, messages are logged with `rate_limited` status and can be retried later.

### Delivery Logs

All message deliveries are logged with:
- Service type and configuration name
- Alert ID and name (if applicable)
- Message content
- Delivery status (pending, sent, failed, rate_limited)
- Error details (if failed)
- Retry count
- Timestamp

Logs can be viewed in Settings → Chat Integrations → Logs tab.

### Alert Integration

When creating or editing price alerts, users can select chat notification channels in addition to existing channels (in_app, system, email, webhook).

The system automatically routes alerts to all enabled chat integrations when alerts are triggered.

## API Commands

### Configuration Management

- `chat_integration_get_settings` - Get all configurations
- `chat_integration_save_settings` - Save all configurations
- `chat_integration_add_telegram` - Add Telegram config
- `chat_integration_update_telegram` - Update Telegram config
- `chat_integration_delete_telegram` - Delete Telegram config
- `chat_integration_add_slack` - Add Slack config
- `chat_integration_update_slack` - Update Slack config
- `chat_integration_delete_slack` - Delete Slack config
- `chat_integration_add_discord` - Add Discord config
- `chat_integration_update_discord` - Update Discord config
- `chat_integration_delete_discord` - Delete Discord config

### Testing

- `chat_integration_test_telegram` - Send test message to Telegram
- `chat_integration_test_slack` - Send test message to Slack
- `chat_integration_test_discord` - Send test message to Discord

### Monitoring

- `chat_integration_get_delivery_logs` - Retrieve delivery logs
- `chat_integration_clear_delivery_logs` - Clear all logs
- `chat_integration_get_rate_limits` - Get current rate limit status

## Database Schema

The system uses two SQLite tables:

### chat_integrations
```sql
CREATE TABLE chat_integrations (
    service_type TEXT NOT NULL,
    config_id TEXT PRIMARY KEY,
    config_data TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
```

### delivery_logs
```sql
CREATE TABLE delivery_logs (
    id TEXT PRIMARY KEY,
    service_type TEXT NOT NULL,
    config_id TEXT NOT NULL,
    config_name TEXT NOT NULL,
    alert_id TEXT,
    alert_name TEXT,
    message TEXT NOT NULL,
    status TEXT NOT NULL,
    error TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT NOT NULL
)
```

## Error Handling

The system includes comprehensive error handling:

1. **Network Errors** - Logged with error details
2. **Rate Limiting** - Queued with rate_limited status
3. **Invalid Configurations** - Validated on save
4. **API Errors** - Captured with response details
5. **Timeouts** - 10-second timeout per request

## Best Practices

1. **Test configurations** before enabling for production alerts
2. **Monitor delivery logs** regularly for failed deliveries
3. **Respect rate limits** by not creating too many simultaneous alerts
4. **Use descriptive names** for configurations to identify them easily
5. **Keep webhooks secure** - don't share them publicly
6. **Rotate tokens** periodically for security

## Troubleshooting

### Telegram Issues

- **Bot not receiving messages:** Ensure bot is added to the group/channel
- **Invalid chat ID:** Use @userinfobot to get correct chat ID
- **Markdown errors:** Check for unescaped special characters

### Slack Issues

- **Webhook fails:** Verify webhook URL is correct
- **Messages not appearing:** Check channel permissions
- **Rate limited:** Reduce alert frequency

### Discord Issues

- **Webhook not found:** Recreate the webhook in Discord
- **Role mentions not working:** Ensure bot has mention permissions
- **Embed not displaying:** Check embed JSON structure

## Future Enhancements

Potential improvements for future versions:

- Webhook security with signature verification
- Message templates with custom formatting
- Attachment support (images, charts)
- Priority-based routing
- Advanced retry strategies
- Webhook rotation for high-volume scenarios
- Analytics dashboard for delivery metrics
