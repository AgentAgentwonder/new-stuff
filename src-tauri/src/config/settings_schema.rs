use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Version of the settings schema
pub const SETTINGS_SCHEMA_VERSION: u32 = 1;

/// Complete settings structure with all categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversalSettings {
    pub version: u32,
    pub trading: TradingSettings,
    pub ai_assistant: AIAssistantSettings,
    pub voice: VoiceSettings,
    pub ui_theme: UIThemeSettings,
    pub alerts: AlertSettings,
    pub performance: PerformanceSettings,
    pub security: SecuritySettings,
    pub data_privacy: DataPrivacySettings,
    pub network: NetworkSettings,
    pub automation: AutomationSettings,
    pub developer: DeveloperSettings,
}

/// Trading settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingSettings {
    pub default_slippage: f64,
    pub gas_priority: GasPriority,
    pub default_order_type: OrderType,
    pub auto_confirm_below: Option<f64>,
    pub trade_confirmation_timeout: u32,
    pub max_position_size_percent: f64,
    pub paper_trading_mode: bool,
    pub multi_wallet_behavior: MultiWalletBehavior,
    pub mev_protection: bool,
    pub jito_enabled: bool,
    pub private_rpc_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GasPriority {
    Slow,
    Medium,
    Fast,
    Custom(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MultiWalletBehavior {
    AskEachTime,
    UseFirst,
    UseLast,
    PreferHardware,
}

/// AI Assistant settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIAssistantSettings {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub context_window_size: u32,
    pub auto_suggestions: bool,
    pub pattern_learning: bool,
    pub voice_personality: VoicePersonality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    Claude,
    #[serde(rename = "gpt-4")]
    GPT4,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoicePersonality {
    Formal,
    Casual,
    Technical,
}

/// Voice settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSettings {
    pub wake_word: String,
    pub language: String,
    pub speech_rate: f32,
    pub voice_preference: String,
    pub confirmation_requirements: ConfirmationLevel,
    pub audio_alerts_volume: f32,
    pub tts_provider: String,
    pub microphone_sensitivity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfirmationLevel {
    Always,
    HighValue,
    Never,
}

/// UI/Theme settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIThemeSettings {
    pub lunar_theme_intensity: ThemeIntensity,
    pub gradient_strength: f32,
    pub animation_speed: AnimationSpeed,
    pub glass_effect_opacity: f32,
    pub corona_glow_intensity: f32,
    pub font_size_multiplier: f32,
    pub color_blindness_mode: Option<ColorBlindnessMode>,
    pub reduce_motion: bool,
    pub custom_colors: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeIntensity {
    Subtle,
    Normal,
    Intense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationSpeed {
    Slow,
    Normal,
    Fast,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorBlindnessMode {
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

/// Alert settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertSettings {
    pub default_channels: Vec<AlertChannel>,
    pub cooldown_seconds: u32,
    pub smart_filter_threshold: f64,
    pub notification_sound: String,
    pub do_not_disturb_schedule: Option<DNDSchedule>,
    pub priority_levels: bool,
    pub batch_alerts: bool,
    pub desktop_notification_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertChannel {
    Push,
    Email,
    Telegram,
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DNDSchedule {
    pub enabled: bool,
    pub start_time: String,
    pub end_time: String,
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSettings {
    pub chart_update_frequency_ms: u32,
    pub data_cache_ttl_seconds: u32,
    pub max_concurrent_requests: u32,
    pub websocket_reconnect_strategy: ReconnectStrategy,
    pub prefetch_aggressiveness: PrefetchLevel,
    pub memory_limit_mb: u32,
    pub gpu_acceleration: bool,
    pub virtual_scrolling_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReconnectStrategy {
    Immediate,
    Exponential,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrefetchLevel {
    Low,
    Medium,
    High,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettings {
    pub session_timeout_minutes: u32,
    pub two_fa_requirements: TwoFARequirements,
    pub biometric_enabled: bool,
    pub keystore_backup_frequency_days: u32,
    pub auto_lock_on_idle: bool,
    pub auto_lock_minutes: u32,
    pub transaction_confirmation_requirements: TransactionConfirmation,
    pub hardware_wallet_preferred: bool,
    pub api_key_rotation_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TwoFARequirements {
    Login,
    Trades,
    Both,
    CustomThreshold(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionConfirmation {
    Always,
    AboveThreshold(f64),
    Never,
}

/// Data & Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPrivacySettings {
    pub data_retention_days: u32,
    pub analytics_opt_in: bool,
    pub share_anonymous_usage: bool,
    pub activity_log_retention_days: u32,
    pub export_format: ExportFormat,
    pub telemetry_enabled: bool,
    pub crash_reporting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Json,
    Csv,
    Excel,
}

/// Network settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub solana_rpc_endpoint: String,
    pub rpc_fallback_endpoints: Vec<String>,
    pub websocket_endpoint: String,
    pub api_rate_limit_strategy: RateLimitStrategy,
    pub retry_attempts: u32,
    pub timeout_seconds: u32,
    pub offline_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitStrategy {
    Aggressive,
    Balanced,
    Conservative,
}

/// Automation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationSettings {
    pub dca_default_frequency_hours: u32,
    pub copy_trade_delay_seconds: u32,
    pub auto_rebalance_threshold_percent: f64,
    pub bot_execution_limits: bool,
    pub safety_override_controls: bool,
}

/// Developer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperSettings {
    pub debug_mode: bool,
    pub console_log_level: LogLevel,
    pub experimental_features: bool,
    pub api_mock_mode: bool,
    pub custom_api_endpoints: HashMap<String, String>,
    pub webhook_urls: Vec<String>,
    pub custom_indicators_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Setting metadata for UI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingMetadata {
    pub key: String,
    pub category: String,
    pub label: String,
    pub description: String,
    pub setting_type: SettingType,
    pub default_value: serde_json::Value,
    pub constraints: Option<SettingConstraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum SettingType {
    Boolean,
    Number { min: f64, max: f64, step: f64 },
    Text { multiline: bool },
    Select { options: Vec<String> },
    Slider { min: f64, max: f64, step: f64 },
    Color,
    Array { item_type: Box<SettingType> },
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingConstraints {
    pub required: bool,
    pub pattern: Option<String>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

impl Default for UniversalSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_SCHEMA_VERSION,
            trading: TradingSettings::default(),
            ai_assistant: AIAssistantSettings::default(),
            voice: VoiceSettings::default(),
            ui_theme: UIThemeSettings::default(),
            alerts: AlertSettings::default(),
            performance: PerformanceSettings::default(),
            security: SecuritySettings::default(),
            data_privacy: DataPrivacySettings::default(),
            network: NetworkSettings::default(),
            automation: AutomationSettings::default(),
            developer: DeveloperSettings::default(),
        }
    }
}

impl Default for TradingSettings {
    fn default() -> Self {
        Self {
            default_slippage: 1.0,
            gas_priority: GasPriority::Medium,
            default_order_type: OrderType::Market,
            auto_confirm_below: Some(10.0),
            trade_confirmation_timeout: 30,
            max_position_size_percent: 25.0,
            paper_trading_mode: true,
            multi_wallet_behavior: MultiWalletBehavior::AskEachTime,
            mev_protection: true,
            jito_enabled: false,
            private_rpc_enabled: false,
        }
    }
}

impl Default for AIAssistantSettings {
    fn default() -> Self {
        Self {
            provider: AIProvider::Claude,
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            context_window_size: 200000,
            auto_suggestions: true,
            pattern_learning: true,
            voice_personality: VoicePersonality::Casual,
        }
    }
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            wake_word: "Hey Eclipse".to_string(),
            language: "en-US".to_string(),
            speech_rate: 1.0,
            voice_preference: "default".to_string(),
            confirmation_requirements: ConfirmationLevel::HighValue,
            audio_alerts_volume: 0.7,
            tts_provider: "system".to_string(),
            microphone_sensitivity: 0.5,
        }
    }
}

impl Default for UIThemeSettings {
    fn default() -> Self {
        Self {
            lunar_theme_intensity: ThemeIntensity::Normal,
            gradient_strength: 0.7,
            animation_speed: AnimationSpeed::Normal,
            glass_effect_opacity: 0.1,
            corona_glow_intensity: 0.5,
            font_size_multiplier: 1.0,
            color_blindness_mode: None,
            reduce_motion: false,
            custom_colors: None,
        }
    }
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            default_channels: vec![AlertChannel::Push],
            cooldown_seconds: 60,
            smart_filter_threshold: 5.0,
            notification_sound: "default".to_string(),
            do_not_disturb_schedule: None,
            priority_levels: true,
            batch_alerts: false,
            desktop_notification_style: "modern".to_string(),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            chart_update_frequency_ms: 1000,
            data_cache_ttl_seconds: 300,
            max_concurrent_requests: 10,
            websocket_reconnect_strategy: ReconnectStrategy::Exponential,
            prefetch_aggressiveness: PrefetchLevel::Medium,
            memory_limit_mb: 512,
            gpu_acceleration: true,
            virtual_scrolling_threshold: 100,
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            session_timeout_minutes: 30,
            two_fa_requirements: TwoFARequirements::Login,
            biometric_enabled: false,
            keystore_backup_frequency_days: 7,
            auto_lock_on_idle: true,
            auto_lock_minutes: 15,
            transaction_confirmation_requirements: TransactionConfirmation::Always,
            hardware_wallet_preferred: false,
            api_key_rotation_days: 90,
        }
    }
}

impl Default for DataPrivacySettings {
    fn default() -> Self {
        Self {
            data_retention_days: 90,
            analytics_opt_in: true,
            share_anonymous_usage: true,
            activity_log_retention_days: 90,
            export_format: ExportFormat::Json,
            telemetry_enabled: true,
            crash_reporting: true,
        }
    }
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            solana_rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            rpc_fallback_endpoints: vec!["https://solana-api.projectserum.com".to_string()],
            websocket_endpoint: "wss://api.mainnet-beta.solana.com".to_string(),
            api_rate_limit_strategy: RateLimitStrategy::Balanced,
            retry_attempts: 3,
            timeout_seconds: 30,
            offline_mode: false,
        }
    }
}

impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            dca_default_frequency_hours: 24,
            copy_trade_delay_seconds: 5,
            auto_rebalance_threshold_percent: 10.0,
            bot_execution_limits: true,
            safety_override_controls: true,
        }
    }
}

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            debug_mode: false,
            console_log_level: LogLevel::Info,
            experimental_features: false,
            api_mock_mode: false,
            custom_api_endpoints: HashMap::new(),
            webhook_urls: Vec::new(),
            custom_indicators_path: None,
        }
    }
}
