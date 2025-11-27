use crate::alerts::price_alerts::{
    AlertCondition, AlertConditionType, CompoundCondition, CreateAlertRequest, LogicalOperator,
    NotificationChannel, PriceAlert, SharedAlertManager,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceTradeCommand {
    pub action: String,
    pub token: String,
    pub amount: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioData {
    pub total_value: f64,
    pub positions: Vec<PortfolioPosition>,
    pub change_24h: f64,
    pub change_percent_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioPosition {
    pub token: String,
    pub amount: f64,
    pub value_usd: f64,
    pub change_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSummary {
    pub total_market_cap: f64,
    pub total_volume_24h: f64,
    pub btc_dominance: f64,
    pub trending_tokens: Vec<TrendingToken>,
    pub market_sentiment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingToken {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_24h: f64,
    pub volume_24h: f64,
}

/// Execute a voice trading command
/// This is a stub implementation - in production this would:
/// 1. Parse the voice command
/// 2. Validate permissions
/// 3. Execute through the trading engine
#[tauri::command]
pub async fn execute_voice_trade(command: String) -> Result<String, String> {
    // Parse command (stub)
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    // Stub response indicating command received
    Ok(format!(
        "Voice trade command received: {}. Trading feature is under development.",
        command
    ))
}

/// Get portfolio data for voice assistant
/// Returns mock data for now - in production would fetch real portfolio data
#[tauri::command]
pub async fn get_portfolio_data() -> Result<PortfolioData, String> {
    // Mock portfolio data
    Ok(PortfolioData {
        total_value: 50000.0,
        change_24h: 1250.0,
        change_percent_24h: 2.56,
        positions: vec![
            PortfolioPosition {
                token: "SOL".to_string(),
                amount: 150.0,
                value_usd: 15000.0,
                change_24h: 450.0,
            },
            PortfolioPosition {
                token: "USDC".to_string(),
                amount: 25000.0,
                value_usd: 25000.0,
                change_24h: 0.0,
            },
            PortfolioPosition {
                token: "RAY".to_string(),
                amount: 5000.0,
                value_usd: 10000.0,
                change_24h: 800.0,
            },
        ],
    })
}

/// Get current price for a token (voice command)
/// Delegates to market data service
#[tauri::command]
pub async fn get_current_price(token: String) -> Result<f64, String> {
    // Mock price data - in production would call market::get_coin_price
    match token.to_uppercase().as_str() {
        "SOL" => Ok(100.0),
        "USDC" => Ok(1.0),
        "RAY" => Ok(2.0),
        "BONK" => Ok(0.000015),
        "JUP" => Ok(0.85),
        _ => Ok(0.5), // Default mock price
    }
}

/// Create a price alert through voice command
/// Delegates to AlertManager
#[tauri::command]
pub async fn create_price_alert(
    alert_manager: State<'_, SharedAlertManager>,
    token: String,
    price: f64,
) -> Result<String, String> {
    let manager = alert_manager.read().await;

    // Create alert with above condition
    let condition = AlertCondition {
        condition_type: AlertConditionType::Above,
        value: price,
        timeframe_minutes: None,
    };

    let compound_condition = CompoundCondition {
        conditions: vec![condition],
        operator: LogicalOperator::And,
    };

    let request = CreateAlertRequest {
        name: format!("{} above ${}", token, price),
        symbol: token.clone(),
        mint: token.clone(), // In production, resolve to actual mint address
        watchlist_id: None,
        compound_condition,
        notification_channels: vec![NotificationChannel::InApp, NotificationChannel::System],
        cooldown_minutes: 60,
    };

    match manager.create_alert(request).await {
        Ok(alert) => Ok(alert.id),
        Err(e) => Err(format!("Failed to create alert: {}", e)),
    }
}

/// List all alerts (voice command)
/// Delegates to AlertManager
#[tauri::command]
pub async fn list_alerts(
    alert_manager: State<'_, SharedAlertManager>,
) -> Result<Vec<PriceAlert>, String> {
    let manager = alert_manager.read().await;
    match manager.list_alerts().await {
        Ok(alerts) => Ok(alerts),
        Err(e) => Err(format!("Failed to list alerts: {}", e)),
    }
}

/// Get market summary for voice assistant
/// Returns aggregated market data
#[tauri::command]
pub async fn get_market_summary() -> Result<MarketSummary, String> {
    // Mock market summary - in production would aggregate real data
    Ok(MarketSummary {
        total_market_cap: 2_500_000_000_000.0,
        total_volume_24h: 85_000_000_000.0,
        btc_dominance: 52.3,
        market_sentiment: "Bullish".to_string(),
        trending_tokens: vec![
            TrendingToken {
                symbol: "SOL".to_string(),
                name: "Solana".to_string(),
                price: 100.0,
                change_24h: 5.2,
                volume_24h: 1_200_000_000.0,
            },
            TrendingToken {
                symbol: "BONK".to_string(),
                name: "Bonk".to_string(),
                price: 0.000015,
                change_24h: 12.5,
                volume_24h: 45_000_000.0,
            },
            TrendingToken {
                symbol: "JUP".to_string(),
                name: "Jupiter".to_string(),
                price: 0.85,
                change_24h: 8.3,
                volume_24h: 30_000_000.0,
            },
        ],
    })
}

/// Synthesize speech from text (voice assistant)
/// This is a stub - in production would use TTS engine
#[tauri::command]
pub async fn synthesize_speech(text: String) -> Result<(), String> {
    // Stub - in production would delegate to TTS engine
    tracing::info!("Synthesizing speech: {}", text);
    Ok(())
}

/// Validate voice MFA code
/// This is a stub - in production would validate against stored codes
#[tauri::command]
pub async fn validate_voice_mfa(code: String) -> Result<bool, String> {
    // Stub validation - in production would check against actual MFA system
    // For now, accept any 6-digit code as valid
    Ok(code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()))
}

/// Check if voice trading is permitted for current session
/// This is a stub - in production would check permissions/settings
#[tauri::command]
pub async fn check_voice_permission() -> Result<bool, String> {
    // Stub - in production would check user settings and security policies
    // For now, return false to indicate voice trading needs explicit enablement
    Ok(false)
}

/// Get voice trading capabilities
/// Returns current status and available features
#[tauri::command]
pub async fn get_voice_capabilities() -> Result<serde_json::Value, String> {
    Ok(json!({
        "enabled": false,
        "features": {
            "portfolio_query": true,
            "price_alerts": true,
            "market_summary": true,
            "trade_execution": false,
            "mfa_verification": false
        },
        "supported_commands": [
            "get portfolio",
            "check price of [token]",
            "create alert for [token] at [price]",
            "list my alerts",
            "market summary"
        ],
        "message": "Voice trading is in beta. Some features are limited."
    }))
}
