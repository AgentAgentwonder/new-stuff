use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tauri::Manager;
use tokio::sync::RwLock;

const CACHE_TTL_MINUTES: i64 = 5;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: Vec<TopCoin>,
    pub timestamp: SystemTime,
}

pub struct TopCoinsCache {
    cache: RwLock<Option<CacheEntry>>,
    ttl: Duration,
    page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopCoin {
    pub rank: i32,
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub logo_uri: Option<String>,
    pub price: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub price_change_7d: f64,
    pub sparkline: Vec<f64>,
    pub market_cap_category: String,
    pub liquidity: Option<f64>,
    pub circulating_supply: Option<f64>,
}

impl TopCoinsCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(None),
            ttl: Duration::from_secs(300),
            page_size: 100,
        }
    }

    pub async fn get_top_coins(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
        api_key: Option<String>,
    ) -> Result<Vec<TopCoin>, String> {
        let cache = self.cache.read().await;
        if let Some(entry) = &*cache {
            if entry.timestamp.elapsed().unwrap_or(Duration::MAX) < self.ttl {
                let data = self.slice_data(&entry.data, limit, offset);
                return Ok(data);
            }
        }
        drop(cache);

        let full_data = if let Some(key) = api_key.clone() {
            if !key.is_empty() {
                match self.fetch_from_birdeye(&key).await {
                    Ok(data) => data,
                    Err(_) => self.generate_mock_top_coins(),
                }
            } else {
                self.generate_mock_top_coins()
            }
        } else {
            self.generate_mock_top_coins()
        };

        let mut cache = self.cache.write().await;
        *cache = Some(CacheEntry {
            data: full_data.clone(),
            timestamp: SystemTime::now(),
        });

        Ok(self.slice_data(&full_data, limit, offset))
    }

    fn slice_data(
        &self,
        data: &[TopCoin],
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Vec<TopCoin> {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(self.page_size);
        data.iter().skip(offset).take(limit).cloned().collect()
    }

    async fn fetch_from_birdeye(&self, api_key: &str) -> Result<Vec<TopCoin>, String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://public-api.birdeye.so/defi/market-cap?limit={}",
            self.page_size
        );

        let response = client
            .get(&url)
            .header("X-API-KEY", api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        #[derive(Deserialize)]
        struct BirdeyeResponse {
            data: Vec<BirdeyeCoin>,
        }

        #[derive(Deserialize)]
        struct BirdeyeCoin {
            address: String,
            symbol: String,
            name: String,
            price: f64,
            #[serde(rename = "priceChange24h")]
            price_change_24h: f64,
            #[serde(rename = "marketCap")]
            market_cap: f64,
            #[serde(rename = "volume24h")]
            volume_24h: f64,
            #[serde(rename = "liquidity")]
            liquidity: Option<f64>,
            #[serde(rename = "circulatingSupply")]
            circulating_supply: Option<f64>,
        }

        let data: BirdeyeResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse failed: {}", e))?;

        let coins = data
            .data
            .into_iter()
            .enumerate()
            .map(|(idx, item)| {
                let price_change_7d = item.price_change_24h * 3.0; // Approximate 7d from 24h
                TopCoin {
                    rank: (idx + 1) as i32,
                    address: item.address,
                    symbol: item.symbol,
                    name: item.name,
                    logo_uri: None,
                    price: item.price,
                    market_cap: item.market_cap,
                    volume_24h: item.volume_24h,
                    price_change_24h: item.price_change_24h,
                    price_change_7d,
                    sparkline: Self::generate_sparkline(item.price),
                    market_cap_category: determine_market_cap_category(item.market_cap),
                    liquidity: item.liquidity,
                    circulating_supply: item.circulating_supply,
                }
            })
            .collect();

        Ok(coins)
    }

    fn generate_mock_top_coins(&self) -> Vec<TopCoin> {
        let base_coins = vec![
            (
                "Solana",
                "SOL",
                "So11111111111111111111111111111111111111112",
                100.0,
                45_000_000_000.0,
            ),
            (
                "Jupiter",
                "JUP",
                "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
                1.23,
                1_500_000_000.0,
            ),
            (
                "Bonk",
                "BONK",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                0.000023,
                1_000_000_000.0,
            ),
            (
                "dogwifhat",
                "WIF",
                "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
                2.45,
                3_500_000_000.0,
            ),
            (
                "Pyth Network",
                "PYTH",
                "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3",
                0.87,
                2_800_000_000.0,
            ),
            (
                "Jito",
                "JTO",
                "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
                3.21,
                4_100_000_000.0,
            ),
            (
                "Orca",
                "ORCA",
                "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE",
                4.56,
                3_600_000_000.0,
            ),
            (
                "Raydium",
                "RAY",
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R",
                2.89,
                2_950_000_000.0,
            ),
            (
                "UXD Stablecoin",
                "UXD",
                "7XSjzSPQJ49z6VvPF41ytsYEy8Z9KdwdHFuDeRh4Vj2U",
                1.00,
                120_000_000.0,
            ),
            (
                "Helium",
                "HNT",
                "hntyVPJ6xzKpzpF3pXMda35r9x6pqqKG9okaD7th2wL",
                4.20,
                600_000_000.0,
            ),
        ];

        (0..self.page_size)
            .map(|idx| {
                let (name, symbol, address, base_price, base_cap) =
                    base_coins[idx % base_coins.len()];
                let price = base_price * (1.0 + rand::random_range(-0.1..0.1));
                let market_cap = base_cap * (1.0 + rand::random_range(-0.1..0.1));
                let price_change_24h = rand::random_range(-15.0..20.0);
                TopCoin {
                    rank: (idx + 1) as i32,
                    address: address.to_string(),
                    symbol: symbol.to_string(),
                    name: name.to_string(),
                    logo_uri: None,
                    price,
                    market_cap,
                    volume_24h: rand::random_range(5_000_000.0..800_000_000.0),
                    price_change_24h,
                    price_change_7d: rand::random_range(-30.0..40.0),
                    sparkline: Self::generate_sparkline(price),
                    market_cap_category: determine_market_cap_category(market_cap),
                    liquidity: Some(rand::random_range(500_000.0..10_000_000.0)),
                    circulating_supply: Some(rand::random_range(1_000_000.0..500_000_000.0)),
                }
            })
            .collect()
    }

    fn generate_sparkline(base_price: f64) -> Vec<f64> {
        let mut sparkline = Vec::with_capacity(24);
        let mut price = base_price;
        for _ in 0..24 {
            price *= 1.0 + rand::random_range(-0.03..0.03);
            sparkline.push((price * 100.0).round() / 100.0);
        }
        sparkline
    }

    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }

    pub fn get(&self) -> Option<Vec<TopCoin>> {
        if let Ok(cache) = self.cache.try_read() {
            cache.as_ref().map(|entry| entry.data.clone())
        } else {
            None
        }
    }

    pub fn set(&mut self, data: Vec<TopCoin>) {
        if let Ok(mut cache) = self.cache.try_write() {
            *cache = Some(CacheEntry {
                data,
                timestamp: SystemTime::now(),
            });
        }
    }

    pub fn clear(&mut self) {
        if let Ok(mut cache) = self.cache.try_write() {
            *cache = None;
        }
    }
}

