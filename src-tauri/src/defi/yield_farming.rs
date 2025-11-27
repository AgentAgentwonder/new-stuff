use crate::defi::types::*;
use serde::{Deserialize, Serialize};

// Custom YieldFarm structure for farming adapter (different from types::YieldFarm)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct YieldFarm {
    pub id: String,
    pub protocol: Protocol,
    pub name: String,
    pub farm_address: String,
    pub lp_token: String,
    pub reward_tokens: Vec<String>,
    pub tvl_usd: f64,
    pub base_apy: f64,
    pub reward_apy: f64,
    pub total_apy: f64,
    pub deposit_fee: f64,
    pub withdrawal_fee: f64,
    pub lock_period: Option<u64>,
    pub risk_score: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FarmingOpportunity {
    pub farm: YieldFarm,
    pub projected_earnings_24h: f64,
    pub projected_earnings_30d: f64,
    pub risk_adjusted_apy: f64,
}

#[derive(Clone, Default)]
pub struct YieldFarmingAdapter;

impl YieldFarmingAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_all_farms(&self) -> Result<Vec<YieldFarm>, String> {
        Ok(self.generate_mock_farms())
    }

    pub async fn get_opportunities(
        &self,
        min_apy: f64,
        max_risk: u8,
    ) -> Result<Vec<FarmingOpportunity>, String> {
        let farms = self.get_all_farms().await?;
        let opportunities: Vec<FarmingOpportunity> = farms
            .into_iter()
            .filter(|farm| farm.total_apy >= min_apy && farm.risk_score <= max_risk)
            .map(|farm| FarmingOpportunity {
                projected_earnings_24h: (farm.tvl_usd * farm.total_apy / 100.0) / 365.0,
                projected_earnings_30d: (farm.tvl_usd * farm.total_apy / 100.0) / 12.0,
                risk_adjusted_apy: farm.total_apy * (1.0 - (farm.risk_score as f64 / 100.0) * 0.3),
                farm,
            })
            .collect();
        Ok(opportunities)
    }

    pub async fn get_positions(&self, wallet: &str) -> Result<Vec<DeFiPosition>, String> {
        let positions = self.generate_mock_positions(wallet);
        Ok(positions)
    }

    fn generate_mock_farms(&self) -> Vec<YieldFarm> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        vec![
            YieldFarm {
                id: "raydium-sol-usdc".to_string(),
                protocol: Protocol::Raydium,
                name: "Raydium SOL-USDC Farm".to_string(),
                farm_address: "raydium-sol-usdc".to_string(),
                lp_token: "SOL-USDC LP".to_string(),
                reward_tokens: vec!["RAY".to_string()],
                tvl_usd: rng.random_range(20_000_000.0..80_000_000.0),
                base_apy: rng.random_range(8.0..12.0),
                reward_apy: rng.random_range(4.0..13.0),
                total_apy: rng.random_range(12.0..25.0),
                deposit_fee: 0.0,
                withdrawal_fee: 0.0,
                lock_period: None,
                risk_score: 45,
            },
            YieldFarm {
                id: "raydium-ray-sol".to_string(),
                protocol: Protocol::Raydium,
                name: "Raydium RAY-SOL Farm".to_string(),
                farm_address: "raydium-ray-sol".to_string(),
                lp_token: "RAY-SOL LP".to_string(),
                reward_tokens: vec!["RAY".to_string()],
                tvl_usd: rng.random_range(8_000_000.0..30_000_000.0),
                base_apy: rng.random_range(15.0..25.0),
                reward_apy: rng.random_range(10.0..20.0),
                total_apy: rng.random_range(25.0..45.0),
                deposit_fee: 0.0,
                withdrawal_fee: 0.0,
                lock_period: None,
                risk_score: 60,
            },
            YieldFarm {
                id: "orca-sol-usdc".to_string(),
                protocol: Protocol::Orca,
                name: "Orca SOL-USDC Whirlpool".to_string(),
                farm_address: "orca-sol-usdc".to_string(),
                lp_token: "SOL-USDC Whirlpool".to_string(),
                reward_tokens: vec!["ORCA".to_string()],
                tvl_usd: rng.random_range(18_000_000.0..70_000_000.0),
                base_apy: rng.random_range(10.0..15.0),
                reward_apy: rng.random_range(5.0..13.0),
                total_apy: rng.random_range(15.0..28.0),
                deposit_fee: 0.0,
                withdrawal_fee: 0.0,
                lock_period: None,
                risk_score: 40,
            },
            YieldFarm {
                id: "orca-orca-usdc".to_string(),
                protocol: Protocol::Orca,
                name: "Orca ORCA-USDC Whirlpool".to_string(),
                farm_address: "orca-orca-usdc".to_string(),
                lp_token: "ORCA-USDC Whirlpool".to_string(),
                reward_tokens: vec!["ORCA".to_string()],
                tvl_usd: rng.random_range(5_000_000.0..25_000_000.0),
                base_apy: rng.random_range(20.0..30.0),
                reward_apy: rng.random_range(10.0..25.0),
                total_apy: rng.random_range(30.0..55.0),
                deposit_fee: 0.0,
                withdrawal_fee: 0.0,
                lock_period: None,
                risk_score: 70,
            },
        ]
    }

    fn generate_mock_positions(&self, _wallet: &str) -> Vec<DeFiPosition> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let timestamp = chrono::Utc::now().timestamp();

        vec![
            DeFiPosition {
                id: "raydium-sol-usdc-farm".to_string(),
                protocol: Protocol::Raydium,
                position_type: PositionType::Farming,
                asset: "SOL-USDC LP".to_string(),
                amount: rng.random_range(500.0..5000.0),
                value_usd: rng.random_range(3000.0..30000.0),
                apy: rng.random_range(15.0..25.0),
                rewards: vec![Reward {
                    token: "RAY".to_string(),
                    amount: rng.random_range(5.0..50.0),
                    value_usd: rng.random_range(10.0..100.0),
                }],
                health_factor: None,
                created_at: timestamp,
                last_updated: timestamp,
            },
            DeFiPosition {
                id: "orca-sol-usdc-farm".to_string(),
                protocol: Protocol::Orca,
                position_type: PositionType::Farming,
                asset: "SOL-USDC Whirlpool".to_string(),
                amount: rng.random_range(300.0..3000.0),
                value_usd: rng.random_range(2000.0..20000.0),
                apy: rng.random_range(18.0..28.0),
                rewards: vec![Reward {
                    token: "ORCA".to_string(),
                    amount: rng.random_range(8.0..80.0),
                    value_usd: rng.random_range(15.0..150.0),
                }],
                health_factor: None,
                created_at: timestamp,
                last_updated: timestamp,
            },
        ]
    }
}

#[tauri::command]
pub async fn get_yield_farms() -> Result<Vec<YieldFarm>, String> {
    YieldFarmingAdapter::new().get_all_farms().await
}

#[tauri::command]
pub async fn get_farming_opportunities(
    min_apy: f64,
    max_risk: u8,
) -> Result<Vec<FarmingOpportunity>, String> {
    YieldFarmingAdapter::new()
        .get_opportunities(min_apy, max_risk)
        .await
}

#[tauri::command]
pub async fn get_farming_positions(wallet: String) -> Result<Vec<DeFiPosition>, String> {
    YieldFarmingAdapter::new().get_positions(&wallet).await
}
