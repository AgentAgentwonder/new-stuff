use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::delivery_log::DeliveryLogger;
use super::discord::DiscordClient;
use super::rate_limiter::RateLimiter;
use super::slack::SlackClient;
use super::telegram::{format_alert_message, TelegramClient};
use super::types::{
    notifications_db_path, ChatIntegrationSettings, ChatServiceType, DeliveryStatus, DiscordConfig,
    NotificationError, SlackConfig, TelegramConfig, TestMessageResult,
};

pub struct NotificationRouter {
    pool: Pool<Sqlite>,
    telegram_client: TelegramClient,
    slack_client: SlackClient,
    discord_client: DiscordClient,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    delivery_logger: DeliveryLogger,
}

pub type SharedNotificationRouter = Arc<RwLock<NotificationRouter>>;

impl NotificationRouter {
    pub async fn new(app: &AppHandle) -> Result<Self, NotificationError> {
        let db_path = notifications_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let delivery_logger = DeliveryLogger::new(pool.clone());
        delivery_logger.initialize().await?;

        let router = Self {
            pool,
            telegram_client: TelegramClient::new(),
            slack_client: SlackClient::new(),
            discord_client: DiscordClient::new(),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            delivery_logger,
        };

        router.initialize().await?;
        Ok(router)
    }

