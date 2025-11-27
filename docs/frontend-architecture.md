# Frontend Architecture Guide

## Overview

Eclipse Market Pro uses a modern React 18 + TypeScript frontend with Tauri for desktop functionality. This guide explains the architectural patterns and best practices for contributing to the frontend codebase.

## Core Architecture

### Technology Stack
- **Frontend**: React 18 + TypeScript + Vite
- **State Management**: Zustand with persistence
- **Styling**: Tailwind CSS + CSS-in-JS for dynamic styles
- **Desktop Bridge**: Tauri 2.0 with Rust backend
- **Animations**: Framer Motion
- **Testing**: Vitest (unit) + Playwright (E2E)

### Project Structure
```
src/
├── components/          # Reusable UI components
├── hooks/              # Custom React hooks
├── lib/                # Utility libraries and external integrations
│   └── tauri/         # Tauri client and type definitions
├── store/              # Zustand state stores
├── pages/              # Route-level components
├── layouts/            # Layout components
└── styles/             # Global styles and theme configurations
```

## Tauri Client Architecture

### 🚨 IMPORTANT: Backend Command Invocation Pattern

**NEVER** call `invoke()` directly from components or stores. All backend communication must go through the typed Tauri client:

```typescript
// ❌ WRONG - Direct invoke usage
import { invoke } from '@tauri-apps/api/core';
const result = await invoke('wallet_get_token_balances', { address });

// ✅ CORRECT - Use typed client
import { walletCommands } from '@/lib/tauri/commands';
const response = await walletCommands.getTokenBalances(address);
if (response.success) {
  const balances = response.data;
}
```

### Command Categories

The Tauri client is organized by domain:

#### Wallet Commands (`walletCommands`)
- `getTokenBalances(address, forceRefresh?)`
- `estimateFee(recipient, amount, tokenMint?)`
- `sendTransaction(input, walletAddress)`
- `generateQR(data)`
- `generateSolanaPayQR(...)`

#### Trading Commands (`tradingCommands`)
- `init()`
- `createOrder(request)`
- `cancelOrder(orderId)`
- `getActiveOrders(walletAddress)`
- `getOrderHistory(walletAddress, limit?)`

#### AI Commands (`aiCommands`)
- `chatMessage(message, commandType?, history?)`
- `getPatternWarnings()`
- `optimizePortfolio(allocation, riskTolerance?)`

#### Portfolio Commands (`portfolioCommands`)
- `calculateAnalytics(positions)`
- `getSectorAllocation(positions)`
- `clearCache()`

### Streaming Commands

For streaming operations (like AI chat), use the streaming client:

```typescript
import { StreamingCommandManager } from '@/lib/tauri/commands';

const streamId = await StreamingCommandManager.startChatStream(
  message,
  commandType,
  history,
  (chunk) => {
    console.log('Received chunk:', chunk);
  }
);
```

## React Hooks Usage

### Custom Hooks for Tauri Commands

Use the provided hooks for consistent error handling and loading states:

```typescript
import { useTauriCommand } from '@/hooks';

const { data, isLoading, error, execute } = useTauriCommand(
  (address) => walletCommands.getTokenBalances(address),
  {
    onSuccess: (balances) => console.log('Balances loaded:', balances),
    onError: (error) => console.error('Failed to load balances:', error),
    showToastOnError: true,
    loadingId: 'wallet-balances',
    loadingMessage: 'Loading wallet balances...',
  }
);

// Execute the command
const loadBalances = () => execute('wallet-address');
```

### Streaming Commands Hook

For streaming operations like AI chat:

```typescript
import { useAIChatStream } from '@/hooks';

const { 
  messages, 
  isStreaming, 
  content, 
  sendMessage, 
  clearChat 
} = useAIChatStream({
  onComplete: (fullContent) => console.log('Chat completed:', fullContent),
  onError: (error) => console.error('Chat error:', error),
});

// Send a message
const handleSendMessage = () => {
  sendMessage('What is the market analysis?', 'market_analysis');
};
```

### Stable Callbacks

Use stable callbacks to prevent infinite re-renders:

```typescript
import { useStableCallback } from '@/hooks';

const handleClick = useStableCallback((id: string) => {
  // This function maintains stable identity
  setSelectedId(id);
});
```

## State Management

### Zustand Store Pattern

All state is managed through Zustand stores:

```typescript
// Store definition
interface AppStore {
  user: User | null;
  setUser: (user: User | null) => void;
  updateUser: (updates: Partial<User>) => void;
}

export const useAppStore = create<AppStore>((set, get) => ({
  user: null,
  setUser: (user) => set({ user }),
  updateUser: (updates) => set((state) => ({
    user: state.user ? { ...state.user, ...updates } : null
  })),
}));

// Usage in components
const { user, setUser, updateUser } = useAppStore();
```

### UI Store for Global State

Use the UI store for global loading states, toasts, and errors:

```typescript
import { useUIStore } from '@/store/uiStore';

const { 
  setLoading, 
  addToast, 
  setGlobalError, 
  setDevConsoleOpen 
} = useUIStore();

// Show loading state
setLoading('operation-id', true, 'Processing...');

// Show toast
addToast({
  type: 'success',
  title: 'Operation Complete',
  message: 'Transaction was successful',
});

// Handle global error
setGlobalError(new Error('Something went wrong'));
```

## Error Handling

### Error Boundaries

Wrap components with error boundaries:

