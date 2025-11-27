use super::counterfactual::{CounterfactualRequest, CounterfactualResult};
use super::fetcher::FetchRequest;
use super::manager::{SharedHistoricalReplayManager, SimulationPayload};
use super::storage::{HistoricalDataPoint, HistoricalDataSet, OrderBookSnapshot};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn historical_fetch_dataset(
    manager: State<'_, SharedHistoricalReplayManager>,
    request: FetchRequest,
) -> Result<HistoricalDataSet, String> {
    let mgr = manager.read().await;
    mgr.fetch_dataset(request).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn historical_fetch_orderbooks(
    manager: State<'_, SharedHistoricalReplayManager>,
    symbol: String,
    start_time: i64,
    end_time: i64,
) -> Result<Vec<OrderBookSnapshot>, String> {
    let mgr = manager.read().await;
    mgr.fetch_orderbooks(&symbol, start_time, end_time)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn historical_run_simulation(
    manager: State<'_, SharedHistoricalReplayManager>,
    payload: SimulationPayload,
) -> Result<super::simulator::SimulationResult, String> {
    let mgr = manager.read().await;
    mgr.run_simulation(payload).await
}

#[tauri::command]
pub async fn historical_compute_counterfactual(
    manager: State<'_, SharedHistoricalReplayManager>,
    request: CounterfactualRequest,
) -> Result<Option<CounterfactualResult>, String> {
    let mgr = manager.read().await;
    mgr.compute_counterfactual(request).await
}

#[tauri::command]
pub async fn historical_get_cache_stats(
    manager: State<'_, SharedHistoricalReplayManager>,
    symbol: String,
) -> Result<HashMap<String, u64>, String> {
    let mgr = manager.read().await;
    mgr.get_cache_stats(&symbol).await
}

#[tauri::command]
pub async fn historical_clear_old_data(
    manager: State<'_, SharedHistoricalReplayManager>,
    days: i64,
) -> Result<u64, String> {
    let mgr = manager.read().await;
    mgr.clear_old_data(days).await
}

#[tauri::command]
pub async fn historical_set_api_key(
    manager: State<'_, SharedHistoricalReplayManager>,
    api_key: Option<String>,
) -> Result<(), String> {
    let mut mgr = manager.write().await;
    mgr.set_api_key(api_key);
    Ok(())
}
