use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUsageRecord {
    pub service: String,
    pub endpoint: String,
    pub timestamp: DateTime<Utc>,
    pub status_code: u16,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageStats {
    pub service: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub average_latency_ms: f64,
    pub estimated_cost: f64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointUsage {
    pub endpoint: String,
    pub call_count: u64,
    pub average_latency_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiUsageAnalytics {
    pub services: HashMap<String, UsageStats>,
    pub endpoint_breakdown: HashMap<String, Vec<EndpointUsage>>,
    pub daily_calls: HashMap<String, u64>,
    pub alerts: Vec<UsageAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageAlert {
    pub service: String,
    pub alert_type: AlertType,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    ApproachingLimit,
    LimitExceeded,
    HighLatency,
    HighErrorRate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FairUseLimit {
    pub service: String,
    pub daily_limit: u64,
    pub monthly_limit: u64,
    pub current_daily_usage: u64,
    pub current_monthly_usage: u64,
    pub reset_at: DateTime<Utc>,
}

pub struct ApiUsageTracker {
    usage_log: Arc<Mutex<Vec<ApiUsageRecord>>>,
    fair_use_limits: Arc<Mutex<HashMap<String, FairUseLimit>>>,
    data_path: PathBuf,
}

impl ApiUsageTracker {
    pub fn new(data_path: PathBuf) -> Result<Self, String> {
        let tracker = Self {
            usage_log: Arc::new(Mutex::new(Vec::new())),
            fair_use_limits: Arc::new(Mutex::new(HashMap::new())),
            data_path,
        };

        tracker.load_usage_data()?;
        tracker.initialize_fair_use_limits()?;

        Ok(tracker)
    }

    fn load_usage_data(&self) -> Result<(), String> {
        if self.data_path.exists() {
            let data = fs::read_to_string(&self.data_path)
                .map_err(|e| format!("Failed to read usage data: {}", e))?;
            let records: Vec<ApiUsageRecord> = serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse usage data: {}", e))?;

            if let Ok(mut log) = self.usage_log.lock() {
                *log = records;
            }
        }
        Ok(())
    }

    fn save_usage_data(&self) -> Result<(), String> {
        if let Some(parent) = self.data_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }

        if let Ok(log) = self.usage_log.lock() {
            let data = serde_json::to_string_pretty(&*log)
                .map_err(|e| format!("Failed to serialize usage data: {}", e))?;
            fs::write(&self.data_path, data)
                .map_err(|e| format!("Failed to write usage data: {}", e))?;
        }
        Ok(())
    }

    fn initialize_fair_use_limits(&self) -> Result<(), String> {
        let mut limits = self
            .fair_use_limits
            .lock()
            .map_err(|_| "Failed to lock fair use limits".to_string())?;

        // Default fair-use limits for developer keys
        let services = vec![
            ("helius", 10000, 300000),
            ("birdeye", 5000, 150000),
            ("jupiter", 20000, 600000),
            ("solana_rpc", 50000, 1500000),
        ];

        for (service, daily, monthly) in services {
            limits.insert(
                service.to_string(),
                FairUseLimit {
                    service: service.to_string(),
                    daily_limit: daily,
                    monthly_limit: monthly,
                    current_daily_usage: 0,
                    current_monthly_usage: 0,
                    reset_at: calculate_next_reset(),
                },
            );
        }

        Ok(())
    }

    pub fn record_usage(&self, record: ApiUsageRecord) -> Result<(), String> {
        // Update usage log
        if let Ok(mut log) = self.usage_log.lock() {
            log.push(record.clone());

            // Keep only last 30 days
            let cutoff = Utc::now() - chrono::Duration::days(30);
            log.retain(|r| r.timestamp > cutoff);
        }

        // Update fair use limits
        if let Ok(mut limits) = self.fair_use_limits.lock() {
            if let Some(limit) = limits.get_mut(&record.service) {
                limit.current_daily_usage += 1;
                limit.current_monthly_usage += 1;

                // Reset if needed
                if Utc::now() > limit.reset_at {
                    limit.current_daily_usage = 0;
                    limit.reset_at = calculate_next_reset();
                }
            }
        }

        self.save_usage_data()?;
        Ok(())
    }

    pub fn get_analytics(&self, days: i64) -> Result<ApiUsageAnalytics, String> {
        let log = self
            .usage_log
            .lock()
            .map_err(|_| "Failed to lock usage log".to_string())?;

        let cutoff = Utc::now() - chrono::Duration::days(days);
        let recent_records: Vec<_> = log.iter().filter(|r| r.timestamp > cutoff).collect();

        let mut services: HashMap<String, UsageStats> = HashMap::new();
        let mut endpoint_breakdown: HashMap<String, Vec<EndpointUsage>> = HashMap::new();
        let mut daily_calls: HashMap<String, u64> = HashMap::new();

        // Group by service
        for record in &recent_records {
            let entry = services
                .entry(record.service.clone())
                .or_insert(UsageStats {
                    service: record.service.clone(),
                    total_calls: 0,
                    successful_calls: 0,
                    failed_calls: 0,
                    average_latency_ms: 0.0,
                    estimated_cost: 0.0,
                    period_start: cutoff,
                    period_end: Utc::now(),
                });

            entry.total_calls += 1;
            if record.status_code >= 200 && record.status_code < 300 {
                entry.successful_calls += 1;
            } else {
                entry.failed_calls += 1;
            }

            // Update daily calls
            let date_key = record.timestamp.format("%Y-%m-%d").to_string();
            *daily_calls.entry(date_key).or_insert(0) += 1;
        }

        // Calculate averages and costs
        for (service, stats) in services.iter_mut() {
            let service_records: Vec<_> = recent_records
                .iter()
                .filter(|r| &r.service == service)
                .collect();

            if !service_records.is_empty() {
                let total_latency: u64 = service_records.iter().map(|r| r.latency_ms).sum();
                stats.average_latency_ms = total_latency as f64 / service_records.len() as f64;
            }

            // Estimate costs (example rates per 1000 calls)
            stats.estimated_cost = match service.as_str() {
                "helius" => stats.total_calls as f64 * 0.01 / 1000.0,
                "birdeye" => stats.total_calls as f64 * 0.02 / 1000.0,
                "jupiter" => stats.total_calls as f64 * 0.005 / 1000.0,
                "solana_rpc" => stats.total_calls as f64 * 0.001 / 1000.0,
                _ => 0.0,
            };

            // Build endpoint breakdown
            let mut endpoint_map: HashMap<String, (u64, u64, u64)> = HashMap::new();
            for record in &service_records {
                let entry = endpoint_map
                    .entry(record.endpoint.clone())
                    .or_insert((0, 0, 0));
                entry.0 += 1; // count
                entry.1 += record.latency_ms; // total latency
                entry.2 += if record.status_code >= 200 && record.status_code < 300 {
                    1
                } else {
                    0
                }; // success count
            }

            let mut endpoints = Vec::new();
            for (endpoint, (count, total_latency, success_count)) in endpoint_map {
                endpoints.push(EndpointUsage {
                    endpoint,
                    call_count: count,
                    average_latency_ms: total_latency as f64 / count as f64,
                    success_rate: success_count as f64 / count as f64,
                });
            }
            endpoints.sort_by(|a, b| b.call_count.cmp(&a.call_count));
            endpoint_breakdown.insert(service.clone(), endpoints);
        }

        // Generate alerts
        let alerts = self.generate_alerts(&services)?;

        Ok(ApiUsageAnalytics {
            services,
            endpoint_breakdown,
            daily_calls,
            alerts,
        })
    }

    fn generate_alerts(
        &self,
        services: &HashMap<String, UsageStats>,
    ) -> Result<Vec<UsageAlert>, String> {
        let mut alerts = Vec::new();
        let limits = self
            .fair_use_limits
            .lock()
            .map_err(|_| "Failed to lock fair use limits".to_string())?;

        for (service, limit) in limits.iter() {
            // Check daily limit
            let daily_usage_percent =
                (limit.current_daily_usage as f64 / limit.daily_limit as f64) * 100.0;
            if daily_usage_percent >= 90.0 {
                alerts.push(UsageAlert {
                    service: service.clone(),
                    alert_type: if daily_usage_percent >= 100.0 {
                        AlertType::LimitExceeded
                    } else {
                        AlertType::ApproachingLimit
                    },
                    message: format!(
                        "Daily usage at {:.1}% ({}/{})",
                        daily_usage_percent, limit.current_daily_usage, limit.daily_limit
                    ),
                    timestamp: Utc::now(),
                });
            }

            // Check error rate
            if let Some(stats) = services.get(service) {
                if stats.total_calls > 100 {
                    let error_rate = (stats.failed_calls as f64 / stats.total_calls as f64) * 100.0;
                    if error_rate > 10.0 {
                        alerts.push(UsageAlert {
                            service: service.clone(),
                            alert_type: AlertType::HighErrorRate,
                            message: format!("Error rate: {:.1}%", error_rate),
                            timestamp: Utc::now(),
                        });
                    }
                }

                // Check latency
                if stats.average_latency_ms > 1000.0 {
                    alerts.push(UsageAlert {
                        service: service.clone(),
                        alert_type: AlertType::HighLatency,
                        message: format!("Average latency: {:.0}ms", stats.average_latency_ms),
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        Ok(alerts)
    }

    pub fn get_fair_use_limits(&self) -> Result<Vec<FairUseLimit>, String> {
        let limits = self
            .fair_use_limits
            .lock()
            .map_err(|_| "Failed to lock fair use limits".to_string())?;
        Ok(limits.values().cloned().collect())
    }
}

fn calculate_next_reset() -> DateTime<Utc> {
    let now = Utc::now();
    now + chrono::Duration::days(1)
}

#[tauri::command]
pub async fn record_api_usage(
    service: String,
    endpoint: String,
    status_code: u16,
    latency_ms: u64,
    tracker: State<'_, Arc<Mutex<ApiUsageTracker>>>,
) -> Result<(), String> {
    let tracker = tracker
        .lock()
        .map_err(|_| "Failed to lock usage tracker".to_string())?;

    let record = ApiUsageRecord {
        service,
        endpoint,
        timestamp: Utc::now(),
        status_code,
        latency_ms,
    };

    tracker.record_usage(record)
}

#[tauri::command]
pub async fn get_api_analytics(
    days: Option<i64>,
    tracker: State<'_, Arc<Mutex<ApiUsageTracker>>>,
) -> Result<ApiUsageAnalytics, String> {
    let tracker = tracker
        .lock()
        .map_err(|_| "Failed to lock usage tracker".to_string())?;

    tracker.get_analytics(days.unwrap_or(30))
}

#[tauri::command]
pub async fn get_fair_use_status(
    tracker: State<'_, Arc<Mutex<ApiUsageTracker>>>,
) -> Result<Vec<FairUseLimit>, String> {
    let tracker = tracker
        .lock()
        .map_err(|_| "Failed to lock usage tracker".to_string())?;

    tracker.get_fair_use_limits()
}

pub fn initialize_usage_tracker(app: &AppHandle) -> Result<Arc<Mutex<ApiUsageTracker>>, String> {
    let mut data_path = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("Unable to resolve app data directory: {err}"))?;

    data_path.push("api_usage.json");

    let tracker = ApiUsageTracker::new(data_path)?;
    Ok(Arc::new(Mutex::new(tracker)))
}
