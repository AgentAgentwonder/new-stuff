use reqwest;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

const CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrendingCoin {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub volume_change_24h: f64,
    pub market_cap: f64,
    pub market_cap_change_24h: f64,
    pub liquidity: f64,
    pub trend_score: f64,
    pub logo_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoinSentiment {
    pub symbol: String,
    pub score: f64,
    pub label: String,
    pub mentions: u32,
    pub positive_ratio: f64,
}

#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    timestamp: SystemTime,
}

impl<T> CacheEntry<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            timestamp: SystemTime::now(),
        }
    }

    fn is_expired(&self) -> bool {
        SystemTime::now()
            .duration_since(self.timestamp)
            .unwrap_or(Duration::from_secs(0))
            >= CACHE_TTL
    }
}

pub struct TrendingCoinsCache {
    cache: Arc<RwLock<Option<CacheEntry<Vec<TrendingCoin>>>>>,
}

impl TrendingCoinsCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get(&self) -> Option<Vec<TrendingCoin>> {
        let cache = self.cache.read().ok()?;
        cache.as_ref().and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set(&self, data: Vec<TrendingCoin>) {
        if let Ok(mut cache) = self.cache.write() {
            *cache = Some(CacheEntry::new(data));
        }
    }

    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            *cache = None;
        }
    }
}

impl Default for TrendingCoinsCache {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    static ref TRENDING_CACHE: TrendingCoinsCache = TrendingCoinsCache::new();
}

async fn fetch_birdeye_trending(api_key: &str, limit: usize) -> Result<Vec<TrendingCoin>, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://public-api.birdeye.so/defi/token_trending?sort_by=rank&sort_type=asc&offset=0&limit={}",
        limit
    );

    let response = client
        .get(&url)
        .header("X-API-KEY", api_key)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    #[derive(Deserialize)]
    struct BirdeyeResponse {
        data: BirdeyeData,
    }

    #[derive(Deserialize)]
    struct BirdeyeData {
        items: Vec<BirdeyeTrendingItem>,
    }

    #[derive(Deserialize)]
    struct BirdeyeTrendingItem {
        address: String,
        symbol: String,
        name: String,
        #[serde(rename = "price")]
        value: f64,
        #[serde(rename = "priceChange24h")]
        price_change_24h: Option<f64>,
        #[serde(rename = "volume24h")]
        volume_24h: Option<f64>,
        #[serde(rename = "volume24hChange")]
        volume_24h_change: Option<f64>,
        #[serde(rename = "mc")]
        market_cap: Option<f64>,
        #[serde(rename = "mcChange24h")]
        market_cap_change_24h: Option<f64>,
        liquidity: Option<f64>,
        #[serde(rename = "logoURI")]
        logo_uri: Option<String>,
    }

    let data: BirdeyeResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;

    let coins: Vec<TrendingCoin> = data
        .data
        .items
        .into_iter()
        .enumerate()
        .map(|(idx, item)| {
            let price_change = item.price_change_24h.unwrap_or(0.0);
            let volume_change = item.volume_24h_change.unwrap_or(0.0);
            let mc_change = item.market_cap_change_24h.unwrap_or(0.0);

            let trend_score = calculate_trend_score(
                price_change,
                volume_change,
                mc_change,
                item.volume_24h.unwrap_or(0.0),
            );

            TrendingCoin {
                address: item.address,
                symbol: item.symbol,
                name: item.name,
                price: item.value,
                price_change_24h: price_change,
                volume_24h: item.volume_24h.unwrap_or(0.0),
                volume_change_24h: volume_change,
                market_cap: item.market_cap.unwrap_or(0.0),
                market_cap_change_24h: mc_change,
                liquidity: item.liquidity.unwrap_or(0.0),
                trend_score,
                logo_uri: item.logo_uri,
            }
        })
        .collect();

    Ok(coins)
}

