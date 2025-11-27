use super::retry::RetryExecutor;
use super::template::TemplateEngine;
use super::types::{
    DeliveryStatus, RetryPolicy, WebhookConfig, WebhookDeliveryLog, WebhookError, WebhookMethod,
    WebhookTestResult,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde_json::Value;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::time::Instant;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use uuid::Uuid;

const WEBHOOKS_DB_FILE: &str = "webhooks.db";

pub struct WebhookManager {
    pool: Pool<Sqlite>,
    client: Client,
    template_engine: TemplateEngine,
    sending_lock: Mutex<()>,
}

impl WebhookManager {
    pub async fn new(app: &AppHandle) -> Result<Self, WebhookError> {
        let db_path = Self::webhooks_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: WebhookManager failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for WebhookManager");
                eprintln!("WebhookManager using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        let manager = Self {
            pool,
            client: Client::new(),
            template_engine: TemplateEngine::new(),
            sending_lock: Mutex::new(()),
        };

        manager.initialize().await?;
        Ok(manager)
    }

    fn webhooks_db_path(app: &AppHandle) -> Result<std::path::PathBuf, WebhookError> {
        let mut path = app.path().app_data_dir().map_err(|err| {
            WebhookError::Internal(format!("Unable to resolve app data directory: {err}"))
        })?;

        std::fs::create_dir_all(&path)?;
        path.push(WEBHOOKS_DB_FILE);
        Ok(path)
    }

    async fn initialize(&self) -> Result<(), WebhookError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webhooks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                url TEXT NOT NULL,
                method TEXT NOT NULL,
                headers_json TEXT NOT NULL,
                body_template TEXT,
                variables_json TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                retry_policy_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS webhook_delivery_logs (
                id TEXT PRIMARY KEY,
                webhook_id TEXT NOT NULL,
                webhook_name TEXT NOT NULL,
                status TEXT NOT NULL,
                attempt INTEGER NOT NULL,
                response_code INTEGER,
                response_time_ms INTEGER,
                error TEXT,
                payload_preview TEXT,
                triggered_at TEXT NOT NULL,
                completed_at TEXT,
                FOREIGN KEY(webhook_id) REFERENCES webhooks(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_webhooks(&self) -> Result<Vec<WebhookConfig>, WebhookError> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM webhooks
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(self.row_to_config(row)?);
        }

        Ok(configs)
    }

    pub async fn get_webhook(&self, id: &str) -> Result<WebhookConfig, WebhookError> {
        let row = sqlx::query(
            r#"
            SELECT * FROM webhooks WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| WebhookError::NotFound(id.to_string()))?;

        self.row_to_config(row)
    }

    pub async fn create_webhook(
        &self,
        mut config: WebhookConfig,
    ) -> Result<WebhookConfig, WebhookError> {
        let now = Utc::now();
        config.id = Uuid::new_v4().to_string();
        config.created_at = now;
        config.updated_at = now;

        self.insert_or_update(&config).await?;
        Ok(config)
    }

    pub async fn update_webhook(
        &self,
        id: &str,
        config: WebhookConfig,
    ) -> Result<(), WebhookError> {
        let mut updated = config.clone();
        updated.id = id.to_string();
        updated.updated_at = Utc::now();
        self.insert_or_update(&updated).await
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<(), WebhookError> {
        sqlx::query("DELETE FROM webhooks WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn insert_or_update(&self, config: &WebhookConfig) -> Result<(), WebhookError> {
        sqlx::query(
            r#"
            INSERT INTO webhooks (
                id, name, description, url, method, headers_json, body_template,
                variables_json, enabled, retry_policy_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                url = excluded.url,
                method = excluded.method,
                headers_json = excluded.headers_json,
                body_template = excluded.body_template,
                variables_json = excluded.variables_json,
                enabled = excluded.enabled,
                retry_policy_json = excluded.retry_policy_json,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.description)
        .bind(&config.url)
        .bind(match config.method {
            WebhookMethod::Get => "GET",
            WebhookMethod::Post => "POST",
        })
        .bind(serde_json::to_string(&config.headers).map_err(WebhookError::Serialization)?)
        .bind(&config.body_template)
        .bind(serde_json::to_string(&config.variables).map_err(WebhookError::Serialization)?)
        .bind(if config.enabled { 1 } else { 0 })
        .bind(serde_json::to_string(&config.retry_policy).map_err(WebhookError::Serialization)?)
        .bind(config.created_at.to_rfc3339())
        .bind(config.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn row_to_config(&self, row: sqlx::sqlite::SqliteRow) -> Result<WebhookConfig, WebhookError> {
        let method_str: String = row.try_get("method")?;
        let headers_json: String = row.try_get("headers_json")?;
        let variables_json: String = row.try_get("variables_json")?;
        let retry_policy_json: String = row.try_get("retry_policy_json")?;

        Ok(WebhookConfig {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            url: row.try_get("url")?,
            method: match method_str.as_str() {
                "GET" => WebhookMethod::Get,
                "POST" => WebhookMethod::Post,
                _ => WebhookMethod::Post,
            },
            headers: serde_json::from_str(&headers_json).map_err(WebhookError::Serialization)?,
            body_template: row.try_get("body_template")?,
            variables: serde_json::from_str(&variables_json)
                .map_err(WebhookError::Serialization)?,
            enabled: row.try_get::<i64, _>("enabled")? == 1,
            retry_policy: serde_json::from_str(&retry_policy_json)
                .map_err(WebhookError::Serialization)?,
            created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .map_err(|e| WebhookError::Internal(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .map_err(|e| WebhookError::Internal(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&Utc),
        })
    }

    pub async fn list_delivery_logs(
        &self,
        webhook_id: Option<&str>,
        limit: i32,
    ) -> Result<Vec<WebhookDeliveryLog>, WebhookError> {
        let query = if let Some(id) = webhook_id {
            sqlx::query(
                r#"
                SELECT * FROM webhook_delivery_logs
                WHERE webhook_id = ?1
                ORDER BY triggered_at DESC
                LIMIT ?2
                "#,
            )
            .bind(id)
            .bind(limit)
        } else {
            sqlx::query(
                r#"
                SELECT * FROM webhook_delivery_logs
                ORDER BY triggered_at DESC
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

    fn row_to_log(&self, row: sqlx::sqlite::SqliteRow) -> Result<WebhookDeliveryLog, WebhookError> {
        Ok(WebhookDeliveryLog {
            id: row.try_get("id")?,
            webhook_id: row.try_get("webhook_id")?,
            webhook_name: row.try_get("webhook_name")?,
            status: match row.try_get::<String, _>("status")?.as_str() {
                "pending" => DeliveryStatus::Pending,
                "sent" => DeliveryStatus::Sent,
                "failed" => DeliveryStatus::Failed,
                "retrying" => DeliveryStatus::Retrying,
                _ => DeliveryStatus::Failed,
            },
            attempt: row.try_get("attempt")?,
            response_code: row.try_get("response_code")?,
            response_time_ms: row.try_get::<Option<i64>, _>("response_time_ms")?.map(|v| v as u64),
            error: row.try_get("error")?,
            payload_preview: row.try_get("payload_preview")?,
            triggered_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("triggered_at")?)
                .map_err(|e| WebhookError::Internal(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&Utc),
            completed_at: row
                .try_get::<Option<String>, _>("completed_at")?
                .map(|ts| {
                    DateTime::parse_from_rfc3339(&ts)
                        .map_err(|e| WebhookError::Internal(format!("Invalid timestamp: {}", e)))
                        .map(|dt| dt.with_timezone(&Utc))
                })
                .transpose()?,
        })
    }

    pub async fn trigger_webhook(
        &self,
        id: &str,
        variables: HashMap<String, Value>,
    ) -> Result<WebhookDeliveryLog, WebhookError> {
        let config = self.get_webhook(id).await?;
        if !config.enabled {
            return Err(WebhookError::Disabled);
        }

        self.send_with_retries(&config, variables).await
    }

    pub async fn test_webhook(
        &self,
        id: &str,
        variables: HashMap<String, Value>,
    ) -> Result<WebhookTestResult, WebhookError> {
        let config = self.get_webhook(id).await?;
        let result = self.send_once(&config, variables, true, 1).await?;

        Ok(result)
    }

    async fn send_with_retries(
        &self,
        config: &WebhookConfig,
        variables: HashMap<String, Value>,
    ) -> Result<WebhookDeliveryLog, WebhookError> {
        let executor = RetryExecutor::new(config.retry_policy.clone());
        let log_id = Uuid::new_v4().to_string();
        let triggered_at = Utc::now();

        let mut last_log = None;
        let payload_preview = self.preview_payload(config, &variables)?;

        let _guard = self.sending_lock.lock().await;

        let result = executor
            .execute(|| {
                let config = config.clone();
                let variables = variables.clone();
                let payload_preview = payload_preview.clone();
                let log_id = log_id.clone();
                async move {
                    self.log_status(
                        &log_id,
                        &config,
                        DeliveryStatus::Retrying,
                        0,
                        None,
                        None,
                        Some("Scheduling retry"),
                        Some(&payload_preview),
                        triggered_at,
                        None,
                    )
                    .await?;

                    let attempt = 1; // actual attempt tracked inside send_once
                    match self
                        .send_once(&config, variables.clone(), false, attempt)
                        .await
                    {
                        Ok(test_result) => {
                            self.log_status(
                                &log_id,
                                &config,
                                DeliveryStatus::Sent,
                                attempt,
                                test_result.response_code,
                                test_result.latency_ms,
                                None,
                                Some(&payload_preview),
                                triggered_at,
                                Some(Utc::now()),
                            )
                            .await?;

                            Ok(self
                                .get_delivery_log(&log_id)
                                .await
                                .expect("log should exist"))
                        }
                        Err(err) => {
                            self.log_status(
                                &log_id,
                                &config,
                                DeliveryStatus::Failed,
                                attempt,
                                None,
                                None,
                                Some(&err.to_string()),
                                Some(&payload_preview),
                                triggered_at,
                                Some(Utc::now()),
                            )
                            .await?;
                            Err(err)
                        }
                    }
                }
            })
            .await;

        match result {
            Ok(log) => Ok(log),
            Err(err) => {
                last_log = self.get_delivery_log(&log_id).await.ok();
                if let Some(mut log) = last_log {
                    log.status = DeliveryStatus::Failed;
                    log.error = Some(err.to_string());
                    Ok(log)
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn send_once(
        &self,
        config: &WebhookConfig,
        variables: HashMap<String, Value>,
        test_only: bool,
        attempt: u32,
    ) -> Result<WebhookTestResult, WebhookError> {
        let mut request_builder = match config.method {
            WebhookMethod::Get => self.client.get(&config.url),
            WebhookMethod::Post => self.client.post(&config.url),
        };

        for (key, value) in &config.headers {
            request_builder = request_builder.header(key, value);
        }

        let payload_preview = self.preview_payload(config, &variables)?;

        if let Some(body_template) = &config.body_template {
            let rendered = self.template_engine.render(body_template, &variables)?;
            let json: Value = serde_json::from_str(&rendered)
                .map_err(|e| WebhookError::InvalidTemplate(e.to_string()))?;
            request_builder = request_builder.json(&json);
        } else if config.method == WebhookMethod::Post {
            request_builder = request_builder.json(&variables);
        }

        let start = Instant::now();
        let response = request_builder.send().await?;
        let latency = start.elapsed().as_millis();
        let status = response.status();
        let text = response.text().await.ok();

        if status.is_success() {
            if !test_only {
                self.log_status(
                    Uuid::new_v4().to_string().as_str(),
                    config,
                    DeliveryStatus::Sent,
                    attempt,
                    Some(status.as_u16()),
                    Some(latency as u64),
                    None,
                    Some(&payload_preview),
                    Utc::now(),
                    Some(Utc::now()),
                )
                .await?;
            }

            Ok(WebhookTestResult {
                success: true,
                message: "Webhook delivered successfully".to_string(),
                response_code: Some(status.as_u16()),
                response_body: text.clone(),
                latency_ms: Some(latency as u64),
            })
        } else {
            let error_message = format!("Webhook failed with status {}", status);
            Err(WebhookError::Internal(error_message))
        }
    }

    fn preview_payload(
        &self,
        config: &WebhookConfig,
        variables: &HashMap<String, Value>,
    ) -> Result<String, WebhookError> {
        if let Some(body_template) = &config.body_template {
            let rendered = self.template_engine.render(body_template, variables)?;
            Ok(rendered)
        } else {
            Ok(serde_json::to_string(variables).map_err(WebhookError::Serialization)?)
        }
    }

    async fn log_status(
        &self,
        log_id: &str,
        config: &WebhookConfig,
        status: DeliveryStatus,
        attempt: u32,
        response_code: Option<u16>,
        response_time_ms: Option<u64>,
        error: Option<&str>,
        payload_preview: Option<&str>,
        triggered_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
    ) -> Result<(), WebhookError> {
        sqlx::query(
            r#"
            INSERT INTO webhook_delivery_logs (
                id, webhook_id, webhook_name, status, attempt, response_code, response_time_ms,
                error, payload_preview, triggered_at, completed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                attempt = excluded.attempt,
                response_code = excluded.response_code,
                response_time_ms = excluded.response_time_ms,
                error = excluded.error,
                payload_preview = excluded.payload_preview,
                completed_at = excluded.completed_at
            "#,
        )
        .bind(log_id)
        .bind(&config.id)
        .bind(&config.name)
        .bind(match status {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Retrying => "retrying",
        })
        .bind(attempt as i64)
        .bind(response_code.map(|code| code as i64))
        .bind(response_time_ms.map(|v| v as i64))
        .bind(error)
        .bind(payload_preview)
        .bind(triggered_at.to_rfc3339())
        .bind(completed_at.map(|dt| dt.to_rfc3339()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_delivery_log(&self, id: &str) -> Result<WebhookDeliveryLog, WebhookError> {
        let row = sqlx::query("SELECT * FROM webhook_delivery_logs WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| WebhookError::NotFound(id.to_string()))?;

        self.row_to_log(row)
    }
}
