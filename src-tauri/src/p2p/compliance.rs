use super::types::*;
use crate::security::reputation::{ReputationEngine, WalletReputation};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ComplianceChecker {
    min_reputation: f64,
    max_trade_amount: f64,
    require_verified: bool,
}

impl ComplianceChecker {
    pub fn new() -> Self {
        Self {
            min_reputation: 30.0,
            max_trade_amount: 10000.0,
            require_verified: false,
        }
    }

    pub fn with_min_reputation(mut self, min: f64) -> Self {
        self.min_reputation = min;
        self
    }

    pub fn with_max_trade_amount(mut self, max: f64) -> Self {
        self.max_trade_amount = max;
        self
    }

    pub fn require_verified(mut self, required: bool) -> Self {
        self.require_verified = required;
        self
    }

    pub async fn check_offer(
        &self,
        offer: &P2POffer,
        creator_reputation: Option<&WalletReputation>,
    ) -> Result<ComplianceCheck> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut checks_performed = Vec::new();
        let mut risk_level = "low".to_string();

        checks_performed.push("offer_amount".to_string());
        if offer.amount * offer.price > self.max_trade_amount {
            warnings.push(format!(
                "Trade amount ${:.2} exceeds recommended maximum ${:.2}",
                offer.amount * offer.price,
                self.max_trade_amount
            ));
            risk_level = "medium".to_string();
        }

        checks_performed.push("price_validity".to_string());
        if offer.price <= 0.0 {
            errors.push("Price must be greater than zero".to_string());
        }

        checks_performed.push("amount_validity".to_string());
        if offer.amount <= 0.0 {
            errors.push("Amount must be greater than zero".to_string());
        }

        if let Some(min) = offer.min_amount {
            if let Some(max) = offer.max_amount {
                if min > max {
                    errors.push("Minimum amount cannot exceed maximum amount".to_string());
                }
            }
        }

        checks_performed.push("time_limit".to_string());
        if offer.time_limit < 5 {
            warnings.push("Time limit is very short (less than 5 minutes)".to_string());
        } else if offer.time_limit > 1440 {
            warnings.push("Time limit is very long (more than 24 hours)".to_string());
        }

        checks_performed.push("creator_reputation".to_string());
        if let Some(reputation) = creator_reputation {
            if reputation.trust_score < self.min_reputation {
                warnings.push(format!(
                    "Creator has low reputation score: {:.1}",
                    reputation.trust_score
                ));
                risk_level = "high".to_string();
            }

            if reputation.is_blacklisted {
                errors.push(format!(
                    "Creator is blacklisted: {}",
                    reputation
                        .blacklist_reason
                        .as_ref()
                        .unwrap_or(&"Unknown reason".to_string())
                ));
                risk_level = "critical".to_string();
            }

            if reputation.transaction_count < 5 {
                warnings.push("Creator has limited trading history".to_string());
                if risk_level == "low" {
                    risk_level = "medium".to_string();
                }
            }
        } else {
            warnings.push("Creator reputation could not be verified".to_string());
            risk_level = "medium".to_string();
        }

        checks_performed.push("payment_methods".to_string());
        if offer.payment_methods.is_empty() {
            warnings.push("No payment methods specified".to_string());
        }

        let passed = errors.is_empty();

