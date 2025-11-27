use super::settings_manager::{
    SettingsChange, SettingsExport, SettingsManager, SettingsProfile, SharedSettingsManager,
};
use super::settings_schema::{SettingMetadata, SettingType, UniversalSettings};
use serde_json::json;
use std::collections::HashMap;

#[tauri::command]
pub async fn get_all_settings(
    settings: tauri::State<'_, SharedSettingsManager>,
) -> Result<UniversalSettings, String> {
    let manager = settings.read().await;
    Ok(manager.get_all_settings())
}

#[tauri::command]
pub async fn update_setting(
    settings: tauri::State<'_, SharedSettingsManager>,
    category: String,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let mut manager = settings.write().await;
    manager
        .update_setting(category, key, value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_update_settings(
    settings: tauri::State<'_, SharedSettingsManager>,
    changes: HashMap<String, HashMap<String, serde_json::Value>>,
) -> Result<(), String> {
    let mut manager = settings.write().await;
    manager
        .bulk_update_settings(changes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_config_settings(
    settings: tauri::State<'_, SharedSettingsManager>,
    category: Option<String>,
) -> Result<(), String> {
    let mut manager = settings.write().await;
    manager.reset_settings(category).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_config_settings(
    settings: tauri::State<'_, SharedSettingsManager>,
    profile_name: Option<String>,
) -> Result<SettingsExport, String> {
    let manager = settings.read().await;
    manager
        .export_settings(profile_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_config_settings(
    settings: tauri::State<'_, SharedSettingsManager>,
    export: SettingsExport,
) -> Result<(), String> {
    let mut manager = settings.write().await;
    manager.import_settings(export).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting_schema() -> Result<Vec<SettingMetadata>, String> {
    Ok(generate_settings_schema())
}

#[tauri::command]
pub async fn create_settings_profile(
    settings: tauri::State<'_, SharedSettingsManager>,
    name: String,
    description: Option<String>,
) -> Result<(), String> {
    let mut manager = settings.write().await;
    manager
        .create_profile(name, description)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_settings_profile(
    settings: tauri::State<'_, SharedSettingsManager>,
    name: String,
) -> Result<(), String> {
    let mut manager = settings.write().await;
    manager.load_profile(name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_settings_profile(
    settings: tauri::State<'_, SharedSettingsManager>,
    name: String,
) -> Result<(), String> {
    let mut manager = settings.write().await;
    manager.delete_profile(name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_settings_profiles(
    settings: tauri::State<'_, SharedSettingsManager>,
) -> Result<Vec<SettingsProfile>, String> {
    let manager = settings.read().await;
    Ok(manager.list_profiles())
}

#[tauri::command]
pub async fn get_settings_change_history(
    settings: tauri::State<'_, SharedSettingsManager>,
) -> Result<Vec<SettingsChange>, String> {
    let manager = settings.read().await;
    Ok(manager.get_change_history())
}

#[tauri::command]
pub async fn get_config_settings_template(
    template_type: String,
) -> Result<UniversalSettings, String> {
    SettingsManager::get_template(&template_type).map_err(|e| e.to_string())
}

/// Generate metadata for all settings for UI generation
fn generate_settings_schema() -> Vec<SettingMetadata> {
    vec![
        // Trading Settings
        SettingMetadata {
            key: "defaultSlippage".to_string(),
            category: "trading".to_string(),
            label: "Default Slippage".to_string(),
            description: "Maximum slippage tolerance for trades (%)".to_string(),
            setting_type: SettingType::Slider {
                min: 0.1,
                max: 10.0,
                step: 0.1,
            },
            default_value: json!(1.0),
            constraints: None,
        },
        SettingMetadata {
            key: "gasPriority".to_string(),
            category: "trading".to_string(),
            label: "Gas Priority".to_string(),
            description: "Priority fee level for transactions".to_string(),
            setting_type: SettingType::Select {
                options: vec!["slow".to_string(), "medium".to_string(), "fast".to_string()],
            },
            default_value: json!("medium"),
            constraints: None,
        },
        SettingMetadata {
            key: "defaultOrderType".to_string(),
            category: "trading".to_string(),
            label: "Default Order Type".to_string(),
            description: "Default order type for new trades".to_string(),
            setting_type: SettingType::Select {
                options: vec!["market".to_string(), "limit".to_string()],
            },
            default_value: json!("market"),
            constraints: None,
        },
        SettingMetadata {
            key: "autoConfirmBelow".to_string(),
            category: "trading".to_string(),
            label: "Auto-Confirm Below ($)".to_string(),
            description: "Automatically confirm trades below this dollar amount".to_string(),
            setting_type: SettingType::Number {
                min: 0.0,
                max: 10000.0,
                step: 1.0,
            },
            default_value: json!(10.0),
            constraints: None,
        },
        SettingMetadata {
            key: "paperTradingMode".to_string(),
            category: "trading".to_string(),
            label: "Paper Trading Mode".to_string(),
            description: "Enable paper trading (simulated trades)".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(true),
            constraints: None,
        },
        SettingMetadata {
            key: "mevProtection".to_string(),
            category: "trading".to_string(),
            label: "MEV Protection".to_string(),
            description: "Enable MEV (Maximal Extractable Value) protection".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(true),
            constraints: None,
        },
        // AI Assistant Settings
        SettingMetadata {
            key: "provider".to_string(),
            category: "aiAssistant".to_string(),
            label: "AI Provider".to_string(),
            description: "AI service provider".to_string(),
            setting_type: SettingType::Select {
                options: vec!["claude".to_string(), "gpt-4".to_string()],
            },
            default_value: json!("claude"),
            constraints: None,
        },
        SettingMetadata {
            key: "model".to_string(),
            category: "aiAssistant".to_string(),
            label: "AI Model".to_string(),
            description: "Specific AI model to use".to_string(),
            setting_type: SettingType::Text { multiline: false },
            default_value: json!("claude-3-5-sonnet-20241022"),
            constraints: None,
        },
        SettingMetadata {
            key: "temperature".to_string(),
            category: "aiAssistant".to_string(),
            label: "Temperature".to_string(),
            description: "AI creativity level (0.0 = deterministic, 2.0 = very creative)"
                .to_string(),
            setting_type: SettingType::Slider {
                min: 0.0,
                max: 2.0,
                step: 0.1,
            },
            default_value: json!(0.7),
            constraints: None,
        },
        SettingMetadata {
            key: "maxTokens".to_string(),
            category: "aiAssistant".to_string(),
            label: "Max Tokens".to_string(),
            description: "Maximum tokens per AI response".to_string(),
            setting_type: SettingType::Number {
                min: 100.0,
                max: 200000.0,
                step: 100.0,
            },
            default_value: json!(4096),
            constraints: None,
        },
        SettingMetadata {
            key: "autoSuggestions".to_string(),
            category: "aiAssistant".to_string(),
            label: "Auto Suggestions".to_string(),
            description: "Enable automatic AI suggestions".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(true),
            constraints: None,
        },
        // Voice Settings
        SettingMetadata {
            key: "wakeWord".to_string(),
            category: "voice".to_string(),
            label: "Wake Word".to_string(),
            description: "Voice activation phrase".to_string(),
            setting_type: SettingType::Text { multiline: false },
            default_value: json!("Hey Eclipse"),
            constraints: None,
        },
        SettingMetadata {
            key: "language".to_string(),
            category: "voice".to_string(),
            label: "Language".to_string(),
            description: "Voice recognition language".to_string(),
            setting_type: SettingType::Select {
                options: vec![
                    "en-US".to_string(),
                    "en-GB".to_string(),
                    "es-ES".to_string(),
                    "fr-FR".to_string(),
                    "de-DE".to_string(),
                ],
            },
            default_value: json!("en-US"),
            constraints: None,
        },
        SettingMetadata {
            key: "speechRate".to_string(),
            category: "voice".to_string(),
            label: "Speech Rate".to_string(),
            description: "Text-to-speech speed multiplier".to_string(),
            setting_type: SettingType::Slider {
                min: 0.5,
                max: 2.0,
                step: 0.1,
            },
            default_value: json!(1.0),
            constraints: None,
        },
        SettingMetadata {
            key: "audioAlertsVolume".to_string(),
            category: "voice".to_string(),
            label: "Audio Alerts Volume".to_string(),
            description: "Volume level for audio alerts (0.0 - 1.0)".to_string(),
            setting_type: SettingType::Slider {
                min: 0.0,
                max: 1.0,
                step: 0.05,
            },
            default_value: json!(0.7),
            constraints: None,
        },
        // UI Theme Settings
        SettingMetadata {
            key: "lunarThemeIntensity".to_string(),
            category: "uiTheme".to_string(),
            label: "Lunar Theme Intensity".to_string(),
            description: "Visual intensity of the lunar theme".to_string(),
            setting_type: SettingType::Select {
                options: vec![
                    "subtle".to_string(),
                    "normal".to_string(),
                    "intense".to_string(),
                ],
            },
            default_value: json!("normal"),
            constraints: None,
        },
        SettingMetadata {
            key: "gradientStrength".to_string(),
            category: "uiTheme".to_string(),
            label: "Gradient Strength".to_string(),
            description: "Intensity of gradient effects".to_string(),
            setting_type: SettingType::Slider {
                min: 0.0,
                max: 1.0,
                step: 0.1,
            },
            default_value: json!(0.7),
            constraints: None,
        },
        SettingMetadata {
            key: "animationSpeed".to_string(),
            category: "uiTheme".to_string(),
            label: "Animation Speed".to_string(),
            description: "Speed of UI animations".to_string(),
            setting_type: SettingType::Select {
                options: vec![
                    "slow".to_string(),
                    "normal".to_string(),
                    "fast".to_string(),
                    "off".to_string(),
                ],
            },
            default_value: json!("normal"),
            constraints: None,
        },
        SettingMetadata {
            key: "fontSizeMultiplier".to_string(),
            category: "uiTheme".to_string(),
            label: "Font Size Multiplier".to_string(),
            description: "Scale factor for all text".to_string(),
            setting_type: SettingType::Slider {
                min: 0.5,
                max: 2.0,
                step: 0.1,
            },
            default_value: json!(1.0),
            constraints: None,
        },
        SettingMetadata {
            key: "reduceMotion".to_string(),
            category: "uiTheme".to_string(),
            label: "Reduce Motion".to_string(),
            description: "Minimize animations for accessibility".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(false),
            constraints: None,
        },
        // Alert Settings
        SettingMetadata {
            key: "cooldownSeconds".to_string(),
            category: "alerts".to_string(),
            label: "Alert Cooldown (seconds)".to_string(),
            description: "Minimum time between similar alerts".to_string(),
            setting_type: SettingType::Number {
                min: 0.0,
                max: 3600.0,
                step: 10.0,
            },
            default_value: json!(60),
            constraints: None,
        },
        SettingMetadata {
            key: "batchAlerts".to_string(),
            category: "alerts".to_string(),
            label: "Batch Alerts".to_string(),
            description: "Group multiple alerts together".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(false),
            constraints: None,
        },
        // Performance Settings
        SettingMetadata {
            key: "chartUpdateFrequencyMs".to_string(),
            category: "performance".to_string(),
            label: "Chart Update Frequency (ms)".to_string(),
            description: "How often charts refresh".to_string(),
            setting_type: SettingType::Select {
                options: vec![
                    "500".to_string(),
                    "1000".to_string(),
                    "5000".to_string(),
                    "10000".to_string(),
                ],
            },
            default_value: json!(1000),
            constraints: None,
        },
        SettingMetadata {
            key: "dataCacheTtlSeconds".to_string(),
            category: "performance".to_string(),
            label: "Cache TTL (seconds)".to_string(),
            description: "How long to cache data".to_string(),
            setting_type: SettingType::Number {
                min: 10.0,
                max: 3600.0,
                step: 10.0,
            },
            default_value: json!(300),
            constraints: None,
        },
        SettingMetadata {
            key: "gpuAcceleration".to_string(),
            category: "performance".to_string(),
            label: "GPU Acceleration".to_string(),
            description: "Use GPU for rendering when available".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(true),
            constraints: None,
        },
        // Security Settings
        SettingMetadata {
            key: "sessionTimeoutMinutes".to_string(),
            category: "security".to_string(),
            label: "Session Timeout (minutes)".to_string(),
            description: "Auto-logout after inactivity".to_string(),
            setting_type: SettingType::Number {
                min: 1.0,
                max: 1440.0,
                step: 5.0,
            },
            default_value: json!(30),
            constraints: None,
        },
        SettingMetadata {
            key: "biometricEnabled".to_string(),
            category: "security".to_string(),
            label: "Biometric Authentication".to_string(),
            description: "Enable fingerprint/face recognition".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(false),
            constraints: None,
        },
        SettingMetadata {
            key: "autoLockOnIdle".to_string(),
            category: "security".to_string(),
            label: "Auto-Lock on Idle".to_string(),
            description: "Lock app when idle".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(true),
            constraints: None,
        },
        SettingMetadata {
            key: "autoLockMinutes".to_string(),
            category: "security".to_string(),
            label: "Auto-Lock Timeout (minutes)".to_string(),
            description: "Time until auto-lock triggers".to_string(),
            setting_type: SettingType::Number {
                min: 1.0,
                max: 60.0,
                step: 1.0,
            },
            default_value: json!(15),
            constraints: None,
        },
        // Data & Privacy Settings
        SettingMetadata {
            key: "analyticsOptIn".to_string(),
            category: "dataPrivacy".to_string(),
            label: "Analytics".to_string(),
            description: "Share usage analytics".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(true),
            constraints: None,
        },
        SettingMetadata {
            key: "crashReporting".to_string(),
            category: "dataPrivacy".to_string(),
            label: "Crash Reporting".to_string(),
            description: "Send crash reports to improve stability".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(true),
            constraints: None,
        },
        SettingMetadata {
            key: "dataRetentionDays".to_string(),
            category: "dataPrivacy".to_string(),
            label: "Data Retention (days)".to_string(),
            description: "How long to keep historical data".to_string(),
            setting_type: SettingType::Select {
                options: vec![
                    "30".to_string(),
                    "60".to_string(),
                    "90".to_string(),
                    "180".to_string(),
                    "365".to_string(),
                ],
            },
            default_value: json!(90),
            constraints: None,
        },
        // Network Settings
        SettingMetadata {
            key: "solanaRpcEndpoint".to_string(),
            category: "network".to_string(),
            label: "Solana RPC Endpoint".to_string(),
            description: "Primary RPC endpoint URL".to_string(),
            setting_type: SettingType::Text { multiline: false },
            default_value: json!("https://api.mainnet-beta.solana.com"),
            constraints: None,
        },
        SettingMetadata {
            key: "retryAttempts".to_string(),
            category: "network".to_string(),
            label: "Retry Attempts".to_string(),
            description: "Number of retry attempts for failed requests".to_string(),
            setting_type: SettingType::Number {
                min: 1.0,
                max: 10.0,
                step: 1.0,
            },
            default_value: json!(3),
            constraints: None,
        },
        SettingMetadata {
            key: "timeoutSeconds".to_string(),
            category: "network".to_string(),
            label: "Request Timeout (seconds)".to_string(),
            description: "Maximum time to wait for network requests".to_string(),
            setting_type: SettingType::Number {
                min: 5.0,
                max: 120.0,
                step: 5.0,
            },
            default_value: json!(30),
            constraints: None,
        },
        // Developer Settings
        SettingMetadata {
            key: "debugMode".to_string(),
            category: "developer".to_string(),
            label: "Debug Mode".to_string(),
            description: "Enable debug logging and features".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(false),
            constraints: None,
        },
        SettingMetadata {
            key: "experimentalFeatures".to_string(),
            category: "developer".to_string(),
            label: "Experimental Features".to_string(),
            description: "Enable experimental and unstable features".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(false),
            constraints: None,
        },
        SettingMetadata {
            key: "apiMockMode".to_string(),
            category: "developer".to_string(),
            label: "API Mock Mode".to_string(),
            description: "Use mocked API responses for testing".to_string(),
            setting_type: SettingType::Boolean,
            default_value: json!(false),
            constraints: None,
        },
    ]
}
