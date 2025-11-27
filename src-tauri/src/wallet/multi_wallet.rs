use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use tauri::State;
use uuid::Uuid;

use crate::security::keystore::{Keystore, KeystoreError};

const KEYSTORE_STATE_KEY: &str = "wallet.multi_state";

fn default_chain_id() -> String {
    "solana".to_string()
}

fn infer_chain_id(network: &str) -> String {
    let normalized = network.to_lowercase();
    if normalized.contains("sol") {
        "solana".to_string()
    } else if normalized.contains("eth") {
        "ethereum".to_string()
    } else if normalized.contains("base") {
        "base".to_string()
    } else if normalized.contains("matic") || normalized.contains("polygon") {
        "polygon".to_string()
    } else if normalized.contains("arb") {
        "arbitrum".to_string()
    } else {
        default_chain_id()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletInfo {
    pub id: String,
    pub public_key: String,
    pub label: String,
    pub network: String,
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
    pub wallet_type: WalletType,
    pub group_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub balance: f64,
    pub preferences: WalletPreferences,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletType {
    Phantom,
    HardwareLedger,
    HardwareTrezor,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletPreferences {
    pub trading_enabled: bool,
    pub auto_approve_limit: Option<f64>,
    pub max_slippage: Option<f64>,
    pub default_priority_fee: Option<u64>,
    pub notifications_enabled: bool,
    pub isolation_mode: bool,
}

impl Default for WalletPreferences {
    fn default() -> Self {
        Self {
            trading_enabled: true,
            auto_approve_limit: None,
            max_slippage: None,
            default_priority_fee: None,
            notifications_enabled: true,
            isolation_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_volume: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub wallet_ids: Vec<String>,
    pub shared_settings: GroupSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupSettings {
    pub max_slippage: Option<f64>,
    pub default_priority_fee: Option<u64>,
    pub risk_level: Option<String>,
    pub auto_rebalance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiWalletState {
    pub wallets: HashMap<String, WalletInfo>,
    pub groups: HashMap<String, WalletGroup>,
    pub active_wallet_id: Option<String>,
    pub last_updated: DateTime<Utc>,
}

impl Default for MultiWalletState {
    fn default() -> Self {
        Self {
            wallets: HashMap::new(),
            groups: HashMap::new(),
            active_wallet_id: None,
            last_updated: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWalletRequest {
    pub public_key: String,
    pub label: String,
    pub network: String,
    pub wallet_type: WalletType,
    pub group_id: Option<String>,
    pub chain_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWalletRequest {
    pub wallet_id: String,
    pub label: Option<String>,
    pub group_id: Option<Option<String>>,
    pub preferences: Option<WalletPreferences>,
    pub chain_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub wallet_ids: Vec<String>,
    pub shared_settings: Option<GroupSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupRequest {
    pub group_id: String,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub wallet_ids: Option<Vec<String>>,
    pub shared_settings: Option<GroupSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedPortfolio {
    pub total_balance: f64,
    pub total_wallets: usize,
    pub total_groups: usize,
    pub total_trades: u64,
    pub total_volume: f64,
    pub total_realized_pnl: f64,
    pub total_unrealized_pnl: f64,
    pub wallets: Vec<WalletInfo>,
}

#[derive(Debug, thiserror::Error)]
pub enum MultiWalletError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("wallet not found: {0}")]
    WalletNotFound(String),
    #[error("group not found: {0}")]
    GroupNotFound(String),
    #[error("wallet already exists: {0}")]
    WalletExists(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    #[error("internal error")]
    Internal,
}

pub struct MultiWalletManager {
    state: Mutex<MultiWalletState>,
}

impl MultiWalletManager {
    pub fn initialize(keystore: &Keystore) -> Result<Self, MultiWalletError> {
        let state = match keystore.retrieve_secret(KEYSTORE_STATE_KEY) {
            Ok(raw) => serde_json::from_slice::<MultiWalletState>(&raw)?,
            Err(KeystoreError::NotFound) => MultiWalletState::default(),
            Err(err) => return Err(MultiWalletError::Keystore(err)),
        };

        Ok(Self {
            state: Mutex::new(state),
        })
    }

    pub fn add_wallet(
        &self,
        request: AddWalletRequest,
        keystore: &Keystore,
    ) -> Result<WalletInfo, MultiWalletError> {
        let mut guard = self.lock_state()?;

        if guard
            .wallets
            .values()
            .any(|w| w.public_key == request.public_key)
        {
            return Err(MultiWalletError::WalletExists(request.public_key));
        }

        let wallet_id = format!("wallet_{}", Uuid::new_v4());
        let now = Utc::now();

        if let Some(group_id) = &request.group_id {
            if !guard.groups.contains_key(group_id) {
                return Err(MultiWalletError::GroupNotFound(group_id.clone()));
            }
        }

        let chain_id = request
            .chain_id
            .clone()
            .unwrap_or_else(|| infer_chain_id(&request.network));

        let wallet = WalletInfo {
            id: wallet_id.clone(),
            public_key: request.public_key,
            label: request.label,
            network: request.network,
            chain_id,
            wallet_type: request.wallet_type,
            group_id: request.group_id.clone(),
            created_at: now,
            updated_at: now,
            last_used: None,
            balance: 0.0,
            preferences: WalletPreferences::default(),
            performance: PerformanceMetrics::default(),
        };

        if let Some(group_id) = &request.group_id {
            if let Some(group) = guard.groups.get_mut(group_id) {
                if !group.wallet_ids.contains(&wallet_id) {
                    group.wallet_ids.push(wallet_id.clone());
                }
            }
        }

        if guard.active_wallet_id.is_none() {
            guard.active_wallet_id = Some(wallet_id.clone());
        }

        guard.wallets.insert(wallet_id.clone(), wallet.clone());
        guard.last_updated = now;

        self.persist_locked(&guard, keystore)?;

        Ok(wallet)
    }

    pub fn update_wallet(
        &self,
        request: UpdateWalletRequest,
        keystore: &Keystore,
    ) -> Result<WalletInfo, MultiWalletError> {
        let mut guard = self.lock_state()?;

        // Check wallet exists
        if !guard.wallets.contains_key(&request.wallet_id) {
            return Err(MultiWalletError::WalletNotFound(request.wallet_id.clone()));
        }

        // Get wallet ID for later operations
        let wallet_id = request.wallet_id.clone();

        // Update basic fields
        {
            let wallet = guard.wallets.get_mut(&wallet_id).unwrap();

            if let Some(label) = request.label {
                wallet.label = label;
            }

            if let Some(chain_id) = request.chain_id {
                wallet.chain_id = chain_id;
            }

            if let Some(preferences) = request.preferences {
                wallet.preferences = preferences;
            }
        }

        // Handle group changes (needs access to both wallets and groups)
        if let Some(group_id_update) = request.group_id {
            match group_id_update {
                Some(group_id) => {
                    if !guard.groups.contains_key(&group_id) {
                        return Err(MultiWalletError::GroupNotFound(group_id));
                    }

                    // Update wallet's group_id
                    if let Some(wallet) = guard.wallets.get_mut(&wallet_id) {
                        wallet.group_id = Some(group_id.clone());
                    }

                    // Update all groups
                    for group in guard.groups.values_mut() {
                        if group.id == group_id {
                            if !group.wallet_ids.contains(&wallet_id) {
                                group.wallet_ids.push(wallet_id.clone());
                            }
                        } else {
                            group.wallet_ids.retain(|id| id != &wallet_id);
                        }
                    }
                }
                None => {
                    // Remove from all groups
                    for group in guard.groups.values_mut() {
                        group.wallet_ids.retain(|id| id != &wallet_id);
                    }

                    // Update wallet's group_id
                    if let Some(wallet) = guard.wallets.get_mut(&wallet_id) {
                        wallet.group_id = None;
                    }
                }
            }
        }

        // Update timestamps
        if let Some(wallet) = guard.wallets.get_mut(&wallet_id) {
            wallet.updated_at = Utc::now();
        }
        guard.last_updated = Utc::now();

        // Clone result before persist
        let updated_wallet = guard.wallets.get(&wallet_id).unwrap().clone();
        self.persist_locked(&guard, keystore)?;

        Ok(updated_wallet)
    }

    pub fn remove_wallet(
        &self,
        wallet_id: &str,
        keystore: &Keystore,
    ) -> Result<(), MultiWalletError> {
        let mut guard = self.lock_state()?;

        guard
            .wallets
            .remove(wallet_id)
            .ok_or_else(|| MultiWalletError::WalletNotFound(wallet_id.to_string()))?;

        if guard.active_wallet_id.as_deref() == Some(wallet_id) {
            guard.active_wallet_id = guard.wallets.keys().next().cloned();
        }

        for group in guard.groups.values_mut() {
            group.wallet_ids.retain(|id| id != wallet_id);
        }

        guard.last_updated = Utc::now();
        self.persist_locked(&guard, keystore)?;

        Ok(())
    }

    pub fn set_active_wallet(
        &self,
        wallet_id: &str,
        keystore: &Keystore,
    ) -> Result<WalletInfo, MultiWalletError> {
        let mut guard = self.lock_state()?;

        // Check wallet exists and update its fields
        {
            let wallet = guard
                .wallets
                .get_mut(wallet_id)
                .ok_or_else(|| MultiWalletError::WalletNotFound(wallet_id.to_string()))?;

            wallet.last_used = Some(Utc::now());
            wallet.updated_at = Utc::now();
        }

        // Update guard fields after wallet borrow is dropped
        guard.active_wallet_id = Some(wallet_id.to_string());
        guard.last_updated = Utc::now();

        // Clone result
        let active_wallet = guard.wallets.get(wallet_id).unwrap().clone();
        self.persist_locked(&guard, keystore)?;

        Ok(active_wallet)
    }

    pub fn get_active_wallet(&self) -> Result<Option<WalletInfo>, MultiWalletError> {
        let guard = self.lock_state()?;
        Ok(guard
            .active_wallet_id
            .as_ref()
            .and_then(|id| guard.wallets.get(id).cloned()))
    }

    pub fn list_wallets(&self) -> Result<Vec<WalletInfo>, MultiWalletError> {
        let guard = self.lock_state()?;
        let mut wallets: Vec<WalletInfo> = guard.wallets.values().cloned().collect();
        wallets.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        Ok(wallets)
    }

    pub fn update_wallet_balance(
        &self,
        wallet_id: &str,
        balance: f64,
        keystore: &Keystore,
    ) -> Result<(), MultiWalletError> {
        let mut guard = self.lock_state()?;

        let wallet = guard
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| MultiWalletError::WalletNotFound(wallet_id.to_string()))?;

        wallet.balance = balance;
        wallet.updated_at = Utc::now();
        guard.last_updated = Utc::now();

        self.persist_locked(&guard, keystore)?;

        Ok(())
    }

    pub fn update_performance_metrics(
        &self,
        wallet_id: &str,
        metrics: PerformanceMetrics,
        keystore: &Keystore,
    ) -> Result<(), MultiWalletError> {
        let mut guard = self.lock_state()?;

        let wallet = guard
            .wallets
            .get_mut(wallet_id)
            .ok_or_else(|| MultiWalletError::WalletNotFound(wallet_id.to_string()))?;

        wallet.performance = metrics;
        wallet.updated_at = Utc::now();
        guard.last_updated = Utc::now();

        self.persist_locked(&guard, keystore)?;

        Ok(())
    }

    pub fn create_group(
        &self,
        request: CreateGroupRequest,
        keystore: &Keystore,
    ) -> Result<WalletGroup, MultiWalletError> {
        let mut guard = self.lock_state()?;

        let group_id = format!("group_{}", Uuid::new_v4());
        let now = Utc::now();

        for wallet_id in &request.wallet_ids {
            if !guard.wallets.contains_key(wallet_id) {
                return Err(MultiWalletError::WalletNotFound(wallet_id.clone()));
            }
        }

        let group = WalletGroup {
            id: group_id.clone(),
            name: request.name,
            description: request.description,
            created_at: now,
            wallet_ids: request.wallet_ids.clone(),
            shared_settings: request.shared_settings.unwrap_or_default(),
        };

        for wallet_id in &group.wallet_ids {
            if let Some(wallet) = guard.wallets.get_mut(wallet_id) {
                wallet.group_id = Some(group_id.clone());
                wallet.updated_at = now;
            }
        }

        guard.groups.insert(group_id.clone(), group.clone());
        guard.last_updated = now;

        self.persist_locked(&guard, keystore)?;

        Ok(group)
    }

    pub fn update_group(
        &self,
        request: UpdateGroupRequest,
        keystore: &Keystore,
    ) -> Result<WalletGroup, MultiWalletError> {
        let mut guard = self.lock_state()?;

        // Check group exists
        if !guard.groups.contains_key(&request.group_id) {
            return Err(MultiWalletError::GroupNotFound(request.group_id.clone()));
        }

        let group_id = request.group_id.clone();

        // Get the current group ID before any mutations
        let current_group_id = guard.groups.get(&group_id).map(|g| g.id.clone()).unwrap();

        // Update basic fields
        {
            let group = guard.groups.get_mut(&group_id).unwrap();

            if let Some(name) = request.name {
                group.name = name;
            }

            if let Some(description) = request.description {
                group.description = description;
            }

            if let Some(shared_settings) = request.shared_settings {
                group.shared_settings = shared_settings;
            }
        }

        // Handle wallet_ids changes (needs access to both wallets and groups)
        if let Some(wallet_ids) = &request.wallet_ids {
            // Validate all wallet IDs exist
            for wallet_id in wallet_ids {
                if !guard.wallets.contains_key(wallet_id) {
                    return Err(MultiWalletError::WalletNotFound(wallet_id.clone()));
                }
            }

            // Clear group_id from wallets that were in this group
            for wallet in guard.wallets.values_mut() {
                if wallet.group_id.as_deref() == Some(&current_group_id) {
                    wallet.group_id = None;
                }
            }

            // Update group's wallet_ids
            if let Some(group) = guard.groups.get_mut(&group_id) {
                group.wallet_ids = wallet_ids.clone();
            }

            // Set group_id on new wallets
            for wallet_id in wallet_ids {
                if let Some(wallet) = guard.wallets.get_mut(wallet_id) {
                    wallet.group_id = Some(current_group_id.clone());
                }
            }
        }

        // Update timestamp
        guard.last_updated = Utc::now();

        // Clone result
        let updated_group = guard.groups.get(&group_id).unwrap().clone();

        self.persist_locked(&guard, keystore)?;

        Ok(updated_group)
    }

    pub fn delete_group(
        &self,
        group_id: &str,
        keystore: &Keystore,
    ) -> Result<(), MultiWalletError> {
        let mut guard = self.lock_state()?;

        let group = guard
            .groups
            .remove(group_id)
            .ok_or_else(|| MultiWalletError::GroupNotFound(group_id.to_string()))?;

        for wallet_id in group.wallet_ids {
            if let Some(wallet) = guard.wallets.get_mut(&wallet_id) {
                wallet.group_id = None;
                wallet.updated_at = Utc::now();
            }
        }

        guard.last_updated = Utc::now();
        self.persist_locked(&guard, keystore)?;

        Ok(())
    }

    pub fn list_groups(&self) -> Result<Vec<WalletGroup>, MultiWalletError> {
        let guard = self.lock_state()?;
        Ok(guard.groups.values().cloned().collect())
    }

    pub fn get_aggregated_portfolio(&self) -> Result<AggregatedPortfolio, MultiWalletError> {
        let guard = self.lock_state()?;

        let wallets: Vec<WalletInfo> = guard.wallets.values().cloned().collect();

        let total_balance = wallets.iter().map(|w| w.balance).sum();
        let total_trades = wallets.iter().map(|w| w.performance.total_trades).sum();
        let total_volume = wallets.iter().map(|w| w.performance.total_volume).sum();
        let total_realized_pnl = wallets.iter().map(|w| w.performance.realized_pnl).sum();
        let total_unrealized_pnl = wallets.iter().map(|w| w.performance.unrealized_pnl).sum();

        Ok(AggregatedPortfolio {
            total_balance,
            total_wallets: wallets.len(),
            total_groups: guard.groups.len(),
            total_trades,
            total_volume,
            total_realized_pnl,
            total_unrealized_pnl,
            wallets,
        })
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, MultiWalletState>, MultiWalletError> {
        self.state.lock().map_err(|_| MultiWalletError::Internal)
    }

    fn persist_locked(
        &self,
        state: &MultiWalletState,
        keystore: &Keystore,
    ) -> Result<(), MultiWalletError> {
        let serialized = serde_json::to_vec(state)?;
        keystore
            .store_secret(KEYSTORE_STATE_KEY, &serialized)
            .map_err(MultiWalletError::Keystore)
    }
}

#[tauri::command]
pub async fn multi_wallet_add(
    request: AddWalletRequest,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<WalletInfo, String> {
    manager
        .add_wallet(request, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_update(
    request: UpdateWalletRequest,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<WalletInfo, String> {
    manager
        .update_wallet(request, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_remove(
    wallet_id: String,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    manager
        .remove_wallet(&wallet_id, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_set_active(
    wallet_id: String,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<WalletInfo, String> {
    manager
        .set_active_wallet(&wallet_id, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_get_active(
    manager: State<'_, MultiWalletManager>,
) -> Result<Option<WalletInfo>, String> {
    manager.get_active_wallet().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_list(
    manager: State<'_, MultiWalletManager>,
) -> Result<Vec<WalletInfo>, String> {
    manager.list_wallets().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_update_balance(
    wallet_id: String,
    balance: f64,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    manager
        .update_wallet_balance(&wallet_id, balance, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_update_performance(
    wallet_id: String,
    metrics: PerformanceMetrics,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    manager
        .update_performance_metrics(&wallet_id, metrics, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_create_group(
    request: CreateGroupRequest,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<WalletGroup, String> {
    manager
        .create_group(request, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_update_group(
    request: UpdateGroupRequest,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<WalletGroup, String> {
    manager
        .update_group(request, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_delete_group(
    group_id: String,
    manager: State<'_, MultiWalletManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    manager
        .delete_group(&group_id, &keystore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_list_groups(
    manager: State<'_, MultiWalletManager>,
) -> Result<Vec<WalletGroup>, String> {
    manager.list_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn multi_wallet_get_aggregated(
    manager: State<'_, MultiWalletManager>,
) -> Result<AggregatedPortfolio, String> {
    manager
        .get_aggregated_portfolio()
        .map_err(|e| e.to_string())
}
