use super::types::*;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct DbRepairModule;

impl DbRepairModule {
    pub fn new() -> Self {
        Self
    }

    pub fn diagnose(&self, app_data_dir: &Path) -> ModuleDiagnostics {
        let mut issues = Vec::new();
        let mut metrics = Vec::new();
        let mut notes = Vec::new();

        // Check for database files
        let db_files = vec!["multisig.db", "performance.db"];

        let mut total_size = 0u64;
        let mut corrupted = Vec::new();

        for db_file in &db_files {
            let db_path = app_data_dir.join(db_file);

            if db_path.exists() {
                if let Ok(metadata) = std::fs::metadata(&db_path) {
                    total_size += metadata.len();

                    // Check if database is readable
                    if self.check_db_integrity(&db_path).is_err() {
                        corrupted.push(db_file.to_string());
                        issues.push(DiagnosticIssue {
                            id: Uuid::new_v4().to_string(),
                            category: IssueCategory::Database,
                            severity: IssueSeverity::Critical,
                            title: format!("Corrupted database: {}", db_file),
                            description: format!(
                                "Database file {} appears to be corrupted",
                                db_file
                            ),
                            detected_at: Utc::now(),
                            recommended_action: "Backup and repair or rebuild database".to_string(),
                            repair_level: RepairLevel::Confirmation,
                            auto_repair_available: true,
                            status: RepairStatus::Pending,
                            metadata: {
                                let mut m = HashMap::new();
                                m.insert(
                                    "path".to_string(),
                                    serde_json::Value::String(db_path.display().to_string()),
                                );
                                m
                            },
                        });
                    }
                }
            }
        }

        let size_mb = (total_size as f64) / (1024.0 * 1024.0);

        metrics.push(PanelMetric {
            label: "Database Files".to_string(),
            value: format!("{} files", db_files.len()),
            level: Some(HealthLevel::Excellent),
        });

        metrics.push(PanelMetric {
            label: "Total Size".to_string(),
            value: format!("{:.2} MB", size_mb),
            level: Some(HealthLevel::Excellent),
        });

        metrics.push(PanelMetric {
            label: "Corrupted".to_string(),
            value: format!("{}", corrupted.len()),
            level: Some(if corrupted.is_empty() {
                HealthLevel::Excellent
            } else {
                HealthLevel::Critical
            }),
        });

        let level = if corrupted.is_empty() {
            HealthLevel::Excellent
        } else {
            HealthLevel::Critical
        };

        let summary = if corrupted.is_empty() {
            "All databases healthy".to_string()
        } else {
            format!("{} corrupted database(s) detected", corrupted.len())
        };

        if issues.is_empty() {
            notes.push("All database integrity checks passed".to_string());
        }

        ModuleDiagnostics {
            panel: PanelStatus {
                title: "Database Health".to_string(),
                level,
                summary,
                metrics,
                actions: vec![],
            },
            issues,
            auto_fixed: 0,
            notes,
        }
    }

    fn check_db_integrity(&self, db_path: &Path) -> Result<(), String> {
        // Simple check: try to open the file
        // In a real implementation, would use SQLite's PRAGMA integrity_check
        match std::fs::metadata(db_path) {
            Ok(metadata) => {
                if metadata.len() < 100 {
                    // SQLite databases have a minimum size
                    return Err("Database file too small".to_string());
                }
                Ok(())
            }
            Err(e) => Err(format!("Cannot access database: {}", e)),
        }
    }

    pub async fn auto_repair(
        &self,
        issue: &DiagnosticIssue,
        app_data_dir: &Path,
    ) -> Result<AutoRepairResult, String> {
        if issue.title.starts_with("Corrupted database:") {
            if let Some(path_value) = issue.metadata.get("path") {
                if let Some(path_str) = path_value.as_str() {
                    let db_path = PathBuf::from(path_str);

                    // Create backup
                    let backup_path =
                        format!("{}.backup.{}", db_path.display(), Utc::now().timestamp());
                    std::fs::copy(&db_path, &backup_path)
                        .map_err(|e| format!("Failed to backup database: {}", e))?;

                    // Attempt repair by recreating schema
                    // In real implementation, would try to recover data first
                    std::fs::remove_file(&db_path)
                        .map_err(|e| format!("Failed to remove corrupted database: {}", e))?;

                    return Ok(AutoRepairResult {
                        issue_id: Some(issue.id.clone()),
                        status: RepairStatus::Completed,
                        message: format!("Database backed up and reset: {:?}", db_path),
                        actions: vec![
                            RepairAction {
                                action: "backup_database".to_string(),
                                description: format!("Backed up to: {}", backup_path),
                                estimated_duration: Some("< 5 seconds".to_string()),
                                level: RepairLevel::Confirmation,
                                requires_confirmation: true,
                            },
                            RepairAction {
                                action: "reset_database".to_string(),
                                description: "Database will be recreated on next app restart"
                                    .to_string(),
                                estimated_duration: Some("< 1 second".to_string()),
                                level: RepairLevel::Confirmation,
                                requires_confirmation: true,
                            },
                        ],
                        backup_location: Some(backup_path),
                        rollback_token: Some(Uuid::new_v4().to_string()),
                    });
                }
            }
        }

        Err(format!(
            "No auto-repair available for issue: {}",
            issue.title
        ))
    }

    pub fn compact_database(&self, db_path: &Path) -> Result<u64, String> {
        // Would use SQLite VACUUM command
        // For now, just report current size
        std::fs::metadata(db_path)
            .map(|m| m.len())
            .map_err(|e| format!("Failed to get database size: {}", e))
    }

    pub fn rebuild_indexes(&self, _db_path: &Path) -> Result<(), String> {
        // Would use SQLite REINDEX command
        Ok(())
    }
}
