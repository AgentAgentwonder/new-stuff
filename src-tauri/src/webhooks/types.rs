use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type WebhookHeaders = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WebhookMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookTemplateVariable {
    pub key: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookConfig {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub method: WebhookMethod,
    pub headers: WebhookHeaders,
    pub body_template: Option<String>,
    pub variables: Vec<WebhookTemplateVariable>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_secs: u64,
    pub max_delay_secs: u64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_secs: 2,
            max_delay_secs: 60,
            jitter: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookDeliveryLog {
    pub id: String,
    pub webhook_id: String,
    pub webhook_name: String,
    pub status: DeliveryStatus,
    pub attempt: u32,
    pub response_code: Option<u16>,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
    pub payload_preview: Option<String>,
    pub triggered_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookTestResult {
    pub success: bool,
    pub message: String,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid template: {0}")]
    InvalidTemplate(String),
    #[error("webhook not found: {0}")]
    NotFound(String),
    #[error("webhook disabled")]
    Disabled,
    #[error("internal error: {0}")]
    Internal(String),
}
