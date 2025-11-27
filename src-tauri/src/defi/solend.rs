use crate::defi::types::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const SOLEND_API_BASE: &str = "https://api.solend.fi";
const CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolendReserve {
    pub address: String,
    pub symbol: String,
    pub liquidity_token: String,
    pub liquidity_mint: String,
    pub supply_apy: f64,
    pub borrow_apy: f64,
    pub total_supply: f64,
    pub total_borrow: f64,
    pub available_amount: f64,
    pub utilization_ratio: f64,
    pub ltv: f64,
    pub liquidation_threshold: f64,
    pub liquidation_penalty: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolendObligation {
    pub address: String,
    pub owner: String,
    pub deposits: Vec<SolendDeposit>,
    pub borrows: Vec<SolendBorrow>,
    pub borrowed_value: f64,
    pub deposited_value: f64,
    pub health_factor: f64,
    pub liquidation_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolendDeposit {
    pub reserve_address: String,
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SolendBorrow {
    pub reserve_address: String,
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
}

#[derive(Clone)]
struct CacheEntry {
    value: serde_json::Value,
    expires_at: Instant,
}

impl CacheEntry {
    fn new(value: serde_json::Value, ttl: Duration) -> Self {
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
pub struct SolendAdapter {
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl Default for SolendAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl SolendAdapter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn cache_get<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
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

    pub async fn fetch_reserves(&self) -> Result<Vec<SolendReserve>, String> {
        let cache_key = "solend:reserves:all";
        if let Some(reserves) = self.cache_get::<Vec<SolendReserve>>(cache_key).await {
            return Ok(reserves);
        }

        let reserves = self.generate_mock_reserves();
        self.cache_set(cache_key, &reserves).await;
        Ok(reserves)
    }

    pub async fn fetch_obligation(&self, wallet: &str) -> Result<Option<SolendObligation>, String> {
        let cache_key = format!("solend:obligation:{}", wallet);
        if let Some(obligation) = self.cache_get::<Option<SolendObligation>>(&cache_key).await {
            return Ok(obligation);
        }

        let obligation = self.generate_mock_obligation(wallet);
        self.cache_set(&cache_key, &obligation).await;
        Ok(obligation)
    }

    pub async fn get_lending_pools(&self) -> Result<Vec<LendingPool>, String> {
        let reserves = self.fetch_reserves().await?;
        let pools: Vec<LendingPool> = reserves
            .into_iter()
            .map(|reserve| LendingPool {
                pool_address: reserve.address.clone(),
                protocol: Protocol::Solend,
                asset: reserve.symbol.clone(),
                total_supply: reserve.total_supply,
                total_borrowed: reserve.total_borrow,
                supply_apy: reserve.supply_apy,
                borrow_apy: reserve.borrow_apy,
                utilization_rate: reserve.utilization_ratio,
                liquidation_threshold: reserve.liquidation_threshold,
                liquidation_bonus: reserve.liquidation_penalty,
            })
            .collect();
        Ok(pools)
    }

    pub async fn get_user_positions(&self, wallet: &str) -> Result<Vec<DeFiPosition>, String> {
        let obligation = self.fetch_obligation(wallet).await?;

        if let Some(obl) = obligation {
            let mut positions = Vec::new();
            let timestamp = chrono::Utc::now().timestamp();

            for deposit in obl.deposits {
                positions.push(DeFiPosition {
                    id: format!("solend-deposit-{}", deposit.reserve_address),
                    protocol: Protocol::Solend,
                    position_type: PositionType::Lending,
                    asset: deposit.symbol.clone(),
                    amount: deposit.amount,
                    value_usd: deposit.value_usd,
                    apy: 5.5,
                    rewards: vec![],
                    health_factor: Some(obl.health_factor),
                    created_at: timestamp,
                    last_updated: timestamp,
                });
            }

            for borrow in obl.borrows {
                positions.push(DeFiPosition {
                    id: format!("solend-borrow-{}", borrow.reserve_address),
                    protocol: Protocol::Solend,
                    position_type: PositionType::Borrowing,
                    asset: borrow.symbol.clone(),
                    amount: borrow.amount,
                    value_usd: borrow.value_usd,
                    apy: -3.2,
                    rewards: vec![],
                    health_factor: Some(obl.health_factor),
                    created_at: timestamp,
                    last_updated: timestamp,
                });
            }

            Ok(positions)
        } else {
            Ok(vec![])
        }
    }

    fn generate_mock_reserves(&self) -> Vec<SolendReserve> {
        use rand::Rng;

        vec![
            SolendReserve {
                address: "solend-usdc".to_string(),
                symbol: "USDC".to_string(),
                liquidity_token: "cUSDC".to_string(),
                liquidity_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                supply_apy: rand::random_range(3.5..8.5),
                borrow_apy: rand::random_range(5.0..12.0),
                total_supply: rand::random_range(50_000_000.0..200_000_000.0),
                total_borrow: rand::random_range(20_000_000.0..100_000_000.0),
                available_amount: rand::random_range(10_000_000.0..50_000_000.0),
                utilization_ratio: rand::random_range(0.5..0.85),
                ltv: 0.80,
                liquidation_threshold: 0.85,
                liquidation_penalty: 0.05,
            },
            SolendReserve {
                address: "solend-sol".to_string(),
                symbol: "SOL".to_string(),
                liquidity_token: "cSOL".to_string(),
                liquidity_mint: "So11111111111111111111111111111111111111112".to_string(),
                supply_apy: rand::random_range(2.0..6.0),
                borrow_apy: rand::random_range(4.5..10.0),
                total_supply: rand::random_range(500_000.0..2_000_000.0),
                total_borrow: rand::random_range(200_000.0..1_000_000.0),
                available_amount: rand::random_range(100_000.0..500_000.0),
                utilization_ratio: rand::random_range(0.4..0.75),
                ltv: 0.75,
                liquidation_threshold: 0.80,
                liquidation_penalty: 0.05,
            },
            SolendReserve {
                address: "solend-usdt".to_string(),
                symbol: "USDT".to_string(),
                liquidity_token: "cUSDT".to_string(),
                liquidity_mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(),
                supply_apy: rand::random_range(3.0..7.5),
                borrow_apy: rand::random_range(5.5..11.0),
                total_supply: rand::random_range(40_000_000.0..150_000_000.0),
                total_borrow: rand::random_range(15_000_000.0..80_000_000.0),
                available_amount: rand::random_range(8_000_000.0..40_000_000.0),
                utilization_ratio: rand::random_range(0.45..0.80),
                ltv: 0.80,
                liquidation_threshold: 0.85,
                liquidation_penalty: 0.05,
            },
        ]
    }

    fn generate_mock_obligation(&self, _wallet: &str) -> Option<SolendObligation> {
        use rand::Rng;

        Some(SolendObligation {
            address: "mock-obligation-address".to_string(),
            owner: _wallet.to_string(),
            deposits: vec![
                SolendDeposit {
                    reserve_address: "solend-usdc".to_string(),
                    symbol: "USDC".to_string(),
                    amount: rand::random_range(5000.0..50000.0),
                    value_usd: rand::random_range(5000.0..50000.0),
                },
                SolendDeposit {
                    reserve_address: "solend-sol".to_string(),
                    symbol: "SOL".to_string(),
                    amount: rand::random_range(50.0..500.0),
                    value_usd: rand::random_range(5000.0..50000.0),
                },
            ],
            borrows: vec![SolendBorrow {
                reserve_address: "solend-usdc".to_string(),
                symbol: "USDC".to_string(),
                amount: rand::random_range(1000.0..10000.0),
                value_usd: rand::random_range(1000.0..10000.0),
            }],
            borrowed_value: rand::random_range(1000.0..10000.0),
            deposited_value: rand::random_range(10000.0..100000.0),
            health_factor: rand::random_range(1.5..3.0),
            liquidation_threshold: 0.85,
        })
    }
}

#[tauri::command]
pub async fn get_solend_reserves() -> Result<Vec<SolendReserve>, String> {
    let adapter = SolendAdapter::new();
    adapter.fetch_reserves().await
}

#[tauri::command]
pub async fn get_solend_pools() -> Result<Vec<LendingPool>, String> {
    let adapter = SolendAdapter::new();
    adapter.get_lending_pools().await
}

#[tauri::command]
pub async fn get_solend_positions(wallet: String) -> Result<Vec<DeFiPosition>, String> {
    let adapter = SolendAdapter::new();
    adapter.get_user_positions(&wallet).await
}
