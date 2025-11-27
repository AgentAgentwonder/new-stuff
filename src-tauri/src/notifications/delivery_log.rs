use chrono::Utc;
use sqlx::{Pool, Row, Sqlite};
use uuid::Uuid;

use super::types::{ChatServiceType, DeliveryLog, DeliveryStatus, NotificationError};

pub struct DeliveryLogger {
    pool: Pool<Sqlite>,
}

impl DeliveryLogger {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn initialize(&self) -> Result<(), NotificationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS delivery_logs (
                id TEXT PRIMARY KEY,
                service_type TEXT NOT NULL,
                config_id TEXT NOT NULL,
                config_name TEXT NOT NULL,
                alert_id TEXT,
                alert_name TEXT,
                message TEXT NOT NULL,
                status TEXT NOT NULL,
                error TEXT,
                retry_count INTEGER NOT NULL DEFAULT 0,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_delivery_logs_service ON delivery_logs(service_type);
            CREATE INDEX IF NOT EXISTS idx_delivery_logs_config ON delivery_logs(config_id);
            CREATE INDEX IF NOT EXISTS idx_delivery_logs_status ON delivery_logs(status);
            CREATE INDEX IF NOT EXISTS idx_delivery_logs_timestamp ON delivery_logs(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn log(
        &self,
        service_type: ChatServiceType,
        config_id: &str,
        config_name: &str,
        alert_id: Option<&str>,
        alert_name: Option<&str>,
        message: &str,
        status: DeliveryStatus,
        error: Option<&str>,
        retry_count: i32,
    ) -> Result<String, NotificationError> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO delivery_logs (
                id, service_type, config_id, config_name, alert_id, alert_name,
                message, status, error, retry_count, timestamp
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&id)
        .bind(service_type.as_str())
        .bind(config_id)
        .bind(config_name)
        .bind(alert_id)
        .bind(alert_name)
        .bind(message)
        .bind(status.as_str())
        .bind(error)
        .bind(retry_count)
        .bind(&timestamp)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn get_logs(
        &self,
        limit: i32,
        service_type: Option<&str>,
    ) -> Result<Vec<DeliveryLog>, NotificationError> {
        let query = if let Some(svc) = service_type {
            sqlx::query(
                r#"
                SELECT id, service_type, config_id, config_name, alert_id, alert_name,
                       message, status, error, retry_count, timestamp
                FROM delivery_logs
                WHERE service_type = ?1
                ORDER BY timestamp DESC
                LIMIT ?2
                "#,
            )
            .bind(svc)
            .bind(limit)
        } else {
            sqlx::query(
                r#"
                SELECT id, service_type, config_id, config_name, alert_id, alert_name,
                       message, status, error, retry_count, timestamp
                FROM delivery_logs
                ORDER BY timestamp DESC
                LIMIT ?1
                "#,
            )
            .bind(limit)
        };

        let rows = query.fetch_all(&self.pool).await?;

        let mut logs = Vec::new();
        for row in rows {
            logs.push(self.row_to_log(row)?);
        }

        Ok(logs)
    }

    pub async fn clear_logs(&self) -> Result<(), NotificationError> {
        sqlx::query("DELETE FROM delivery_logs")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn cleanup_old_logs(&self, days: i64) -> Result<usize, NotificationError> {
        let cutoff = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();

        let result = sqlx::query("DELETE FROM delivery_logs WHERE timestamp < ?1")
            .bind(&cutoff)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }

    fn row_to_log(&self, row: sqlx::sqlite::SqliteRow) -> Result<DeliveryLog, NotificationError> {
        let service_type_str: String = row.try_get("service_type")?;
        let status_str: String = row.try_get("status")?;

        Ok(DeliveryLog {
            id: row.try_get("id")?,
            service_type: ChatServiceType::from_str(&service_type_str).ok_or_else(|| {
                NotificationError::Internal(format!("Invalid service type: {}", service_type_str))
            })?,
            config_id: row.try_get("config_id")?,
            config_name: row.try_get("config_name")?,
            alert_id: row.try_get("alert_id")?,
            alert_name: row.try_get("alert_name")?,
            message: row.try_get("message")?,
            status: DeliveryStatus::from_str(&status_str).ok_or_else(|| {
                NotificationError::Internal(format!("Invalid status: {}", status_str))
            })?,
            error: row.try_get("error")?,
            retry_count: row.try_get("retry_count")?,
            timestamp: row.try_get("timestamp")?,
        })
    }
}
