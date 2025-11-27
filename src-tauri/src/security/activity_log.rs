use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::ser::Serialize as SerializeValue;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteArguments;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};

const ACTIVITY_DB_FILE: &str = "activity_logs.db";
const ACTIVITY_CONFIG_FILE: &str = "activity_log_config.json";
pub const DEFAULT_RETENTION_DAYS: i64 = 90;
const MAX_RETENTION_DAYS: i64 = 3650; // ~10 years

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityAction {
    Connect,
    Disconnect,
    Sign,
    Send,
    Swap,
    Approve,
    Reject,
}

impl ActivityAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivityAction::Connect => "connect",
            ActivityAction::Disconnect => "disconnect",
            ActivityAction::Sign => "sign",
            ActivityAction::Send => "send",
            ActivityAction::Swap => "swap",
            ActivityAction::Approve => "approve",
            ActivityAction::Reject => "reject",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLog {
    pub id: i64,
    pub wallet_address: String,
    pub action: String,
    pub details_json: String,
    pub ip_address: Option<String>,
    pub timestamp: String,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLogFilter {
    pub wallet_address: Option<String>,
    pub action: Option<String>,
    pub result: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityStats {
    pub total_actions: i64,
    pub actions_today: i64,
    pub actions_this_week: i64,
    pub actions_this_month: i64,
    pub success_rate: f64,
    pub action_type_counts: HashMap<String, i64>,
    pub suspicious_activities: Vec<SuspiciousActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuspiciousActivity {
    pub activity_type: String,
    pub description: String,
    pub timestamp: String,
    pub wallet_address: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActivityLogConfig {
    retention_days: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ActivityLogError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid retention period: {0}")]
    InvalidRetention(String),
    #[error("invalid timestamp filter: {0}")]
    InvalidTimestamp(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Clone)]
pub struct ActivityLogger {
    pool: Pool<Sqlite>,
    retention_days: Arc<RwLock<i64>>,
    config_path: Arc<PathBuf>,
}

enum BindValue {
    Text(String),
    Integer(i64),
}

impl ActivityLogger {
    pub async fn new(app: &AppHandle) -> Result<Self, ActivityLogError> {
        let db_path = activity_log_path(app)?;
        let config_path = activity_config_path(app)?;
        Self::new_with_paths(db_path, config_path).await
    }

    pub async fn new_with_paths(
        db_path: PathBuf,
        config_path: PathBuf,
    ) -> Result<Self, ActivityLogError> {
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let retention_days = load_retention_days(&config_path)?;

        let logger = Self {
            pool,
            retention_days: Arc::new(RwLock::new(retention_days)),
            config_path: Arc::new(config_path.into()),
        };

        logger.initialize().await?;
        Ok(logger)
    }

    async fn initialize(&self) -> Result<(), ActivityLogError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS activity_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_address TEXT NOT NULL,
                action TEXT NOT NULL,
                details_json TEXT NOT NULL,
                ip_address TEXT,
                timestamp TEXT NOT NULL,
                result TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_activity_wallet ON activity_logs(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_activity_timestamp ON activity_logs(timestamp);
            CREATE INDEX IF NOT EXISTS idx_activity_action ON activity_logs(action);
            CREATE INDEX IF NOT EXISTS idx_activity_result ON activity_logs(result);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn log_activity<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        action: ActivityAction,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        let timestamp = Utc::now().to_rfc3339();
        let result_str = if result { "success" } else { "failure" };
        let details_json = serde_json::to_string(&details)?;

        sqlx::query(
            r#"
            INSERT INTO activity_logs (
                wallet_address, action, details_json, ip_address, timestamp, result
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(wallet_address)
        .bind(action.as_str())
        .bind(details_json)
        .bind(ip_address)
        .bind(timestamp)
        .bind(result_str)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn log_connect<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        self.log_activity(
            wallet_address,
            ActivityAction::Connect,
            details,
            result,
            ip_address,
        )
        .await
    }

    pub async fn log_disconnect<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        self.log_activity(
            wallet_address,
            ActivityAction::Disconnect,
            details,
            result,
            ip_address,
        )
        .await
    }

    pub async fn log_sign<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        self.log_activity(
            wallet_address,
            ActivityAction::Sign,
            details,
            result,
            ip_address,
        )
        .await
    }

    pub async fn log_send<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        self.log_activity(
            wallet_address,
            ActivityAction::Send,
            details,
            result,
            ip_address,
        )
        .await
    }

    pub async fn log_swap<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        self.log_activity(
            wallet_address,
            ActivityAction::Swap,
            details,
            result,
            ip_address,
        )
        .await
    }

    pub async fn log_approve<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        self.log_activity(
            wallet_address,
            ActivityAction::Approve,
            details,
            result,
            ip_address,
        )
        .await
    }

    pub async fn log_reject<T: SerializeValue + Send + Sync>(
        &self,
        wallet_address: &str,
        details: T,
        result: bool,
        ip_address: Option<String>,
    ) -> Result<(), ActivityLogError> {
        self.log_activity(
            wallet_address,
            ActivityAction::Reject,
            details,
            result,
            ip_address,
        )
        .await
    }

    pub async fn get_logs(
        &self,
        filter: ActivityLogFilter,
    ) -> Result<Vec<ActivityLog>, ActivityLogError> {
        let mut sql = String::from(
            "SELECT id, wallet_address, action, details_json, ip_address, timestamp, result FROM activity_logs WHERE 1=1",
        );
        let mut binds: Vec<BindValue> = Vec::new();

        if let Some(wallet) = filter.wallet_address {
            sql.push_str(" AND wallet_address = ?");
            binds.push(BindValue::Text(wallet));
        }
        if let Some(action) = filter.action {
            sql.push_str(" AND action = ?");
            binds.push(BindValue::Text(action));
        }
        if let Some(result) = filter.result {
            sql.push_str(" AND result = ?");
            binds.push(BindValue::Text(result));
        }
        if let Some(start) = filter.start_date {
            let normalized = normalize_timestamp(&start)?;
            sql.push_str(" AND timestamp >= ?");
            binds.push(BindValue::Text(normalized));
        }
        if let Some(end) = filter.end_date {
            let normalized = normalize_timestamp(&end)?;
            sql.push_str(" AND timestamp <= ?");
            binds.push(BindValue::Text(normalized));
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filter.limit {
            if limit > 0 {
                sql.push_str(" LIMIT ?");
                binds.push(BindValue::Integer(limit));
            }
        }
        if let Some(offset) = filter.offset {
            if offset >= 0 {
                sql.push_str(" OFFSET ?");
                binds.push(BindValue::Integer(offset));
            }
        }

        let rows = bind_values(sqlx::query(&sql), binds)
            .fetch_all(&self.pool)
            .await?;

        let logs = rows
            .into_iter()
            .map(|row| ActivityLog {
                id: row.get::<i64, _>("id"),
                wallet_address: row.get::<String, _>("wallet_address"),
                action: row.get::<String, _>("action"),
                details_json: row.get::<String, _>("details_json"),
                ip_address: row.get::<Option<String>, _>("ip_address"),
                timestamp: row.get::<String, _>("timestamp"),
                result: row.get::<String, _>("result"),
            })
            .collect();

        Ok(logs)
    }

    pub async fn export_to_csv(
        &self,
        filter: ActivityLogFilter,
    ) -> Result<String, ActivityLogError> {
        let logs = self.get_logs(filter).await?;
        let mut csv =
            String::from("id,wallet_address,action,details,ip_address,timestamp,result\n");

        for log in logs {
            let escaped_details = log.details_json.replace('"', "\"\"");
            let ip = log.ip_address.unwrap_or_default();
            csv.push_str(&format!(
                "{},{},{},\"{}\",{},{},{}\n",
                log.id,
                log.wallet_address,
                log.action,
                escaped_details,
                ip,
                log.timestamp,
                log.result
            ));
        }

        Ok(csv)
    }

    pub async fn get_stats(
        &self,
        wallet_address: Option<String>,
    ) -> Result<ActivityStats, ActivityLogError> {
        let total_actions = self
            .fetch_count(
                if wallet_address.is_some() {
                    "SELECT COUNT(*) FROM activity_logs WHERE wallet_address = ?"
                } else {
                    "SELECT COUNT(*) FROM activity_logs"
                },
                binds_for_wallet(&wallet_address),
            )
            .await?;

        let now = Utc::now();
        let today_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .to_rfc3339();
        let week_start = (now - ChronoDuration::days(7)).to_rfc3339();
        let month_start = (now - ChronoDuration::days(30)).to_rfc3339();

        let actions_today = self
            .fetch_count(
                if wallet_address.is_some() {
                    "SELECT COUNT(*) FROM activity_logs WHERE wallet_address = ? AND timestamp >= ?"
                } else {
                    "SELECT COUNT(*) FROM activity_logs WHERE timestamp >= ?"
                },
                binds_for_wallet_and_time(&wallet_address, today_start.clone()),
            )
            .await?;

        let actions_week = self
            .fetch_count(
                if wallet_address.is_some() {
                    "SELECT COUNT(*) FROM activity_logs WHERE wallet_address = ? AND timestamp >= ?"
                } else {
                    "SELECT COUNT(*) FROM activity_logs WHERE timestamp >= ?"
                },
                binds_for_wallet_and_time(&wallet_address, week_start.clone()),
            )
            .await?;

        let actions_month = self
            .fetch_count(
                if wallet_address.is_some() {
                    "SELECT COUNT(*) FROM activity_logs WHERE wallet_address = ? AND timestamp >= ?"
                } else {
                    "SELECT COUNT(*) FROM activity_logs WHERE timestamp >= ?"
                },
                binds_for_wallet_and_time(&wallet_address, month_start.clone()),
            )
            .await?;

        let success_count = self
            .fetch_count(
                if wallet_address.is_some() {
                    "SELECT COUNT(*) FROM activity_logs WHERE wallet_address = ? AND result = 'success'"
                } else {
                    "SELECT COUNT(*) FROM activity_logs WHERE result = 'success'"
                },
                binds_for_wallet(&wallet_address),
            )
            .await?;

        let success_rate = if total_actions > 0 {
            (success_count as f64 / total_actions as f64) * 100.0
        } else {
            0.0
        };

        let action_rows = bind_values(
            sqlx::query(
                if wallet_address.is_some() {
                    "SELECT action, COUNT(*) as count FROM activity_logs WHERE wallet_address = ? GROUP BY action"
                } else {
                    "SELECT action, COUNT(*) as count FROM activity_logs GROUP BY action"
                }
            ),
            binds_for_wallet(&wallet_address),
        )
        .fetch_all(&self.pool)
        .await?;

        let mut action_counts = HashMap::new();
        for row in action_rows {
            let action: String = row.get("action");
            let count: i64 = row.get("count");
            action_counts.insert(action, count);
        }

        let suspicious = self
            .check_suspicious_activity(wallet_address.clone())
            .await?;

        Ok(ActivityStats {
            total_actions,
            actions_today,
            actions_this_week: actions_week,
            actions_this_month: actions_month,
            success_rate,
            action_type_counts: action_counts,
            suspicious_activities: suspicious,
        })
    }

    pub async fn check_suspicious_activity(
        &self,
        wallet_address: Option<String>,
    ) -> Result<Vec<SuspiciousActivity>, ActivityLogError> {
        let mut suspicious = Vec::new();
        let now = Utc::now();
        let one_minute_ago = (now - ChronoDuration::minutes(1)).to_rfc3339();
        let five_minutes_ago = (now - ChronoDuration::minutes(5)).to_rfc3339();

        // Rapid connect/disconnect cycles
        let rapid_rows = bind_values(
            sqlx::query(
                if wallet_address.is_some() {
                    "SELECT wallet_address, COUNT(*) as count FROM activity_logs WHERE action IN ('connect', 'disconnect') AND timestamp >= ? AND wallet_address = ? GROUP BY wallet_address HAVING COUNT(*) > 5"
                } else {
                    "SELECT wallet_address, COUNT(*) as count FROM activity_logs WHERE action IN ('connect', 'disconnect') AND timestamp >= ? GROUP BY wallet_address HAVING COUNT(*) > 5"
                }
            ),
            binds_for_time_and_wallet(&one_minute_ago, &wallet_address),
        )
        .fetch_all(&self.pool)
        .await?;

        for row in rapid_rows {
            let wallet: String = row.get("wallet_address");
            let count: i64 = row.get("count");
            suspicious.push(SuspiciousActivity {
                activity_type: "rapid_connections".to_string(),
                description: format!("Detected {} connect/disconnect events in 1 minute", count),
                timestamp: now.to_rfc3339(),
                wallet_address: wallet,
                severity: "high".to_string(),
            });
        }

        // Failed signature attempts
        let failed_rows = bind_values(
            sqlx::query(
                if wallet_address.is_some() {
                    "SELECT wallet_address, COUNT(*) as count FROM activity_logs WHERE action IN ('sign', 'reject') AND result = 'failure' AND timestamp >= ? AND wallet_address = ? GROUP BY wallet_address HAVING COUNT(*) > 3"
                } else {
                    "SELECT wallet_address, COUNT(*) as count FROM activity_logs WHERE action IN ('sign', 'reject') AND result = 'failure' AND timestamp >= ? GROUP BY wallet_address HAVING COUNT(*) > 3"
                }
            ),
            binds_for_time_and_wallet(&five_minutes_ago, &wallet_address),
        )
        .fetch_all(&self.pool)
        .await?;

        for row in failed_rows {
            let wallet: String = row.get("wallet_address");
            let count: i64 = row.get("count");
            suspicious.push(SuspiciousActivity {
                activity_type: "failed_signatures".to_string(),
                description: format!("Detected {} failed signature attempts in 5 minutes", count),
                timestamp: now.to_rfc3339(),
                wallet_address: wallet,
                severity: "medium".to_string(),
            });
        }

        // Unusual transaction volume
        let volume_rows = bind_values(
            sqlx::query(
                if wallet_address.is_some() {
                    "SELECT wallet_address, COUNT(*) as count FROM activity_logs WHERE action IN ('send', 'swap') AND timestamp >= ? AND wallet_address = ? GROUP BY wallet_address HAVING COUNT(*) > 20"
                } else {
                    "SELECT wallet_address, COUNT(*) as count FROM activity_logs WHERE action IN ('send', 'swap') AND timestamp >= ? GROUP BY wallet_address HAVING COUNT(*) > 20"
                }
            ),
            binds_for_time_and_wallet(&one_minute_ago, &wallet_address),
        )
        .fetch_all(&self.pool)
        .await?;

        for row in volume_rows {
            let wallet: String = row.get("wallet_address");
            let count: i64 = row.get("count");
            suspicious.push(SuspiciousActivity {
                activity_type: "unusual_transaction_volume".to_string(),
                description: format!("Detected {} send/swap operations in 1 minute", count),
                timestamp: now.to_rfc3339(),
                wallet_address: wallet,
                severity: "high".to_string(),
            });
        }

        Ok(suspicious)
    }

    pub async fn cleanup_old_logs(
        &self,
        override_days: Option<i64>,
    ) -> Result<u64, ActivityLogError> {
        let days = match override_days {
            Some(value) => validate_retention(value)?,
            None => self.current_retention_days()?,
        };

        let cutoff = (Utc::now() - ChronoDuration::days(days)).to_rfc3339();
        let result = sqlx::query("DELETE FROM activity_logs WHERE timestamp < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub fn current_retention_days(&self) -> Result<i64, ActivityLogError> {
        self.retention_days.read().map(|guard| *guard).map_err(|_| {
            ActivityLogError::Internal("Failed to read retention configuration".to_string())
        })
    }

    pub fn set_retention_days(&self, days: i64) -> Result<i64, ActivityLogError> {
        let validated = validate_retention(days)?;
        {
            let mut guard = self.retention_days.write().map_err(|_| {
                ActivityLogError::Internal("Failed to update retention configuration".to_string())
            })?;
            *guard = validated;
        }
        persist_retention_days(&self.config_path, validated)?;
        Ok(validated)
    }

    async fn fetch_count(&self, sql: &str, binds: Vec<BindValue>) -> Result<i64, ActivityLogError> {
        let row = bind_values(sqlx::query(sql), binds)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>(0))
    }
}

fn bind_values<'q>(
    mut query: sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>,
    binds: Vec<BindValue>,
) -> sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>> {
    for value in binds {
        query = match value {
            BindValue::Text(v) => query.bind(v),
            BindValue::Integer(v) => query.bind(v),
        };
    }
    query
}

fn activity_log_path(app: &AppHandle) -> Result<PathBuf, ActivityLogError> {
    let mut path = app.path().app_data_dir().map_err(|err| {
        ActivityLogError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    path.push(ACTIVITY_DB_FILE);
    Ok(path)
}

fn activity_config_path(app: &AppHandle) -> Result<PathBuf, ActivityLogError> {
    let mut path = app.path().app_data_dir().map_err(|err| {
        ActivityLogError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    path.push(ACTIVITY_CONFIG_FILE);
    Ok(path)
}

fn load_retention_days(path: &Path) -> Result<i64, ActivityLogError> {
    if path.exists() {
        let data = fs::read_to_string(path)?;
        let config: ActivityLogConfig = serde_json::from_str(&data)?;
        validate_retention(config.retention_days)
    } else {
        persist_retention_days(path, DEFAULT_RETENTION_DAYS)?;
        Ok(DEFAULT_RETENTION_DAYS)
    }
}

fn persist_retention_days(path: &Path, days: i64) -> Result<(), ActivityLogError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    let config = ActivityLogConfig {
        retention_days: days,
    };
    let serialized = serde_json::to_string_pretty(&config)?;
    fs::write(path, serialized)?;
    Ok(())
}

fn validate_retention(days: i64) -> Result<i64, ActivityLogError> {
    if days <= 0 {
        return Err(ActivityLogError::InvalidRetention(
            "Retention must be greater than zero".to_string(),
        ));
    }
    if days > MAX_RETENTION_DAYS {
        return Err(ActivityLogError::InvalidRetention(format!(
            "Retention must be less than or equal to {} days",
            MAX_RETENTION_DAYS
        )));
    }
    Ok(days)
}

fn normalize_timestamp(value: &str) -> Result<String, ActivityLogError> {
    let parsed = DateTime::parse_from_rfc3339(value)
        .map_err(|_| ActivityLogError::InvalidTimestamp(value.to_string()))?;
    Ok(parsed.with_timezone(&Utc).to_rfc3339())
}

fn binds_for_wallet(wallet: &Option<String>) -> Vec<BindValue> {
    wallet
        .as_ref()
        .map(|addr| vec![BindValue::Text(addr.clone())])
        .unwrap_or_default()
}

fn binds_for_wallet_and_time(wallet: &Option<String>, timestamp: String) -> Vec<BindValue> {
    let mut binds = Vec::new();
    if let Some(addr) = wallet {
        binds.push(BindValue::Text(addr.clone()));
    }
    binds.push(BindValue::Text(timestamp));
    if wallet.is_some() {
        // When wallet is Some, query expects wallet first, then timestamp.
        binds.rotate_right(1);
    }
    binds
}

fn binds_for_time_and_wallet(time: &str, wallet: &Option<String>) -> Vec<BindValue> {
    let mut binds = vec![BindValue::Text(time.to_string())];
    if let Some(addr) = wallet {
        binds.push(BindValue::Text(addr.clone()));
    }
    binds
}

#[tauri::command]
pub async fn get_activity_logs(
    filter: ActivityLogFilter,
    logger: tauri::State<'_, ActivityLogger>,
) -> Result<Vec<ActivityLog>, String> {
    logger.get_logs(filter).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_activity_logs(
    filter: ActivityLogFilter,
    logger: tauri::State<'_, ActivityLogger>,
) -> Result<String, String> {
    logger
        .export_to_csv(filter)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_activity_stats(
    wallet_address: Option<String>,
    logger: tauri::State<'_, ActivityLogger>,
) -> Result<ActivityStats, String> {
    logger
        .get_stats(wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_suspicious_activity(
    wallet_address: Option<String>,
    logger: tauri::State<'_, ActivityLogger>,
) -> Result<Vec<SuspiciousActivity>, String> {
    logger
        .check_suspicious_activity(wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_activity_logs(
    retention_days: Option<i64>,
    logger: tauri::State<'_, ActivityLogger>,
) -> Result<u64, String> {
    logger
        .cleanup_old_logs(retention_days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_activity_retention(
    logger: tauri::State<'_, ActivityLogger>,
) -> Result<i64, String> {
    logger.current_retention_days().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_activity_retention(
    retention_days: i64,
    logger: tauri::State<'_, ActivityLogger>,
) -> Result<i64, String> {
    logger
        .set_retention_days(retention_days)
        .map_err(|e| e.to_string())
}
