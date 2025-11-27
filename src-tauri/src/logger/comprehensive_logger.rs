use crate::logger::{LogBuffer, LogEntry, LogLevel, SharedLogBuffer};
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use sysinfo::CpuExt;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub type SharedLogger = Arc<ComprehensiveLogger>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggerConfig {
    pub min_level: LogLevel,
    pub console_enabled: bool,
    pub file_enabled: bool,
    pub buffer_enabled: bool,
    pub colored_output: bool,
    pub include_metadata: bool,
    pub max_file_size_mb: u64,
    pub max_files: usize,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Info,
            console_enabled: true,
            file_enabled: true,
            buffer_enabled: true,
            colored_output: true,
            include_metadata: true,
            max_file_size_mb: 100,
            max_files: 10,
        }
    }
}

#[derive(Debug)]
pub struct ComprehensiveLogger {
    config: RwLock<LoggerConfig>,
    buffer: SharedLogBuffer,
    log_dir: PathBuf,
    current_log_file: RwLock<Option<File>>,
    session_id: String,
    environment: String,
    app_version: String,
}

impl ComprehensiveLogger {
    pub fn new(app: &AppHandle) -> Result<Self, std::io::Error> {
        let mut log_dir = app.path().app_data_dir().map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("App data dir not found: {err}"),
            )
        })?;

        log_dir.push("logs");
        std::fs::create_dir_all(&log_dir)?;

        let session_id = Uuid::new_v4().to_string();
        let environment =
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let app_version = app.package_info().version.to_string();

        let log_file = Self::create_log_file(&log_dir)?;

        Ok(Self {
            config: RwLock::new(LoggerConfig::default()),
            buffer: Arc::new(LogBuffer::new(Some(5000))),
            log_dir,
            current_log_file: RwLock::new(Some(log_file)),
            session_id,
            environment,
            app_version,
        })
    }

    fn create_log_file(log_dir: &Path) -> Result<File, std::io::Error> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("app_{}.log", timestamp);
        let log_path = log_dir.join(filename);

        OpenOptions::new().create(true).append(true).open(log_path)
    }

    pub fn set_config(&self, config: LoggerConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> LoggerConfig {
        self.config.read().clone()
    }

    pub fn trace(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Trace, message, None, details, None);
    }

    pub fn debug(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Debug, message, None, details, None);
    }

    pub fn info(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Info, message, None, details, None);
    }

    pub fn warn(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Warn, message, None, details, None);
    }

    pub fn error(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Error, message, None, details, None);
    }

    pub fn fatal(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Fatal, message, None, details, None);
    }

    pub fn success(&self, message: &str, details: Option<serde_json::Value>) {
        self.log(LogLevel::Success, message, None, details, None);
    }

    pub fn performance(&self, message: &str, duration_ms: f64, details: Option<serde_json::Value>) {
        let mut d = details.unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = d.as_object_mut() {
            obj.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
        }
        self.log(
            LogLevel::Performance,
            message,
            None,
            Some(d),
            Some(duration_ms),
        );
    }

    pub fn log(
        &self,
        level: LogLevel,
        message: &str,
        category: Option<&str>,
        details: Option<serde_json::Value>,
        duration_ms: Option<f64>,
    ) {
        let config = self.config.read();

        if level < config.min_level {
            return;
        }

        let thread_id = format!("{:?}", thread::current().id());

        let (memory_usage, cpu_usage) = if config.include_metadata {
            get_system_metrics()
        } else {
            (None, None)
        };

        let entry = LogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            message: message.to_string(),
            category: category.map(|s| s.to_string()),
            details,
            file: None,
            line: None,
            function: None,
            thread_id: Some(thread_id),
            session_id: Some(self.session_id.clone()),
            request_id: None,
            user_id: None,
            environment: Some(self.environment.clone()),
            app_version: Some(self.app_version.clone()),
            os_version: Some(get_os_version()),
            memory_usage,
            cpu_usage,
            duration_ms,
        };

        if config.buffer_enabled {
            self.buffer.push(entry.clone());
        }

        if config.console_enabled {
            self.log_to_console(&entry, config.colored_output);
        }

        if config.file_enabled {
            let _ = self.log_to_file(&entry);
        }
    }

    fn log_to_console(&self, entry: &LogEntry, colored: bool) {
        let color = if colored { entry.level.color() } else { "" };
        let reset = if colored { "\x1b[0m" } else { "" };

        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let level_str = entry.level.as_str();

        let category_str = entry
            .category
            .as_ref()
            .map(|c| format!(" [{}]", c))
            .unwrap_or_default();

        let details_str = entry
            .details
            .as_ref()
            .map(|d| format!(" | {}", serde_json::to_string(d).unwrap_or_default()))
            .unwrap_or_default();

        println!(
            "{}{} {} {}{} {}{}",
            color, timestamp, level_str, entry.message, category_str, details_str, reset
        );
    }

    fn log_to_file(&self, entry: &LogEntry) -> Result<(), std::io::Error> {
        let mut file_guard = self.current_log_file.write();

        if let Some(ref mut file) = *file_guard {
            let log_line = self.format_log_entry(entry);
            writeln!(file, "{}", log_line)?;
            file.flush()?;
        }

        Ok(())
    }

    fn format_log_entry(&self, entry: &LogEntry) -> String {
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let level = entry.level.as_str();
        let category = entry
            .category
            .as_ref()
            .map(|c| format!(" [{}]", c))
            .unwrap_or_default();
        let details = entry
            .details
            .as_ref()
            .map(|d| format!(" | {}", serde_json::to_string(d).unwrap_or_default()))
            .unwrap_or_default();

        format!(
            "{} {} {}{}{}",
            timestamp, level, entry.message, category, details
        )
    }

    pub fn get_buffer(&self) -> SharedLogBuffer {
        self.buffer.clone()
    }

    pub fn get_recent_logs(&self, limit: usize, min_level: Option<LogLevel>) -> Vec<LogEntry> {
        self.buffer.take_recent(limit, min_level)
    }

    pub fn clear_buffer(&self) {
        self.buffer.clear();
    }

    pub fn rotate_log_file(&self) -> Result<(), std::io::Error> {
        let new_file = Self::create_log_file(&self.log_dir)?;
        let mut file_guard = self.current_log_file.write();
        *file_guard = Some(new_file);
        self.cleanup_old_logs()?;
        Ok(())
    }

    fn cleanup_old_logs(&self) -> Result<(), std::io::Error> {
        let max_files = self.config.read().max_files;

        let mut log_files: Vec<_> = std::fs::read_dir(&self.log_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("log"))
            .collect();

        if log_files.len() <= max_files {
            return Ok(());
        }

        log_files.sort_by_key(|entry| {
            entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        let files_to_delete = log_files.len().saturating_sub(max_files);
        for entry in log_files.iter().take(files_to_delete) {
            let _ = std::fs::remove_file(entry.path());
        }

        Ok(())
    }

    pub fn export_logs(&self, format: &str) -> Result<String, String> {
        let logs = self.buffer.iter();

        match format {
            "json" => serde_json::to_string_pretty(&logs)
                .map_err(|e| format!("Failed to serialize logs: {}", e)),
            "csv" => {
                let mut csv = String::from("timestamp,level,message,category,details\n");
                for log in logs {
                    let details = log
                        .details
                        .as_ref()
                        .and_then(|d| serde_json::to_string(d).ok())
                        .unwrap_or_default()
                        .replace('"', "\"\"");

                    csv.push_str(&format!(
                        "{},{},{},{},{}\n",
                        log.timestamp.to_rfc3339(),
                        log.level.as_str(),
                        log.message.replace(',', ";"),
                        log.category.as_ref().unwrap_or(&String::new()),
                        details
                    ));
                }
                Ok(csv)
            }
            _ => Err(format!("Unsupported format: {}", format)),
        }
    }
}

fn get_system_metrics() -> (Option<f64>, Option<f64>) {
    use sysinfo::{System, SystemExt};

    let mut sys = System::new_all();
    sys.refresh_all();

    let memory_usage = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
    let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;

    (Some(memory_usage), Some(cpu_usage))
}

fn get_os_version() -> String {
    std::env::consts::OS.to_string()
}
