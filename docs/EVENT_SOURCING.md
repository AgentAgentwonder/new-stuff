# Event Sourcing Audit Trail

This document describes the event sourcing implementation for immutable audit trail of all state changes.

## Overview

Event sourcing is an architectural pattern where all changes to application state are stored as a sequence of events. Instead of storing just the current state, we store the entire history of state changes. This provides:

- **Complete audit trail** - Every state change is recorded immutably
- **Point-in-time recovery** - Reconstruct system state at any past moment
- **Event replay** - Rebuild current state from event history
- **Debugging and analysis** - Full visibility into how state evolved

## Architecture

### Event Store

The event store is implemented in `src-tauri/src/data/event_store.rs` and provides:

- Immutable append-only event log
- Sequence numbers per aggregate to maintain order
- Snapshot support for performance optimization
- Point-in-time recovery with caching
- Export functionality (JSON/CSV)

### Events

All state-changing operations publish events:

```rust
pub enum Event {
    OrderPlaced { order_id, symbol, side, quantity, price, timestamp },
    OrderFilled { order_id, fill_price, filled_quantity, timestamp },
    OrderCancelled { order_id, reason, timestamp },
    PositionOpened { position_id, symbol, quantity, entry_price, timestamp },
    PositionClosed { position_id, exit_price, pnl, timestamp },
    BalanceChanged { wallet, token, old_balance, new_balance, reason, timestamp },
    SettingChanged { key, old_value, new_value, timestamp },
    WalletConnected { wallet_address, wallet_type, timestamp },
    WalletDisconnected { wallet_address, timestamp },
    TradeExecuted { trade_id, from_token, to_token, from_amount, to_amount, price, timestamp },
}
```

### Database Schema

#### Events Table

```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,              -- UUID
    event_type TEXT NOT NULL,         -- Type of event (order_placed, etc.)
    event_data TEXT NOT NULL,         -- JSON serialized event data
    aggregate_id TEXT NOT NULL,       -- Entity ID (order_123, wallet_abc)
    sequence INTEGER NOT NULL,        -- Monotonically increasing per aggregate
    timestamp TEXT NOT NULL           -- RFC3339 timestamp
);

CREATE INDEX idx_events_aggregate ON events(aggregate_id, sequence);
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_type ON events(event_type);
```

#### Snapshots Table

```sql
CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,              -- UUID
    aggregate_id TEXT NOT NULL,       -- Entity ID
    state_data TEXT NOT NULL,         -- JSON serialized state
    sequence INTEGER NOT NULL,        -- Event sequence at snapshot
    timestamp TEXT NOT NULL           -- RFC3339 timestamp
);

CREATE INDEX idx_snapshots_aggregate ON snapshots(aggregate_id, sequence DESC);
CREATE INDEX idx_snapshots_timestamp ON snapshots(timestamp);
```

## Usage

### Publishing Events

Events are automatically published by state-changing operations:

```rust
// In OrderManager::create_order()
let event = AuditEvent::OrderPlaced {
    order_id: order.id.clone(),
    symbol: symbol.clone(),
    side: order.side.to_string(),
    quantity: order.amount,
    price: order.limit_price.or(order.stop_price),
    timestamp: Utc::now(),
};

let aggregate_id = format!("order_{}", order.id);
event_store.read().await.publish_event(event, &aggregate_id).await?;
```

### Querying Events

Get events with filters:

```rust
let filter = EventFilter {
    aggregate_id: Some("order_123".to_string()),
    event_type: Some("order_placed".to_string()),
    from_time: Some(start_time),
    to_time: Some(end_time),
    limit: Some(100),
    offset: Some(0),
};

let events = event_store.read().await.get_events(filter).await?;
```

### Event Replay

Reconstruct state from event history:

```rust
let events = event_store.read().await.replay_events("order_123").await?;
// Apply events in sequence to rebuild state
for event in events {
    match event {
        Event::OrderPlaced { .. } => { /* process */ },
        Event::OrderFilled { .. } => { /* process */ },
        // ...
    }
}
```

### Point-in-Time Recovery

Get state at a specific time:

```rust
let timestamp = DateTime::parse_from_rfc3339("2024-01-15T12:00:00Z")?;
let events = event_store.read().await
    .get_state_at_time("order_123", timestamp)
    .await?;
```

### Snapshots

Snapshots are created automatically every 1000 events per aggregate. Manual snapshots:

```rust
let state_data = serde_json::to_string(&current_state)?;
let snapshot_id = event_store.read().await
    .create_snapshot("order_123", &state_data)
    .await?;
```

### Export

