pub mod cooldown;
pub mod insurance;
pub mod policy;
pub mod simulator;

use cooldown::CooldownManager;
use insurance::InsuranceCoordinator;
use policy::PolicyEngine;
use simulator::TransactionSimulator;

pub use cooldown::CooldownStatus;
pub use insurance::{InsuranceProvider, InsuranceQuote, InsuranceSelection};
pub use policy::{PolicyCheckResult, PolicyViolation, SafetyPolicy, ViolationSeverity};
pub use simulator::{ImpactPreview, MevRiskLevel, RouteHop, TransactionSimulation};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckRequest {
    pub wallet_address: String,
    pub input_amount: f64,
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub amount_usd: f64,
    pub slippage_bps: u64,
    pub price_impact_percent: f64,
    pub security_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckResult {
    pub allowed: bool,
    pub policy_result: PolicyCheckResult,
    pub cooldown_status: Option<CooldownStatus>,
    pub simulation: Option<TransactionSimulation>,
    pub impact_preview: Option<ImpactPreview>,
    pub insurance_required: bool,
    pub insurance_recommendation: Option<InsuranceQuote>,
    pub mev_suggestions: Vec<String>,
}

pub struct SafetyEngine {
    policy_engine: PolicyEngine,
    cooldown_manager: CooldownManager,
    simulator: TransactionSimulator,
    insurance_coordinator: InsuranceCoordinator,
    emergency_halt: bool,
}

impl SafetyEngine {
    pub fn new(policy: SafetyPolicy, cooldown_seconds: u64) -> Self {
        Self {
            policy_engine: PolicyEngine::new(policy.clone()),
            cooldown_manager: CooldownManager::new(cooldown_seconds),
            simulator: TransactionSimulator::default(),
            insurance_coordinator: InsuranceCoordinator::default(),
            emergency_halt: false,
        }
    }

    pub fn get_policy(&self) -> &SafetyPolicy {
        self.policy_engine.get_policy()
    }

    pub fn update_policy(&mut self, policy: SafetyPolicy) {
        if policy.cooldown_enabled {
            self.cooldown_manager
                .set_cooldown_duration(policy.cooldown_seconds);
        }
        self.policy_engine.update_policy(policy);
    }

    pub async fn check_trade_safety(
        &mut self,
        request: SafetyCheckRequest,
    ) -> Result<SafetyCheckResult, String> {
        if self.emergency_halt {
            let policy_result = PolicyCheckResult::new_blocked(vec![PolicyViolation {
                rule: "emergency_halt".to_string(),
                message: "Emergency trading halt is active".to_string(),
                severity: ViolationSeverity::Critical,
                can_override: false,
            }]);

            return Ok(SafetyCheckResult {
                allowed: false,
                policy_result,
                cooldown_status: None,
                simulation: None,
                impact_preview: None,
                insurance_required: false,
                insurance_recommendation: None,
                mev_suggestions: Vec::new(),
            });
        }

        // Check policy violations
        let policy_result = self.policy_engine.check_trade_policy(
            &request.wallet_address,
            request.amount_usd,
            request.price_impact_percent,
            request.slippage_bps as f64 / 100.0,
            request.security_score,
        );

        // Check cooldown
        let cooldown_status = if self.get_policy().cooldown_enabled {
            self.cooldown_manager
                .get_remaining_cooldown(&request.wallet_address)
        } else {
            None
        };

        let on_cooldown = cooldown_status.is_some();

        // Run transaction simulation if required
        let simulation = if self.get_policy().require_simulation {
            Some(
                self.simulator
                    .simulate_transaction(
                        request.input_amount,
                        &request.input_mint,
                        &request.output_mint,
                        request.slippage_bps,
                    )
                    .await?,
            )
        } else {
            None
        };

        // Generate impact preview
        let impact_preview = Some(
            self.simulator
                .preview_impact(
                    request.input_amount,
                    request.input_symbol.clone(),
                    request.output_symbol.clone(),
                    request.slippage_bps as f64 / 100.0,
                )
                .await?,
        );

        // Check if insurance is required or recommended
        let insurance_required = policy_result.requires_insurance;
        let insurance_recommendation = if insurance_required || request.amount_usd > 10000.0 {
            let mev_risk = simulation
                .as_ref()
                .map(|s| match s.mev_risk_level {
                    simulator::MevRiskLevel::Low => 0.2,
                    simulator::MevRiskLevel::Medium => 0.5,
                    simulator::MevRiskLevel::High => 0.8,
                    simulator::MevRiskLevel::Critical => 1.0,
                })
                .unwrap_or(0.3);

            self.insurance_coordinator.recommend_provider(
                request.amount_usd,
                request.price_impact_percent,
                mev_risk,
            )
        } else {
            None
        };

        // Generate MEV protection suggestions
        let mev_suggestions = self.simulator.suggest_mev_protection(request.amount_usd);

        let allowed = policy_result.allowed && !on_cooldown;

        Ok(SafetyCheckResult {
            allowed,
            policy_result,
            cooldown_status,
            simulation,
            impact_preview,
            insurance_required,
            insurance_recommendation,
            mev_suggestions,
        })
    }

    pub fn approve_trade(&mut self, wallet_address: &str) {
        if self.get_policy().cooldown_enabled {
            self.cooldown_manager.record_trade(wallet_address);
        }
        self.policy_engine
            .increment_daily_trade_count(wallet_address);
    }

    pub fn get_cooldown_status(&self, wallet_address: &str) -> Option<CooldownStatus> {
        self.cooldown_manager.get_remaining_cooldown(wallet_address)
    }

    pub fn reset_daily_limits(&mut self) {
        self.policy_engine.reset_daily_counts();
    }

    pub fn set_emergency_halt(&mut self, enabled: bool) {
        self.emergency_halt = enabled;
    }

    pub fn is_emergency_halt(&self) -> bool {
        self.emergency_halt
    }

    pub fn get_insurance_quote(
        &mut self,
        provider_id: &str,
        trade_amount_usd: f64,
        price_impact_percent: f64,
        mev_risk_level: f64,
    ) -> Result<InsuranceQuote, String> {
        self.insurance_coordinator.request_quote(
            provider_id,
            trade_amount_usd,
            price_impact_percent,
            mev_risk_level,
        )
    }

    pub fn select_insurance(
        &mut self,
        provider_id: &str,
        trade_amount_usd: f64,
        price_impact_percent: f64,
        mev_risk_level: f64,
    ) -> Result<InsuranceSelection, String> {
        self.insurance_coordinator.select_insurance(
            provider_id,
            trade_amount_usd,
            price_impact_percent,
            mev_risk_level,
        )
    }

    pub fn list_insurance_providers(&self) -> Vec<InsuranceProvider> {
        self.insurance_coordinator
            .list_providers()
            .into_iter()
            .cloned()
            .collect()
    }
}

pub type SharedSafetyEngine = Arc<RwLock<SafetyEngine>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_safety_check_allowed() {
        let policy = SafetyPolicy::default();
        let mut engine = SafetyEngine::new(policy, 30);

        let request = SafetyCheckRequest {
            wallet_address: "wallet1".to_string(),
            input_amount: 100.0,
            input_mint: "SOL".to_string(),
            output_mint: "USDC".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "USDC".to_string(),
            amount_usd: 5000.0,
            slippage_bps: 50,
            price_impact_percent: 1.5,
            security_score: Some(85.0),
        };

        let result = engine.check_trade_safety(request).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.allowed);
    }

    #[tokio::test]
    async fn test_safety_check_with_cooldown() {
        let policy = SafetyPolicy::default();
        let mut engine = SafetyEngine::new(policy, 30);

        let request = SafetyCheckRequest {
            wallet_address: "wallet1".to_string(),
            input_amount: 100.0,
            input_mint: "SOL".to_string(),
            output_mint: "USDC".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "USDC".to_string(),
            amount_usd: 5000.0,
            slippage_bps: 50,
            price_impact_percent: 1.5,
            security_score: Some(85.0),
        };

        // First trade allowed
        engine.approve_trade("wallet1");

        // Second trade should be blocked by cooldown
        let result = engine.check_trade_safety(request).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(!check.allowed);
        assert!(check.cooldown_status.is_some());
    }

    #[tokio::test]
    async fn test_safety_check_high_risk() {
        let policy = SafetyPolicy::default();
        let mut engine = SafetyEngine::new(policy, 30);

        let request = SafetyCheckRequest {
            wallet_address: "wallet1".to_string(),
            input_amount: 100.0,
            input_mint: "SOL".to_string(),
            output_mint: "USDC".to_string(),
            input_symbol: "SOL".to_string(),
            output_symbol: "USDC".to_string(),
            amount_usd: 5000.0,
            slippage_bps: 50,
            price_impact_percent: 1.5,
            security_score: Some(30.0),
        };

        let result = engine.check_trade_safety(request).await;
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(!check.allowed);
        assert!(!check.policy_result.violations.is_empty());
    }
}
