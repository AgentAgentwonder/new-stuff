use super::code_repair::CodeRepairModule;
use super::db_repair::DbRepairModule;
use super::dependency_manager::DependencyManager;
use super::file_repair::FileRepairModule;
use super::issue_detector::IssueDetector;
use super::network_repair::NetworkDiagnostics;
use super::performance_repair::PerformanceRepairModule;
use super::types::*;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct DiagnosticsEngine {
    file_repair: FileRepairModule,
    dependency_manager: DependencyManager,
    db_repair: DbRepairModule,
    code_repair: CodeRepairModule,
    network_diagnostics: NetworkDiagnostics,
    performance_repair: PerformanceRepairModule,
    issue_detector: IssueDetector,
    app_data_dir: PathBuf,
    project_root: PathBuf,
    last_report: Option<DiagnosticsReport>,
    repair_history: Vec<RepairRecord>,
}

impl DiagnosticsEngine {
    pub fn new(app_data_dir: PathBuf, project_root: PathBuf) -> Self {
        Self {
            file_repair: FileRepairModule::new(),
            dependency_manager: DependencyManager::new(),
            db_repair: DbRepairModule::new(),
            code_repair: CodeRepairModule::new(),
            network_diagnostics: NetworkDiagnostics::new(),
            performance_repair: PerformanceRepairModule::new(),
            issue_detector: IssueDetector::new(),
            app_data_dir,
            project_root,
            last_report: None,
            repair_history: Vec::new(),
        }
    }

    pub async fn run_full_diagnostics(&mut self) -> DiagnosticsReport {
        let mut panels = BTreeMap::new();
        let mut all_issues = Vec::new();
        let mut all_notes = Vec::new();

        // File system diagnostics
        let file_diag = self.file_repair.diagnose(&self.app_data_dir);
        all_issues.extend(file_diag.issues.clone());
        all_notes.extend(file_diag.notes.clone());
        panels.insert("file_system".to_string(), file_diag.panel);

        // Dependency diagnostics
        let dep_diag = self.dependency_manager.diagnose(&self.project_root);
        all_issues.extend(dep_diag.issues.clone());
        all_notes.extend(dep_diag.notes.clone());
        panels.insert("dependencies".to_string(), dep_diag.panel);

        // Database diagnostics
        let db_diag = self.db_repair.diagnose(&self.app_data_dir);
        all_issues.extend(db_diag.issues.clone());
        all_notes.extend(db_diag.notes.clone());
        panels.insert("database".to_string(), db_diag.panel);

        // Code integrity diagnostics
        let code_diag = self.code_repair.diagnose(&self.project_root);
        all_issues.extend(code_diag.issues.clone());
        all_notes.extend(code_diag.notes.clone());
        panels.insert("code_integrity".to_string(), code_diag.panel);

        // Network diagnostics
        let net_diag = self.network_diagnostics.diagnose().await;
        all_issues.extend(net_diag.issues.clone());
        all_notes.extend(net_diag.notes.clone());
        panels.insert("network".to_string(), net_diag.panel);

        // Performance diagnostics
        let perf_diag = self.performance_repair.diagnose();
        all_issues.extend(perf_diag.issues.clone());
        all_notes.extend(perf_diag.notes.clone());
        panels.insert("performance".to_string(), perf_diag.panel);

        // Calculate health score (0-100)
        let mut health_score = 100;
        for panel in panels.values() {
            health_score = (health_score as i32 + panel.level.score_modifier()) as u8;
        }
        health_score = health_score.max(0).min(100);

        // Summary stats
        let summary = DiagnosticsSummary {
            issues_found: all_issues.len(),
            auto_fixed: 0,
            manual_needed: all_issues
                .iter()
                .filter(|i| matches!(i.repair_level, RepairLevel::Manual))
                .count(),
            ignored: 0,
        };

        let report = DiagnosticsReport {
            generated_at: Utc::now(),
            health_score,
            summary,
            panels,
            issues: all_issues,
            repair_history: self.repair_history.clone(),
            notes: all_notes,
        };

        self.last_report = Some(report.clone());
        report
    }

