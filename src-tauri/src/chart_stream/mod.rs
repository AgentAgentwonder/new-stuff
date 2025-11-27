use crate::core::price_engine::{get_price_engine, PriceUpdate};
use crate::core::WebSocketManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPriceUpdate {
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub timestamp: u64,
    pub change_24h: f64,
}

#[derive(Default)]
struct ChartSubscriptions {
    symbols: HashMap<String, ChartSubscriptionInfo>,
}

struct ChartSubscriptionInfo {
    interval_ms: u64,
    ref_count: u32,
}

lazy_static::lazy_static! {
    static ref CHART_SUBS: Arc<RwLock<ChartSubscriptions>> = Arc::new(RwLock::new(ChartSubscriptions::default()));
}

/// Subscribe to high-frequency chart price updates
#[tauri::command]
pub async fn subscribe_chart_prices(
    app_handle: AppHandle,
    ws_manager: State<'_, WebSocketManager>,
    symbol: String,
    interval_ms: Option<u64>,
) -> Result<(), String> {
    let interval_ms = interval_ms.unwrap_or(1000); // Default to 1 second

    // Subscribe to the WebSocket manager for real-time updates
    ws_manager
        .subscribe_prices(vec![symbol.clone()])
        .await
        .map_err(|e| e.to_string())?;

    // Track subscription with ref counting
    let should_start_task = {
        let mut subs = CHART_SUBS.write().await;
        let entry = subs
            .symbols
            .entry(symbol.clone())
            .or_insert(ChartSubscriptionInfo {
                interval_ms,
                ref_count: 0,
            });
        entry.ref_count += 1;
        entry.ref_count == 1
    };

    if !should_start_task {
        // Already running
        return Ok(());
    }

    // Start emission task for this symbol if not already running
    let app_handle_clone = app_handle.clone();
    let symbol_clone = symbol.clone();

    tokio::spawn(async move {
        loop {
            let interval_ms = {
                let subs = CHART_SUBS.read().await;
                subs.symbols.get(&symbol_clone).map(|info| info.interval_ms)
            };

            let interval_ms = match interval_ms {
                Some(value) => value,
                None => break,
            };

            tokio::time::sleep(Duration::from_millis(interval_ms)).await;

            // After sleep, verify still subscribed
            let still_subscribed = {
                let subs = CHART_SUBS.read().await;
                subs.symbols.contains_key(&symbol_clone)
            };

            if !still_subscribed {
                break;
            }

            // Get latest price from engine
            let engine = get_price_engine();
            if let Some(cached_price) = engine.get_cached_price(&symbol_clone) {
                let update = ChartPriceUpdate {
                    symbol: symbol_clone.clone(),
                    price: cached_price.price,
                    volume: cached_price.volume,
                    timestamp: cached_price.timestamp,
                    change_24h: cached_price.change_24h,
                };

                // Emit event to frontend
                let _ = app_handle_clone.emit("chart_price_update", &update);
            }
        }
    });

    Ok(())
}

impl Clone for ChartSubscriptionInfo {
    fn clone(&self) -> Self {
        Self {
            interval_ms: self.interval_ms,
            ref_count: self.ref_count,
        }
    }
}

/// Unsubscribe from chart price updates
#[tauri::command]
pub async fn unsubscribe_chart_prices(
    ws_manager: State<'_, WebSocketManager>,
    symbol: String,
) -> Result<(), String> {
    // Decrement ref count and potentially remove
    let should_unsubscribe = {
        let mut subs = CHART_SUBS.write().await;
        if let Some(info) = subs.symbols.get_mut(&symbol) {
            info.ref_count = info.ref_count.saturating_sub(1);
            if info.ref_count == 0 {
                subs.symbols.remove(&symbol);
                true
            } else {
                false
            }
        } else {
            false
        }
    };

    // Only unsubscribe from WebSocket if no more refs
    if should_unsubscribe {
        ws_manager
            .unsubscribe_prices(vec![symbol])
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get current chart subscription status
#[tauri::command]
pub async fn get_chart_subscriptions() -> Result<Vec<String>, String> {
    let subs = CHART_SUBS.read().await;
    Ok(subs.symbols.keys().cloned().collect())
}
