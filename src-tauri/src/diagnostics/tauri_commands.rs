use super::engine::DiagnosticsEngine;
use super::types::*;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;

pub type SharedDiagnosticsEngine = Arc<RwLock<DiagnosticsEngine>>;

#[tauri::command]
pub async fn run_diagnostics(
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<DiagnosticsReport, String> {
    let mut engine = engine.write().await;
    Ok(engine.run_full_diagnostics().await)
}

#[tauri::command]
pub async fn get_health_report(
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<DiagnosticsReport, String> {
    let mut engine = engine.write().await;
    if let Some(report) = engine.get_last_report() {
        Ok(report)
    } else {
        Ok(engine.run_full_diagnostics().await)
    }
}

#[tauri::command]
pub async fn auto_repair_issue(
    issue: DiagnosticIssue,
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<AutoRepairResult, String> {
    let mut engine = engine.write().await;
    engine.auto_repair(&issue).await
}

#[tauri::command]
pub async fn auto_repair(
    issues: Vec<DiagnosticIssue>,
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<Vec<AutoRepairResult>, String> {
    let mut engine = engine.write().await;
    Ok(engine.auto_repair_all(issues).await)
}

#[tauri::command]
pub async fn verify_integrity(
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<bool, String> {
    let engine = engine.read().await;
    engine.verify_integrity()
}

#[tauri::command]
pub async fn manual_repair(
    issue_id: String,
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<RepairPlan, String> {
    let engine = engine.read().await;
    engine.generate_repair_plan(&issue_id)
}

#[tauri::command]
pub async fn download_missing(
    dependency: Option<String>,
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<AutoRepairResult, String> {
    let mut engine = engine.write().await;
    let packages = dependency
        .filter(|dep| !dep.trim().is_empty())
        .map(|dep| vec![dep]);
    engine.install_dependencies(packages).await
}

#[tauri::command]
pub async fn restore_defaults(
    component: String,
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<String, String> {
    let engine = engine.read().await;
    engine.restore_defaults(&component)
}

#[tauri::command]
pub async fn get_repair_history(
    engine: tauri::State<'_, SharedDiagnosticsEngine>,
) -> Result<Vec<RepairRecord>, String> {
    let engine = engine.read().await;
    Ok(engine.get_repair_history())
}

#[tauri::command]
pub async fn get_diagnostics_settings(
    app_handle: tauri::AppHandle,
) -> Result<DiagnosticsSettings, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let settings_file = app_data_dir.join("settings").join("diagnostics.json");

    if settings_file.exists() {
        let content = std::fs::read_to_string(&settings_file)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
    } else {
        Ok(DiagnosticsSettings::default())
    }
}

#[tauri::command]
pub async fn save_diagnostics_settings(
    settings: DiagnosticsSettings,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let settings_dir = app_data_dir.join("settings");
    std::fs::create_dir_all(&settings_dir)
        .map_err(|e| format!("Failed to create settings directory: {}", e))?;

    let settings_file = settings_dir.join("diagnostics.json");
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_file, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn backup_before_repair(app_handle: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let backup_dir = app_data_dir.join("backups");
    std::fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let timestamp = chrono::Utc::now().timestamp();
    let backup_name = format!("pre_repair_backup_{}.tar.gz", timestamp);
    let backup_path = backup_dir.join(&backup_name);

    // TODO: Actually create a backup archive
    // For now, just create a marker file
    std::fs::write(&backup_path, b"backup placeholder")
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    Ok(backup_path.display().to_string())
}

#[tauri::command]
pub async fn rollback_repair(
    _rollback_token: String,
    _app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // TODO: Implement rollback logic
    Err("Rollback not yet implemented".to_string())
}

#[tauri::command]
pub async fn export_diagnostics_report(
    report: DiagnosticsReport,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let exports_dir = app_data_dir.join("exports");
    std::fs::create_dir_all(&exports_dir)
        .map_err(|e| format!("Failed to create exports directory: {}", e))?;

    let timestamp = chrono::Utc::now().timestamp();
    let export_name = format!("diagnostics_report_{}.json", timestamp);
    let export_path = exports_dir.join(&export_name);

    let content = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("Failed to serialize report: {}", e))?;

    std::fs::write(&export_path, content).map_err(|e| format!("Failed to write report: {}", e))?;

    Ok(export_path.display().to_string())
}

pub fn initialize_diagnostics_engine(
    app_handle: &tauri::AppHandle,
) -> Result<SharedDiagnosticsEngine, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to resolve project root: {}", e))?;

    let engine = DiagnosticsEngine::new(app_data_dir, project_root);
    Ok(Arc::new(RwLock::new(engine)))
}