    pub async fn auto_repair(
        &mut self,
        issue: &DiagnosticIssue,
    ) -> Result<AutoRepairResult, String> {
        let started_at = Utc::now();
        let result = match issue.category {
            IssueCategory::FileSystem => self.file_repair.auto_repair(issue, &self.app_data_dir),
            IssueCategory::Dependencies => {
                if let Some(packages) = self
                    .dependency_manager
                    .auto_repair(issue, &self.project_root)
                {
                    self.dependency_manager
                        .install_dependencies(
                            &self.project_root,
                            if packages.is_empty() {
                                None
                            } else {
                                Some(packages)
                            },
                        )
                        .await
                } else {
                    Err("No auto-repair available".to_string())
                }
            }
            IssueCategory::Database => self.db_repair.auto_repair(issue, &self.app_data_dir).await,
            IssueCategory::CodeIntegrity | IssueCategory::Configuration => {
                self.code_repair.auto_repair(issue)
            }
            IssueCategory::Network => self.network_diagnostics.auto_switch_endpoint(issue).await,
            IssueCategory::Performance => self
                .performance_repair
                .auto_repair(issue, &self.app_data_dir),
            _ => Err(format!(
                "No auto-repair available for category: {:?}",
                issue.category
            )),
        };

        if let Ok(mut repair_result) = result {
            if repair_result.issue_id.is_none() {
                repair_result.issue_id = Some(issue.id.clone());
            }
            self.record_repair(Some(issue), &repair_result, started_at);
            Ok(repair_result)
        } else {
            result
        }
    }

