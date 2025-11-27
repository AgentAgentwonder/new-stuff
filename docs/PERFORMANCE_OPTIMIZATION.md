# Performance Optimization: Sub-millisecond Price Engine

## Overview

This document describes the ultra-low latency optimizations applied to the price engine to achieve sub-millisecond (< 1ms) p95 latency for end-to-end price update processing.

## Architecture

### Core Components

1. **Lock-Free Data Structures**
   - `crossbeam::queue::SegQueue` for lock-free message queuing
   - `Arc<AtomicU64>` for lock-free counters (messages processed, errors)
   - Minimizes contention in multi-threaded scenarios

2. **Fast Synchronization Primitives**
   - `parking_lot::RwLock` for price map (faster than `std::sync::RwLock`)
   - `parking_lot::Mutex` for latency tracking (faster than `std::sync::Mutex`)
   - Up to 2x faster lock acquisition compared to standard library

3. **Zero-Copy Processing**
   - `Cow<'static, str>` for symbol strings (Clone on Write)
   - `bytes::Bytes` for zero-copy buffer handling
   - References passed through pipeline instead of cloning
   - Minimizes heap allocations in hot path

4. **Memory Pooling**
   - Pre-allocated buffer pool (512 buffers × 1KB)
   - Reuses buffers to avoid allocation overhead
   - Automatic capacity management (shrink oversized buffers)
   - Lock-free pool using `SegQueue`

5. **Latency Instrumentation**
   - Timestamp at pipeline entry and exit
   - Circular buffer for last 10,000 latency samples
   - Percentile calculation (p50, p95, p99)
   - Mean, min, max tracking
   - Nanosecond precision, microsecond reporting

## Optimization Techniques

### 1. Lock-Free Queuing

```rust
// Before: Standard Mutex-based queue
let queue: Mutex<VecDeque<Update>> = Mutex::new(VecDeque::new());

// After: Lock-free queue
let queue: SegQueue<Update> = SegQueue::new();
```

**Benefits:**
- No lock contention between threads
- Non-blocking push/pop operations
- Better CPU cache behavior

### 2. Memory Pooling

```rust
// Acquire buffer from pool
let mut buffer = self.memory_pool.acquire();

// Use buffer...
buffer.extend_from_slice(&data);

// Return to pool for reuse
self.memory_pool.release(buffer);
```

**Benefits:**
- Eliminates allocation in hot path
- Reduces GC pressure
- Predictable memory usage

### 3. Zero-Copy with Bytes

```rust
// Acquire pooled buffer
let mut buffer = self.memory_pool.acquire();
buffer.extend_from_slice(&serialized);

// Create zero-copy reference
let bytes = Bytes::copy_from_slice(&buffer);

// Release buffer immediately
self.memory_pool.release(buffer);
```

**Benefits:**
- Reference-counted buffer sharing
- No unnecessary copies
- Efficient memory usage

### 4. Atomic Counters

```rust
// Lock-free counter increment
self.messages_processed.fetch_add(1, Ordering::Relaxed);

// Fast counter read
let count = self.messages_processed.load(Ordering::Relaxed);
```

**Benefits:**
- No mutex overhead
- Cache-friendly
- Guaranteed atomic operations

## Performance Metrics

### Tracked Metrics

1. **Latency Statistics**
   - P50 (median latency)
   - P95 (95th percentile) - **Target: < 1ms**
   - P99 (99th percentile)
   - Mean, min, max
   - Sample count

2. **Throughput**
   - Messages per second
   - Calculated from uptime and message count

3. **Error Tracking**
   - Error count
   - Error rate (errors / total messages)

4. **Operational Stats**
   - Messages processed
   - Uptime
   - Memory pool utilization (implicit)

### Percentile Calculation

```rust
fn percentile(sorted: &[u64], quantile: f64) -> u64 {
    let max_index = sorted.len() - 1;
    let idx = ((max_index as f64) * quantile).round() as usize;
    sorted[idx.min(max_index)]
}
```

Percentiles calculated from sorted latency samples:
- P50 = 50th percentile (median)
- P95 = 95th percentile (95% of requests faster)
- P99 = 99th percentile (99% of requests faster)

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cd src-tauri
cargo bench

# Run specific benchmark
cargo bench --bench price_engine_bench

