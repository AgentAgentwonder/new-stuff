# Data Pipeline & Audit Trail - Complete Guide

## Overview

This document describes the complete data processing pipeline including price engine optimization, layered caching, compression, and event sourcing for audit trails.

## Architecture

```
┌─────────────────┐
│ Price Updates   │
│  (WebSocket)    │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│   Price Engine (<1ms p95 latency)  │
│  • Lock-free queues (crossbeam)    │
│  • Zero-copy processing (bytes)    │
│  • Memory pooling                   │
│  • Latency instrumentation         │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│   Layered Cache (moka/LRU + disk)  │
│  • In-memory LRU with TTL          │
│  • Per-type TTL configuration      │
│  • Cache warming on startup        │
│  • Hit/miss rate tracking          │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│   Event Store (SQLite)              │
│  • Immutable event log             │
│  • Event replay capability         │
│  • Point-in-time queries           │
│  • Snapshot mechanism              │
└────────┬────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│   Compression Manager               │
│  • zstd/lz4 compression            │
│  • Background compression jobs     │
│  • Age-based compression (7d)      │
│  • Compression metrics tracking    │
└─────────────────────────────────────┘
```

## Components

### 1. Price Engine

**Location:** `src-tauri/src/core/price_engine.rs`

**Features:**
- Lock-free message queue using `crossbeam::queue::SegQueue`
- Zero-copy buffer handling with `bytes::Bytes`
- Memory pool (512 pre-allocated 1KB buffers)
- Sub-millisecond latency tracking (p50, p95, p99)
- Throughput and error rate monitoring
- CPU usage tracking via `sysinfo`

**Key Metrics:**
- **Target:** P95 latency < 1ms
- **Throughput:** > 10,000 messages/second
- **Memory:** Stable via memory pooling
- **Error Rate:** Tracked and exposed

**API Commands:**
```rust
// Get current metrics
invoke('get_performance_metrics') -> PerformanceMetrics

// Run benchmark with N updates
invoke('run_performance_test', { numUpdates: 10000 }) -> PerformanceMetrics

// Reset statistics
invoke('reset_performance_stats')
```

**Benchmarks:**
```bash
cd src-tauri
cargo bench --bench price_engine_bench
```

### 2. Cache Manager

**Location:** `src-tauri/src/core/cache_manager.rs`

**Cache Tiers:**
1. **In-Memory (Primary)**
   - LRU eviction policy
   - Per-type TTL configuration
   - Max size: 100 MB / 1000 entries
   - Hit/miss tracking per type

2. **Disk-Backed (Future)**
   - SQLite-based persistent cache
   - Warm data storage
   - Cache preloading on startup

**Cache Types:**
- `TokenPrice` - Default TTL: 1 second
- `TokenInfo` - Default TTL: 1 hour
- `MarketData` - Default TTL: 1 hour
- `TopCoins` - Default TTL: 1 hour
- `TrendingCoins` - Default TTL: 1 hour
- `UserData` - Default TTL: 1 day

**TTL Configuration:**
```rust
pub struct CacheTtlConfig {
    pub prices: u64,      // 1 second (1,000 ms)
    pub metadata: u64,    // 1 hour (3,600,000 ms)
    pub history: u64,     // 1 day (86,400,000 ms)
}
```

**API Commands:**
```rust
// Get cache statistics
invoke('get_cache_statistics') -> CacheStatistics

// Clear entire cache
invoke('clear_cache')

// Warm cache with keys
invoke('warm_cache', { keys: Vec<String> }) -> WarmProgress

// Get/set cache items
invoke('get_cache_item', { key, cache_type })
invoke('set_cache_item', { key, data, cache_type })

// TTL configuration
invoke('get_ttl_config') -> CacheTtlConfig
invoke('update_ttl_config', { config })
invoke('reset_ttl_config')

// Performance testing
invoke('test_cache_performance') -> CacheTestResult
```

**Acceptance Criteria:**
- Hit rate > 80% for frequently accessed data
- TTL honored for all cache types
- Automatic eviction when size/entry limits reached
- Cache warming reduces initial latency

