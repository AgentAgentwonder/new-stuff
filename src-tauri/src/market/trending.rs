use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingCoin {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub volume_change_24h: f64,
    pub market_cap: f64,
    pub social_mentions: u32,
    pub social_change_24h: f64,
    pub rank: u32,
}

#[derive(Debug)]
struct CacheEntry {
    data: Vec<TrendingCoin>,
    timestamp: SystemTime,
}

pub struct TrendingCoinsCache {
    cache: RwLock<Option<CacheEntry>>,
    ttl: Duration,
}

impl TrendingCoinsCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(None),
            ttl: Duration::from_secs(60),
        }
    }

    pub async fn get_trending_coins(
        &self,
        api_key: Option<String>,
    ) -> Result<Vec<TrendingCoin>, String> {
        let cache = self.cache.read().await;
        if let Some(entry) = &*cache {
            if entry.timestamp.elapsed().unwrap_or(Duration::MAX) < self.ttl {
                return Ok(entry.data.clone());
            }
        }
        drop(cache);

        let coins = if let Some(key) = api_key {
            if !key.is_empty() {
                match self.fetch_from_birdeye(&key).await {
                    Ok(coins) => coins,
                    Err(_) => self.generate_mock_trending(),
                }
            } else {
                self.generate_mock_trending()
            }
        } else {
            self.generate_mock_trending()
        };

        let mut cache = self.cache.write().await;
        *cache = Some(CacheEntry {
            data: coins.clone(),
            timestamp: SystemTime::now(),
        });

        Ok(coins)
    }

    async fn fetch_from_birdeye(&self, api_key: &str) -> Result<Vec<TrendingCoin>, String> {
        let client = reqwest::Client::new();
        let url = "https://public-api.birdeye.so/defi/trending";

        let response = client
            .get(url)
            .header("X-API-KEY", api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        #[derive(Deserialize)]
        struct BirdeyeResponse {
            data: Vec<BirdeyeTrending>,
        }

        #[derive(Deserialize)]
        struct BirdeyeTrending {
            address: String,
            symbol: String,
            name: String,
            price: f64,
            #[serde(rename = "priceChange24h")]
            price_change_24h: f64,
            #[serde(rename = "volume24h")]
            volume_24h: f64,
            #[serde(rename = "volumeChange24h")]
            volume_change_24h: Option<f64>,
            #[serde(rename = "marketCap")]
            market_cap: f64,
        }

        let data: BirdeyeResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse failed: {}", e))?;

        let coins = data
            .data
            .into_iter()
            .enumerate()
            .map(|(idx, item)| TrendingCoin {
                address: item.address,
                symbol: item.symbol,
                name: item.name,
                price: item.price,
                price_change_24h: item.price_change_24h,
                volume_24h: item.volume_24h,
                volume_change_24h: item.volume_change_24h.unwrap_or(0.0),
                market_cap: item.market_cap,
                social_mentions: 0,
                social_change_24h: 0.0,
                rank: (idx + 1) as u32,
            })
            .collect();

        Ok(coins)
    }

    fn generate_mock_trending(&self) -> Vec<TrendingCoin> {
        let symbols = vec![
            (
                "BONK",
                "Bonk",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            ),
            (
                "JUP",
                "Jupiter",
                "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
            ),
            (
                "WIF",
                "dogwifhat",
                "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
            ),
            (
                "PYTH",
                "Pyth Network",
                "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3",
            ),
            ("JTO", "Jito", "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL"),
            (
                "ORCA",
                "Orca",
                "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE",
            ),
            (
                "RAY",
                "Raydium",
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
            ),
            (
                "SAMO",
                "Samoyedcoin",
                "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU",
            ),
        ];

        symbols
            .iter()
            .enumerate()
            .map(|(idx, (symbol, name, address))| {
                let base_price = match *symbol {
                    "JUP" => 1.23,
                    "WIF" => 2.45,
                    "PYTH" => 0.87,
                    "JTO" => 3.21,
                    "ORCA" => 4.56,
                    "RAY" => 2.89,
                    "SAMO" => 0.015,
                    _ => 0.000023,
                };

                TrendingCoin {
                    address: address.to_string(),
                    symbol: symbol.to_string(),
                    name: name.to_string(),
                    price: base_price * (1.0 + rand::random_range(-0.05..0.05)),
                    price_change_24h: rand::random_range(-15.0..25.0),
                    volume_24h: rand::random_range(500000.0..20000000.0),
                    volume_change_24h: rand::random_range(-30.0..50.0),
                    market_cap: rand::random_range(5000000.0..500000000.0),
                    social_mentions: rand::random_range(100..5000),
                    social_change_24h: rand::random_range(-20.0..80.0),
                    rank: (idx + 1) as u32,
                }
            })
            .collect()
    }

    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }
}
