use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::types::{NotificationError, TelegramConfig};

const TELEGRAM_API_URL: &str = "https://api.telegram.org";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize)]
struct TelegramMessage {
    chat_id: String,
    text: String,
    parse_mode: Option<String>,
    disable_web_page_preview: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TelegramResponse {
    ok: bool,
    description: Option<String>,
}

pub struct TelegramClient {
    client: Client,
}

impl TelegramClient {
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
        config: &TelegramConfig,
        message: &str,
        use_markdown: bool,
    ) -> Result<(), NotificationError> {
        let url = format!("{}/bot{}/sendMessage", TELEGRAM_API_URL, config.bot_token);

        let formatted_message = if use_markdown {
            Self::format_markdown(message)
        } else {
            message.to_string()
        };

        let payload = TelegramMessage {
            chat_id: config.chat_id.clone(),
            text: formatted_message,
            parse_mode: if use_markdown {
                Some("MarkdownV2".to_string())
            } else {
                None
            },
            disable_web_page_preview: Some(true),
        };

        let response = self.client.post(&url).json(&payload).send().await?;

        let status = response.status();
        let body: TelegramResponse = response.json().await?;

        if !body.ok || !status.is_success() {
            return Err(NotificationError::Internal(
                body.description
                    .unwrap_or_else(|| format!("Telegram API error: {}", status)),
            ));
        }

        Ok(())
    }

    pub async fn test_connection(&self, config: &TelegramConfig) -> Result<(), NotificationError> {
        let url = format!("{}/bot{}/getMe", TELEGRAM_API_URL, config.bot_token);

        let response = self.client.get(&url).send().await?;

        let status = response.status();
        let body: TelegramResponse = response.json().await?;

        if !body.ok || !status.is_success() {
            return Err(NotificationError::Internal(
                body.description
                    .unwrap_or_else(|| format!("Invalid bot token: {}", status)),
            ));
        }

        Ok(())
    }

    fn format_markdown(text: &str) -> String {
        text.replace('.', "\\.")
            .replace('-', "\\-")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('!', "\\!")
            .replace('=', "\\=")
            .replace('+', "\\+")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('>', "\\>")
            .replace('<', "\\<")
            .replace('|', "\\|")
            .replace('#', "\\#")
    }

    pub fn escape_markdown(text: &str) -> String {
        Self::format_markdown(text)
    }
}

impl Default for TelegramClient {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_alert_message(
    alert_name: &str,
    symbol: &str,
    current_price: f64,
    condition: &str,
    use_markdown: bool,
) -> String {
    if use_markdown {
        format!(
            "*🚨 Price Alert Triggered*\n\n\
            *Alert:* {}\n\
            *Symbol:* {}\n\
            *Price:* ${:.4}\n\
            *Condition:* {}\n\n\
            _Triggered at: {}_",
            TelegramClient::escape_markdown(alert_name),
            TelegramClient::escape_markdown(symbol),
            current_price,
            TelegramClient::escape_markdown(condition),
            TelegramClient::escape_markdown(
                &chrono::Utc::now()
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string()
            )
        )
    } else {
        format!(
            "🚨 Price Alert Triggered\n\n\
            Alert: {}\n\
            Symbol: {}\n\
            Price: ${:.4}\n\
            Condition: {}\n\n\
            Triggered at: {}",
            alert_name,
            symbol,
            current_price,
            condition,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}
