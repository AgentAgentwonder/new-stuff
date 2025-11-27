# Smart Alerts System Guide

## Overview

The Smart Alerts Builder is a comprehensive rule-engine powered alert system that supports complex AND/OR conditions, time windows, whale triggers, and auto-trade actions. It provides a visual drag-drop builder, templates library, team sharing, and safe dry-run simulations.

## Features

### 1. **Advanced Rule Engine**

The rule engine supports sophisticated alert conditions with nested logic:

- **Complex AND/OR Logic**: Build multi-level conditional trees
- **Time Windows**: Restrict alerts to specific time periods
- **Whale Triggers**: Monitor large wallet movements
- **Auto-Trade Actions**: Execute trades automatically when conditions are met
- **Dry-Run Simulations**: Test rules safely before enabling

### 2. **Condition Types**

#### Basic Price Conditions
- **Price Above**: Trigger when price exceeds threshold
- **Price Below**: Trigger when price drops below threshold
- **Price Range**: Trigger when price is within a specific range
- **Percent Change**: Monitor price movements over time windows

#### Volume & Liquidity
- **Volume Spike**: Detect unusual trading volume
- **Trading Volume**: Monitor 24h trading volume
- **Liquidity**: Check liquidity thresholds

#### Advanced Conditions
- **Whale Transaction**: Track large wallet transfers
- **Market Cap**: Monitor market capitalization changes
- **Volatility**: Detect volatility spikes
- **Trend Change**: Identify momentum shifts
- **Time Window**: Restrict execution to specific times/days

### 3. **Action Types**

#### Notifications
- **In-App Notifications**: System notifications within the app
- **Email**: Send email alerts
- **Webhook**: POST to custom webhook URLs
- **Telegram**: Send Telegram messages
- **Slack**: Post to Slack channels
- **Discord**: Send Discord messages

#### Trading Actions
- **Execute Trade**: Automatically execute buy/sell orders
- **Pause Strategy**: Stop running trading strategies
- **Update Alert**: Modify alert settings dynamically

#### Logging
- **Log Event**: Record events to system logs

### 4. **Team Sharing & Permissions**

Share alerts with team members with granular permissions:

- **View**: Read-only access to alert configuration
- **Edit**: Modify alert settings
- **Execute**: Manually trigger alerts
- **Admin**: Full control including sharing and deletion

```typescript
interface SharedAccess {
  userId: string;
  permission: 'view' | 'edit' | 'execute' | 'admin';
  grantedAt: string;
}
```

### 5. **Dry-Run Simulations**

Test alerts safely before enabling:

```typescript
const result = await invoke('smart_alert_dry_run', {
  id: 'alert-id',
  marketData: {
    symbol: 'SOL',
    currentPrice: 150.0,
    volume24h: 1000000.0,
  },
  whaleActivity: null,
});

// Returns:
{
  ruleId: 'alert-id',
  ruleName: 'My Alert',
  wouldTrigger: true,
  evaluationMessage: 'All conditions met',
  actionsSimulated: [
    {
      actionType: 'notify',
      wouldExecute: true,
      reason: 'Action would execute successfully',
      validationErrors: [],
      estimatedImpact: 'Would send notification',
    },
  ],
  warnings: [],
  executionTimeMs: 5,
  dryRunAt: '2024-01-01T12:00:00Z',
}
```

## Backend API

### Rust Module Structure

```
src-tauri/src/alerts/logic/
├── mod.rs              # Module exports
├── conditions.rs       # Condition types and evaluation
├── actions.rs          # Action types and execution
├── rule_engine.rs      # Rule evaluation engine
├── dry_run.rs          # Safe simulation logic
├── serialization.rs    # Rule import/export
└── manager.rs          # Database and command handlers
```

### Creating Rules

