use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm,
};
use base64::{engine::general_purpose::STANDARD as BASE64_ENGINE, Engine};
use chrono::Utc;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::security::keystore::{Keystore, KeystoreError};

use super::cloud_providers::{
    BackupMetadata, CloudProvider, CloudProviderConfig, CloudProviderError, CloudProviderManager,
};
use super::scheduler::{
    BackupSchedule, BackupScheduler, BackupStatus, SchedulerError, SharedBackupScheduler,
};
use super::settings_manager::{AppSettings, SettingsError, SettingsManager};

const BACKUP_KEY_ID: &str = "backup.encryption_key";
const BACKUP_CONFIG_FILE: &str = "backup_config.enc";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedBackup {
    pub version: u32,
    pub nonce: String,
    pub ciphertext: String,
    pub checksum: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    #[error("keystore state not available")]
    KeystoreUnavailable,
    #[error("settings error: {0}")]
    Settings(#[from] SettingsError),
    #[error("cloud provider error: {0}")]
    CloudProvider(#[from] CloudProviderError),
    #[error("scheduler error: {0}")]
    Scheduler(#[from] SchedulerError),
    #[error("encryption error")]
    Encryption,
    #[error("decryption error")]
    Decryption,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("integrity check failed")]
    IntegrityCheckFailed,
}

pub type SharedBackupService = Arc<RwLock<BackupService>>;

pub struct BackupService {
    app_handle: AppHandle,
    settings_manager: SettingsManager,
    cloud_manager: CloudProviderManager,
}

impl BackupService {
    pub fn new(app: &AppHandle) -> Self {
        Self {
            app_handle: app.clone(),
            settings_manager: SettingsManager::new(app),
            cloud_manager: CloudProviderManager::new(app),
        }
    }

    fn get_or_create_backup_key(&self, keystore: &Keystore) -> Result<Vec<u8>, BackupError> {
        match keystore.retrieve_secret(BACKUP_KEY_ID) {
            Ok(key) => Ok(key.to_vec()),
            Err(KeystoreError::NotFound) => {
                // Generate new key
                let mut key = vec![0u8; 32];
                OsRng.fill_bytes(&mut key);
                keystore.store_secret(BACKUP_KEY_ID, &key)?;
                Ok(key)
            }
            Err(e) => Err(BackupError::Keystore(e)),
        }
    }

    fn encrypt_data(
        &self,
        keystore: &Keystore,
        data: &[u8],
    ) -> Result<EncryptedBackup, BackupError> {
        let key = self.get_or_create_backup_key(keystore)?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));

        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);

        let ciphertext = cipher
            .encrypt(GenericArray::from_slice(&nonce), data)
            .map_err(|_| BackupError::Encryption)?;

        // Calculate checksum of original data
        let mut hasher = Sha256::new();
        hasher.update(data);
        let checksum = format!("{:x}", hasher.finalize());

        Ok(EncryptedBackup {
            version: 1,
            nonce: BASE64_ENGINE.encode(nonce),
            ciphertext: BASE64_ENGINE.encode(ciphertext),
            checksum,
            created_at: Utc::now(),
        })
    }

    fn decrypt_data(
        &self,
        keystore: &Keystore,
        backup: &EncryptedBackup,
    ) -> Result<Vec<u8>, BackupError> {
        let key = self.get_or_create_backup_key(keystore)?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));

        let nonce = BASE64_ENGINE
            .decode(backup.nonce.as_bytes())
            .map_err(|_| BackupError::Decryption)?;
        let ciphertext = BASE64_ENGINE
            .decode(backup.ciphertext.as_bytes())
            .map_err(|_| BackupError::Decryption)?;

        let plaintext = cipher
            .decrypt(GenericArray::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|_| BackupError::Decryption)?;

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&plaintext);
        let checksum = format!("{:x}", hasher.finalize());

        if checksum != backup.checksum {
            return Err(BackupError::IntegrityCheckFailed);
        }

        Ok(plaintext)
    }

    pub fn create_backup_with_provider(
        &self,
        provider: &CloudProvider,
        sections: Option<Vec<String>>,
    ) -> Result<BackupMetadata, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;

        // Export settings
        let settings = self.settings_manager.export_settings(sections)?;

        // Serialize to JSON
        let json = serde_json::to_vec(&settings)?;

        // Encrypt
        let encrypted = self.encrypt_data(&*keystore, &json)?;

        // Serialize encrypted backup
        let backup_data = serde_json::to_vec(&encrypted)?;

        // Create metadata
        let filename = format!("backup_{}.enc", Utc::now().format("%Y%m%d_%H%M%S"));
        let metadata = BackupMetadata {
            filename: filename.clone(),
            size_bytes: backup_data.len() as u64,
            created_at: encrypted.created_at,
            version: encrypted.version,
            checksum: encrypted.checksum.clone(),
        };

        // Upload to cloud
        self.cloud_manager
            .upload_backup(provider, &backup_data, metadata.clone())?;

        Ok(metadata)
    }

    fn config_path(&self) -> Result<PathBuf, BackupError> {
        let mut path = self.app_handle.path().app_data_dir().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::NotFound, format!("App data directory not found: {}", e))
        })?;
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        path.push(BACKUP_CONFIG_FILE);
        Ok(path)
    }

    fn load_provider_configs_internal(
        &self,
        keystore: &Keystore,
    ) -> Result<Vec<CloudProviderConfig>, BackupError> {
        let path = self.config_path()?;
        if !path.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read(&path)?;
        let encrypted: EncryptedBackup = serde_json::from_slice(&data)?;
        let plaintext = self.decrypt_data(keystore, &encrypted)?;
        let configs: Vec<CloudProviderConfig> = serde_json::from_slice(&plaintext)?;
        Ok(configs)
    }

    fn save_provider_configs_internal(
        &self,
        keystore: &Keystore,
        configs: &[CloudProviderConfig],
    ) -> Result<(), BackupError> {
        let path = self.config_path()?;
        let json = serde_json::to_vec(configs)?;
        let encrypted = self.encrypt_data(keystore, &json)?;
        let data = serde_json::to_vec(&encrypted)?;
        fs::write(path, data)?;
        Ok(())
    }

    pub fn get_provider_configs(&self) -> Result<Vec<CloudProviderConfig>, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;
        self.load_provider_configs_internal(&*keystore)
    }

    pub fn upsert_provider_config(
        &self,
        mut config: CloudProviderConfig,
    ) -> Result<CloudProviderConfig, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;
        let mut configs = self.load_provider_configs_internal(&*keystore)?;

        if config.id.is_empty() {
            config.id = Uuid::new_v4().to_string();
        }

        if config.is_default {
            for existing in configs.iter_mut() {
                existing.is_default = false;
            }
        }

        if let Some(existing) = configs.iter_mut().find(|c| c.id == config.id) {
            *existing = config.clone();
        } else {
            configs.push(config.clone());
        }

        self.save_provider_configs_internal(&*keystore, &configs)?;
        Ok(config)
    }

    pub fn delete_provider_config(&self, id: &str) -> Result<(), BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;
        let mut configs = self.load_provider_configs_internal(&*keystore)?;
        let original_len = configs.len();
        configs.retain(|c| c.id != id);
        if configs.len() == original_len {
            return Err(BackupError::CloudProvider(
                CloudProviderError::NotConfigured,
            ));
        }
        self.save_provider_configs_internal(&*keystore, &configs)?;
        Ok(())
    }

    pub fn set_default_provider(&self, id: &str) -> Result<Vec<CloudProviderConfig>, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;
        let mut configs = self.load_provider_configs_internal(&*keystore)?;

        let mut found = false;
        for config in configs.iter_mut() {
            if config.id == id {
                config.is_default = true;
                found = true;
            } else {
                config.is_default = false;
            }
        }

        if !found {
            return Err(BackupError::CloudProvider(
                CloudProviderError::NotConfigured,
            ));
        }

        self.save_provider_configs_internal(&*keystore, &configs)?;
        Ok(configs)
    }

    pub fn get_default_provider_config(&self) -> Result<Option<CloudProviderConfig>, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;
        let configs = self.load_provider_configs_internal(&*keystore)?;

        if let Some(config) = configs.iter().find(|c| c.enabled && c.is_default) {
            return Ok(Some(config.clone()));
        }

        Ok(configs.into_iter().find(|c| c.enabled))
    }

    pub fn create_backup_by_id(
        &self,
        provider_id: &str,
        sections: Option<Vec<String>>,
    ) -> Result<BackupMetadata, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;
        let mut configs = self.load_provider_configs_internal(&*keystore)?;

        let index =
            configs
                .iter()
                .position(|c| c.id == provider_id)
                .ok_or(BackupError::CloudProvider(
                    CloudProviderError::NotConfigured,
                ))?;

        if !configs[index].enabled {
            return Err(BackupError::CloudProvider(
                CloudProviderError::NotConfigured,
            ));
        }

        let provider = configs[index].provider.clone();
        let metadata = self.create_backup_with_provider(&provider, sections)?;
        configs[index].last_sync = Some(metadata.created_at);
        self.save_provider_configs_internal(&*keystore, &configs)?;
        Ok(metadata)
    }

    pub fn create_default_backup(&self) -> Result<Option<BackupMetadata>, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;
        let mut configs = self.load_provider_configs_internal(&*keystore)?;

        let maybe_index = configs
            .iter()
            .position(|c| c.enabled && c.is_default)
            .or_else(|| configs.iter().position(|c| c.enabled));

        let Some(index) = maybe_index else {
            return Ok(None);
        };

        let provider = configs[index].provider.clone();
        let metadata = self.create_backup_with_provider(&provider, None)?;
        configs[index].last_sync = Some(metadata.created_at);
        self.save_provider_configs_internal(&*keystore, &configs)?;
        Ok(Some(metadata))
    }

    pub fn restore_backup(
        &self,
        provider: &CloudProvider,
        filename: &str,
        merge: bool,
    ) -> Result<(), BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;

        // Download from cloud
        let backup_data = self.cloud_manager.download_backup(provider, filename)?;

        // Deserialize encrypted backup
        let encrypted: EncryptedBackup = serde_json::from_slice(&backup_data)?;

        // Decrypt
        let plaintext = self.decrypt_data(&*keystore, &encrypted)?;

        // Deserialize settings
        let settings: AppSettings = serde_json::from_slice(&plaintext)?;

        // Import settings
        self.settings_manager.import_settings(settings, merge)?;

        Ok(())
    }

    pub fn list_backups(
        &self,
        provider: &CloudProvider,
    ) -> Result<Vec<BackupMetadata>, BackupError> {
        let backups = self.cloud_manager.list_backups(provider)?;
        Ok(backups)
    }

    pub fn delete_backup(
        &self,
        provider: &CloudProvider,
        filename: &str,
    ) -> Result<(), BackupError> {
        self.cloud_manager.delete_backup(provider, filename)?;
        Ok(())
    }

    pub fn verify_backup_integrity(
        &self,
        provider: &CloudProvider,
        filename: &str,
    ) -> Result<bool, BackupError> {
        let keystore = self
            .app_handle
            .try_state::<Keystore>()
            .ok_or(BackupError::KeystoreUnavailable)?;

        let backup_data = self.cloud_manager.download_backup(provider, filename)?;
        let encrypted: EncryptedBackup = serde_json::from_slice(&backup_data)?;

        match self.decrypt_data(&*keystore, &encrypted) {
            Ok(_) => Ok(true),
            Err(BackupError::IntegrityCheckFailed) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn export_settings(
        &self,
        sections: Option<Vec<String>>,
    ) -> Result<AppSettings, BackupError> {
        Ok(self.settings_manager.export_settings(sections)?)
    }

    pub fn import_settings(&self, settings: AppSettings, merge: bool) -> Result<(), BackupError> {
        self.settings_manager.import_settings(settings, merge)?;
        Ok(())
    }

    pub fn reset_settings(&self) -> Result<(), BackupError> {
        self.settings_manager.reset_to_defaults()?;
        Ok(())
    }

    pub fn get_settings_template(&self, template_type: &str) -> Result<AppSettings, BackupError> {
        Ok(self.settings_manager.get_template(template_type)?)
    }
}

// Tauri commands
#[tauri::command]
pub async fn create_backup(
    provider: CloudProvider,
    sections: Option<Vec<String>>,
    backup_service: State<'_, SharedBackupService>,
) -> Result<BackupMetadata, String> {
    let service = backup_service.read().await;
    service
        .create_backup_with_provider(&provider, sections)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_backup(
    provider: CloudProvider,
    filename: String,
    merge: bool,
    backup_service: State<'_, SharedBackupService>,
) -> Result<(), String> {
    let service = backup_service.read().await;
    service
        .restore_backup(&provider, &filename, merge)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_backups(
    provider: CloudProvider,
    backup_service: State<'_, SharedBackupService>,
) -> Result<Vec<BackupMetadata>, String> {
    let service = backup_service.read().await;
    service.list_backups(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_backup(
    provider: CloudProvider,
    filename: String,
    backup_service: State<'_, SharedBackupService>,
) -> Result<(), String> {
    let service = backup_service.read().await;
    service
        .delete_backup(&provider, &filename)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_backup_integrity(
    provider: CloudProvider,
    filename: String,
    backup_service: State<'_, SharedBackupService>,
) -> Result<bool, String> {
    let service = backup_service.read().await;
    service
        .verify_backup_integrity(&provider, &filename)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_settings(
    sections: Option<Vec<String>>,
    backup_service: State<'_, SharedBackupService>,
) -> Result<AppSettings, String> {
    let service = backup_service.read().await;
    service.export_settings(sections).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_settings(
    settings: AppSettings,
    merge: bool,
    backup_service: State<'_, SharedBackupService>,
) -> Result<(), String> {
    let service = backup_service.read().await;
    service
        .import_settings(settings, merge)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_settings(backup_service: State<'_, SharedBackupService>) -> Result<(), String> {
    let service = backup_service.read().await;
    service.reset_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings_template(
    template_type: String,
    backup_service: State<'_, SharedBackupService>,
) -> Result<AppSettings, String> {
    let service = backup_service.read().await;
    service
        .get_settings_template(&template_type)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_backup_schedule(
    scheduler: State<'_, SharedBackupScheduler>,
) -> Result<BackupSchedule, String> {
    let scheduler = scheduler.read().await;
    Ok(scheduler.get_schedule())
}

#[tauri::command]
pub async fn update_backup_schedule(
    schedule: BackupSchedule,
    scheduler: State<'_, SharedBackupScheduler>,
) -> Result<(), String> {
    let mut scheduler = scheduler.write().await;
    scheduler
        .update_schedule(schedule)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_backup_status(
    scheduler: State<'_, SharedBackupScheduler>,
) -> Result<BackupStatus, String> {
    let scheduler = scheduler.read().await;
    Ok(scheduler.get_status())
}

#[tauri::command]
pub async fn trigger_manual_backup(
    provider: CloudProvider,
    backup_service: State<'_, SharedBackupService>,
    scheduler: State<'_, SharedBackupScheduler>,
) -> Result<BackupMetadata, String> {
    {
        let mut sched = scheduler.write().await;
        sched.start_backup();
    }

    let result = {
        let service = backup_service.read().await;
        service.create_backup_with_provider(&provider, None)
    };

    match result {
        Ok(metadata) => {
            let mut sched = scheduler.write().await;
            let _ = sched.mark_backup_complete(metadata.size_bytes);
            Ok(metadata)
        }
        Err(e) => {
            let mut sched = scheduler.write().await;
            let _ = sched.mark_backup_failed(e.to_string());
            Err(e.to_string())
        }
    }
}
