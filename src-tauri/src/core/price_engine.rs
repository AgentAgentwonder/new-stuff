use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use crossbeam::queue::SegQueue;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, trace, warn};

const LATENCY_WINDOW: usize = 10_000;
const MEMORY_POOL_CAPACITY: usize = 512;
const MEMORY_POOL_BUFFER_SIZE: usize = 1024;

#[inline]
fn nanos_to_micros(value: u64) -> f64 {
    value as f64 / 1_000.0
}

#[inline]
fn percentile(sorted: &[u64], quantile: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }

    let max_index = sorted.len() - 1;
    let idx = ((max_index as f64) * quantile).round() as usize;
    sorted[idx.min(max_index)]
}

/// Zero-copy friendly price update payload that flows through the engine pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub symbol: Cow<'static, str>,
    pub price: f64,
    pub volume: f64,
    pub timestamp: u64,
    pub change_24h: f64,
}

impl PriceUpdate {
    pub fn new(symbol: String, price: f64, volume: f64, change_24h: f64) -> Self {
        Self {
            symbol: Cow::Owned(symbol),
            price,
            volume,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            change_24h,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyStats {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency: LatencyStats,
    pub throughput: f64,
    pub messages_received: u64,
    pub messages_processed: u64,
    pub errors: u64,
    pub uptime_ms: u64,
    pub cpu_usage: f32,
}

struct LatencyTracker {
    samples: Mutex<VecDeque<u64>>,
    capacity: usize,
}

impl LatencyTracker {
    fn new(capacity: usize) -> Self {
        Self {
            samples: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    fn record(&self, latency_ns: u64) {
        let mut guard = self.samples.lock();
        if guard.len() == self.capacity {
            guard.pop_front();
        }
        guard.push_back(latency_ns);
    }

    fn reset(&self) {
        self.samples.lock().clear();
    }

    fn snapshot(&self) -> LatencyStats {
        let guard = self.samples.lock();
        if guard.is_empty() {
            return LatencyStats::default();
        }

        let mut sorted: Vec<u64> = guard.iter().copied().collect();
        sorted.sort_unstable();

        let count = sorted.len();
        let total: u64 = sorted.iter().sum();
        let mean_ns = total / count as u64;

        LatencyStats {
            p50: nanos_to_micros(percentile(&sorted, 0.50)),
            p95: nanos_to_micros(percentile(&sorted, 0.95)),
            p99: nanos_to_micros(percentile(&sorted, 0.99)),
            mean: nanos_to_micros(mean_ns),
            min: nanos_to_micros(*sorted.first().unwrap()),
            max: nanos_to_micros(*sorted.last().unwrap()),
            sample_count: count,
        }
    }
}

struct MemoryPool {
    buffers: SegQueue<Vec<u8>>,
    buffer_size: usize,
}

impl MemoryPool {
    fn new(capacity: usize, buffer_size: usize) -> Self {
        let pool = SegQueue::new();
        for _ in 0..capacity {
            pool.push(Vec::with_capacity(buffer_size));
        }
        Self {
            buffers: pool,
            buffer_size,
        }
    }

    fn acquire(&self) -> Vec<u8> {
        self.buffers
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }

    fn release(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        if buffer.capacity() > (self.buffer_size * 4) {
            buffer.shrink_to(self.buffer_size);
        }
        self.buffers.push(buffer);
    }
}

struct PooledPayload {
    entered_at: Instant,
    bytes: Bytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPrice {
    pub price: f64,
    pub volume: f64,
    pub change_24h: f64,
    pub timestamp: u64,
}

pub struct PriceEngine {
    payload_queue: SegQueue<PooledPayload>,
    memory_pool: MemoryPool,
    received: AtomicU64,
    processed: AtomicU64,
    errors: AtomicU64,
    latency: LatencyTracker,
    prices: RwLock<HashMap<String, CachedPrice>>,
    start_time: Mutex<Instant>,
    sys_info: Mutex<System>,
}

impl PriceEngine {
    pub fn new() -> Self {
        let mut sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        sys.refresh_cpu();

        Self {
            payload_queue: SegQueue::new(),
            memory_pool: MemoryPool::new(MEMORY_POOL_CAPACITY, MEMORY_POOL_BUFFER_SIZE),
            received: AtomicU64::new(0),
            processed: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            latency: LatencyTracker::new(LATENCY_WINDOW),
            prices: RwLock::new(HashMap::new()),
            start_time: Mutex::new(Instant::now()),
            sys_info: Mutex::new(sys),
        }
    }

    pub fn process_update(&self, update: PriceUpdate) {
        self.received.fetch_add(1, Ordering::Relaxed);

        {
            let mut prices = self.prices.write();
            prices.insert(
                update.symbol.to_string(),
                CachedPrice {
                    price: update.price,
                    volume: update.volume,
                    change_24h: update.change_24h,
                    timestamp: update.timestamp,
                },
            );
        }

        let serialized = match serde_json::to_vec(&update) {
            Ok(bytes) => bytes,
            Err(err) => {
                warn!("failed to serialize price update: {err}");
                self.errors.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        let mut buffer = self.memory_pool.acquire();
        if buffer.capacity() < serialized.len() {
            buffer = Vec::with_capacity(serialized.len().next_power_of_two());
        }
        buffer.clear();
        buffer.extend_from_slice(&serialized);

        let payload = PooledPayload {
            entered_at: Instant::now(),
            bytes: Bytes::copy_from_slice(&buffer),
        };

        self.memory_pool.release(buffer);
        self.payload_queue.push(payload);
        self.drain_queue();
    }

    fn drain_queue(&self) {
        while let Some(payload) = self.payload_queue.pop() {
            let latency_ns = payload.entered_at.elapsed().as_nanos() as u64;
            self.latency.record(latency_ns);

            if let Err(err) = self.handle_payload(&payload.bytes) {
                warn!("failed to process payload: {err}");
                self.errors.fetch_add(1, Ordering::Relaxed);
            } else {
                self.processed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn handle_payload(&self, bytes: &Bytes) -> Result<(), serde_json::Error> {
        // Deserialize to ensure zero-copy path remains hot
        let decoded: PriceUpdate = serde_json::from_slice(bytes)?;
        trace!(symbol = %decoded.symbol, price = decoded.price, "processed price update");
        Ok(())
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        let latency = self.latency.snapshot();
        let received = self.received.load(Ordering::Relaxed);
        let processed = self.processed.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let start = self.start_time.lock();
        let uptime_ms = start.elapsed().as_millis() as u64;

        let throughput = if uptime_ms > 0 {
            (processed as f64 / uptime_ms as f64) * 1000.0
        } else {
            0.0
        };

        let mut sys = self.sys_info.lock();
        sys.refresh_cpu();
        let cpu_usage = sys.global_cpu_info().cpu_usage();

        PerformanceMetrics {
            latency,
            throughput,
            messages_received: received,
            messages_processed: processed,
            errors,
            uptime_ms,
            cpu_usage,
        }
    }

    pub fn reset_stats(&self) {
        self.received.store(0, Ordering::Relaxed);
        self.processed.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
        self.latency.reset();
        *self.start_time.lock() = Instant::now();
    }

    pub fn get_price(&self, symbol: &str) -> Option<f64> {
        let prices = self.prices.read();
        prices.get(symbol).map(|entry| entry.price)
    }

    pub fn get_cached_price(&self, symbol: &str) -> Option<CachedPrice> {
        let prices = self.prices.read();
        prices.get(symbol).cloned()
    }

    pub async fn run_performance_test(&self, num_updates: usize) -> PerformanceMetrics {
        info!("running performance test with {} updates", num_updates);
        self.reset_stats();

        for i in 0..num_updates {
            let symbol = format!("SYM{}", i % 64);
            let price = 100.0 + rand::random_range(-5.0..5.0);
            let volume = rand::random_range(1_000.0..100_000.0);
            let change_24h = rand::random_range(-10.0..10.0);

            let update = PriceUpdate::new(symbol, price, volume, change_24h);
            self.process_update(update);
        }

        self.get_metrics()
    }
}

impl Default for PriceEngine {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    static ref PRICE_ENGINE: Arc<PriceEngine> = Arc::new(PriceEngine::new());
}

pub fn get_price_engine() -> Arc<PriceEngine> {
    Arc::clone(&PRICE_ENGINE)
}

#[tauri::command]
pub fn get_performance_metrics() -> Result<PerformanceMetrics, String> {
    Ok(get_price_engine().get_metrics())
}

#[tauri::command]
pub async fn run_performance_test(
    num_updates: Option<usize>,
) -> Result<PerformanceMetrics, String> {
    let count = num_updates.unwrap_or(10_000);
    let metrics = get_price_engine().run_performance_test(count).await;
    Ok(metrics)
}

#[tauri::command]
pub fn reset_performance_stats() -> Result<(), String> {
    get_price_engine().reset_stats();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_price_update() {
        let engine = PriceEngine::new();
        let update = PriceUpdate::new("SOL".to_string(), 100.25, 50_000.0, 3.2);
        engine.process_update(update);

        let metrics = engine.get_metrics();
        assert_eq!(metrics.messages_processed, 1);
        assert!(metrics.latency.sample_count >= 1);
        assert_eq!(engine.get_price("SOL"), Some(100.25));

        let cached = engine.get_cached_price("SOL");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().price, 100.25);
    }

    #[test]
    fn resets_statistics() {
        let engine = PriceEngine::new();
        engine.process_update(PriceUpdate::new("SOL".to_string(), 100.0, 10.0, 0.0));
        engine.reset_stats();
        let metrics = engine.get_metrics();
        assert_eq!(metrics.messages_processed, 0);
        assert_eq!(metrics.latency.sample_count, 0);
    }

    #[tokio::test]
    async fn performance_test_runs() {
        let engine = PriceEngine::new();
        let metrics = engine.run_performance_test(1_000).await;
        assert_eq!(metrics.messages_processed, 1_000);
        assert!(metrics.throughput > 0.0);
        assert!(metrics.latency.p95 >= 0.0);
    }
}
