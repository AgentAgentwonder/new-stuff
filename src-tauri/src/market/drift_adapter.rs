use reqwest;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const DRIFT_API_BASE: &str = "https://api.drift.trade";
const CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriftMarket {
    pub market_index: u32,
    pub market_type: String,
    pub symbol: String,
    pub base_asset_symbol: String,
    pub quote_asset_symbol: String,
    pub oracle_price: f64,
    pub mark_price: f64,
    pub funding_rate: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub number_of_users: u32,
    pub number_of_orders: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriftPrediction {
    pub id: String,
    pub market_index: u32,
    pub title: String,
    pub description: String,
    pub outcomes: Vec<String>,
    pub outcome_prices: Vec<f64>,
    pub total_volume: f64,
    pub resolution_time: Option<i64>,
    pub resolved: bool,
    pub winning_outcome: Option<usize>,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriftOrderBook {
    pub market_index: u32,
    pub slot: u64,
    pub bids: Vec<DriftOrderBookLevel>,
    pub asks: Vec<DriftOrderBookLevel>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriftOrderBookLevel {
    pub price: f64,
    pub size: f64,
    pub sources: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DriftPosition {
    pub user: String,
    pub market_index: u32,
    pub base_asset_amount: f64,
    pub quote_asset_amount: f64,
    pub entry_price: f64,
    pub unrealized_pnl: f64,
    pub last_cumulative_funding_rate: f64,
}

#[derive(Clone)]
struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

impl CacheEntry {
    fn new(value: Value, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

#[derive(Clone)]
pub struct DriftAdapter {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl Default for DriftAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DriftAdapter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn cache_key(prefix: &str, identifier: &str) -> String {
        format!("drift:{}:{}", prefix, identifier)
    }

    async fn cache_get<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.get(key) {
            if entry.is_valid() {
                return serde_json::from_value(entry.value.clone()).ok();
            } else {
                cache.remove(key);
            }
        }
        None
    }

    async fn cache_set<T>(&self, key: &str, value: &T)
    where
        T: Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), CacheEntry::new(json_value, CACHE_TTL));
        }
    }

    async fn fetch_markets_raw(&self) -> Result<Vec<DriftMarket>, String> {
        let url = format!("{}/v2/markets", DRIFT_API_BASE);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Drift API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Drift API returned status: {}", response.status()));
        }

        response
            .json::<Vec<DriftMarket>>()
            .await
            .map_err(|e| format!("Failed to parse Drift markets response: {e}"))
    }

    pub async fn fetch_markets(&self) -> Result<Vec<DriftMarket>, String> {
        let cache_key = Self::cache_key("markets", "all");

        if let Some(markets) = self.cache_get::<Vec<DriftMarket>>(&cache_key).await {
            return Ok(markets);
        }

        let markets = self.fetch_markets_raw().await?;
        self.cache_set(&cache_key, &markets).await;
        Ok(markets)
    }

    async fn fetch_market_raw(&self, market_index: u32) -> Result<DriftMarket, String> {
        let url = format!("{}/v2/markets/{}", DRIFT_API_BASE, market_index);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Drift API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Drift API returned status: {}", response.status()));
        }

        response
            .json::<DriftMarket>()
            .await
            .map_err(|e| format!("Failed to parse Drift market response: {e}"))
    }

    pub async fn fetch_market(&self, market_index: u32) -> Result<DriftMarket, String> {
        let cache_key = Self::cache_key("market", &market_index.to_string());

        if let Some(market) = self.cache_get::<DriftMarket>(&cache_key).await {
            return Ok(market);
        }

        let market = self.fetch_market_raw(market_index).await?;
        self.cache_set(&cache_key, &market).await;
        Ok(market)
    }

    pub async fn fetch_order_book(&self, market_index: u32) -> Result<DriftOrderBook, String> {
        let url = format!("{}/v2/orderbook/{}", DRIFT_API_BASE, market_index);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Drift order book request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Drift API returned status: {}", response.status()));
        }

        response
            .json::<DriftOrderBook>()
            .await
            .map_err(|e| format!("Failed to parse Drift order book: {e}"))
    }

    pub async fn fetch_predictions(&self) -> Result<Vec<DriftPrediction>, String> {
        let cache_key = Self::cache_key("predictions", "all");

        if let Some(predictions) = self.cache_get::<Vec<DriftPrediction>>(&cache_key).await {
            return Ok(predictions);
        }

        let url = format!("{}/v2/predictions", DRIFT_API_BASE);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Drift predictions request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("Drift API returned status: {}", response.status()));
        }

        let predictions = response
            .json::<Vec<DriftPrediction>>()
            .await
            .map_err(|e| format!("Failed to parse Drift predictions: {e}"))?;

        self.cache_set(&cache_key, &predictions).await;
        Ok(predictions)
    }

    pub async fn search_predictions(&self, query: &str) -> Result<Vec<DriftPrediction>, String> {
        let predictions = self.fetch_predictions().await?;

        let query_lower = query.to_lowercase();
        let filtered: Vec<DriftPrediction> = predictions
            .into_iter()
            .filter(|pred| {
                pred.title.to_lowercase().contains(&query_lower)
                    || pred.description.to_lowercase().contains(&query_lower)
                    || pred.category.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(filtered)
    }
}

// Mock data generator for development/testing
pub fn generate_mock_drift_predictions() -> Vec<DriftPrediction> {
    use rand::Rng;

    vec![
        DriftPrediction {
            id: "drift-pred-1".to_string(),
            market_index: 1,
            title: "SOL Price Above $150 by EOY 2024".to_string(),
            description: "Will Solana's price exceed $150 by the end of 2024?".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec![rand::random_range(0.4..0.8), rand::random_range(0.2..0.6)],
            total_volume: rand::random_range(500000.0..5000000.0),
            resolution_time: Some(1735689600), // Dec 31, 2024
            resolved: false,
            winning_outcome: None,
            category: "Crypto Price".to_string(),
        },
        DriftPrediction {
            id: "drift-pred-2".to_string(),
            market_index: 2,
            title: "Drift Protocol TVL Exceeds $1B".to_string(),
            description: "Will Drift's total value locked surpass $1 billion in 2024?".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec![rand::random_range(0.5..0.9), rand::random_range(0.1..0.5)],
            total_volume: rand::random_range(250000.0..2500000.0),
            resolution_time: Some(1735689600),
            resolved: false,
            winning_outcome: None,
            category: "DeFi".to_string(),
        },
        DriftPrediction {
            id: "drift-pred-3".to_string(),
            market_index: 3,
            title: "Solana Mainnet Downtime < 1 Hour in Q1 2024".to_string(),
            description: "Will Solana experience less than 1 hour of downtime in Q1?".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec![rand::random_range(0.3..0.7), rand::random_range(0.3..0.7)],
            total_volume: rand::random_range(100000.0..1000000.0),
            resolution_time: Some(1711929600), // Mar 31, 2024
            resolved: false,
            winning_outcome: None,
            category: "Infrastructure".to_string(),
        },
    ]
}

pub fn generate_mock_drift_markets() -> Vec<DriftMarket> {
    use rand::Rng;

    vec![
        DriftMarket {
            market_index: 0,
            market_type: "perp".to_string(),
            symbol: "SOL-PERP".to_string(),
            base_asset_symbol: "SOL".to_string(),
            quote_asset_symbol: "USDC".to_string(),
            oracle_price: rand::random_range(95.0..105.0),
            mark_price: rand::random_range(95.0..105.0),
            funding_rate: rand::random_range(-0.001..0.001),
            open_interest: rand::random_range(10000000.0..50000000.0),
            volume_24h: rand::random_range(50000000.0..500000000.0),
            number_of_users: rand::random_range(1000..10000),
            number_of_orders: rand::random_range(5000..50000),
        },
        DriftMarket {
            market_index: 1,
            market_type: "perp".to_string(),
            symbol: "BTC-PERP".to_string(),
            base_asset_symbol: "BTC".to_string(),
            quote_asset_symbol: "USDC".to_string(),
            oracle_price: rand::random_range(60000.0..70000.0),
            mark_price: rand::random_range(60000.0..70000.0),
            funding_rate: rand::random_range(-0.001..0.001),
            open_interest: rand::random_range(50000000.0..150000000.0),
            volume_24h: rand::random_range(200000000.0..2000000000.0),
            number_of_users: rand::random_range(5000..50000),
            number_of_orders: rand::random_range(10000..100000),
        },
        DriftMarket {
            market_index: 2,
            market_type: "perp".to_string(),
            symbol: "ETH-PERP".to_string(),
            base_asset_symbol: "ETH".to_string(),
            quote_asset_symbol: "USDC".to_string(),
            oracle_price: rand::random_range(3000.0..3500.0),
            mark_price: rand::random_range(3000.0..3500.0),
            funding_rate: rand::random_range(-0.001..0.001),
            open_interest: rand::random_range(30000000.0..100000000.0),
            volume_24h: rand::random_range(150000000.0..1500000000.0),
            number_of_users: rand::random_range(3000..30000),
            number_of_orders: rand::random_range(8000..80000),
        },
    ]
}