        Ok(ComplianceCheck {
            passed,
            warnings,
            errors,
            risk_level,
            checks_performed,
        })
    }

    pub async fn check_escrow(
        &self,
        escrow: &Escrow,
        buyer_reputation: Option<&WalletReputation>,
        seller_reputation: Option<&WalletReputation>,
    ) -> Result<ComplianceCheck> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut checks_performed = Vec::new();
        let mut risk_level = "low".to_string();

        checks_performed.push("escrow_amount".to_string());
        if escrow.fiat_amount > self.max_trade_amount {
            warnings.push(format!(
                "Escrow amount ${:.2} exceeds recommended maximum ${:.2}",
                escrow.fiat_amount, self.max_trade_amount
            ));
            risk_level = "medium".to_string();
        }

        checks_performed.push("buyer_reputation".to_string());
        if let Some(reputation) = buyer_reputation {
            if reputation.trust_score < self.min_reputation {
                warnings.push(format!(
                    "Buyer has low reputation score: {:.1}",
                    reputation.trust_score
                ));
                risk_level = "high".to_string();
            }

            if reputation.is_blacklisted {
                errors.push(format!(
                    "Buyer is blacklisted: {}",
                    reputation
                        .blacklist_reason
                        .as_ref()
                        .unwrap_or(&"Unknown reason".to_string())
                ));
                risk_level = "critical".to_string();
            }
        } else {
            warnings.push("Buyer reputation could not be verified".to_string());
            if risk_level == "low" {
                risk_level = "medium".to_string();
            }
        }

        checks_performed.push("seller_reputation".to_string());
        if let Some(reputation) = seller_reputation {
            if reputation.trust_score < self.min_reputation {
                warnings.push(format!(
                    "Seller has low reputation score: {:.1}",
                    reputation.trust_score
                ));
                risk_level = "high".to_string();
            }

            if reputation.is_blacklisted {
                errors.push(format!(
                    "Seller is blacklisted: {}",
                    reputation
                        .blacklist_reason
                        .as_ref()
                        .unwrap_or(&"Unknown reason".to_string())
                ));
                risk_level = "critical".to_string();
            }
        } else {
            warnings.push("Seller reputation could not be verified".to_string());
            if risk_level == "low" {
                risk_level = "medium".to_string();
            }
        }

        checks_performed.push("timeout".to_string());
        if escrow.timeout_at < chrono::Utc::now() {
            errors.push("Escrow has already timed out".to_string());
        }

        checks_performed.push("state_validity".to_string());
        if escrow.state == EscrowState::Disputed && escrow.arbitrators.is_empty() {
            warnings.push("Disputed escrow has no assigned arbitrators".to_string());
        }

        let passed = errors.is_empty();

        Ok(ComplianceCheck {
            passed,
            warnings,
            errors,
            risk_level,
            checks_performed,
        })
    }

    pub fn check_trade_limit(
        &self,
        user_24h_volume: f64,
        new_trade_amount: f64,
    ) -> ComplianceCheck {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let checks_performed = vec!["daily_limit".to_string()];
        let mut risk_level = "low".to_string();

        let daily_limit = self.max_trade_amount * 5.0;
        let projected_volume = user_24h_volume + new_trade_amount;

        if projected_volume > daily_limit {
            errors.push(format!(
                "Trade would exceed 24-hour limit of ${:.2} (current: ${:.2}, new: ${:.2})",
                daily_limit, user_24h_volume, new_trade_amount
            ));
            risk_level = "high".to_string();
        } else if projected_volume > daily_limit * 0.8 {
            warnings.push(format!(
                "Approaching 24-hour limit: ${:.2} / ${:.2}",
                projected_volume, daily_limit
            ));
            risk_level = "medium".to_string();
        }

        let passed = errors.is_empty();

        ComplianceCheck {
            passed,
            warnings,
            errors,
            risk_level,
            checks_performed,
        }
    }

    pub fn generate_safety_warnings(&self, risk_level: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        warnings.push("⚠️ Only trade with users you trust or have verified reputation".to_string());
        warnings.push("⚠️ Never share your private keys or seed phrases".to_string());
        warnings.push("⚠️ Verify payment details carefully before releasing escrow".to_string());
        warnings
            .push("⚠️ Use the built-in chat to maintain a record of all communications".to_string());

        match risk_level {
            "high" | "critical" => {
                warnings.push("🚨 HIGH RISK: This trade has significant risk factors".to_string());
                warnings.push(
                    "🚨 Consider cancelling or requesting additional verification".to_string(),
                );
                warnings.push("🚨 Document all communications and save evidence".to_string());
            }
            "medium" => {
                warnings.push("⚠️ MODERATE RISK: Exercise caution with this trade".to_string());
                warnings.push("⚠️ Verify counterparty information before proceeding".to_string());
            }
            _ => {}
        }

        warnings
    }
}

impl Default for ComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_offer() -> P2POffer {
        P2POffer {
            id: "test_offer".to_string(),
            creator: "creator_address".to_string(),
            offer_type: OfferType::Sell,
            token_address: "token_address".to_string(),
            token_symbol: "SOL".to_string(),
            amount: 100.0,
            price: 50.0,
            fiat_currency: "USD".to_string(),
            payment_methods: vec!["Bank Transfer".to_string()],
            min_amount: Some(10.0),
            max_amount: Some(100.0),
            terms: Some("Test terms".to_string()),
            time_limit: 30,
            created_at: Utc::now(),
            expires_at: None,
            is_active: true,
            completed_trades: 0,
            reputation_required: Some(50.0),
        }
    }

    #[tokio::test]
    async fn test_valid_offer() {
        let checker = ComplianceChecker::new();
        let offer = create_test_offer();
        let result = checker.check_offer(&offer, None).await.unwrap();

        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_invalid_price() {
        let checker = ComplianceChecker::new();
        let mut offer = create_test_offer();
        offer.price = -10.0;

        let result = checker.check_offer(&offer, None).await.unwrap();
        assert!(!result.passed);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_high_amount_warning() {
        let checker = ComplianceChecker::new().with_max_trade_amount(100.0);
        let mut offer = create_test_offer();
        offer.amount = 100.0;
        offer.price = 10.0;

        let result = checker.check_offer(&offer, None).await.unwrap();
        assert!(result.passed);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_trade_limit() {
        let checker = ComplianceChecker::new();
        let result = checker.check_trade_limit(5000.0, 1000.0);
        assert!(result.passed);

        let result = checker.check_trade_limit(50000.0, 10000.0);
        assert!(!result.passed);
    }

    #[test]
    fn test_safety_warnings() {
        let checker = ComplianceChecker::new();

        let warnings_low = checker.generate_safety_warnings("low");
        let warnings_high = checker.generate_safety_warnings("high");

        assert!(warnings_high.len() > warnings_low.len());
    }
}
