use super::hardware_wallet::DeviceType;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Device communication error: {0}")]
    Communication(String),
    #[error("Invalid response from device: {0}")]
    InvalidResponse(String),
    #[error("User rejected on device")]
    UserRejected,
    #[error("Device disconnected")]
    Disconnected,
    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),
    #[error("Transaction too large")]
    TransactionTooLarge,
    #[error("Firmware outdated: {0}")]
    FirmwareOutdated(String),
    #[error("Solana app not opened")]
    SolanaAppNotOpened,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("WebHID not supported")]
    WebHIDNotSupported,
    #[error("Permission denied")]
    PermissionDenied,
}

impl Serialize for LedgerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerDevice {
    pub device_id: String,
    pub product_name: String,
    pub manufacturer: String,
    pub connected: bool,
    pub firmware_version: Option<String>,
    pub address: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerDeviceInfo {
    pub target_id: String,
    pub se_version: String,
    pub mcu_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLedgerAddressRequest {
    pub device_id: String,
    pub derivation_path: String,
    pub display: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetLedgerAddressResponse {
    pub address: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignLedgerTransactionRequest {
    pub device_id: String,
    pub transaction: String,
    pub derivation_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignLedgerTransactionResponse {
    pub signature: String,
}

#[derive(Default)]
pub struct LedgerState {
    devices: Mutex<Vec<LedgerDevice>>,
    active_device_id: Mutex<Option<String>>,
}

impl LedgerState {
    pub fn new() -> Self {
        Self {
            devices: Mutex::new(Vec::new()),
            active_device_id: Mutex::new(None),
        }
    }
}

fn validate_derivation_path(path: &str) -> Result<(), LedgerError> {
    let path = path.trim();
    if !path.starts_with("m/") && !path.starts_with("M/") {
        return Err(LedgerError::InvalidDerivationPath(
            "Path must start with m/".to_string(),
        ));
    }

    let parts: Vec<&str> = path[2..].split('/').collect();

    if parts.is_empty() {
        return Err(LedgerError::InvalidDerivationPath(
            "Path must have at least one component".to_string(),
        ));
    }

    for part in parts {
        let (num_str, _hardened) = if part.ends_with('\'') || part.ends_with('h') {
            (&part[..part.len() - 1], true)
        } else {
            (part, false)
        };

        num_str.parse::<u32>().map_err(|_| {
            LedgerError::InvalidDerivationPath(format!("Invalid number in path: {}", part))
        })?;
    }

    Ok(())
}

#[tauri::command]
pub async fn ledger_register_device(
    device: LedgerDevice,
    state: State<'_, LedgerState>,
) -> Result<LedgerDevice, LedgerError> {
    let mut devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    if let Some(existing) = devices_guard
        .iter_mut()
        .find(|d| d.device_id == device.device_id)
    {
        existing.connected = device.connected;
        existing.firmware_version = device.firmware_version.clone();
        existing.address = device.address.clone();
        existing.public_key = device.public_key.clone();
        return Ok(existing.clone());
    }

    devices_guard.push(device.clone());

    let mut active_id = state
        .active_device_id
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock active device ID: {}", e)))?;

    if active_id.is_none() && device.connected {
        *active_id = Some(device.device_id.clone());
    }

    Ok(device)
}

#[tauri::command]
pub async fn ledger_list_devices(
    state: State<'_, LedgerState>,
) -> Result<Vec<LedgerDevice>, LedgerError> {
    let devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    Ok(devices_guard.clone())
}

#[tauri::command]
pub async fn ledger_get_device(
    device_id: String,
    state: State<'_, LedgerState>,
) -> Result<LedgerDevice, LedgerError> {
    let devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    devices_guard
        .iter()
        .find(|d| d.device_id == device_id)
        .cloned()
        .ok_or(LedgerError::DeviceNotFound)
}

#[tauri::command]
pub async fn ledger_connect_device(
    device_id: String,
    state: State<'_, LedgerState>,
) -> Result<LedgerDevice, LedgerError> {
    let mut devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    let device = devices_guard
        .iter_mut()
        .find(|d| d.device_id == device_id)
        .ok_or(LedgerError::DeviceNotFound)?;

    device.connected = true;

    let mut active_id = state
        .active_device_id
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock active device ID: {}", e)))?;
    *active_id = Some(device_id);

    Ok(device.clone())
}

#[tauri::command]
pub async fn ledger_disconnect_device(
    device_id: String,
    state: State<'_, LedgerState>,
) -> Result<(), LedgerError> {
    let mut devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    let device = devices_guard
        .iter_mut()
        .find(|d| d.device_id == device_id)
        .ok_or(LedgerError::DeviceNotFound)?;

    device.connected = false;
    device.address = None;
    device.public_key = None;

    let mut active_id = state
        .active_device_id
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock active device ID: {}", e)))?;

    if *active_id == Some(device_id.clone()) {
        *active_id = None;
    }

    Ok(())
}

#[tauri::command]
pub async fn ledger_update_device_address(
    device_id: String,
    address: String,
    public_key: String,
    state: State<'_, LedgerState>,
) -> Result<(), LedgerError> {
    let mut devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    let device = devices_guard
        .iter_mut()
        .find(|d| d.device_id == device_id)
        .ok_or(LedgerError::DeviceNotFound)?;

    device.address = Some(address);
    device.public_key = Some(public_key);

    Ok(())
}

#[tauri::command]
pub async fn ledger_validate_transaction(
    request: SignLedgerTransactionRequest,
    state: State<'_, LedgerState>,
) -> Result<(), LedgerError> {
    let devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    let device = devices_guard
        .iter()
        .find(|d| d.device_id == request.device_id)
        .ok_or(LedgerError::DeviceNotFound)?;

    if !device.connected {
        return Err(LedgerError::Disconnected);
    }

    let default_path = "m/44'/501'/0'/0'".to_string();
    let derivation_path = request.derivation_path.as_ref().unwrap_or(&default_path);
    validate_derivation_path(derivation_path)?;

    let transaction_bytes = BASE64_ENGINE
        .decode(request.transaction.as_bytes())
        .map_err(|e| LedgerError::Internal(format!("Invalid transaction encoding: {}", e)))?;

    if transaction_bytes.len() > 65535 {
        return Err(LedgerError::TransactionTooLarge);
    }

    Ok(())
}

#[tauri::command]
pub async fn ledger_get_active_device(
    state: State<'_, LedgerState>,
) -> Result<Option<LedgerDevice>, LedgerError> {
    let active_id_guard = state
        .active_device_id
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock active device ID: {}", e)))?;

    if let Some(device_id) = active_id_guard.as_ref() {
        let devices_guard = state
            .devices
            .lock()
            .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

        Ok(devices_guard
            .iter()
            .find(|d| d.device_id == *device_id)
            .cloned())
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn ledger_remove_device(
    device_id: String,
    state: State<'_, LedgerState>,
) -> Result<(), LedgerError> {
    let mut devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    let index = devices_guard
        .iter()
        .position(|d| d.device_id == device_id)
        .ok_or(LedgerError::DeviceNotFound)?;

    devices_guard.remove(index);

    let mut active_id = state
        .active_device_id
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock active device ID: {}", e)))?;

    if *active_id == Some(device_id) {
        *active_id = None;
    }

    Ok(())
}

#[tauri::command]
pub async fn ledger_clear_devices(state: State<'_, LedgerState>) -> Result<(), LedgerError> {
    let mut devices_guard = state
        .devices
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock devices: {}", e)))?;

    devices_guard.clear();

    let mut active_id = state
        .active_device_id
        .lock()
        .map_err(|e| LedgerError::Internal(format!("Failed to lock active device ID: {}", e)))?;
    *active_id = None;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_derivation_path() {
        assert!(validate_derivation_path("m/44'/501'/0'/0'").is_ok());
        assert!(validate_derivation_path("m/44'/501'/0'/0'/0'").is_ok());
        assert!(validate_derivation_path("m/44'/501'/0'").is_ok());
        assert!(validate_derivation_path("M/44'/501'/0'/0'").is_ok());

        assert!(validate_derivation_path("44'/501'/0'/0'").is_err());
        assert!(validate_derivation_path("m/").is_err());
        assert!(validate_derivation_path("m/abc").is_err());
        assert!(validate_derivation_path("invalid").is_err());
    }

    #[tokio::test]
    async fn test_ledger_state_register_device() {
        let state = tauri::State::from(LedgerState::new());

        let device = LedgerDevice {
            device_id: "test-device-1".to_string(),
            product_name: "Ledger Nano S Plus".to_string(),
            manufacturer: "Ledger".to_string(),
            connected: true,
            firmware_version: Some("2.1.0".to_string()),
            address: None,
            public_key: None,
        };

        let result = ledger_register_device(device.clone(), state.clone()).await;
        assert!(result.is_ok());

        let devices = ledger_list_devices(state.clone()).await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, "test-device-1");
    }

    #[tokio::test]
    async fn test_ledger_connect_disconnect() {
        let state = tauri::State::from(LedgerState::new());

        let device = LedgerDevice {
            device_id: "test-device-1".to_string(),
            product_name: "Ledger Nano S Plus".to_string(),
            manufacturer: "Ledger".to_string(),
            connected: false,
            firmware_version: Some("2.1.0".to_string()),
            address: None,
            public_key: None,
        };

        ledger_register_device(device, state.clone()).await.unwrap();

        let connected = ledger_connect_device("test-device-1".to_string(), state.clone())
            .await
            .unwrap();
        assert!(connected.connected);

        let active = ledger_get_active_device(state.clone()).await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().device_id, "test-device-1");

        ledger_disconnect_device("test-device-1".to_string(), state.clone())
            .await
            .unwrap();

        let device = ledger_get_device("test-device-1".to_string(), state)
            .await
            .unwrap();
        assert!(!device.connected);
    }
}
