use super::{types::*, AlertManager, SmartMoneyDetector};
use crate::core::WebSocketManager;
use crate::websocket::types::{StreamEvent, TransactionUpdate};
use chrono::Utc;
use serde_json::json;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Listener, Manager};
use tokio::sync::{broadcast, OnceCell, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Clone)]
pub struct WalletMonitor {
    db: Arc<RwLock<WalletMonitorDatabase>>,
    app_handle: AppHandle,
    ws_manager: WebSocketManager,
    smart_money_detector: Arc<SmartMoneyDetector>,
    alert_manager: Arc<AlertManager>,
    monitored_wallets: Arc<RwLock<HashSet<String>>>,
    processed_transactions: Arc<RwLock<HashSet<String>>>,
    event_handler: Arc<tokio::sync::Mutex<Option<tauri::EventId>>>,
    batch_queue: Arc<tokio::sync::Mutex<Vec<WalletActivity>>>,
}

impl WalletMonitor {
    pub fn new(
        db: Arc<RwLock<WalletMonitorDatabase>>,
        app_handle: AppHandle,
        ws_manager: WebSocketManager,
        smart_money_detector: Arc<SmartMoneyDetector>,
        alert_manager: Arc<AlertManager>,
    ) -> Self {
        Self {
            db,
            app_handle,
            ws_manager,
            smart_money_detector,
            alert_manager,
            monitored_wallets: Arc::new(RwLock::new(HashSet::new())),
            processed_transactions: Arc::new(RwLock::new(HashSet::new())),
            event_handler: Arc::new(tokio::sync::Mutex::new(None)),
            batch_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        let wallets = self
            .db
            .read()
            .await
            .get_active_monitored_wallets()
            .await
            .map_err(|e| format!("Failed to load monitored wallets: {e}"))?;

        let addresses: Vec<String> = wallets.iter().map(|w| w.wallet_address.clone()).collect();

        if !addresses.is_empty() {
            self.ws_manager
                .subscribe_wallets(addresses.clone())
                .await
                .map_err(|e| format!("Failed to subscribe to wallet streams: {e}"))?;

            let mut monitored = self.monitored_wallets.write().await;
            monitored.clear();
            monitored.extend(addresses);
        }

        self.attach_event_listener().await?;

        Ok(())
    }

    async fn attach_event_listener(&self) -> Result<(), String> {
        let mut handler = self.event_handler.lock().await;
        if handler.is_some() {
            return Ok(());
        }

        let monitor = self.clone();
        let event_handler = self
            .app_handle
            .listen("transaction_update", move |event| {
                let payload = event.payload();
                if let Ok(stream_event) = serde_json::from_str::<StreamEvent>(payload) {
                    if let StreamEvent::TransactionUpdate(tx) = stream_event {
                        let monitor_clone = monitor.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(err) = monitor_clone.process_transaction(tx).await {
                                eprintln!("Failed to process wallet transaction: {err}");
                            }
                        });
                    }
                }
            });

