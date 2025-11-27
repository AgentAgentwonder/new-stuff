use crate::trading::safety::policy::SafetyPolicy;
use crate::trading::safety::{
    InsuranceProvider, SafetyCheckRequest, SafetyCheckResult, SharedSafetyEngine,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SafetyConfigUpdate {
    pub policy: SafetyPolicy,
}

#[tauri::command]
pub async fn check_trade_safety(
    request: SafetyCheckRequest,
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<SafetyCheckResult, String> {
    let mut engine = safety_engine.write().await;
    engine.check_trade_safety(request).await
}

#[tauri::command]
pub async fn approve_trade(
    wallet_address: String,
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<(), String> {
    let mut engine = safety_engine.write().await;
    engine.approve_trade(&wallet_address);
    Ok(())
}

#[tauri::command]
pub async fn get_safety_policy(
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<SafetyPolicy, String> {
    let engine = safety_engine.read().await;
    Ok(engine.get_policy().clone())
}

#[tauri::command]
pub async fn update_safety_policy(
    policy: SafetyPolicy,
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<(), String> {
    let mut engine = safety_engine.write().await;
    engine.update_policy(policy);
    Ok(())
}

#[tauri::command]
pub async fn get_cooldown_status(
    wallet_address: String,
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<Option<crate::trading::safety::cooldown::CooldownStatus>, String> {
    let engine = safety_engine.read().await;
    Ok(engine.get_cooldown_status(&wallet_address))
}

#[tauri::command]
pub async fn reset_daily_limits(
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<(), String> {
    let mut engine = safety_engine.write().await;
    engine.reset_daily_limits();
    Ok(())
}

#[tauri::command]
pub async fn get_insurance_quote(
    provider_id: String,
    trade_amount_usd: f64,
    price_impact_percent: f64,
    mev_risk_level: f64,
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<crate::trading::safety::insurance::InsuranceQuote, String> {
    let mut engine = safety_engine.write().await;
    engine.get_insurance_quote(
        &provider_id,
        trade_amount_usd,
        price_impact_percent,
        mev_risk_level,
    )
}

#[tauri::command]
pub async fn select_insurance(
    provider_id: String,
    trade_amount_usd: f64,
    price_impact_percent: f64,
    mev_risk_level: f64,
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<crate::trading::safety::insurance::InsuranceSelection, String> {
    let mut engine = safety_engine.write().await;
    engine.select_insurance(
        &provider_id,
        trade_amount_usd,
        price_impact_percent,
        mev_risk_level,
    )
}

#[tauri::command]
pub async fn list_insurance_providers(
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<Vec<InsuranceProvider>, String> {
    let engine = safety_engine.read().await;
    Ok(engine.list_insurance_providers())
}
