use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Manager, Window};
use serde::Serialize;

#[derive(Clone, Serialize)]
struct PriceUpdate {
    symbol: String,
    price: f64,
    change: f64,
    timestamp: i64,
}

lazy_static::lazy_static! {
    static ref ACTIVE_STREAMS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
}

#[tauri::command]
pub async fn start_price_stream(symbol: String, window: Window) -> Result<(), String> {
    let mut streams = ACTIVE_STREAMS.lock().await;
    
    if streams.contains(&symbol) {
        return Ok(());
    }
    
    streams.push(symbol.clone());
    
    // Spawn background task for price updates
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        
        loop {
            interval.tick().await;
            
            // Check if stream is still active
            let streams = ACTIVE_STREAMS.lock().await;
            if !streams.contains(&symbol) {
                break;
            }
            drop(streams);
            
            // Generate mock price update
            use rand::Rng;
            
            let base_price = match symbol.as_str() {
                "SOL" => 100.0,
                "BONK" => 0.000023,
                "JUP" => 1.23,
                _ => 1.0,
            };
            
            let update = PriceUpdate {
                symbol: symbol.clone(),
                price: base_price * (1.0 + rand::random_range(-0.02..0.02)),
                change: rand::random_range(-5.0..5.0),
                timestamp: chrono::Utc::now().timestamp(),
            };
            
            // Emit to frontend
            let _ = window.emit("price-update", &update);
        }
    });
    
    Ok(())
}

#[tauri::command]
pub async fn stop_price_stream(symbol: String) -> Result<(), String> {
    let mut streams = ACTIVE_STREAMS.lock().await;
    streams.retain(|s| s != &symbol);
    Ok(())
}
