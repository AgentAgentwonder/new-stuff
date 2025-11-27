# AI Assistant Core

The AI Assistant integrates Claude and GPT-4 models with function calling support to help traders analyze markets, manage portfolios, execute trades, and configure alerts directly from chat.

## Configuration

1. Obtain API credentials from your provider(s):
   - **Anthropic Claude** – https://console.anthropic.com/
   - **OpenAI GPT-4** – https://platform.openai.com/account/api-keys
2. Launch the desktop app and open **Settings → AI Assistant** (or equivalent UI entry point).
3. Provide the API key and select the provider (`claude` or `gpt4`). Keys are encrypted using the existing keystore and stored locally.
4. Optional: pre-configure keys in `.env` before building the app by setting `CLAUDE_API_KEY`, `OPENAI_API_KEY`, and `LLM_PROVIDER` (see `.env.example`). The Settings UI can read and persist these values into the keystore.

> Keys are encrypted using the keystore master key. The assistant is disabled until a valid key is provided.

## Conversation Storage

Conversations, messages, and usage statistics are persisted inside `conversations.db` (SQLite) under the app data directory. The schema includes:

- `conversations(id, user_id, context, created_at, updated_at)`
- `messages(id, conversation_id, role, content, timestamp)`
- `usage_stats(id, user_id, requests_count, tokens_used, timestamp)`

Conversation history is automatically replayed on each request (up to 20 messages), ensuring proper context.

## Usage Throttling

The assistant enforces sensible limits to protect provider quotas:

- **Requests per hour**: 60
- **Tokens per day**: 100,000 (estimated per call)

When limits are exceeded the API returns `Rate limit exceeded. Please try again later.`

## Registered Functions (Tool Schema)

The LLM can call the following functions via the OpenAI/Anthropic tool calling interface:

| Name | Purpose | Parameters |
| ---- | ------- | ---------- |
| `execute_trade` | Execute spot trades | `{ action: "buy" | "sell", token_address: string, amount: number, slippage?: number }` |
| `create_alert` | Create price alerts | `{ token_address: string, condition: "above" | "below", price: number, message?: string }` |
| `get_portfolio_analytics` | Portfolio analytics snapshot | `{ timeframe?: "24h" | "7d" | "30d" | "all", include_breakdown?: boolean }` |
| `get_token_info` | Token data & risk score | `{ token_address: string, include_risk?: boolean }` |
| `search_tokens` | Symbol/name search | `{ query: string, limit?: number }` |

Function execution is routed through existing trading, alert, and analytics subsystems. Responses from tool calls should be returned as structured JSON for deterministic handling on the frontend.

## Tauri Commands

The backend exposes the following commands for the frontend:

- `ai_chat(userId, request)` → Execute a chat turn and optionally trigger function calls.
- `ai_get_conversations(userId)` → Retrieve recent conversations for the user.
- `ai_delete_conversation(conversationId)` → Remove a conversation and its messages.
- `ai_get_usage_stats(userId)` → Inspect current request/token usage.
- `ai_set_api_key(provider, apiKey)` → Persist credentials and hot-reload the assistant.
- `ai_is_configured()` → Check if the assistant is ready for use.

Errors from the provider are surfaced with descriptive messages and the assistant falls back to local handling if the remote call fails.

## Testing

Unit tests in `src-tauri/src/ai.rs` cover:

- Function registration and schema validation
- Usage throttling edge cases
- Serialization/deserialization of key data structures

HTTP calls are mocked at the transport layer to ensure deterministic behavior without hitting live APIs.

## Security Considerations

- API keys are encrypted using the keystore; plaintext values are zeroized after use.
- Conversation data is stored locally; sensitive content should be redacted before logging or exporting.
- Throttle limits can be adjusted by updating the `UsageThrottle` configuration.
