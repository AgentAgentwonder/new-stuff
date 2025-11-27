use super::types::*;
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;
use sysinfo::{CpuExt, System, SystemExt};
use uuid::Uuid;

pub struct PerformanceRepairModule {
    sys: System,
}

impl PerformanceRepairModule {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    pub fn diagnose(&mut self) -> ModuleDiagnostics {
        let mut issues = Vec::new();
        let mut metrics = Vec::new();
        let mut notes = Vec::new();

        // Refresh system info
        self.sys.refresh_all();

        // Memory checks
        let total_memory = self.sys.total_memory();
        let used_memory = self.sys.used_memory();
        let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;

        metrics.push(PanelMetric {
            label: "Memory Usage".to_string(),
            value: format!("{:.1}%", memory_usage_percent),
            level: Some(if memory_usage_percent < 80.0 {
                HealthLevel::Excellent
            } else if memory_usage_percent < 90.0 {
                HealthLevel::Good
            } else {
                HealthLevel::Degraded
            }),
        });

        if memory_usage_percent > 90.0 {
            issues.push(DiagnosticIssue {
                id: Uuid::new_v4().to_string(),
                category: IssueCategory::Performance,
                severity: IssueSeverity::Warning,
                title: "High memory usage".to_string(),
                description: format!("System memory usage is at {:.1}%", memory_usage_percent),
                detected_at: Utc::now(),
                recommended_action: "Clear caches to free memory".to_string(),
                repair_level: RepairLevel::Automatic,
                auto_repair_available: true,
                status: RepairStatus::Pending,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert(
                        "memory_percent".to_string(),
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(memory_usage_percent).unwrap(),
                        ),
                    );
                    m
                },
            });
        } else {
            notes.push(format!(
                "Memory usage is healthy at {:.1}%",
                memory_usage_percent
            ));
        }

        // CPU checks
        let cpu_count = self.sys.cpus().len();
        metrics.push(PanelMetric {
            label: "CPU Cores".to_string(),
            value: format!("{}", cpu_count),
            level: Some(HealthLevel::Excellent),
        });

        // Process count (simulated)
        let process_count = self.sys.processes().len();
        metrics.push(PanelMetric {
            label: "Running Processes".to_string(),
            value: format!("{}", process_count),
            level: Some(HealthLevel::Good),
        });
        notes.push(format!("{} system processes running", process_count));

        // Disk space checks are done in file repair module

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

        ModuleDiagnostics {
            panel: PanelStatus {
                title: "Performance Metrics".to_string(),
                level,
                summary: if issues.is_empty() {
                    "System performance is optimal".to_string()
                } else {
                    format!("{} performance issue(s) detected", issues.len())
                },
                metrics,
                actions: vec!["Clear caches".to_string(), "Optimize settings".to_string()],
            },
            issues,
            auto_fixed: 0,
            notes,
        }
    }

    pub fn auto_repair(
        &self,
        issue: &DiagnosticIssue,
        app_data_dir: &Path,
    ) -> Result<AutoRepairResult, String> {
        match issue.title.as_str() {
            "High memory usage" => {
                // Clear caches
                let cache_dir = app_data_dir.join("cache");
                let cleared = self.clear_cache_dir(&cache_dir)?;

                Ok(AutoRepairResult {
                    issue_id: Some(issue.id.clone()),
                    status: RepairStatus::Completed,
                    message: format!("Cleared {} cache files", cleared),
                    actions: vec![RepairAction {
                        action: "clear_caches".to_string(),
                        description: format!("Removed {} cache files to free memory", cleared),
                        estimated_duration: Some("< 5 seconds".to_string()),
                        level: RepairLevel::Automatic,
                        requires_confirmation: false,
                    }],
                    backup_location: None,
                    rollback_token: None,
                })
            }
            _ => Err(format!("No auto-repair available for: {}", issue.title)),
        }
    }

    fn clear_cache_dir(&self, cache_dir: &Path) -> Result<usize, String> {
        if !cache_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(cache_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    if std::fs::remove_file(entry.path()).is_ok() {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    pub fn get_memory_stats(&mut self) -> (u64, u64, f64) {
        self.sys.refresh_memory();
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let percent = (used as f64 / total as f64) * 100.0;
        (total, used, percent)
    }

    pub fn get_cpu_usage(&mut self) -> f32 {
        self.sys.refresh_cpu();
        self.sys.global_cpu_info().cpu_usage()
    }
}