    pub async fn auto_repair_all(&mut self, issues: Vec<DiagnosticIssue>) -> Vec<AutoRepairResult> {
        let mut results = Vec::new();

        for issue in issues {
            if issue.auto_repair_available {
                match self.auto_repair(&issue).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        results.push(AutoRepairResult {
                            issue_id: Some(issue.id.clone()),
                            status: RepairStatus::Failed,
                            message: e,
                            actions: vec![],
                            backup_location: None,
                            rollback_token: None,
                        });
                    }
                }
            } else {
                results.push(AutoRepairResult {
                    issue_id: Some(issue.id.clone()),
                    status: RepairStatus::Skipped,
                    message: "Auto repair not available for this issue".to_string(),
                    actions: vec![],
                    backup_location: None,
                    rollback_token: None,
                });
            }
        }

        results
    }

    pub fn verify_integrity(&self) -> Result<bool, String> {
        // Verify app data directory structure
        if !self.app_data_dir.exists() {
            return Ok(false);
        }

        // Verify dependency integrity
        if !self.dependency_manager.verify(&self.project_root) {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn get_health_score(&self, report: &DiagnosticsReport) -> u8 {
        report.health_score
    }

    fn record_repair(
        &mut self,
        issue: Option<&DiagnosticIssue>,
        result: &AutoRepairResult,
        started_at: DateTime<Utc>,
    ) {
        let issue_id = issue
            .map(|i| i.id.clone())
            .or_else(|| result.issue_id.clone());

        let record = RepairRecord {
            id: Uuid::new_v4().to_string(),
            issue_id,
            started_at,
            completed_at: Some(Utc::now()),
            status: result.status.clone(),
            summary: result.message.clone(),
            actions: result.actions.clone(),
            backup_location: result.backup_location.clone(),
            rollback_token: result.rollback_token.clone(),
            metadata: HashMap::new(),
        };

        self.repair_history.push(record);

        if self.repair_history.len() > 100 {
            self.repair_history.remove(0);
        }
    }

    pub fn get_last_report(&self) -> Option<DiagnosticsReport> {
        self.last_report.clone()
    }

    pub fn get_issue(&self, issue_id: &str) -> Option<DiagnosticIssue> {
        self.last_report
            .as_ref()
            .and_then(|report| report.issues.iter().find(|issue| issue.id == issue_id))
            .cloned()
    }

    pub fn get_repair_history(&self) -> Vec<RepairRecord> {
        self.repair_history.clone()
    }

    pub async fn install_dependencies(
        &mut self,
        packages: Option<Vec<String>>,
    ) -> Result<AutoRepairResult, String> {
        let started_at = Utc::now();
        let result = self
            .dependency_manager
            .install_dependencies(&self.project_root, packages)
            .await;

        if let Ok(ref repair_result) = result {
            self.record_repair(None, repair_result, started_at);
        }

        result
    }

    pub fn restore_defaults(&self, component: &str) -> Result<String, String> {
        match component {
            "settings" => {
                let settings_dir = self.app_data_dir.join("settings");
                fs::create_dir_all(&settings_dir)
                    .map_err(|e| format!("Failed to create settings dir: {}", e))?;
                let path = settings_dir.join("app.json");
                let defaults = json!({
                    "autoScan": true,
                    "autoRepair": "ask",
                    "notifications": {
                        "critical": true,
                        "warnings": true,
                        "weeklySummary": true
                    }
                });
                let payload = serde_json::to_string_pretty(&defaults)
                    .map_err(|e| format!("Failed to serialize defaults: {}", e))?;
                fs::write(&path, payload)
                    .map_err(|e| format!("Failed to write defaults: {}", e))?;
                Ok(path.display().to_string())
            }
            "themes" => {
                let themes_dir = self.app_data_dir.join("themes");
                fs::create_dir_all(&themes_dir)
                    .map_err(|e| format!("Failed to create themes dir: {}", e))?;
                let path = themes_dir.join("default.json");
                let defaults = json!({
                    "name": "Default",
                    "primaryColor": "#0ea5e9",
                    "accentColor": "#f59e0b"
                });
                let payload = serde_json::to_string_pretty(&defaults)
                    .map_err(|e| format!("Failed to serialize theme defaults: {}", e))?;
                fs::write(&path, payload)
                    .map_err(|e| format!("Failed to write default theme: {}", e))?;
                Ok(path.display().to_string())
            }
            _ => Err(format!("Unsupported component: {}", component)),
        }
    }

    pub fn generate_repair_plan(&self, issue_id: &str) -> Result<RepairPlan, String> {
        let issue = self
            .get_issue(issue_id)
            .ok_or_else(|| format!("Issue {} not found", issue_id))?;

        let mut steps = Vec::new();
        let requires_backup = matches!(
            issue.category,
            IssueCategory::Database | IssueCategory::FileSystem
        );

        match issue.category {
            IssueCategory::Dependencies => {
                steps.push(RepairAction {
                    action: "verify_package_json".to_string(),
                    description: "Validate package.json entries".to_string(),
                    estimated_duration: Some("1 minute".to_string()),
                    level: RepairLevel::Confirmation,
                    requires_confirmation: false,
                });
                steps.push(RepairAction {
                    action: "install_dependencies".to_string(),
                    description: "Run npm install for missing packages".to_string(),
                    estimated_duration: Some("5 minutes".to_string()),
                    level: RepairLevel::Confirmation,
                    requires_confirmation: true,
                });
            }
            IssueCategory::Database => {
                steps.push(RepairAction {
                    action: "backup_database".to_string(),
                    description: "Create backup of current database".to_string(),
                    estimated_duration: Some("1 minute".to_string()),
                    level: RepairLevel::Confirmation,
                    requires_confirmation: true,
                });
                steps.push(RepairAction {
                    action: "rebuild_schema".to_string(),
                    description: "Re-create database schema and restore data".to_string(),
                    estimated_duration: Some("3 minutes".to_string()),
                    level: RepairLevel::Manual,
                    requires_confirmation: true,
                });
            }
            IssueCategory::FileSystem => {
                steps.push(RepairAction {
                    action: "restore_missing_files".to_string(),
                    description: "Restore files from defaults or backup".to_string(),
                    estimated_duration: Some("2 minutes".to_string()),
                    level: RepairLevel::Automatic,
                    requires_confirmation: false,
                });
                steps.push(RepairAction {
                    action: "verify_permissions".to_string(),
                    description: "Ensure correct file permissions".to_string(),
                    estimated_duration: Some("1 minute".to_string()),
                    level: issue.repair_level.clone(),
                    requires_confirmation: false,
                });
            }
            IssueCategory::Network => {
                steps.push(RepairAction {
                    action: "test_alternate_endpoints".to_string(),
                    description: "Test alternative API endpoints".to_string(),
                    estimated_duration: Some("30 seconds".to_string()),
                    level: RepairLevel::Automatic,
                    requires_confirmation: false,
                });
                steps.push(RepairAction {
                    action: "update_firewall".to_string(),
                    description: "Ensure application access through firewall".to_string(),
                    estimated_duration: Some("2 minutes".to_string()),
                    level: RepairLevel::Manual,
                    requires_confirmation: true,
                });
            }
            IssueCategory::Performance => {
                steps.push(RepairAction {
                    action: "clear_caches".to_string(),
                    description: "Clear application caches to free memory".to_string(),
                    estimated_duration: Some("30 seconds".to_string()),
                    level: RepairLevel::Automatic,
                    requires_confirmation: false,
                });
                steps.push(RepairAction {
                    action: "restart_services".to_string(),
                    description: "Restart background services".to_string(),
                    estimated_duration: Some("1 minute".to_string()),
                    level: RepairLevel::Automatic,
                    requires_confirmation: true,
                });
            }
            _ => {
                steps.push(RepairAction {
                    action: "investigate_issue".to_string(),
                    description: "Review logs and context".to_string(),
                    estimated_duration: Some("2 minutes".to_string()),
                    level: issue.repair_level.clone(),
                    requires_confirmation: false,
                });
            }
        }

        Ok(RepairPlan {
            level: issue.repair_level.clone(),
            steps,
            requires_backup,
            estimated_time: if requires_backup {
                "5-10 minutes".to_string()
            } else {
                "2-5 minutes".to_string()
            },
            risk: match issue.severity {
                IssueSeverity::Critical => "High".to_string(),
                IssueSeverity::Warning => "Medium".to_string(),
                IssueSeverity::Info => "Low".to_string(),
            },
        })
    }
}
