use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub volume: Option<f64>,
    pub ts: i64,
}

pub async fn start_price_listener(_app_handle: AppHandle) {
    println!("Price listener initialized - will receive updates from WebSocket manager");
}

#[tauri::command]
pub async fn update_order_prices(symbol: String, price: f64) -> Result<(), String> {
    use crate::trading::limit_orders::require_state;

    let state = require_state()?;
    state.manager.update_price(&symbol, price).await;
    Ok(())
}
