# Zustand Stores

This directory contains Zustand 5 stores for Eclipse Market Pro. All stores use the `createBoundStore` or `createBoundStoreWithMiddleware` helper for consistent patterns, with `subscribeWithSelector` enabled by default to prevent stale snapshots.

## Architecture

- **createBoundStore.ts** - Helper that creates Zustand stores with `subscribeWithSelector` middleware enabled by default, typed hooks, and re-exports `useShallow` from `zustand/react/shallow`
- **walletStore.ts** - Wallet accounts, balances, fee estimates, send workflow (uses `subscribeWithSelector`)
- **tradingStore.ts** - Orders, drafts, optimistic updates (uses `subscribeWithSelector`)
- **portfolioStore.ts** - Positions, analytics cache, sector allocation (uses `subscribeWithSelector`)
- **aiStore.ts** - Chat history, pattern warnings, streaming metadata (uses `subscribeWithSelector`)
- **uiStore.ts** - Theme, panel visibility, dev console toggle (uses `subscribeWithSelector` + `persist`)
- **themeStore.ts** - Custom theme management (uses `subscribeWithSelector` + `persist`)
- **accessibilityStore.ts** - Accessibility preferences (uses `subscribeWithSelector` + `persist`)

## Store Creation Helpers

### `createBoundStore<T>(initializer)`

Creates a bound Zustand store with `subscribeWithSelector` middleware enabled by default. This ensures all stores can handle selective subscriptions and prevents stale snapshots.

**Use this for:** Standard stores that don't need persistence.

```typescript
import { createBoundStore } from './createBoundStore';

interface MyStoreState {
  count: number;
  increment: () => void;
}

const storeResult = createBoundStore<MyStoreState>((set, get) => ({
  count: 0,
  increment: () => set(state => ({ count: state.count + 1 })),
}));

export const useMyStore = storeResult.useStore;
export const myStore = storeResult.store;
```

### `createBoundStoreWithMiddleware<T>()(creator)`

Creates a bound Zustand store with custom middleware. Use this when you need to compose additional middleware like `persist` or `devtools`. **Important:** `subscribeWithSelector` is NOT automatically applied - you must include it if needed.

**Use this for:** Stores that need persistence (UI preferences, settings).

```typescript
import { persist, createJSONStorage } from 'zustand/middleware';
import { subscribeWithSelector } from 'zustand/middleware';
import { createBoundStoreWithMiddleware } from './createBoundStore';
import { getPersistentStorage } from './storage';

const storeResult = createBoundStoreWithMiddleware<MyStoreState>()(
  subscribeWithSelector(
    persist(
      (set, get) => ({
        theme: 'dark',
        setTheme: (theme) => set({ theme }),
      }),
      {
        name: 'my-store',
        storage: createJSONStorage(getPersistentStorage),
      }
    )
  )
);

export const useMyStore = storeResult.useStore;
export const myStore = storeResult.store;
```

## Usage

### Basic Store Access

```typescript
import { useWalletStore } from '@/store';

function MyComponent() {
  // Get entire state (will re-render on any change)
  const state = useWalletStore();
  
  // Select specific state (will only re-render when this changes)
  const accounts = useWalletStore(state => state.accounts);
  
  // Get an action
  const fetchBalances = useWalletStore(state => state.fetchBalances);
}
```

### Using Shallow Comparison for Multiple Fields

When selecting multiple fields, use `useShallow` to prevent unnecessary re-renders:

```typescript
import { useWalletStore, useShallow } from '@/store';
import { useCallback } from 'react';

function MyComponent() {
  // CORRECT: Use shallow comparison for object/array selectors
  const selector = useCallback(
    (state: ReturnType<typeof useWalletStore.getState>) => ({
      accounts: state.accounts,
      activeAccount: state.activeAccount,
      fetchBalances: state.fetchBalances,
    }),
    []
  );
  
  const { accounts, activeAccount, fetchBalances } = useWalletStore(selector, useShallow);
  
  // Component only re-renders if accounts, activeAccount, or fetchBalances change
}
```

### Using Convenience Hooks

Each store exports convenience hooks for common selections:

```typescript
import {
  useWalletBalances,
  useActiveAccount,
  useAddressBook,
  useWalletStatus,
  useSendWorkflow,
} from '@/store';

function MyComponent() {
  const balances = useWalletBalances('wallet-address');
  const activeAccount = useActiveAccount();
  const contacts = useAddressBook();
  const { isLoading, error } = useWalletStatus();
  const sendWorkflow = useSendWorkflow();
}
```

## Store Details

### WalletStore

Manages wallet accounts, token balances, and transaction workflows.

