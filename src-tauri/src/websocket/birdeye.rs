use crate::core::websocket_manager::{ConnectionStateInternal, StreamConnection};
use crate::websocket::types::*;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

const BIRDEYE_WS_URL: &str = "wss://public-api.birdeye.so/socket";

pub struct BirdeyeStream {
    connection: StreamConnection,
    app_handle: AppHandle,
}

struct RawPriceUpdate {
    symbol: String,
    price: Option<f64>,
    change: Option<f64>,
    volume: Option<f64>,
    snapshot: bool,
    ts: i64,
}

impl BirdeyeStream {
    pub fn new(connection: StreamConnection, app_handle: AppHandle) -> Self {
        Self {
            connection,
            app_handle,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let url = url::Url::parse(BIRDEYE_WS_URL)?;
        let (ws_stream, _) = connect_async(url).await?;

        {
            let mut state = self.connection.state.write().await;
            *state = ConnectionStateInternal::Connected;
        }
        {
            let mut fallback = self.connection.fallback.write().await;
            fallback.active = false;
            fallback.reason = None;
        }

        self.emit_status().await;
        self.handle_stream(ws_stream).await
    }

    async fn handle_stream(
        &self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> anyhow::Result<()> {
        let (write, mut read) = ws_stream.split();
        let write = Arc::new(Mutex::new(write));

        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<StreamCommand>();
        {
            let mut command_tx = self.connection.command_tx.lock().await;
            *command_tx = Some(cmd_tx);
        }

        let write_clone = write.clone();
        let connection_clone = self.connection.clone();

        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                let mut writer = write_clone.lock().await;
                match cmd {
                    StreamCommand::SubscribePrices(symbols) => {
                        let msg = json!({
                            "type": "subscribe",
                            "data": {
                                "channel": "prices",
                                "symbols": symbols
                            }
                        });
                        if let Err(e) = writer.send(Message::Text(msg.to_string())).await {
                            eprintln!("Failed to send subscribe command: {}", e);
                        }
                        let mut stats = connection_clone.statistics.write().await;
                        stats.messages_sent += 1;
                    }
                    StreamCommand::UnsubscribePrices(symbols) => {
                        let msg = json!({
                            "type": "unsubscribe",
                            "data": {
                                "channel": "prices",
                                "symbols": symbols
                            }
                        });
                        if let Err(e) = writer.send(Message::Text(msg.to_string())).await {
                            eprintln!("Failed to send unsubscribe command: {}", e);
                        }
                        let mut stats = connection_clone.statistics.write().await;
                        stats.messages_sent += 1;
                    }
                    StreamCommand::Ping => {
                        if let Err(e) = writer.send(Message::Ping(vec![])).await {
                            eprintln!("Failed to send ping: {}", e);
                        }
                    }
                    StreamCommand::Close => {
                        let _ = writer.send(Message::Close(None)).await;
                        break;
                    }
                    _ => {}
                }
            }
        });