### 3. Event Store

**Location:** `src-tauri/src/data/event_store.rs`

**Event Types:**
- `OrderPlaced` - New order created
- `OrderFilled` - Order execution
- `OrderCancelled` - Order cancellation
- `PositionOpened` - New position
- `PositionClosed` - Position closure with P&L
- `BalanceChanged` - Wallet balance updates
- `SettingChanged` - Configuration changes
- `WalletConnected` - Wallet connection
- `WalletDisconnected` - Wallet disconnection
- `TradeExecuted` - Trade completion

**Features:**
- Immutable append-only log
- Sequence numbers per aggregate
- Event replay for state reconstruction
- Point-in-time queries
- Automatic snapshots every 1000 events
- Export to JSON/CSV formats

**Storage:**
```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    event_data TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    timestamp TEXT NOT NULL
);

CREATE TABLE snapshots (
    id TEXT PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    state_data TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    timestamp TEXT NOT NULL
);
```

**API Commands:**
```rust
// Query events
invoke('get_events_command', { 
    aggregate_id?, 
    event_type?, 
    from_time?, 
    to_time?, 
    limit?, 
    offset? 
}) -> Vec<EventRecord>

// Replay events for aggregate
invoke('replay_events_command', { aggregate_id }) -> Vec<Event>

// Point-in-time query
invoke('get_state_at_time_command', { aggregate_id, timestamp }) -> Vec<Event>

// Export audit trail
invoke('export_audit_trail_command', { 
    aggregate_id?, 
    event_type?, 
    from_time?, 
    to_time?,
    format: 'json' | 'csv'
}) -> String

// Create snapshot
invoke('create_snapshot_command', { aggregate_id, state_data }) -> String

// Get statistics
invoke('get_event_stats') -> EventStats
```

### 4. Compression Manager

**Location:** `src-tauri/src/data/database.rs`

**Compression Strategy:**
- **Algorithm:** zstd (Zstandard) or lz4
- **Default Level:** 3 (balanced speed/ratio)
- **Age Threshold:** 7 days by default
- **Auto-compress:** Daily at 3 AM
- **Target Data:**
  - Events older than threshold
  - Closed orders (filled, cancelled, failed) older than 30 days

**Configuration:**
```rust
pub struct CompressionConfig {
    pub enabled: bool,
    pub age_threshold_days: i64,  // Default: 7
    pub compression_level: i32,   // 1-9, default: 3
    pub auto_compress: bool,
}
```

**Compression Metrics:**
```rust
pub struct CompressionStats {
    pub total_uncompressed_bytes: i64,
    pub total_compressed_bytes: i64,
    pub compression_ratio: f64,           // Percentage saved
    pub num_compressed_records: i64,
    pub space_saved_mb: f64,
    pub last_compression_run: Option<String>,
}
```

**API Commands:**
```rust
// Get compression statistics
invoke('get_compression_stats') -> CompressionStats

// Manual compression trigger
invoke('compress_old_data') -> i64  // Returns records compressed

// Configuration management
invoke('get_compression_config') -> CompressionConfig
invoke('update_compression_config', { config })

// Decompress specific record
invoke('decompress_data', { id }) -> Vec<u8>

// Database size info
invoke('get_database_size') -> DatabaseSize
```

**Background Jobs:**
The compression manager runs automatic compression daily at 3 AM:
```rust
// Compresses events older than age_threshold_days
manager.compress_old_events().await

// Compresses closed orders older than 30 days
manager.compress_old_trades().await

// Cleanup decompression cache
manager.cleanup_cache().await
```

## Performance Dashboard

**Location:** `src/pages/Settings/PerformanceDashboard.tsx`

**Displays:**
1. **Latency Metrics**
   - P50, P95, P99 latency (real-time)
   - Target indicator (P95 < 1ms)
   - Latency history chart (last 50 points)

