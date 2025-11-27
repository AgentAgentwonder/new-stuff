use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub trading: Option<TradingSettings>,
    pub security: Option<SecuritySettings>,
    pub appearance: Option<AppearanceSettings>,
    pub api: Option<ApiSettings>,
    pub notifications: Option<NotificationSettings>,
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingSettings {
    pub slippage: f64,
    pub slippage_auto_adjust: bool,
    pub slippage_max_tolerance: f64,
    pub slippage_reject_above: f64,
    pub mev_protection: bool,
    pub jito_enabled: bool,
    pub private_rpc_enabled: bool,
    pub gas_optimization: bool,
    pub priority_fee: u64,
    pub paper_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettings {
    pub biometric_enabled: bool,
    pub two_factor_enabled: bool,
    pub auto_lock_enabled: bool,
    pub auto_lock_minutes: u32,
    pub hardware_wallet_preferred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: String,
    pub custom_theme: Option<HashMap<String, String>>,
    pub accessibility: Option<AccessibilitySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilitySettings {
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub screen_reader: bool,
    pub font_size: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSettings {
    pub birdeye_key: Option<String>,
    pub helius_key: Option<String>,
    pub custom_rpc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub email_enabled: bool,
    pub webhook_enabled: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid settings version")]
    InvalidVersion,
    #[error("Settings not found")]
    NotFound,
    #[error("Validation failed: {0}")]
    Validation(String),
}

pub struct SettingsManager {
    app_handle: AppHandle,
}

impl SettingsManager {
    pub fn new(app: &AppHandle) -> Self {
        Self {
            app_handle: app.clone(),
        }
    }

    fn settings_path(&self) -> Result<PathBuf, SettingsError> {
        let mut path = self
            .app_handle
            .path()
            .app_data_dir()
            .map_err(|e| {
                SettingsError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("App data directory not found: {}", e),
                ))
            })?;

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        path.push("settings.json");
        Ok(path)
    }

    pub fn export_settings(
        &self,
        sections: Option<Vec<String>>,
    ) -> Result<AppSettings, SettingsError> {
        let path = self.settings_path()?;

        let settings = if path.exists() {
            let data = fs::read_to_string(&path)?;
            let mut settings: AppSettings = serde_json::from_str(&data)?;
            settings.exported_at = Utc::now();

            // Filter sections if requested
            if let Some(sections) = sections {
                if !sections.contains(&"trading".to_string()) {
                    settings.trading = None;
                }
                if !sections.contains(&"security".to_string()) {
                    settings.security = None;
                }
                if !sections.contains(&"appearance".to_string()) {
                    settings.appearance = None;
                }
                if !sections.contains(&"api".to_string()) {
                    settings.api = None;
                }
                if !sections.contains(&"notifications".to_string()) {
                    settings.notifications = None;
                }
            }

            settings
        } else {
            // Return default settings
            AppSettings {
                version: SETTINGS_VERSION,
                exported_at: Utc::now(),
                trading: Some(TradingSettings::default()),
                security: Some(SecuritySettings::default()),
                appearance: Some(AppearanceSettings::default()),
                api: None,
                notifications: Some(NotificationSettings::default()),
                custom: None,
            }
        };

        Ok(settings)
    }

    pub fn import_settings(&self, settings: AppSettings, merge: bool) -> Result<(), SettingsError> {
        // Validate version
        if settings.version > SETTINGS_VERSION {
            return Err(SettingsError::InvalidVersion);
        }

        // Validate settings values
        self.validate_settings(&settings)?;

        let path = self.settings_path()?;

        let final_settings = if merge && path.exists() {
            let data = fs::read_to_string(&path)?;
            let mut existing: AppSettings = serde_json::from_str(&data)?;

            // Merge settings
            if settings.trading.is_some() {
                existing.trading = settings.trading;
            }
            if settings.security.is_some() {
                existing.security = settings.security;
            }
            if settings.appearance.is_some() {
                existing.appearance = settings.appearance;
            }
            if settings.api.is_some() {
                existing.api = settings.api;
            }
            if settings.notifications.is_some() {
                existing.notifications = settings.notifications;
            }
            if let Some(custom) = settings.custom {
                let mut existing_custom = existing.custom.unwrap_or_default();
                for (key, value) in custom {
                    existing_custom.insert(key, value);
                }
                existing.custom = Some(existing_custom);
            }

            existing.version = SETTINGS_VERSION;
            existing
        } else {
            settings
        };

        // Save to file
        let json = serde_json::to_string_pretty(&final_settings)?;
        fs::write(&path, json)?;

        Ok(())
    }

    pub fn reset_to_defaults(&self) -> Result<(), SettingsError> {
        let settings = AppSettings {
            version: SETTINGS_VERSION,
            exported_at: Utc::now(),
            trading: Some(TradingSettings::default()),
            security: Some(SecuritySettings::default()),
            appearance: Some(AppearanceSettings::default()),
            api: None,
            notifications: Some(NotificationSettings::default()),
            custom: None,
        };

        let path = self.settings_path()?;
        let json = serde_json::to_string_pretty(&settings)?;
        fs::write(&path, json)?;

        Ok(())
    }

    fn validate_settings(&self, settings: &AppSettings) -> Result<(), SettingsError> {
        // Validate trading settings
        if let Some(ref trading) = settings.trading {
            if trading.slippage < 0.0 || trading.slippage > 100.0 {
                return Err(SettingsError::Validation(
                    "Slippage must be between 0 and 100".to_string(),
                ));
            }
            if trading.slippage_max_tolerance < 0.0 || trading.slippage_max_tolerance > 100.0 {
                return Err(SettingsError::Validation(
                    "Max tolerance must be between 0 and 100".to_string(),
                ));
            }
        }

        // Validate security settings
        if let Some(ref security) = settings.security {
            if security.auto_lock_minutes == 0 || security.auto_lock_minutes > 1440 {
                return Err(SettingsError::Validation(
                    "Auto lock must be between 1 and 1440 minutes".to_string(),
                ));
            }
        }

        // Validate appearance settings
        if let Some(ref appearance) = settings.appearance {
            if let Some(ref accessibility) = appearance.accessibility {
                if accessibility.font_size < 8 || accessibility.font_size > 32 {
                    return Err(SettingsError::Validation(
                        "Font size must be between 8 and 32".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn get_template(&self, template_type: &str) -> Result<AppSettings, SettingsError> {
        let settings = match template_type {
            "conservative" => AppSettings {
                version: SETTINGS_VERSION,
                exported_at: Utc::now(),
                trading: Some(TradingSettings {
                    slippage: 0.5,
                    slippage_auto_adjust: false,
                    slippage_max_tolerance: 1.0,
                    slippage_reject_above: 2.0,
                    mev_protection: true,
                    jito_enabled: true,
                    private_rpc_enabled: true,
                    gas_optimization: true,
                    priority_fee: 100000,
                    paper_mode: false,
                }),
                security: Some(SecuritySettings {
                    biometric_enabled: true,
                    two_factor_enabled: true,
                    auto_lock_enabled: true,
                    auto_lock_minutes: 5,
                    hardware_wallet_preferred: true,
                }),
                appearance: Some(AppearanceSettings::default()),
                api: None,
                notifications: Some(NotificationSettings {
                    enabled: true,
                    sound_enabled: true,
                    email_enabled: true,
                    webhook_enabled: false,
                }),
                custom: None,
            },
            "aggressive" => AppSettings {
                version: SETTINGS_VERSION,
                exported_at: Utc::now(),
                trading: Some(TradingSettings {
                    slippage: 2.0,
                    slippage_auto_adjust: true,
                    slippage_max_tolerance: 5.0,
                    slippage_reject_above: 10.0,
                    mev_protection: false,
                    jito_enabled: false,
                    private_rpc_enabled: false,
                    gas_optimization: false,
                    priority_fee: 1000000,
                    paper_mode: false,
                }),
                security: Some(SecuritySettings {
                    biometric_enabled: false,
                    two_factor_enabled: false,
                    auto_lock_enabled: false,
                    auto_lock_minutes: 30,
                    hardware_wallet_preferred: false,
                }),
                appearance: Some(AppearanceSettings::default()),
                api: None,
                notifications: Some(NotificationSettings {
                    enabled: true,
                    sound_enabled: false,
                    email_enabled: false,
                    webhook_enabled: true,
                }),
                custom: None,
            },
            _ => {
                return Err(SettingsError::Validation(
                    "Unknown template type".to_string(),
                ))
            }
        };

        Ok(settings)
    }
}

impl Default for TradingSettings {
    fn default() -> Self {
        Self {
            slippage: 1.0,
            slippage_auto_adjust: true,
            slippage_max_tolerance: 3.0,
            slippage_reject_above: 5.0,
            mev_protection: true,
            jito_enabled: false,
            private_rpc_enabled: false,
            gas_optimization: true,
            priority_fee: 100000,
            paper_mode: true,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            biometric_enabled: false,
            two_factor_enabled: false,
            auto_lock_enabled: true,
            auto_lock_minutes: 15,
            hardware_wallet_preferred: false,
        }
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            custom_theme: None,
            accessibility: Some(AccessibilitySettings::default()),
        }
    }
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            high_contrast: false,
            reduced_motion: false,
            screen_reader: false,
            font_size: 14,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sound_enabled: true,
            email_enabled: false,
            webhook_enabled: false,
        }
    }
}
