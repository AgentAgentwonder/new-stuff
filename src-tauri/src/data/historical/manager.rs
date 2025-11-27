use super::counterfactual::{
    compute_hold_counterfactual, CounterfactualRequest, CounterfactualResult,
};
use super::fetcher::{FetchProgress, FetchRequest, HistoricalDataFetcher};
use super::simulator::{run_simulation, PortfolioHolding, SimulationConfig, SimulationResult};
use super::storage::{
    HistoricalDataPoint, HistoricalDataSet, HistoricalStorage, OrderBookSnapshot,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

pub type SharedHistoricalReplayManager = Arc<RwLock<HistoricalReplayManager>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationPayload {
    pub config: SimulationConfig,
    pub datasets: HashMap<String, Vec<HistoricalDataPoint>>,
}

pub struct HistoricalReplayManager {
    storage: Arc<HistoricalStorage>,
    api_key: Option<String>,
}

impl HistoricalReplayManager {
    pub async fn new(
        app_handle: &tauri::AppHandle,
        api_key: Option<String>,
    ) -> Result<Self, String> {
        let mut db_path = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Unable to resolve app data directory: {}", e))?;

        std::fs::create_dir_all(&db_path)
            .map_err(|e| format!("Failed to create data directory: {e}"))?;

        db_path.push("historical_replay.db");

        let storage = HistoricalStorage::new(db_path)
            .await
            .map_err(|e| format!("Failed to initialize historical storage: {e}"))?;

        Ok(Self {
            storage: Arc::new(storage),
            api_key,
        })
    }

    fn fetcher(&self) -> HistoricalDataFetcher {
        HistoricalDataFetcher::new(self.storage.clone(), self.api_key.clone())
    }

    pub fn set_api_key(&mut self, api_key: Option<String>) {
        self.api_key = api_key;
    }

    pub async fn fetch_dataset(
        &self,
        request: FetchRequest,
    ) -> Result<HistoricalDataSet, Box<dyn std::error::Error>> {
        self.fetcher().fetch_data(request).await
    }

    pub async fn fetch_dataset_chunked<F>(
        &self,
        request: FetchRequest,
        chunk_size_hours: i64,
        progress_callback: F,
    ) -> Result<HistoricalDataSet, Box<dyn std::error::Error>>
    where
        F: Fn(FetchProgress) + Send,
    {
        self.fetcher()
            .fetch_in_chunks(request, chunk_size_hours, progress_callback)
            .await
    }

    pub async fn fetch_orderbooks(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<OrderBookSnapshot>, Box<dyn std::error::Error>> {
        self.fetcher()
            .fetch_orderbook_history(symbol, start_time, end_time)
            .await
    }

    pub async fn run_simulation(
        &self,
        payload: SimulationPayload,
    ) -> Result<SimulationResult, String> {
        let mut datasets_map = HashMap::new();
        for (symbol, data_points) in payload.datasets.iter() {
            datasets_map.insert(symbol.clone(), data_points.clone());
        }

        run_simulation(payload.config, &datasets_map)
    }

    pub async fn compute_counterfactual(
        &self,
        request: CounterfactualRequest,
    ) -> Result<Option<CounterfactualResult>, String> {
        let data = self
            .storage
            .get_price_data(&request.symbol, "1h", request.start_time, request.end_time)
            .await
            .map_err(|e| e.to_string())?;

        Ok(compute_hold_counterfactual(request, &data))
    }

    pub async fn get_cache_stats(&self, symbol: &str) -> Result<HashMap<String, u64>, String> {
        self.storage
            .get_cache_stats(symbol)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn clear_old_data(&self, days: i64) -> Result<u64, String> {
        self.storage
            .clear_old_data(days)
            .await
            .map_err(|e| e.to_string())
    }
}
