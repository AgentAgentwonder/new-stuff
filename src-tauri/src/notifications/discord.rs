use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

use super::types::{DiscordConfig, NotificationError};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize)]
struct DiscordEmbed {
    title: String,
    description: String,
    color: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<EmbedField>>,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct EmbedField {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Debug, Serialize)]
struct DiscordMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<DiscordEmbed>>,
}

pub struct DiscordClient {
    client: Client,
}

impl DiscordClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    pub async fn send_message(
        &self,
        config: &DiscordConfig,
        message: &str,
        use_embed: bool,
    ) -> Result<(), NotificationError> {
        let payload = if use_embed {
            DiscordMessage {
                content: None,
                username: config.username.clone(),
                embeds: Some(vec![DiscordEmbed {
                    title: "Alert Notification".to_string(),
                    description: message.to_string(),
                    color: 0x5865F2,
                    fields: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }]),
            }
        } else {
            DiscordMessage {
                content: Some(message.to_string()),
                username: config.username.clone(),
                embeds: None,
            }
        };

        let response = self
            .client
            .post(&config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::Internal(format!(
                "Discord webhook failed: {} {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn send_alert_embed(
        &self,
        config: &DiscordConfig,
        alert_name: &str,
        symbol: &str,
        current_price: f64,
        condition: &str,
    ) -> Result<(), NotificationError> {
        let role_mentions = config
            .role_mentions
            .as_ref()
            .map(|mentions| {
                mentions
                    .iter()
                    .map(|id| format!("<@&{}>", id))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        let content = if !role_mentions.is_empty() {
            Some(role_mentions)
        } else {
            None
        };

        let embed = DiscordEmbed {
            title: "🚨 Price Alert Triggered".to_string(),
            description: format!("Alert **{}** has been triggered", alert_name),
            color: 0xFF0000,
            fields: Some(vec![
                EmbedField {
                    name: "Symbol".to_string(),
                    value: symbol.to_string(),
                    inline: true,
                },
                EmbedField {
                    name: "Price".to_string(),
                    value: format!("${:.4}", current_price),
                    inline: true,
                },
                EmbedField {
                    name: "Condition".to_string(),
                    value: condition.to_string(),
                    inline: false,
                },
            ]),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let payload = DiscordMessage {
            content,
            username: config.username.clone(),
            embeds: Some(vec![embed]),
        };

        let response = self
            .client
            .post(&config.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::Internal(format!(
                "Discord webhook failed: {} {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn test_connection(&self, config: &DiscordConfig) -> Result<(), NotificationError> {
        self.send_message(config, "Discord webhook connected successfully. ✅", false)
            .await
    }
}

impl Default for DiscordClient {
    fn default() -> Self {
        Self::new()
    }
}
