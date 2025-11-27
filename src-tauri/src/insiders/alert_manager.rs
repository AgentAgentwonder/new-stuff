use super::types::*;
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

pub struct AlertManager {
    pool: SqlitePool,
    app_handle: AppHandle,
}

impl AlertManager {
    pub fn new(pool: SqlitePool, app_handle: AppHandle) -> Self {
        Self { pool, app_handle }
    }

    pub async fn process_whale_transaction(&self, activity: &WalletActivity) -> Result<(), String> {
        let configs = self.get_alert_configs().await?;
        let whale_config = configs
            .iter()
            .find(|c| c.alert_type == AlertType::WhaleTransaction && c.enabled);

        if let Some(config) = whale_config {
            if let Some(amount_usd) = activity.amount_usd {
                let threshold = config.threshold.unwrap_or(50000.0);

                if amount_usd >= threshold {
                    let alert = WhaleAlert {
                        id: Uuid::new_v4().to_string(),
                        wallet_address: activity.wallet_address.clone(),
                        wallet_label: activity.wallet_label.clone(),
                        activity_id: activity.id.clone(),
                        tx_signature: activity.tx_signature.clone(),
                        action_type: activity.action_type.clone(),
                        token_symbol: activity.output_symbol.clone(),
                        amount_usd,
                        threshold,
                        alert_sent: false,
                        timestamp: activity.timestamp,
                    };

                    self.save_whale_alert(&alert).await?;
                    self.send_alert(config, &alert).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn process_smart_money_activity(
        &self,
        activity: &WalletActivity,
        is_smart_money: bool,
    ) -> Result<(), String> {
        if !is_smart_money {
            return Ok(());
        }

        let configs = self.get_alert_configs().await?;

        let alert_type = match activity.action_type.as_str() {
            "buy" => AlertType::SmartMoneyBuy,
            "sell" => AlertType::SmartMoneySell,
            _ => return Ok(()),
        };

        let config = configs
            .iter()
            .find(|c| c.alert_type == alert_type && c.enabled);

        if let Some(config) = config {
            if let Some(amount_usd) = activity.amount_usd {
                let threshold = config.threshold.unwrap_or(10000.0);

                if amount_usd >= threshold {
                    let message =
                        format!(
                        "🧠 Smart Money {} Alert\n\nWallet: {}\nToken: {}\nAmount: ${:.2}\nTx: {}",
                        activity.action_type.to_uppercase(),
                        activity.wallet_label
                            .as_ref()
                            .unwrap_or(&activity.wallet_address),
                        activity.output_symbol.as_ref().unwrap_or(&"Unknown".to_string()),
                        amount_usd,
                        &activity.tx_signature[..8]
                    );

                    self.send_push_notification(config, &message, activity)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn save_whale_alert(&self, alert: &WhaleAlert) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO whale_alerts (
                id, wallet_address, wallet_label, activity_id, tx_signature,
                action_type, token_symbol, amount_usd, threshold, alert_sent, timestamp
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&alert.id)
        .bind(&alert.wallet_address)
        .bind(&alert.wallet_label)
        .bind(&alert.activity_id)
        .bind(&alert.tx_signature)
        .bind(&alert.action_type)
        .bind(&alert.token_symbol)
        .bind(alert.amount_usd)
        .bind(alert.threshold)
        .bind(if alert.alert_sent { 1 } else { 0 })
        .bind(alert.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save whale alert: {e}"))?;

        Ok(())
    }

    async fn send_alert(&self, config: &AlertConfig, alert: &WhaleAlert) -> Result<(), String> {
        let message = format!(
            "🐋 Whale Alert!\n\nWallet: {}\nAction: {}\nToken: {}\nAmount: ${:.2}\nThreshold: ${:.2}\nTx: {}",
            alert.wallet_label.as_ref().unwrap_or(&alert.wallet_address),
            alert.action_type.to_uppercase(),
            alert.token_symbol.as_ref().unwrap_or(&"Unknown".to_string()),
            alert.amount_usd,
            alert.threshold,
            &alert.tx_signature[..8]
        );

        if config.push_enabled {
            let _ = self.app_handle.emit("whale_alert", alert);
        }

        if config.telegram_enabled {
            if let Some(telegram_config_id) = &config.telegram_config_id {
                let _ = self
                    .app_handle
                    .emit("send_telegram_alert", (telegram_config_id, &message));
            }
        }

        Ok(())
    }

    async fn send_push_notification(
        &self,
        config: &AlertConfig,
        message: &str,
        activity: &WalletActivity,
    ) -> Result<(), String> {
        if config.push_enabled {
            let _ = self.app_handle.emit("smart_money_alert", activity);
        }

        if config.telegram_enabled {
            if let Some(telegram_config_id) = &config.telegram_config_id {
                let _ = self
                    .app_handle
                    .emit("send_telegram_alert", (telegram_config_id, message));
            }
        }

        Ok(())
    }

    pub async fn get_alert_configs(&self) -> Result<Vec<AlertConfig>, String> {
        let rows = sqlx::query(
            "SELECT id, alert_type, enabled, threshold, push_enabled, email_enabled, telegram_enabled, telegram_config_id FROM alert_configs"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch alert configs: {e}"))?;

        let mut configs = Vec::new();
        for row in rows {
            let alert_type_str: String = row
                .try_get("alert_type")
                .map_err(|e| format!("Failed to get alert_type: {e}"))?;
            let alert_type = match alert_type_str.as_str() {
                "whale_transaction" => AlertType::WhaleTransaction,
                "smart_money_buy" => AlertType::SmartMoneyBuy,
                "smart_money_sell" => AlertType::SmartMoneySell,
                "smart_money_consensus" => AlertType::SmartMoneyConsensus,
                _ => continue,
            };

            configs.push(AlertConfig {
                id: row
                    .try_get("id")
                    .map_err(|e| format!("Failed to get id: {e}"))?,
                alert_type,
                enabled: row
                    .try_get::<i64, _>("enabled")
                    .map_err(|e| format!("Failed to get enabled: {e}"))?
                    == 1,
                threshold: row.try_get("threshold").ok(),
                push_enabled: row
                    .try_get::<i64, _>("push_enabled")
                    .map_err(|e| format!("Failed to get push_enabled: {e}"))?
                    == 1,
                email_enabled: row
                    .try_get::<i64, _>("email_enabled")
                    .map_err(|e| format!("Failed to get email_enabled: {e}"))?
                    == 1,
                telegram_enabled: row
                    .try_get::<i64, _>("telegram_enabled")
                    .map_err(|e| format!("Failed to get telegram_enabled: {e}"))?
                    == 1,
                telegram_config_id: row.try_get("telegram_config_id").ok(),
            });
        }

        Ok(configs)
    }

    pub async fn update_alert_config(&self, config: &AlertConfig) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE alert_configs
            SET enabled = ?1, threshold = ?2, push_enabled = ?3, email_enabled = ?4, telegram_enabled = ?5, telegram_config_id = ?6
            WHERE id = ?7
            "#,
        )
        .bind(if config.enabled { 1 } else { 0 })
        .bind(config.threshold)
        .bind(if config.push_enabled { 1 } else { 0 })
        .bind(if config.email_enabled { 1 } else { 0 })
        .bind(if config.telegram_enabled { 1 } else { 0 })
        .bind(&config.telegram_config_id)
        .bind(&config.id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update alert config: {e}"))?;

        Ok(())
    }

    pub async fn get_recent_whale_alerts(&self, limit: i64) -> Result<Vec<WhaleAlert>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, wallet_address, wallet_label, activity_id, tx_signature,
                   action_type, token_symbol, amount_usd, threshold, alert_sent, timestamp
            FROM whale_alerts
            ORDER BY timestamp DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch whale alerts: {e}"))?;

        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(WhaleAlert {
                id: row.try_get("id").unwrap_or_default(),
                wallet_address: row.try_get("wallet_address").unwrap_or_default(),
                wallet_label: row.try_get("wallet_label").ok(),
                activity_id: row.try_get("activity_id").unwrap_or_default(),
                tx_signature: row.try_get("tx_signature").unwrap_or_default(),
                action_type: row.try_get("action_type").unwrap_or_default(),
                token_symbol: row.try_get("token_symbol").ok(),
                amount_usd: row.try_get("amount_usd").unwrap_or(0.0),
                threshold: row.try_get("threshold").unwrap_or(0.0),
                alert_sent: row.try_get::<i64, _>("alert_sent").unwrap_or(0) == 1,
                timestamp: row
                    .try_get::<String, _>("timestamp")
                    .ok()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|| Utc::now()),
            });
        }

        Ok(alerts)
    }
}