Export events for compliance or analysis:

```rust
// JSON export
let json = event_store.read().await
    .export_events(filter, "json")
    .await?;

// CSV export with human-readable descriptions
let csv = event_store.read().await
    .export_events(filter, "csv")
    .await?;
```

## Frontend UI

The Event Sourcing Audit Trail UI is available in Settings:

**Location**: Settings → Event Sourcing Audit Trail

**Features**:
- View all events with pagination
- Filter by aggregate ID, event type, date range
- Expandable rows showing full JSON event data
- Export to CSV or JSON
- Replay events for any aggregate
- Create manual snapshots
- Event statistics and counts

## Tauri Commands

The following commands are available from the frontend:

```typescript
// Get events with filters
await invoke('get_events_command', {
  aggregateId: 'order_123',
  eventType: 'order_placed',
  fromTime: '2024-01-01T00:00:00Z',
  toTime: '2024-12-31T23:59:59Z',
  limit: 50,
  offset: 0
});

// Replay events for an aggregate
await invoke('replay_events_command', {
  aggregateId: 'order_123'
});

// Get state at specific time
await invoke('get_state_at_time_command', {
  aggregateId: 'order_123',
  timestamp: '2024-01-15T12:00:00Z'
});

// Export audit trail
await invoke('export_audit_trail_command', {
  aggregateId: null,  // optional
  eventType: null,    // optional
  fromTime: null,     // optional
  toTime: null,       // optional
  format: 'csv'       // 'json' or 'csv'
});

// Create manual snapshot
await invoke('create_snapshot_command', {
  aggregateId: 'order_123',
  stateData: JSON.stringify({ state: 'data' })
});

// Get event statistics
await invoke('get_event_stats');
```

## Performance Optimizations

### Snapshots

- Automatically created every 1000 events per aggregate
- Replay starts from latest snapshot + subsequent events
- Reduces replay time by >10x for large event histories

### Point-in-Time Cache

- Recent point-in-time queries cached for 5 minutes
- Cache limited to 100 entries (LRU eviction)
- Significantly reduces database load for repeated queries

### Indexes

- Composite index on (aggregate_id, sequence) for fast replay
- Timestamp index for time-range queries
- Event type index for filtered queries

## Best Practices

### Event Design

1. **Self-contained** - Events should include all necessary context
2. **Immutable** - Never modify or delete events
3. **Versioning** - Consider event versioning for schema changes
4. **Timestamps** - Always include UTC timestamps

### Aggregate IDs

Use descriptive, consistent aggregate IDs:
- Orders: `order_{order_id}`
- Wallets: `wallet_{address}`
- Positions: `position_{position_id}`
- Trades: `trade_{trade_id}`
- Settings: `settings` (singleton)

### Event Publishing

1. Publish events **after** successful state change
2. Include both old and new values for changes
3. Handle publishing failures gracefully (log error, don't block)
4. Never throw exceptions from event publishing

### Privacy & Compliance

- Events are append-only and immutable
- Consider data retention policies for old events
- Redact sensitive data in exports if needed
- Use aggregate IDs for data subject access requests

## Testing

Test event sourcing functionality:

```bash
# Run event sourcing tests
cargo test event_store

# Test replay accuracy
cargo test test_event_replay

# Test point-in-time recovery
cargo test test_point_in_time_recovery

# Test snapshot creation
cargo test test_snapshot_creation
```

## Monitoring

Monitor event store health:

- Total events count
- Events per day/hour
- Event type distribution
- Snapshot count and size
- Replay performance
- Point-in-time cache hit rate

Access statistics via `get_event_stats` command.

## Troubleshooting

### Slow Replay

- Check if snapshots are being created
- Verify indexes exist on events table
- Consider creating manual snapshot

### Missing Events

- Check error logs for publishing failures
- Verify event_store is properly initialized
- Ensure state-changing operations call publish_event

### Database Growth

- Events are append-only and grow over time
- Consider archiving old events (>1 year)
- Monitor database size and performance
- Optimize snapshot frequency

## Future Enhancements

Potential improvements:

1. **Event Versioning** - Support schema evolution
2. **Event Streaming** - Real-time event stream subscription
3. **Distributed Events** - Multi-node event replication
4. **Event Projections** - Materialized views from events
5. **Command Sourcing** - Store intent alongside events
6. **Saga Pattern** - Distributed transaction support

## References

- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
- [CQRS and Event Sourcing](https://docs.microsoft.com/en-us/azure/architecture/patterns/cqrs)
- [Event Store Documentation](https://www.eventstore.com/event-sourcing)
