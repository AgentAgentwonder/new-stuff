use tauri::State;

use super::router::SharedNotificationRouter;
use super::types::{
    ChatIntegrationSettings, DeliveryLog, DiscordConfig, RateLimitStatus, SlackConfig,
    TelegramConfig, TestMessageResult,
};

#[tauri::command]
pub async fn chat_integration_get_settings(
    router: State<'_, SharedNotificationRouter>,
) -> Result<ChatIntegrationSettings, String> {
    let router = router.read().await;
    router
        .get_settings()
        .await
        .map_err(|e| format!("Failed to get settings: {}", e))
}

#[tauri::command]
pub async fn chat_integration_save_settings(
    settings: ChatIntegrationSettings,
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    router
        .save_settings(&settings)
        .await
        .map_err(|e| format!("Failed to save settings: {}", e))
}

#[tauri::command]
pub async fn chat_integration_add_telegram(
    config: TelegramConfig,
    router: State<'_, SharedNotificationRouter>,
) -> Result<TelegramConfig, String> {
    let router = router.read().await;
    router
        .add_telegram_config(config)
        .await
        .map_err(|e| format!("Failed to add Telegram config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_update_telegram(
    id: String,
    config: TelegramConfig,
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    router
        .update_telegram_config(&id, config)
        .await
        .map_err(|e| format!("Failed to update Telegram config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_delete_telegram(
    id: String,
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    router
        .delete_telegram_config(&id)
        .await
        .map_err(|e| format!("Failed to delete Telegram config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_add_slack(
    config: SlackConfig,
    router: State<'_, SharedNotificationRouter>,
) -> Result<SlackConfig, String> {
    let router = router.read().await;
    router
        .add_slack_config(config)
        .await
        .map_err(|e| format!("Failed to add Slack config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_update_slack(
    id: String,
    config: SlackConfig,
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    router
        .update_slack_config(&id, config)
        .await
        .map_err(|e| format!("Failed to update Slack config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_delete_slack(
    id: String,
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    router
        .delete_slack_config(&id)
        .await
        .map_err(|e| format!("Failed to delete Slack config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_add_discord(
    config: DiscordConfig,
    router: State<'_, SharedNotificationRouter>,
) -> Result<DiscordConfig, String> {
    let router = router.read().await;
    router
        .add_discord_config(config)
        .await
        .map_err(|e| format!("Failed to add Discord config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_update_discord(
    id: String,
    config: DiscordConfig,
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    router
        .update_discord_config(&id, config)
        .await
        .map_err(|e| format!("Failed to update Discord config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_delete_discord(
    id: String,
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    router
        .delete_discord_config(&id)
        .await
        .map_err(|e| format!("Failed to delete Discord config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_test_telegram(
    id: String,
    message: String,
    router: State<'_, SharedNotificationRouter>,
) -> Result<TestMessageResult, String> {
    let router = router.read().await;
    router
        .test_telegram(&id, &message)
        .await
        .map_err(|e| format!("Failed to test Telegram config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_test_slack(
    id: String,
    message: String,
    router: State<'_, SharedNotificationRouter>,
) -> Result<TestMessageResult, String> {
    let router = router.read().await;
    router
        .test_slack(&id, &message)
        .await
        .map_err(|e| format!("Failed to test Slack config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_test_discord(
    id: String,
    message: String,
    router: State<'_, SharedNotificationRouter>,
) -> Result<TestMessageResult, String> {
    let router = router.read().await;
    router
        .test_discord(&id, &message)
        .await
        .map_err(|e| format!("Failed to test Discord config: {}", e))
}

#[tauri::command]
pub async fn chat_integration_get_delivery_logs(
    limit: i32,
    service_type: Option<String>,
    router: State<'_, SharedNotificationRouter>,
) -> Result<Vec<DeliveryLog>, String> {
    let router = router.read().await;
    let logger = router.get_delivery_logger();
    logger
        .get_logs(limit, service_type.as_deref())
        .await
        .map_err(|e| format!("Failed to get delivery logs: {}", e))
}

#[tauri::command]
pub async fn chat_integration_clear_delivery_logs(
    router: State<'_, SharedNotificationRouter>,
) -> Result<(), String> {
    let router = router.read().await;
    let logger = router.get_delivery_logger();
    logger
        .clear_logs()
        .await
        .map_err(|e| format!("Failed to clear delivery logs: {}", e))
}

#[tauri::command]
pub async fn chat_integration_get_rate_limits(
    router: State<'_, SharedNotificationRouter>,
) -> Result<Vec<RateLimitStatus>, String> {
    let router = router.read().await;
    let rate_limiter = router.get_rate_limiter();
    let limiter = rate_limiter.read().await;
    Ok(limiter.get_statuses().await)
}
