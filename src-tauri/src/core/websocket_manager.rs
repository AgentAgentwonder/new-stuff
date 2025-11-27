use crate::market::get_coin_price;
use crate::websocket::birdeye::BirdeyeStream;
use crate::websocket::helius::HeliusStream;
use crate::websocket::reconnect::ExponentialBackoff;
use crate::websocket::types::*;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::time::{interval_at, Instant as TokioInstant};
use tokio_tungstenite::tungstenite::Message;

const PING_INTERVAL: Duration = Duration::from_secs(30);
const STALE_THRESHOLD: Duration = Duration::from_secs(60);
const QUEUE_CAPACITY: usize = 1000;
const POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_SYMBOL_BATCH: usize = 100;
const UI_BATCH_WINDOW_MS: u64 = 16;

#[derive(Clone)]
pub struct StreamConnection {
    pub provider: StreamProvider,
    pub state: Arc<RwLock<ConnectionStateInternal>>,
    pub subscriptions: Arc<RwLock<StreamSubscriptions>>,
    pub queue: Arc<Mutex<MessageQueue<StreamEvent>>>,
    pub delta_prices: Arc<Mutex<DeltaState<PriceSnapshot>>>,
    pub last_message: Arc<RwLock<Option<Instant>>>,
    pub backoff: Arc<Mutex<ExponentialBackoff>>,
    pub fallback: Arc<RwLock<FallbackState>>,
    pub statistics: Arc<RwLock<StreamStatisticsInternal>>,
    pub event_tx: broadcast::Sender<StreamEvent>,
    pub command_tx: Arc<Mutex<Option<mpsc::UnboundedSender<StreamCommand>>>>,
}

#[derive(Clone)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<StreamProvider, StreamConnection>>>,
    app_handle: AppHandle,
}

