use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedMarketData {
    pub symbol: String,
    pub price: f64,
    pub change_24h: f64,
    pub volume_24h: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedPortfolioData {
    pub total_value: f64,
    pub total_change_24h: f64,
    pub total_change_pct: f64,
    pub top_holdings: Vec<ReducedHolding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedHolding {
    pub symbol: String,
    pub amount: f64,
    pub value: f64,
    pub change_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducedAlert {
    pub alert_id: String,
    pub symbol: String,
    pub condition: String,
    pub value: f64,
    pub triggered: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSyncData {
    pub markets: Vec<ReducedMarketData>,
    pub portfolio: Option<ReducedPortfolioData>,
    pub alerts: Vec<ReducedAlert>,
    pub last_sync: i64,
}

pub struct MobileSyncManager {
    last_sync_times: HashMap<String, i64>,
    cached_sync_data: HashMap<String, MobileSyncData>,
}

impl MobileSyncManager {
    pub fn new() -> Self {
        Self {
            last_sync_times: HashMap::new(),
            cached_sync_data: HashMap::new(),
        }
    }

    pub async fn sync_device(&mut self, device_id: String) -> Result<MobileSyncData> {
        let now = Utc::now().timestamp();
        self.last_sync_times.insert(device_id.clone(), now);

        let sync_data = MobileSyncData {
            markets: self.get_reduced_market_data().await?,
            portfolio: self.get_reduced_portfolio_data().await?,
            alerts: self.get_reduced_alerts().await?,
            last_sync: now,
        };

        self.cached_sync_data.insert(device_id, sync_data.clone());

        Ok(sync_data)
    }

    pub fn get_last_sync(&self, device_id: &str) -> Option<i64> {
        self.last_sync_times.get(device_id).copied()
    }

    pub fn get_cached_data(&self, device_id: &str) -> Option<MobileSyncData> {
        self.cached_sync_data.get(device_id).cloned()
    }

    async fn get_reduced_market_data(&self) -> Result<Vec<ReducedMarketData>> {
        let now = Utc::now().timestamp();

        Ok(vec![
            ReducedMarketData {
                symbol: "SOL".to_string(),
                price: 145.67,
                change_24h: 5.23,
                volume_24h: 2_345_678_901.0,
                timestamp: now,
            },
            ReducedMarketData {
                symbol: "USDC".to_string(),
                price: 1.00,
                change_24h: 0.01,
                volume_24h: 5_678_901_234.0,
                timestamp: now,
            },
            ReducedMarketData {
                symbol: "BONK".to_string(),
                price: 0.000023,
                change_24h: -2.45,
                volume_24h: 987_654_321.0,
                timestamp: now,
            },
        ])
    }

    async fn get_reduced_portfolio_data(&self) -> Result<Option<ReducedPortfolioData>> {
        Ok(Some(ReducedPortfolioData {
            total_value: 12_345.67,
            total_change_24h: 234.56,
            total_change_pct: 1.93,
            top_holdings: vec![
                ReducedHolding {
                    symbol: "SOL".to_string(),
                    amount: 50.0,
                    value: 7_283.50,
                    change_pct: 5.23,
                },
                ReducedHolding {
                    symbol: "USDC".to_string(),
                    amount: 5000.0,
                    value: 5000.0,
                    change_pct: 0.01,
                },
            ],
        }))
    }

    async fn get_reduced_alerts(&self) -> Result<Vec<ReducedAlert>> {
        let now = Utc::now().timestamp();

        Ok(vec![ReducedAlert {
            alert_id: "alert_1".to_string(),
            symbol: "SOL".to_string(),
            condition: "above".to_string(),
            value: 150.0,
            triggered: false,
            timestamp: now,
        }])
    }
}

// Tauri commands
#[tauri::command]
pub async fn mobile_sync_data(
    device_id: String,
    sync_manager: tauri::State<'_, Arc<RwLock<MobileSyncManager>>>,
    mobile_auth: tauri::State<'_, Arc<RwLock<crate::mobile::auth::MobileAuthManager>>>,
) -> Result<MobileSyncData, String> {
    let devices = {
        let auth = mobile_auth.read().await;
        auth.get_devices()
    };

    let device_registered = devices.iter().any(|device| device.device_id == device_id);
    if !device_registered {
        return Err("Device not registered".into());
    }

    let mut manager = sync_manager.write().await;
    manager
        .sync_device(device_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_get_last_sync(
    device_id: String,
    sync_manager: tauri::State<'_, Arc<RwLock<MobileSyncManager>>>,
) -> Result<Option<i64>, String> {
    let manager = sync_manager.read().await;
    Ok(manager.get_last_sync(&device_id))
}

#[tauri::command]
pub async fn mobile_get_cached_sync_data(
    device_id: String,
    sync_manager: tauri::State<'_, Arc<RwLock<MobileSyncManager>>>,
) -> Result<Option<MobileSyncData>, String> {
    let manager = sync_manager.read().await;
    Ok(manager.get_cached_data(&device_id))
}
