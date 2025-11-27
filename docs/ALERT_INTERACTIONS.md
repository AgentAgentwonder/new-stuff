# Alert Interactions Feature

## Overview

This feature enhances the alert system with comprehensive transaction details, address management, and chart integration as specified in Phase 4 Tasks 4.10–4.12.

## Components

### 1. Enhanced Alert Notifications

**Location**: `src/components/alerts/EnhancedAlertNotification.tsx`

Displays rich alert notifications with:
- Full wallet addresses with copy functionality
- Explorer links (Solscan integration)
- QR code generation for addresses
- Address nicknames and labels
- Known address highlighting
- Token details (amount, USD value, fees)
- Transaction execution details (slippage, execution time)
- Similar opportunities suggestions

### 2. Alert Notification Container

**Location**: `src/components/alerts/AlertNotificationContainer.tsx`

Manages the display of multiple alert notifications with:
- Maximum 3 visible notifications
- Automatic dismissal
- Chart navigation integration

### 3. Alert Chart Modal

**Location**: `src/components/alerts/AlertChartModal.tsx`

Displays chart view focused on alert timestamp with:
- Real-time chart integration
- Quick trade button
- Alert context information
- Timestamp highlighting

## State Management

### Alert Store

**Location**: `src/store/alertStore.ts`

Enhanced with:
- `enhancedNotifications`: Array of rich alert notifications
- `addEnhancedNotification()`: Add new enhanced notification
- `dismissNotification()`: Dismiss notification by ID

### Address Label Store

**Location**: `src/store/addressLabelStore.ts`

New store for managing address labels:
- `labels`: Array of address labels
- `getLabel()`: Get label for address
- `addLabel()`: Add or update address label
- `updateLabel()`: Update existing label
- `removeLabel()`: Remove label
- `isKnownAddress()`: Check if address is known

## Types

### Enhanced Alert Notification

**Location**: `src/types/alertNotifications.ts`

```typescript
interface EnhancedAlertNotification {
  alertId: string;
  alertName: string;
  symbol: string;
  currentPrice: number;
  priceChange24h?: number;
  priceChange7d?: number;
  conditionsMet: string;
  triggeredAt: string;
  transaction?: TransactionDetails;
  contextMessage?: string;
  similarOpportunities?: SimilarOpportunity[];
}
```

### Transaction Details

```typescript
interface TransactionDetails {
  signature: string;
  timestamp: string;
  blockTime: number;
  tokenSymbol: string;
  tokenMint: string;
  amount: number;
  usdValue: number;
  fee: number;
  feeUsd?: number;
  executionPrice?: number;
  slippage?: number;
  expectedPrice?: number;
  fromAddress: string;
  toAddress: string;
  fromLabel?: string;
  toLabel?: string;
  fromEns?: string;
  fromSns?: string;
  toEns?: string;
  toSns?: string;
  fromKnownAddress?: boolean;
  toKnownAddress?: boolean;
}
```

### Address Label

```typescript
interface AddressLabel {
  address: string;
  label: string;
  nickname?: string;
  isKnown: boolean;
  category?: 'exchange' | 'whale' | 'protocol' | 'custom';
  addedAt: string;
}
```

## Hooks

### useAlertNotifications

**Location**: `src/hooks/useAlertNotifications.ts`

Enhanced to handle:
- Listen for `alert_triggered` events
- Convert basic alerts to enhanced notifications
- Add notifications to store
- Show system notifications

## Integration with App

The alert notification system is integrated into `App.tsx`:

1. `AlertNotificationContainer` renders at bottom-right
2. Clicking "View Chart" opens `AlertChartModal`
3. Chart modal displays real-time price data
4. Quick trade button navigates to trading page

## Features

### 1. Full Address Details

- Display complete wallet addresses
- Copy button for easy clipboard access
- Shortened display with full address on hover
- ENS/SNS resolution support

### 2. Explorer Links

- Direct links to Solscan for:
  - Transaction signatures
  - Wallet addresses
  - Token contracts

### 3. QR Codes

- Generate QR codes for any address
- Display in modal overlay
- Easy sharing for mobile devices

### 4. Address Nicknames

- Add custom labels to addresses
- Store in persistent storage
- Display labels in all alerts
- Highlight known addresses

### 5. Transaction Details

For transaction-based alerts:
- Token amount and USD value
- Transaction fees
- Execution price
- Slippage percentage
- Expected vs actual price
- Execution timestamp

### 6. Chart Integration

- Click alert to open chart
- Chart focuses on alert timestamp
- Highlighted reference point on chart
- Quick trade button for immediate action

### 7. Similar Opportunities

- Display related tokens
- Match reason explanation
- Click to open their charts
- Price change indicators

## Testing

**Location**: `src/__tests__/alertInteractions.test.ts`

Test coverage includes:
- Adding enhanced notifications
- Notification limit (max 3)
- Dismissing notifications
- Transaction details formatting
- Similar opportunities
- Address label management
- Known address identification
- Chart navigation integration

## Backend Integration

The backend should emit enhanced `alert_triggered` events with:

```rust
#[derive(Serialize)]
struct AlertTriggerEvent {
    alert_id: String,
    alert_name: String,
    symbol: String,
    current_price: f64,
    conditions_met: String,
    triggered_at: String,
    price_change_24h: Option<f64>,
    price_change_7d: Option<f64>,
    transaction: Option<TransactionDetails>,
    context_message: Option<String>,
    similar_opportunities: Option<Vec<SimilarOpportunity>>,
}
```

## Usage Example

```typescript
// In backend, emit enhanced alert
app.emit("alert_triggered", {
  alertId: "alert-123",
  alertName: "SOL Price Alert",
  symbol: "SOL",
  currentPrice: 150.5,
  priceChange24h: 5.2,
  conditionsMet: "Price above $150",
  triggeredAt: new Date().toISOString(),
  transaction: {
    signature: "5j7s8VUL...",
    timestamp: new Date().toISOString(),
    blockTime: Date.now(),
    tokenSymbol: "SOL",
    tokenMint: "So11111111...",
    amount: 10,
    usdValue: 1505,
    fee: 0.000005,
    fromAddress: "7UX2i7...",
    toAddress: "9WzDXw...",
    fromLabel: "My Wallet",
    toKnownAddress: true,
  },
  contextMessage: "Large transfer detected from known whale wallet",
  similarOpportunities: [
    {
      symbol: "USDC",
      mint: "EPjFWdd5...",
      currentPrice: 1.0,
      priceChange24h: 0.1,
      matchReason: "Similar volume pattern",
    }
  ]
});
```

## Future Enhancements

1. **ENS/SNS Resolution**: Integrate with ENS/SNS services to resolve domain names
2. **Push Notifications**: Add mobile push notification support
3. **Alert History**: Store and display alert history
4. **Custom Alert Actions**: Allow users to define custom actions per alert
5. **Alert Templates**: Pre-configured alert templates for common scenarios
6. **Multi-Chain Support**: Extend to other blockchains beyond Solana
