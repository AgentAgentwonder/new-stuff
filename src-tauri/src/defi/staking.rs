use crate::defi::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StakingRewardSchedule {
    pub period: String,
    pub reward_rate: f64,
    pub expected_apy: f64,
}

#[derive(Clone, Default)]
pub struct StakingAdapter;

impl StakingAdapter {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_pools(&self) -> Result<Vec<StakingPool>, String> {
        Ok(self.generate_mock_pools())
    }

    pub async fn get_positions(&self, _wallet: &str) -> Result<Vec<DeFiPosition>, String> {
        let pools = self.get_pools().await?;
        let timestamp = chrono::Utc::now().timestamp();

        let mut positions = Vec::new();
        for pool in pools {
            positions.push(DeFiPosition {
                id: format!("staking-{}", pool.pool_address),
                protocol: pool.protocol.clone(),
                position_type: PositionType::Staking,
                asset: pool.stake_token.clone(),
                amount: 100.0,
                value_usd: pool.total_staked / 1000.0,
                apy: pool.apy,
                rewards: vec![Reward {
                    token: pool.reward_token.clone(),
                    amount: 2.5,
                    value_usd: 2.5,
                }],
                health_factor: None,
                created_at: timestamp,
                last_updated: timestamp,
            });
        }
        Ok(positions)
    }

    pub async fn get_reward_schedule(
        &self,
        pool_id: &str,
    ) -> Result<Vec<StakingRewardSchedule>, String> {
        Ok(self.generate_mock_schedule(pool_id))
    }

    fn generate_mock_pools(&self) -> Vec<StakingPool> {
        vec![
            StakingPool {
                pool_address: "sol-stake-pool".to_string(),
                protocol: Protocol::Solend,
                stake_token: "SOL".to_string(),
                reward_token: "SOL".to_string(),
                total_staked: 45_000_000.0,
                apy: 6.5,
                lock_duration: None,
                early_withdrawal_penalty: 0.0,
                minimum_stake: 0.1,
            },
            StakingPool {
                pool_address: "mngo-stake-pool".to_string(),
                protocol: Protocol::MarginFi,
                stake_token: "MNGO".to_string(),
                reward_token: "MNGO".to_string(),
                total_staked: 8_500_000.0,
                apy: 14.2,
                lock_duration: Some(30),
                early_withdrawal_penalty: 2.0,
                minimum_stake: 100.0,
            },
            StakingPool {
                pool_address: "kmno-stake-pool".to_string(),
                protocol: Protocol::Kamino,
                stake_token: "KMNO".to_string(),
                reward_token: "KMNO".to_string(),
                total_staked: 12_300_000.0,
                apy: 18.7,
                lock_duration: Some(60),
                early_withdrawal_penalty: 3.0,
                minimum_stake: 50.0,
            },
        ]
    }

    fn generate_mock_schedule(&self, pool_id: &str) -> Vec<StakingRewardSchedule> {
        match pool_id {
            "sol-stake-pool" => vec![
                StakingRewardSchedule {
                    period: "Daily".to_string(),
                    reward_rate: 0.018,
                    expected_apy: 6.5,
                },
                StakingRewardSchedule {
                    period: "Weekly".to_string(),
                    reward_rate: 0.125,
                    expected_apy: 6.6,
                },
            ],
            "mngo-stake-pool" => vec![
                StakingRewardSchedule {
                    period: "Daily".to_string(),
                    reward_rate: 0.035,
                    expected_apy: 14.2,
                },
                StakingRewardSchedule {
                    period: "Monthly".to_string(),
                    reward_rate: 1.2,
                    expected_apy: 14.5,
                },
            ],
            _ => vec![
                StakingRewardSchedule {
                    period: "Daily".to_string(),
                    reward_rate: 0.040,
                    expected_apy: 18.7,
                },
                StakingRewardSchedule {
                    period: "Monthly".to_string(),
                    reward_rate: 1.45,
                    expected_apy: 19.2,
                },
            ],
        }
    }
}

#[tauri::command]
pub async fn get_staking_pools() -> Result<Vec<StakingPool>, String> {
    StakingAdapter::new().get_pools().await
}

#[tauri::command]
pub async fn get_staking_positions(wallet: String) -> Result<Vec<DeFiPosition>, String> {
    StakingAdapter::new().get_positions(&wallet).await
}

#[tauri::command]
pub async fn get_staking_schedule(pool_id: String) -> Result<Vec<StakingRewardSchedule>, String> {
    StakingAdapter::new().get_reward_schedule(&pool_id).await
}
