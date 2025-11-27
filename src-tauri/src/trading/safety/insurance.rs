use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceProvider {
    pub id: String,
    pub name: String,
    pub coverage_limit_usd: f64,
    pub premium_rate_bps: f64,
    pub response_time_ms: u64,
    pub reliability_percent: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceQuote {
    pub provider_id: String,
    pub total_premium_usd: f64,
    pub coverage_amount_usd: f64,
    pub coverage_percentage: f64,
    pub estimated_slippage_reimbursement: f64,
    pub mev_protection_included: bool,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceSelection {
    pub provider_id: String,
    pub premium_usd: f64,
    pub coverage_usd: f64,
    pub includes_mev_protection: bool,
}

pub struct InsuranceCoordinator {
    providers: HashMap<String, InsuranceProvider>,
    quote_cache: HashMap<String, InsuranceQuote>,
    cache_ttl: Duration,
}

impl Default for InsuranceCoordinator {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "sol_shield".to_string(),
            InsuranceProvider {
                id: "sol_shield".to_string(),
                name: "SolShield Mutual".to_string(),
                coverage_limit_usd: 250000.0,
                premium_rate_bps: 15.0,
                response_time_ms: 300,
                reliability_percent: 98.5,
                is_active: true,
            },
        );
        providers.insert(
            "perp_guard".to_string(),
            InsuranceProvider {
                id: "perp_guard".to_string(),
                name: "PerpGuard".to_string(),
                coverage_limit_usd: 500000.0,
                premium_rate_bps: 20.0,
                response_time_ms: 450,
                reliability_percent: 96.0,
                is_active: true,
            },
        );

        Self {
            providers,
            quote_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(60),
        }
    }
}

impl InsuranceCoordinator {
    pub fn with_providers(providers: Vec<InsuranceProvider>) -> Self {
        let provider_map = providers
            .into_iter()
            .map(|provider| (provider.id.clone(), provider))
            .collect();

        Self {
            providers: provider_map,
            quote_cache: HashMap::new(),
            cache_ttl: Duration::from_secs(60),
        }
    }

    pub fn list_providers(&self) -> Vec<&InsuranceProvider> {
        self.providers
            .values()
            .filter(|provider| provider.is_active)
            .collect()
    }

    pub fn get_provider(&self, id: &str) -> Option<&InsuranceProvider> {
        self.providers.get(id)
    }

    pub fn request_quote(
        &mut self,
        provider_id: &str,
        trade_amount_usd: f64,
        price_impact_percent: f64,
        mev_risk_level: f64,
    ) -> Result<InsuranceQuote, String> {
        let provider = self
            .providers
            .get(provider_id)
            .ok_or_else(|| format!("Insurance provider {} not found", provider_id))?;

        if !provider.is_active {
            return Err(format!("Insurance provider {} is not active", provider_id));
        }

        // Check cache first
        if let Some(quote) = self.quote_cache.get(provider_id) {
            if quote.expires_at > Utc::now() {
                return Ok(quote.clone());
            }
        }

        let coverage_percentage = if mev_risk_level > 0.7 {
            0.9
        } else if price_impact_percent > 5.0 {
            0.85
        } else {
            0.75
        };

        let coverage_amount_usd =
            (trade_amount_usd * coverage_percentage).min(provider.coverage_limit_usd);

        let premium_rate = provider.premium_rate_bps / 10000.0;
        let mut premium_multiplier = 1.0;

        // Adjust premium based on risk factors
        if mev_risk_level > 0.5 {
            premium_multiplier += 0.35;
        }
        if price_impact_percent > 3.0 {
            premium_multiplier += 0.2;
        }

        let total_premium_usd = trade_amount_usd * premium_rate * premium_multiplier;
        let cache_ttl = ChronoDuration::from_std(self.cache_ttl)
            .unwrap_or_else(|_| ChronoDuration::seconds(self.cache_ttl.as_secs() as i64));

        let quote = InsuranceQuote {
            provider_id: provider_id.to_string(),
            total_premium_usd,
            coverage_amount_usd,
            coverage_percentage,
            estimated_slippage_reimbursement: coverage_amount_usd * 0.6,
            mev_protection_included: mev_risk_level > 0.5,
            expires_at: Utc::now() + cache_ttl,
        };

        self.quote_cache
            .insert(provider_id.to_string(), quote.clone());
        Ok(quote)
    }

    pub fn recommend_provider(
        &mut self,
        trade_amount_usd: f64,
        price_impact_percent: f64,
        mev_risk_level: f64,
    ) -> Option<InsuranceQuote> {
        // Collect provider IDs first to avoid borrowing conflict
        let provider_ids: Vec<String> = self.list_providers()
            .into_iter()
            .map(|p| p.id.clone())
            .collect();

        provider_ids
            .into_iter()
            .filter_map(|provider_id| {
                self.request_quote(
                    &provider_id,
                    trade_amount_usd,
                    price_impact_percent,
                    mev_risk_level,
                )
                .ok()
            })
            .min_by(|a, b| {
                a.total_premium_usd
                    .partial_cmp(&b.total_premium_usd)
                    .unwrap()
            })
    }

    pub fn select_insurance(
        &mut self,
        provider_id: &str,
        trade_amount_usd: f64,
        price_impact_percent: f64,
        mev_risk_level: f64,
    ) -> Result<InsuranceSelection, String> {
        let quote = self.request_quote(
            provider_id,
            trade_amount_usd,
            price_impact_percent,
            mev_risk_level,
        )?;

        Ok(InsuranceSelection {
            provider_id: provider_id.to_string(),
            premium_usd: quote.total_premium_usd,
            coverage_usd: quote.coverage_amount_usd,
            includes_mev_protection: quote.mev_protection_included,
        })
    }

    pub fn set_cache_ttl(&mut self, ttl_seconds: u64) {
        self.cache_ttl = Duration::from_secs(ttl_seconds);
    }

    pub fn invalidate_quotes(&mut self) {
        self.quote_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_listing() {
        let coordinator = InsuranceCoordinator::default();
        let providers = coordinator.list_providers();
        assert!(!providers.is_empty());
        assert!(providers.iter().any(|p| p.id == "sol_shield"));
    }

    #[test]
    fn test_request_quote() {
        let mut coordinator = InsuranceCoordinator::default();
        let quote = coordinator
            .request_quote("sol_shield", 100000.0, 2.5, 0.6)
            .unwrap();
        assert!(quote.total_premium_usd > 0.0);
        assert!(quote.coverage_amount_usd > 0.0);
    }

    #[test]
    fn test_recommend_provider() {
        let mut coordinator = InsuranceCoordinator::default();
        let recommendation = coordinator.recommend_provider(150000.0, 4.0, 0.4);
        assert!(recommendation.is_some());
        let quote = recommendation.unwrap();
        assert!(quote.coverage_amount_usd > 0.0);
    }

    #[test]
    fn test_select_insurance() {
        let mut coordinator = InsuranceCoordinator::default();
        let selection = coordinator
            .select_insurance("sol_shield", 50000.0, 3.0, 0.3)
            .unwrap();
        assert_eq!(selection.provider_id, "sol_shield");
        assert!(selection.coverage_usd > 0.0);
    }
}