fn calculate_trend_score(
    price_change: f64,
    volume_change: f64,
    mc_change: f64,
    volume: f64,
) -> f64 {
    let price_score =
        (price_change.abs() / 100.0).min(1.0) * if price_change > 0.0 { 1.0 } else { 0.5 };
    let volume_score = (volume_change / 100.0).min(1.0) * 0.8;
    let mc_score = (mc_change / 100.0).min(1.0) * 0.5;
    let liquidity_score = (volume / 1_000_000.0).min(1.0) * 0.3;

    (price_score + volume_score + mc_score + liquidity_score).clamp(0.0, 10.0) * 10.0
}

fn generate_mock_trending(limit: usize) -> Vec<TrendingCoin> {
    use rand::Rng;

    let mock_tokens = vec![
        (
            "BONK",
            "Bonk",
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            0.000023,
        ),
        (
            "JUP",
            "Jupiter",
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
            1.23,
        ),
        (
            "WIF",
            "dogwifhat",
            "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
            2.45,
        ),
        (
            "PYTH",
            "Pyth Network",
            "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3",
            0.87,
        ),
        (
            "JTO",
            "Jito",
            "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
            3.21,
        ),
        (
            "ORCA",
            "Orca",
            "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE",
            4.56,
        ),
        (
            "RAY",
            "Raydium",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            3.78,
        ),
        (
            "MNGO",
            "Mango",
            "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac",
            0.045,
        ),
        (
            "COPE",
            "Cope",
            "8HGyAAB1yoM1ttS7pXjHMa3dukTFGQggnFFH3hJZgzQh",
            0.12,
        ),
        (
            "STEP",
            "Step Finance",
            "StepAscQoEioFxxWGnh2sLBDFp9d8rvKz2Yp39iDpyT",
            0.089,
        ),
    ];

    mock_tokens
        .into_iter()
        .take(limit)
        .enumerate()
        .map(|(idx, (symbol, name, address, base_price))| {
            let price = base_price * (1.0 + rand::random_range(-0.1..0.1));
            let price_change = rand::random_range(-25.0..35.0);
            let volume_change = rand::random_range(-30.0..50.0);
            let mc_change = rand::random_range(-15.0..25.0);
            let volume = rand::random_range(100_000.0..10_000_000.0);

            TrendingCoin {
                address: address.to_string(),
                symbol: symbol.to_string(),
                name: name.to_string(),
                price,
                price_change_24h: price_change,
                volume_24h: volume,
                volume_change_24h: volume_change,
                market_cap: rand::random_range(1_000_000.0..100_000_000.0),
                market_cap_change_24h: mc_change,
                liquidity: rand::random_range(50_000.0..5_000_000.0),
                trend_score: calculate_trend_score(price_change, volume_change, mc_change, volume),
                logo_uri: None,
            }
        })
        .collect()
}

fn generate_mock_sentiment(symbol: &str) -> CoinSentiment {
    use rand::Rng;

    let score = rand::random_range(-1.0..1.0);
    let label = if score > 0.3 {
        "Positive"
    } else if score < -0.3 {
        "Negative"
    } else {
        "Neutral"
    };

    CoinSentiment {
        symbol: symbol.to_string(),
        score,
        label: label.to_string(),
        mentions: rand::random_range(10..1000),
        positive_ratio: ((score + 1.0) / 2.0).clamp(0.0, 1.0),
    }
}

#[tauri::command]
pub async fn get_trending_coins(
    limit: usize,
    api_key: Option<String>,
) -> Result<Vec<TrendingCoin>, String> {
    if let Some(cached) = TRENDING_CACHE.get() {
        return Ok(cached.into_iter().take(limit).collect());
    }

    let coins = if let Some(key) = api_key {
        if !key.is_empty() {
            match fetch_birdeye_trending(&key, limit).await {
                Ok(coins) => {
                    TRENDING_CACHE.set(coins.clone());
                    coins
                }
                Err(_) => generate_mock_trending(limit),
            }
        } else {
            generate_mock_trending(limit)
        }
    } else {
        generate_mock_trending(limit)
    };

    Ok(coins)
}

#[tauri::command]
pub async fn get_coin_sentiment(
    symbol: String,
    _api_key: Option<String>,
) -> Result<CoinSentiment, String> {
    Ok(generate_mock_sentiment(&symbol))
}

#[tauri::command]
pub async fn refresh_trending() -> Result<(), String> {
    TRENDING_CACHE.clear();
    Ok(())
}