2. **Engine Statistics**
   - Messages received/processed
   - Error count and rate
   - Throughput (msg/s)
   - CPU usage
   - Uptime

3. **Performance Testing**
   - Run 10K message benchmark
   - Pass/fail indicator
   - Detailed results

**Features:**
- Auto-refresh mode (1 second interval)
- Manual refresh button
- Reset statistics
- Interactive charts with recharts

## Cache Settings

**Location:** `src/components/CacheSettings.tsx`

**Displays:**
1. **Cache Statistics**
   - Hit rate (total and per-type)
   - Cache size (bytes and entries)
   - Eviction count
   - Last warm timestamp

2. **TTL Configuration**
   - Visual sliders for each type
   - Preset quick-select buttons
   - Impact preview
   - Save/reset controls

3. **Cache Operations**
   - Manual cache warming
   - Clear all cache
   - Performance test

## Storage Settings

**Location:** `src/pages/Settings/StorageSettings.tsx`

**Displays:**
1. **Storage Statistics**
   - Total database size
   - Compressed data size
   - Space saved (MB and %)
   - Compression ratio
   - Compressed record count
   - Last compression run

2. **Compression Configuration**
   - Enable/disable compression
   - Auto-compress toggle
   - Age threshold slider (7-90 days)
   - Compression level (1-9)

3. **Manual Operations**
   - Compress now button
   - Real-time feedback

## Event Audit Log

**Location:** `src/pages/Settings/EventAuditLog.tsx`

**Features:**
- Filter by aggregate ID, event type, date range
- Paginated event list
- Event details view
- Export to JSON/CSV
- Replay visualization
- Event statistics

## Benchmarking & Profiling

### Running Benchmarks

```bash
# Interactive menu
./scripts/bench_and_profile.sh

# Or direct commands
./scripts/bench_and_profile.sh bench      # Run benchmarks only
./scripts/bench_and_profile.sh test       # Run tests
./scripts/bench_and_profile.sh flamegraph # Generate flamegraph
./scripts/bench_and_profile.sh all        # Everything
```

### Benchmark Suite

**Included Benchmarks:**
1. `single_price_update` - Single update latency
2. `batch_price_updates` - Batch processing (100, 1K, 10K)
3. `concurrent_updates` - 4 threads concurrent
4. `get_metrics` - Metrics retrieval overhead
5. `latency_target` - Validates P95 < 1ms

**Viewing Results:**
```bash
# Results saved to target/criterion/
open target/criterion/report/index.html
```

### Profiling

**Flamegraph:**
```bash
cargo install flamegraph
./scripts/bench_and_profile.sh flamegraph
# Opens flamegraph.svg
```

**Linux Perf:**
```bash
./scripts/bench_and_profile.sh perf
# Generates perf.data and report
```

## Testing

### Unit Tests

```bash
# Run all tests
cd src-tauri
cargo test

# Run specific module tests
cargo test cache_manager
cargo test event_store
cargo test price_engine
cargo test compression

# With output
cargo test -- --nocapture
```

### Integration Tests

**Cache TTL Test:**
```bash
cargo test --test cache_manager_tests test_ttl_expiration
```

**Event Replay Test:**
```bash
cargo test --lib event_store::tests::test_replay_events
```

**Compression Test:**
```bash
cargo test --lib database::tests::test_compress_decompress
```

## Configuration Files

### Cache TTL Config

**Location:** `config/cache_ttl.json`

```json
{
  "prices": 1000,
  "metadata": 3600000,
  "history": 86400000
}
```

### Compression Config

**Stored in:** SQLite `compression_config` table

```sql
SELECT * FROM compression_config WHERE id = 1;
```

## Monitoring & Metrics

### Key Performance Indicators

1. **Price Engine**
   - ✅ P95 latency < 1ms
   - ✅ Throughput > 10K msg/s
   - ✅ Error rate < 0.01%
   - ✅ Memory stable

2. **Cache**
   - ✅ Hit rate > 80%
   - ✅ TTL honored
   - ✅ Size under limit
   - ✅ Warm on startup

