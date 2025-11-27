use super::settings_schema::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

pub type SharedSettingsManager = Arc<RwLock<SettingsManager>>;

const SETTINGS_FILE: &str = "universal_settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsExport {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub profile_name: Option<String>,
    pub settings: UniversalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsProfile {
    pub name: String,
    pub description: Option<String>,
    pub settings: UniversalSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid settings version: expected {expected}, got {actual}")]
    InvalidVersion { expected: u32, actual: u32 },
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    #[error("Setting not found: {category}.{key}")]
    SettingNotFound { category: String, key: String },
}

pub struct SettingsManager {
    app_handle: AppHandle,
    current_settings: UniversalSettings,
    profiles: HashMap<String, SettingsProfile>,
    change_history: Vec<SettingsChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsChange {
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub key: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

impl SettingsManager {
    pub fn new(app: &AppHandle) -> Result<Self, SettingsError> {
        let mut manager = Self {
            app_handle: app.clone(),
            current_settings: UniversalSettings::default(),
            profiles: HashMap::new(),
            change_history: Vec::new(),
        };

        // Try to load existing settings
        if let Err(e) = manager.load_settings() {
            eprintln!("Failed to load settings, using defaults: {}", e);
            // Save default settings
            let _ = manager.save_settings();
        }

        // Load profiles
        if let Err(e) = manager.load_profiles() {
            eprintln!("Failed to load profiles: {}", e);
        }

        Ok(manager)
    }

    fn settings_path(&self) -> Result<PathBuf, SettingsError> {
        let app_handle = self.app_handle.clone();
        let mut path = app_handle.path().app_data_dir().map_err(|e| {
            SettingsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("App data directory not found: {}", e),
            ))
        })?;

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        path.push(SETTINGS_FILE);
        Ok(path)
    }

