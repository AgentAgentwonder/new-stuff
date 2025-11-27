use crate::defi::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarginfiAccount {
    pub authority: String,
    pub assets: Vec<MarginfiPosition>,
    pub liabilities: Vec<MarginfiPosition>,
    pub bankruptcy: bool,
    pub health_factor: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarginfiPosition {
    pub bank: String,
    pub symbol: String,
    pub amount: f64,
    pub value_usd: f64,
    pub entry_price: f64,
    pub liquidation_price: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarginfiBank {
    pub address: String,
    pub symbol: String,
    pub lending_apy: f64,
    pub borrowing_apy: f64,
    pub total_deposits: f64,
    pub total_loans: f64,
    pub utilization: f64,
    pub risk_tier: RiskLevel,
}

#[derive(Clone, Default)]
pub struct MarginfiAdapter;

impl MarginfiAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_banks(&self) -> Result<Vec<MarginfiBank>, String> {
        Ok(self.generate_mock_banks())
    }

    pub async fn get_account(&self, wallet: &str) -> Result<Option<MarginfiAccount>, String> {
        Ok(Some(self.generate_mock_account(wallet)))
    }

    pub async fn get_positions(&self, wallet: &str) -> Result<Vec<DeFiPosition>, String> {
        let account = self.get_account(wallet).await?;
        let Some(account) = account else {
            return Ok(vec![]);
        };

        let timestamp = chrono::Utc::now().timestamp();
        let mut positions = Vec::new();

        for asset in account.assets {
            positions.push(DeFiPosition {
                id: format!("marginfi-asset-{}", asset.bank),
                protocol: Protocol::MarginFi,
                position_type: PositionType::Lending,
                asset: asset.symbol.clone(),
                amount: asset.amount,
                value_usd: asset.value_usd,
                apy: 7.4,
                rewards: vec![Reward {
                    token: "MRX".to_string(),
                    amount: 12.5,
                    value_usd: 25.0,
                }],
                health_factor: Some(account.health_factor),
                created_at: timestamp,
                last_updated: timestamp,
            });
        }

        for liability in account.liabilities {
            positions.push(DeFiPosition {
                id: format!("marginfi-liability-{}", liability.bank),
                protocol: Protocol::MarginFi,
                position_type: PositionType::Borrowing,
                asset: liability.symbol.clone(),
                amount: liability.amount,
                value_usd: liability.value_usd,
                apy: -5.2,
                rewards: vec![],
                health_factor: Some(account.health_factor),
                created_at: timestamp,
                last_updated: timestamp,
            });
        }

        Ok(positions)
    }

    fn generate_mock_banks(&self) -> Vec<MarginfiBank> {
        use rand::Rng;

        vec![
            MarginfiBank {
                address: "marginfi-usdc".to_string(),
                symbol: "USDC".to_string(),
                lending_apy: rand::random_range(4.0..9.0),
                borrowing_apy: rand::random_range(6.0..12.0),
                total_deposits: rand::random_range(30_000_000.0..120_000_000.0),
                total_loans: rand::random_range(10_000_000.0..80_000_000.0),
                utilization: rand::random_range(0.4..0.75),
                risk_tier: RiskLevel::Low,
            },
            MarginfiBank {
                address: "marginfi-sol".to_string(),
                symbol: "SOL".to_string(),
                lending_apy: rand::random_range(3.0..7.0),
                borrowing_apy: rand::random_range(5.0..11.0),
                total_deposits: rand::random_range(400_000.0..1_500_000.0),
                total_loans: rand::random_range(200_000.0..900_000.0),
                utilization: rand::random_range(0.35..0.70),
                risk_tier: RiskLevel::Medium,
            },
            MarginfiBank {
                address: "marginfi-eth".to_string(),
                symbol: "ETH".to_string(),
                lending_apy: rand::random_range(2.5..6.5),
                borrowing_apy: rand::random_range(4.5..9.5),
                total_deposits: rand::random_range(10_000.0..50_000.0),
                total_loans: rand::random_range(5_000.0..25_000.0),
                utilization: rand::random_range(0.30..0.65),
                risk_tier: RiskLevel::Medium,
            },
        ]
    }

    fn generate_mock_account(&self, wallet: &str) -> MarginfiAccount {
        use rand::Rng;

        MarginfiAccount {
            authority: wallet.to_string(),
            assets: vec![MarginfiPosition {
                bank: "marginfi-usdc".to_string(),
                symbol: "USDC".to_string(),
                amount: rand::random_range(5000.0..40000.0),
                value_usd: rand::random_range(5000.0..40000.0),
                entry_price: 1.0,
                liquidation_price: None,
            }],
            liabilities: vec![MarginfiPosition {
                bank: "marginfi-sol".to_string(),
                symbol: "SOL".to_string(),
                amount: rand::random_range(10.0..100.0),
                value_usd: rand::random_range(1500.0..15000.0),
                entry_price: rand::random_range(70.0..110.0),
                liquidation_price: Some(rand::random_range(35.0..60.0)),
            }],
            bankruptcy: false,
            health_factor: rand::random_range(1.7..3.5),
        }
    }
}

#[tauri::command]
pub async fn get_marginfi_banks() -> Result<Vec<MarginfiBank>, String> {
    MarginfiAdapter::new().get_banks().await
}

#[tauri::command]
pub async fn get_marginfi_positions(wallet: String) -> Result<Vec<DeFiPosition>, String> {
    MarginfiAdapter::new().get_positions(&wallet).await
}
