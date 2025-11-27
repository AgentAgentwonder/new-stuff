use super::manager::WebhookManager;
use super::types::{WebhookConfig, WebhookDeliveryLog, WebhookError, WebhookTestResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub type SharedWebhookManager = Arc<RwLock<WebhookManager>>;

#[tauri::command]
pub async fn list_webhooks(
    manager: State<'_, SharedWebhookManager>,
) -> Result<Vec<WebhookConfig>, String> {
    let mgr = manager.read().await;
    mgr.list_webhooks().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_webhook(
    manager: State<'_, SharedWebhookManager>,
    id: String,
) -> Result<WebhookConfig, String> {
    let mgr = manager.read().await;
    mgr.get_webhook(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_webhook(
    manager: State<'_, SharedWebhookManager>,
    config: WebhookConfig,
) -> Result<WebhookConfig, String> {
    let mgr = manager.read().await;
    mgr.create_webhook(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_webhook(
    manager: State<'_, SharedWebhookManager>,
    id: String,
    config: WebhookConfig,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.update_webhook(&id, config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_webhook(
    manager: State<'_, SharedWebhookManager>,
    id: String,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_webhook(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn trigger_webhook(
    manager: State<'_, SharedWebhookManager>,
    id: String,
    variables: HashMap<String, Value>,
) -> Result<WebhookDeliveryLog, String> {
    let mgr = manager.read().await;
    mgr.trigger_webhook(&id, variables)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_webhook(
    manager: State<'_, SharedWebhookManager>,
    id: String,
    variables: HashMap<String, Value>,
) -> Result<WebhookTestResult, String> {
    let mgr = manager.read().await;
    mgr.test_webhook(&id, variables)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_webhook_delivery_logs(
    manager: State<'_, SharedWebhookManager>,
    webhook_id: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<WebhookDeliveryLog>, String> {
    let mgr = manager.read().await;
    mgr.list_delivery_logs(webhook_id.as_deref(), limit.unwrap_or(100))
        .await
        .map_err(|e| e.to_string())
}