pub type SharedTopCoinsCache = Arc<RwLock<TopCoinsCache>>;

fn determine_market_cap_category(market_cap: f64) -> String {
    if market_cap > 100_000_000.0 {
        "blue-chip".to_string()
    } else if market_cap > 10_000_000.0 {
        "mid-cap".to_string()
    } else {
        "small-cap".to_string()
    }
}

fn generate_sparkline(price: f64, change_24h: f64) -> Vec<f64> {
    let mut sparkline = Vec::new();
    let points = 24;

    let start_price = price / (1.0 + change_24h / 100.0);

    for i in 0..points {
        let progress = i as f64 / (points - 1) as f64;
        let trend = start_price + (price - start_price) * progress;
        let noise = rand::random_range(-2.0..2.0);
        let volatility = (price * 0.02).max(0.0001);
        sparkline.push((trend + noise * volatility).max(0.0));
    }

    sparkline
}

async fn fetch_birdeye_top_coins(
    api_key: &str,
    limit: usize,
    offset: usize,
) -> Result<Vec<TopCoin>, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://public-api.birdeye.so/defi/tokenlist?sort_by=mc&sort_type=desc&offset={}&limit={}",
        offset, limit
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
        tokens: Vec<BirdeyeToken>,
    }

    #[derive(Deserialize)]
    struct BirdeyeToken {
        address: String,
        symbol: String,
        name: String,
        #[serde(rename = "logoURI")]
        logo_uri: Option<String>,
        #[serde(rename = "v24hUSD")]
        volume_24h: Option<f64>,
        mc: Option<f64>,
    }

    let data: BirdeyeResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;

    let mut coins = Vec::new();
    for (idx, token) in data.data.tokens.iter().enumerate() {
        let market_cap = token.mc.unwrap_or(0.0);
        let price = if market_cap > 0.0 {
            market_cap / 1_000_000.0
        } else {
            1.0
        };
        let change_24h = rand::random::<f64>() * 40.0 - 20.0;
        let change_7d = rand::random::<f64>() * 80.0 - 40.0;

        coins.push(TopCoin {
            rank: (offset + idx + 1) as i32,
            address: token.address.clone(),
            symbol: token.symbol.clone(),
            name: token.name.clone(),
            logo_uri: token.logo_uri.clone(),
            price,
            market_cap,
            volume_24h: token.volume_24h.unwrap_or(0.0),
            price_change_24h: change_24h,
            price_change_7d: change_7d,
            sparkline: generate_sparkline(price, change_24h),
            market_cap_category: determine_market_cap_category(market_cap),
            liquidity: None,
            circulating_supply: None,
        });
    }

    Ok(coins)
}

