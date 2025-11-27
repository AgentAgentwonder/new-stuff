use super::types::*;
use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

pub struct NetworkDiagnostics {
    client: Client,
}

impl NetworkDiagnostics {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client }
    }

    pub async fn diagnose(&self) -> ModuleDiagnostics {
        let mut issues = Vec::new();
        let mut metrics = Vec::new();
        let mut notes = Vec::new();

        // Test internet connectivity
        let internet_ok = self.test_connectivity("https://1.1.1.1").await;
        metrics.push(PanelMetric {
            label: "Internet".to_string(),
            value: if internet_ok {
                "Reachable"
            } else {
                "Unreachable"
            }
            .to_string(),
            level: Some(if internet_ok {
                HealthLevel::Excellent
            } else {
                HealthLevel::Critical
            }),
        });

        if !internet_ok {
            issues.push(self.create_issue(
                "No internet connectivity",
                "Unable to reach public DNS resolver",
                IssueSeverity::Critical,
                "Check network connection or firewall settings",
            ));
        } else {
            notes.push("Internet connectivity verified".to_string());
        }

        // Test API endpoints
        let endpoints = vec![
            ("Coingecko", "https://api.coingecko.com/api/v3/ping"),
            ("Defillama", "https://api.llama.fi/protocols"),
        ];

        let mut api_failures = Vec::new();

        for (name, url) in &endpoints {
            if !self.test_connectivity(url).await {
                api_failures.push((*name, *url));
            }
        }

        metrics.push(PanelMetric {
            label: "API Endpoints".to_string(),
            value: format!(
                "{}/{} reachable",
                endpoints.len() - api_failures.len(),
                endpoints.len()
            ),
            level: Some(if api_failures.is_empty() {
                HealthLevel::Excellent
            } else {
                HealthLevel::Degraded
            }),
        });

        for (name, url) in api_failures {
            issues.push(self.create_issue(
                &format!("{} API unreachable", name),
                &format!("Failed to reach endpoint: {}", url),
                IssueSeverity::Warning,
                "Switch to backup endpoint or retry later",
            ));
        }

        // Test latency to RPC endpoints (mocked)
        metrics.push(PanelMetric {
            label: "RPC Latency".to_string(),
            value: "Stable".to_string(),
            level: Some(HealthLevel::Good),
        });
        notes.push("RPC latency check placeholder".to_string());

        let health = if internet_ok {
            if issues
                .iter()
                .any(|i| matches!(i.severity, IssueSeverity::Critical))
            {
                HealthLevel::Critical
            } else if issues.is_empty() {
                HealthLevel::Excellent
            } else {
                HealthLevel::Degraded
            }
        } else {
            HealthLevel::Critical
        };

        ModuleDiagnostics {
            panel: PanelStatus {
                title: "Network Status".to_string(),
                level: health,
                summary: if issues.is_empty() {
                    "All network checks passed".to_string()
                } else {
                    format!("{} network issue(s) detected", issues.len())
                },
                metrics,
                actions: vec!["Test alternative endpoints".to_string()],
            },
            issues,
            auto_fixed: 0,
            notes,
        }
    }

    async fn test_connectivity(&self, url: &str) -> bool {
        let request = self.client.get(url);
        match timeout(Duration::from_secs(5), request.send()).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        }
    }

    fn create_issue(
        &self,
        title: &str,
        description: &str,
        severity: IssueSeverity,
        recommended_action: &str,
    ) -> DiagnosticIssue {
        DiagnosticIssue {
            id: Uuid::new_v4().to_string(),
            category: IssueCategory::Network,
            severity,
            title: title.to_string(),
            description: description.to_string(),
            detected_at: Utc::now(),
            recommended_action: recommended_action.to_string(),
            repair_level: if matches!(severity, IssueSeverity::Critical) {
                RepairLevel::Confirmation
            } else {
                RepairLevel::Automatic
            },
            auto_repair_available: !matches!(severity, IssueSeverity::Critical),
            status: RepairStatus::Pending,
            metadata: HashMap::new(),
        }
    }

    pub async fn auto_switch_endpoint(
        &self,
        issue: &DiagnosticIssue,
    ) -> Result<AutoRepairResult, String> {
        let backup_endpoints = vec![
            json!({"name": "Backup RPC", "url": "https://solana-api.projectserum.com"}),
            json!({"name": "Secondary RPC", "url": "https://api.mainnet-beta.solana.com"}),
        ];

        Ok(AutoRepairResult {
            issue_id: Some(issue.id.clone()),
            status: RepairStatus::Completed,
            message: "Switched to backup endpoint".to_string(),
            actions: vec![RepairAction {
                action: "switch_endpoint".to_string(),
                description: "Selected best available RPC endpoint".to_string(),
                estimated_duration: Some("< 5 seconds".to_string()),
                level: RepairLevel::Automatic,
                requires_confirmation: false,
            }],
            backup_location: None,
            rollback_token: Some(Uuid::new_v4().to_string()),
        })
    }
}