**State:**
- `accounts`: Array of wallet accounts
- `activeAccount`: Currently selected account
- `balances`: Token balances per address
- `feeEstimates`: Cached fee estimates
- `addressBook`: Saved contacts
- `sendWorkflow`: Multi-step send transaction state
- `swapHistory`: Token swap history

**Key Actions:**
- `fetchBalances(address, forceRefresh?)` - Fetch token balances
- `estimateFee(recipient, amount, tokenMint?)` - Estimate transaction fee
- `sendTransaction(input, walletAddress)` - Send transaction
- `startSendWorkflow(input)` - Begin send flow

**Convenience Hooks:**
- `useWalletBalances(address?)` - Get token balances for address
- `useActiveAccount()` - Get active account
- `useAddressBook()` - Get address book contacts
- `useWalletStatus()` - Get loading/error state
- `useSendWorkflow()` - Get send workflow state
- `useSwapHistory()` - Get swap history

### TradingStore

Manages orders with optimistic updates and draft support. Uses `subscribeWithSelector` for real-time order updates.

**State:**
- `isInitialized`: Whether trading module is initialized
- `activeOrders`: Current active orders
- `orderHistory`: Past orders
- `drafts`: Saved order drafts
- `optimisticOrders`: Orders being created (optimistic UI)

**Key Actions:**
- `initialize()` - Initialize trading module (call once on app start)
- `createOrder(request)` - Create order with optimistic update
- `cancelOrder(orderId)` - Cancel order
- `getActiveOrders(walletAddress)` - Fetch active orders
- `addDraft(request)` - Save order as draft
- `handleOrderUpdate(update)` - Handle real-time order updates

**Convenience Hooks:**
- `useActiveOrders()` - Get active orders
- `useOrderDrafts()` - Get order drafts
- `useCombinedOrders()` - Get optimistic + active orders

### PortfolioStore

Manages portfolio positions and analytics with caching.

**State:**
- `positions`: Current positions
- `analyticsCache`: Cached analytics per wallet (5min TTL)
- `sectorAllocations`: Asset allocation by sector
- `concentrationAlerts`: Risk concentration warnings
- `totalValue`, `totalPnl`, `totalPnlPercent`: Portfolio totals

**Key Actions:**
- `setPositions(positions)` - Update positions
- `fetchAnalytics(walletAddress, forceRefresh?)` - Fetch analytics (cached)
- `fetchSectorAllocations(walletAddress)` - Fetch sector breakdown
- `refreshPortfolio(walletAddress)` - Refresh all portfolio data

**Convenience Hooks:**
- `usePositions()` - Get positions
- `useSectorAllocations()` - Get sector allocations
- `useConcentrationAlerts()` - Get concentration alerts
- `usePortfolioTotals()` - Get total value/PnL
- `usePortfolioStatus()` - Get loading/error state
- `useAnalyticsCache()` - Get analytics cache

### AiStore

Manages AI chat, pattern warnings, and streaming responses. Uses `subscribeWithSelector` for streaming updates.

**State:**
- `chatHistory`: Conversation messages
- `patternWarnings`: Active pattern warnings
- `streamingMetadata`: Current streaming state
- `currentResponse`: Streaming response in progress
- `isStreaming`: Whether AI is streaming

**Key Actions:**
- `sendMessage(message, commandType?)` - Send message (non-streaming)
- `sendMessageStream(message, commandType?)` - Send message with streaming
- `fetchPatternWarnings()` - Get pattern warnings
- `optimizePortfolio(holdings)` - Request portfolio optimization

**Convenience Hooks:**
- `useChatHistory()` - Get chat history
- `usePatternWarnings()` - Get pattern warnings
- `useStreamingStatus()` - Get streaming state

### UiStore

Manages UI preferences with persistence. Uses `subscribeWithSelector` + `persist` middleware.

**State:**
- `theme`: Current theme ('dark' | 'light' | 'auto')
- `panelVisibility`: Visibility of each panel
- `devConsoleVisible`: Dev console visibility
- `sidebarCollapsed`: Sidebar state
- `notificationsEnabled`, `soundEnabled`, `animationsEnabled`: User preferences
- `toasts`: Active toast notifications

**Key Actions:**
- `setTheme(theme)` - Change theme
- `togglePanel(panel)` - Toggle panel visibility
- `toggleDevConsole()` - Toggle dev console
- `toggleSidebar()` - Toggle sidebar
- `addToast(toast)` - Show toast notification

**Convenience Hooks:**
- `usePanelVisibility(panel)` - Get panel visibility
- `useDevConsole()` - Get dev console state
- `useTheme()` - Get current theme
- `useToasts()` - Get active toasts

