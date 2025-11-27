use crate::data::database::{CompressionConfig, CompressionStats, SharedCompressionManager};
use tauri::{Manager, State};

#[tauri::command]
pub async fn get_compression_stats(
    compression_manager: State<'_, SharedCompressionManager>,
) -> Result<CompressionStats, String> {
    let manager = compression_manager.read().await;
    manager
        .get_compression_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compress_old_data(
    compression_manager: State<'_, SharedCompressionManager>,
) -> Result<i64, String> {
    let manager = compression_manager.read().await;

    let events_compressed = manager
        .compress_old_events()
        .await
        .map_err(|e| e.to_string())?;

    let trades_compressed = manager
        .compress_old_trades()
        .await
        .map_err(|e| e.to_string())?;

    Ok(events_compressed + trades_compressed)
}

#[tauri::command]
pub async fn update_compression_config(
    compression_manager: State<'_, SharedCompressionManager>,
    config: CompressionConfig,
) -> Result<(), String> {
    let manager = compression_manager.read().await;
    manager
        .update_config(config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_compression_config(
    compression_manager: State<'_, SharedCompressionManager>,
) -> Result<CompressionConfig, String> {
    let manager = compression_manager.read().await;
    Ok(manager.get_config().await)
}

#[tauri::command]
pub async fn decompress_data(
    compression_manager: State<'_, SharedCompressionManager>,
    id: String,
) -> Result<Vec<u8>, String> {
    let manager = compression_manager.read().await;
    manager
        .decompress_data(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_database_size(app_handle: tauri::AppHandle) -> Result<DatabaseSize, String> {
    use std::fs;

    let mut data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Unable to resolve app data directory: {}", e))?;

    let mut total_size = 0u64;
    let mut compressed_db_size = 0u64;

    // Get events.db size
    data_dir.push("events.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    // Get multisig.db size
    data_dir.push("multisig.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    // Get other db files
    data_dir.push("orders.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("paper_trading.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("new_coins.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("wallet_monitor.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("dca.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("copy_trading.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("watchlists.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("alerts.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    data_dir.push("activity.db");
    if let Ok(metadata) = fs::metadata(&data_dir) {
        total_size += metadata.len();
    }
    data_dir.pop();

    // Estimate compressed data size by checking events.db internal compressed_data table
    // This is just an approximation as we can't easily separate it from the main db file
    compressed_db_size = (total_size as f64 * 0.1) as u64; // Rough estimate

    Ok(DatabaseSize {
        total_bytes: total_size,
        total_mb: (total_size as f64) / 1024.0 / 1024.0,
        compressed_data_bytes: compressed_db_size,
        uncompressed_data_bytes: total_size - compressed_db_size,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct DatabaseSize {
    pub total_bytes: u64,
    pub total_mb: f64,
    pub compressed_data_bytes: u64,
    pub uncompressed_data_bytes: u64,
}
