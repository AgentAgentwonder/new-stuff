use crate::mobile::{MobileDevice, MobileSession};
use crate::security::keystore::Keystore;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricChallenge {
    pub challenge_id: String,
    pub device_id: String,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileAuthRequest {
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
    pub biometric_public_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileAuthResponse {
    pub session_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub device_id: String,
}

pub struct MobileAuthManager {
    devices: HashMap<String, MobileDevice>,
    sessions: HashMap<String, MobileSession>,
    challenges: HashMap<String, BiometricChallenge>,
    data_dir: PathBuf,
}

impl MobileAuthManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            devices: HashMap::new(),
            sessions: HashMap::new(),
            challenges: HashMap::new(),
            data_dir,
        }
    }

    /// Register a new mobile device
    pub async fn register_device(&mut self, req: MobileAuthRequest) -> Result<MobileDevice> {
        let device_id = if req.device_id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            req.device_id
        };

        let device = MobileDevice {
            device_id: device_id.clone(),
            device_name: req.device_name,
            platform: req.platform,
            push_token: None,
            last_sync: None,
            biometric_enabled: req.biometric_public_key.is_some(),
        };

        self.devices.insert(device_id.clone(), device.clone());
        self.save_devices().await?;

        Ok(device)
    }

    /// Create a biometric challenge for authentication
    pub async fn create_biometric_challenge(
        &mut self,
        device_id: String,
    ) -> Result<BiometricChallenge> {
        if !self.devices.contains_key(&device_id) {
            return Err(anyhow!("Device not registered"));
        }

        let challenge_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        let challenge = BiometricChallenge {
            challenge_id: challenge_id.clone(),
            device_id: device_id.clone(),
            created_at: now,
            expires_at: now + 300, // 5 minutes
        };

        self.challenges
            .insert(challenge_id.clone(), challenge.clone());

        Ok(challenge)
    }

    /// Verify biometric authentication
    pub async fn verify_biometric(
        &mut self,
        challenge_id: String,
        signature: String,
    ) -> Result<MobileAuthResponse> {
        let challenge = self
            .challenges
            .get(&challenge_id)
            .ok_or_else(|| anyhow!("Invalid challenge"))?;

        let now = Utc::now().timestamp();
        if now > challenge.expires_at {
            self.challenges.remove(&challenge_id);
            return Err(anyhow!("Challenge expired"));
        }

        let device = self
            .devices
            .get(&challenge.device_id)
            .ok_or_else(|| anyhow!("Device not found"))?;

        if !device.biometric_enabled {
            return Err(anyhow!("Biometric not enabled for device"));
        }

        // In production, verify the signature with the device's public key
        // For now, we'll accept any non-empty signature
        if signature.is_empty() {
            return Err(anyhow!("Invalid signature"));
        }

        self.challenges.remove(&challenge_id);

        // Create session
        let session_token = Uuid::new_v4().to_string();
        let refresh_token = Uuid::new_v4().to_string();
        let expires_at = now + 86400; // 24 hours

        let session = MobileSession {
            session_id: session_token.clone(),
            device_id: device.device_id.clone(),
            user_id: "default_user".to_string(), // In production, link to actual user
            created_at: now,
            expires_at,
            is_active: true,
        };

        self.sessions.insert(session_token.clone(), session);
        self.save_sessions().await?;

        Ok(MobileAuthResponse {
            session_token,
            refresh_token,
            expires_in: 86400,
            device_id: device.device_id.clone(),
        })
    }

    /// Authenticate with session token
    pub async fn authenticate_session(&self, session_token: String) -> Result<MobileSession> {
        let session = self
            .sessions
            .get(&session_token)
            .ok_or_else(|| anyhow!("Invalid session"))?;

        let now = Utc::now().timestamp();
        if now > session.expires_at {
            return Err(anyhow!("Session expired"));
        }

        if !session.is_active {
            return Err(anyhow!("Session inactive"));
        }

        Ok(session.clone())
    }

    /// Revoke a session
    pub async fn revoke_session(&mut self, session_token: String) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(&session_token) {
            session.is_active = false;
        }
        self.save_sessions().await?;
        Ok(())
    }

    /// Update device push token
    pub async fn update_push_token(&mut self, device_id: String, push_token: String) -> Result<()> {
        let device = self
            .devices
            .get_mut(&device_id)
            .ok_or_else(|| anyhow!("Device not found"))?;

        device.push_token = Some(push_token);
        self.save_devices().await?;

        Ok(())
    }

    /// Get all devices for a user
    pub fn get_devices(&self) -> Vec<MobileDevice> {
        self.devices.values().cloned().collect()
    }

    /// Remove a device
    pub async fn remove_device(&mut self, device_id: String) -> Result<()> {
        self.devices.remove(&device_id);

        // Remove all sessions for this device
        self.sessions
            .retain(|_, session| session.device_id != device_id);

        self.save_devices().await?;
        self.save_sessions().await?;

        Ok(())
    }

    async fn save_devices(&self) -> Result<()> {
        let path = self.data_dir.join("mobile_devices.json");
        let json = serde_json::to_string_pretty(&self.devices)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    async fn save_sessions(&self) -> Result<()> {
        let path = self.data_dir.join("mobile_sessions.json");
        let json = serde_json::to_string_pretty(&self.sessions)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load(&mut self) -> Result<()> {
        let devices_path = self.data_dir.join("mobile_devices.json");
        if devices_path.exists() {
            let content = tokio::fs::read_to_string(devices_path).await?;
            self.devices = serde_json::from_str(&content)?;
        }

        let sessions_path = self.data_dir.join("mobile_sessions.json");
        if sessions_path.exists() {
            let content = tokio::fs::read_to_string(sessions_path).await?;
            self.sessions = serde_json::from_str(&content)?;
        }

        Ok(())
    }
}

// Tauri commands
#[tauri::command]
pub async fn mobile_register_device(
    req: MobileAuthRequest,
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<MobileDevice, String> {
    let mut manager = mobile_auth.write().await;
    manager
        .register_device(req)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_create_biometric_challenge(
    device_id: String,
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<BiometricChallenge, String> {
    let mut manager = mobile_auth.write().await;
    manager
        .create_biometric_challenge(device_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_verify_biometric(
    challenge_id: String,
    signature: String,
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<MobileAuthResponse, String> {
    let mut manager = mobile_auth.write().await;
    manager
        .verify_biometric(challenge_id, signature)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_authenticate_session(
    session_token: String,
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<MobileSession, String> {
    let manager = mobile_auth.read().await;
    manager
        .authenticate_session(session_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_revoke_session(
    session_token: String,
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<(), String> {
    let mut manager = mobile_auth.write().await;
    manager
        .revoke_session(session_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_update_push_token(
    device_id: String,
    push_token: String,
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<(), String> {
    let mut manager = mobile_auth.write().await;
    manager
        .update_push_token(device_id, push_token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_get_devices(
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<Vec<MobileDevice>, String> {
    let manager = mobile_auth.read().await;
    Ok(manager.get_devices())
}

#[tauri::command]
pub async fn mobile_remove_device(
    device_id: String,
    mobile_auth: tauri::State<'_, Arc<RwLock<MobileAuthManager>>>,
) -> Result<(), String> {
    let mut manager = mobile_auth.write().await;
    manager
        .remove_device(device_id)
        .await
        .map_err(|e| e.to_string())
}
