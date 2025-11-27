use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const NOTIFICATIONS_DB_FILE: &str = "chat_integrations.db";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChatServiceType {
    Telegram,
    Slack,
    Discord,
}

impl ChatServiceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChatServiceType::Telegram => "telegram",
            ChatServiceType::Slack => "slack",
            ChatServiceType::Discord => "discord",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "telegram" => Some(ChatServiceType::Telegram),
            "slack" => Some(ChatServiceType::Slack),
            "discord" => Some(ChatServiceType::Discord),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Failed,
    RateLimited,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Sent => "sent",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::RateLimited => "rate_limited",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(DeliveryStatus::Pending),
            "sent" => Some(DeliveryStatus::Sent),
            "failed" => Some(DeliveryStatus::Failed),
            "rate_limited" => Some(DeliveryStatus::RateLimited),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramConfig {
    pub id: String,
    pub name: String,
    pub bot_token: String,
    pub chat_id: String,
    pub enabled: bool,
    pub alert_types: Option<Vec<String>>,
    pub alert_priorities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlackConfig {
    pub id: String,
    pub name: String,
    pub webhook_url: String,
    pub channel: Option<String>,
    pub enabled: bool,
    pub alert_types: Option<Vec<String>>,
    pub alert_priorities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    pub id: String,
    pub name: String,
    pub webhook_url: String,
    pub username: Option<String>,
    pub enabled: bool,
    pub role_mentions: Option<Vec<String>>,
    pub alert_types: Option<Vec<String>>,
    pub alert_priorities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatIntegrationSettings {
    pub telegram: Vec<TelegramConfig>,
    pub slack: Vec<SlackConfig>,
    pub discord: Vec<DiscordConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryLog {
    pub id: String,
    pub service_type: ChatServiceType,
    pub config_id: String,
    pub config_name: String,
    pub alert_id: Option<String>,
    pub alert_name: Option<String>,
    pub message: String,
    pub status: DeliveryStatus,
    pub error: Option<String>,
    pub retry_count: i32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestMessageResult {
    pub success: bool,
    pub message: String,
    pub delivery_time: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitStatus {
    pub service_type: ChatServiceType,
    pub config_id: String,
    pub current_count: i32,
    pub max_per_minute: i32,
    pub reset_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("rate limited: {0}")]
    RateLimited(String),
    #[error("config not found: {0}")]
    ConfigNotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub fn notifications_db_path(app: &AppHandle) -> Result<PathBuf, NotificationError> {
    let mut path = app.path().app_data_dir().map_err(|err| {
        NotificationError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;

    std::fs::create_dir_all(&path).map_err(|e| NotificationError::Io(e))?;

    path.push(NOTIFICATIONS_DB_FILE);
    Ok(path)
}
