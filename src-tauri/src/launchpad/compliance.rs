use super::types::*;
use crate::errors::AppError;
use crate::security::audit::{
    perform_audit, AuditMetadata, AuditResult, Finding, RiskLevel, Severity,
};
use chrono::Utc;

pub struct ComplianceChecker;

impl ComplianceChecker {
    pub async fn check_launch_safety(
        config: &TokenLaunchConfig,
    ) -> Result<LaunchSafetyCheck, AppError> {
        let mut checks = Vec::new();

        checks.push(Self::check_token_supply(config));
        checks.push(Self::check_authorities(config));
        checks.push(Self::check_metadata(config));
        checks.push(Self::check_socials(config));

        let passed = checks.iter().all(|c| c.passed);

        let risk_score = Self::calculate_risk_score(&checks);
        let risk_level = if risk_score >= 80 {
            "low"
        } else if risk_score >= 60 {
            "medium"
        } else if risk_score >= 40 {
            "high"
        } else {
            "critical"
        };

        Ok(LaunchSafetyCheck {
            passed,
            security_score: risk_score,
            risk_level: risk_level.to_string(),
            checks,
            timestamp: Utc::now(),
        })
    }

    pub async fn audit_token_contract(
        token_mint: &str,
        metadata: TokenMetadata,
    ) -> Result<AuditResult, AppError> {
        let audit_metadata = AuditMetadata {
            is_mintable: false,
            has_freeze_authority: false,
            is_mutable: false,
            has_blacklist: false,
            is_honeypot: false,
            creator_address: None,
            total_supply: None,
            holder_count: None,
        };

        perform_audit(token_mint, audit_metadata)
            .await
            .map_err(|e| AppError::Generic(e))
    }

    pub fn check_vesting_compliance(
        vesting: &CreateVestingRequest,
    ) -> Result<SafetyCheckResult, AppError> {
        let mut issues = Vec::new();

        if vesting.vesting_duration_seconds < 86400 * 30 {
            issues.push("Vesting duration is less than 30 days".to_string());
        }

        if vesting.cliff_duration_seconds.is_none() {
            issues.push("No cliff period configured".to_string());
        }

        let passed = issues.is_empty();
        let severity = if passed { "info" } else { "medium" };

        Ok(SafetyCheckResult {
            check_name: "Vesting Schedule Compliance".to_string(),
            passed,
            severity: severity.to_string(),
            message: if passed {
                "Vesting schedule follows best practices".to_string()
            } else {
                format!("Issues found: {}", issues.join(", "))
            },
            recommendation: if passed {
                None
            } else {
                Some("Consider extending vesting period and adding cliff".to_string())
            },
        })
    }

    pub fn check_liquidity_lock_compliance(
        lock: &LockLiquidityRequest,
    ) -> Result<SafetyCheckResult, AppError> {
        let min_lock_duration = 86400 * 180; // 180 days
        let passed = lock.duration_seconds >= min_lock_duration;

        let severity = if passed { "info" } else { "high" };

        Ok(SafetyCheckResult {
            check_name: "Liquidity Lock Compliance".to_string(),
            passed,
            severity: severity.to_string(),
            message: if passed {
                "Liquidity lock duration meets minimum requirements".to_string()
            } else {
                format!(
                    "Lock duration ({} days) is below recommended minimum (180 days)",
                    lock.duration_seconds / 86400
                )
            },
            recommendation: if passed {
                None
            } else {
                Some("Lock liquidity for at least 180 days to build trust".to_string())
            },
        })
    }

    fn check_token_supply(config: &TokenLaunchConfig) -> SafetyCheckResult {
        let max_safe_supply = 1_000_000_000_000; // 1 trillion
        let passed = config.total_supply <= max_safe_supply;

        SafetyCheckResult {
            check_name: "Token Supply".to_string(),
            passed,
            severity: if passed { "info" } else { "medium" }.to_string(),
            message: if passed {
                "Token supply is within safe limits".to_string()
            } else {
                "Token supply exceeds recommended maximum".to_string()
            },
            recommendation: if passed {
                None
            } else {
                Some("Consider reducing total supply to avoid inflation concerns".to_string())
            },
        }
    }

    fn check_authorities(config: &TokenLaunchConfig) -> SafetyCheckResult {
        let has_authorities = config.mint_authority_enabled || config.freeze_authority_enabled;

        SafetyCheckResult {
            check_name: "Token Authorities".to_string(),
            passed: !has_authorities,
            severity: if has_authorities { "high" } else { "info" }.to_string(),
            message: if has_authorities {
                "Mint or freeze authority is enabled".to_string()
            } else {
                "No potentially dangerous authorities enabled".to_string()
            },
            recommendation: if has_authorities {
                Some(
                    "Consider disabling mint/freeze authorities after launch for better trust"
                        .to_string(),
                )
            } else {
                None
            },
        }
    }

    fn check_metadata(config: &TokenLaunchConfig) -> SafetyCheckResult {
        let has_description = !config.description.is_empty();
        let has_image = config.image_url.is_some();

        let passed = has_description && has_image;

        SafetyCheckResult {
            check_name: "Token Metadata".to_string(),
            passed,
            severity: if passed { "info" } else { "low" }.to_string(),
            message: if passed {
                "Token has complete metadata".to_string()
            } else {
                "Token metadata is incomplete".to_string()
            },
            recommendation: if passed {
                None
            } else {
                Some("Add description and logo for better visibility".to_string())
            },
        }
    }

    fn check_socials(config: &TokenLaunchConfig) -> SafetyCheckResult {
        let social_count = [
            config.website.as_ref(),
            config.twitter.as_ref(),
            config.telegram.as_ref(),
            config.discord.as_ref(),
        ]
        .iter()
        .filter(|s| s.is_some())
        .count();

        let passed = social_count >= 2;

        SafetyCheckResult {
            check_name: "Social Presence".to_string(),
            passed,
            severity: if passed { "info" } else { "low" }.to_string(),
            message: if passed {
                format!("Token has {} social links", social_count)
            } else {
                "Limited social presence".to_string()
            },
            recommendation: if passed {
                None
            } else {
                Some("Add more social links to build community trust".to_string())
            },
        }
    }

    fn calculate_risk_score(checks: &[SafetyCheckResult]) -> u8 {
        let total_checks = checks.len();
        if total_checks == 0 {
            return 0;
        }

        let mut score = 100u8;

        for check in checks {
            if !check.passed {
                let deduction = match check.severity.as_str() {
                    "critical" => 30,
                    "high" => 20,
                    "medium" => 10,
                    "low" => 5,
                    _ => 0,
                };
                score = score.saturating_sub(deduction);
            }
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_risk_score() {
        let checks = vec![
            SafetyCheckResult {
                check_name: "Test".to_string(),
                passed: true,
                severity: "info".to_string(),
                message: "Passed".to_string(),
                recommendation: None,
            },
            SafetyCheckResult {
                check_name: "Test2".to_string(),
                passed: false,
                severity: "medium".to_string(),
                message: "Failed".to_string(),
                recommendation: Some("Fix this".to_string()),
            },
        ];

        let score = ComplianceChecker::calculate_risk_score(&checks);
        assert_eq!(score, 90);
    }
}
