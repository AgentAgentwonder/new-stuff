use chrono::Utc;
use lettre::message::{header, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::security::keystore::Keystore;

const EMAIL_DB_FILE: &str = "email_notifications.db";
const KEY_EMAIL_CONFIG: &str = "email_smtp_config";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
    pub use_tls: bool,
    pub use_starttls: bool,
    pub provider: SmtpProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SmtpProvider {
    Gmail,
    Outlook,
    SendGrid,
    Custom,
}

impl SmtpProvider {
    pub fn preset(&self) -> Option<(String, u16, bool, bool)> {
        match self {
            SmtpProvider::Gmail => Some(("smtp.gmail.com".to_string(), 587, false, true)),
            SmtpProvider::Outlook => Some(("smtp-mail.outlook.com".to_string(), 587, false, true)),
            SmtpProvider::SendGrid => Some(("smtp.sendgrid.net".to_string(), 587, false, true)),
            SmtpProvider::Custom => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailTemplate {
    pub name: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailRequest {
    pub to: Vec<String>,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub template: Option<String>,
    pub template_vars: Option<serde_json::Value>,
    pub attachments: Option<Vec<EmailAttachment>>,
    pub include_unsubscribe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAttachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailDeliveryRecord {
    pub id: String,
    pub to: Vec<String>,
    pub subject: String,
    pub status: EmailStatus,
    pub error: Option<String>,
    pub sent_at: String,
    pub retry_count: i32,
    pub delivery_time_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EmailStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

impl EmailStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailStatus::Pending => "pending",
            EmailStatus::Sent => "sent",
            EmailStatus::Failed => "failed",
            EmailStatus::Retrying => "retrying",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(EmailStatus::Pending),
            "sent" => Some(EmailStatus::Sent),
            "failed" => Some(EmailStatus::Failed),
            "retrying" => Some(EmailStatus::Retrying),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailStats {
    pub total_sent: i64,
    pub total_failed: i64,
    pub total_pending: i64,
    pub average_delivery_time_ms: f64,
    pub last_24h_sent: i64,
    pub last_24h_failed: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("smtp error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error("email error: {0}")]
    Email(#[from] lettre::error::Error),
    #[error("address parse error: {0}")]
    AddressParse(#[from] lettre::address::AddressError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("configuration not found")]
    ConfigNotFound,
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Clone)]
pub struct EmailManager {
    pool: Pool<Sqlite>,
    app_handle: AppHandle,
}

pub type SharedEmailManager = Arc<RwLock<EmailManager>>;

impl EmailManager {
    pub async fn new(app: &AppHandle) -> Result<Self, EmailError> {
        let db_path = email_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self {
            pool,
            app_handle: app.clone(),
        };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), EmailError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS email_deliveries (
                id TEXT PRIMARY KEY,
                recipients TEXT NOT NULL,
                subject TEXT NOT NULL,
                status TEXT NOT NULL,
                error TEXT,
                sent_at TEXT NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                delivery_time_ms INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_email_status ON email_deliveries(status);
            CREATE INDEX IF NOT EXISTS idx_email_sent_at ON email_deliveries(sent_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_config(
        &self,
        config: SmtpConfig,
        keystore: &Keystore,
    ) -> Result<(), EmailError> {
        let serialized = serde_json::to_vec(&config)?;
        keystore
            .store_secret(KEY_EMAIL_CONFIG, &serialized)
            .map_err(|e| EmailError::Internal(format!("Failed to store config: {}", e)))?;
        Ok(())
    }

    pub async fn get_config(&self, keystore: &Keystore) -> Result<SmtpConfig, EmailError> {
        let data = keystore
            .retrieve_secret(KEY_EMAIL_CONFIG)
            .map_err(|_| EmailError::ConfigNotFound)?;
        let config: SmtpConfig = serde_json::from_slice(&data)?;
        Ok(config)
    }

    pub async fn delete_config(&self, keystore: &Keystore) -> Result<(), EmailError> {
        keystore
            .remove_secret(KEY_EMAIL_CONFIG)
            .map_err(|e| EmailError::Internal(format!("Failed to delete config: {}", e)))?;
        Ok(())
    }

    pub async fn test_connection(&self, config: &SmtpConfig) -> Result<i64, EmailError> {
        let start = std::time::Instant::now();

        let mailer = self.build_mailer(config)?;
        mailer.test_connection()?;

        let latency = start.elapsed().as_millis() as i64;
        Ok(latency)
    }

    pub async fn send_email(
        &self,
        req: SendEmailRequest,
        config: &SmtpConfig,
    ) -> Result<EmailDeliveryRecord, EmailError> {
        let id = uuid::Uuid::new_v4().to_string();
        let start = std::time::Instant::now();

        // Build the email message
        let mut message_builder = Message::builder()
            .from(format!("{} <{}>", config.from_name, config.from_address).parse()?)
            .subject(&req.subject);

        for recipient in &req.to {
            message_builder = message_builder.to(recipient.parse()?);
        }

        // Handle template or direct content
        let (html_body, text_body) = if let Some(template_name) = &req.template {
            let template = self.get_template(template_name).await?;
            let rendered = self.render_template(&template, req.template_vars.as_ref())?;
            (rendered.0, rendered.1)
        } else {
            (req.html_body.clone(), req.text_body.clone())
        };

        // Build multipart message with text and/or HTML parts
        let mut multipart = MultiPart::alternative().build();

        if let Some(text) = text_body {
            let text_part = SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(text);
            multipart = multipart.singlepart(text_part);
        }

        if let Some(html) = html_body {
            let mut html_with_unsubscribe = html.clone();
            if req.include_unsubscribe {
                html_with_unsubscribe.push_str(
                    r#"<br><br><p style="font-size:12px;color:#666;">
                    <a href="{{unsubscribe_url}}">Unsubscribe</a> from these notifications.
                    </p>"#,
                );
            }

            let html_part = SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(html_with_unsubscribe);
            multipart = multipart.singlepart(html_part);
        }

        let message = message_builder.multipart(multipart)?;

        // Send with retry logic
        let mailer = self.build_mailer(config)?;
        let mut retry_count = 0;
        let max_retries = 3;
        let mut last_error: Option<String> = None;

        loop {
            match mailer.send(&message) {
                Ok(_) => {
                    let delivery_time = start.elapsed().as_millis() as i64;
                    let record = self
                        .record_delivery(
                            &id,
                            &req.to,
                            &req.subject,
                            EmailStatus::Sent,
                            None,
                            retry_count,
                            Some(delivery_time),
                        )
                        .await?;
                    return Ok(record);
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    retry_count += 1;

                    if retry_count >= max_retries {
                        let record = self
                            .record_delivery(
                                &id,
                                &req.to,
                                &req.subject,
                                EmailStatus::Failed,
                                last_error.clone(),
                                retry_count,
                                None,
                            )
                            .await?;
                        return Err(EmailError::Smtp(e));
                    }

                    // Exponential backoff
                    tokio::time::sleep(Duration::from_secs(2u64.pow(retry_count as u32))).await;
                }
            }
        }
    }

    fn build_mailer(&self, config: &SmtpConfig) -> Result<SmtpTransport, EmailError> {
        let credentials = Credentials::new(config.username.clone(), config.password.clone());

        let mut transport = SmtpTransport::relay(&config.server)?
            .port(config.port)
            .credentials(credentials);

        if config.use_tls {
            transport = transport.tls(lettre::transport::smtp::client::Tls::Required(
                lettre::transport::smtp::client::TlsParameters::new(config.server.clone())?,
            ));
        } else if config.use_starttls {
            transport = transport.tls(lettre::transport::smtp::client::Tls::Required(
                lettre::transport::smtp::client::TlsParameters::new(config.server.clone())?
            ));
        }

        Ok(transport.build())
    }

    async fn record_delivery(
        &self,
        id: &str,
        to: &[String],
        subject: &str,
        status: EmailStatus,
        error: Option<String>,
        retry_count: i32,
        delivery_time_ms: Option<i64>,
    ) -> Result<EmailDeliveryRecord, EmailError> {
        let now = Utc::now().to_rfc3339();
        let recipients_json = serde_json::to_string(to)?;

        sqlx::query(
            r#"
            INSERT INTO email_deliveries (
                id, recipients, subject, status, error, sent_at, retry_count, delivery_time_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(id)
        .bind(&recipients_json)
        .bind(subject)
        .bind(status.as_str())
        .bind(&error)
        .bind(&now)
        .bind(retry_count)
        .bind(delivery_time_ms)
        .execute(&self.pool)
        .await?;

        Ok(EmailDeliveryRecord {
            id: id.to_string(),
            to: to.to_vec(),
            subject: subject.to_string(),
            status,
            error,
            sent_at: now,
            retry_count,
            delivery_time_ms,
        })
    }

    pub async fn get_delivery_stats(&self) -> Result<EmailStats, EmailError> {
        let total_sent = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_deliveries WHERE status = 'sent'",
        )
        .fetch_one(&self.pool)
        .await?;

        let total_failed = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_deliveries WHERE status = 'failed'",
        )
        .fetch_one(&self.pool)
        .await?;

        let total_pending = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_deliveries WHERE status IN ('pending', 'retrying')",
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_delivery = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT AVG(delivery_time_ms) FROM email_deliveries WHERE status = 'sent' AND delivery_time_ms IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0.0);

        let last_24h_sent = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_deliveries WHERE status = 'sent' AND datetime(sent_at) > datetime('now', '-1 day')"
        )
        .fetch_one(&self.pool)
        .await?;

        let last_24h_failed = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM email_deliveries WHERE status = 'failed' AND datetime(sent_at) > datetime('now', '-1 day')"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(EmailStats {
            total_sent,
            total_failed,
            total_pending,
            average_delivery_time_ms: avg_delivery,
            last_24h_sent,
            last_24h_failed,
        })
    }

    pub async fn get_delivery_history(
        &self,
        limit: i32,
    ) -> Result<Vec<EmailDeliveryRecord>, EmailError> {
        let rows = sqlx::query(
            r#"
            SELECT id, recipients, subject, status, error, sent_at, retry_count, delivery_time_ms
            FROM email_deliveries
            ORDER BY sent_at DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut records = Vec::new();
        for row in rows {
            let recipients_json: String = row.try_get("recipients")?;
            let to: Vec<String> = serde_json::from_str(&recipients_json)?;
            let status_str: String = row.try_get("status")?;
            let status = EmailStatus::from_str(&status_str).unwrap_or(EmailStatus::Failed);

            records.push(EmailDeliveryRecord {
                id: row.try_get("id")?,
                to,
                subject: row.try_get("subject")?,
                status,
                error: row.try_get("error")?,
                sent_at: row.try_get("sent_at")?,
                retry_count: row.try_get("retry_count")?,
                delivery_time_ms: row.try_get("delivery_time_ms")?,
            });
        }

        Ok(records)
    }

    async fn get_template(&self, _name: &str) -> Result<EmailTemplate, EmailError> {
        // Predefined templates
        Ok(EmailTemplate {
            name: "alert".to_string(),
            subject: "Price Alert Triggered: {{symbol}}".to_string(),
            html_body: r#"
                <html>
                <body style="font-family: Arial, sans-serif;">
                    <h2>Price Alert Triggered</h2>
                    <p>Your alert for <strong>{{symbol}}</strong> has been triggered.</p>
                    <p>Current Price: <strong>{{price}}</strong></p>
                    <p>Condition: {{condition}}</p>
                    <p>Triggered at: {{timestamp}}</p>
                </body>
                </html>
            "#.to_string(),
            text_body: Some("Price Alert Triggered: {{symbol}}\nCurrent Price: {{price}}\nCondition: {{condition}}\nTriggered at: {{timestamp}}".to_string()),
            variables: vec!["symbol".to_string(), "price".to_string(), "condition".to_string(), "timestamp".to_string()],
        })
    }

    fn render_template(
        &self,
        template: &EmailTemplate,
        vars: Option<&serde_json::Value>,
    ) -> Result<(Option<String>, Option<String>), EmailError> {
        let vars = vars.cloned().unwrap_or_default();

        let html = Some(self.replace_template_vars(&template.html_body, &vars));
        let text = template
            .text_body
            .as_ref()
            .map(|t| self.replace_template_vars(t, &vars));

        Ok((html, text))
    }

    fn replace_template_vars(&self, template: &str, vars: &serde_json::Value) -> String {
        let mut result = template.to_string();

        if let Some(obj) = vars.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }

        result
    }
}

fn email_db_path(app: &AppHandle) -> Result<PathBuf, EmailError> {
    let app_handle = app.clone();
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| EmailError::Internal(format!("Unable to resolve app data directory: {}", e)))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| EmailError::Internal(format!("Failed to create app data directory: {}", e)))?;

    Ok(app_dir.join(EMAIL_DB_FILE))
}

// Tauri Commands
#[tauri::command]
pub async fn email_save_config(
    config: SmtpConfig,
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<String, String> {
    let manager = EmailManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .save_config(config, &keystore)
        .await
        .map_err(|e| e.to_string())?;

    Ok("SMTP configuration saved successfully".to_string())
}

#[tauri::command]
pub async fn email_get_config(
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<SmtpConfig, String> {
    let manager = EmailManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .get_config(&keystore)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn email_delete_config(
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<String, String> {
    let manager = EmailManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .delete_config(&keystore)
        .await
        .map_err(|e| e.to_string())?;

    Ok("SMTP configuration deleted successfully".to_string())
}

#[tauri::command]
pub async fn email_test_connection(config: SmtpConfig, app: AppHandle) -> Result<i64, String> {
    let manager = EmailManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .test_connection(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn email_send(
    req: SendEmailRequest,
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<EmailDeliveryRecord, String> {
    let manager = EmailManager::new(&app).await.map_err(|e| e.to_string())?;

    let config = manager
        .get_config(&keystore)
        .await
        .map_err(|e| e.to_string())?;

    manager
        .send_email(req, &config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn email_get_stats(app: AppHandle) -> Result<EmailStats, String> {
    let manager = EmailManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .get_delivery_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn email_get_history(
    limit: i32,
    app: AppHandle,
) -> Result<Vec<EmailDeliveryRecord>, String> {
    let manager = EmailManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .get_delivery_history(limit)
        .await
        .map_err(|e| e.to_string())
}
