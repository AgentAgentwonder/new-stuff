use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

use super::types::{NotificationError, SlackConfig};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize)]
struct SlackMessage<'a> {
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<&'a str>,
    #[serde(rename = "mrkdwn")]
    markdown: bool,
}

pub struct SlackClient {
    client: Client,
}

impl SlackClient {
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
        config: &SlackConfig,
        message: &str,
    ) -> Result<(), NotificationError> {
        let payload = SlackMessage {
            text: message,
            channel: config.channel.as_deref(),
            markdown: true,
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
                "Slack webhook failed: {} {}",
                status, body
            )));
        }

        Ok(())
    }

    pub async fn test_connection(&self, config: &SlackConfig) -> Result<(), NotificationError> {
        self.send_message(config, "Slack webhook connected successfully. ✅")
            .await
    }
}

impl Default for SlackClient {
    fn default() -> Self {
        Self::new()
    }
}
