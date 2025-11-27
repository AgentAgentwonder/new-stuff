use crate::compiler::{AutoCompiler, BuildStatus, CompilationResult};
use crate::errors::{CrashReport, SharedCrashReporter, SharedRuntimeHandler};
use crate::fixer::{AutoFixer, FixAttempt, FixStats};
use crate::logger::{ComprehensiveLogger, LogEntry, LogLevel, LoggerConfig, SharedLogger};
use crate::monitor::{PerformanceMetrics, SharedPerformanceMonitor};
use crate::recovery::{ErrorRecoveryManager, RecoveryPlan};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn compile_now(
    compiler: State<'_, Arc<AutoCompiler>>,
) -> Result<CompilationResult, String> {
    compiler.compile_now()
}

#[tauri::command]
pub async fn get_build_status(
    compiler: State<'_, Arc<AutoCompiler>>,
) -> Result<BuildStatus, String> {
    Ok(compiler.get_status())
}

#[tauri::command]
pub async fn get_compile_errors(
    compiler: State<'_, Arc<AutoCompiler>>,
) -> Result<Vec<crate::compiler::CompilationError>, String> {
    Ok(compiler.get_errors())
}

#[tauri::command]
pub async fn auto_fix_errors(
    fixer: State<'_, Arc<AutoFixer>>,
    errors: Vec<String>,
) -> Result<Vec<FixAttempt>, String> {
    let mut results = Vec::new();
    for error in errors {
        match fixer.attempt_fix(&error) {
            Ok(attempt) => results.push(attempt),
            Err(e) => {
                return Err(format!("Failed to fix error '{}': {}", error, e));
            }
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn get_fix_stats(fixer: State<'_, Arc<AutoFixer>>) -> Result<FixStats, String> {
    Ok(fixer.get_stats())
}

#[tauri::command]
pub async fn get_fix_attempts(fixer: State<'_, Arc<AutoFixer>>) -> Result<Vec<FixAttempt>, String> {
    Ok(fixer.get_attempts())
}

#[tauri::command]
pub async fn clear_fix_history(fixer: State<'_, Arc<AutoFixer>>) -> Result<(), String> {
    fixer.clear_history();
    Ok(())
}

#[tauri::command]
pub async fn get_logs(
    logger: State<'_, SharedLogger>,
    limit: usize,
    level: Option<String>,
) -> Result<Vec<LogEntry>, String> {
    let min_level = level.and_then(|l| LogLevel::from_str(&l));
    Ok(logger.get_recent_logs(limit, min_level))
}

#[tauri::command]
pub async fn clear_logs(logger: State<'_, SharedLogger>) -> Result<(), String> {
    logger.clear_buffer();
    Ok(())
}

#[tauri::command]
pub async fn export_logs(
    logger: State<'_, SharedLogger>,
    format: String,
) -> Result<String, String> {
    logger.export_logs(&format)
}

#[tauri::command]
pub async fn log_message(
    logger: State<'_, SharedLogger>,
    level: String,
    message: String,
    category: Option<String>,
    details: Option<serde_json::Value>,
) -> Result<(), String> {
    let log_level =
        LogLevel::from_str(&level).ok_or_else(|| format!("Invalid log level: {}", level))?;
    logger.log(log_level, &message, category.as_deref(), details, None);
    Ok(())
}

#[tauri::command]
pub async fn get_logger_config(logger: State<'_, SharedLogger>) -> Result<LoggerConfig, String> {
    Ok(logger.get_config())
}

#[tauri::command]
pub async fn set_logger_config(
    logger: State<'_, SharedLogger>,
    config: LoggerConfig,
) -> Result<(), String> {
    logger.set_config(config);
    Ok(())
}

#[tauri::command]
pub async fn get_dev_performance_metrics(
    monitor: State<'_, SharedPerformanceMonitor>,
) -> Result<PerformanceMetrics, String> {
    Ok(monitor.latest_metrics())
}

#[tauri::command]
pub async fn get_error_stats(
    handler: State<'_, SharedRuntimeHandler>,
) -> Result<crate::errors::ErrorStats, String> {
    Ok(handler.get_error_stats())
}

#[tauri::command]
pub async fn report_crash(
    crash_reporter: State<'_, SharedCrashReporter>,
    message: String,
    stack_trace: Option<String>,
) -> Result<CrashReport, String> {
    let system_state = serde_json::json!({
        "timestamp": chrono::Utc::now(),
    });

    crash_reporter.capture_crash(&message, stack_trace, system_state)
}

#[tauri::command]
pub async fn get_crash_report(
    crash_reporter: State<'_, SharedCrashReporter>,
    crash_id: String,
) -> Result<CrashReport, String> {
    crash_reporter.get_report(&crash_id)
}

#[tauri::command]
pub async fn list_crash_reports(
    crash_reporter: State<'_, SharedCrashReporter>,
) -> Result<Vec<String>, String> {
    Ok(crash_reporter.list_reports())
}

#[tauri::command]
pub async fn force_gc() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn restart_service(service_name: String) -> Result<(), String> {
    tracing::info!("Restart requested for service: {}", service_name);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevSettings {
    pub auto_compilation_enabled: bool,
    pub auto_fix_enabled: bool,
    pub log_level: String,
    pub log_retention_days: u32,
    pub crash_reporting_enabled: bool,
    pub performance_monitoring_enabled: bool,
    pub auto_fix_confidence_threshold: f64,
    pub max_compilation_retries: u32,
    pub developer_console_hotkey: String,
}

impl Default for DevSettings {
    fn default() -> Self {
        Self {
            auto_compilation_enabled: true,
            auto_fix_enabled: true,
            log_level: "INFO".to_string(),
            log_retention_days: 30,
            crash_reporting_enabled: true,
            performance_monitoring_enabled: true,
            auto_fix_confidence_threshold: 0.7,
            max_compilation_retries: 3,
            developer_console_hotkey: "Ctrl+Shift+D".to_string(),
        }
    }
}

#[tauri::command]
pub async fn get_dev_settings() -> Result<DevSettings, String> {
    Ok(DevSettings::default())
}

#[tauri::command]
pub async fn update_dev_settings(settings: DevSettings) -> Result<(), String> {
    tracing::info!("Updated dev settings: {:?}", settings);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriHealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub backend_initialized: bool,
}

#[tauri::command]
pub async fn check_tauri_health() -> Result<TauriHealthStatus, String> {
    Ok(TauriHealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        backend_initialized: true,
    })
}
