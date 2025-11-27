use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

const STORAGE_FILE: &str = "themes.json";
const DEFAULT_THEME_ID: &str = "lunar-eclipse";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    pub background: String,
    pub background_secondary: String,
    pub background_tertiary: String,
    pub text: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub primary: String,
    pub primary_hover: String,
    pub primary_active: String,
    pub accent: String,
    pub accent_hover: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub border: String,
    pub border_hover: String,
    pub chart_bullish: String,
    pub chart_bearish: String,
    pub chart_neutral: String,
    pub gradient_start: String,
    pub gradient_middle: String,
    pub gradient_end: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deep_space: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eclipse_orange: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moonlight_silver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_accent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeEffects {
    pub glow_strength: String,
    pub ambience: String,
    pub glassmorphism: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub colors: ThemeColors,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<ThemeEffects>,
    pub is_custom: bool,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_for: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemePreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub best_for: String,
    pub colors: ThemeColors,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<ThemeEffects>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTheme {
    pub start_hour: u8,
    pub end_hour: u8,
    pub theme_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSettings {
    pub current_theme_id: String,
    pub custom_themes: Vec<Theme>,
    pub font_family: String,
    pub density_mode: String,
    pub adaptive_theme_enabled: bool,
    pub high_contrast_mode: bool,
    pub focus_mode_enabled: bool,
    pub market_mood_reactive: bool,
    pub respect_os_theme: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_blind_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_accent_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_themes: Option<Vec<ScheduledTheme>>,
}

pub struct ThemeEngine {
    presets: Vec<ThemePreset>,
    settings: ThemeSettings,
    cache: HashMap<String, Theme>,
    storage_path: PathBuf,
}

impl ThemeEngine {
    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        let presets = Self::load_builtin_presets();
        let storage_path = Self::resolve_storage_path(app)?;
        let settings = Self::load_settings(&storage_path).unwrap_or_else(|_| Self::default_settings());

        let cache = Self::hydrate_cache(&presets, &settings);

        Ok(Self {
            presets,
            settings,
            cache,
            storage_path,
        })
    }

    fn resolve_storage_path(app: &AppHandle) -> Result<PathBuf, String> {
        let mut dir = app
            .path()
            .app_data_dir()
            .map_err(|_| "Unable to resolve app data directory".to_string())?;

        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create app data directory: {e}"))?;
        }

        dir.push(STORAGE_FILE);
        Ok(dir)
    }

    fn load_settings(path: &PathBuf) -> Result<ThemeSettings, String> {
        if !path.exists() {
            return Err("Settings file not found".into());
        }

        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read theme settings: {e}"))?;

        let settings: ThemeSettings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse theme settings: {e}"))?;

        Ok(settings)
    }

    fn hydrate_cache(presets: &[ThemePreset], settings: &ThemeSettings) -> HashMap<String, Theme> {
        let mut cache = HashMap::new();

        for preset in presets {
            cache.insert(preset.id.clone(), Self::theme_from_preset(preset));
        }

        for custom in &settings.custom_themes {
            cache.insert(custom.id.clone(), custom.clone());
        }

        cache
    }

    fn default_settings() -> ThemeSettings {
        ThemeSettings {
            current_theme_id: DEFAULT_THEME_ID.to_string(),
            custom_themes: Vec::new(),
            font_family: "Inter".to_string(),
            density_mode: "normal".to_string(),
            adaptive_theme_enabled: false,
            high_contrast_mode: false,
            focus_mode_enabled: false,
            market_mood_reactive: false,
            respect_os_theme: true,
            color_blind_mode: None,
            custom_accent_override: None,
            scheduled_themes: None,
        }
    }

    fn save_settings(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.settings)
            .map_err(|e| format!("Failed to serialize theme settings: {e}"))?;

        fs::write(&self.storage_path, json)
            .map_err(|e| format!("Failed to persist theme settings: {e}"))
    }

    fn theme_from_preset(preset: &ThemePreset) -> Theme {
        let now = Utc::now().timestamp_millis();
        Theme {
            id: preset.id.clone(),
            name: preset.name.clone(),
            colors: preset.colors.clone(),
            effects: preset.effects.clone(),
            is_custom: false,
            created_at: now,
            updated_at: now,
            author: Some("Eclipse Market".to_string()),
            description: Some(preset.description.clone()),
            best_for: Some(preset.best_for.clone()),
        }
    }

    fn load_builtin_presets() -> Vec<ThemePreset> {
        vec![
            ThemePreset {
                id: "lunar-eclipse".into(),
                name: "Lunar Eclipse".into(),
                description: "Deep space blacks with eclipse orange accents and glassmorphism panels.".into(),
                best_for: "Night traders, immersive experience".into(),
                colors: ThemeColors {
                    background: "#0A0A0F".into(),
                    background_secondary: "#11121A".into(),
                    background_tertiary: "#181B26".into(),
                    text: "#E8EAF6".into(),
                    text_secondary: "#C5CADE".into(),
                    text_muted: "#8B92A9".into(),
                    primary: "#FF6B35".into(),
                    primary_hover: "#FF5722".into(),
                    primary_active: "#E64A19".into(),
                    accent: "#FF8C42".into(),
                    accent_hover: "#FF7A29".into(),
                    success: "#4ECDC4".into(),
                    warning: "#FFB347".into(),
                    error: "#FF6B6B".into(),
                    info: "#4A90E2".into(),
                    border: "#232838".into(),
                    border_hover: "#32394F".into(),
                    chart_bullish: "#4ECDC4".into(),
                    chart_bearish: "#FF6B6B".into(),
                    chart_neutral: "#818CF8".into(),
                    gradient_start: "#0A0A0F".into(),
                    gradient_middle: "#131426".into(),
                    gradient_end: "#1E2438".into(),
                    deep_space: Some("#050812".into()),
                    eclipse_orange: Some("#FF6B35".into()),
                    moonlight_silver: Some("#E8EAF6".into()),
                    shadow_accent: Some("#1F2433".into()),
                },
                effects: Some(ThemeEffects {
                    glow_strength: "normal".into(),
                    ambience: "immersive".into(),
                    glassmorphism: true,
                }),
            },
            ThemePreset {
                id: "professional-dark".into(),
                name: "Professional Dark".into(),
                description: "Bloomberg-inspired pure blacks with crisp typography and data-first visuals.".into(),
                best_for: "Serious traders, data-first approach".into(),
                colors: ThemeColors {
                    background: "#000000".into(),
                    background_secondary: "#090909".into(),
                    background_tertiary: "#141414".into(),
                    text: "#FFFFFF".into(),
                    text_secondary: "#CCCCCC".into(),
                    text_muted: "#888888".into(),
                    primary: "#00A3FF".into(),
                    primary_hover: "#0088D1".into(),
                    primary_active: "#006BA3".into(),
                    accent: "#FFA726".into(),
                    accent_hover: "#FB8C00".into(),
                    success: "#00E676".into(),
                    warning: "#FFD740".into(),
                    error: "#FF5252".into(),
                    info: "#40C4FF".into(),
                    border: "#1F1F1F".into(),
                    border_hover: "#2E2E2E".into(),
                    chart_bullish: "#00E676".into(),
                    chart_bearish: "#FF5252".into(),
                    chart_neutral: "#40C4FF".into(),
                    gradient_start: "#000000".into(),
                    gradient_middle: "#090909".into(),
                    gradient_end: "#141414".into(),
                    deep_space: None,
                    eclipse_orange: None,
                    moonlight_silver: None,
                    shadow_accent: None,
                },
                effects: Some(ThemeEffects {
                    glow_strength: "none".into(),
                    ambience: "minimal".into(),
                    glassmorphism: false,
                }),
            },
            ThemePreset {
                id: "light-professional".into(),
                name: "Light Professional".into(),
                description: "Clean whites with subtle shadows, pastel accents, and daylight-optimized readability.".into(),
                best_for: "Day traders, bright environments".into(),
                colors: ThemeColors {
                    background: "#FFFFFF".into(),
                    background_secondary: "#F7F7F9".into(),
                    background_tertiary: "#ECEEF3".into(),
                    text: "#1C1E21".into(),
                    text_secondary: "#3A3D42".into(),
                    text_muted: "#6E737C".into(),
                    primary: "#0A84FF".into(),
                    primary_hover: "#0060DF".into(),
                    primary_active: "#0047B3".into(),
                    accent: "#FF7B7B".into(),
                    accent_hover: "#FF5C5C".into(),
                    success: "#34C759".into(),
                    warning: "#FFC300".into(),
                    error: "#FF3B30".into(),
                    info: "#5AC8FA".into(),
                    border: "#D0D5DD".into(),
                    border_hover: "#B0B7C3".into(),
                    chart_bullish: "#34C759".into(),
                    chart_bearish: "#FF3B30".into(),
                    chart_neutral: "#0A84FF".into(),
                    gradient_start: "#FFFFFF".into(),
                    gradient_middle: "#F7F7F9".into(),
                    gradient_end: "#E1E5ED".into(),
                    deep_space: None,
                    eclipse_orange: None,
                    moonlight_silver: None,
                    shadow_accent: None,
                },
                effects: Some(ThemeEffects {
                    glow_strength: "subtle".into(),
                    ambience: "minimal".into(),
                    glassmorphism: false,
                }),
            },
            ThemePreset {
                id: "arctic-blue".into(),
                name: "Arctic Blue".into(),
                description: "Cool blues and professional gradients reminiscent of traditional finance terminals.".into(),
                best_for: "Conservative traders, institutional look".into(),
                colors: ThemeColors {
                    background: "#0F172A".into(),
                    background_secondary: "#16213C".into(),
                    background_tertiary: "#1E2C4F".into(),
                    text: "#F8FAFC".into(),
                    text_secondary: "#CBD5E1".into(),
                    text_muted: "#94A3B8".into(),
                    primary: "#3B82F6".into(),
                    primary_hover: "#2563EB".into(),
                    primary_active: "#1D4ED8".into(),
                    accent: "#06B6D4".into(),
                    accent_hover: "#0E7490".into(),
                    success: "#10B981".into(),
                    warning: "#F59E0B".into(),
                    error: "#EF4444".into(),
                    info: "#60A5FA".into(),
                    border: "#233554".into(),
                    border_hover: "#314265".into(),
                    chart_bullish: "#10B981".into(),
                    chart_bearish: "#EF4444".into(),
                    chart_neutral: "#38BDF8".into(),
                    gradient_start: "#0F172A".into(),
                    gradient_middle: "#1E3A5F".into(),
                    gradient_end: "#2563EB".into(),
                    deep_space: None,
                    eclipse_orange: None,
                    moonlight_silver: None,
                    shadow_accent: None,
                },
                effects: Some(ThemeEffects {
                    glow_strength: "subtle".into(),
                    ambience: "balanced".into(),
                    glassmorphism: true,
                }),
            },
            ThemePreset {
                id: "emerald-wealth".into(),
                name: "Emerald Wealth".into(),
                description: "Deep greens with gold accents conveying prosperity and long-term stability.".into(),
                best_for: "Long-term investors, positive mindset".into(),
                colors: ThemeColors {
                    background: "#064E3B".into(),
                    background_secondary: "#075E46".into(),
                    background_tertiary: "#047857".into(),
                    text: "#ECFDF5".into(),
                    text_secondary: "#D1FAE5".into(),
                    text_muted: "#A7F3D0".into(),
                    primary: "#10B981".into(),
                    primary_hover: "#059669".into(),
                    primary_active: "#047857".into(),
                    accent: "#FBBF24".into(),
                    accent_hover: "#F59E0B".into(),
                    success: "#34D399".into(),
                    warning: "#FCD34D".into(),
                    error: "#F87171".into(),
                    info: "#5EEAD4".into(),
                    border: "#0F9F6E".into(),
                    border_hover: "#34D399".into(),
                    chart_bullish: "#34D399".into(),
                    chart_bearish: "#F87171".into(),
                    chart_neutral: "#FBBF24".into(),
                    gradient_start: "#064E3B".into(),
                    gradient_middle: "#047857".into(),
                    gradient_end: "#FBBF24".into(),
                    deep_space: None,
                    eclipse_orange: None,
                    moonlight_silver: None,
                    shadow_accent: None,
                },
                effects: Some(ThemeEffects {
                    glow_strength: "normal".into(),
                    ambience: "balanced".into(),
                    glassmorphism: true,
                }),
            },
            ThemePreset {
                id: "midnight-purple".into(),
                name: "Midnight Purple".into(),
                description: "Tech-forward deep purples with neon cyan accents for modern traders.".into(),
                best_for: "Tech-savvy traders, modern aesthetic".into(),
                colors: ThemeColors {
                    background: "#2E1065".into(),
                    background_secondary: "#3B1591".into(),
                    background_tertiary: "#4C1D95".into(),
                    text: "#F5F3FF".into(),
                    text_secondary: "#DDD6FE".into(),
                    text_muted: "#C4B5FD".into(),
                    primary: "#8B5CF6".into(),
                    primary_hover: "#7C3AED".into(),
                    primary_active: "#6D28D9".into(),
                    accent: "#06B6D4".into(),
                    accent_hover: "#0EA5E9".into(),
                    success: "#10B981".into(),
                    warning: "#F59E0B".into(),
                    error: "#EF4444".into(),
                    info: "#22D3EE".into(),
                    border: "#5B21B6".into(),
                    border_hover: "#7C3AED".into(),
                    chart_bullish: "#10B981".into(),
                    chart_bearish: "#EF4444".into(),
                    chart_neutral: "#06B6D4".into(),
                    gradient_start: "#2E1065".into(),
                    gradient_middle: "#5B21B6".into(),
                    gradient_end: "#06B6D4".into(),
                    deep_space: None,
                    eclipse_orange: None,
                    moonlight_silver: None,
                    shadow_accent: None,
                },
                effects: Some(ThemeEffects {
                    glow_strength: "strong".into(),
                    ambience: "immersive".into(),
                    glassmorphism: true,
                }),
            },
            ThemePreset {
                id: "monochrome-pro".into(),
                name: "Monochrome Pro".into(),
                description: "Ultra-minimal monochrome palette with maximum data density and speed.".into(),
                best_for: "Professional traders, focus mode".into(),
                colors: ThemeColors {
                    background: "#000000".into(),
                    background_secondary: "#0D0D0D".into(),
                    background_tertiary: "#1A1A1A".into(),
                    text: "#FFFFFF".into(),
                    text_secondary: "#E6E6E6".into(),
                    text_muted: "#999999".into(),
                    primary: "#FFFFFF".into(),
                    primary_hover: "#E6E6E6".into(),
                    primary_active: "#CCCCCC".into(),
                    accent: "#FFFFFF".into(),
                    accent_hover: "#E6E6E6".into(),
                    success: "#00FF00".into(),
                    warning: "#FFFFFF".into(),
                    error: "#FF0000".into(),
                    info: "#FFFFFF".into(),
                    border: "#333333".into(),
                    border_hover: "#4D4D4D".into(),
                    chart_bullish: "#00FF00".into(),
                    chart_bearish: "#FF0000".into(),
                    chart_neutral: "#FFFFFF".into(),
                    gradient_start: "#000000".into(),
                    gradient_middle: "#0D0D0D".into(),
                    gradient_end: "#1A1A1A".into(),
                    deep_space: None,
                    eclipse_orange: None,
                    moonlight_silver: None,
                    shadow_accent: None,
                },
                effects: Some(ThemeEffects {
                    glow_strength: "none".into(),
                    ambience: "minimal".into(),
                    glassmorphism: false,
                }),
            },
        ]
    }

    pub fn get_presets(&self) -> Vec<ThemePreset> {
        self.presets.clone()
    }

    pub fn get_settings(&self) -> ThemeSettings {
        self.settings.clone()
    }

    pub fn current_theme(&self) -> Option<Theme> {
        self.cache.get(&self.settings.current_theme_id).cloned()
    }

    pub fn set_current_theme(&mut self, theme_id: &str) -> Result<Theme, String> {
        if let Some(theme) = self.cache.get(theme_id) {
            self.settings.current_theme_id = theme_id.to_string();
            self.settings.high_contrast_mode = theme_id == "monochrome-pro";
            self.save_settings()?;
            Ok(theme.clone())
        } else {
            Err(format!("Theme {theme_id} not found"))
        }
    }

    pub fn update_settings(&mut self, settings: ThemeSettings) -> Result<(), String> {
        self.settings = settings;
        self.save_settings()
    }

    pub fn save_custom_theme(&mut self, mut theme: Theme) -> Result<(), String> {
        self.validate_theme(&theme)?;

        if self.settings.custom_themes.iter().any(|t| t.id == theme.id) {
            theme.updated_at = Utc::now().timestamp_millis();
            if let Some(existing) = self
                .settings
                .custom_themes
                .iter_mut()
                .find(|t| t.id == theme.id)
            {
                *existing = theme.clone();
            }
        } else {
            theme.created_at = Utc::now().timestamp_millis();
            theme.updated_at = theme.created_at;
            self.settings.custom_themes.push(theme.clone());
        }

        self.cache.insert(theme.id.clone(), theme);
        self.save_settings()
    }

    pub fn delete_custom_theme(&mut self, theme_id: &str) -> Result<(), String> {
        self.settings.custom_themes.retain(|t| t.id != theme_id);
        self.cache.remove(theme_id);

        if self.settings.current_theme_id == theme_id {
            self.settings.current_theme_id = DEFAULT_THEME_ID.to_string();
        }

        self.save_settings()
    }

    pub fn export_theme(&self, theme_id: &str) -> Result<String, String> {
        let theme = self
            .cache
            .get(theme_id)
            .ok_or_else(|| format!("Theme {theme_id} not found"))?;

        serde_json::to_string_pretty(theme).map_err(|e| format!("Failed to serialize theme: {e}"))
    }

    pub fn import_theme(&mut self, json: &str) -> Result<Theme, String> {
        let mut theme: Theme =
            serde_json::from_str(json).map_err(|e| format!("Failed to parse theme: {e}"))?;

        self.validate_theme(&theme)?;

        if !theme.is_custom {
            theme.is_custom = true;
        }

        if theme.id.is_empty() {
            theme.id = format!("custom-{}", Utc::now().timestamp_millis());
        }

        self.save_custom_theme(theme.clone())?;
        Ok(theme)
    }

    pub fn validate_theme(&self, theme: &Theme) -> Result<(), String> {
        if theme.name.trim().is_empty() {
            return Err("Theme name cannot be empty".into());
        }

        let colors = &theme.colors;
        Self::validate_color(&colors.background)?;
        Self::validate_color(&colors.text)?;
        Self::validate_color(&colors.primary)?;
        Self::validate_color(&colors.accent)?;

        Ok(())
    }

    fn validate_color(value: &str) -> Result<(), String> {
        let sanitized = value.trim();
        if !sanitized.starts_with('#') {
            return Err(format!("Color {value} must start with #"));
        }

        let length_ok = sanitized.len() == 7 || sanitized.len() == 4;
        if !length_ok {
            return Err(format!("Color {value} must be 3 or 6 hex characters"));
        }

        if sanitized.chars().skip(1).all(|c| c.is_ascii_hexdigit()) {
            Ok(())
        } else {
            Err(format!("Color {value} must be valid hexadecimal"))
        }
    }
}

