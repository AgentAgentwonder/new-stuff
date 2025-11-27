use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BuildStatus {
    Idle,
    Building,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilationError {
    pub file: String,
    pub line: u32,
    pub column: Option<u32>,
    pub message: String,
    pub severity: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilationResult {
    pub status: BuildStatus,
    pub timestamp: DateTime<Utc>,
    pub errors: Vec<CompilationError>,
    pub warnings: Vec<CompilationError>,
    pub duration_ms: f64,
}

#[derive(Clone)]
pub struct AutoCompiler {
    status: Arc<RwLock<BuildStatus>>,
    last_result: Arc<RwLock<Option<CompilationResult>>>,
}

impl Default for AutoCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoCompiler {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(BuildStatus::Idle)),
            last_result: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_status(&self) -> BuildStatus {
        self.status.read().clone()
    }

    pub fn set_status(&self, status: BuildStatus) {
        *self.status.write() = status;
    }

    pub fn get_last_result(&self) -> Option<CompilationResult> {
        self.last_result.read().clone()
    }

    pub fn set_result(&self, result: CompilationResult) {
        *self.last_result.write() = Some(result);
    }

    pub fn get_errors(&self) -> Vec<CompilationError> {
        self.last_result
            .read()
            .as_ref()
            .map(|r| r.errors.clone())
            .unwrap_or_default()
    }

    pub fn compile_now(&self) -> Result<CompilationResult, String> {
        self.set_status(BuildStatus::Building);

        let start = std::time::Instant::now();

        let result = CompilationResult {
            status: BuildStatus::Success,
            timestamp: Utc::now(),
            errors: vec![],
            warnings: vec![],
            duration_ms: start.elapsed().as_millis() as f64,
        };

        self.set_status(result.status.clone());
        self.set_result(result.clone());

        Ok(result)
    }
}
