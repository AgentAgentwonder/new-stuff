use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::State;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum HardwareWalletError {
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
    #[error("Unsupported device type")]
    UnsupportedDevice,
    #[error("Invalid derivation path")]
    InvalidDerivationPath,
    #[error("Transaction too large")]
    TransactionTooLarge,
    #[error("Firmware outdated: {0}")]
    FirmwareOutdated(String),
    #[error("Wrong app opened on device")]
    WrongApp,
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Serialize for HardwareWalletError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Ledger,
    Trezor,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Ledger => write!(f, "Ledger"),
            DeviceType::Trezor => write!(f, "Trezor"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareWalletDevice {
    pub device_id: String,
    pub device_type: DeviceType,
    pub product_name: String,
    pub manufacturer: String,
    pub connected: bool,
    pub firmware_version: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl std::fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignTransactionRequest {
    pub device_id: String,
    pub transaction: String,
    pub derivation_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignTransactionResponse {
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAddressRequest {
    pub device_id: String,
    pub derivation_path: String,
    pub display: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAddressResponse {
    pub address: String,
    pub public_key: String,
}

#[derive(Default)]
pub struct HardwareWalletState {
    devices: Mutex<Vec<HardwareWalletDevice>>,
    simulated: Mutex<bool>,
}

impl HardwareWalletState {
    pub fn new() -> Self {
        Self {
            devices: Mutex::new(Vec::new()),
            simulated: Mutex::new(true),
        }
    }
}

fn parse_derivation_path(path: &str) -> Result<Vec<u32>, HardwareWalletError> {
    let path = path.trim();
    if !path.starts_with("m/") && !path.starts_with("M/") {
        return Err(HardwareWalletError::InvalidDerivationPath);
    }

    let parts: Result<Vec<u32>, _> = path[2..]
        .split('/')
        .map(|part| {
            let (num_str, hardened) = if part.ends_with('\'') || part.ends_with('h') {
                (&part[..part.len() - 1], true)
            } else {
                (part, false)
            };

            let num: u32 = num_str
                .parse()
                .map_err(|_| HardwareWalletError::InvalidDerivationPath)?;

            if hardened {
                Ok(num | 0x8000_0000)
            } else {
                Ok(num)
            }
        })
        .collect();

    parts
}

fn create_simulated_devices() -> Vec<HardwareWalletDevice> {
    vec![
        HardwareWalletDevice {
            device_id: "ledger-sim-001".to_string(),
            device_type: DeviceType::Ledger,
            product_name: "Ledger Nano S Plus (Simulated)".to_string(),
            manufacturer: "Ledger".to_string(),
            connected: false,
            firmware_version: Some("1.0.4".to_string()),
            address: None,
        },
        HardwareWalletDevice {
            device_id: "trezor-sim-001".to_string(),
            device_type: DeviceType::Trezor,
            product_name: "Trezor Model T (Simulated)".to_string(),
            manufacturer: "SatoshiLabs".to_string(),
            connected: false,
            firmware_version: Some("2.5.3".to_string()),
            address: None,
        },
    ]
}

fn generate_deterministic_address(device_id: &str, derivation_path: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(derivation_path.as_bytes());
    let hash = hasher.finalize();

    let truncated: &[u8] = &hash[..32];
    bs58::encode(truncated).into_string()
}

fn generate_deterministic_signature(device_id: &str, transaction: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(device_id.as_bytes());
    hasher.update(transaction.as_bytes());
    hasher.update(b"signature");
    let hash = hasher.finalize();

    let signature: &[u8] = &hash[..];
    bs58::encode(signature).into_string()
}

#[tauri::command]
pub async fn list_hardware_wallets(
    state: State<'_, HardwareWalletState>,
) -> Result<Vec<HardwareWalletDevice>, HardwareWalletError> {
    let is_simulated = *state.simulated.lock().await;

    if is_simulated {
        let devices = create_simulated_devices();
        let mut devices_guard = state.devices.lock().await;
        *devices_guard = devices.clone();
        return Ok(devices);
    }

    Ok(Vec::new())
}

#[tauri::command]
pub async fn connect_hardware_wallet(
    device_id: String,
    state: State<'_, HardwareWalletState>,
) -> Result<HardwareWalletDevice, HardwareWalletError> {
    let mut devices_guard = state.devices.lock().await;

    if let Some(device) = devices_guard.iter_mut().find(|d| d.device_id == device_id) {
        device.connected = true;
        return Ok(device.clone());
    }

    Err(HardwareWalletError::DeviceNotFound)
}

#[tauri::command]
pub async fn disconnect_hardware_wallet(
    device_id: String,
    state: State<'_, HardwareWalletState>,
) -> Result<(), HardwareWalletError> {
    let mut devices_guard = state.devices.lock().await;

    if let Some(device) = devices_guard.iter_mut().find(|d| d.device_id == device_id) {
        device.connected = false;
        return Ok(());
    }

    Err(HardwareWalletError::DeviceNotFound)
}

#[tauri::command]
pub async fn get_hardware_wallet_address(
    request: GetAddressRequest,
    state: State<'_, HardwareWalletState>,
) -> Result<GetAddressResponse, HardwareWalletError> {
    let devices_guard = state.devices.lock().await;

    let device = devices_guard
        .iter()
        .find(|d| d.device_id == request.device_id)
        .ok_or(HardwareWalletError::DeviceNotFound)?;

    if !device.connected {
        return Err(HardwareWalletError::Disconnected);
    }

    parse_derivation_path(&request.derivation_path)?;

    let address = generate_deterministic_address(&request.device_id, &request.derivation_path);
    let public_key = generate_deterministic_address(
        &format!("{}-pubkey", request.device_id),
        &request.derivation_path,
    );

    Ok(GetAddressResponse {
        address,
        public_key,
    })
}

#[tauri::command]
pub async fn sign_with_hardware_wallet(
    request: SignTransactionRequest,
    state: State<'_, HardwareWalletState>,
) -> Result<SignTransactionResponse, HardwareWalletError> {
    let device_id = {
        let devices_guard = state.devices.lock().await;

        let device = devices_guard
            .iter()
            .find(|d| d.device_id == request.device_id)
            .ok_or(HardwareWalletError::DeviceNotFound)?;

        if !device.connected {
            return Err(HardwareWalletError::Disconnected);
        }

        device.device_id.clone()
    };

    let default_path = "m/44'/501'/0'/0'".to_string();
    let derivation_path = request.derivation_path.as_ref().unwrap_or(&default_path);
    parse_derivation_path(derivation_path)?;

    let _transaction_bytes = BASE64_ENGINE
        .decode(request.transaction.as_bytes())
        .map_err(|e| {
            HardwareWalletError::Internal(format!("Invalid transaction encoding: {}", e))
        })?;

    if _transaction_bytes.len() > 65535 {
        return Err(HardwareWalletError::TransactionTooLarge);
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    let signature = generate_deterministic_signature(&device_id, &request.transaction);

    Ok(SignTransactionResponse { signature })
}

#[tauri::command]
pub async fn get_firmware_version(
    device_id: String,
    state: State<'_, HardwareWalletState>,
) -> Result<FirmwareVersion, HardwareWalletError> {
    let devices_guard = state.devices.lock().await;

    let device = devices_guard
        .iter()
        .find(|d| d.device_id == device_id)
        .ok_or(HardwareWalletError::DeviceNotFound)?;

    if !device.connected {
        return Err(HardwareWalletError::Disconnected);
    }

    let version_str = device
        .firmware_version
        .as_ref()
        .ok_or_else(|| HardwareWalletError::Internal("No firmware version".to_string()))?;

    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() != 3 {
        return Err(HardwareWalletError::InvalidResponse(
            "Invalid version format".to_string(),
        ));
    }

    Ok(FirmwareVersion {
        major: parts[0].parse().map_err(|_| {
            HardwareWalletError::InvalidResponse("Invalid major version".to_string())
        })?,
        minor: parts[1].parse().map_err(|_| {
            HardwareWalletError::InvalidResponse("Invalid minor version".to_string())
        })?,
        patch: parts[2].parse().map_err(|_| {
            HardwareWalletError::InvalidResponse("Invalid patch version".to_string())
        })?,
    })
}