    async fn initialize(&self) -> Result<(), NotificationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chat_integrations (
                service_type TEXT NOT NULL,
                config_id TEXT PRIMARY KEY,
                config_data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_settings(
        &self,
        settings: &ChatIntegrationSettings,
    ) -> Result<(), NotificationError> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM chat_integrations")
            .execute(&mut *tx)
            .await?;

        for config in &settings.telegram {
            let config_data = serde_json::to_string(config)?;
            sqlx::query(
                r#"
                INSERT INTO chat_integrations (service_type, config_id, config_data, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .bind(ChatServiceType::Telegram.as_str())
            .bind(&config.id)
            .bind(&config_data)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }

        for config in &settings.slack {
            let config_data = serde_json::to_string(config)?;
            sqlx::query(
                r#"
                INSERT INTO chat_integrations (service_type, config_id, config_data, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .bind(ChatServiceType::Slack.as_str())
            .bind(&config.id)
            .bind(&config_data)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }

        for config in &settings.discord {
            let config_data = serde_json::to_string(config)?;
            sqlx::query(
                r#"
                INSERT INTO chat_integrations (service_type, config_id, config_data, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .bind(ChatServiceType::Discord.as_str())
            .bind(&config.id)
            .bind(&config_data)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_settings(&self) -> Result<ChatIntegrationSettings, NotificationError> {
        let rows = sqlx::query(
            r#"
            SELECT service_type, config_id, config_data
            FROM chat_integrations
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut settings = ChatIntegrationSettings {
            telegram: Vec::new(),
            slack: Vec::new(),
            discord: Vec::new(),
        };

        for row in rows {
            let service_type_str: String = row.try_get("service_type")?;
            let config_data: String = row.try_get("config_data")?;

            match service_type_str.as_str() {
                "telegram" => {
                    let config: TelegramConfig = serde_json::from_str(&config_data)?;
                    settings.telegram.push(config);
                }
                "slack" => {
                    let config: SlackConfig = serde_json::from_str(&config_data)?;
                    settings.slack.push(config);
                }
                "discord" => {
                    let config: DiscordConfig = serde_json::from_str(&config_data)?;
                    settings.discord.push(config);
                }
                _ => {}
            }
        }

        Ok(settings)
    }

    pub async fn add_telegram_config(
        &self,
        mut config: TelegramConfig,
    ) -> Result<TelegramConfig, NotificationError> {
        config.id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let config_data = serde_json::to_string(&config)?;

        sqlx::query(
            r#"
            INSERT INTO chat_integrations (service_type, config_id, config_data, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(ChatServiceType::Telegram.as_str())
        .bind(&config.id)
        .bind(&config_data)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(config)
    }

    pub async fn update_telegram_config(
        &self,
        id: &str,
        updates: TelegramConfig,
    ) -> Result<(), NotificationError> {
        let now = chrono::Utc::now().to_rfc3339();
        let config_data = serde_json::to_string(&updates)?;

        sqlx::query(
            r#"
            UPDATE chat_integrations
            SET config_data = ?1, updated_at = ?2
            WHERE service_type = ?3 AND config_id = ?4
            "#,
        )
        .bind(&config_data)
        .bind(&now)
        .bind(ChatServiceType::Telegram.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_telegram_config(&self, id: &str) -> Result<(), NotificationError> {
        sqlx::query(
            r#"
            DELETE FROM chat_integrations
            WHERE service_type = ?1 AND config_id = ?2
            "#,
        )
        .bind(ChatServiceType::Telegram.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_slack_config(
        &self,
        mut config: SlackConfig,
    ) -> Result<SlackConfig, NotificationError> {
        config.id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let config_data = serde_json::to_string(&config)?;

        sqlx::query(
            r#"
            INSERT INTO chat_integrations (service_type, config_id, config_data, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(ChatServiceType::Slack.as_str())
        .bind(&config.id)
        .bind(&config_data)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(config)
    }

    pub async fn update_slack_config(
        &self,
        id: &str,
        updates: SlackConfig,
    ) -> Result<(), NotificationError> {
        let now = chrono::Utc::now().to_rfc3339();
        let config_data = serde_json::to_string(&updates)?;

        sqlx::query(
            r#"
            UPDATE chat_integrations
            SET config_data = ?1, updated_at = ?2
            WHERE service_type = ?3 AND config_id = ?4
            "#,
        )
        .bind(&config_data)
        .bind(&now)
        .bind(ChatServiceType::Slack.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_slack_config(&self, id: &str) -> Result<(), NotificationError> {
        sqlx::query(
            r#"
            DELETE FROM chat_integrations
            WHERE service_type = ?1 AND config_id = ?2
            "#,
        )
        .bind(ChatServiceType::Slack.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_discord_config(
        &self,
        mut config: DiscordConfig,
    ) -> Result<DiscordConfig, NotificationError> {
        config.id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let config_data = serde_json::to_string(&config)?;

        sqlx::query(
            r#"
            INSERT INTO chat_integrations (service_type, config_id, config_data, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(ChatServiceType::Discord.as_str())
        .bind(&config.id)
        .bind(&config_data)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(config)
    }

    pub async fn update_discord_config(
        &self,
        id: &str,
        updates: DiscordConfig,
    ) -> Result<(), NotificationError> {
        let now = chrono::Utc::now().to_rfc3339();
        let config_data = serde_json::to_string(&updates)?;

        sqlx::query(
            r#"
            UPDATE chat_integrations
            SET config_data = ?1, updated_at = ?2
            WHERE service_type = ?3 AND config_id = ?4
            "#,
        )
        .bind(&config_data)
        .bind(&now)
        .bind(ChatServiceType::Discord.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_discord_config(&self, id: &str) -> Result<(), NotificationError> {
        sqlx::query(
            r#"
            DELETE FROM chat_integrations
            WHERE service_type = ?1 AND config_id = ?2
            "#,
        )
        .bind(ChatServiceType::Discord.as_str())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn test_telegram(
        &self,
        id: &str,
        message: &str,
    ) -> Result<TestMessageResult, NotificationError> {
        let config = self.get_telegram_config(id).await?;
        let start = std::time::Instant::now();

        match self
            .telegram_client
            .send_message(&config, message, false)
            .await
        {
            Ok(_) => {
                let duration = start.elapsed().as_millis() as u64;
                Ok(TestMessageResult {
                    success: true,
                    message: "Test message sent successfully".to_string(),
                    delivery_time: Some(duration),
                    error: None,
                })
            }
            Err(e) => Ok(TestMessageResult {
                success: false,
                message: "Failed to send test message".to_string(),
                delivery_time: None,
                error: Some(e.to_string()),
            }),
        }
    }

    pub async fn test_slack(
        &self,
        id: &str,
        message: &str,
    ) -> Result<TestMessageResult, NotificationError> {
        let config = self.get_slack_config(id).await?;
        let start = std::time::Instant::now();

        match self.slack_client.send_message(&config, message).await {
            Ok(_) => {
                let duration = start.elapsed().as_millis() as u64;
                Ok(TestMessageResult {
                    success: true,
                    message: "Test message sent successfully".to_string(),
                    delivery_time: Some(duration),
                    error: None,
                })
            }
            Err(e) => Ok(TestMessageResult {
                success: false,
                message: "Failed to send test message".to_string(),
                delivery_time: None,
                error: Some(e.to_string()),
            }),
        }
    }

    pub async fn test_discord(
        &self,
        id: &str,
        message: &str,
    ) -> Result<TestMessageResult, NotificationError> {
        let config = self.get_discord_config(id).await?;
        let start = std::time::Instant::now();

        match self
            .discord_client
            .send_message(&config, message, false)
            .await
        {
            Ok(_) => {
                let duration = start.elapsed().as_millis() as u64;
                Ok(TestMessageResult {
                    success: true,
                    message: "Test message sent successfully".to_string(),
                    delivery_time: Some(duration),
                    error: None,
                })
            }
            Err(e) => Ok(TestMessageResult {
                success: false,
                message: "Failed to send test message".to_string(),
                delivery_time: None,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn get_telegram_config(&self, id: &str) -> Result<TelegramConfig, NotificationError> {
        let row = sqlx::query(
            r#"
            SELECT config_data
            FROM chat_integrations
            WHERE service_type = ?1 AND config_id = ?2
            "#,
        )
        .bind(ChatServiceType::Telegram.as_str())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| NotificationError::ConfigNotFound(id.to_string()))?;

        let config_data: String = row.try_get("config_data")?;
        let config: TelegramConfig = serde_json::from_str(&config_data)?;
        Ok(config)
    }

    async fn get_slack_config(&self, id: &str) -> Result<SlackConfig, NotificationError> {
        let row = sqlx::query(
            r#"
            SELECT config_data
            FROM chat_integrations
            WHERE service_type = ?1 AND config_id = ?2
            "#,
        )
        .bind(ChatServiceType::Slack.as_str())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| NotificationError::ConfigNotFound(id.to_string()))?;

        let config_data: String = row.try_get("config_data")?;
        let config: SlackConfig = serde_json::from_str(&config_data)?;
        Ok(config)
    }

    async fn get_discord_config(&self, id: &str) -> Result<DiscordConfig, NotificationError> {
        let row = sqlx::query(
            r#"
            SELECT config_data
            FROM chat_integrations
            WHERE service_type = ?1 AND config_id = ?2
            "#,
        )
        .bind(ChatServiceType::Discord.as_str())
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| NotificationError::ConfigNotFound(id.to_string()))?;

        let config_data: String = row.try_get("config_data")?;
        let config: DiscordConfig = serde_json::from_str(&config_data)?;
        Ok(config)
    }

    pub async fn send_alert_notification(
        &self,
        alert_id: &str,
        alert_name: &str,
        symbol: &str,
        current_price: f64,
        condition: &str,
    ) -> Result<(), NotificationError> {
        let settings = self.get_settings().await?;

        for config in settings.telegram.iter().filter(|c| c.enabled) {
            let result = self
                .send_telegram_alert(
                    config,
                    alert_id,
                    alert_name,
                    symbol,
                    current_price,
                    condition,
                )
                .await;

            self.log_delivery(
                ChatServiceType::Telegram,
                &config.id,
                &config.name,
                Some(alert_id),
                Some(alert_name),
                "Alert notification",
                &result,
            )
            .await;
        }

        for config in settings.slack.iter().filter(|c| c.enabled) {
            let result = self
                .send_slack_alert(
                    config,
                    alert_id,
                    alert_name,
                    symbol,
                    current_price,
                    condition,
                )
                .await;

            self.log_delivery(
                ChatServiceType::Slack,
                &config.id,
                &config.name,
                Some(alert_id),
                Some(alert_name),
                "Alert notification",
                &result,
            )
            .await;
        }

        for config in settings.discord.iter().filter(|c| c.enabled) {
            let result = self
                .send_discord_alert(
                    config,
                    alert_id,
                    alert_name,
                    symbol,
                    current_price,
                    condition,
                )
                .await;

            self.log_delivery(
                ChatServiceType::Discord,
                &config.id,
                &config.name,
                Some(alert_id),
                Some(alert_name),
                "Alert notification",
                &result,
            )
            .await;
        }

        Ok(())
    }

    async fn send_telegram_alert(
        &self,
        config: &TelegramConfig,
        _alert_id: &str,
        alert_name: &str,
        symbol: &str,
        current_price: f64,
        condition: &str,
    ) -> Result<(), NotificationError> {
        let rate_limiter = self.rate_limiter.read().await;
        rate_limiter
            .acquire(&ChatServiceType::Telegram, &config.id)
            .await?;
        drop(rate_limiter);

        let message = format_alert_message(alert_name, symbol, current_price, condition, true);

        match self
            .telegram_client
            .send_message(config, &message, true)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                let rate_limiter = self.rate_limiter.read().await;
                rate_limiter
                    .register_failure(&ChatServiceType::Telegram, &config.id)
                    .await;
                Err(e)
            }
        }
    }

    async fn send_slack_alert(
        &self,
        config: &SlackConfig,
        _alert_id: &str,
        alert_name: &str,
        symbol: &str,
        current_price: f64,
        condition: &str,
    ) -> Result<(), NotificationError> {
        let rate_limiter = self.rate_limiter.read().await;
        rate_limiter
            .acquire(&ChatServiceType::Slack, &config.id)
            .await?;
        drop(rate_limiter);

        let message = format!(
            "*🚨 Price Alert Triggered*\n\n\
            *Alert:* {}\n\
            *Symbol:* {}\n\
            *Price:* ${:.4}\n\
            *Condition:* {}\n\n\
            _Triggered at: {}_",
            alert_name,
            symbol,
            current_price,
            condition,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        match self.slack_client.send_message(config, &message).await {
            Ok(_) => Ok(()),
            Err(e) => {
                let rate_limiter = self.rate_limiter.read().await;
                rate_limiter
                    .register_failure(&ChatServiceType::Slack, &config.id)
                    .await;
                Err(e)
            }
        }
    }

    async fn send_discord_alert(
        &self,
        config: &DiscordConfig,
        _alert_id: &str,
        alert_name: &str,
        symbol: &str,
        current_price: f64,
        condition: &str,
    ) -> Result<(), NotificationError> {
        let rate_limiter = self.rate_limiter.read().await;
        rate_limiter
            .acquire(&ChatServiceType::Discord, &config.id)
            .await?;
        drop(rate_limiter);

        match self
            .discord_client
            .send_alert_embed(config, alert_name, symbol, current_price, condition)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                let rate_limiter = self.rate_limiter.read().await;
                rate_limiter
                    .register_failure(&ChatServiceType::Discord, &config.id)
                    .await;
                Err(e)
            }
        }
    }

    async fn log_delivery(
        &self,
        service_type: ChatServiceType,
        config_id: &str,
        config_name: &str,
        alert_id: Option<&str>,
        alert_name: Option<&str>,
        message: &str,
        result: &Result<(), NotificationError>,
    ) {
        let (status, error) = match result {
            Ok(_) => (DeliveryStatus::Sent, None),
            Err(NotificationError::RateLimited(e)) => {
                (DeliveryStatus::RateLimited, Some(e.as_str()))
            }
            Err(e) => {
                (DeliveryStatus::Failed, None)
            }
        };

        if let Err(e) = self
            .delivery_logger
            .log(
                service_type,
                config_id,
                config_name,
                alert_id,
                alert_name,
                message,
                status,
                error,
                0,
            )
            .await
        {
            eprintln!("Failed to log delivery: {}", e);
        }
    }

    pub fn get_delivery_logger(&self) -> &DeliveryLogger {
        &self.delivery_logger
    }

    pub fn get_rate_limiter(&self) -> Arc<RwLock<RateLimiter>> {
        Arc::clone(&self.rate_limiter)
    }
}