```typescript
import { AppErrorBoundary, withErrorBoundary } from '@/components';

// Using component directly
<AppErrorBoundary onError={(error, errorInfo) => console.error('Error:', error)}>
  <MyComponent />
</AppErrorBoundary>

// Using HOC
export default withErrorBoundary(MyComponent, {
  onError: (error) => reportError(error),
});
```

### Command Error Handling

The Tauri client automatically normalizes errors. Handle them in hooks:

```typescript
const { error, execute } = useTauriCommand(
  (id) => tradingCommands.cancelOrder(id),
  {
    onError: (error) => {
      // Error is already normalized to TauriError format
      console.error('Cancel order failed:', error.message);
      if (error.code === 'ORDER_NOT_FOUND') {
        // Handle specific error codes
      }
    },
  }
);
```

## Development Tools

### Dev Console

The dev console is available in development builds:

```typescript
import { useDevConsole } from '@/hooks';

const { toggleDevConsole, isDevConsoleOpen, isDevConsoleAvailable } = useDevConsole();

// Toggle dev console (only works in development)
const handleToggleConsole = () => {
  if (isDevConsoleAvailable) {
    toggleDevConsole();
  }
};
```

Keyboard shortcuts:
- `F12` or `Ctrl+Shift+I` (Cmd+Opt+I on Mac) to toggle dev console

### Auto Setup

For automatic keyboard shortcut setup:

```typescript
import { useDevConsoleAutoSetup } from '@/hooks';

// In your root component
function App() {
  useDevConsoleAutoSetup(); // Sets up keyboard shortcuts automatically
  return <AppContent />;
}
```

## Component Patterns

### Loading States

Use the LoadingOverlay for global loading:

```typescript
import { LoadingOverlay } from '@/components';

// Manual control
<LoadingOverlay show={isLoading} message="Processing..." />

// Automatic via UI store
<LoadingOverlay />
```

For inline loading states:

```typescript
import { Spinner, Skeleton } from '@/components';

// Spinner
{isLoading && <Spinner size="sm" />}

// Skeleton placeholders
<Skeleton lines={3} height="1rem" />
<CardSkeleton />
<TableSkeleton rows={5} columns={4} />
```

### Toast Notifications

Use the toast system for user feedback:

```typescript
import { useToast } from '@/hooks';

const { success, error, warning, info } = useToast();

// Show notifications
success('Transaction Complete', 'Your order has been placed');
error('Transaction Failed', 'Insufficient balance');
warning('Market Volatility', 'Prices are changing rapidly');
info('New Feature', 'Check out the latest updates');
```

## Best Practices

### 1. Component Structure
- Keep components focused on single responsibilities
- Use TypeScript interfaces for all props
- Export components as default, types as named exports

### 2. Hook Rules
- All hooks must be called unconditionally at the top level
- Never place hooks inside try-catch blocks
- Use `useStableCallback` for event handlers passed to child components

### 3. State Management
- Keep component state local when possible
- Use Zustand for shared state across components
- Persist state when it should survive page refreshes

### 4. Error Handling
- Always handle errors from Tauri commands
- Use error boundaries for component-level error handling
- Provide user-friendly error messages

### 5. Performance
- Use `React.memo` for expensive components
- Use `useMemo` and `useCallback` for expensive computations
- Use the provided hooks which are already optimized

### 6. TypeScript
- Use strict TypeScript mode
- Define interfaces for all data structures
- Avoid `any` type - use proper typing

## Testing

### Unit Tests
Write tests for hooks and utility functions:

```typescript
import { renderHook, waitFor } from '@testing-library/react';
import { useTauriCommand } from '@/hooks/useTauriCommand';

test('should handle successful command execution', async () => {
  const { result } = renderHook(() => 
    useTauriCommand(() => Promise.resolve({ success: true, data: 'test' }))
  );

  await waitFor(() => {
    expect(result.current.data).toBe('test');
    expect(result.current.isLoading).toBe(false);
  });
});
```

### Component Tests
Test components with React Testing Library:

```typescript
import { render, screen } from '@testing-library/react';
import { AppErrorBoundary } from '@/components';

test('should render error fallback when error occurs', () => {
  const ThrowError = () => {
    throw new Error('Test error');
  };

  render(
    <AppErrorBoundary>
      <ThrowError />
    </AppErrorBoundary>
  );

  expect(screen.getByText('Something went wrong')).toBeInTheDocument();
});
```

## Contributing Guidelines

1. **Follow the patterns**: Use the established patterns for Tauri commands, state management, and error handling
2. **Type safety**: Ensure all new code is properly typed
3. **Testing**: Add tests for new hooks and components
4. **Documentation**: Update this guide when introducing new patterns
5. **Code review**: Ensure all PRs follow these architectural guidelines

## Migration Guide

If you find existing code that violates these patterns:

1. **Direct invoke calls**: Replace with typed client commands
2. **Inline error handling**: Replace with hook-based error handling
3. **Manual loading states**: Replace with UI store integration
4. **Unstable callbacks**: Replace with `useStableCallback`

Example migration:

```typescript
// Before
const [loading, setLoading] = useState(false);
const [error, setError] = useState(null);

const handleLoad = async () => {
  setLoading(true);
  try {
    const result = await invoke('get_data', { id });
    setData(result);
  } catch (err) {
    setError(err);
  } finally {
    setLoading(false);
  }
};

// After
const { data, isLoading, error, execute } = useTauriCommand(
  (id) => dataCommands.getData(id),
  {
    loadingId: 'data-load',
    showToastOnError: true,
  }
);

const handleLoad = () => execute(id);
```

This ensures consistent behavior across the application and better maintainability.