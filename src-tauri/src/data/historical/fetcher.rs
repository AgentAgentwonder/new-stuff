use super::storage::{
    HistoricalDataPoint, HistoricalDataSet, HistoricalStorage, OrderBookSnapshot,
};
use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRequest {
    pub symbol: String,
    pub interval: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchProgress {
    pub symbol: String,
    pub total_points: u64,
    pub fetched_points: u64,
    pub percent_complete: f64,
}

pub struct HistoricalDataFetcher {
    storage: Arc<HistoricalStorage>,
    api_key: Option<String>,
}

impl HistoricalDataFetcher {
    pub fn new(storage: Arc<HistoricalStorage>, api_key: Option<String>) -> Self {
        Self { storage, api_key }
    }

    pub async fn fetch_data(
        &self,
        request: FetchRequest,
    ) -> Result<HistoricalDataSet, Box<dyn std::error::Error>> {
        // Check if data already exists in cache
        let has_data = self
            .storage
            .check_data_coverage(
                &request.symbol,
                &request.interval,
                request.start_time,
                request.end_time,
            )
            .await?;

        if has_data {
            // Return cached data
            let data = self
                .storage
                .get_price_data(
                    &request.symbol,
                    &request.interval,
                    request.start_time,
                    request.end_time,
                )
                .await?;

            return Ok(HistoricalDataSet {
                symbol: request.symbol,
                interval: request.interval,
                data,
                fetched_at: Utc::now(),
            });
        }

        // Fetch from API or generate mock data
        let data = if let Some(ref api_key) = self.api_key {
            match self.fetch_from_birdeye(&request, api_key).await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Birdeye API error: {}, falling back to mock data", e);
                    self.generate_mock_data(&request)
                }
            }
        } else {
            self.generate_mock_data(&request)
        };

        // Store in cache
        self.storage
            .store_price_data(&request.symbol, &request.interval, &data)
            .await?;

        Ok(HistoricalDataSet {
            symbol: request.symbol,
            interval: request.interval,
            data,
            fetched_at: Utc::now(),
        })
    }

    async fn fetch_from_birdeye(
        &self,
        request: &FetchRequest,
        api_key: &str,
    ) -> Result<Vec<HistoricalDataPoint>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Birdeye OHLCV endpoint
        let url = format!(
            "https://public-api.birdeye.so/defi/ohlcv?address={}&type={}&time_from={}&time_to={}",
            request.symbol, request.interval, request.start_time, request.end_time
        );

        let response = client.get(&url).header("X-API-KEY", api_key).send().await?;

        if !response.status().is_success() {
            return Err(format!("API error: {}", response.status()).into());
        }

        #[derive(Deserialize)]
        struct BirdeyeResponse {
            data: BirdeyeOhlcvData,
        }

        #[derive(Deserialize)]
        struct BirdeyeOhlcvData {
            items: Vec<BirdeyeOhlcvItem>,
        }

        #[derive(Deserialize)]
        struct BirdeyeOhlcvItem {
            #[serde(rename = "unixTime")]
            unix_time: i64,
            #[serde(rename = "o")]
            open: f64,
            #[serde(rename = "h")]
            high: f64,
            #[serde(rename = "l")]
            low: f64,
            #[serde(rename = "c")]
            close: f64,
            #[serde(rename = "v")]
            volume: f64,
        }

        let birdeye_data: BirdeyeResponse = response.json().await?;

        let data = birdeye_data
            .data
            .items
            .into_iter()
            .map(|item| HistoricalDataPoint {
                timestamp: item.unix_time,
                open: item.open,
                high: item.high,
                low: item.low,
                close: item.close,
                volume: item.volume,
            })
            .collect();

        Ok(data)
    }

    fn generate_mock_data(&self, request: &FetchRequest) -> Vec<HistoricalDataPoint> {
        use rand::Rng;

        let interval_seconds = match request.interval.as_str() {
            "1m" => 60,
            "5m" => 300,
            "15m" => 900,
            "1h" => 3600,
            "4h" => 14400,
            "1d" => 86400,
            _ => 3600,
        };

        let mut data = Vec::new();
        let mut current_time = request.start_time;
        let mut price = 100.0_f64;

        while current_time <= request.end_time {
            let change = rand::random_range(-0.02..0.02);
            price *= 1.0 + change;
            price = price.max(0.01); // Ensure positive

            let volatility = price * 0.005;
            let high = price + rand::random_range(0.0..volatility);
            let low = price - rand::random_range(0.0..volatility);
            let close = price + rand::random_range(-volatility..volatility);
            let volume = rand::random_range(10000.0..100000.0);

            data.push(HistoricalDataPoint {
                timestamp: current_time,
                open: price,
                high,
                low,
                close,
                volume,
            });

            current_time += interval_seconds;
        }

        data
    }

    pub async fn fetch_orderbook_history(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<OrderBookSnapshot>, Box<dyn std::error::Error>> {
        // Check cache first
        let snapshots = self
            .storage
            .get_orderbook_snapshots(symbol, start_time, end_time)
            .await?;

        if !snapshots.is_empty() {
            return Ok(snapshots);
        }

        // Generate mock orderbook snapshots
        let snapshots = self.generate_mock_orderbooks(symbol, start_time, end_time);

        // Store in cache
        for snapshot in &snapshots {
            self.storage.store_orderbook_snapshot(snapshot).await?;
        }

        Ok(snapshots)
    }

    fn generate_mock_orderbooks(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Vec<OrderBookSnapshot> {
        use rand::Rng;
        let mut snapshots = Vec::new();

        let mut current_time = start_time;
        let interval = 60; // 1 minute snapshots
        let mut mid_price = 100.0;

        while current_time <= end_time {
            mid_price *= 1.0 + rand::random_range(-0.001..0.001);

            let mut bids = Vec::new();
            let mut asks = Vec::new();

            // Generate bid side (below mid price)
            for i in 0..20 {
                let price = mid_price * (1.0 - (i as f64 * 0.001));
                let quantity = rand::random_range(100.0..1000.0);
                bids.push((price, quantity));
            }

            // Generate ask side (above mid price)
            for i in 0..20 {
                let price = mid_price * (1.0 + (i as f64 * 0.001));
                let quantity = rand::random_range(100.0..1000.0);
                asks.push((price, quantity));
            }

            snapshots.push(OrderBookSnapshot {
                timestamp: current_time,
                symbol: symbol.to_string(),
                bids,
                asks,
            });

            current_time += interval;
        }

        snapshots
    }

    pub async fn fetch_in_chunks(
        &self,
        request: FetchRequest,
        chunk_size_hours: i64,
        progress_callback: impl Fn(FetchProgress),
    ) -> Result<HistoricalDataSet, Box<dyn std::error::Error>> {
        let total_duration = request.end_time - request.start_time;
        let chunk_duration = chunk_size_hours * 3600;
        let num_chunks = (total_duration as f64 / chunk_duration as f64).ceil() as u64;

        let mut all_data = Vec::new();
        let mut current_start = request.start_time;

        for chunk_idx in 0..num_chunks {
            let current_end = (current_start + chunk_duration).min(request.end_time);

            let chunk_request = FetchRequest {
                symbol: request.symbol.clone(),
                interval: request.interval.clone(),
                start_time: current_start,
                end_time: current_end,
            };

            let chunk_data = self.fetch_data(chunk_request).await?;
            all_data.extend(chunk_data.data);

            progress_callback(FetchProgress {
                symbol: request.symbol.clone(),
                total_points: num_chunks,
                fetched_points: chunk_idx + 1,
                percent_complete: ((chunk_idx + 1) as f64 / num_chunks as f64) * 100.0,
            });

            current_start = current_end;
        }

        Ok(HistoricalDataSet {
            symbol: request.symbol,
            interval: request.interval,
            data: all_data,
            fetched_at: Utc::now(),
        })
    }
}
