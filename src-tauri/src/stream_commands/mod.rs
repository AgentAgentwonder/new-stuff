use crate::core::WebSocketManager;
use crate::websocket::types::{StreamProvider, StreamStatus};
use tauri::State;

#[tauri::command]
pub async fn subscribe_price_stream(
    manager: State<'_, WebSocketManager>,
    symbols: Vec<String>,
) -> Result<(), String> {
    manager
        .subscribe_prices(symbols)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unsubscribe_price_stream(
    manager: State<'_, WebSocketManager>,
    symbols: Vec<String>,
) -> Result<(), String> {
    manager
        .unsubscribe_prices(symbols)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn subscribe_wallet_stream(
    manager: State<'_, WebSocketManager>,
    addresses: Vec<String>,
) -> Result<(), String> {
    manager
        .subscribe_wallets(addresses)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unsubscribe_wallet_stream(
    manager: State<'_, WebSocketManager>,
    addresses: Vec<String>,
) -> Result<(), String> {
    manager
        .unsubscribe_wallets(addresses)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stream_status(
    manager: State<'_, WebSocketManager>,
) -> Result<Vec<StreamStatus>, String> {
    Ok(manager.get_status().await)
}

#[tauri::command]
pub async fn reconnect_stream(
    manager: State<'_, WebSocketManager>,
    provider_id: String,
) -> Result<(), String> {
    let provider = match provider_id.as_str() {
        "birdeye" => StreamProvider::Birdeye,
        "helius" => StreamProvider::Helius,
        _ => return Err("Invalid provider".to_string()),
    };

    manager.reconnect(provider).await.map_err(|e| e.to_string())
}
