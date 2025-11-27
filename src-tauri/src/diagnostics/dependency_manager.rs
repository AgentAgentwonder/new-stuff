use super::types::*;
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: HashMap<String, Value>,
    #[serde(default)]
    dev_dependencies: HashMap<String, Value>,
}

pub struct DependencyManager;

impl DependencyManager {
    pub fn new() -> Self {
        Self
    }

    pub fn diagnose(&self, project_root: &Path) -> ModuleDiagnostics {
        let mut issues = Vec::new();
        let mut metrics = Vec::new();
        let mut notes = Vec::new();
        let mut auto_fixed = 0;

        let package_json_path = project_root.join("package.json");
        let node_modules_path = project_root.join("node_modules");

        let package_json_exists = package_json_path.exists();
        let node_modules_exists = node_modules_path.exists();

        metrics.push(PanelMetric {
            label: "package.json".to_string(),
            value: if package_json_exists {
                "Found"
            } else {
                "Missing"
            }
            .to_string(),
            level: Some(if package_json_exists {
                HealthLevel::Excellent
            } else {
                HealthLevel::Critical
            }),
        });

        metrics.push(PanelMetric {
            label: "node_modules".to_string(),
            value: if node_modules_exists {
                "Present"
            } else {
                "Missing"
            }
            .to_string(),
            level: Some(if node_modules_exists {
                HealthLevel::Excellent
            } else {
                HealthLevel::Critical
            }),
        });

        if !package_json_exists {
            issues.push(DiagnosticIssue {
                id: Uuid::new_v4().to_string(),
                category: IssueCategory::Dependencies,
                severity: IssueSeverity::Critical,
                title: "package.json missing".to_string(),
                description: "The package.json file is required to manage dependencies."
                    .to_string(),
                detected_at: Utc::now(),
                recommended_action: "Restore package.json from backup or defaults.".to_string(),
                repair_level: RepairLevel::Manual,
                auto_repair_available: false,
                status: RepairStatus::Pending,
                metadata: HashMap::new(),
            });
        } else {
            match fs::read_to_string(&package_json_path) {
                Ok(content) => {
                    match serde_json::from_str::<PackageJson>(&content) {
                        Ok(package_json) => {
                            let missing_dependencies =
                                self.detect_missing_dependencies(&package_json, &node_modules_path);

                            if !missing_dependencies.is_empty() {
                                issues.push(DiagnosticIssue {
                                    id: Uuid::new_v4().to_string(),
                                    category: IssueCategory::Dependencies,
                                    severity: IssueSeverity::Critical,
                                    title: "Missing npm packages".to_string(),
                                    description: format!(
                                        "Missing dependencies: {}",
                                        missing_dependencies.join(", ")
                                    ),
                                    detected_at: Utc::now(),
                                    recommended_action: "Install missing npm packages".to_string(),
                                    repair_level: RepairLevel::Confirmation,
                                    auto_repair_available: true,
                                    status: RepairStatus::Pending,
                                    metadata: {
                                        let mut metadata = HashMap::new();
                                        metadata.insert(
                                            "missing_dependencies".to_string(),
                                            Value::Array(
                                                missing_dependencies
                                                    .into_iter()
                                                    .map(Value::String)
                                                    .collect(),
                                            ),
                                        );
                                        metadata
                                    },
                                });
                            }

                            // TODO: detect outdated dependencies by checking versions
                            notes.push("Dependency manifest parsed successfully".to_string());
                        }
                        Err(e) => {
                            issues.push(DiagnosticIssue {
                                id: Uuid::new_v4().to_string(),
                                category: IssueCategory::Dependencies,
                                severity: IssueSeverity::Warning,
                                title: "Invalid package.json".to_string(),
                                description: format!("package.json is invalid JSON: {}", e),
                                detected_at: Utc::now(),
                                recommended_action: "Repair package.json formatting".to_string(),
                                repair_level: RepairLevel::Confirmation,
                                auto_repair_available: false,
                                status: RepairStatus::Pending,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
                Err(e) => {
                    issues.push(DiagnosticIssue {
                        id: Uuid::new_v4().to_string(),
                        category: IssueCategory::Dependencies,
                        severity: IssueSeverity::Warning,
                        title: "Cannot read package.json".to_string(),
                        description: format!("Failed to read package.json: {}", e),
                        detected_at: Utc::now(),
                        recommended_action: "Check file permissions and accessibility".to_string(),
                        repair_level: RepairLevel::Confirmation,
                        auto_repair_available: false,
                        status: RepairStatus::Pending,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        if !node_modules_exists && package_json_exists {
            issues.push(DiagnosticIssue {
                id: Uuid::new_v4().to_string(),
                category: IssueCategory::Dependencies,
                severity: IssueSeverity::Critical,
                title: "node_modules missing".to_string(),
                description:
                    "The node_modules directory is missing; dependencies need to be installed."
                        .to_string(),
                detected_at: Utc::now(),
                recommended_action: "Run npm install to restore node_modules".to_string(),
                repair_level: RepairLevel::Confirmation,
                auto_repair_available: true,
                status: RepairStatus::Pending,
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert(
                        "action".to_string(),
                        Value::String("npm install".to_string()),
                    );
                    metadata
                },
            });
        }

        let level = if issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Critical))
        {
            HealthLevel::Critical
        } else if issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Warning))
        {
            HealthLevel::Degraded
        } else {
            HealthLevel::Excellent
        };

        let summary = if issues.is_empty() {
            "All dependencies verified".to_string()
        } else {
            format!("{} dependency issue(s) detected", issues.len())
        };

        ModuleDiagnostics {
            panel: PanelStatus {
                title: "Dependencies".to_string(),
                level,
                summary,
                metrics,
                actions: vec![
                    "Install missing packages".to_string(),
                    "Repair node_modules".to_string(),
                ],
            },
            issues,
            auto_fixed,
            notes,
        }
    }

    fn detect_missing_dependencies(
        &self,
        package_json: &PackageJson,
        node_modules: &Path,
    ) -> Vec<String> {
        let mut missing = Vec::new();

        for (name, _version) in package_json
            .dependencies
            .iter()
            .chain(package_json.dev_dependencies.iter())
        {
            if !node_modules.join(name).exists() {
                missing.push(name.clone());
            }
        }

        missing
    }

    pub async fn install_dependencies(
        &self,
        project_root: &Path,
        packages: Option<Vec<String>>,
    ) -> Result<AutoRepairResult, String> {
        let mut command = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C");
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c");
            cmd
        };

        let command_str = if let Some(packages) = packages {
            if packages.is_empty() {
                "npm install".to_string()
            } else {
                format!("npm install {}", packages.join(" "))
            }
        } else {
            "npm install".to_string()
        };

        command.arg(&command_str);
        command.current_dir(project_root);

        let output = command
            .output()
            .await
            .map_err(|e| format!("Failed to spawn npm install: {}", e))?;

        if output.status.success() {
            // Allow filesystem to settle
            sleep(Duration::from_millis(500)).await;

            Ok(AutoRepairResult {
                issue_id: None,
                status: RepairStatus::Completed,
                message: "Dependencies installed successfully".to_string(),
                actions: vec![RepairAction {
                    action: "npm_install".to_string(),
                    description: command_str,
                    estimated_duration: Some("1-5 minutes".to_string()),
                    level: RepairLevel::Confirmation,
                    requires_confirmation: true,
                }],
                backup_location: None,
                rollback_token: None,
            })
        } else {
            Err(format!(
                "npm install failed with status {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub fn auto_repair(&self, issue: &DiagnosticIssue, project_root: &Path) -> Option<Vec<String>> {
        match issue.title.as_str() {
            "node_modules missing" => Some(vec!["npm install".to_string()]),
            "Missing npm packages" => issue
                .metadata
                .get("missing_dependencies")
                .and_then(|value| value.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|value| value.as_str().map(String::from))
                        .collect::<Vec<_>>()
                }),
            _ => None,
        }
    }

    pub fn verify(&self, project_root: &Path) -> bool {
        let node_modules_path = project_root.join("node_modules");
        node_modules_path.exists()
    }

    pub fn detect_outdated_dependencies(&self, _project_root: &Path) -> Vec<DiagnosticIssue> {
        // TODO: integrate with npm outdated or package registry
        Vec::new()
    }

    pub fn repair_node_modules(&self, project_root: &Path) -> Result<(), String> {
        let node_modules_path = project_root.join("node_modules");
        if node_modules_path.exists() {
            fs::remove_dir_all(&node_modules_path)
                .map_err(|e| format!("Failed to remove node_modules: {}", e))?;
        }

        Ok(())
    }

    pub fn ensure_runtime_libraries(&self, _project_root: &Path) -> Vec<DiagnosticIssue> {
        // TODO: verify native dependencies (Rust toolchain, Tauri requirements)
        Vec::new()
    }
}
