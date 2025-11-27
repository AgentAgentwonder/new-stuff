use reqwest;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const POLYMARKET_API_BASE: &str = "https://clob.polymarket.com";
const CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketMarket {
    pub condition_id: String,
    pub question: String,
    pub description: Option<String>,
    pub end_date: Option<String>,
    pub game_start_time: Option<String>,
    pub question_id: String,
    pub market_slug: String,
    pub outcomes: Vec<String>,
    pub outcome_prices: Vec<f64>,
    pub volume: f64,
    pub liquidity: f64,
    pub active: bool,
    pub closed: bool,
    pub accepting_orders: bool,
    pub neg_risk: bool,
    pub tags: Vec<String>,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketOrderBook {
    pub market: String,
    pub asset_id: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookEntry {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketTrade {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub price: String,
    pub size: String,
    pub side: String,
    pub timestamp: i64,
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
pub struct PolymarketAdapter {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    base_url: String,
}

impl Default for PolymarketAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PolymarketAdapter {
    pub fn new() -> Self {
        Self::with_base_url(POLYMARKET_API_BASE)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache: Arc::new(RwLock::new(HashMap::new())),
            base_url: base_url.into(),
        }
    }

    fn cache_key(prefix: &str, identifier: &str) -> String {
        format!("polymarket:{}:{}", prefix, identifier)
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

    async fn fetch_markets_raw(&self) -> Result<Vec<PolymarketMarket>, String> {
        let url = format!("{}/markets", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Polymarket API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Polymarket API returned status: {}",
                response.status()
            ));
        }

        response
            .json::<Vec<PolymarketMarket>>()
            .await
            .map_err(|e| format!("Failed to parse Polymarket response: {e}"))
    }

    pub async fn fetch_markets(&self) -> Result<Vec<PolymarketMarket>, String> {
        let cache_key = Self::cache_key("markets", "all");

        if let Some(markets) = self.cache_get::<Vec<PolymarketMarket>>(&cache_key).await {
            return Ok(markets);
        }

        let markets = self.fetch_markets_raw().await?;
        self.cache_set(&cache_key, &markets).await;
        Ok(markets)
    }

    async fn fetch_market_raw(&self, condition_id: &str) -> Result<PolymarketMarket, String> {
        let url = format!("{}/markets/{}", self.base_url, condition_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Polymarket API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Polymarket API returned status: {}",
                response.status()
            ));
        }

        response
            .json::<PolymarketMarket>()
            .await
            .map_err(|e| format!("Failed to parse Polymarket market response: {e}"))
    }

    pub async fn fetch_market(&self, condition_id: &str) -> Result<PolymarketMarket, String> {
        let cache_key = Self::cache_key("market", condition_id);

        if let Some(market) = self.cache_get::<PolymarketMarket>(&cache_key).await {
            return Ok(market);
        }

        let market = self.fetch_market_raw(condition_id).await?;
        self.cache_set(&cache_key, &market).await;
        Ok(market)
    }

    pub async fn fetch_order_book(&self, token_id: &str) -> Result<PolymarketOrderBook, String> {
        let url = format!("{}/book", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[("token_id", token_id)])
            .send()
            .await
            .map_err(|e| format!("Polymarket order book request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Polymarket API returned status: {}",
                response.status()
            ));
        }

        response
            .json::<PolymarketOrderBook>()
            .await
            .map_err(|e| format!("Failed to parse Polymarket order book: {e}"))
    }

    pub async fn fetch_trades(&self, market: &str) -> Result<Vec<PolymarketTrade>, String> {
        let url = format!("{}/trades", self.base_url);

        let response = self
            .client
            .get(&url)
            .query(&[("market", market)])
            .send()
            .await
            .map_err(|e| format!("Polymarket trades request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!(
                "Polymarket API returned status: {}",
                response.status()
            ));
        }

        response
            .json::<Vec<PolymarketTrade>>()
            .await
            .map_err(|e| format!("Failed to parse Polymarket trades: {e}"))
    }

    pub async fn search_markets(&self, query: &str) -> Result<Vec<PolymarketMarket>, String> {
        let markets = self.fetch_markets().await?;

        let query_lower = query.to_lowercase();
        let filtered: Vec<PolymarketMarket> = markets
            .into_iter()
            .filter(|market| {
                market.question.to_lowercase().contains(&query_lower)
                    || market
                        .description
                        .as_ref()
                        .map_or(false, |d| d.to_lowercase().contains(&query_lower))
                    || market
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(filtered)
    }
}

// Mock data generator for development/testing
pub fn generate_mock_polymarket_markets() -> Vec<PolymarketMarket> {
    vec![
        PolymarketMarket {
            condition_id: "0x1234567890abcdef".to_string(),
            question: "Will Bitcoin reach $100,000 by end of 2024?".to_string(),
            description: Some("Binary outcome market for Bitcoin price milestone".to_string()),
            end_date: Some("2024-12-31T23:59:59Z".to_string()),
            game_start_time: None,
            question_id: "btc-100k-2024".to_string(),
            market_slug: "bitcoin-100k-2024".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec![rand::random_range(0.3..0.7), rand::random_range(0.3..0.7)],
            volume: rand::random_range(100000.0..1000000.0),
            liquidity: rand::random_range(50000.0..500000.0),
            active: true,
            closed: false,
            accepting_orders: true,
            neg_risk: false,
            tags: vec!["crypto".to_string(), "bitcoin".to_string()],
            image: None,
        },
        PolymarketMarket {
            condition_id: "0xabcdef1234567890".to_string(),
            question: "Will Ethereum merge successfully complete in Q1 2024?".to_string(),
            description: Some("Prediction on Ethereum protocol upgrade".to_string()),
            end_date: Some("2024-03-31T23:59:59Z".to_string()),
            game_start_time: None,
            question_id: "eth-merge-q1-2024".to_string(),
            market_slug: "ethereum-merge-2024".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec![rand::random_range(0.4..0.8), rand::random_range(0.2..0.6)],
            volume: rand::random_range(50000.0..500000.0),
            liquidity: rand::random_range(25000.0..250000.0),
            active: true,
            closed: false,
            accepting_orders: true,
            neg_risk: false,
            tags: vec!["crypto".to_string(), "ethereum".to_string()],
            image: None,
        },
        PolymarketMarket {
            condition_id: "0xfedcba0987654321".to_string(),
            question: "Will Solana TVL exceed $10 billion in 2024?".to_string(),
            description: Some("Market for Solana ecosystem growth metrics".to_string()),
            end_date: Some("2024-12-31T23:59:59Z".to_string()),
            game_start_time: None,
            question_id: "sol-tvl-10b-2024".to_string(),
            market_slug: "solana-tvl-2024".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            outcome_prices: vec![rand::random_range(0.5..0.9), rand::random_range(0.1..0.5)],
            volume: rand::random_range(75000.0..750000.0),
            liquidity: rand::random_range(35000.0..350000.0),
            active: true,
            closed: false,
            accepting_orders: true,
            neg_risk: false,
            tags: vec![
                "crypto".to_string(),
                "solana".to_string(),
                "defi".to_string(),
            ],
            image: None,
        },
    ]
}
