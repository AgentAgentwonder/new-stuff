use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedDrawingManager = Arc<RwLock<DrawingManager>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingPoint {
    pub x: f64,
    pub y: f64,
    pub timestamp: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingStyle {
    pub stroke_color: String,
    pub stroke_width: f64,
    pub fill_color: Option<String>,
    pub opacity: f64,
    pub line_style: Option<String>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub background: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingObject {
    pub id: String,
    pub user_id: String,
    pub symbol: String,
    pub tool: String,
    pub points: Vec<DrawingPoint>,
    pub style: DrawingStyle,
    pub locked: bool,
    pub hidden: bool,
    pub template_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub shared_with: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tool: String,
    pub style: DrawingStyle,
    pub default_points: Vec<DrawingPoint>,
    pub metadata: Option<serde_json::Value>,
}

pub struct DrawingManager {
    drawings_path: PathBuf,
    templates_path: PathBuf,
}

impl DrawingManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let drawings_dir = app_data_dir.join("drawings");
        let _ = fs::create_dir_all(&drawings_dir);

        Self {
            drawings_path: drawings_dir.join("drawings.json"),
            templates_path: drawings_dir.join("templates.json"),
        }
    }

    fn read_json<T: for<'de> Deserialize<'de>>(&self, path: &PathBuf) -> Result<T, String> {
        if !path.exists() {
            return serde_json::from_str("{}").map_err(|e| e.to_string());
        }
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }

    fn write_json<T: Serialize>(&self, path: &PathBuf, data: &T) -> Result<(), String> {
        let serialized = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        fs::write(path, serialized).map_err(|e| e.to_string())
    }

    pub fn list_drawings(&self, symbol: &str) -> Result<Vec<DrawingObject>, String> {
        let all_drawings: Vec<DrawingObject> =
            self.read_json(&self.drawings_path).unwrap_or_default();
        Ok(all_drawings
            .into_iter()
            .filter(|d| d.symbol == symbol)
            .collect::<Vec<_>>())
    }

    pub fn save_drawings(&self, symbol: &str, drawings: &[DrawingObject]) -> Result<(), String> {
        // Load all drawings, replace those matching symbol
        let mut all_drawings: Vec<DrawingObject> =
            self.read_json(&self.drawings_path).unwrap_or_default();
        all_drawings.retain(|d| d.symbol != symbol);
        all_drawings.extend_from_slice(drawings);
        self.write_json(&self.drawings_path, &all_drawings)
    }

    pub fn sync_drawings(&self, symbol: &str) -> Result<Vec<DrawingObject>, String> {
        // For now, sync is equivalent to list, but may include remote sync in future
        self.list_drawings(symbol)
    }

    pub fn list_templates(&self) -> Result<Vec<DrawingTemplate>, String> {
        self.read_json(&self.templates_path)
    }

    pub fn save_templates(&self, templates: &[DrawingTemplate]) -> Result<(), String> {
        self.write_json(&self.templates_path, &templates)
    }
}

#[tauri::command]
pub async fn drawing_list(
    symbol: String,
    manager: tauri::State<'_, SharedDrawingManager>,
) -> Result<Vec<DrawingObject>, String> {
    let mgr = manager.read().await;
    mgr.list_drawings(&symbol)
}

#[tauri::command]
pub async fn drawing_save(
    symbol: String,
    drawings: Vec<DrawingObject>,
    manager: tauri::State<'_, SharedDrawingManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.save_drawings(&symbol, &drawings)
}

#[tauri::command]
pub async fn drawing_sync(
    symbol: String,
    manager: tauri::State<'_, SharedDrawingManager>,
) -> Result<Vec<DrawingObject>, String> {
    let mgr = manager.read().await;
    mgr.sync_drawings(&symbol)
}

#[tauri::command]
pub async fn drawing_list_templates(
    manager: tauri::State<'_, SharedDrawingManager>,
) -> Result<Vec<DrawingTemplate>, String> {
    let mgr = manager.read().await;
    mgr.list_templates()
}

#[tauri::command]
pub async fn drawing_save_templates(
    templates: Vec<DrawingTemplate>,
    manager: tauri::State<'_, SharedDrawingManager>,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.save_templates(&templates)
}
