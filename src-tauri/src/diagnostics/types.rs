use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    FileSystem,
    Dependencies,
    Database,
    Network,
    Performance,
    CodeIntegrity,
    Configuration,
    Security,
    Environment,
    Service,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairLevel {
    Automatic,
    Confirmation,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RepairStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthLevel {
    Excellent,
    Good,
    Warning,
    Degraded,
    Critical,
    Unknown,
}

impl HealthLevel {
    pub fn score_modifier(&self) -> i32 {
        match self {
            HealthLevel::Excellent => 0,
            HealthLevel::Good => -5,
            HealthLevel::Warning => -8,
            HealthLevel::Degraded => -15,
            HealthLevel::Critical => -35,
            HealthLevel::Unknown => -10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelMetric {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<HealthLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelStatus {
    pub title: String,
    pub level: HealthLevel,
    pub summary: String,
    pub metrics: Vec<PanelMetric>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticsSummary {
    #[serde(default)]
    pub issues_found: usize,
    #[serde(default)]
    pub auto_fixed: usize,
    #[serde(default)]
    pub manual_needed: usize,
    #[serde(default)]
    pub ignored: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticIssue {
    pub id: String,
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub recommended_action: String,
    pub repair_level: RepairLevel,
    pub auto_repair_available: bool,
    pub status: RepairStatus,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAction {
    pub action: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration: Option<String>,
    pub level: RepairLevel,
    #[serde(default)]
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairRecord {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub status: RepairStatus,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<RepairAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_token: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub generated_at: DateTime<Utc>,
    pub health_score: u8,
    pub summary: DiagnosticsSummary,
    pub panels: BTreeMap<String, PanelStatus>,
    pub issues: Vec<DiagnosticIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repair_history: Vec<RepairRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDiagnostics {
    pub panel: PanelStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<DiagnosticIssue>,
    #[serde(default)]
    pub auto_fixed: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoRepairMode {
    Auto,
    Ask,
    Never,
}

impl Default for AutoRepairMode {
    fn default() -> Self {
        AutoRepairMode::Ask
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSettings {
    pub auto_scan_on_startup: bool,
    pub scan_interval_minutes: u64,
    pub auto_repair_mode: AutoRepairMode,
    pub backup_before_repair: bool,
    pub history_retention_days: u32,
    pub dry_run: bool,
}

impl Default for DiagnosticsSettings {
    fn default() -> Self {
        Self {
            auto_scan_on_startup: true,
            scan_interval_minutes: 60,
            auto_repair_mode: AutoRepairMode::Ask,
            backup_before_repair: true,
            history_retention_days: 30,
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRepairResult {
    pub issue_id: Option<String>,
    pub status: RepairStatus,
    pub message: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<RepairAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlan {
    pub level: RepairLevel,
    pub steps: Vec<RepairAction>,
    pub requires_backup: bool,
    pub estimated_time: String,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueDetectionContext {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub logs: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueDetectionResult {
    pub issues: Vec<DiagnosticIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}
