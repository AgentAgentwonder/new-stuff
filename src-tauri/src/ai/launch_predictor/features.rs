use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::EarlyWarning;

/// Core feature representation for launch predictor pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenFeatures {
    pub token_address: String,
    pub developer_reputation: f64,
    pub developer_launch_count: u32,
    pub developer_success_rate: f64,
    pub developer_category: String,
    pub contract_complexity: f64,
    pub proxy_pattern_detected: bool,
    pub upgradeable_contract: bool,
    pub liquidity_usd: f64,
    pub liquidity_ratio: f64,
    pub liquidity_change_24h: f64,
    pub initial_market_cap: f64,
    pub marketing_hype: f64,
    pub marketing_spend_usd: f64,
    pub social_followers_growth: f64,
    pub community_engagement: f64,
    pub influencer_sentiment: f64,
    pub security_audit_score: Option<f64>,
    pub dex_depth_score: f64,
    pub watchlist_interest: f64,
    pub retention_score: f64,
    pub launch_timestamp: DateTime<Utc>,
    #[serde(default)]
    pub actual_outcome: Option<f64>,
}

impl TokenFeatures {
    pub fn feature_vector(&self) -> HashMap<String, f64> {
        let mut features = HashMap::new();

        features.insert(
            "developer_reputation".to_string(),
            self.developer_reputation.clamp(0.0, 1.0),
        );
        features.insert(
            "developer_launch_experience".to_string(),
            (self.developer_launch_count as f64 + 1.0)
                .ln()
                .clamp(0.0, 5.0),
        );
        features.insert(
            "developer_success_rate".to_string(),
            self.developer_success_rate.clamp(0.0, 1.0),
        );
        features.insert(
            "contract_complexity".to_string(),
            self.contract_complexity.clamp(0.0, 1.0),
        );
        features.insert(
            "proxy_pattern_detected".to_string(),
            if self.proxy_pattern_detected {
                1.0
            } else {
                0.0
            },
        );
        features.insert(
            "upgradeable_contract".to_string(),
            if self.upgradeable_contract { 1.0 } else { 0.0 },
        );
        features.insert(
            "liquidity_depth".to_string(),
            (self.liquidity_usd + 1.0).ln().clamp(0.0, 15.0),
        );
        features.insert(
            "liquidity_ratio".to_string(),
            self.liquidity_ratio.clamp(0.0, 1.5),
        );
        features.insert(
            "liquidity_change".to_string(),
            (self.liquidity_change_24h / 100.0).clamp(-1.0, 1.0),
        );
        features.insert(
            "market_cap".to_string(),
            (self.initial_market_cap + 1.0).ln().clamp(0.0, 15.0),
        );
        features.insert(
            "marketing_hype".to_string(),
            self.marketing_hype.clamp(0.0, 1.0),
        );
        features.insert(
            "marketing_spend".to_string(),
            (self.marketing_spend_usd + 1.0).ln().clamp(0.0, 15.0),
        );
        features.insert(
            "social_followers_growth".to_string(),
            self.social_followers_growth.clamp(-1.0, 1.0),
        );
        features.insert(
            "community_engagement".to_string(),
            self.community_engagement.clamp(0.0, 1.0),
        );
        features.insert(
            "influencer_sentiment".to_string(),
            self.influencer_sentiment.clamp(-1.0, 1.0),
        );
        features.insert(
            "security_audit".to_string(),
            self.security_audit_score.unwrap_or(0.5).clamp(0.0, 1.0),
        );
        features.insert(
            "dex_depth".to_string(),
            self.dex_depth_score.clamp(0.0, 1.0),
        );
        features.insert(
            "watchlist_interest".to_string(),
            self.watchlist_interest.clamp(0.0, 1.0),
        );
        features.insert(
            "retention".to_string(),
            self.retention_score.clamp(0.0, 1.0),
        );
        features.insert(
            "launch_age_days".to_string(),
            ((Utc::now() - self.launch_timestamp).num_hours() as f64 / 24.0).clamp(0.0, 14.0),
        );
        features.insert(
            "developer_reputation_x_engagement".to_string(),
            (self.developer_reputation * self.community_engagement).clamp(0.0, 1.0),
        );
        features.insert(
            "marketing_hype_x_social".to_string(),
            (self.marketing_hype * (self.social_followers_growth + 1.0) / 2.0).clamp(0.0, 1.0),
        );

        features
    }

    pub fn early_warnings(&self) -> Vec<EarlyWarning> {
        let mut warnings = Vec::new();
        let now = Utc::now().to_rfc3339();

        if self.developer_reputation < 0.35 {
            warnings.push(EarlyWarning {
                warning_type: "lowDeveloperReputation".to_string(),
                severity: "High".to_string(),
                message: "Developer reputation is below healthy launch thresholds".to_string(),
                detected_at: now.clone(),
            });
        }

        if self.liquidity_usd < 75_000.0 {
            warnings.push(EarlyWarning {
                warning_type: "shallowLiquidity".to_string(),
                severity: if self.liquidity_usd < 25_000.0 {
                    "Critical"
                } else {
                    "High"
                }
                .to_string(),
                message: "Liquidity backing is low which increases slippage and exit risk"
                    .to_string(),
                detected_at: now.clone(),
            });
        }

        if self.proxy_pattern_detected {
            warnings.push(EarlyWarning {
                warning_type: "proxyPattern".to_string(),
                severity: "Medium".to_string(),
                message: "Upgradeable proxy patterns detected – monitor for governance changes"
                    .to_string(),
                detected_at: now.clone(),
            });
        }

        if self.marketing_hype > 0.75 && self.community_engagement < 0.45 {
            warnings.push(EarlyWarning {
                warning_type: "marketingMismatch".to_string(),
                severity: "Medium".to_string(),
                message:
                    "Marketing hype exceeds organic engagement suggesting manufactured interest"
                        .to_string(),
                detected_at: now.clone(),
            });
        }

        if self.security_audit_score.unwrap_or(0.3) < 0.4 {
            warnings.push(EarlyWarning {
                warning_type: "missingAudit".to_string(),
                severity: "High".to_string(),
                message: "Security audit coverage is weak for a new token launch".to_string(),
                detected_at: now.clone(),
            });
        }

        if self.social_followers_growth > 0.85 && self.watchlist_interest < 0.3 {
            warnings.push(EarlyWarning {
                warning_type: "botActivity".to_string(),
                severity: "Medium".to_string(),
                message: "Follower spikes are not translating into watchlist conversions"
                    .to_string(),
                detected_at: now.clone(),
            });
        }

        warnings
    }
}
