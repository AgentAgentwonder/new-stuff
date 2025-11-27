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
use url::Url;

const HELIUS_WS_URL: &str = "wss://mainnet.helius-rpc.com/?api-key=YOUR_KEY";

pub struct HeliusStream {
    connection: StreamConnection,
    app_handle: AppHandle,
}

impl HeliusStream {
    pub fn new(connection: StreamConnection, app_handle: AppHandle) -> Self {
        Self {
            connection,
            app_handle,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let url = url::Url::parse(HELIUS_WS_URL)?;
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

        self.handle_stream(ws_stream).await
    }

    async fn handle_stream(
        &self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> anyhow::Result<()> {
        let (ws_stream_tx, mut ws_stream_rx) = ws_stream.split();
        let write = Arc::new(Mutex::new(ws_stream_tx));

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
                    StreamCommand::SubscribeWallets(addresses) => {
                        let msg = json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "accountSubscribe",
                            "params": addresses
                        });
                        if let Err(e) = writer.send(Message::Text(msg.to_string())).await {
                            eprintln!("Failed to send subscribe command: {}", e);
                        }
                        let mut stats = connection_clone.statistics.write().await;
                        stats.messages_sent += 1;
                    }
                    StreamCommand::UnsubscribeWallets(addresses) => {
                        let msg = json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "accountUnsubscribe",
                            "params": addresses
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

        let existing_addresses = self.connection.subscriptions.read().await.wallets.clone();
        if !existing_addresses.is_empty() {
            let mut writer = write.lock().await;
            let msg = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "accountSubscribe",
                "params": existing_addresses
            });
            writer.send(Message::Text(msg.to_string())).await?;
        }

        while let Some(msg) = ws_stream_rx.next().await {
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
        if let Some(method) = value.get("method").and_then(|v| v.as_str()) {
            if method == "accountNotification" || method == "notification" {
                if let Ok(tx) = self.parse_transaction(&value) {
                    let event = StreamEvent::TransactionUpdate(tx);
                    let _ = self.connection.event_tx.send(event.clone());
                    let _ = self.app_handle.emit("transaction_update", &event);

                    let mut queue = self.connection.queue.lock().await;
                    queue.push(event);
                }
            }
        }
    }

    fn parse_transaction(
        &self,
        value: &serde_json::Value,
    ) -> anyhow::Result<TransactionUpdate> {
        let params = value
            .get("params")
            .and_then(|v| v.get("result"))
            .ok_or_else(|| anyhow::anyhow!("Missing params parameter"))?;

        Ok(TransactionUpdate {
            signature: params
                .get("signature")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            slot: params
                .get("slot")
                .and_then(|v| v.as_u64())
                .unwrap_or_default(),
            timestamp: params
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or_else(|| chrono::Utc::now().timestamp()),
            typ: params
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            amount: params.get("amount").and_then(|v| v.as_f64()),
            symbol: params
                .get("symbol")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            from: params
                .get("from")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            to: params
                .get("to")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
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

    pub async fn subscribe(
        &self,
        connection: StreamConnection,
        addresses: Vec<String>,
    ) -> anyhow::Result<()> {
        // Actual subscription logic must send a message to the WebSocket stream
        // Requires connection to have access to the writer handle - omitted for brevity
        let mut subs = connection.subscriptions.write().await;
        for address in addresses {
            if !subs.wallets.contains(&address) {
                subs.wallets.push(address);
            }
        }
        Ok(())
    }

    pub async fn unsubscribe(
        &self,
        connection: StreamConnection,
        addresses: Vec<String>,
    ) -> anyhow::Result<()> {
        let mut subs = connection.subscriptions.write().await;
        subs.wallets.retain(|a| !addresses.contains(a));
        Ok(())
    }
}