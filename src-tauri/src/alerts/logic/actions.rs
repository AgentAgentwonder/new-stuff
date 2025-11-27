use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Notify,
    SendEmail,
    SendWebhook,
    SendTelegram,
    SendSlack,
    SendDiscord,
    ExecuteTrade,
    PauseStrategy,
    UpdateAlert,
    LogEvent,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Notify => "notify",
            ActionType::SendEmail => "send_email",
            ActionType::SendWebhook => "send_webhook",
            ActionType::SendTelegram => "send_telegram",
            ActionType::SendSlack => "send_slack",
            ActionType::SendDiscord => "send_discord",
            ActionType::ExecuteTrade => "execute_trade",
            ActionType::PauseStrategy => "pause_strategy",
            ActionType::UpdateAlert => "update_alert",
            ActionType::LogEvent => "log_event",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "notify" | "notification" => Some(ActionType::Notify),
            "send_email" | "email" => Some(ActionType::SendEmail),
            "send_webhook" | "webhook" => Some(ActionType::SendWebhook),
            "send_telegram" | "telegram" => Some(ActionType::SendTelegram),
            "send_slack" | "slack" => Some(ActionType::SendSlack),
            "send_discord" | "discord" => Some(ActionType::SendDiscord),
            "execute_trade" | "trade" | "auto_trade" => Some(ActionType::ExecuteTrade),
            "pause_strategy" | "pause" => Some(ActionType::PauseStrategy),
            "update_alert" | "update" => Some(ActionType::UpdateAlert),
            "log_event" | "log" => Some(ActionType::LogEvent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    pub action_type: ActionType,

    #[serde(default)]
    pub parameters: ActionParameters,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActionParameters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<NotificationPriority>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_to: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email_subject: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_config: Option<TradeConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        NotificationPriority::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeConfig {
    pub token_mint: String,
    pub side: TradeSide,
    pub order_type: OrderType,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount_percent: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,

    #[serde(default)]
    pub slippage_bps: u16,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_loss_percent: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub take_profit_percent: Option<f64>,

    #[serde(default)]
    pub max_retries: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionExecutionResult {
    pub action_id: String,
    pub success: bool,
    pub message: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    pub executed_at: String,
}

impl Action {
    pub fn action_id(&self) -> String {
        self.id
            .clone()
            .unwrap_or_else(|| self.action_type.as_str().to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.action_type {
            ActionType::SendEmail => {
                if self.parameters.email_to.is_none()
                    || self.parameters.email_to.as_ref().unwrap().is_empty()
                {
                    return Err("Email action requires 'email_to' parameter".to_string());
                }
            }
            ActionType::SendWebhook => {
                if self.parameters.webhook_url.is_none() {
                    return Err("Webhook action requires 'webhook_url' parameter".to_string());
                }
            }
            ActionType::SendTelegram => {
                if self.parameters.chat_id.is_none() {
                    return Err("Telegram action requires 'chat_id' parameter".to_string());
                }
            }
            ActionType::SendSlack | ActionType::SendDiscord => {
                if self.parameters.webhook_url.is_none() {
                    return Err(format!(
                        "{} action requires 'webhook_url' parameter",
                        self.action_type.as_str()
                    ));
                }
            }
            ActionType::ExecuteTrade => {
                if self.parameters.trade_config.is_none() {
                    return Err("ExecuteTrade action requires 'trade_config' parameter".to_string());
                }
                let trade_config = self.parameters.trade_config.as_ref().unwrap();
                if trade_config.amount.is_none() && trade_config.amount_percent.is_none() {
                    return Err(
                        "Trade config requires either 'amount' or 'amount_percent'".to_string()
                    );
                }
            }
            ActionType::PauseStrategy => {
                if self.parameters.strategy_id.is_none() {
                    return Err("PauseStrategy action requires 'strategy_id' parameter".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn build_message(&self, context: &serde_json::Value) -> String {
        if let Some(template) = &self.parameters.message {
            interpolate_template(template, context)
        } else {
            match self.action_type {
                ActionType::Notify => "Alert condition met".to_string(),
                ActionType::ExecuteTrade => "Executing trade based on alert condition".to_string(),
                ActionType::PauseStrategy => "Pausing strategy due to alert condition".to_string(),
                _ => "Alert triggered".to_string(),
            }
        }
    }
}

fn interpolate_template(template: &str, context: &serde_json::Value) -> String {
    let mut result = template.to_string();

    if let Some(obj) = context.as_object() {
        for (key, value) in obj {
            let placeholder = format!("{{{}}}", key);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionExecutionContext {
    pub alert_id: String,
    pub alert_name: String,
    pub symbol: String,
    pub current_price: f64,
    pub conditions_met: String,
    pub trigger_data: serde_json::Value,
    pub dry_run: bool,
}

impl ActionExecutionContext {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "alertId": self.alert_id,
            "alertName": self.alert_name,
            "symbol": self.symbol,
            "currentPrice": self.current_price,
            "conditionsMet": self.conditions_met,
            "triggerData": self.trigger_data,
        })
    }
}
