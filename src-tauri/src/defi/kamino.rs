use crate::defi::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KaminoVault {
    pub address: String,
    pub name: String,
    pub token_a: String,
    pub token_b: String,
    pub strategy: String,
    pub tvl: f64,
    pub apy: f64,
    pub fee_apr: f64,
    pub reward_apr: f64,
    pub auto_compound: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KaminoPosition {
    pub vault_address: String,
    pub shares: f64,
    pub token_a_amount: f64,
    pub token_b_amount: f64,
    pub value_usd: f64,
    pub unrealized_pnl: f64,
}

#[derive(Clone, Default)]
pub struct KaminoAdapter;

impl KaminoAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_vaults(&self) -> Result<Vec<KaminoVault>, String> {
        Ok(self.generate_mock_vaults())
    }

    pub async fn get_user_positions(&self, wallet: &str) -> Result<Vec<DeFiPosition>, String> {
        let positions = self.generate_mock_user_positions(wallet);
        Ok(positions)
    }

    pub async fn get_yield_farms(&self) -> Result<Vec<YieldFarm>, String> {
        let vaults = self.get_vaults().await?;
        let farms: Vec<YieldFarm> = vaults
            .into_iter()
            .map(|vault| {
                let lp_token = format!("{}-{}", vault.token_a, vault.token_b);
                YieldFarm {
                    farm_address: vault.address.clone(),
                    protocol: Protocol::Kamino,
                    lp_token,
                    reward_tokens: vec!["KMNO".to_string()],
                    tvl_usd: vault.tvl,
                    base_apy: vault.fee_apr,
                    reward_apy: vault.reward_apr,
                    total_apy: vault.apy,
                    deposit_fee: 0.0,
                    withdrawal_fee: 0.0,
                    lock_period: None,
                    risk_score: 65,
                }
            })
            .collect();
        Ok(farms)
    }

    fn generate_mock_vaults(&self) -> Vec<KaminoVault> {
        use rand::Rng;

        vec![
            KaminoVault {
                address: "kamino-sol-usdc".to_string(),
                name: "SOL-USDC Concentrated".to_string(),
                token_a: "SOL".to_string(),
                token_b: "USDC".to_string(),
                strategy: "Concentrated Liquidity".to_string(),
                tvl: rand::random_range(15_000_000.0..50_000_000.0),
                apy: rand::random_range(15.0..35.0),
                fee_apr: rand::random_range(8.0..20.0),
                reward_apr: rand::random_range(5.0..15.0),
                auto_compound: true,
            },
            KaminoVault {
                address: "kamino-eth-usdc".to_string(),
                name: "ETH-USDC Concentrated".to_string(),
                token_a: "ETH".to_string(),
                token_b: "USDC".to_string(),
                strategy: "Concentrated Liquidity".to_string(),
                tvl: rand::random_range(10_000_000.0..40_000_000.0),
                apy: rand::random_range(12.0..30.0),
                fee_apr: rand::random_range(7.0..18.0),
                reward_apr: rand::random_range(4.0..12.0),
                auto_compound: true,
            },
            KaminoVault {
                address: "kamino-btc-usdc".to_string(),
                name: "BTC-USDC Concentrated".to_string(),
                token_a: "BTC".to_string(),
                token_b: "USDC".to_string(),
                strategy: "Concentrated Liquidity".to_string(),
                tvl: rand::random_range(8_000_000.0..35_000_000.0),
                apy: rand::random_range(10.0..28.0),
                fee_apr: rand::random_range(6.0..16.0),
                reward_apr: rand::random_range(3.0..10.0),
                auto_compound: true,
            },
        ]
    }

    fn generate_mock_user_positions(&self, _wallet: &str) -> Vec<DeFiPosition> {
        use rand::Rng;
        let timestamp = chrono::Utc::now().timestamp();

        vec![DeFiPosition {
            id: "kamino-sol-usdc-lp".to_string(),
            protocol: Protocol::Kamino,
            position_type: PositionType::LiquidityPool,
            asset: "SOL-USDC".to_string(),
            amount: rand::random_range(1000.0..10000.0),
            value_usd: rand::random_range(5000.0..50000.0),
            apy: rand::random_range(18.0..32.0),
            rewards: vec![Reward {
                token: "KMNO".to_string(),
                amount: rand::random_range(10.0..100.0),
                value_usd: rand::random_range(50.0..500.0),
            }],
            health_factor: None,
            created_at: timestamp,
            last_updated: timestamp,
        }]
    }
}

#[tauri::command]
pub async fn get_kamino_vaults() -> Result<Vec<KaminoVault>, String> {
    KaminoAdapter::new().get_vaults().await
}

#[tauri::command]
pub async fn get_kamino_positions(wallet: String) -> Result<Vec<DeFiPosition>, String> {
    KaminoAdapter::new().get_user_positions(&wallet).await
}

#[tauri::command]
pub async fn get_kamino_farms() -> Result<Vec<YieldFarm>, String> {
    KaminoAdapter::new().get_yield_farms().await
}