        *handler = Some(event_handler);
        Ok(())
    }

    pub async fn add_wallet(
        &self,
        request: AddMonitoredWalletRequest,
    ) -> Result<MonitoredWallet, String> {
        let wallet = MonitoredWallet {
            id: Uuid::new_v4().to_string(),
            wallet_address: request.wallet_address.clone(),
            label: request.label,
            min_transaction_size: request.min_transaction_size,
            is_whale: request.is_whale,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.db
            .write()
            .await
            .add_monitored_wallet(&wallet)
            .await
            .map_err(|e| format!("Failed to add monitored wallet: {e}"))?;

        self.ws_manager
            .subscribe_wallets(vec![request.wallet_address.clone()])
            .await
            .map_err(|e| format!("Failed to subscribe to wallet: {e}"))?;

        self.monitored_wallets
            .write()
            .await
            .insert(request.wallet_address);

        Ok(wallet)
    }

    pub async fn remove_wallet(&self, id: &str) -> Result<(), String> {
        let wallet = self
            .db
            .read()
            .await
            .get_monitored_wallet(id)
            .await
            .map_err(|e| format!("Failed to get wallet: {e}"))?
            .ok_or_else(|| "Wallet not found".to_string())?;

        self.db
            .write()
            .await
            .remove_monitored_wallet(id)
            .await
            .map_err(|e| format!("Failed to remove wallet: {e}"))?;

        self.ws_manager
            .unsubscribe_wallets(vec![wallet.wallet_address.clone()])
            .await
            .ok();

        self.monitored_wallets
            .write()
            .await
            .remove(&wallet.wallet_address);

        Ok(())
    }

    pub async fn update_wallet(
        &self,
        request: UpdateMonitoredWalletRequest,
    ) -> Result<MonitoredWallet, String> {
        let wallet = self
            .db
            .read()
            .await
            .get_monitored_wallet(&request.id)
            .await
            .map_err(|e| format!("Failed to get wallet: {e}"))?
            .ok_or_else(|| "Wallet not found".to_string())?;

        self.db
            .write()
            .await
            .update_monitored_wallet(
                &request.id,
                request.label,
                request.min_transaction_size,
                request.is_whale,
                request.is_active,
            )
            .await
            .map_err(|e| format!("Failed to update wallet: {e}"))?;

        if let Some(is_active) = request.is_active {
            if is_active && !wallet.is_active {
                self.ws_manager
                    .subscribe_wallets(vec![wallet.wallet_address.clone()])
                    .await
                    .ok();
                self.monitored_wallets
                    .write()
                    .await
                    .insert(wallet.wallet_address.clone());
            } else if !is_active && wallet.is_active {
                self.ws_manager
                    .unsubscribe_wallets(vec![wallet.wallet_address.clone()])
                    .await
                    .ok();
                self.monitored_wallets
                    .write()
                    .await
                    .remove(&wallet.wallet_address);
            }
        }

        self.db
            .read()
            .await
            .get_monitored_wallet(&request.id)
            .await
            .map_err(|e| format!("Failed to get updated wallet: {e}"))?
            .ok_or_else(|| "Wallet not found after update".to_string())
    }

    pub async fn list_wallets(&self) -> Result<Vec<MonitoredWallet>, String> {
        self.db
            .read()
            .await
            .list_monitored_wallets()
            .await
            .map_err(|e| format!("Failed to list wallets: {e}"))
    }

    pub async fn get_activities(
        &self,
        filter: ActivityFilter,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<WalletActivity>, String> {
        let records = self
            .db
            .read()
            .await
            .get_activities(&filter, limit, offset)
            .await
            .map_err(|e| format!("Failed to get activities: {e}"))?;

        let wallets = self.list_wallets().await?;
        let wallet_map: std::collections::HashMap<String, Option<String>> = wallets
            .into_iter()
            .map(|w| (w.wallet_address.clone(), w.label))
            .collect();

        let activities: Vec<WalletActivity> = records
            .into_iter()
            .map(|r| {
                let wallet_address = r.wallet_address.clone();
                let is_whale = wallet_map.get(&wallet_address).is_some();

                WalletActivity {
                    id: r.id,
                    wallet_address: r.wallet_address.clone(),
                    wallet_label: wallet_map.get(&r.wallet_address).cloned().flatten(),
                    tx_signature: r.tx_signature,
                    action_type: r.action_type,
                    input_mint: r.input_mint,
                    output_mint: r.output_mint,
                    input_symbol: r.input_symbol,
                    output_symbol: r.output_symbol,
                    amount: r.amount,
                    amount_usd: r.amount_usd,
                    price: r.price,
                    is_whale,
                    timestamp: r.timestamp,
                }
            })
            .collect();

        Ok(activities)
    }

    pub async fn get_wallet_statistics(
        &self,
        wallet_address: &str,
    ) -> Result<WalletStatistics, String> {
        self.db
            .read()
            .await
            .get_wallet_statistics(wallet_address)
            .await
            .map_err(|e| format!("Failed to get statistics: {e}"))
    }

    pub async fn run_batch_processor(self: Arc<Self>) {
        let mut ticker = interval(Duration::from_millis(500));
        loop {
            ticker.tick().await;
            let mut queue = self.batch_queue.lock().await;
            if queue.is_empty() {
                continue;
            }
            let activities: Vec<WalletActivity> = queue.drain(..).collect();
            drop(queue);

            let batch = WalletActivityBatch {
                activities: activities.clone(),
                timestamp: Utc::now(),
            };

            let _ = self.app_handle.emit("wallet_activity_batch", &batch);
        }
    }

    async fn process_transaction(&self, tx: TransactionUpdate) -> Result<(), String> {
        {
            let mut seen = self.processed_transactions.write().await;
            if seen.contains(&tx.signature) {
                return Ok(());
            }
            seen.insert(tx.signature.clone());
            if seen.len() > 100_000 {
                seen.clear();
            }
        }

        let from_address = tx.from.clone().unwrap_or_default();
        let to_address = tx.to.clone().unwrap_or_default();

        let monitored = self.monitored_wallets.read().await;
        let relevant_wallet = if monitored.contains(&from_address) {
            Some(from_address.clone())
        } else if monitored.contains(&to_address) {
            Some(to_address.clone())
        } else {
            None
        };

        if let Some(wallet_address) = relevant_wallet {
            let action_type = tx.typ.as_deref().unwrap_or("unknown");

            let activity = WalletActivityRecord {
                id: Uuid::new_v4().to_string(),
                wallet_address: wallet_address.clone(),
                tx_signature: tx.signature.clone(),
                action_type: action_type.to_string(),
                input_mint: tx.from.clone(),
                output_mint: tx.to.clone(),
                input_symbol: None,
                output_symbol: tx.symbol.clone(),
                amount: tx.amount,
                amount_usd: tx.amount,
                price: Some(1.0),
                timestamp: chrono::DateTime::from_timestamp(tx.timestamp, 0)
                    .unwrap_or_else(|| Utc::now()),
            };

            self.db
                .write()
                .await
                .add_activity(&activity)
                .await
                .map_err(|e| format!("Failed to save activity: {e}"))?;

            let wallets = self.list_wallets().await?;
            let wallet_info = wallets.iter().find(|w| w.wallet_address == wallet_address);

            let whale_threshold = 50_000.0;
            let is_whale_flag = wallet_info.map(|w| w.is_whale).unwrap_or(false)
                || activity.amount_usd.unwrap_or(0.0).ge(&whale_threshold);

            let wallet_activity = WalletActivity {
                id: activity.id.clone(),
                wallet_address: wallet_address.clone(),
                wallet_label: wallet_info.and_then(|w| w.label.clone()),
                tx_signature: activity.tx_signature.clone(),
                action_type: activity.action_type.clone(),
                input_mint: activity.input_mint.clone(),
                output_mint: activity.output_mint.clone(),
                input_symbol: activity.input_symbol.clone(),
                output_symbol: activity.output_symbol.clone(),
                amount: activity.amount,
                amount_usd: activity.amount_usd,
                price: activity.price,
                is_whale: is_whale_flag,
                timestamp: activity.timestamp,
            };

            let _ = self.app_handle.emit("wallet_activity", &wallet_activity);

            if let Err(err) = self
                .alert_manager
                .process_whale_transaction(&wallet_activity)
                .await
            {
                eprintln!("Failed to process whale alert: {err}");
            }

            match self
                .smart_money_detector
                .classify_wallet(&wallet_address)
                .await
            {
                Ok(classification) => {
                    if let Err(err) = self
                        .smart_money_detector
                        .update_smart_money_wallet(
                            &classification,
                            wallet_activity.wallet_label.as_deref(),
                        )
                        .await
                    {
                        eprintln!(
                            "Failed to update smart money wallet {}: {}",
                            wallet_address, err
                        );
                    }

                    let _ = self
                        .app_handle
                        .emit("smart_money_classification", &classification);

                    if classification.is_smart_money {
                        if let Err(err) = self
                            .alert_manager
                            .process_smart_money_activity(&wallet_activity, true)
                            .await
                        {
                            eprintln!(
                                "Failed to dispatch smart money alert for {}: {}",
                                wallet_address, err
                            );
                        }
                    }
                }
                Err(err) => {
                    eprintln!(
                        "Failed to classify smart money wallet {}: {}",
                        wallet_address, err
                    );
                }
            }

            if let Some(amount_usd) = wallet_activity.amount_usd {
                if let Some(info) = wallet_info {
                    if let Some(min_size) = info.min_transaction_size {
                        if amount_usd >= min_size {
                            let _ = self
                                .app_handle
                                .emit("wallet_large_transaction", &wallet_activity);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn start_monitoring(monitor: Arc<Self>) {
        let mut ticker = interval(Duration::from_secs(120));
        loop {
            ticker.tick().await;
            if let Err(err) = monitor.initialize().await {
                eprintln!("Failed to refresh monitored wallets: {err}");
            }
        }
    }
}

pub struct WalletMonitorState {
    pub db: Arc<RwLock<WalletMonitorDatabase>>,
    pub monitor: Arc<WalletMonitor>,
    pub smart_money_detector: Arc<SmartMoneyDetector>,
    pub alert_manager: Arc<AlertManager>,
}

static WALLET_MONITOR_STATE: OnceCell<WalletMonitorState> = OnceCell::const_new();

pub async fn init_wallet_monitor(app_handle: &AppHandle) -> Result<(), String> {
    if WALLET_MONITOR_STATE.get().is_some() {
        return Ok(());
    }

    let app = app_handle.clone();
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Unable to resolve app data directory: {}", e))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {e}"))?;

    let mut db_path = PathBuf::from(&app_dir);
    db_path.push("wallet_monitor.db");

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&db_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {e}"))?;

    let db = WalletMonitorDatabase::new(pool.clone())
        .await
        .map_err(|e| format!("Failed to initialize wallet monitor database: {e}"))?;

    let shared_db = Arc::new(RwLock::new(db));

    let ws_manager = app_handle.state::<WebSocketManager>().inner().clone();

    let pool_for_detector = pool.clone();
    let smart_money_detector = Arc::new(SmartMoneyDetector::new(pool_for_detector));

    let alert_manager = Arc::new(AlertManager::new(pool.clone(), app_handle.clone()));

    let monitor = Arc::new(WalletMonitor::new(
        shared_db.clone(),
        app_handle.clone(),
        ws_manager,
        smart_money_detector.clone(),
        alert_manager.clone(),
    ));

    monitor.initialize().await?;

    let monitor_clone = monitor.clone();
    tauri::async_runtime::spawn(async move {
        WalletMonitor::start_monitoring(monitor_clone).await;
    });

    let batch_processor = monitor.clone();
    tauri::async_runtime::spawn(async move {
        batch_processor.run_batch_processor().await;
    });

    WALLET_MONITOR_STATE
        .set(WalletMonitorState {
            db: shared_db,
            monitor: monitor.clone(),
            smart_money_detector: smart_money_detector.clone(),
            alert_manager: alert_manager.clone(),
        })
        .map_err(|_| "Wallet monitor state already initialized".to_string())?;

    Ok(())
}

pub(crate) fn require_state<'a>() -> Result<&'a WalletMonitorState, String> {
    WALLET_MONITOR_STATE
        .get()
        .ok_or_else(|| "Wallet monitor not initialized".to_string())
}

#[tauri::command]
pub async fn wallet_monitor_init(handle: AppHandle) -> Result<(), String> {
    init_wallet_monitor(&handle).await
}

#[tauri::command]
pub async fn wallet_monitor_add_wallet(
    request: AddMonitoredWalletRequest,
) -> Result<MonitoredWallet, String> {
    let state = require_state()?;
    state.monitor.add_wallet(request).await
}

#[tauri::command]
pub async fn wallet_monitor_remove_wallet(id: String) -> Result<(), String> {
    let state = require_state()?;
    state.monitor.remove_wallet(&id).await
}

#[tauri::command]
pub async fn wallet_monitor_update_wallet(
    request: UpdateMonitoredWalletRequest,
) -> Result<MonitoredWallet, String> {
    let state = require_state()?;
    state.monitor.update_wallet(request).await
}

#[tauri::command]
pub async fn wallet_monitor_list_wallets() -> Result<Vec<MonitoredWallet>, String> {
    let state = require_state()?;
    state.monitor.list_wallets().await
}

#[tauri::command]
pub async fn wallet_monitor_get_activities(
    filter: ActivityFilter,
    limit: i32,
    offset: i32,
) -> Result<Vec<WalletActivity>, String> {
    let state = require_state()?;
    state.monitor.get_activities(filter, limit, offset).await
}

#[tauri::command]
pub async fn wallet_monitor_get_statistics(
    wallet_address: String,
) -> Result<WalletStatistics, String> {
    let state = require_state()?;
    state.monitor.get_wallet_statistics(&wallet_address).await
}

#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn test_activity_action_from_str() {
        assert_eq!(ActivityAction::from_str("buy"), ActivityAction::Buy);
        assert_eq!(ActivityAction::from_str("BUY"), ActivityAction::Buy);
        assert_eq!(ActivityAction::from_str("sell"), ActivityAction::Sell);
        assert_eq!(
            ActivityAction::from_str("transfer"),
            ActivityAction::Transfer
        );
        assert_eq!(
            ActivityAction::from_str("unknown_action"),
            ActivityAction::Unknown
        );
    }
}
