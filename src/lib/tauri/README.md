# Tauri Client

This directory contains the typed client library for communicating with the Tauri backend commands.

## Files

- `types.ts` - TypeScript type definitions for all backend command data structures
- `commands.ts` - Typed wrapper functions for all backend commands with error handling
- `index.ts` - Re-exports for easy importing

## Usage

### Importing

```typescript
import { 
  walletCommands, 
  tradingCommands, 
  aiCommands, 
  portfolioCommands,
  StreamingCommandManager 
} from '@/lib/tauri';
```

### Basic Commands

```typescript
// Wallet commands
const balancesResponse = await walletCommands.getTokenBalances('address', true);
if (balancesResponse.success) {
  const balances = balancesResponse.data;
}

// Trading commands
const orderResponse = await tradingCommands.createOrder(orderRequest);
if (orderResponse.success) {
  const order = orderResponse.data;
}

// AI commands
const chatResponse = await aiCommands.chatMessage('Hello', 'general', history);
if (chatResponse.success) {
  const response = chatResponse.data;
}
```

### Streaming Commands

```typescript
const streamId = await StreamingCommandManager.startChatStream(
  message,
  commandType,
  history,
  (chunk) => {
    console.log('Received chunk:', chunk);
  }
);

// Stop streaming
StreamingCommandManager.stopChatStream(streamId);
```

## Error Handling

All commands return an `ApiResponse<T>` object:

```typescript
interface ApiResponse<T> {
  data: T;
  success: boolean;
  error?: TauriError;
}
```

Use the provided hooks for consistent error handling:

```typescript
import { useTauriCommand } from '@/hooks';

const { data, isLoading, error, execute } = useTauriCommand(
  (address) => walletCommands.getTokenBalances(address),
  {
    onSuccess: (data) => console.log('Success:', data),
    onError: (error) => console.error('Error:', error),
    showToastOnError: true,
  }
);
```

## Available Commands

### Wallet Commands
- `getTokenBalances(address, forceRefresh?)`
- `estimateFee(recipient, amount, tokenMint?)`
- `sendTransaction(input, walletAddress)`
- `generateQR(data)`
- `generateSolanaPayQR(...)`

### Trading Commands
- `init()`
- `createOrder(request)`
- `cancelOrder(orderId)`
- `getActiveOrders(walletAddress)`
- `getOrderHistory(walletAddress, limit?)`

### AI Commands
- `chatMessage(message, commandType?, history?)`
- `getPatternWarnings()`
- `optimizePortfolio(allocation, riskTolerance?)`

### Portfolio Commands
- `calculateAnalytics(positions)`
- `getSectorAllocation(positions)`
- `clearCache()`

## Best Practices

1. **Always use the typed client** - Never call `invoke()` directly
2. **Use the provided hooks** - They handle loading states, errors, and toasts
3. **Handle success/error states** - Check `success` property before accessing `data`
4. **Use streaming for AI chat** - Use `StreamingCommandManager` for real-time responses
5. **Type safety** - All commands are fully typed with TypeScript

## Testing

The client is fully tested with Vitest. See `tests/tauri-client.test.ts` for examples.