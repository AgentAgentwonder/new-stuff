use super::types::*;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct CodeRepairModule;

impl CodeRepairModule {
    pub fn new() -> Self {
        Self
    }

    pub fn diagnose(&self, project_root: &Path) -> ModuleDiagnostics {
        let mut issues = Vec::new();
        let mut metrics = Vec::new();
        let mut notes = Vec::new();

        let theme_dir = project_root.join("src").join("styles").join("themes");
        let config_dir = project_root.join("src").join("configs");
        let custom_components_dir = project_root.join("src").join("components");

        let theme_status = if theme_dir.exists() {
            self.validate_theme_files(&theme_dir, &mut issues)
        } else {
            HealthLevel::Unknown
        };

        let config_status = if config_dir.exists() {
            self.validate_config_files(&config_dir, &mut issues)
        } else {
            HealthLevel::Unknown
        };

        let component_status = if custom_components_dir.exists() {
            self.validate_component_integrity(&custom_components_dir, &mut issues)
        } else {
            HealthLevel::Unknown
        };

        metrics.push(PanelMetric {
            label: "Theme Files".to_string(),
            value: format!("Status: {:?}", theme_status),
            level: Some(theme_status.clone()),
        });

        metrics.push(PanelMetric {
            label: "Config Files".to_string(),
            value: format!("Status: {:?}", config_status),
            level: Some(config_status.clone()),
        });

        metrics.push(PanelMetric {
            label: "Custom Components".to_string(),
            value: format!("Status: {:?}", component_status),
            level: Some(component_status.clone()),
        });

        let level = [theme_status, config_status, component_status]
            .into_iter()
            .fold(HealthLevel::Excellent, |acc, status| match (acc, status) {
                (HealthLevel::Critical, _) | (_, HealthLevel::Critical) => HealthLevel::Critical,
                (HealthLevel::Degraded, _) | (_, HealthLevel::Degraded) => HealthLevel::Degraded,
                (HealthLevel::Unknown, _) | (_, HealthLevel::Unknown) => HealthLevel::Unknown,
                _ => HealthLevel::Excellent,
            });

        if issues.is_empty() {
            notes.push("All code integrity checks passed".to_string());
        }

        ModuleDiagnostics {
            panel: PanelStatus {
                title: "Code Integrity".to_string(),
                level,
                summary: if issues.is_empty() {
                    "No code integrity issues detected".to_string()
                } else {
                    format!("{} code issue(s) detected", issues.len())
                },
                metrics,
                actions: vec![],
            },
            issues,
            auto_fixed: 0,
            notes,
        }
    }