#[derive(Debug, Clone)]
pub struct FallbackState {
    pub active: bool,
    pub last_success: Option<Instant>,
    pub interval: Duration,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StreamStatisticsInternal {
    pub messages_received: u64,
    pub messages_sent: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub reconnect_count: u64,
    pub connected_at: Option<Instant>,
    pub latency_samples: VecDeque<f64>,
    pub dropped_messages: u64,
}

impl Default for StreamStatisticsInternal {
    fn default() -> Self {
        Self {
            messages_received: 0,
            messages_sent: 0,
            bytes_received: 0,
            bytes_sent: 0,
            reconnect_count: 0,
            connected_at: None,
            latency_samples: VecDeque::new(),
            dropped_messages: 0,
        }
    }
}

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum ConnectionStateInternal {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
    Failed,
    Fallback,
}

impl From<ConnectionStateInternal> for ConnectionState {
    fn from(value: ConnectionStateInternal) -> Self {
        match value {
            ConnectionStateInternal::Connecting => ConnectionState::Connecting,
            ConnectionStateInternal::Connected => ConnectionState::Connected,
            ConnectionStateInternal::Disconnecting => ConnectionState::Disconnecting,
            ConnectionStateInternal::Disconnected => ConnectionState::Disconnected,
            ConnectionStateInternal::Failed => ConnectionState::Failed,
            ConnectionStateInternal::Fallback => ConnectionState::Fallback,
        }
    }
}

impl WebSocketManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            app_handle,
        };

        manager.initialize_connection(StreamProvider::Birdeye);
        manager.initialize_connection(StreamProvider::Helius);

        manager
    }

    fn initialize_connection(&self, provider: StreamProvider) {
        let (tx, _rx) = broadcast::channel(1024);

        let connection = StreamConnection {
            provider: provider.clone(),
            state: Arc::new(RwLock::new(ConnectionStateInternal::Disconnected)),
            subscriptions: Arc::new(RwLock::new(StreamSubscriptions::default())),
            queue: Arc::new(Mutex::new(MessageQueue::with_capacity(QUEUE_CAPACITY))),
            delta_prices: Arc::new(Mutex::new(DeltaState::new(Duration::from_millis(100)))),
            last_message: Arc::new(RwLock::new(None)),
            backoff: Arc::new(Mutex::new(
                ExponentialBackoff::new(BackoffConfig::default()),
            )),
            fallback: Arc::new(RwLock::new(FallbackState {
                active: false,
                last_success: None,
                interval: POLL_INTERVAL,
                reason: None,
            })),
            statistics: Arc::new(RwLock::new(StreamStatisticsInternal::default())),
            event_tx: tx,
            command_tx: Arc::new(Mutex::new(None)),
        };

        self.connections
            .blocking_write()
            .insert(provider.clone(), connection.clone());

        let manager = self.clone();
        let provider_clone = provider.clone();
        tauri::async_runtime::spawn(async move {
            manager.start_connection(provider_clone).await;
        });

        let manager = self.clone();
        tauri::async_runtime::spawn(async move {
            manager.run_heartbeat(provider).await;
        });
    }

    async fn get_connection(&self, provider: &StreamProvider) -> Option<StreamConnection> {
        self.connections.read().await.get(provider).cloned()
    }

    async fn start_connection(&self, provider: StreamProvider) {
        if let Some(connection) = self.get_connection(&provider).await {
            self.transition_state(&connection, ConnectionStateInternal::Connecting)
                .await;
            self.emit_status(&connection).await;

            match provider {
                StreamProvider::Birdeye => {
                    let stream = BirdeyeStream::new(connection.clone(), self.app_handle.clone());
                    if let Err(err) = stream.start().await {
                        self.handle_connection_error(&connection, err).await;
                    }
                }
                StreamProvider::Helius => {
                    let stream = HeliusStream::new(connection.clone(), self.app_handle.clone());
                    if let Err(err) = stream.start().await {
                        self.handle_connection_error(&connection, err).await;
                    }
                }
            }
        }
    }

    async fn handle_connection_error(&self, connection: &StreamConnection, error: anyhow::Error) {
        {
            let mut state = connection.state.write().await;
            *state = ConnectionStateInternal::Failed;
        }
        {
            let mut fallback = connection.fallback.write().await;
            fallback.active = true;
            fallback.reason = Some(error.to_string());
        }
        self.emit_status(connection).await;

        let manager = self.clone();
        let provider = connection.provider.clone();
        tauri::async_runtime::spawn(async move {
            manager.start_polling_fallback(provider.clone()).await;
        });

        if let Some(delay) = connection.backoff.lock().await.next_delay() {
            let manager = self.clone();
            let provider = connection.provider.clone();
            self.schedule_reconnect(provider, delay, manager);
        }
    }

    fn schedule_reconnect(
        &self,
        provider: StreamProvider,
        delay: Duration,
        manager: WebSocketManager,
    ) {
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(delay).await;
            manager.start_connection(provider).await;
        });
    }

    async fn transition_state(
        &self,
        connection: &StreamConnection,
        new_state: ConnectionStateInternal,
    ) {
        {
            let mut state = connection.state.write().await;
            *state = new_state.clone();
        }

        if matches!(new_state, ConnectionStateInternal::Connected) {
            connection.backoff.lock().await.reset();
            let mut stats = connection.statistics.write().await;
            stats.connected_at = Some(Instant::now());
        }
    }

    async fn run_heartbeat(&self, provider: StreamProvider) {
        if let Some(connection) = self.get_connection(&provider).await {
            let mut ticker = interval_at(TokioInstant::now() + PING_INTERVAL, PING_INTERVAL);
            loop {
                ticker.tick().await;

                let state = connection.state.read().await.clone();
                if matches!(
                    state,
                    ConnectionStateInternal::Disconnected | ConnectionStateInternal::Failed
                ) {
                    break;
                }

                let last_message = connection.last_message.read().await.clone();
                if let Some(last) = last_message {
                    if last.elapsed() > STALE_THRESHOLD {
                        self.force_reconnect(&connection, "heartbeat stale").await;
                    }
                } else {
                    self.force_reconnect(&connection, "no message received")
                        .await;
                }
            }
        }
    }

    pub async fn force_reconnect(&self, connection: &StreamConnection, reason: &str) {
        {
            let mut state = connection.state.write().await;
            *state = ConnectionStateInternal::Disconnecting;
        }

        {
            let mut fallback = connection.fallback.write().await;
            fallback.active = true;
            fallback.reason = Some(reason.to_string());
        }
        self.emit_status(connection).await;
        self.start_polling_fallback(connection.provider.clone())
            .await;

        let delay = connection
            .backoff
            .lock()
            .await
            .next_delay()
            .unwrap_or(Duration::from_secs(60));
        self.schedule_reconnect(connection.provider.clone(), delay, self.clone());
    }

    async fn emit_status(&self, connection: &StreamConnection) {
        if let Ok(status) = self.current_status(connection).await {
            let _ = connection
                .event_tx
                .send(StreamEvent::StatusChange(status.clone()));
            let _ = self.app_handle.emit("stream_status_change", &status);
        }
    }

    async fn emit_error(&self, connection: &StreamConnection, message: String) {
        let event = StreamEvent::Error {
            provider: connection.provider.clone(),
            message: message.clone(),
        };
        let _ = connection.event_tx.send(event.clone());
        let _ = self.app_handle.emit("stream_error", &event);
    }

    pub async fn subscribe_prices(&self, symbols: Vec<String>) -> anyhow::Result<()> {
        let connection = self
            .get_connection(&StreamProvider::Birdeye)
            .await
            .ok_or_else(|| anyhow::anyhow!("Birdeye connection not available"))?;

        let mut subs = connection.subscriptions.write().await;
        let mut unique_symbols: HashSet<String> = subs.prices.iter().cloned().collect();
        let mut new_symbols: Vec<String> = Vec::new();

        for symbol in symbols {
            if unique_symbols.insert(symbol.clone()) {
                new_symbols.push(symbol.clone());
                subs.prices.push(symbol);
            }
        }
        drop(subs);

        if new_symbols.is_empty() {
            return Ok(());
        }

        let mut batches: Vec<Vec<String>> = Vec::new();
        let mut current_batch: Vec<String> = Vec::new();

        for symbol in new_symbols {
            current_batch.push(symbol);
            if current_batch.len() >= MAX_SYMBOL_BATCH {
                batches.push(current_batch);
                current_batch = Vec::new();
            }
        }

        if !current_batch.is_empty() {
            batches.push(current_batch);
        }

        for batch in batches {
            if batch.is_empty() {
                continue;
            }

            let command_tx = connection.command_tx.lock().await;
            if let Some(ref tx) = *command_tx {
                let _ = tx.send(StreamCommand::SubscribePrices(batch));
            }
        }

        Ok(())
    }

    pub async fn unsubscribe_prices(&self, symbols: Vec<String>) -> anyhow::Result<()> {
        let connection = self
            .get_connection(&StreamProvider::Birdeye)
            .await
            .ok_or_else(|| anyhow::anyhow!("Birdeye connection not available"))?;

        let mut subs = connection.subscriptions.write().await;
        let mut to_remove = Vec::new();
        for symbol in symbols {
            if subs.prices.contains(&symbol) {
                to_remove.push(symbol);
            }
        }

        if !to_remove.is_empty() {
            subs.prices.retain(|s| !to_remove.contains(s));
            drop(subs);

            let command_tx = connection.command_tx.lock().await;
            if let Some(ref tx) = *command_tx {
                let _ = tx.send(StreamCommand::UnsubscribePrices(to_remove));
            }
        }

        Ok(())
    }

    pub async fn subscribe_wallets(&self, addresses: Vec<String>) -> anyhow::Result<()> {
        let connection = self
            .get_connection(&StreamProvider::Helius)
            .await
            .ok_or_else(|| anyhow::anyhow!("Helius connection not available"))?;

        let mut subs = connection.subscriptions.write().await;
        let mut new_addresses = Vec::new();
        for address in addresses {
            if !subs.wallets.contains(&address) {
                new_addresses.push(address.clone());
                subs.wallets.push(address);
            }
        }

        if !new_addresses.is_empty() {
            drop(subs);
            let command_tx = connection.command_tx.lock().await;
            if let Some(ref tx) = *command_tx {
                let _ = tx.send(StreamCommand::SubscribeWallets(new_addresses));
            }
        }

        Ok(())
    }

    pub async fn unsubscribe_wallets(&self, addresses: Vec<String>) -> anyhow::Result<()> {
        let connection = self
            .get_connection(&StreamProvider::Helius)
            .await
            .ok_or_else(|| anyhow::anyhow!("Helius connection not available"))?;

        let mut subs = connection.subscriptions.write().await;
        let mut to_remove = Vec::new();
        for address in addresses {
            if subs.wallets.contains(&address) {
                to_remove.push(address.clone());
            }
        }

        if !to_remove.is_empty() {
            subs.wallets.retain(|a| !to_remove.contains(a));
            drop(subs);

            let command_tx = connection.command_tx.lock().await;
            if let Some(ref tx) = *command_tx {
                let _ = tx.send(StreamCommand::UnsubscribeWallets(to_remove));
            }
        }

        Ok(())
    }

    pub async fn get_status(&self) -> Vec<StreamStatus> {
        let mut statuses = Vec::new();
        let connections = self.connections.read().await;
        for (_, conn) in connections.iter() {
            if let Ok(status) = self.current_status(conn).await {
                statuses.push(status);
            }
        }
        statuses
    }

    pub async fn reconnect(&self, provider: StreamProvider) -> anyhow::Result<()> {
        let connection = self
            .get_connection(&provider)
            .await
            .ok_or_else(|| anyhow::anyhow!("Connection not found"))?;

        self.force_reconnect(&connection, "manual reconnect").await;
        Ok(())
    }

    async fn current_status(&self, connection: &StreamConnection) -> anyhow::Result<StreamStatus> {
        let state = connection.state.read().await.clone();
        let last_message = connection
            .last_message
            .read()
            .await
            .clone()
            .map(|ts| ts.elapsed().as_millis() as i64);
        let subscriptions = connection.subscriptions.read().await.clone();
        let fallback = connection.fallback.read().await.clone();
        let stats = connection.statistics.read().await.clone();

        Ok(StreamStatus {
            provider: connection.provider.clone(),
            state: state.into(),
            last_message,
            staging: false,
            statistics: StreamStatistics {
                messages_received: stats.messages_received,
                messages_sent: stats.messages_sent,
                bytes_received: stats.bytes_received,
                bytes_sent: stats.bytes_sent,
                reconnect_count: stats.reconnect_count,
                uptime_ms: stats
                    .connected_at
                    .map(|ts| ts.elapsed().as_millis() as u64)
                    .unwrap_or_default(),
                last_connected: stats.connected_at.map(|ts| {
                    (chrono::Utc::now() - chrono::Duration::from_std(ts.elapsed()).unwrap())
                        .timestamp()
                }),
                average_latency_ms: if stats.latency_samples.is_empty() {
                    0.0
                } else {
                    stats.latency_samples.iter().sum::<f64>() / stats.latency_samples.len() as f64
                },
                dropped_messages: stats.dropped_messages,
            },
            subscriptions,
            fallback: Some(FallbackStatus {
                active: fallback.active,
                last_success: fallback.last_success.map(|ts| {
                    (chrono::Utc::now() - chrono::Duration::from_std(ts.elapsed()).unwrap())
                        .timestamp()
                }),
                interval_ms: fallback.interval.as_millis() as u64,
                reason: fallback.reason.clone(),
            }),
        })
    }

    pub async fn start_polling_fallback(&self, provider: StreamProvider) {
        if let Some(connection) = self.get_connection(&provider).await {
            {
                let mut state = connection.state.write().await;
                *state = ConnectionStateInternal::Fallback;
            }
            self.emit_status(&connection).await;

            let manager = self.clone();
            tauri::async_runtime::spawn(async move {
                manager.process_polling(provider).await;
            });
        }
    }

    pub async fn process_polling(&self, provider: StreamProvider) {
        if let Some(connection) = self.get_connection(&provider).await {
            let fallback_active = connection.fallback.read().await.active;
            if !fallback_active {
                return;
            }

            let mut interval = tokio::time::interval(connection.fallback.read().await.interval);

            loop {
                interval.tick().await;

                if !connection.fallback.read().await.active {
                    break;
                }

                let subs = connection.subscriptions.read().await.clone();

                match provider {
                    StreamProvider::Birdeye => {
                        for symbol in &subs.prices {
                            if let Ok(price) = get_coin_price(symbol.clone(), None).await {
                                let delta = PriceDelta {
                                    symbol: symbol.clone(),
                                    price: Some(price.price),
                                    change: Some(price.price_change_24h),
                                    volume: Some(price.volume_24h),
                                    ts: chrono::Utc::now().timestamp(),
                                    snapshot: true,
                                };
                                self.enqueue_event(&connection, StreamEvent::PriceUpdate(delta))
                                    .await;
                            }
                        }
                    }
                    StreamProvider::Helius => {
                        // Polling fallback for Helius transactions is not implemented here due to API limitations
                    }
                }

                {
                    let mut fallback = connection.fallback.write().await;
                    fallback.last_success = Some(Instant::now());
                }
            }
        }
    }

    pub async fn enqueue_event(&self, connection: &StreamConnection, event: StreamEvent) {
        {
            let mut queue = connection.queue.lock().await;
            queue.push(event.clone());
            let dropped = queue.dropped_count();
            let mut stats = connection.statistics.write().await;
            stats.dropped_messages = dropped;
        }

        match &event {
            StreamEvent::PriceUpdate(delta) => {
                let _ = self.app_handle.emit("price_update", delta);
            }
            StreamEvent::TransactionUpdate(tx) => {
                let _ = self.app_handle.emit("transaction_update", tx);
            }
            StreamEvent::StatusChange(status) => {
                let _ = self.app_handle.emit("stream_status_change", status);
            }
            StreamEvent::Error { .. } => {}
        }

        let _ = connection.event_tx.send(event);
    }

    pub async fn drain_queue(&self, provider: StreamProvider) -> Vec<StreamEvent> {
        if let Some(connection) = self.get_connection(&provider).await {
            let mut queue = connection.queue.lock().await;
            queue.drain()
        } else {
            Vec::new()
        }
    }

    pub fn subscribe_events(
        &self,
        provider: StreamProvider,
    ) -> Option<broadcast::Receiver<StreamEvent>> {
        self.connections
            .blocking_read()
            .get(&provider)
            .map(|conn| conn.event_tx.subscribe())
    }
}