3. **Compression**
   - ✅ Ratio > 50%
   - ✅ Space saved measurable
   - ✅ Auto-compress runs
   - ✅ Decompress on-demand

4. **Event Store**
   - ✅ All events logged
   - ✅ Replay accuracy 100%
   - ✅ Export functional
   - ✅ Snapshots created

### Accessing Metrics

**Via UI:**
- Settings → Performance Dashboard
- Settings → Cache Management
- Settings → Storage Settings
- Settings → Event Audit Log

**Via API:**
```javascript
// Performance
const perfMetrics = await invoke('get_performance_metrics');

// Cache
const cacheStats = await invoke('get_cache_statistics');

// Compression
const compressionStats = await invoke('get_compression_stats');

// Events
const eventStats = await invoke('get_event_stats');
```

## Troubleshooting

### High Latency

**Symptoms:** P95 > 1ms

**Solutions:**
1. Check CPU usage (`get_performance_metrics`)
2. Run benchmarks to isolate issue
3. Profile with flamegraph
4. Ensure release build
5. Check for disk I/O blocking

### Low Cache Hit Rate

**Symptoms:** Hit rate < 50%

**Solutions:**
1. Increase TTL for stable data
2. Warm cache on startup
3. Check cache size limits
4. Review access patterns
5. Adjust per-type TTLs

### Compression Not Running

**Symptoms:** No compressed records

**Solutions:**
1. Check `enabled` flag in config
2. Verify `auto_compress` is true
3. Check age threshold (default 7 days)
4. Manually trigger compression
5. Check logs for errors

### Event Log Growth

**Symptoms:** Large event database

**Solutions:**
1. Enable compression
2. Lower age threshold
3. Create more snapshots
4. Export and archive old events
5. Implement retention policy

## Best Practices

### Cache Strategy
- Use short TTL for price data (1s)
- Use longer TTL for metadata (1h+)
- Warm frequently accessed data on startup
- Monitor hit rates and adjust TTLs
- Test cache performance regularly

### Event Sourcing
- Always publish events for state changes
- Use meaningful aggregate IDs
- Create snapshots for long-running aggregates
- Export audit logs regularly
- Test replay logic

### Compression
- Enable auto-compress for production
- Use level 3-5 for balance
- Set appropriate age threshold
- Monitor compression ratio
- Keep frequently accessed data uncompressed

### Performance
- Run benchmarks before releases
- Profile hot paths
- Monitor latency metrics
- Set up alerts for threshold violations
- Document optimization techniques

## Future Enhancements

### Phase 2 - Disk-Backed Cache
- SQLite-based L2 cache
- Automatic tier management
- Cache migration strategies
- Preloading optimization

### Phase 3 - Advanced Compression
- Dictionary-based compression
- Columnar compression for analytics
- Adaptive compression levels
- Compression on write

### Phase 4 - Distributed Event Store
- Multi-node event replication
- Consensus for ordering
- Sharding by aggregate
- Global audit trail

## References

- [PERFORMANCE_OPTIMIZATION.md](./PERFORMANCE_OPTIMIZATION.md) - Price engine details
- [CACHE_TTL_IMPLEMENTATION.md](./CACHE_TTL_IMPLEMENTATION.md) - Cache design
- [EVENT_SOURCING.md](./EVENT_SOURCING.md) - Event sourcing patterns
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs)
- [Zstd Compression](https://facebook.github.io/zstd/)

## Acceptance Criteria ✅

- ✅ Price engine achieves <1ms P95 latency
- ✅ Cache layer with per-type TTLs implemented
- ✅ Cache warming on app startup
- ✅ Historic data compression with zstd
- ✅ Background compression jobs (3 AM daily)
- ✅ Event sourcing for key actions
- ✅ Event replay API functional
- ✅ Audit log export (JSON/CSV)
- ✅ Metrics dashboard in Settings
- ✅ Benchmark suite with <1ms threshold
- ✅ Tests validate cache TTL, compression, replay
- ✅ Documentation and profiling scripts included