        let write_clone = write.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let mut writer = write_clone.lock().await;
                if writer.send(Message::Ping(vec![])).await.is_err() {
                    break;
                }
            }
        });

        let existing_symbols = self.connection.subscriptions.read().await.prices.clone();
        if !existing_symbols.is_empty() {
            let mut writer = write.lock().await;
            let msg = json!({
            "type": "subscribe",
            "data": {
            "channel": "prices",
            "symbols": existing_symbols
            }
            });
            writer.send(Message::Text(msg.to_string())).await?;
        }

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    self.update_last_message().await;
                    self.increment_stats(text.len()).await;

                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        self.process_message(value).await;
                    }
                }
                Ok(Message::Binary(data)) => {
                    self.update_last_message().await;
                    self.increment_stats(data.len()).await;

                    if let Ok(value) = rmp_serde::from_slice::<serde_json::Value>(&data) {
                        self.process_message(value).await;
                    }
                }
                Ok(Message::Ping(_)) => {
                    let mut writer = write.lock().await;
                    if let Err(e) = writer.send(Message::Pong(vec![])).await {
                        return Err(anyhow::anyhow!("Failed to send pong: {}", e));
                    }
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("WebSocket error: {}", e));
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn process_message(&self, value: serde_json::Value) {
        if let Some(typ) = value.get("type").and_then(|v| v.as_str()) {
            match typ {
                "price" | "delta" => {
                    if let Ok(raw) = self.parse_raw_price_update(&value) {
                        let delta = self.merge_delta(raw).await;
                        if let Some(delta) = delta {
                            let mut delta_state = self.connection.delta_prices.lock().await;
                            let now = Instant::now();
                            if delta_state.should_emit(&delta.symbol, now) {
                                delta_state.mark_emitted(delta.symbol.clone(), now);
                                drop(delta_state);
                                let event = StreamEvent::PriceUpdate(delta);
                                self.enqueue_event(event).await;
                            }
                        }
                    }
                }
                "snapshot" => {
                    if let Ok(raw) = self.parse_raw_price_update(&value) {
                        if let Some(delta) = self.merge_delta(raw).await {
                            let event = StreamEvent::PriceUpdate(delta);
                            self.enqueue_event(event).await;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    async fn merge_delta(&self, raw: RawPriceUpdate) -> Option<PriceDelta> {
        let mut delta_state = self.connection.delta_prices.lock().await;

        let snapshot_entry = delta_state
            .current
            .entry(raw.symbol.clone())
            .or_insert_with(PriceSnapshot::default);

        if let Some(price) = raw.price {
            snapshot_entry.price = price;
        }
        if let Some(change) = raw.change {
            snapshot_entry.change = change;
        }
        if raw.volume.is_some() {
            snapshot_entry.volume = raw.volume;
        }
        snapshot_entry.ts = raw.ts;

        let mut delta = PriceDelta {
            symbol: raw.symbol,
            price: None,
            change: None,
            volume: None,
            ts: raw.ts,
            snapshot: raw.snapshot,
        };

        if raw.snapshot {
            delta.price = Some(snapshot_entry.price);
            delta.change = Some(snapshot_entry.change);
            delta.volume = snapshot_entry.volume;
        } else {
            if let Some(price) = raw.price {
                delta.price = Some(price);
            }
            if let Some(change) = raw.change {
                delta.change = Some(change);
            }
            if raw.volume.is_some() {
                delta.volume = raw.volume;
            }
        }

        if delta.snapshot
            || delta.price.is_some()
            || delta.change.is_some()
            || delta.volume.is_some()
        {
            Some(delta)
        } else {
            None
        }
    }

    fn parse_raw_price_update(&self, value: &serde_json::Value) -> anyhow::Result<RawPriceUpdate> {
        let data = value.get("data").unwrap_or(value);
        let msg_type = value.get("type").and_then(|v| v.as_str());

        let symbol = data
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing symbol"))?
            .to_string();
        let price = data
            .get("price")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow::anyhow!("Missing price"))?;
        let change = data
            .get("change_24h")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let volume = data.get("volume_24h").and_then(|v| v.as_f64());

        Ok(RawPriceUpdate {
            symbol,
            price: Some(price),
            change: Some(change),
            volume,
            snapshot: msg_type == Some("snapshot"),
            ts: chrono::Utc::now().timestamp(),
        })
    }

    async fn update_last_message(&self) {
        let mut last = self.connection.last_message.write().await;
        *last = Some(Instant::now());
    }

    async fn increment_stats(&self, bytes: usize) {
        let mut stats = self.connection.statistics.write().await;
        stats.messages_received += 1;
        stats.bytes_received += bytes as u64;
    }

    async fn emit_status(&self) {}

    async fn enqueue_event(&self, event: StreamEvent) {
        let mut queue = self.connection.queue.lock().await;
        queue.push(event.clone());
        drop(queue);

        match &event {
            StreamEvent::PriceUpdate(delta) => {
                let _ = self.app_handle.emit("price_update", delta);
            }
            _ => {}
        }

        let _ = self.connection.event_tx.send(event);
    }

    pub async fn subscribe(
        connection: StreamConnection,
        symbols: Vec<String>,
    ) -> anyhow::Result<()> {
        let mut subs = connection.subscriptions.write().await;
        subs.prices.extend(symbols.clone());
        drop(subs);

        // Birdeye subscription message format:
        // {"type": "subscribe", "symbols": ["SOL", "BONK", ...]}
        // This is a placeholder - actual implementation depends on Birdeye API docs
        Ok(())
    }

    pub async fn unsubscribe(
        connection: StreamConnection,
        symbols: Vec<String>,
    ) -> anyhow::Result<()> {
        let mut subs = connection.subscriptions.write().await;
        subs.prices.retain(|s| !symbols.contains(s));
        drop(subs);

        // Birdeye unsubscribe message format:
        // {"type": "unsubscribe", "symbols": ["SOL", "BONK", ...]}
        // This is a placeholder - actual implementation depends on Birdeye API docs
        Ok(())
    }
}
