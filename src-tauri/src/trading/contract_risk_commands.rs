use crate::trading::contract_risk::{ContractAssessment, RiskEvent, SharedContractRiskService};
use crate::trading::safety::{SafetyCheckRequest, SafetyCheckResult, SharedSafetyEngine};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreTradeSafetyResult {
    pub contract_assessment: ContractAssessment,
    pub safety_result: SafetyCheckResult,
    pub security_score: f64,
}

#[tauri::command]
pub async fn assess_contract_risk(
    contract_address: String,
    service: State<'_, SharedContractRiskService>,
) -> Result<ContractAssessment, String> {
    let service = service.read().await;
    service
        .assess_contract(&contract_address)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_contract_risk_events(
    contract_address: String,
    limit: Option<i64>,
    service: State<'_, SharedContractRiskService>,
) -> Result<Vec<RiskEvent>, String> {
    let service = service.read().await;
    service
        .list_risk_events(&contract_address, limit.unwrap_or(25))
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn monitor_contract(
    contract_address: String,
    service: State<'_, SharedContractRiskService>,
) -> Result<(), String> {
    let service = service.read().await;
    service.monitor_contract(&contract_address).await;
    Ok(())
}

#[tauri::command]
pub async fn unmonitor_contract(
    contract_address: String,
    service: State<'_, SharedContractRiskService>,
) -> Result<(), String> {
    let service = service.read().await;
    service.unmonitor_contract(&contract_address).await;
    Ok(())
}

#[tauri::command]
pub async fn list_monitored_contracts(
    service: State<'_, SharedContractRiskService>,
) -> Result<Vec<String>, String> {
    let service = service.read().await;
    Ok(service.list_monitored_contracts().await)
}

#[tauri::command]
pub async fn refresh_monitored_contracts(
    service: State<'_, SharedContractRiskService>,
) -> Result<Vec<ContractAssessment>, String> {
    let service = service.read().await;
    service
        .refresh_monitored_contracts()
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn pre_trade_contract_check(
    contract_address: String,
    mut request: SafetyCheckRequest,
    service: State<'_, SharedContractRiskService>,
    safety_engine: State<'_, SharedSafetyEngine>,
) -> Result<PreTradeSafetyResult, String> {
    let assessment = {
        let service = service.read().await;
        service
            .assess_contract(&contract_address)
            .await
            .map_err(|err| err.to_string())?
    };

    let security_score = ((1.0 - assessment.risk_score) * 100.0).clamp(0.0, 100.0);
    request.security_score = Some(security_score);

    let safety_result = {
        let mut engine = safety_engine.write().await;
        engine.check_trade_safety(request).await?
    };

    Ok(PreTradeSafetyResult {
        contract_assessment: assessment,
        safety_result,
        security_score,
    })
}
