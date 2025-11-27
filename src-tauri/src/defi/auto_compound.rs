use crate::defi::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompoundTransaction {
    pub position_id: String,
    pub timestamp: i64,
    pub rewards_claimed: Vec<Reward>,
    pub amount_compounded: f64,
    pub gas_cost: f64,
    pub net_gain: f64,
}

#[derive(Clone)]
pub struct AutoCompoundEngine {
    settings: Arc<RwLock<HashMap<String, AutoCompoundSettings>>>,
    history: Arc<RwLock<Vec<CompoundTransaction>>>,
}

impl Default for AutoCompoundEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoCompoundEngine {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn configure(&self, settings: AutoCompoundSettings) {
        let mut map = self.settings.write().await;
        map.insert(settings.position_id.clone(), settings);
    }

    pub async fn get_config(&self, position_id: &str) -> Option<AutoCompoundSettings> {
        let map = self.settings.read().await;
        map.get(position_id).cloned()
    }

    pub async fn analyze_positions(&self, positions: &[DeFiPosition]) -> Vec<AutoCompoundSettings> {
        positions
            .iter()
            .filter_map(|position| {
                if !position.rewards.is_empty() {
                    let total_rewards_value: f64 =
                        position.rewards.iter().map(|r| r.value_usd).sum();
                    let recommended_threshold = (position.value_usd * 0.02).max(10.0);

                    Some(AutoCompoundSettings {
                        position_id: position.id.clone(),
                        enabled: total_rewards_value >= recommended_threshold,
                        threshold: recommended_threshold,
                        frequency: 86400,
                        slippage_tolerance: 1.0,
                        gas_limit: 200000,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn should_compound(&self, position: &DeFiPosition) -> bool {
        let settings = match self.get_config(&position.id).await {
            Some(s) => s,
            None => return false,
        };

        if !settings.enabled {
            return false;
        }

        let total_rewards: f64 = position.rewards.iter().map(|r| r.value_usd).sum();
        total_rewards >= settings.threshold
    }

    pub async fn execute_compound(
        &self,
        position: &DeFiPosition,
    ) -> Result<CompoundTransaction, String> {
        let settings = match self.get_config(&position.id).await {
            Some(s) => s,
            None => return Err("Auto compound not configured for position".to_string()),
        };

        if !self.should_compound(position).await {
            return Err("Compound threshold not met".to_string());
        }

        let total_rewards_value: f64 = position.rewards.iter().map(|r| r.value_usd).sum();
        let gas_cost = 0.01;
        let net_gain = total_rewards_value - gas_cost;

        if net_gain <= 0.0 {
            return Err("Gas cost exceeds rewards value".to_string());
        }

        let transaction = CompoundTransaction {
            position_id: position.id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            rewards_claimed: position.rewards.clone(),
            amount_compounded: total_rewards_value,
            gas_cost,
            net_gain,
        };

        let mut history = self.history.write().await;
        history.push(transaction.clone());

        Ok(transaction)
    }

    pub async fn get_history(&self, position_id: &str) -> Vec<CompoundTransaction> {
        let history = self.history.read().await;
        history
            .iter()
            .filter(|tx| tx.position_id == position_id)
            .cloned()
            .collect()
    }

    pub async fn estimate_apy_boost(
        &self,
        position: &DeFiPosition,
        compound_frequency: u64,
    ) -> f64 {
        let daily_rate = position.apy / 365.0 / 100.0;
        let compounds_per_year = 365.0 * 86400.0 / compound_frequency as f64;
        let compound_apy = ((1.0 + daily_rate).powf(compounds_per_year) - 1.0) * 100.0;
        compound_apy - position.apy
    }
}

#[tauri::command]
pub async fn configure_auto_compound(settings: AutoCompoundSettings) -> Result<(), String> {
    let engine = AutoCompoundEngine::new();
    engine.configure(settings).await;
    Ok(())
}

#[tauri::command]
pub async fn get_auto_compound_config(
    position_id: String,
) -> Result<Option<AutoCompoundSettings>, String> {
    let engine = AutoCompoundEngine::new();
    Ok(engine.get_config(&position_id).await)
}

#[tauri::command]
pub async fn get_compound_history(position_id: String) -> Result<Vec<CompoundTransaction>, String> {
    let engine = AutoCompoundEngine::new();
    Ok(engine.get_history(&position_id).await)
}

#[tauri::command]
pub async fn estimate_compound_apy_boost(
    position_id: String,
    apy: f64,
    value_usd: f64,
    compound_frequency: u64,
) -> Result<f64, String> {
    let engine = AutoCompoundEngine::new();
    let position = DeFiPosition {
        id: position_id,
        protocol: Protocol::Kamino,
        position_type: PositionType::Farming,
        asset: "TEST".to_string(),
        amount: 0.0,
        value_usd,
        apy,
        rewards: vec![],
        health_factor: None,
        created_at: 0,
        last_updated: 0,
    };
    Ok(engine
        .estimate_apy_boost(&position, compound_frequency)
        .await)
}