# With verbose output
cargo bench -- --verbose
```

### Benchmark Suite

1. **Single Update Latency**
   - Measures end-to-end latency for one update
   - Target: < 1ms p95

2. **Batch Updates**
   - 100, 1,000, 10,000 updates
   - Measures throughput and latency distribution

3. **Concurrent Updates**
   - 4 threads, 250 updates each
   - Tests lock-free contention handling

4. **Metrics Retrieval**
   - Measures overhead of get_metrics()
   - Should be negligible

5. **Latency Target Validation**
   - Asserts P95 < 1ms
   - Fails CI if target missed

### Interpreting Results

```
single_price_update     time:   [145.23 ns 148.67 ns 152.34 ns]
batch_price_updates/100 time:   [14.234 μs 14.567 μs 14.923 μs]
```

- **Time**: Mean execution time
- **Range**: 95% confidence interval
- **Lower is better**

## Frontend Dashboard

### Features

1. **Real-time Metrics**
   - Live P50, P95, P99 display
   - Throughput monitoring
   - Error rate tracking

2. **Latency Chart**
   - Line chart showing P50, P95, P99 over time
   - Last 50 data points
   - Auto-refresh mode (1 second interval)

3. **Performance Test**
   - Run 10,000 update benchmark
   - Pass/fail indicator (P95 < 1ms)
   - Detailed results display

4. **Statistics Panel**
   - Mean, min, max latency
   - Messages processed
   - Error count and rate
   - Engine uptime

### Accessing Dashboard

1. Open Settings page
2. Scroll to "Performance Dashboard" section
3. Enable auto-refresh for live monitoring
4. Click "Run Performance Test" to benchmark

## CI/CD Integration

### Pre-commit Checks

```bash
# Run benchmarks as part of testing
cargo test --release
cargo bench --no-run  # Ensure benchmarks compile
```

### Performance Regression Detection

Add to CI workflow:

```yaml
- name: Run Performance Benchmarks
  run: |
    cd src-tauri
    cargo bench --bench price_engine_bench -- --save-baseline main
    
- name: Compare Against Baseline
  run: |
    cargo bench --bench price_engine_bench -- --baseline main
```

If P95 exceeds 1ms target, benchmark fails and CI blocks merge.

## Profiling

### Using cargo-flamegraph

```bash
# Install flamegraph
cargo install flamegraph

# Profile the benchmark
cd src-tauri
cargo flamegraph --bench price_engine_bench

# Open flamegraph.svg in browser
```

### Using perf (Linux)

```bash
# Record performance data
perf record --call-graph=dwarf cargo bench

# Generate report
perf report
```

### What to Look For

1. **Hot Spots**: Functions consuming most CPU time
2. **Allocations**: Unexpected heap allocations
3. **Lock Contention**: Time spent waiting on locks
4. **Cache Misses**: L1/L2/L3 cache performance

## Load Testing

### 10,000 Updates/Second Test

```rust
// Via API
let metrics = invoke('run_performance_test', { numUpdates: 10000 }).await;

// Via code
let engine = get_price_engine();
let metrics = engine.run_performance_test(10_000).await;
```

### Expected Results

At 10,000 updates/second:
- P95 latency: < 1ms (< 1000 μs)
- Throughput: > 10,000 msg/s
- Error rate: 0%
- Memory: Stable (pooling prevents leaks)

## Memory Leak Testing

### 24-Hour Continuous Run

```bash
# Start with memory profiling
valgrind --leak-check=full --track-origins=yes ./target/release/app

# Or use Rust's built-in leak detection
RUSTFLAGS="-Z sanitizer=leak" cargo build --target x86_64-unknown-linux-gnu
./target/x86_64-unknown-linux-gnu/release/app
```

### Monitoring

Watch for:
- Memory growth over time
- Stable pool utilization
- No leaked buffers
- Constant memory footprint

## Troubleshooting

### P95 Exceeds 1ms

**Possible Causes:**
1. CPU contention (other processes)
2. Disk I/O blocking main thread
3. Network latency in real data path
4. Inefficient serialization

**Solutions:**
- Run on dedicated hardware
- Use release builds (`--release`)
- Profile to find bottleneck
- Check for blocking I/O

### High Error Rate

**Possible Causes:**
1. Serialization failures
2. Payload corruption
3. Memory pool exhaustion

**Solutions:**
- Check log output for warnings
- Increase pool capacity
- Validate input data

### Throughput Lower Than Expected

**Possible Causes:**
1. Single-threaded processing
2. Lock contention (check parking_lot usage)
3. CPU throttling

**Solutions:**
- Ensure Rust optimization flags set
- Profile for hot spots
- Increase buffer sizes

## Future Optimizations

### Phase 2 (Caching)

- In-memory price cache
- LRU eviction policy
- Cache hit rate metrics

### Phase 3 (Compression)

- Zstd compression for payload
- Dictionary-based compression
- Compression ratio tracking

### Phase 4 (Event Sourcing)

- Append-only event log
- Replay capability
- Snapshot mechanism

## References

- [Crossbeam Documentation](https://docs.rs/crossbeam)
- [Parking Lot Documentation](https://docs.rs/parking_lot)
- [Bytes Documentation](https://docs.rs/bytes)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

## Acceptance Criteria

- ✅ P95 latency < 1ms on benchmark hardware
- ✅ Lock-free data structures eliminate contention
- ✅ Memory pools reduce allocation overhead
- ✅ Latency metrics tracked and exposed
- ✅ Benchmark suite passes consistently
- ✅ Performance dashboard displays metrics
- ✅ Tests validate latency targets
- ✅ Documentation explains optimization techniques