    fn profiles_path(&self) -> Result<PathBuf, SettingsError> {
        let app_handle = self.app_handle.clone();
        let mut path = app_handle.path().app_data_dir().map_err(|e| {
            SettingsError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("App data directory not found: {}", e),
            ))
        })?;

        path.push("settings_profiles.json");
        Ok(path)
    }

    fn load_settings(&mut self) -> Result<(), SettingsError> {
        let path = self.settings_path()?;

        if path.exists() {
            let data = fs::read_to_string(&path)?;
            let export: SettingsExport = serde_json::from_str(&data)?;

            // Check version compatibility
            if export.version > SETTINGS_SCHEMA_VERSION {
                return Err(SettingsError::InvalidVersion {
                    expected: SETTINGS_SCHEMA_VERSION,
                    actual: export.version,
                });
            }

            self.current_settings = export.settings;
        }

        Ok(())
    }

    fn save_settings(&self) -> Result<(), SettingsError> {
        let path = self.settings_path()?;

        let export = SettingsExport {
            version: SETTINGS_SCHEMA_VERSION,
            exported_at: Utc::now(),
            profile_name: None,
            settings: self.current_settings.clone(),
        };

        let json = serde_json::to_string_pretty(&export)?;
        fs::write(&path, json)?;

        Ok(())
    }

    fn load_profiles(&mut self) -> Result<(), SettingsError> {
        let path = self.profiles_path()?;

        if path.exists() {
            let data = fs::read_to_string(&path)?;
            self.profiles = serde_json::from_str(&data)?;
        }

        Ok(())
    }

    fn save_profiles(&self) -> Result<(), SettingsError> {
        let path = self.profiles_path()?;
        let json = serde_json::to_string_pretty(&self.profiles)?;
        fs::write(&path, json)?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> UniversalSettings {
        self.current_settings.clone()
    }

    pub fn update_setting(
        &mut self,
        category: String,
        key: String,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        // Record old value for history
        let old_value = self.get_setting_value(&category, &key)?;

        // Apply the update
        self.apply_setting_update(&category, &key, value.clone())?;

        // Validate after update
        self.validate_settings()?;

        // Save to disk
        self.save_settings()?;

        // Record change
        self.change_history.push(SettingsChange {
            timestamp: Utc::now(),
            category: category.clone(),
            key: key.clone(),
            old_value,
            new_value: value,
        });

        // Keep history limited to last 100 changes
        if self.change_history.len() > 100 {
            self.change_history.remove(0);
        }

        Ok(())
    }

    fn get_setting_value(
        &self,
        category: &str,
        key: &str,
    ) -> Result<serde_json::Value, SettingsError> {
        let settings_json = serde_json::to_value(&self.current_settings)
            .map_err(|e| SettingsError::Serialization(e))?;

        settings_json
            .get(category)
            .and_then(|cat| cat.get(key))
            .cloned()
            .ok_or_else(|| SettingsError::SettingNotFound {
                category: category.to_string(),
                key: key.to_string(),
            })
    }

    fn apply_setting_update(
        &mut self,
        category: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match category {
            "trading" => self.update_trading_setting(key, value)?,
            "aiAssistant" => self.update_ai_assistant_setting(key, value)?,
            "voice" => self.update_voice_setting(key, value)?,
            "uiTheme" => self.update_ui_theme_setting(key, value)?,
            "alerts" => self.update_alerts_setting(key, value)?,
            "performance" => self.update_performance_setting(key, value)?,
            "security" => self.update_security_setting(key, value)?,
            "dataPrivacy" => self.update_data_privacy_setting(key, value)?,
            "network" => self.update_network_setting(key, value)?,
            "automation" => self.update_automation_setting(key, value)?,
            "developer" => self.update_developer_setting(key, value)?,
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: category.to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_trading_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "defaultSlippage" => {
                self.current_settings.trading.default_slippage = serde_json::from_value(value)?
            }
            "gasPriority" => {
                self.current_settings.trading.gas_priority = serde_json::from_value(value)?
            }
            "defaultOrderType" => {
                self.current_settings.trading.default_order_type = serde_json::from_value(value)?
            }
            "autoConfirmBelow" => {
                self.current_settings.trading.auto_confirm_below = serde_json::from_value(value)?
            }
            "tradeConfirmationTimeout" => {
                self.current_settings.trading.trade_confirmation_timeout =
                    serde_json::from_value(value)?
            }
            "maxPositionSizePercent" => {
                self.current_settings.trading.max_position_size_percent =
                    serde_json::from_value(value)?
            }
            "paperTradingMode" => {
                self.current_settings.trading.paper_trading_mode = serde_json::from_value(value)?
            }
            "multiWalletBehavior" => {
                self.current_settings.trading.multi_wallet_behavior = serde_json::from_value(value)?
            }
            "mevProtection" => {
                self.current_settings.trading.mev_protection = serde_json::from_value(value)?
            }
            "jitoEnabled" => {
                self.current_settings.trading.jito_enabled = serde_json::from_value(value)?
            }
            "privateRpcEnabled" => {
                self.current_settings.trading.private_rpc_enabled = serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "trading".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_ai_assistant_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "provider" => {
                self.current_settings.ai_assistant.provider = serde_json::from_value(value)?
            }
            "apiKey" => self.current_settings.ai_assistant.api_key = serde_json::from_value(value)?,
            "model" => self.current_settings.ai_assistant.model = serde_json::from_value(value)?,
            "temperature" => {
                self.current_settings.ai_assistant.temperature = serde_json::from_value(value)?
            }
            "maxTokens" => {
                self.current_settings.ai_assistant.max_tokens = serde_json::from_value(value)?
            }
            "contextWindowSize" => {
                self.current_settings.ai_assistant.context_window_size =
                    serde_json::from_value(value)?
            }
            "autoSuggestions" => {
                self.current_settings.ai_assistant.auto_suggestions = serde_json::from_value(value)?
            }
            "patternLearning" => {
                self.current_settings.ai_assistant.pattern_learning = serde_json::from_value(value)?
            }
            "voicePersonality" => {
                self.current_settings.ai_assistant.voice_personality =
                    serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "aiAssistant".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_voice_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "wakeWord" => self.current_settings.voice.wake_word = serde_json::from_value(value)?,
            "language" => self.current_settings.voice.language = serde_json::from_value(value)?,
            "speechRate" => {
                self.current_settings.voice.speech_rate = serde_json::from_value(value)?
            }
            "voicePreference" => {
                self.current_settings.voice.voice_preference = serde_json::from_value(value)?
            }
            "confirmationRequirements" => {
                self.current_settings.voice.confirmation_requirements =
                    serde_json::from_value(value)?
            }
            "audioAlertsVolume" => {
                self.current_settings.voice.audio_alerts_volume = serde_json::from_value(value)?
            }
            "ttsProvider" => {
                self.current_settings.voice.tts_provider = serde_json::from_value(value)?
            }
            "microphoneSensitivity" => {
                self.current_settings.voice.microphone_sensitivity = serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "voice".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_ui_theme_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "lunarThemeIntensity" => {
                self.current_settings.ui_theme.lunar_theme_intensity =
                    serde_json::from_value(value)?
            }
            "gradientStrength" => {
                self.current_settings.ui_theme.gradient_strength = serde_json::from_value(value)?
            }
            "animationSpeed" => {
                self.current_settings.ui_theme.animation_speed = serde_json::from_value(value)?
            }
            "glassEffectOpacity" => {
                self.current_settings.ui_theme.glass_effect_opacity = serde_json::from_value(value)?
            }
            "coronaGlowIntensity" => {
                self.current_settings.ui_theme.corona_glow_intensity =
                    serde_json::from_value(value)?
            }
            "fontSizeMultiplier" => {
                self.current_settings.ui_theme.font_size_multiplier = serde_json::from_value(value)?
            }
            "colorBlindnessMode" => {
                self.current_settings.ui_theme.color_blindness_mode = serde_json::from_value(value)?
            }
            "reduceMotion" => {
                self.current_settings.ui_theme.reduce_motion = serde_json::from_value(value)?
            }
            "customColors" => {
                self.current_settings.ui_theme.custom_colors = serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "uiTheme".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_alerts_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "defaultChannels" => {
                self.current_settings.alerts.default_channels = serde_json::from_value(value)?
            }
            "cooldownSeconds" => {
                self.current_settings.alerts.cooldown_seconds = serde_json::from_value(value)?
            }
            "smartFilterThreshold" => {
                self.current_settings.alerts.smart_filter_threshold = serde_json::from_value(value)?
            }
            "notificationSound" => {
                self.current_settings.alerts.notification_sound = serde_json::from_value(value)?
            }
            "doNotDisturbSchedule" => {
                self.current_settings.alerts.do_not_disturb_schedule =
                    serde_json::from_value(value)?
            }
            "priorityLevels" => {
                self.current_settings.alerts.priority_levels = serde_json::from_value(value)?
            }
            "batchAlerts" => {
                self.current_settings.alerts.batch_alerts = serde_json::from_value(value)?
            }
            "desktopNotificationStyle" => {
                self.current_settings.alerts.desktop_notification_style =
                    serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "alerts".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_performance_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "chartUpdateFrequencyMs" => {
                self.current_settings.performance.chart_update_frequency_ms =
                    serde_json::from_value(value)?
            }
            "dataCacheTtlSeconds" => {
                self.current_settings.performance.data_cache_ttl_seconds =
                    serde_json::from_value(value)?
            }
            "maxConcurrentRequests" => {
                self.current_settings.performance.max_concurrent_requests =
                    serde_json::from_value(value)?
            }
            "websocketReconnectStrategy" => {
                self.current_settings
                    .performance
                    .websocket_reconnect_strategy = serde_json::from_value(value)?
            }
            "prefetchAggressiveness" => {
                self.current_settings.performance.prefetch_aggressiveness =
                    serde_json::from_value(value)?
            }
            "memoryLimitMb" => {
                self.current_settings.performance.memory_limit_mb = serde_json::from_value(value)?
            }
            "gpuAcceleration" => {
                self.current_settings.performance.gpu_acceleration = serde_json::from_value(value)?
            }
            "virtualScrollingThreshold" => {
                self.current_settings
                    .performance
                    .virtual_scrolling_threshold = serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "performance".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_security_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "sessionTimeoutMinutes" => {
                self.current_settings.security.session_timeout_minutes =
                    serde_json::from_value(value)?
            }
            "twoFaRequirements" => {
                self.current_settings.security.two_fa_requirements = serde_json::from_value(value)?
            }
            "biometricEnabled" => {
                self.current_settings.security.biometric_enabled = serde_json::from_value(value)?
            }
            "keystoreBackupFrequencyDays" => {
                self.current_settings
                    .security
                    .keystore_backup_frequency_days = serde_json::from_value(value)?
            }
            "autoLockOnIdle" => {
                self.current_settings.security.auto_lock_on_idle = serde_json::from_value(value)?
            }
            "autoLockMinutes" => {
                self.current_settings.security.auto_lock_minutes = serde_json::from_value(value)?
            }
            "transactionConfirmationRequirements" => {
                self.current_settings
                    .security
                    .transaction_confirmation_requirements = serde_json::from_value(value)?
            }
            "hardwareWalletPreferred" => {
                self.current_settings.security.hardware_wallet_preferred =
                    serde_json::from_value(value)?
            }
            "apiKeyRotationDays" => {
                self.current_settings.security.api_key_rotation_days =
                    serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "security".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_data_privacy_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "dataRetentionDays" => {
                self.current_settings.data_privacy.data_retention_days =
                    serde_json::from_value(value)?
            }
            "analyticsOptIn" => {
                self.current_settings.data_privacy.analytics_opt_in = serde_json::from_value(value)?
            }
            "shareAnonymousUsage" => {
                self.current_settings.data_privacy.share_anonymous_usage =
                    serde_json::from_value(value)?
            }
            "activityLogRetentionDays" => {
                self.current_settings
                    .data_privacy
                    .activity_log_retention_days = serde_json::from_value(value)?
            }
            "exportFormat" => {
                self.current_settings.data_privacy.export_format = serde_json::from_value(value)?
            }
            "telemetryEnabled" => {
                self.current_settings.data_privacy.telemetry_enabled =
                    serde_json::from_value(value)?
            }
            "crashReporting" => {
                self.current_settings.data_privacy.crash_reporting = serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "dataPrivacy".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_network_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "solanaRpcEndpoint" => {
                self.current_settings.network.solana_rpc_endpoint = serde_json::from_value(value)?
            }
            "rpcFallbackEndpoints" => {
                self.current_settings.network.rpc_fallback_endpoints =
                    serde_json::from_value(value)?
            }
            "websocketEndpoint" => {
                self.current_settings.network.websocket_endpoint = serde_json::from_value(value)?
            }
            "apiRateLimitStrategy" => {
                self.current_settings.network.api_rate_limit_strategy =
                    serde_json::from_value(value)?
            }
            "retryAttempts" => {
                self.current_settings.network.retry_attempts = serde_json::from_value(value)?
            }
            "timeoutSeconds" => {
                self.current_settings.network.timeout_seconds = serde_json::from_value(value)?
            }
            "offlineMode" => {
                self.current_settings.network.offline_mode = serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "network".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_automation_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "dcaDefaultFrequencyHours" => {
                self.current_settings.automation.dca_default_frequency_hours =
                    serde_json::from_value(value)?
            }
            "copyTradeDelaySeconds" => {
                self.current_settings.automation.copy_trade_delay_seconds =
                    serde_json::from_value(value)?
            }
            "autoRebalanceThresholdPercent" => {
                self.current_settings
                    .automation
                    .auto_rebalance_threshold_percent = serde_json::from_value(value)?
            }
            "botExecutionLimits" => {
                self.current_settings.automation.bot_execution_limits =
                    serde_json::from_value(value)?
            }
            "safetyOverrideControls" => {
                self.current_settings.automation.safety_override_controls =
                    serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "automation".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    fn update_developer_setting(
        &mut self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), SettingsError> {
        match key {
            "debugMode" => {
                self.current_settings.developer.debug_mode = serde_json::from_value(value)?
            }
            "consoleLogLevel" => {
                self.current_settings.developer.console_log_level = serde_json::from_value(value)?
            }
            "experimentalFeatures" => {
                self.current_settings.developer.experimental_features =
                    serde_json::from_value(value)?
            }
            "apiMockMode" => {
                self.current_settings.developer.api_mock_mode = serde_json::from_value(value)?
            }
            "customApiEndpoints" => {
                self.current_settings.developer.custom_api_endpoints =
                    serde_json::from_value(value)?
            }
            "webhookUrls" => {
                self.current_settings.developer.webhook_urls = serde_json::from_value(value)?
            }
            "customIndicatorsPath" => {
                self.current_settings.developer.custom_indicators_path =
                    serde_json::from_value(value)?
            }
            _ => {
                return Err(SettingsError::SettingNotFound {
                    category: "developer".to_string(),
                    key: key.to_string(),
                })
            }
        }
        Ok(())
    }

    pub fn bulk_update_settings(
        &mut self,
        changes: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Result<(), SettingsError> {
        // Create a backup of current settings
        let backup = self.current_settings.clone();

        // Apply all changes
        for (category, settings) in changes {
            for (key, value) in settings {
                if let Err(e) = self.update_setting(category.clone(), key.clone(), value) {
                    // Rollback on error
                    self.current_settings = backup;
                    self.save_settings()?;
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn reset_settings(&mut self, category: Option<String>) -> Result<(), SettingsError> {
        if let Some(cat) = category {
            // Reset specific category
            match cat.as_str() {
                "trading" => self.current_settings.trading = TradingSettings::default(),
                "aiAssistant" => {
                    self.current_settings.ai_assistant = AIAssistantSettings::default()
                }
                "voice" => self.current_settings.voice = VoiceSettings::default(),
                "uiTheme" => self.current_settings.ui_theme = UIThemeSettings::default(),
                "alerts" => self.current_settings.alerts = AlertSettings::default(),
                "performance" => self.current_settings.performance = PerformanceSettings::default(),
                "security" => self.current_settings.security = SecuritySettings::default(),
                "dataPrivacy" => {
                    self.current_settings.data_privacy = DataPrivacySettings::default()
                }
                "network" => self.current_settings.network = NetworkSettings::default(),
                "automation" => self.current_settings.automation = AutomationSettings::default(),
                "developer" => self.current_settings.developer = DeveloperSettings::default(),
                _ => {
                    return Err(SettingsError::SettingNotFound {
                        category: cat,
                        key: String::new(),
                    })
                }
            }
        } else {
            // Reset all settings
            self.current_settings = UniversalSettings::default();
        }

        self.save_settings()?;
        Ok(())
    }

    pub fn export_settings(
        &self,
        profile_name: Option<String>,
    ) -> Result<SettingsExport, SettingsError> {
        Ok(SettingsExport {
            version: SETTINGS_SCHEMA_VERSION,
            exported_at: Utc::now(),
            profile_name,
            settings: self.current_settings.clone(),
        })
    }

    pub fn import_settings(&mut self, export: SettingsExport) -> Result<(), SettingsError> {
        // Check version compatibility
        if export.version > SETTINGS_SCHEMA_VERSION {
            return Err(SettingsError::InvalidVersion {
                expected: SETTINGS_SCHEMA_VERSION,
                actual: export.version,
            });
        }

        // Validate imported settings
        let temp_settings = export.settings.clone();
        let backup = self.current_settings.clone();
        self.current_settings = temp_settings;

        if let Err(e) = self.validate_settings() {
            // Rollback on validation error
            self.current_settings = backup;
            return Err(e);
        }

        self.save_settings()?;
        Ok(())
    }

    pub fn create_profile(
        &mut self,
        name: String,
        description: Option<String>,
    ) -> Result<(), SettingsError> {
        let profile = SettingsProfile {
            name: name.clone(),
            description,
            settings: self.current_settings.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.profiles.insert(name, profile);
        self.save_profiles()?;
        Ok(())
    }

    pub fn load_profile(&mut self, name: String) -> Result<(), SettingsError> {
        let profile = self
            .profiles
            .get(&name)
            .ok_or_else(|| SettingsError::ProfileNotFound(name.clone()))?
            .clone();

        self.current_settings = profile.settings;
        self.save_settings()?;
        Ok(())
    }

    pub fn delete_profile(&mut self, name: String) -> Result<(), SettingsError> {
        if self.profiles.remove(&name).is_none() {
            return Err(SettingsError::ProfileNotFound(name));
        }
        self.save_profiles()?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Vec<SettingsProfile> {
        self.profiles.values().cloned().collect()
    }

    pub fn get_change_history(&self) -> Vec<SettingsChange> {
        self.change_history.clone()
    }

    fn validate_settings(&self) -> Result<(), SettingsError> {
        let s = &self.current_settings;

        // Validate trading settings
        if s.trading.default_slippage < 0.0 || s.trading.default_slippage > 100.0 {
            return Err(SettingsError::Validation(
                "Trading slippage must be between 0 and 100".to_string(),
            ));
        }

        if s.trading.max_position_size_percent <= 0.0 || s.trading.max_position_size_percent > 100.0
        {
            return Err(SettingsError::Validation(
                "Max position size must be between 0 and 100".to_string(),
            ));
        }

        // Validate AI settings
        if s.ai_assistant.temperature < 0.0 || s.ai_assistant.temperature > 2.0 {
            return Err(SettingsError::Validation(
                "AI temperature must be between 0.0 and 2.0".to_string(),
            ));
        }

        // Validate voice settings
        if s.voice.speech_rate < 0.5 || s.voice.speech_rate > 2.0 {
            return Err(SettingsError::Validation(
                "Speech rate must be between 0.5x and 2.0x".to_string(),
            ));
        }

        if s.voice.audio_alerts_volume < 0.0 || s.voice.audio_alerts_volume > 1.0 {
            return Err(SettingsError::Validation(
                "Audio volume must be between 0.0 and 1.0".to_string(),
            ));
        }

        // Validate UI settings
        if s.ui_theme.gradient_strength < 0.0 || s.ui_theme.gradient_strength > 1.0 {
            return Err(SettingsError::Validation(
                "Gradient strength must be between 0.0 and 1.0".to_string(),
            ));
        }

        if s.ui_theme.font_size_multiplier < 0.5 || s.ui_theme.font_size_multiplier > 2.0 {
            return Err(SettingsError::Validation(
                "Font size multiplier must be between 0.5 and 2.0".to_string(),
            ));
        }

        // Validate security settings
        if s.security.session_timeout_minutes == 0 {
            return Err(SettingsError::Validation(
                "Session timeout must be greater than 0".to_string(),
            ));
        }

        if s.security.auto_lock_on_idle && s.security.auto_lock_minutes == 0 {
            return Err(SettingsError::Validation(
                "Auto-lock minutes must be greater than 0 when enabled".to_string(),
            ));
        }

        // Validate network settings
        if s.network.solana_rpc_endpoint.is_empty() {
            return Err(SettingsError::Validation(
                "RPC endpoint cannot be empty".to_string(),
            ));
        }

        if s.network.retry_attempts == 0 {
            return Err(SettingsError::Validation(
                "Retry attempts must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    pub fn get_template(template_type: &str) -> Result<UniversalSettings, SettingsError> {
        let mut settings = UniversalSettings::default();

        match template_type {
            "day_trader" => {
                settings.trading.default_slippage = 2.0;
                settings.trading.gas_priority = GasPriority::Fast;
                settings.trading.paper_trading_mode = false;
                settings.performance.chart_update_frequency_ms = 500;
                settings.performance.prefetch_aggressiveness = PrefetchLevel::High;
                settings.alerts.batch_alerts = false;
            }
            "whale_watcher" => {
                settings.trading.default_slippage = 0.5;
                settings.trading.gas_priority = GasPriority::Custom(500000);
                settings.trading.mev_protection = true;
                settings.trading.private_rpc_enabled = true;
                settings.security.hardware_wallet_preferred = true;
                settings.security.transaction_confirmation_requirements =
                    TransactionConfirmation::Always;
            }
            "defi_farmer" => {
                settings.trading.auto_confirm_below = Some(50.0);
                settings.automation.dca_default_frequency_hours = 12;
                settings.automation.auto_rebalance_threshold_percent = 5.0;
                settings.performance.prefetch_aggressiveness = PrefetchLevel::High;
            }
            "conservative" => {
                settings.trading.default_slippage = 0.5;
                settings.trading.paper_trading_mode = true;
                settings.trading.mev_protection = true;
                settings.security.two_fa_requirements = TwoFARequirements::Both;
                settings.security.hardware_wallet_preferred = true;
                settings.security.auto_lock_minutes = 5;
            }
            "balanced" => {
                // Already the default
            }
            "performance" => {
                settings.performance.chart_update_frequency_ms = 5000;
                settings.performance.data_cache_ttl_seconds = 600;
                settings.performance.prefetch_aggressiveness = PrefetchLevel::Low;
                settings.ui_theme.animation_speed = AnimationSpeed::Fast;
            }
            _ => {
                return Err(SettingsError::Validation(format!(
                    "Unknown template type: {}",
                    template_type
                )))
            }
        }

        Ok(settings)
    }
}