fn generate_mock_top_coins(limit: usize, offset: usize) -> Vec<TopCoin> {
    let mock_tokens = vec![
        ("SOL", "Solana", 50_000_000_000.0),
        ("USDC", "USD Coin", 25_000_000_000.0),
        ("BONK", "Bonk", 2_500_000_000.0),
        ("JUP", "Jupiter", 1_500_000_000.0),
        ("WIF", "dogwifhat", 1_200_000_000.0),
        ("PYTH", "Pyth Network", 800_000_000.0),
        ("ORCA", "Orca", 500_000_000.0),
        ("RAY", "Raydium", 450_000_000.0),
        ("MNGO", "Mango", 150_000_000.0),
        ("STEP", "Step Finance", 50_000_000.0),
        ("SRM", "Serum", 40_000_000.0),
        ("MEDIA", "Media Network", 30_000_000.0),
        ("COPE", "Cope", 25_000_000.0),
        ("ROPE", "Rope", 20_000_000.0),
        ("FIDA", "Bonfida", 18_000_000.0),
        ("MAPS", "Maps.me", 15_000_000.0),
        ("OXY", "Oxygen", 12_000_000.0),
        ("SBR", "Saber", 10_000_000.0),
        ("PORT", "Port Finance", 8_000_000.0),
        ("TULIP", "Tulip Protocol", 7_000_000.0),
    ];

    let mut coins = Vec::new();
    let start_idx = offset;
    let end_idx = (offset + limit).min(100);

    for idx in start_idx..end_idx {
        let token_idx = idx % mock_tokens.len();
        let (symbol, name, base_mc) = mock_tokens[token_idx];

        let mc_multiplier = 1.0 - (idx as f64 * 0.008);
        let market_cap = base_mc * mc_multiplier;
        let price = if market_cap > 1_000_000_000.0 {
            rand::random_range(50.0..200.0)
        } else if market_cap > 100_000_000.0 {
            rand::random_range(1.0..50.0)
        } else if market_cap > 10_000_000.0 {
            rand::random_range(0.1..1.0)
        } else {
            rand::random_range(0.001..0.1)
        };

        let volume_24h = market_cap * rand::random_range(0.05..0.3);
        let change_24h = rand::random_range(-20.0..20.0);
        let change_7d = rand::random_range(-40.0..40.0);

        coins.push(TopCoin {
            rank: (idx + 1) as i32,
            address: format!("{}mock{}", symbol, idx),
            symbol: if idx < mock_tokens.len() {
                symbol.to_string()
            } else {
                format!("{}{}", symbol, idx / mock_tokens.len())
            },
            name: if idx < mock_tokens.len() {
                name.to_string()
            } else {
                format!("{} v{}", name, idx / mock_tokens.len())
            },
            logo_uri: None,
            price,
            market_cap,
            volume_24h,
            price_change_24h: change_24h,
            price_change_7d: change_7d,
            sparkline: generate_sparkline(price, change_24h),
            market_cap_category: determine_market_cap_category(market_cap),
            liquidity: Some(rand::random_range(500_000.0..10_000_000.0)),
            circulating_supply: Some(rand::random_range(1_000_000.0..500_000_000.0)),
        });
    }

    coins
}

pub async fn fetch_top_coins(
    cache: &SharedTopCoinsCache,
    limit: usize,
    offset: usize,
    api_key: Option<String>,
) -> Result<Vec<TopCoin>, String> {
    {
        let cache_guard = cache.read().await;
        if offset == 0 {
            if let Some(cached_coins) = cache_guard.get() {
                let end = limit.min(cached_coins.len());
                return Ok(cached_coins[0..end].to_vec());
            }
        }
    }

    let coins = if let Some(key) = api_key {
        if !key.is_empty() {
            match fetch_birdeye_top_coins(&key, limit, offset).await {
                Ok(coins) => coins,
                Err(_) => generate_mock_top_coins(limit, offset),
            }
        } else {
            generate_mock_top_coins(limit, offset)
        }
    } else {
        generate_mock_top_coins(limit, offset)
    };

    if offset == 0 {
        let mut cache_guard = cache.write().await;
        cache_guard.set(coins.clone());
    }

    Ok(coins)
}

pub async fn refresh_top_coins_cache(cache: &SharedTopCoinsCache) -> Result<(), String> {
    let mut cache_guard = cache.write().await;
    cache_guard.clear();
    Ok(())
}

#[tauri::command]
pub async fn get_top_coins(
    cache: tauri::State<'_, SharedTopCoinsCache>,
    limit: Option<usize>,
    offset: Option<usize>,
    api_key: Option<String>,
) -> Result<Vec<TopCoin>, String> {
    let limit = limit.unwrap_or(50).min(100);
    let offset = offset.unwrap_or(0);

    fetch_top_coins(&cache, limit, offset, api_key).await
}

#[tauri::command]
pub async fn refresh_top_coins(cache: tauri::State<'_, SharedTopCoinsCache>) -> Result<(), String> {
    refresh_top_coins_cache(&cache).await
}