```rust
use alerts::logic::{
    AlertRule, RuleNode, RuleGroup, LogicalOperator,
    Condition, ConditionType, ConditionParameters,
    Action, ActionType, ActionParameters,
};

// Create a simple price alert
let rule = AlertRule {
    id: uuid::Uuid::new_v4().to_string(),
    name: "SOL Price Above $150".to_string(),
    description: Some("Alert when SOL exceeds $150".to_string()),
    rule_tree: RuleNode {
        id: Some("root".to_string()),
        label: Some("Price Check".to_string()),
        condition: Some(Condition {
            id: Some("cond1".to_string()),
            condition_type: ConditionType::Above,
            parameters: ConditionParameters {
                threshold: Some(150.0),
                ..Default::default()
            },
            description: None,
        }),
        group: None,
        metadata: None,
    },
    actions: vec![
        Action {
            id: Some("action1".to_string()),
            action_type: ActionType::Notify,
            parameters: ActionParameters {
                message: Some("SOL price exceeded $150!".to_string()),
                title: Some("Price Alert".to_string()),
                priority: Some(NotificationPriority::High),
                ..Default::default()
            },
            description: None,
            enabled: true,
        },
    ],
    enabled: true,
    symbol: Some("SOL".to_string()),
    owner_id: Some("user123".to_string()),
    team_id: None,
    shared_with: vec![],
    tags: vec!["price".to_string(), "sol".to_string()],
    created_at: Utc::now().to_rfc3339(),
    updated_at: Utc::now().to_rfc3339(),
};
```

### Complex AND/OR Rules

```rust
// Create a rule with complex logic:
// (Price > $100 AND Volume > 1M) OR (Whale Transfer > $100k)
let rule = AlertRule {
    id: uuid::Uuid::new_v4().to_string(),
    name: "Complex Breakout Alert".to_string(),
    description: Some("Breakout with volume or whale activity".to_string()),
    rule_tree: RuleNode {
        id: Some("root".to_string()),
        label: Some("Root".to_string()),
        condition: None,
        group: Some(RuleGroup {
            operator: LogicalOperator::Or,
            nodes: vec![
                // First branch: Price AND Volume
                RuleNode {
                    id: Some("branch1".to_string()),
                    label: Some("Price & Volume".to_string()),
                    condition: None,
                    group: Some(RuleGroup {
                        operator: LogicalOperator::And,
                        nodes: vec![
                            RuleNode {
                                id: Some("price".to_string()),
                                label: Some("Price Above".to_string()),
                                condition: Some(Condition {
                                    id: Some("c1".to_string()),
                                    condition_type: ConditionType::Above,
                                    parameters: ConditionParameters {
                                        threshold: Some(100.0),
                                        ..Default::default()
                                    },
                                    description: None,
                                }),
                                group: None,
                                metadata: None,
                            },
                            RuleNode {
                                id: Some("volume".to_string()),
                                label: Some("Volume Spike".to_string()),
                                condition: Some(Condition {
                                    id: Some("c2".to_string()),
                                    condition_type: ConditionType::VolumeSpike,
                                    parameters: ConditionParameters {
                                        threshold: Some(1_000_000.0),
                                        ..Default::default()
                                    },
                                    description: None,
                                }),
                                group: None,
                                metadata: None,
                            },
                        ],
                        window_minutes: None,
                        label: Some("Price & Volume".to_string()),
                        description: None,
                    }),
                    metadata: None,
                },
                // Second branch: Whale Activity
                RuleNode {
                    id: Some("branch2".to_string()),
                    label: Some("Whale Activity".to_string()),
                    condition: Some(Condition {
                        id: Some("c3".to_string()),
                        condition_type: ConditionType::WhaleTransaction,
                        parameters: ConditionParameters {
                            whale_threshold_usd: Some(100_000.0),
                            ..Default::default()
                        },
                        description: None,
                    }),
                    group: None,
                    metadata: None,
                },
            ],
            window_minutes: None,
            label: Some("Breakout Conditions".to_string()),
            description: None,
        }),
        metadata: None,
    },
    actions: vec![],
    enabled: true,
    symbol: Some("SOL".to_string()),
    owner_id: Some("user123".to_string()),
    team_id: None,
    shared_with: vec![],
    tags: vec!["complex".to_string(), "whale".to_string()],
    created_at: Utc::now().to_rfc3339(),
    updated_at: Utc::now().to_rfc3339(),
};
```

### Time Windows

Restrict alerts to specific time periods:

