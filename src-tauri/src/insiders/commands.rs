use super::types::*;
use crate::insiders::wallet_monitor::require_state;
use sqlx::Row;

#[tauri::command]
pub async fn classify_smart_money_wallet(
    wallet_address: String,
) -> Result<SmartMoneyClassification, String> {
    let state = require_state()?;
    let detector = state.smart_money_detector.clone();
    detector.classify_wallet(&wallet_address).await
}

#[tauri::command]
pub async fn get_smart_money_wallets() -> Result<Vec<SmartMoneyWallet>, String> {
    let state = require_state()?;
    let detector = state.smart_money_detector.clone();
    detector.get_smart_money_wallets().await
}

#[tauri::command]
pub async fn get_smart_money_consensus(
    time_window_hours: i64,
) -> Result<Vec<SmartMoneyConsensus>, String> {
    let state = require_state()?;
    let detector = state.smart_money_detector.clone();
    detector.get_consensus(time_window_hours).await
}

#[tauri::command]
pub async fn get_sentiment_comparison(token_mint: String) -> Result<SentimentComparison, String> {
    let state = require_state()?;
    let detector = state.smart_money_detector.clone();
    detector.get_sentiment_comparison(&token_mint).await
}

#[tauri::command]
pub async fn get_alert_configs() -> Result<Vec<AlertConfig>, String> {
    let state = require_state()?;
    let alert_manager = state.alert_manager.clone();
    alert_manager.get_alert_configs().await
}

#[tauri::command]
pub async fn update_alert_config(config: AlertConfig) -> Result<(), String> {
    let state = require_state()?;
    let alert_manager = state.alert_manager.clone();
    alert_manager.update_alert_config(&config).await
}

#[tauri::command]
pub async fn get_recent_whale_alerts(limit: i64) -> Result<Vec<WhaleAlert>, String> {
    let state = require_state()?;
    let alert_manager = state.alert_manager.clone();
    alert_manager.get_recent_whale_alerts(limit).await
}

#[tauri::command]
pub async fn scan_wallets_for_smart_money() -> Result<Vec<SmartMoneyClassification>, String> {
    let state = require_state()?;
    let detector = state.smart_money_detector.clone();
    let pool = detector.pool();

    let activities = sqlx::query("SELECT DISTINCT wallet_address FROM wallet_activities")
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to fetch wallets: {e}"))?;

    let mut classifications = Vec::new();

    for row in activities {
        let wallet_address: String = row
            .try_get("wallet_address")
            .map_err(|e| format!("Failed to get wallet_address: {e}"))?;

        match detector.classify_wallet(&wallet_address).await {
            Ok(classification) => {
                if classification.is_smart_money {
                    let _ = detector
                        .update_smart_money_wallet(&classification, None)
                        .await;
                    classifications.push(classification);
                }
            }
            Err(e) => {
                eprintln!("Failed to classify wallet {}: {}", wallet_address, e);
            }
        }
    }

    Ok(classifications)
}
