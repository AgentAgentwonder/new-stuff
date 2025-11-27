use auto_launch::AutoLaunch;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoStartSettings {
    pub enabled: bool,
    pub start_minimized: bool,
    pub delay_seconds: u32,
}

impl Default for AutoStartSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            start_minimized: false,
            delay_seconds: 0,
        }
    }
}

pub struct AutoStartManager {
    settings: RwLock<AutoStartSettings>,
    settings_path: RwLock<Option<PathBuf>>,
    auto_launch: Option<AutoLaunch>,
}

impl AutoStartManager {
    pub fn new(app_name: &str, app_path: &str) -> Result<Self, String> {
        use auto_launch::AutoLaunchBuilder;

        let mut builder = AutoLaunchBuilder::new();
        builder.set_app_name(app_name);
        builder.set_app_path(app_path);
        builder.set_args(&["--auto-start"]);
        #[cfg(target_os = "macos")]
        builder.set_use_launch_agent(true);

        let auto_launch = builder
            .build()
            .map_err(|e| format!("Failed to build AutoLaunch: {:?}", e))?;

        Ok(Self {
            settings: RwLock::new(AutoStartSettings::default()),
            settings_path: RwLock::new(None),
            auto_launch: Some(auto_launch),
        })
    }

    pub fn initialize(&self, app_handle: &AppHandle) {
        if let Ok(mut data_dir) = app_handle.path().app_data_dir() {
            if let Err(err) = fs::create_dir_all(&data_dir) {
                eprintln!("Failed to ensure auto-start settings directory: {err}");
            } else {
                data_dir.push("auto_start_settings.json");
                let mut settings_guard = self.settings.write();
                self.settings_path.write().replace(data_dir.clone());

                if let Ok(contents) = fs::read_to_string(&data_dir) {
                    if let Ok(parsed) = serde_json::from_str::<AutoStartSettings>(&contents) {
                        *settings_guard = parsed;
                    }
                }
            }
        }

        // Sync current state with OS
        if let Err(err) = self.sync_state() {
            eprintln!("Failed to sync auto-start state: {err}");
        }
    }

    fn save_settings(&self) -> Result<(), String> {
        let path = {
            let path_guard = self.settings_path.read();
            path_guard.clone()
        };

        if let Some(path) = path {
            let settings = self.settings.read().clone();
            let contents = serde_json::to_string_pretty(&settings)
                .map_err(|e| format!("Failed to serialize auto-start settings: {e}"))?;
            fs::write(&path, contents)
                .map_err(|e| format!("Failed to persist auto-start settings: {e}"))?;
        }

        Ok(())
    }

    fn sync_state(&self) -> Result<(), String> {
        if let Some(auto_launch) = &self.auto_launch {
            let enabled = self.settings.read().enabled;
            let is_enabled = auto_launch
                .is_enabled()
                .map_err(|e| format!("Failed to check auto-start status: {e}"))?;

            if enabled && !is_enabled {
                auto_launch
                    .enable()
                    .map_err(|e| format!("Failed to enable auto-start: {e}"))?;
            } else if !enabled && is_enabled {
                auto_launch
                    .disable()
                    .map_err(|e| format!("Failed to disable auto-start: {e}"))?;
            }
        }
        Ok(())
    }

    pub fn get_settings(&self) -> AutoStartSettings {
        self.settings.read().clone()
    }

    pub fn update_settings(&self, new_settings: AutoStartSettings) -> Result<(), String> {
        {
            let mut settings = self.settings.write();
            *settings = new_settings;
        }

        self.save_settings()?;
        self.sync_state()?;
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool, String> {
        if let Some(auto_launch) = &self.auto_launch {
            auto_launch
                .is_enabled()
                .map_err(|e| format!("Failed to check auto-start status: {e}"))
        } else {
            Ok(false)
        }
    }
}

pub type SharedAutoStartManager = Arc<AutoStartManager>;

// Tauri commands

#[tauri::command]
pub fn get_auto_start_settings(
    auto_start_manager: tauri::State<'_, SharedAutoStartManager>,
) -> Result<AutoStartSettings, String> {
    Ok(auto_start_manager.get_settings())
}

#[tauri::command]
pub fn update_auto_start_settings(
    settings: AutoStartSettings,
    auto_start_manager: tauri::State<'_, SharedAutoStartManager>,
) -> Result<(), String> {
    auto_start_manager.update_settings(settings)
}

#[tauri::command]
pub fn check_auto_start_enabled(
    auto_start_manager: tauri::State<'_, SharedAutoStartManager>,
) -> Result<bool, String> {
    auto_start_manager.is_enabled()
}

#[tauri::command]
pub fn enable_auto_start(
    auto_start_manager: tauri::State<'_, SharedAutoStartManager>,
) -> Result<(), String> {
    let mut settings = auto_start_manager.get_settings();
    settings.enabled = true;
    auto_start_manager.update_settings(settings)
}

#[tauri::command]
pub fn disable_auto_start(
    auto_start_manager: tauri::State<'_, SharedAutoStartManager>,
) -> Result<(), String> {
    let mut settings = auto_start_manager.get_settings();
    settings.enabled = false;
    auto_start_manager.update_settings(settings)
}