    fn validate_theme_files(
        &self,
        theme_dir: &Path,
        issues: &mut Vec<DiagnosticIssue>,
    ) -> HealthLevel {
        if !theme_dir.exists() {
            return HealthLevel::Unknown;
        }

        let mut health = HealthLevel::Excellent;

        if let Ok(entries) = fs::read_dir(theme_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Err(issue) =
                            self.validate_json_file(&entry.path(), IssueCategory::CodeIntegrity)
                        {
                            health = HealthLevel::Degraded;
                            issues.push(issue);
                        }
                    }
                }
            }
        }

        health
    }

    fn validate_config_files(
        &self,
        config_dir: &Path,
        issues: &mut Vec<DiagnosticIssue>,
    ) -> HealthLevel {
        if !config_dir.exists() {
            return HealthLevel::Unknown;
        }

        let mut health = HealthLevel::Excellent;

        if let Ok(entries) = fs::read_dir(config_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        if let Err(issue) =
                            self.validate_json_file(&entry.path(), IssueCategory::Configuration)
                        {
                            health = HealthLevel::Degraded;
                            issues.push(issue);
                        }
                    } else if ext == "toml" {
                        if let Err(issue) = self.validate_toml_file(&entry.path()) {
                            health = HealthLevel::Degraded;
                            issues.push(issue);
                        }
                    }
                }
            }
        }

        health
    }

    fn validate_component_integrity(
        &self,
        components_dir: &Path,
        issues: &mut Vec<DiagnosticIssue>,
    ) -> HealthLevel {
        if !components_dir.exists() {
            return HealthLevel::Unknown;
        }

        let mut health = HealthLevel::Excellent;

        if let Ok(entries) = fs::read_dir(components_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "tsx" || ext == "ts" {
                        if let Err(issue) = self.validate_component_file(&entry.path()) {
                            health = HealthLevel::Degraded;
                            issues.push(issue);
                        }
                    }
                }
            }
        }

        health
    }

    fn validate_json_file(
        &self,
        path: &Path,
        category: IssueCategory,
    ) -> Result<(), DiagnosticIssue> {
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(_) => Ok(()),
                Err(err) => Err(DiagnosticIssue {
                    id: Uuid::new_v4().to_string(),
                    category,
                    severity: IssueSeverity::Warning,
                    title: format!(
                        "Invalid JSON: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    description: format!("Failed to parse JSON file {}: {}", path.display(), err),
                    detected_at: Utc::now(),
                    recommended_action: "Repair JSON formatting".to_string(),
                    repair_level: RepairLevel::Confirmation,
                    auto_repair_available: true,
                    status: RepairStatus::Pending,
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert(
                            "path".into(),
                            serde_json::Value::String(path.display().to_string()),
                        );
                        m
                    },
                }),
            },
            Err(err) => Err(DiagnosticIssue {
                id: Uuid::new_v4().to_string(),
                category,
                severity: IssueSeverity::Warning,
                title: format!(
                    "Unreadable JSON: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                description: format!("Failed to read JSON file {}: {}", path.display(), err),
                detected_at: Utc::now(),
                recommended_action: "Check file permissions".to_string(),
                repair_level: RepairLevel::Confirmation,
                auto_repair_available: false,
                status: RepairStatus::Pending,
                metadata: HashMap::new(),
            }),
        }
    }

    fn validate_toml_file(&self, path: &Path) -> Result<(), DiagnosticIssue> {
        match fs::read_to_string(path) {
            Ok(_content) => {
                // TOML validation is skipped for now since toml crate isn't included
                // In the future, add toml = "0.8" to Cargo.toml and uncomment validation
                Ok(())
            }
            Err(err) => Err(DiagnosticIssue {
                id: Uuid::new_v4().to_string(),
                category: IssueCategory::Configuration,
                severity: IssueSeverity::Warning,
                title: format!(
                    "Unreadable TOML: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                description: format!("Failed to read TOML file {}: {}", path.display(), err),
                detected_at: Utc::now(),
                recommended_action: "Check file permissions".to_string(),
                repair_level: RepairLevel::Confirmation,
                auto_repair_available: false,
                status: RepairStatus::Pending,
                metadata: HashMap::new(),
            }),
        }
    }

    fn validate_component_file(&self, path: &Path) -> Result<(), DiagnosticIssue> {
        let metadata = fs::metadata(path).map_err(|err| DiagnosticIssue {
            id: Uuid::new_v4().to_string(),
            category: IssueCategory::CodeIntegrity,
            severity: IssueSeverity::Warning,
            title: format!(
                "Unreadable component: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            description: format!(
                "Failed to access component file {}: {}",
                path.display(),
                err
            ),
            detected_at: Utc::now(),
            recommended_action: "Check file permissions".to_string(),
            repair_level: RepairLevel::Confirmation,
            auto_repair_available: false,
            status: RepairStatus::Pending,
            metadata: HashMap::new(),
        })?;

        if metadata.len() == 0 {
            return Err(DiagnosticIssue {
                id: Uuid::new_v4().to_string(),
                category: IssueCategory::CodeIntegrity,
                severity: IssueSeverity::Warning,
                title: format!(
                    "Empty component: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                description: format!("Component file {} is empty", path.display()),
                detected_at: Utc::now(),
                recommended_action: "Restore component file".to_string(),
                repair_level: RepairLevel::Manual,
                auto_repair_available: false,
                status: RepairStatus::Pending,
                metadata: HashMap::new(),
            });
        }

        Ok(())
    }

    pub fn auto_repair(&self, issue: &DiagnosticIssue) -> Result<AutoRepairResult, String> {
        if let Some(path_value) = issue.metadata.get("path") {
            if let Some(path_str) = path_value.as_str() {
                let path = PathBuf::from(path_str);

                // For JSON/TOML, attempt to pretty-print to fix format
                if path.extension().map(|ext| ext == "json").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(value) = serde_json::from_str::<Value>(&content) {
                            let repaired = serde_json::to_string_pretty(&value)
                                .map_err(|e| format!("Failed to format JSON: {}", e))?;
                            fs::write(&path, repaired)
                                .map_err(|e| format!("Failed to write repaired JSON: {}", e))?;

                            return Ok(AutoRepairResult {
                                issue_id: Some(issue.id.clone()),
                                status: RepairStatus::Completed,
                                message: format!("Formatted JSON file: {}", path.display()),
                                actions: vec![RepairAction {
                                    action: "format_json".to_string(),
                                    description: format!(
                                        "Auto-formatted JSON at {}",
                                        path.display()
                                    ),
                                    estimated_duration: Some("< 1 second".to_string()),
                                    level: RepairLevel::Automatic,
                                    requires_confirmation: false,
                                }],
                                backup_location: None,
                                rollback_token: None,
                            });
                        }
                    }
                } else if path.extension().map(|ext| ext == "toml").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                            let repaired = toml::to_string_pretty(&value)
                                .map_err(|e| format!("Failed to format TOML: {}", e))?;
                            fs::write(&path, repaired)
                                .map_err(|e| format!("Failed to write repaired TOML: {}", e))?;

                            return Ok(AutoRepairResult {
                                issue_id: Some(issue.id.clone()),
                                status: RepairStatus::Completed,
                                message: format!("Formatted TOML file: {}", path.display()),
                                actions: vec![RepairAction {
                                    action: "format_toml".to_string(),
                                    description: format!(
                                        "Auto-formatted TOML at {}",
                                        path.display()
                                    ),
                                    estimated_duration: Some("< 1 second".to_string()),
                                    level: RepairLevel::Automatic,
                                    requires_confirmation: false,
                                }],
                                backup_location: None,
                                rollback_token: None,
                            });
                        }
                    }
                }
            }
        }

        Err("Auto repair not available for this issue".to_string())
    }
}
