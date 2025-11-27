use crate::mobile::{MobileSession, SharedMobileAuthManager};
use crate::trading::safety::policy::{SafetyCheck, SafetyPolicy};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickTradeRequest {
    pub session_token: String,
    pub symbol: String,
    pub side: TradeSide,
    pub amount: f64,
    pub biometric_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickTradeConfirmation {
    pub trade_id: String,
    pub symbol: String,
    pub side: TradeSide,
    pub amount: f64,
    pub executed_price: f64,
    pub timestamp: i64,
    pub status: TradeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeStatus {
    Executed,
    Rejected,
    Pending,
}

pub struct MobileTradeEngine {
    safety_policy: SafetyPolicy,
}

impl MobileTradeEngine {
    pub fn new() -> Self {
        Self {
            safety_policy: SafetyPolicy::default(),
        }
    }

    pub async fn execute_quick_trade(
        &self,
        trade: QuickTradeRequest,
        mobile_auth: SharedMobileAuthManager,
    ) -> Result<QuickTradeConfirmation> {
        if trade.biometric_signature.is_empty() {
            return Err(anyhow!("Biometric signature required"));
        }

        let session = {
            let auth = mobile_auth.read().await;
            auth.authenticate_session(trade.session_token.clone())
                .await?
        };

        self.enforce_safety_checks(&session, &trade)?;

        // Simulated execution
        let confirmation = QuickTradeConfirmation {
            trade_id: uuid::Uuid::new_v4().to_string(),
            symbol: trade.symbol,
            side: trade.side,
            amount: trade.amount,
            executed_price: 123.45,
            timestamp: Utc::now().timestamp(),
            status: TradeStatus::Executed,
        };

        Ok(confirmation)
    }

    fn enforce_safety_checks(
        &self,
        _session: &MobileSession,
        trade: &QuickTradeRequest,
    ) -> Result<()> {
        let checks = vec![
            SafetyCheck::MaxNotionalValue(50_000.0),
            SafetyCheck::MaxOrderSize(1_000.0),
        ];

        for check in checks {
            self.safety_policy
                .check_mobile_quick_trade(check, trade.amount)
                .map_err(|_| anyhow!("Trade violates safety policies"))?;
        }

        Ok(())
    }
}

#[tauri::command]
pub async fn mobile_execute_quick_trade(
    trade: QuickTradeRequest,
    trade_engine: tauri::State<'_, Arc<RwLock<MobileTradeEngine>>>,
    mobile_auth: tauri::State<'_, Arc<RwLock<crate::mobile::auth::MobileAuthManager>>>,
) -> Result<QuickTradeConfirmation, String> {
    let engine = trade_engine.read().await;
    engine
        .execute_quick_trade(trade, mobile_auth.inner().clone())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mobile_safety_checks(
    trade_engine: tauri::State<'_, Arc<RwLock<MobileTradeEngine>>>,
) -> Result<Vec<String>, String> {
    let engine = trade_engine.read().await;
    Ok(engine
        .safety_policy
        .mobile_quick_trade_rules()
        .into_iter()
        .map(|rule| format!("{}", rule))
        .collect())
}
