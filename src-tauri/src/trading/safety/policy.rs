use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyPolicy {
    pub enabled: bool,
    pub cooldown_enabled: bool,
    pub cooldown_seconds: u64,
    pub max_trade_amount_usd: Option<f64>,
    pub max_daily_trades: Option<u32>,
    pub require_simulation: bool,
    pub block_high_risk: bool,
    pub high_risk_threshold: f64,
    pub require_insurance_above_usd: Option<f64>,
    pub max_price_impact_percent: f64,
    pub max_slippage_percent: f64,
}

impl Default for SafetyPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            cooldown_enabled: true,
            cooldown_seconds: 30,
            max_trade_amount_usd: Some(10000.0),
            max_daily_trades: Some(100),
            require_simulation: true,
            block_high_risk: true,
            high_risk_threshold: 40.0,
            require_insurance_above_usd: Some(50000.0),
            max_price_impact_percent: 10.0,
            max_slippage_percent: 5.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SafetyCheck {
    MaxNotionalValue(f64),
    MaxOrderSize(f64),
}

impl std::fmt::Display for SafetyCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyCheck::MaxNotionalValue(val) => write!(f, "Max notional value: ${}", val),
            SafetyCheck::MaxOrderSize(val) => write!(f, "Max order size: ${}", val),
        }
    }
}

impl SafetyPolicy {
    pub fn check_mobile_quick_trade(
        &self,
        check: SafetyCheck,
        amount: f64,
    ) -> Result<(), PolicyViolation> {
        match check {
            SafetyCheck::MaxNotionalValue(max) => {
                if amount > max {
                    return Err(PolicyViolation {
                        rule: "max_notional_value".to_string(),
                        message: format!("Trade amount ${} exceeds maximum ${}", amount, max),
                        severity: ViolationSeverity::Error,
                        can_override: false,
                    });
                }
            }
            SafetyCheck::MaxOrderSize(max) => {
                if amount > max {
                    return Err(PolicyViolation {
                        rule: "max_order_size".to_string(),
                        message: format!("Order size ${} exceeds maximum ${}", amount, max),
                        severity: ViolationSeverity::Error,
                        can_override: false,
                    });
                }
            }
        }
        Ok(())
    }