```rust
RuleNode {
    id: Some("time-window".to_string()),
    label: Some("Trading Hours".to_string()),
    condition: Some(Condition {
        id: Some("tw1".to_string()),
        condition_type: ConditionType::TimeWindow,
        parameters: ConditionParameters {
            start_time: Some("09:00".to_string()),  // 9 AM
            end_time: Some("17:00".to_string()),    // 5 PM
            days_of_week: Some(vec![1, 2, 3, 4, 5]), // Monday-Friday
            ..Default::default()
        },
        description: None,
    }),
    group: None,
    metadata: None,
}
```

### Auto-Trade Actions

Execute trades automatically when conditions are met:

```rust
Action {
    id: Some("auto-buy".to_string()),
    action_type: ActionType::ExecuteTrade,
    parameters: ActionParameters {
        trade_config: Some(TradeConfig {
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            side: TradeSide::Buy,
            order_type: OrderType::Market,
            amount: Some(10.0),  // Buy 10 SOL
            amount_percent: None,
            price: None,
            slippage_bps: 50,  // 0.5% slippage
            stop_loss_percent: Some(5.0),  // 5% stop loss
            take_profit_percent: Some(10.0),  // 10% take profit
            max_retries: 3,
        }),
        ..Default::default()
    },
    description: Some("Auto-buy 10 SOL".to_string()),
    enabled: true,
}
```

## Frontend API

### TypeScript Types

```typescript
// Alert Rule
interface AlertRule {
  id: string;
  name: string;
  description?: string;
  ruleTree: RuleNode;
  actions: Action[];
  enabled: boolean;
  symbol?: string;
  ownerId?: string;
  teamId?: string;
  sharedWith: SharedAccess[];
  tags: string[];
  createdAt: string;
  updatedAt: string;
}

// Rule Node (Tree Structure)
interface RuleNode {
  id?: string;
  label?: string;
  condition?: Condition;
  group?: RuleGroup;
  metadata?: any;
}

// Rule Group
interface RuleGroup {
  operator: 'and' | 'or';
  nodes: RuleNode[];
  windowMinutes?: number;
  label?: string;
  description?: string;
}

// Condition
interface Condition {
  id?: string;
  conditionType: ConditionType;
  parameters: ConditionParameters;
  description?: string;
}

type ConditionType = 
  | 'above'
  | 'below'
  | 'percent_change'
  | 'volume_spike'
  | 'whale_transaction'
  | 'time_window'
  | 'market_cap'
  | 'liquidity'
  | 'trading_volume'
  | 'price_range'
  | 'volatility'
  | 'trend_change';

// Action
interface Action {
  id?: string;
  actionType: ActionType;
  parameters: ActionParameters;
  description?: string;
  enabled: boolean;
}

type ActionType =
  | 'notify'
  | 'send_email'
  | 'send_webhook'
  | 'send_telegram'
  | 'send_slack'
  | 'send_discord'
  | 'execute_trade'
  | 'pause_strategy'
  | 'update_alert'
  | 'log_event';
```

### Tauri Commands

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Create a rule
const rule = await invoke<AlertRule>('smart_alert_create_rule', {
  req: {
    name: 'My Alert',
    description: 'Alert description',
    ruleTree: ruleNode,
    actions: actions,
    enabled: true,
    symbol: 'SOL',
    tags: ['price', 'test'],
  },
});

// List rules
const rules = await invoke<AlertRule[]>('smart_alert_list_rules', {
  filter: {
    ownerId: 'user123',
    includeDisabled: false,
  },
});

// Get a specific rule
const rule = await invoke<AlertRule>('smart_alert_get_rule', {
  id: 'rule-id',
});

// Update a rule
const updated = await invoke<AlertRule>('smart_alert_update_rule', {
  id: 'rule-id',
  req: {
    name: 'Updated Name',
    enabled: false,
  },
});

// Delete a rule
await invoke('smart_alert_delete_rule', {
  id: 'rule-id',
});

// Dry-run simulation
const result = await invoke<DryRunResult>('smart_alert_dry_run', {
  id: 'rule-id',
  marketData: {
    symbol: 'SOL',
    currentPrice: 150.0,
    volume24h: 1000000.0,
  },
  whaleActivity: null,
});

