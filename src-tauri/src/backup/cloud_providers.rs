use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const STORAGE_DIR: &str = "cloud_backups";
const MAX_VERSIONS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum CloudProvider {
    S3 {
        region: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        endpoint: Option<String>,
    },
    GoogleDrive {
        access_token: String,
        refresh_token: String,
        client_id: String,
        client_secret: String,
        folder_id: Option<String>,
    },
    Dropbox {
        access_token: String,
        refresh_token: String,
        app_key: String,
        app_secret: String,
        folder_path: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudProviderConfig {
    #[serde(default)]
    pub id: String,
    pub provider: CloudProvider,
    pub enabled: bool,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub last_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMetadata {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
    pub version: u32,
    pub checksum: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CloudProviderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("provider not configured")]
    NotConfigured,
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("backup not found")]
    NotFound,
    #[error("unsupported provider type")]
    Unsupported,
}

pub struct CloudProviderManager {
    app_handle: AppHandle,
}

impl CloudProviderManager {
    pub fn new(app: &AppHandle) -> Self {
        Self {
            app_handle: app.clone(),
        }
    }

    fn storage_base(&self) -> Result<PathBuf, CloudProviderError> {
        let mut path = self
            .app_handle
            .path()
            .app_data_dir()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, format!("app data dir: {}", e)))?;
        path.push(STORAGE_DIR);
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    fn provider_dir(&self, provider: &CloudProvider) -> Result<PathBuf, CloudProviderError> {
        let mut path = self.storage_base()?;
        path.push(match provider {
            CloudProvider::S3 { .. } => "aws_s3",
            CloudProvider::GoogleDrive { .. } => "google_drive",
            CloudProvider::Dropbox { .. } => "dropbox",
        });
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    fn metadata_path(&self, provider: &CloudProvider) -> Result<PathBuf, CloudProviderError> {
        let mut path = self.provider_dir(provider)?;
        path.push("metadata.json");
        Ok(path)
    }

    fn read_metadata(
        &self,
        provider: &CloudProvider,
    ) -> Result<Vec<BackupMetadata>, CloudProviderError> {
        let path = self.metadata_path(provider)?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = fs::read_to_string(path)?;
        let metadata: Vec<BackupMetadata> = serde_json::from_str(&data)?;
        Ok(metadata)
    }

    fn write_metadata(
        &self,
        provider: &CloudProvider,
        metadata: &[BackupMetadata],
    ) -> Result<(), CloudProviderError> {
        let path = self.metadata_path(provider)?;
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn upload_backup(
        &self,
        provider: &CloudProvider,
        data: &[u8],
        metadata: BackupMetadata,
    ) -> Result<String, CloudProviderError> {
        let mut metadata_list = self.read_metadata(provider)?;
        let dir = self.provider_dir(provider)?;
        let file_path = dir.join(&metadata.filename);
        fs::write(&file_path, data)?;

        metadata_list.insert(0, metadata);
        metadata_list.truncate(MAX_VERSIONS);
        self.write_metadata(provider, &metadata_list)?;

        Ok(file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string())
    }

    pub fn download_backup(
        &self,
        provider: &CloudProvider,
        filename: &str,
    ) -> Result<Vec<u8>, CloudProviderError> {
        let dir = self.provider_dir(provider)?;
        let file_path = dir.join(filename);
        if !file_path.exists() {
            return Err(CloudProviderError::NotFound);
        }
        let data = fs::read(file_path)?;
        Ok(data)
    }

    pub fn list_backups(
        &self,
        provider: &CloudProvider,
    ) -> Result<Vec<BackupMetadata>, CloudProviderError> {
        self.read_metadata(provider)
    }

    pub fn delete_backup(
        &self,
        provider: &CloudProvider,
        filename: &str,
    ) -> Result<(), CloudProviderError> {
        let mut metadata_list = self.read_metadata(provider)?;
        metadata_list.retain(|m| m.filename != filename);
        self.write_metadata(provider, &metadata_list)?;

        let dir = self.provider_dir(provider)?;
        let file_path = dir.join(filename);
        if file_path.exists() {
            let _ = fs::remove_file(file_path);
        }
        Ok(())
    }
}

pub trait ProviderKeyExt {
    fn keystore_key(&self) -> String;
}

impl ProviderKeyExt for CloudProvider {
    fn keystore_key(&self) -> String {
        match self {
            CloudProvider::S3 { bucket, .. } => format!("backup.s3.{}", bucket),
            CloudProvider::GoogleDrive { client_id, .. } => format!("backup.gdrive.{}", client_id),
            CloudProvider::Dropbox { app_key, .. } => format!("backup.dropbox.{}", app_key),
        }
    }
}
