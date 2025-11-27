use crate::logger::{ComprehensiveLogger, LogLevel, SharedLogger};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashReport {
    pub crash_id: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub stack_trace: Option<String>,
    pub system_state: serde_json::Value,
    pub user_actions: Option<Vec<String>>,
    pub logs: Vec<crate::logger::LogEntry>,
    pub app_version: String,
    pub environment: String,
}

pub type SharedCrashReporter = Arc<CrashReporter>;

#[derive(Clone)]
pub struct CrashReporter {
    app_handle: AppHandle,
    logger: SharedLogger,
    report_dir: PathBuf,
}

impl CrashReporter {
    pub fn new(app: &AppHandle, logger: SharedLogger) -> Result<Self, std::io::Error> {
        let mut report_dir = app.path().app_data_dir().map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "App data dir not found")
        })?;

        report_dir.push("crash_reports");
        std::fs::create_dir_all(&report_dir)?;

        Ok(Self {
            app_handle: app.clone(),
            logger,
            report_dir,
        })
    }

    pub fn capture_crash(
        &self,
        message: &str,
        stack_trace: Option<String>,
        system_state: serde_json::Value,
    ) -> Result<CrashReport, String> {
        let crash_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        let app_version = self.app_handle.package_info().version.to_string();
        let environment =
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let logs = self.logger.get_recent_logs(1000, None);

        let report = CrashReport {
            crash_id: crash_id.clone(),
            timestamp,
            message: message.to_string(),
            stack_trace,
            system_state,
            user_actions: None,
            logs,
            app_version,
            environment,
        };

        self.persist_report(&report)
            .map_err(|e| format!("Failed to persist crash report: {}", e))?;

        self.logger.error(
            "Crash captured",
            Some(serde_json::json!({
                "crash_id": crash_id,
                "message": message,
            })),
        );

        Ok(report)
    }

    fn persist_report(&self, report: &CrashReport) -> Result<(), std::io::Error> {
        let filename = format!("{}.json", report.crash_id);
        let path = self.report_dir.join(filename);
        let mut file = File::create(path)?;
        let json = serde_json::to_string_pretty(report)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn get_report(&self, crash_id: &str) -> Result<CrashReport, String> {
        let filename = format!("{}.json", crash_id);
        let path = self.report_dir.join(filename);
        if !path.exists() {
            return Err("Crash report not found".to_string());
        }

        let file =
            std::fs::File::open(path).map_err(|e| format!("Failed to open crash report: {}", e))?;
        serde_json::from_reader(file)
            .map_err(|e| format!("Failed to deserialize crash report: {}", e))
    }

    pub fn list_reports(&self) -> Vec<String> {
        if let Ok(entries) = std::fs::read_dir(&self.report_dir) {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| {
                    entry
                        .path()
                        .file_stem()
                        .and_then(|stem| stem.to_str().map(|s| s.to_string()))
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