// Execute rule
const execution = await invoke<RuleExecutionResult>('smart_alert_execute', {
  id: 'rule-id',
  marketData: {
    symbol: 'SOL',
    currentPrice: 150.0,
  },
  whaleActivity: null,
  dryRun: true,  // Set to false for actual execution
});
```

## Testing

### Backend Tests

Tests are included in the module files:

```bash
cd src-tauri
cargo test --lib alerts::logic
```

Tests cover:
- ✅ Rule evaluation (AND/OR logic)
- ✅ Condition evaluation (all types)
- ✅ Time window enforcement
- ✅ Permission system
- ✅ Dry-run simulations
- ✅ Rule serialization/deserialization

### Frontend Tests

Create UI interaction tests:

```typescript
// src/__tests__/smartAlerts.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/tauri';

describe('Smart Alerts', () => {
  it('should create a rule', async () => {
    const rule = await invoke('smart_alert_create_rule', {
      req: {
        name: 'Test Rule',
        ruleTree: { /* ... */ },
        actions: [],
        enabled: true,
      },
    });
    
    expect(rule).toBeDefined();
    expect(rule.name).toBe('Test Rule');
  });
  
  it('should perform dry-run simulation', async () => {
    const result = await invoke('smart_alert_dry_run', {
      id: 'rule-id',
      marketData: { symbol: 'SOL', currentPrice: 150.0 },
      whaleActivity: null,
    });
    
    expect(result.wouldTrigger).toBeDefined();
    expect(result.actionsSimulated).toBeInstanceOf(Array);
  });
});
```

## Security Considerations

### 1. **Permission Validation**

Always validate user permissions before allowing rule modifications:

```rust
if !rule.has_access(&user_id, Permission::Edit) {
    return Err(SmartAlertError::PermissionDenied);
}
```

### 2. **Action Validation**

All actions are validated before execution:

```rust
for action in &rule.actions {
    action.validate()?;  // Throws error if invalid
}
```

### 3. **Dry-Run First**

Encourage users to test with dry-run before enabling:

```typescript
// Always test first
const dryRun = await invoke('smart_alert_dry_run', { id, marketData, whaleActivity });

if (dryRun.warnings.length > 0) {
  console.warn('Warnings:', dryRun.warnings);
}

// Then enable if safe
await invoke('smart_alert_update_rule', { 
  id, 
  req: { enabled: true } 
});
```

### 4. **Trade Limits**

Implement trading limits for auto-trade actions:

```rust
if trade_config.amount > MAX_TRADE_AMOUNT {
    return Err("Trade amount exceeds maximum limit");
}
```

## Performance

### Rule Evaluation

- Single rule evaluation: < 5ms
- Complex rules (10+ conditions): < 20ms
- Batch evaluation (100 rules): < 500ms

### Optimization Tips

1. **Use Specific Symbols**: Filter rules by symbol to reduce evaluation overhead
2. **Disable Unused Rules**: Keep only active rules enabled
3. **Batch Evaluations**: Evaluate multiple rules in a single call
4. **Cache Market Data**: Reuse market data across evaluations

## Troubleshooting

### Common Issues

#### Rules Not Triggering

1. Check rule is enabled: `rule.enabled === true`
2. Verify conditions are met using dry-run
3. Check time window restrictions
4. Verify symbol matches market data

#### Actions Not Executing

1. Check action is enabled: `action.enabled === true`
2. Validate action parameters: `action.validate()`
3. Review dry-run warnings
4. Check notification channel configurations

#### Permission Errors

1. Verify user ID matches owner or has shared access
2. Check permission level is sufficient for operation
3. Review shared_with array for user

## Examples

Complete examples are available in:
- Backend: `src-tauri/src/alerts/logic/rule_engine.rs` (tests)
- Backend: `src-tauri/src/alerts/logic/dry_run.rs` (tests)
- Frontend: `src/components/alerts/LogicBuilder` (implementation)

## Migration from Simple Alerts

To migrate from simple price alerts to smart alerts:

1. Export existing alerts
2. Convert to rule format
3. Import as smart alert rules
4. Test with dry-run
5. Enable and monitor

## Future Enhancements

Planned features:
- Visual flow chart editor
- More condition types (technical indicators, on-chain metrics)
- Alert scheduling (recurring alerts)
- Multi-chain support
- Alert performance analytics
- Machine learning condition suggestions

## Support

For questions or issues:
- Check the test files for examples
- Review this documentation
- Examine the TypeScript type definitions
- Test with dry-run simulations first
