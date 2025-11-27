use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

use crate::security::keystore::{Keystore, KeystoreError};

const KEY_HELIUS_API: &str = "api_key_helius";
const KEY_BIRDEYE_API: &str = "api_key_birdeye";
const KEY_JUPITER_API: &str = "api_key_jupiter";
const KEY_SOLANA_RPC: &str = "api_rpc_endpoint";
const KEY_API_METADATA: &str = "api_key_metadata";

// Stock API keys
const KEY_ALPHA_VANTAGE_API: &str = "api_key_alpha_vantage";
const KEY_POLYGON_API: &str = "api_key_polygon";
const KEY_IEX_API: &str = "api_key_iex";
const KEY_FINNHUB_API: &str = "api_key_finnhub";

// Default RPC endpoint
const DEFAULT_RPC_ENDPOINT: &str = "https://api.mainnet-beta.solana.com";

// Developer fallback keys (masked for security)
const DEFAULT_HELIUS_KEY: &str = "YOUR_HELIUS_KEY_HERE";
const DEFAULT_BIRDEYE_KEY: &str = "YOUR_BIRDEYE_KEY_HERE";
const DEFAULT_JUPITER_KEY: &str = "YOUR_JUPITER_KEY_HERE";

const ROTATION_INTERVAL_DAYS: i64 = 90;
const ROTATION_REMINDER_THRESHOLD_DAYS: i64 = 15;
const ROTATION_HISTORY_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyConfig {
    pub helius: Option<String>,
    pub birdeye: Option<String>,
    pub jupiter: Option<String>,
    pub solana_rpc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyMetadata {
    pub service: String,
    pub expiry_date: Option<DateTime<Utc>>,
    pub last_rotation: DateTime<Utc>,
    pub use_default: bool,
    pub connection_status: ConnectionStatus,
    pub last_tested: Option<DateTime<Utc>>,
    pub rate_limit_info: Option<RateLimitInfo>,
    #[serde(default)]
    pub rotation_history: Vec<RotationRecord>,
    #[serde(default)]
    pub rotation_due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reminder_sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RotationRecord {
    pub timestamp: DateTime<Utc>,
    pub reason: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub connected: bool,
    pub last_error: Option<String>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiStatus {
    pub helius: ServiceStatus,
    pub birdeye: ServiceStatus,
    pub jupiter: ServiceStatus,
    pub solana_rpc: ServiceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatus {
    pub configured: bool,
    pub using_default: bool,
    pub connection_status: ConnectionStatus,
    pub rate_limit_info: Option<RateLimitInfo>,
    pub last_tested: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub days_until_expiry: Option<i64>,
    pub last_rotation: Option<DateTime<Utc>>,
    pub rotation_due_at: Option<DateTime<Utc>>,
    pub days_until_rotation_due: Option<i64>,
    pub rotation_overdue: bool,
    pub rotation_history: Vec<RotationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfigUpdate {
    pub service: String,
    pub api_key: Option<String>,
    pub expiry_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub service: String,
    pub success: bool,
    pub status_code: Option<u16>,
    pub error: Option<String>,
    pub latency_ms: Option<u64>,
    pub rate_limit_info: Option<RateLimitInfo>,
}

pub struct ApiConfigManager {
    metadata: Arc<Mutex<HashMap<String, ApiKeyMetadata>>>,
}

fn default_metadata(service: &str, use_default: bool) -> ApiKeyMetadata {
    let now = Utc::now();
    ApiKeyMetadata {
        service: service.to_string(),
        expiry_date: None,
        last_rotation: now,
        use_default,
        connection_status: ConnectionStatus {
            connected: false,
            last_error: None,
            status_code: None,
        },
        last_tested: None,
        rate_limit_info: None,
        rotation_history: Vec::new(),
        rotation_due_at: Some(now + Duration::days(ROTATION_INTERVAL_DAYS)),
        reminder_sent_at: None,
    }
}

impl ApiConfigManager {
    pub fn new() -> Self {
        Self {
            metadata: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn initialize(&self, keystore: &Keystore) -> Result<(), KeystoreError> {
        // Load metadata from keystore
        if let Ok(data) = keystore.retrieve_secret(KEY_API_METADATA) {
            if let Ok(metadata_map) =
                serde_json::from_slice::<HashMap<String, ApiKeyMetadata>>(&data)
            {
                if let Ok(mut meta) = self.metadata.lock() {
                    *meta = metadata_map;
                }
            }
        }
        Ok(())
    }

    fn save_metadata(&self, keystore: &Keystore) -> Result<(), KeystoreError> {
        if let Ok(meta) = self.metadata.lock() {
            let serialized = serde_json::to_vec(&*meta).map_err(|_| KeystoreError::Internal)?;
            keystore.store_secret(KEY_API_METADATA, &serialized)?;
        }
        Ok(())
    }

    pub fn get_metadata(&self, service: &str) -> Option<ApiKeyMetadata> {
        self.metadata.lock().ok()?.get(service).cloned()
    }

    pub fn update_metadata(
        &self,
        service: &str,
        metadata: ApiKeyMetadata,
        keystore: &Keystore,
    ) -> Result<(), KeystoreError> {
        if let Ok(mut meta) = self.metadata.lock() {
            meta.insert(service.to_string(), metadata);
            drop(meta);
            self.save_metadata(keystore)?;
        }
        Ok(())
    }

    pub fn get_or_create_metadata(&self, service: &str, use_default: bool) -> ApiKeyMetadata {
        self.get_metadata(service)
            .unwrap_or_else(|| default_metadata(service, use_default))
    }
}

impl Default for ApiConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn save_api_key(
    service: String,
    api_key: String,
    expiry_date: Option<DateTime<Utc>>,
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<String, String> {
    let key_id = match service.as_str() {
        "helius" => KEY_HELIUS_API,
        "birdeye" => KEY_BIRDEYE_API,
        "jupiter" => KEY_JUPITER_API,
        "solana_rpc" => KEY_SOLANA_RPC,
        _ => return Err("Unknown service".to_string()),
    };

    // Store the API key securely
    keystore
        .store_secret(key_id, api_key.as_bytes())
        .map_err(|e| format!("Failed to store API key: {}", e))?;

    // Update metadata
    let mut metadata = config_manager.get_or_create_metadata(&service, false);
    let now = Utc::now();
    metadata.expiry_date = expiry_date;
    metadata.last_rotation = now;
    metadata.use_default = false;
    metadata.connection_status = ConnectionStatus {
        connected: false,
        last_error: None,
        status_code: None,
    };
    metadata.last_tested = None;
    metadata.rotation_due_at = Some(now + Duration::days(ROTATION_INTERVAL_DAYS));
    metadata.reminder_sent_at = None;

    // Add rotation record
    metadata.rotation_history.push(RotationRecord {
        timestamp: now,
        reason: "Manual key update".to_string(),
        success: true,
    });

    // Keep only last N rotation records
    if metadata.rotation_history.len() > ROTATION_HISTORY_LIMIT {
        metadata
            .rotation_history
            .drain(0..(metadata.rotation_history.len() - ROTATION_HISTORY_LIMIT));
    }

    config_manager
        .update_metadata(&service, metadata, &keystore)
        .map_err(|e| format!("Failed to update metadata: {}", e))?;

    Ok(format!("API key for {} saved successfully", service))
}

#[tauri::command]
pub async fn remove_api_key(
    service: String,
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<String, String> {
    let key_id = match service.as_str() {
        "helius" => KEY_HELIUS_API,
        "birdeye" => KEY_BIRDEYE_API,
        "jupiter" => KEY_JUPITER_API,
        "solana_rpc" => KEY_SOLANA_RPC,
        _ => return Err("Unknown service".to_string()),
    };

    keystore
        .remove_secret(key_id)
        .map_err(|e| format!("Failed to remove API key: {}", e))?;

    // Update metadata to use default
    let mut metadata = config_manager.get_or_create_metadata(&service, true);
    metadata.use_default = true;
    metadata.connection_status = ConnectionStatus {
        connected: false,
        last_error: None,
        status_code: None,
    };
    metadata.last_tested = Some(Utc::now());
    metadata.rate_limit_info = None;
    config_manager
        .update_metadata(&service, metadata, &keystore)
        .map_err(|e| format!("Failed to update metadata: {}", e))?;

    Ok(format!("API key for {} removed", service))
}

#[tauri::command]
pub async fn set_use_default_key(
    service: String,
    use_default: bool,
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<String, String> {
    let mut metadata = config_manager.get_or_create_metadata(&service, use_default);

    metadata.use_default = use_default;
    config_manager
        .update_metadata(&service, metadata, &keystore)
        .map_err(|e| format!("Failed to update metadata: {}", e))?;

    Ok(format!(
        "Service {} now using {} keys",
        service,
        if use_default { "default" } else { "custom" }
    ))
}

#[tauri::command]
pub async fn test_api_connection(
    service: String,
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<ConnectionTestResult, String> {
    let start = std::time::Instant::now();

    // Get the API key (either custom or default)
    let metadata = config_manager.get_metadata(&service);
    let use_default = metadata.as_ref().map(|m| m.use_default).unwrap_or(true);

    let api_key = if use_default {
        get_default_key(&service)
    } else {
        let key_id = match service.as_str() {
            "helius" => KEY_HELIUS_API,
            "birdeye" => KEY_BIRDEYE_API,
            "jupiter" => KEY_JUPITER_API,
            "solana_rpc" => KEY_SOLANA_RPC,
            _ => return Err("Unknown service".to_string()),
        };

        match keystore.retrieve_secret(key_id) {
            Ok(secret) => String::from_utf8(secret.to_vec())
                .map_err(|_| "Invalid API key encoding".to_string())?,
            Err(_) => get_default_key(&service),
        }
    };

    // Test the connection based on service
    let result = match service.as_str() {
        "helius" => test_helius_connection(&api_key).await,
        "birdeye" => test_birdeye_connection(&api_key).await,
        "jupiter" => test_jupiter_connection(&api_key).await,
        "solana_rpc" => test_rpc_connection(&api_key).await,
        _ => return Err("Unknown service".to_string()),
    };

    let latency = start.elapsed().as_millis() as u64;

    let test_result = match result {
        Ok((status_code, rate_limit)) => {
            // Update metadata with successful connection
            let mut meta = config_manager.get_or_create_metadata(&service, use_default);
            meta.use_default = use_default;
            meta.connection_status = ConnectionStatus {
                connected: true,
                last_error: None,
                status_code: Some(status_code),
            };
            meta.last_tested = Some(Utc::now());
            meta.rate_limit_info = rate_limit.clone();
            if let Err(err) = config_manager.update_metadata(&service, meta, &keystore) {
                eprintln!("Failed to persist API metadata: {err}");
            }

            ConnectionTestResult {
                service: service.clone(),
                success: true,
                status_code: Some(status_code),
                error: None,
                latency_ms: Some(latency),
                rate_limit_info: rate_limit,
            }
        }
        Err(error) => {
            // Update metadata with error
            let mut meta = config_manager.get_or_create_metadata(&service, use_default);
            meta.use_default = use_default;
            meta.connection_status = ConnectionStatus {
                connected: false,
                last_error: Some(error.clone()),
                status_code: None,
            };
            meta.last_tested = Some(Utc::now());
            meta.rate_limit_info = None;
            if let Err(err) = config_manager.update_metadata(&service, meta, &keystore) {
                eprintln!("Failed to persist API metadata: {err}");
            }

            ConnectionTestResult {
                service: service.clone(),
                success: false,
                status_code: None,
                error: Some(error),
                latency_ms: Some(latency),
                rate_limit_info: None,
            }
        }
    };

    Ok(test_result)
}

#[tauri::command]
pub async fn get_api_status(
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<ApiStatus, String> {
    let status = ApiStatus {
        helius: get_service_status("helius", &keystore, &config_manager)?,
        birdeye: get_service_status("birdeye", &keystore, &config_manager)?,
        jupiter: get_service_status("jupiter", &keystore, &config_manager)?,
        solana_rpc: get_service_status("solana_rpc", &keystore, &config_manager)?,
    };

    Ok(status)
}

fn get_service_status(
    service: &str,
    keystore: &Keystore,
    config_manager: &ApiConfigManager,
) -> Result<ServiceStatus, String> {
    let key_id = match service {
        "helius" => KEY_HELIUS_API,
        "birdeye" => KEY_BIRDEYE_API,
        "jupiter" => KEY_JUPITER_API,
        "solana_rpc" => KEY_SOLANA_RPC,
        _ => return Err("Unknown service".to_string()),
    };

    let configured = keystore.retrieve_secret(key_id).is_ok();
    let metadata = config_manager.get_metadata(service);

    let using_default = metadata.as_ref().map(|m| m.use_default).unwrap_or(true);
    let connection_status = metadata
        .as_ref()
        .map(|m| m.connection_status.clone())
        .unwrap_or(ConnectionStatus {
            connected: false,
            last_error: None,
            status_code: None,
        });

    let rate_limit_info = metadata.as_ref().and_then(|m| m.rate_limit_info.clone());
    let last_tested = metadata.as_ref().and_then(|m| m.last_tested);
    let expiry_date = metadata.as_ref().and_then(|m| m.expiry_date);

    let days_until_expiry = expiry_date.map(|exp| {
        let now = Utc::now();
        (exp - now).num_days()
    });

    let last_rotation = metadata.as_ref().map(|m| m.last_rotation);
    let rotation_due_at = metadata.as_ref().and_then(|m| m.rotation_due_at);
    let days_until_rotation_due = rotation_due_at.map(|due| {
        let now = Utc::now();
        (due - now).num_days()
    });
    let rotation_overdue = rotation_due_at.map(|due| Utc::now() > due).unwrap_or(false);
    let rotation_history = metadata
        .as_ref()
        .map(|m| m.rotation_history.clone())
        .unwrap_or_default();

    Ok(ServiceStatus {
        configured,
        using_default,
        connection_status,
        rate_limit_info,
        last_tested,
        expiry_date,
        days_until_expiry,
        last_rotation,
        rotation_due_at,
        days_until_rotation_due,
        rotation_overdue,
        rotation_history,
    })
}

fn get_default_key(service: &str) -> String {
    match service {
        "helius" => DEFAULT_HELIUS_KEY.to_string(),
        "birdeye" => DEFAULT_BIRDEYE_KEY.to_string(),
        "jupiter" => DEFAULT_JUPITER_KEY.to_string(),
        "solana_rpc" => DEFAULT_RPC_ENDPOINT.to_string(),
        _ => String::new(),
    }
}

async fn test_helius_connection(api_key: &str) -> Result<(u16, Option<RateLimitInfo>), String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.helius.xyz/v0/addresses/HeM8ZhRrPA8QUcLt7ycTGy8AyD1q2CqfRvEdBZ99jqZv/balances?api-key={}", api_key);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let status = response.status().as_u16();

    let rate_limit = extract_rate_limit_info(&response);

    if status == 200 {
        Ok((status, rate_limit))
    } else {
        Err(format!(
            "HTTP {}: {}",
            status,
            response.text().await.unwrap_or_default()
        ))
    }
}

async fn test_birdeye_connection(api_key: &str) -> Result<(u16, Option<RateLimitInfo>), String> {
    let client = reqwest::Client::new();
    let url = "https://public-api.birdeye.so/public/token_list?sort_by=v24hUSD&sort_type=desc&offset=0&limit=1";

    let response = client
        .get(url)
        .header("X-API-KEY", api_key)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let status = response.status().as_u16();
    let rate_limit = extract_rate_limit_info(&response);

    if status == 200 {
        Ok((status, rate_limit))
    } else {
        Err(format!(
            "HTTP {}: {}",
            status,
            response.text().await.unwrap_or_default()
        ))
    }
}

async fn test_jupiter_connection(api_key: &str) -> Result<(u16, Option<RateLimitInfo>), String> {
    let client = reqwest::Client::new();
    let url = "https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=100000000";

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let status = response.status().as_u16();
    let rate_limit = extract_rate_limit_info(&response);

    if status == 200 {
        Ok((status, rate_limit))
    } else {
        Err(format!(
            "HTTP {}: {}",
            status,
            response.text().await.unwrap_or_default()
        ))
    }
}

async fn test_rpc_connection(endpoint: &str) -> Result<(u16, Option<RateLimitInfo>), String> {
    let client = reqwest::Client::new();

    let request_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getHealth"
    });

    let response = client
        .post(endpoint)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let status = response.status().as_u16();
    let rate_limit = extract_rate_limit_info(&response);

    if status == 200 {
        Ok((status, rate_limit))
    } else {
        Err(format!(
            "HTTP {}: {}",
            status,
            response.text().await.unwrap_or_default()
        ))
    }
}

fn extract_rate_limit_info(response: &reqwest::Response) -> Option<RateLimitInfo> {
    let headers = response.headers();

    let limit = headers
        .get("x-ratelimit-limit")
        .or_else(|| headers.get("ratelimit-limit"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())?;

    let remaining = headers
        .get("x-ratelimit-remaining")
        .or_else(|| headers.get("ratelimit-remaining"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())?;

    let reset = headers
        .get("x-ratelimit-reset")
        .or_else(|| headers.get("ratelimit-reset"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .map(|timestamp| DateTime::from_timestamp(timestamp, 0))
        .flatten()?;

    Some(RateLimitInfo {
        limit,
        remaining,
        reset_at: reset,
    })
}

#[tauri::command]
pub async fn rotate_api_key(
    service: String,
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<String, String> {
    let metadata = config_manager.get_metadata(&service);

    if metadata.is_none() || metadata.as_ref().map(|m| m.use_default).unwrap_or(true) {
        return Err("Cannot rotate default keys. Please add a custom key first.".to_string());
    }

    let mut meta = metadata.unwrap();
    let now = Utc::now();

    meta.last_rotation = now;
    meta.rotation_due_at = Some(now + Duration::days(ROTATION_INTERVAL_DAYS));
    meta.reminder_sent_at = None;

    meta.rotation_history.push(RotationRecord {
        timestamp: now,
        reason: "Manual rotation".to_string(),
        success: true,
    });

    if meta.rotation_history.len() > ROTATION_HISTORY_LIMIT {
        meta.rotation_history
            .drain(0..(meta.rotation_history.len() - ROTATION_HISTORY_LIMIT));
    }

    config_manager
        .update_metadata(&service, meta, &keystore)
        .map_err(|e| format!("Failed to update metadata: {}", e))?;

    Ok(format!(
        "Key rotation scheduled for {}. Next rotation due in 90 days.",
        service
    ))
}

#[tauri::command]
pub async fn check_rotation_reminders(
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<Vec<String>, String> {
    let services = vec!["helius", "birdeye", "jupiter", "solana_rpc"];
    let mut reminders = Vec::new();
    let now = Utc::now();

    for service in services {
        if let Some(mut metadata) = config_manager.get_metadata(service) {
            if let Some(rotation_due) = metadata.rotation_due_at {
                let days_until_rotation = (rotation_due - now).num_days();

                // Send reminder if within threshold and not already sent
                if days_until_rotation <= ROTATION_REMINDER_THRESHOLD_DAYS
                    && metadata.reminder_sent_at.is_none()
                {
                    reminders.push(format!(
                        "{}: Key rotation due in {} days",
                        service, days_until_rotation
                    ));

                    metadata.reminder_sent_at = Some(now);
                    let _ = config_manager.update_metadata(service, metadata, &keystore);
                }
            }
        }
    }

    Ok(reminders)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeysExport {
    pub version: u8,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
    pub created_at: DateTime<Utc>,
}

#[tauri::command]
pub async fn export_api_keys(
    password: String,
    keystore: State<'_, Keystore>,
) -> Result<ApiKeysExport, String> {
    // Export the entire keystore backup which includes API keys
    let backup = keystore
        .export_backup(&password)
        .map_err(|e| format!("Failed to export keys: {}", e))?;

    Ok(ApiKeysExport {
        version: backup.version,
        salt: backup.salt,
        nonce: backup.nonce,
        ciphertext: backup.ciphertext,
        created_at: backup.created_at,
    })
}

#[tauri::command]
pub async fn import_api_keys(
    password: String,
    export_data: ApiKeysExport,
    keystore: State<'_, Keystore>,
    config_manager: State<'_, ApiConfigManager>,
) -> Result<String, String> {
    use crate::security::keystore::KeystoreBackup;

    let backup = KeystoreBackup {
        version: export_data.version,
        salt: export_data.salt,
        nonce: export_data.nonce,
        ciphertext: export_data.ciphertext,
        created_at: export_data.created_at,
    };

    keystore
        .import_backup(&password, backup)
        .map_err(|e| format!("Failed to import keys: {}", e))?;

    // Reload metadata after import
    config_manager
        .initialize(&keystore)
        .map_err(|e| format!("Failed to reload metadata: {}", e))?;

    Ok("API keys imported successfully".to_string())
}

pub fn register_api_config_manager(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let config_manager = ApiConfigManager::new();

    // Initialize with keystore
    if let Some(keystore) = app.try_state::<Keystore>() {
        config_manager.initialize(&keystore)?;
    }

    app.manage(config_manager);
    Ok(())
}
