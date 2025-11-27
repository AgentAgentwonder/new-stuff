# API Settings Hub - Setup Guide

## Overview

The API Settings Hub provides a secure, centralized interface for managing API keys and service connections for the Eclipse Market Pro application. This feature enables users to configure their own API keys for various services or fallback to default developer keys.

## Supported Services

### 1. Helius
- **Purpose**: Solana RPC and enhanced APIs
- **Endpoint**: `https://api.helius.xyz`
- **Features**: Account balance checks, transaction history, enhanced RPC
- **Get API Key**: [https://helius.xyz](https://helius.xyz)

### 2. Birdeye
- **Purpose**: Market data and token analytics
- **Endpoint**: `https://public-api.birdeye.so`
- **Features**: Token prices, trading volume, market data
- **Get API Key**: [https://birdeye.so](https://birdeye.so)

### 3. Jupiter
- **Purpose**: DEX aggregation and swap routing
- **Endpoint**: `https://quote-api.jup.ag/v6`
- **Features**: Optimal swap routes, price quotes, token swaps
- **Get API Key**: [https://jup.ag](https://jup.ag) (optional for basic usage)

### 4. Solana RPC
- **Purpose**: Direct blockchain interaction
- **Default**: `https://api.mainnet-beta.solana.com`
- **Features**: Transaction submission, account queries, program interaction
- **Custom RPC Providers**: QuickNode, Alchemy, Helius, GenesysGo

### 5. Twitter / X
- **Purpose**: Social sentiment and influencer tracking
- **Endpoint**: `https://api.twitter.com/2`
- **Features**: Recent tweet search, influencer monitoring, mention aggregation
- **Credentials**: Bearer token (OAuth 2.0) stored securely in the keystore
- **Get API Access**: [https://developer.twitter.com/](https://developer.twitter.com/)
- **Rate Limits**: Recent search allows ~450 requests per 15 minute window (subject to account tier)

### 6. Reddit
- **Purpose**: Community monitoring for crypto subreddits
- **Endpoint**: `https://www.reddit.com/r/<subreddit>/search.json`
- **Features**: Keyword tracking, new post ingestion, sentiment snapshots
- **Credentials**: No API key required for read-only JSON endpoints (authenticated keys recommended for higher throughput)
- **Rate Limits**: Reddit enforces adaptive limits via `x-ratelimit-*` headers; default client obeys limits automatically

## Security Features

### Encryption
- All API keys are encrypted using AES-256-GCM encryption
- Keys are stored in the system keychain using OS-level security
- Argon2id key derivation for enhanced security
- Memory is zeroed after use to prevent key exposure

### Secure Storage
- Keys stored in OS keychain (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- Encrypted keystore file for metadata
- No plain-text storage of sensitive credentials
- Automatic key rotation support

## Features

### 1. API Key Management
- Add/Update/Remove custom API keys per service
- Set expiry dates for key rotation tracking
- Toggle between custom and default keys
- Secure password-protected storage

### 2. Connection Testing
- Test connection to each service
- Display connection latency
- Show current rate limit status
- Error reporting and diagnostics

### 3. Status Indicators
- **Green Checkmark**: Service connected successfully
- **Red X**: Connection failed or not configured
- **Custom Badge**: Using custom API keys
- **Default Badge**: Using shared developer keys

### 4. Rate Limit Monitoring
- Display current rate limit usage
- Track remaining API calls
- Show reset time for limits
- Automatic tracking per service

### 5. Key Expiry Reminders
- Set expiry dates when saving keys
- Warning when key expires in < 30 days
- Critical alert when key expires in < 7 days
- Display days until expiry

### 6. Fallback System
- Automatic fallback to default keys
- Toggle between user/developer keys
- Fair usage disclosure for shared keys
- No service interruption

## Usage Guide

### Adding a New API Key

1. Navigate to **Settings > API Configuration**
2. Locate the service card (Helius, Birdeye, Jupiter, or Solana RPC)
3. Enter your API key in the secure input field
4. (Optional) Set an expiry date for rotation reminders
5. Click **Save Key**
6. Test the connection using the refresh button

### Testing a Connection

1. Click the **Refresh** icon on any service card
2. Wait for the test to complete
3. View the results:
   - Success: Latency and rate limit info displayed
   - Failure: Error message with details
4. Connection status updates automatically

### Using Default Keys

1. Check the **Use Default Keys** toggle on any service
2. Review the warning about shared rate limits
3. The service will use developer-provided fallback keys
4. Uncheck to switch back to custom keys

### Monitoring Rate Limits

- Rate limit information appears automatically after connection tests
- Shows: `Remaining / Total` calls
- Updates in real-time during usage
- Reset time displayed in local timezone

### Key Rotation

1. When a key is near expiry, a warning appears
2. Orange warning: < 30 days until expiry
3. Red alert: < 7 days until expiry
4. Update the key and set a new expiry date
5. Old key is securely overwritten

## Configuration

### Default Keys

Default keys are embedded in the application for testing purposes:

```rust
const DEFAULT_HELIUS_KEY: &str = "YOUR_HELIUS_KEY_HERE";
const DEFAULT_BIRDEYE_KEY: &str = "YOUR_BIRDEYE_KEY_HERE";
const DEFAULT_JUPITER_KEY: &str = "YOUR_JUPITER_KEY_HERE";
const DEFAULT_RPC_ENDPOINT: &str = "https://api.mainnet-beta.solana.com";
```

**Important**: Replace these with actual keys before deploying to production.

### Rate Limit Headers

The system automatically extracts rate limit information from these headers:
- `x-ratelimit-limit` or `ratelimit-limit`
- `x-ratelimit-remaining` or `ratelimit-remaining`
- `x-ratelimit-reset` or `ratelimit-reset`

## Backend API

### Tauri Commands

#### `save_api_key`
Save or update an API key for a service.

```typescript
invoke('save_api_key', {
  service: 'helius',
  apiKey: 'your-api-key',
  expiryDate: '2024-12-31T23:59:59Z' // Optional
});
```

#### `remove_api_key`
Remove a custom API key and fallback to defaults.

```typescript
invoke('remove_api_key', {
  service: 'birdeye'
});
```

#### `set_use_default_key`
Toggle between custom and default keys.

```typescript
invoke('set_use_default_key', {
  service: 'jupiter',
  useDefault: true
});
```

#### `test_api_connection`
Test connection to a service.

```typescript
const result = await invoke('test_api_connection', {
  service: 'helius'
});
// Returns: ConnectionTestResult
```

#### `get_api_status`
Get status of all configured services.

```typescript
const status = await invoke('get_api_status');
// Returns: ApiStatus with all service statuses
```

## Best Practices

### Security
1. Never commit API keys to version control
2. Rotate keys regularly (every 90 days recommended)
3. Use custom keys for production environments
4. Monitor rate limits to prevent service disruption
5. Set expiry dates for rotation reminders

### Performance
1. Test connections before heavy usage
2. Monitor rate limits during high-traffic periods
3. Use custom RPC endpoints for better performance
4. Cache results when appropriate
5. Implement exponential backoff for retries

### Reliability
1. Keep backup keys ready for rotation
2. Test connections regularly
3. Monitor expiry dates
4. Use fallback keys during key rotation
5. Set up alerts for connection failures

## Troubleshooting

### Connection Test Fails
- Verify the API key is correct
- Check if the service is experiencing downtime
- Ensure network connectivity
- Verify rate limits haven't been exceeded
- Try using default keys temporarily

### Rate Limit Exceeded
- Wait for the reset time
- Consider upgrading your API plan
- Temporarily use default keys (if available)
- Implement request throttling
- Cache responses when possible

### Key Not Saving
- Verify the key format is correct
- Check system keychain permissions
- Ensure sufficient disk space
- Review application logs for errors
- Try restarting the application

### Expiry Warnings
- Update the key before expiry
- Set a new expiry date
- Test the new key immediately
- Keep a backup key ready
- Document key rotation schedule

## Development

### Adding a New Service

1. Add service constant to `api_config.rs`:
```rust
const KEY_NEW_SERVICE: &str = "api_key_newservice";
```

2. Update the key mapping in command functions

3. Implement connection test function:
```rust
async fn test_newservice_connection(api_key: &str) -> Result<(u16, Option<RateLimitInfo>), String>
```

4. Add service to frontend status interface

5. Create service card in `ApiSettings.tsx`

### Testing

Run the test suite:
```bash
cd src-tauri
cargo test api_config_tests
```

### Logging

The system logs all API operations:
- Key storage/retrieval
- Connection tests
- Rate limit updates
- Error conditions

Check logs in the application data directory.

## Support

For issues or questions:
1. Check the application logs
2. Verify service status pages
3. Review API documentation
4. Test with default keys
5. Contact support with error details

## Version History

### v1.0.0 (Current)
- Initial release
- Support for Helius, Birdeye, Jupiter, and Solana RPC
- Secure key storage with encryption
- Connection testing and monitoring
- Rate limit tracking
- Key expiry reminders
- Fallback to default keys
