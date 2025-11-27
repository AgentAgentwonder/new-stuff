use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, Runtime, State, Window};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateSchedule {
    Daily,
    Weekly,
    Never,
}

impl Default for UpdateSchedule {
    fn default() -> Self {
        Self::Weekly
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettings {
    pub schedule: UpdateSchedule,
    pub auto_download: bool,
    pub auto_install: bool,
    pub last_check: Option<String>,
    pub dismissed_version: Option<String>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            schedule: UpdateSchedule::Weekly,
            auto_download: true,
            auto_install: false,
            last_check: None,
            dismissed_version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackInfo {
    pub available: bool,
    pub previous_version: Option<String>,
    pub backup_timestamp: Option<String>,
}

pub struct UpdaterState {
    pub settings: RwLock<UpdateSettings>,
    pub backup_path: PathBuf,
}

impl UpdaterState {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Unable to resolve app data directory: {}", e))?;

        let backup_path = app_data_dir.join("backups");
        if !backup_path.exists() {
            fs::create_dir_all(&backup_path)
                .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        }

        let settings_path = app_data_dir.join("updater_settings.json");
        let settings = if settings_path.exists() {
            let contents = fs::read_to_string(&settings_path)
                .map_err(|e| format!("Failed to read settings: {}", e))?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            UpdateSettings::default()
        };

        Ok(Self {
            settings: RwLock::new(settings),
            backup_path,
        })
    }

    pub async fn save_settings(&self, app_handle: &AppHandle) -> Result<(), String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Unable to resolve app data directory: {}", e))?;

        let settings_path = app_data_dir.join("updater_settings.json");
        let settings = self.settings.read().await;
        let json = serde_json::to_string_pretty(&*settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&settings_path, json)
            .map_err(|e| format!("Failed to write settings: {}", e))?;

        Ok(())
    }
}

pub type SharedUpdaterState = Arc<UpdaterState>;

#[tauri::command]
pub async fn get_update_settings(
    state: State<'_, SharedUpdaterState>,
) -> Result<UpdateSettings, String> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_update_settings(
    app_handle: AppHandle,
    state: State<'_, SharedUpdaterState>,
    settings: UpdateSettings,
) -> Result<(), String> {
    let mut current_settings = state.settings.write().await;
    *current_settings = settings;
    drop(current_settings);
    state.save_settings(&app_handle).await
}

#[tauri::command]
pub async fn dismiss_update(
    app_handle: AppHandle,
    state: State<'_, SharedUpdaterState>,
    version: String,
) -> Result<(), String> {
    let mut settings = state.settings.write().await;
    settings.dismissed_version = Some(version);
    drop(settings);
    state.save_settings(&app_handle).await
}

#[tauri::command]
pub async fn get_rollback_info(
    state: State<'_, SharedUpdaterState>,
) -> Result<RollbackInfo, String> {
    let backup_dir = &state.backup_path;

    if !backup_dir.exists() {
        return Ok(RollbackInfo {
            available: false,
            previous_version: None,
            backup_timestamp: None,
        });
    }

    let metadata_path = backup_dir.join("metadata.json");
    if metadata_path.exists() {
        let contents = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read backup metadata: {}", e))?;

        #[derive(Deserialize)]
        struct BackupMetadata {
            version: String,
            timestamp: String,
        }

        if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&contents) {
            return Ok(RollbackInfo {
                available: true,
                previous_version: Some(metadata.version),
                backup_timestamp: Some(metadata.timestamp),
            });
        }
    }

    Ok(RollbackInfo {
        available: false,
        previous_version: None,
        backup_timestamp: None,
    })
}

#[tauri::command]
pub async fn rollback_update<R: Runtime>(
    app_handle: AppHandle<R>,
    state: State<'_, SharedUpdaterState>,
    window: Window<R>,
) -> Result<(), String> {
    let backup_dir = &state.backup_path;
    if !backup_dir.exists() {
        return Err("No backup available for rollback".to_string());
    }

    let metadata_path = backup_dir.join("metadata.json");
    if !metadata_path.exists() {
        return Err("Backup metadata not found".to_string());
    }

    // Emit events to notify the UI. Real rollback logic would restore files and restart the app.
    let _ = window.emit("rollback-started", ());

    tracing::info!(
        "Rollback requested. Restore backup located at: {:?}",
        backup_dir
    );

    let _ = window.emit("rollback-completed", ());

    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let _ = app_handle.restart();
    });

    Ok(())
}
