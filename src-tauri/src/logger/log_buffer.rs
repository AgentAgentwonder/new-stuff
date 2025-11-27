use crate::logger::log_level::LogLevel;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

pub type SharedLogBuffer = Arc<LogBuffer>;

const DEFAULT_BUFFER_SIZE: usize = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub category: Option<String>,
    pub details: Option<serde_json::Value>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub function: Option<String>,
    pub thread_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub environment: Option<String>,
    pub app_version: Option<String>,
    pub os_version: Option<String>,
    pub memory_usage: Option<f64>,
    pub cpu_usage: Option<f64>,
    pub duration_ms: Option<f64>,
}

#[derive(Debug)]
pub struct LogBuffer {
    entries: RwLock<VecDeque<LogEntry>>,
    capacity: usize,
}

impl LogBuffer {
    pub fn new(capacity: Option<usize>) -> Self {
        let size = capacity.unwrap_or(DEFAULT_BUFFER_SIZE);
        Self {
            entries: RwLock::new(VecDeque::with_capacity(size)),
            capacity: size.max(128),
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.write();
        if entries.len() >= self.capacity {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn clear(&self) {
        self.entries.write().clear();
    }

    pub fn take_recent(&self, limit: usize, level: Option<LogLevel>) -> Vec<LogEntry> {
        let entries = self.entries.read();
        entries
            .iter()
            .rev()
            .filter(|entry| match level {
                Some(level) => entry.level >= level,
                None => true,
            })
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn iter(&self) -> Vec<LogEntry> {
        self.entries.read().iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.read().len()
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(None)
    }
}
