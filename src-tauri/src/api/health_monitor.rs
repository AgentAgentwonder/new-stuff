use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

const HEALTH_DB_FILE: &str = "api_health.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiHealthMetrics {
    pub service_name: String,
    pub uptime_percent: f64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub last_success: Option<DateTime<Utc>>,
    pub last_failure: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub rate_limit_info: Option<RateLimitInfo>,
    pub failover_active: bool,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    pub limit: i64,
    pub remaining: i64,
    pub reset_at: DateTime<Utc>,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckRecord {
    pub id: String,
    pub service_name: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub latency_ms: i64,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSeriesDataPoint {
    pub timestamp: DateTime<Utc>,
    pub latency_ms: f64,
    pub error_rate: f64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiHealthDashboard {
    pub services: HashMap<String, ApiHealthMetrics>,
    pub history: HashMap<String, Vec<TimeSeriesDataPoint>>,
    pub overall_health: HealthStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum HealthMonitorError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ApiHealthMonitor {
    pool: Pool<Sqlite>,
}

pub type SharedApiHealthMonitor = Arc<RwLock<ApiHealthMonitor>>;

impl ApiHealthMonitor {
    pub async fn new(app: &AppHandle) -> Result<Self, HealthMonitorError> {
        let db_path = Self::health_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: ApiHealthMonitor failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for ApiHealthMonitor");
                eprintln!("ApiHealthMonitor using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        let monitor = Self { pool };
        monitor.initialize().await?;
        Ok(monitor)
    }

    fn health_db_path(app: &AppHandle) -> Result<std::path::PathBuf, HealthMonitorError> {
        let mut path = app.path().app_data_dir().map_err(|err| {
            HealthMonitorError::Internal(format!("Unable to resolve app data directory: {err}"))
        })?;

        std::fs::create_dir_all(&path)?;
        path.push(HEALTH_DB_FILE);
        Ok(path)
    }

    async fn initialize(&self) -> Result<(), HealthMonitorError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS health_checks (
                id TEXT PRIMARY KEY,
                service_name TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                success INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL,
                status_code INTEGER,
                error TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_health_service ON health_checks(service_name);
            CREATE INDEX IF NOT EXISTS idx_health_timestamp ON health_checks(timestamp);
            CREATE INDEX IF NOT EXISTS idx_health_success ON health_checks(success);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_check(&self, record: HealthCheckRecord) -> Result<(), HealthMonitorError> {
        sqlx::query(
            r#"
            INSERT INTO health_checks (id, service_name, timestamp, success, latency_ms, status_code, error)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&record.id)
        .bind(&record.service_name)
        .bind(record.timestamp.to_rfc3339())
        .bind(if record.success { 1 } else { 0 })
        .bind(record.latency_ms.max(0))
        .bind(record.status_code.map(|c| c as i64))
        .bind(&record.error)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_metrics(
        &self,
        service_name: &str,
    ) -> Result<ApiHealthMetrics, HealthMonitorError> {
        let rows = sqlx::query(
            r#"
            SELECT success, latency_ms, status_code, error, timestamp
            FROM health_checks
            WHERE service_name = ?1
            AND timestamp >= datetime('now', '-24 hours')
            ORDER BY timestamp DESC
            "#,
        )
        .bind(service_name)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(ApiHealthMetrics {
                service_name: service_name.to_string(),
                uptime_percent: 0.0,
                avg_latency_ms: 0.0,
                error_rate: 0.0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                last_success: None,
                last_failure: None,
                last_error: None,
                rate_limit_info: None,
                failover_active: false,
                health_status: HealthStatus::Down,
            });
        }

        let total = rows.len() as i64;
        let mut successful = 0i64;
        let mut failed = 0i64;
        let mut total_latency: i64 = 0;
        let mut last_success: Option<DateTime<Utc>> = None;
        let mut last_failure: Option<DateTime<Utc>> = None;
        let mut last_error: Option<String> = None;

        for row in &rows {
            let success: i64 = row.try_get("success")?;
            let latency: i64 = row.try_get::<i64, _>("latency_ms")?.max(0);
            let timestamp_str: String = row.try_get("timestamp")?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| HealthMonitorError::Internal(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&Utc);

            total_latency = total_latency.saturating_add(latency);

            if success == 1 {
                successful += 1;
                if last_success.is_none() {
                    last_success = Some(timestamp);
                }
            } else {
                failed += 1;
                if last_failure.is_none() {
                    last_failure = Some(timestamp);
                    last_error = row.try_get("error")?;
                }
            }
        }

        let uptime_percent = (successful as f64 / total as f64) * 100.0;
        let avg_latency_ms = (total_latency as f64) / (total as f64);
        let error_rate = (failed as f64 / total as f64) * 100.0;

        let health_status = if error_rate < 1.0 {
            HealthStatus::Healthy
        } else if error_rate < 10.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Down
        };

        Ok(ApiHealthMetrics {
            service_name: service_name.to_string(),
            uptime_percent,
            avg_latency_ms,
            error_rate,
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            last_success,
            last_failure,
            last_error,
            rate_limit_info: None, // Would be populated from external source
            failover_active: false,
            health_status,
        })
    }

    pub async fn get_dashboard(&self) -> Result<ApiHealthDashboard, HealthMonitorError> {
        let service_names = vec!["helius", "birdeye", "jupiter", "solana_rpc"];
        let mut services = HashMap::new();
        let mut history = HashMap::new();

        for service in service_names {
            let metrics = self.get_metrics(service).await?;
            let time_series = self.get_time_series(service, 24).await?;

            services.insert(service.to_string(), metrics);
            history.insert(service.to_string(), time_series);
        }

        let overall_health = Self::calculate_overall_health(&services);

        Ok(ApiHealthDashboard {
            services,
            history,
            overall_health,
        })
    }

    pub async fn get_time_series(
        &self,
        service_name: &str,
        hours: i64,
    ) -> Result<Vec<TimeSeriesDataPoint>, HealthMonitorError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                strftime('%Y-%m-%dT%H:00:00Z', timestamp) as hour,
                AVG(latency_ms) as avg_latency,
                SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) * 100.0 / COUNT(*) as error_rate,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) * 100.0 / COUNT(*) as success_rate
            FROM health_checks
            WHERE service_name = ?1
            AND timestamp >= datetime('now', '-' || ?2 || ' hours')
            GROUP BY hour
            ORDER BY hour DESC
            "#,
        )
        .bind(service_name)
        .bind(hours)
        .fetch_all(&self.pool)
        .await?;

        let mut points = Vec::new();
        for row in rows {
            let hour: String = row.try_get("hour")?;
            let timestamp = DateTime::parse_from_rfc3339(&hour)
                .map_err(|e| HealthMonitorError::Internal(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&Utc);

            points.push(TimeSeriesDataPoint {
                timestamp,
                latency_ms: row.try_get("avg_latency")?,
                error_rate: row.try_get("error_rate")?,
                success_rate: row.try_get("success_rate")?,
            });
        }

        Ok(points)
    }

    pub async fn cleanup_old_records(&self, days: i64) -> Result<usize, HealthMonitorError> {
        let result = sqlx::query(
            r#"
            DELETE FROM health_checks 
            WHERE timestamp < datetime('now', '-' || ?1 || ' days')
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    fn calculate_overall_health(services: &HashMap<String, ApiHealthMetrics>) -> HealthStatus {
        if services
            .values()
            .any(|m| m.health_status == HealthStatus::Down)
        {
            HealthStatus::Degraded
        } else if services
            .values()
            .all(|m| m.health_status == HealthStatus::Healthy)
        {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        }
    }
}
