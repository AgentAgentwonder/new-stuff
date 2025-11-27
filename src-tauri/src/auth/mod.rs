pub mod biometric;
pub mod session_manager;
pub mod two_factor;

use biometric::BiometricStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthState {
    pub connected: bool,
    pub wallet_address: Option<String>,
}

#[tauri::command]
pub async fn connect_phantom(_app: tauri::AppHandle) -> Result<AuthState, String> {
    Ok(AuthState {
        connected: false,
        wallet_address: None,
    })
}

#[tauri::command]
pub async fn biometric_get_status() -> Result<BiometricStatus, String> {
    biometric::current_status().map_err(|e| e.into())
}

#[tauri::command]
pub async fn biometric_enroll(fallback_password: String) -> Result<BiometricStatus, String> {
    biometric::enroll(fallback_password)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn biometric_verify() -> Result<(), String> {
    biometric::verify().await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn biometric_disable() -> Result<BiometricStatus, String> {
    biometric::disable().map_err(|e| e.into())
}

#[tauri::command]
pub async fn biometric_verify_fallback(password: String) -> Result<(), String> {
    biometric::verify_fallback(password).map_err(|e| e.into())
}