impl Default for ThemeEngine {
    fn default() -> Self {
        Self {
            presets: Self::load_builtin_presets(),
            settings: Self::default_settings(),
            cache: HashMap::new(),
            storage_path: PathBuf::new(),
        }
    }
}

pub type SharedThemeEngine = Arc<Mutex<ThemeEngine>>;

#[tauri::command]
pub async fn theme_get_presets(
    engine: State<'_, SharedThemeEngine>,
) -> Result<Vec<ThemePreset>, String> {
    let engine = engine.lock().unwrap();
    Ok(engine.get_presets())
}

#[tauri::command]
pub async fn theme_get_settings(
    engine: State<'_, SharedThemeEngine>,
) -> Result<ThemeSettings, String> {
    let engine = engine.lock().unwrap();
    Ok(engine.get_settings())
}

#[tauri::command]
pub async fn theme_update_settings(
    settings: ThemeSettings,
    engine: State<'_, SharedThemeEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().unwrap();
    engine.update_settings(settings)
}

#[tauri::command]
pub async fn theme_save_custom(
    theme: Theme,
    engine: State<'_, SharedThemeEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().unwrap();
    engine.save_custom_theme(theme)
}

#[tauri::command]
pub async fn theme_delete_custom(
    theme_id: String,
    engine: State<'_, SharedThemeEngine>,
) -> Result<(), String> {
    let mut engine = engine.lock().unwrap();
    engine.delete_custom_theme(&theme_id)
}

#[tauri::command]
pub async fn theme_export(
    theme_id: String,
    engine: State<'_, SharedThemeEngine>,
) -> Result<String, String> {
    let engine = engine.lock().unwrap();
    engine.export_theme(&theme_id)
}

#[tauri::command]
pub async fn theme_import(
    json: String,
    engine: State<'_, SharedThemeEngine>,
) -> Result<Theme, String> {
    let mut engine = engine.lock().unwrap();
    engine.import_theme(&json)
}

#[tauri::command]
pub async fn theme_get_os_preference(app: AppHandle) -> Result<String, String> {
    // Note: Tauri v2 removed the system API for dark mode detection
    // For now, default to dark theme to match app aesthetic
    // TODO: Implement OS-specific detection if needed
    #[cfg(target_os = "macos")]
    {
        Ok("dark".into())
    }

    #[cfg(target_os = "windows")]
    {
        Ok("dark".into())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux theming varies, default to dark to match app aesthetic
        Ok("dark".into())
    }
}
