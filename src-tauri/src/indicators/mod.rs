use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedIndicatorManager = Arc<RwLock<IndicatorManager>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub indicator_type: String,
    pub enabled: bool,
    pub panel: String,
    pub params: HashMap<String, serde_json::Value>,
    pub color: Option<String>,
    pub line_width: Option<u32>,
    pub style: Option<String>,
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorPreset {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub indicators: Vec<IndicatorConfig>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorAlert {
    pub id: String,
    pub indicator_id: String,
    pub condition: String,
    pub threshold: f64,
    pub enabled: bool,
    pub notification_channels: Vec<String>,
}

pub struct IndicatorManager {
    indicators_path: PathBuf,
    presets_path: PathBuf,
    alerts_path: PathBuf,
}

impl IndicatorManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let indicators_dir = app_data_dir.join("indicators");
        let _ = fs::create_dir_all(&indicators_dir);

        Self {
            indicators_path: indicators_dir.join("indicators.json"),
            presets_path: indicators_dir.join("presets.json"),
            alerts_path: indicators_dir.join("alerts.json"),
        }
    }

    pub fn save_indicators(&self, indicators: &[IndicatorConfig]) -> Result<(), String> {
        let json = serde_json::to_string_pretty(indicators)
            .map_err(|e| format!("Failed to serialize indicators: {}", e))?;
        fs::write(&self.indicators_path, json)
            .map_err(|e| format!("Failed to write indicators: {}", e))?;
        Ok(())
    }

    pub fn load_indicators(&self) -> Result<Vec<IndicatorConfig>, String> {
        if !self.indicators_path.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(&self.indicators_path)
            .map_err(|e| format!("Failed to read indicators: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse indicators: {}", e))
    }

    pub fn list_presets(&self) -> Result<Vec<IndicatorPreset>, String> {
        if !self.presets_path.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(&self.presets_path)
            .map_err(|e| format!("Failed to read presets: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse presets: {}", e))
    }

    pub fn save_preset(&self, preset: &IndicatorPreset) -> Result<(), String> {
        let mut presets = self.list_presets().unwrap_or_default();

        // Remove existing preset with the same ID if any
        presets.retain(|p| p.id != preset.id);
        presets.push(preset.clone());

        let json = serde_json::to_string_pretty(&presets)
            .map_err(|e| format!("Failed to serialize presets: {}", e))?;
        fs::write(&self.presets_path, json)
            .map_err(|e| format!("Failed to write presets: {}", e))?;
        Ok(())
    }

    pub fn delete_preset(&self, preset_id: &str) -> Result<(), String> {
        let mut presets = self.list_presets().unwrap_or_default();
        presets.retain(|p| p.id != preset_id);

        let json = serde_json::to_string_pretty(&presets)
            .map_err(|e| format!("Failed to serialize presets: {}", e))?;
        fs::write(&self.presets_path, json)
            .map_err(|e| format!("Failed to write presets: {}", e))?;
        Ok(())
    }

    pub fn update_preset(&self, preset: &IndicatorPreset) -> Result<(), String> {
        self.save_preset(preset)
    }

    pub fn list_alerts(&self) -> Result<Vec<IndicatorAlert>, String> {
        if !self.alerts_path.exists() {
            return Ok(Vec::new());
        }

        let data = fs::read_to_string(&self.alerts_path)
            .map_err(|e| format!("Failed to read alerts: {}", e))?;
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse alerts: {}", e))
    }

    pub fn create_alert(&self, alert: &IndicatorAlert) -> Result<(), String> {
        let mut alerts = self.list_alerts().unwrap_or_default();
        alerts.push(alert.clone());

        let json = serde_json::to_string_pretty(&alerts)
            .map_err(|e| format!("Failed to serialize alerts: {}", e))?;
        fs::write(&self.alerts_path, json).map_err(|e| format!("Failed to write alerts: {}", e))?;
        Ok(())
    }

    pub fn delete_alert(&self, alert_id: &str) -> Result<(), String> {
        let mut alerts = self.list_alerts().unwrap_or_default();
        alerts.retain(|a| a.id != alert_id);

        let json = serde_json::to_string_pretty(&alerts)
            .map_err(|e| format!("Failed to serialize alerts: {}", e))?;
        fs::write(&self.alerts_path, json).map_err(|e| format!("Failed to write alerts: {}", e))?;
        Ok(())
    }

    pub fn update_alert(&self, alert_id: &str, updated: &IndicatorAlert) -> Result<(), String> {
        let mut alerts = self.list_alerts().unwrap_or_default();
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            *alert = updated.clone();
        }

        let json = serde_json::to_string_pretty(&alerts)
            .map_err(|e| format!("Failed to serialize alerts: {}", e))?;
        fs::write(&self.alerts_path, json).map_err(|e| format!("Failed to write alerts: {}", e))?;
        Ok(())
    }
}

// Tauri commands
#[tauri::command]
pub async fn indicator_save_state(
    indicators: Vec<IndicatorConfig>,
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.save_indicators(&indicators)
}

#[tauri::command]
pub async fn indicator_load_state(
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<Vec<IndicatorConfig>, String> {
    let mgr = manager.read().await;
    mgr.load_indicators()
}

#[tauri::command]
pub async fn indicator_list_presets(
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<Vec<IndicatorPreset>, String> {
    let mgr = manager.read().await;
    mgr.list_presets()
}

#[tauri::command]
pub async fn indicator_save_preset(
    preset: IndicatorPreset,
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.save_preset(&preset)
}

#[tauri::command]
pub async fn indicator_delete_preset(
    preset_id: String,
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_preset(&preset_id)
}

#[tauri::command]
pub async fn indicator_update_preset(
    preset: IndicatorPreset,
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.update_preset(&preset)
}

#[tauri::command]
pub async fn indicator_list_alerts(
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<Vec<IndicatorAlert>, String> {
    let mgr = manager.read().await;
    mgr.list_alerts()
}

#[tauri::command]
pub async fn indicator_create_alert(
    alert: IndicatorAlert,
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.create_alert(&alert)
}

#[tauri::command]
pub async fn indicator_delete_alert(
    alert_id: String,
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_alert(&alert_id)
}

#[tauri::command]
pub async fn indicator_update_alert(
    alert_id: String,
    updates: IndicatorAlert,
    manager: tauri::State<'_, SharedIndicatorManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.update_alert(&alert_id, &updates)
}