    pub fn mobile_quick_trade_rules(&self) -> Vec<SafetyCheck> {
        vec![
            SafetyCheck::MaxNotionalValue(50_000.0),
            SafetyCheck::MaxOrderSize(1_000.0),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule: String,
    pub message: String,
    pub severity: ViolationSeverity,
    pub can_override: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckResult {
    pub allowed: bool,
    pub violations: Vec<PolicyViolation>,
    pub warnings: Vec<String>,
    pub requires_insurance: bool,
}

impl PolicyCheckResult {
    pub fn new_allowed() -> Self {
        Self {
            allowed: true,
            violations: Vec::new(),
            warnings: Vec::new(),
            requires_insurance: false,
        }
    }

    pub fn new_blocked(violations: Vec<PolicyViolation>) -> Self {
        Self {
            allowed: false,
            violations,
            warnings: Vec::new(),
            requires_insurance: false,
        }
    }

    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }

    pub fn add_violation(&mut self, violation: PolicyViolation) {
        if violation.severity == ViolationSeverity::Error
            || violation.severity == ViolationSeverity::Critical
        {
            self.allowed = false;
        }
        self.violations.push(violation);
    }
}

pub struct PolicyEngine {
    policy: SafetyPolicy,
    daily_trade_counts: HashMap<String, u32>,
}

impl PolicyEngine {
    pub fn new(policy: SafetyPolicy) -> Self {
        Self {
            policy,
            daily_trade_counts: HashMap::new(),
        }
    }

    pub fn get_policy(&self) -> &SafetyPolicy {
        &self.policy
    }

    pub fn update_policy(&mut self, policy: SafetyPolicy) {
        self.policy = policy;
    }

    pub fn check_trade_policy(
        &mut self,
        wallet_address: &str,
        amount_usd: f64,
        price_impact: f64,
        slippage: f64,
        risk_score: Option<f64>,
    ) -> PolicyCheckResult {
        if !self.policy.enabled {
            return PolicyCheckResult::new_allowed();
        }

        let mut result = PolicyCheckResult::new_allowed();

        // Check trade amount limit
        if let Some(max_amount) = self.policy.max_trade_amount_usd {
            if amount_usd > max_amount {
                result.add_violation(PolicyViolation {
                    rule: "max_trade_amount".to_string(),
                    message: format!(
                        "Trade amount ${:.2} exceeds maximum allowed ${:.2}",
                        amount_usd, max_amount
                    ),
                    severity: ViolationSeverity::Error,
                    can_override: true,
                });
            }
        }

        // Check daily trade limit
        if let Some(max_daily) = self.policy.max_daily_trades {
            let count = self
                .daily_trade_counts
                .get(wallet_address)
                .copied()
                .unwrap_or(0);
            if count >= max_daily {
                result.add_violation(PolicyViolation {
                    rule: "max_daily_trades".to_string(),
                    message: format!(
                        "Daily trade limit reached: {} of {} trades",
                        count, max_daily
                    ),
                    severity: ViolationSeverity::Error,
                    can_override: false,
                });
            }
        }

        // Check price impact
        if price_impact > self.policy.max_price_impact_percent {
            result.add_violation(PolicyViolation {
                rule: "max_price_impact".to_string(),
                message: format!(
                    "Price impact {:.2}% exceeds maximum allowed {:.2}%",
                    price_impact, self.policy.max_price_impact_percent
                ),
                severity: if price_impact > self.policy.max_price_impact_percent * 1.5 {
                    ViolationSeverity::Critical
                } else {
                    ViolationSeverity::Warning
                },
                can_override: price_impact <= self.policy.max_price_impact_percent * 2.0,
            });
        }

        // Check slippage
        if slippage > self.policy.max_slippage_percent {
            result.add_violation(PolicyViolation {
                rule: "max_slippage".to_string(),
                message: format!(
                    "Slippage {:.2}% exceeds maximum allowed {:.2}%",
                    slippage, self.policy.max_slippage_percent
                ),
                severity: ViolationSeverity::Warning,
                can_override: true,
            });
        }

        // Check risk score
        if let Some(score) = risk_score {
            if self.policy.block_high_risk && score < self.policy.high_risk_threshold {
                result.add_violation(PolicyViolation {
                    rule: "high_risk_token".to_string(),
                    message: format!(
                        "Token security score {} is below threshold {}",
                        score, self.policy.high_risk_threshold
                    ),
                    severity: ViolationSeverity::Critical,
                    can_override: true,
                });
            }
        }

        // Check if insurance is required
        if let Some(threshold) = self.policy.require_insurance_above_usd {
            if amount_usd > threshold {
                result.requires_insurance = true;
                result.add_warning(format!(
                    "Insurance recommended for trades above ${:.2}",
                    threshold
                ));
            }
        }

        result
    }

    pub fn increment_daily_trade_count(&mut self, wallet_address: &str) {
        let count = self
            .daily_trade_counts
            .entry(wallet_address.to_string())
            .or_insert(0);
        *count += 1;
    }

    pub fn reset_daily_counts(&mut self) {
        self.daily_trade_counts.clear();
    }

    pub fn get_trade_count(&self, wallet_address: &str) -> u32 {
        self.daily_trade_counts
            .get(wallet_address)
            .copied()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = SafetyPolicy::default();
        assert!(policy.enabled);
        assert_eq!(policy.cooldown_seconds, 30);
        assert_eq!(policy.max_price_impact_percent, 10.0);
    }

    #[test]
    fn test_policy_check_allowed() {
        let mut engine = PolicyEngine::new(SafetyPolicy::default());
        let result = engine.check_trade_policy("wallet1", 1000.0, 1.0, 0.5, Some(80.0));
        assert!(result.allowed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_policy_check_amount_exceeded() {
        let mut engine = PolicyEngine::new(SafetyPolicy::default());
        let result = engine.check_trade_policy("wallet1", 15000.0, 1.0, 0.5, Some(80.0));
        assert!(!result.allowed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "max_trade_amount");
    }

    #[test]
    fn test_policy_check_high_risk() {
        let mut engine = PolicyEngine::new(SafetyPolicy::default());
        let result = engine.check_trade_policy("wallet1", 1000.0, 1.0, 0.5, Some(30.0));
        assert!(!result.allowed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "high_risk_token");
    }

    #[test]
    fn test_daily_trade_limit() {
        let mut policy = SafetyPolicy::default();
        policy.max_daily_trades = Some(3);
        let mut engine = PolicyEngine::new(policy);

        for _ in 0..3 {
            engine.increment_daily_trade_count("wallet1");
        }

        let result = engine.check_trade_policy("wallet1", 1000.0, 1.0, 0.5, Some(80.0));
        assert!(!result.allowed);
        assert_eq!(result.violations[0].rule, "max_daily_trades");
    }

    #[test]
    fn test_insurance_required() {
        let mut engine = PolicyEngine::new(SafetyPolicy::default());
        let result = engine.check_trade_policy("wallet1", 60000.0, 1.0, 0.5, Some(80.0));
        assert!(result.requires_insurance);
        assert!(!result.warnings.is_empty());
    }
}