### ThemeStore

Manages custom theme definitions with persistence. Uses `subscribeWithSelector` + `persist` middleware.

**State:**
- `activeThemeId`: Currently active theme ID
- `currentTheme`: Current theme definition
- `customThemes`: User-created custom themes

**Key Actions:**
- `setActiveTheme(id)` - Switch to theme
- `createCustomTheme(name, colors)` - Create custom theme
- `updateCustomTheme(id, colors)` - Update custom theme
- `deleteCustomTheme(id)` - Delete custom theme
- `exportTheme(id)` - Export theme as JSON
- `importTheme(payload)` - Import theme from JSON

**Convenience Hooks:**
- `useCurrentTheme()` - Get current theme
- `useCustomThemes()` - Get custom themes list

### AccessibilityStore

Manages accessibility preferences with persistence. Uses `subscribeWithSelector` + `persist` middleware.

**State:**
- `fontScale`: Font size multiplier (1-2)
- `highContrastMode`: High contrast mode enabled
- `reducedMotion`: Reduced motion enabled
- `screenReaderOptimizations`: Screen reader optimizations
- `keyboardNavigationHints`: Keyboard navigation hints
- `focusIndicatorEnhanced`: Enhanced focus indicators

**Key Actions:**
- `setFontScale(value)` - Set font scale
- `toggleHighContrast()` - Toggle high contrast
- `toggleReducedMotion()` - Toggle reduced motion
- `resetToDefaults()` - Reset all to defaults

**Convenience Hooks:**
- `useFontScale()` - Get font scale
- `useHighContrastMode()` - Get high contrast mode
- `useReducedMotion()` - Get reduced motion

## Best Practices

### 1. Always Memoize Selectors

Always memoize selector functions with `useCallback` to prevent "getSnapshot should be cached" warnings:

```typescript
const selector = useCallback(
  (state: ReturnType<typeof useStore.getState>) => ({
    data: state.data,
    action: state.action,
  }),
  []
);
const { data, action } = useStore(selector, useShallow);
```

### 2. Use Shallow Comparison for Objects/Arrays

When selecting multiple fields, always use `useShallow`:

```typescript
// ❌ BAD: Will cause infinite re-renders
const { field1, field2 } = useStore(state => ({ field1: state.field1, field2: state.field2 }));

// ✅ GOOD: Shallow comparison prevents unnecessary re-renders
const selector = useCallback(state => ({ field1: state.field1, field2: state.field2 }), []);
const { field1, field2 } = useStore(selector, useShallow);
```

### 3. Place Hooks at Top Level

Never put hooks inside try-catch or conditional statements:

```typescript
// ❌ BAD: Hook inside try-catch violates React Hook Rules
try {
  const data = useStore(state => state.data);
} catch (error) {
  // ...
}

// ✅ GOOD: Hook at top level, try-catch around usage
const data = useStore(state => state.data);
try {
  // use data
} catch (error) {
  // ...
}
```

### 4. Use Convenience Hooks

Prefer convenience hooks for common selections:

```typescript
// ❌ Less convenient
const balances = useWalletStore(state => 
  state.balances[address] || []
);

// ✅ More convenient
const balances = useWalletBalances(address);
```

### 5. Complete useEffect Dependencies

Always include ALL dependencies in useEffect arrays:

```typescript
// ❌ BAD: Missing dependencies causes stale closures
useEffect(() => {
  fetchBalances(activeAccount.publicKey);
}, []); // Missing fetchBalances, activeAccount

// ✅ GOOD: All dependencies included
useEffect(() => {
  if (activeAccount) {
    fetchBalances(activeAccount.publicKey);
  }
}, [activeAccount, fetchBalances]);
```

### 6. Handle Async Errors

Always handle errors from async actions:

```typescript
const sendTransaction = useWalletStore(state => state.sendTransaction);

try {
  await sendTransaction(input, address);
} catch (error) {
  console.error('Transaction failed:', error);
  // Show error to user
}
```

## Middleware Composition

### subscribeWithSelector

All stores have `subscribeWithSelector` enabled by default through `createBoundStore`. This middleware:
- Enables selective subscriptions to specific state slices
- Prevents stale snapshots in components
- Required for streaming updates and real-time data

### persist

Used for stores that need to persist state across sessions:
- `uiStore` - UI preferences
- `themeStore` - Custom themes
- `accessibilityStore` - Accessibility settings

Always use `createJSONStorage(getPersistentStorage)` for Tauri secure storage.

### Middleware Order

When composing middleware, order matters:

