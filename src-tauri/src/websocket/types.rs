use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StreamProvider {
    Birdeye,
    Helius,
}

impl StreamProvider {
    pub fn id(&self) -> &'static str {
        match self {
            StreamProvider::Birdeye => "birdeye",
            StreamProvider::Helius => "helius",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    Failed,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStatus {
    pub provider: StreamProvider,
    pub state: ConnectionState,
    pub last_message: Option<i64>,
    pub staging: bool,
    pub statistics: StreamStatistics,
    pub subscriptions: StreamSubscriptions,
    pub fallback: Option<FallbackStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FallbackStatus {
    pub active: bool,
    pub last_success: Option<i64>,
    pub interval_ms: u64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamStatistics {
    pub messages_received: u64,
    pub messages_sent: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub reconnect_count: u64,
    pub uptime_ms: u64,
    pub last_connected: Option<i64>,
    pub average_latency_ms: f64,
    pub dropped_messages: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreamSubscriptions {
    pub prices: Vec<String>,
    pub wallets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PriceSnapshot {
    pub price: f64,
    pub change: f64,
    pub volume: Option<f64>,
    pub ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceDelta {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    pub ts: i64,
    pub snapshot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionUpdate {
    pub signature: String,
    pub slot: u64,
    pub timestamp: i64,
    pub typ: Option<String>,
    pub amount: Option<f64>,
    pub symbol: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    PriceUpdate(PriceDelta),
    TransactionUpdate(TransactionUpdate),
    StatusChange(StreamStatus),
    Error {
        provider: StreamProvider,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub enum StreamCommand {
    SubscribePrices(Vec<String>),
    UnsubscribePrices(Vec<String>),
    SubscribeWallets(Vec<String>),
    UnsubscribeWallets(Vec<String>),
    Ping,
    Close,
}

#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub max_attempts: u32,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_attempts: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageQueue<T> {
    capacity: usize,
    queue: VecDeque<T>,
    dropped: u64,
}

impl<T> MessageQueue<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            queue: VecDeque::with_capacity(capacity),
            dropped: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.queue.len() >= self.capacity {
            self.queue.pop_front();
            self.dropped += 1;
        }
        self.queue.push_back(item);
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.queue.drain(..).collect()
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped
    }
}

#[derive(Debug, Clone)]
pub struct DeltaState<T> {
    pub current: HashMap<String, T>,
    pub last_emitted: HashMap<String, Instant>,
    pub throttle: Duration,
}

impl<T> DeltaState<T> {
    pub fn new(throttle: Duration) -> Self {
        Self {
            current: HashMap::new(),
            last_emitted: HashMap::new(),
            throttle,
        }
    }

    pub fn should_emit(&mut self, key: &str, now: Instant) -> bool {
        let last = self.last_emitted.get(key);
        match last {
            Some(ts) => now.duration_since(*ts) >= self.throttle,
            None => true,
        }
    }

    pub fn mark_emitted(&mut self, key: String, now: Instant) {
        self.last_emitted.insert(key, now);
    }
}