```typescript
// ✅ CORRECT: subscribeWithSelector wraps persist
subscribeWithSelector(
  persist(
    (set, get) => ({ /* state */ }),
    { /* persist config */ }
  )
)

// ❌ WRONG: persist wrapping subscribeWithSelector breaks selective subscriptions
persist(
  subscribeWithSelector(
    (set, get) => ({ /* state */ })
  ),
  { /* persist config */ }
)
```

## Testing

Store tests are located in `tests/stores/`. Each store has comprehensive tests covering:
- State updates
- Async actions
- Optimistic updates
- Error handling
- State reset
- Persistence (for persisted stores)

Run store tests:
```bash
npm test -- tests/stores/
```

## Type Safety

All stores are fully typed. Import types from `src/types/`:

```typescript
import type { Order, TokenBalance, ChatMessage } from '@/types';
```

## Persistence

Only UI-related stores use persistence:
- `uiStore` - UI preferences
- `themeStore` - Custom themes
- `accessibilityStore` - Accessibility settings

All persistence uses Tauri's secure storage via `getPersistentStorage()` helper.

## Migration Guide

If you're updating an existing store to use the new helpers:

### Standard Store (no persistence)

```typescript
// BEFORE
import { create } from 'zustand';

export const useMyStore = create<MyState>((set, get) => ({
  // state
}));

// AFTER
import { createBoundStore } from './createBoundStore';

const storeResult = createBoundStore<MyState>((set, get) => ({
  // state
}));

export const useMyStore = storeResult.useStore;
export const myStore = storeResult.store;
```

### Persisted Store

```typescript
// BEFORE
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { getPersistentStorage } from './storage';

export const useMyStore = create<MyState>()(
  persist(
    (set, get) => ({ /* state */ }),
    { name: 'my-store', storage: createJSONStorage(getPersistentStorage) }
  )
);

// AFTER
import { persist, createJSONStorage } from 'zustand/middleware';
import { subscribeWithSelector } from 'zustand/middleware';
import { createBoundStoreWithMiddleware } from './createBoundStore';
import { getPersistentStorage } from './storage';

const storeResult = createBoundStoreWithMiddleware<MyState>()(
  subscribeWithSelector(
    persist(
      (set, get) => ({ /* state */ }),
      { name: 'my-store', storage: createJSONStorage(getPersistentStorage) }
    )
  )
);

export const useMyStore = storeResult.useStore;
export const myStore = storeResult.store;
```

### Store with subscribeWithSelector

```typescript
// BEFORE
import { createBoundStore } from './createBoundStore';
import { subscribeWithSelector } from 'zustand/middleware';

const storeResult = createBoundStore<MyState>((set, get, api) =>
  subscribeWithSelector(storeInitializer)(set, get, api)
);

// AFTER
import { createBoundStore } from './createBoundStore';

// subscribeWithSelector is now included by default!
const storeResult = createBoundStore<MyState>((set, get) => ({
  // state
}));
```

## Anti-Patterns to Avoid

### ❌ Don't use raw `create` from Zustand

Always use `createBoundStore` or `createBoundStoreWithMiddleware`:

```typescript
// ❌ BAD
import { create } from 'zustand';
export const useMyStore = create<MyState>((set) => ({ /* ... */ }));

// ✅ GOOD
import { createBoundStore } from './createBoundStore';
const storeResult = createBoundStore<MyState>((set) => ({ /* ... */ }));
export const useMyStore = storeResult.useStore;
```

### ❌ Don't forget useShallow for object selectors

```typescript
// ❌ BAD: Creates new object on every render
const { a, b } = useStore(state => ({ a: state.a, b: state.b }));

// ✅ GOOD: Shallow comparison
const selector = useCallback(state => ({ a: state.a, b: state.b }), []);
const { a, b } = useStore(selector, useShallow);
```

### ❌ Don't skip selector memoization

```typescript
// ❌ BAD: New selector function on every render
const data = useStore(state => ({ a: state.a, b: state.b }), useShallow);

// ✅ GOOD: Memoized selector
const selector = useCallback(state => ({ a: state.a, b: state.b }), []);
const data = useStore(selector, useShallow);
```

## Summary

- **All stores** use `subscribeWithSelector` by default (via `createBoundStore` or included in `createBoundStoreWithMiddleware`)
- **Persisted stores** (ui, theme, accessibility) use `createBoundStoreWithMiddleware` with both `subscribeWithSelector` and `persist`
- **Components** always use memoized selectors + `useShallow` for object/array selections
- **Convenience hooks** simplify common selection patterns
- **Type safety** is enforced throughout

This architecture ensures optimal performance, prevents stale snapshots, and provides a consistent developer experience across all stores.