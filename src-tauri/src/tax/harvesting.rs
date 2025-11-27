use super::types::{HarvestingRecommendation, RecommendationPriority, TaxJurisdiction};
use super::wash_sale::WashSaleDetector;
use crate::portfolio::TaxLot;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct TaxLossHarvester {
    jurisdiction: TaxJurisdiction,
    wash_sale_detector: WashSaleDetector,
    threshold_usd: f64,
}

impl TaxLossHarvester {
    pub fn new(jurisdiction: TaxJurisdiction, threshold_usd: f64) -> Self {
        let wash_sale_detector = WashSaleDetector::new(jurisdiction.wash_sale_period_days);

        Self {
            jurisdiction,
            wash_sale_detector,
            threshold_usd,
        }
    }

    pub fn generate_recommendations(
        &self,
        open_lots: &[TaxLot],
        current_prices: &std::collections::HashMap<String, f64>,
        recent_purchases: &std::collections::HashMap<String, Vec<(DateTime<Utc>, f64)>>,
    ) -> Vec<HarvestingRecommendation> {
        let mut recommendations = Vec::new();

        for lot in open_lots {
            let current_price = match current_prices.get(&lot.symbol) {
                Some(price) => *price,
                None => continue,
            };

            let acquired_date = match lot.acquired_at.parse::<DateTime<Utc>>() {
                Ok(date) => date,
                Err(_) => continue,
            };

            let holding_period_days = (Utc::now() - acquired_date).num_days();
            let unit_cost_basis = lot.cost_basis / lot.amount;
            let current_value = lot.amount * current_price;
            let unrealized_loss = current_value - lot.cost_basis;

            if unrealized_loss >= 0.0 || unrealized_loss.abs() < self.threshold_usd {
                continue;
            }

            let tax_rate = if holding_period_days >= self.jurisdiction.holding_period_days {
                self.jurisdiction.long_term_rate
            } else {
                self.jurisdiction.short_term_rate
            };

            let tax_savings = unrealized_loss.abs() * tax_rate;

            let recent_asset_purchases = recent_purchases
                .get(&lot.symbol)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);

            let wash_sale_risk = self.wash_sale_detector.check_potential_wash_sale(
                &lot.symbol,
                Utc::now(),
                recent_asset_purchases,
            );

            let priority = self.calculate_priority(
                unrealized_loss.abs(),
                tax_savings,
                holding_period_days,
                wash_sale_risk,
            );

            let alternative_assets = self.wash_sale_detector.suggest_alternatives(&lot.symbol);

            let reason = self.generate_reason(
                unrealized_loss.abs(),
                tax_savings,
                holding_period_days,
                wash_sale_risk,
            );

            let expires_at = if holding_period_days >= self.jurisdiction.holding_period_days - 30 {
                Some(acquired_date + chrono::Duration::days(self.jurisdiction.holding_period_days))
            } else {
                None
            };

            recommendations.push(HarvestingRecommendation {
                id: Uuid::new_v4().to_string(),
                asset: lot.symbol.clone(),
                mint_address: lot.mint.clone(),
                lot_id: lot.id.clone(),
                current_price,
                cost_basis: unit_cost_basis,
                unrealized_loss: unrealized_loss.abs(),
                amount: lot.amount,
                holding_period_days,
                tax_savings,
                priority,
                reason,
                wash_sale_risk,
                alternative_assets,
                expires_at,
            });
        }

        recommendations.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then(b.tax_savings.partial_cmp(&a.tax_savings).unwrap())
        });

        recommendations
    }

    fn calculate_priority(
        &self,
        unrealized_loss: f64,
        tax_savings: f64,
        holding_period_days: i64,
        wash_sale_risk: bool,
    ) -> RecommendationPriority {
        let near_long_term_boundary = holding_period_days
            >= self.jurisdiction.holding_period_days - 30
            && holding_period_days < self.jurisdiction.holding_period_days;

        if tax_savings > 5000.0 && !wash_sale_risk {
            RecommendationPriority::Critical
        } else if (tax_savings > 2000.0 && !wash_sale_risk) || near_long_term_boundary {
            RecommendationPriority::High
        } else if tax_savings > 500.0 || wash_sale_risk {
            RecommendationPriority::Medium
        } else {
            RecommendationPriority::Low
        }
    }

    fn generate_reason(
        &self,
        unrealized_loss: f64,
        tax_savings: f64,
        holding_period_days: i64,
        wash_sale_risk: bool,
    ) -> String {
        let mut reasons = Vec::new();

        if tax_savings > 5000.0 {
            reasons.push(format!("High potential tax savings: ${:.2}", tax_savings));
        } else if tax_savings > 1000.0 {
            reasons.push(format!("Significant tax savings: ${:.2}", tax_savings));
        }

        if holding_period_days >= self.jurisdiction.holding_period_days {
            reasons.push("Long-term position - favorable treatment".to_string());
        } else if holding_period_days >= self.jurisdiction.holding_period_days - 30 {
            reasons.push("Approaching long-term status - consider timing".to_string());
        }

        if wash_sale_risk {
            reasons.push("Wash sale risk detected - consider alternative assets".to_string());
        }

        if unrealized_loss > 10000.0 {
            reasons.push(format!("Large unrealized loss: ${:.2}", unrealized_loss));
        }

        if reasons.is_empty() {
            format!(
                "Loss harvesting opportunity: ${:.2} tax savings",
                tax_savings
            )
        } else {
            reasons.join(". ")
        }
    }

    pub fn calculate_optimal_harvest_timing(
        &self,
        lot: &TaxLot,
        current_price: f64,
    ) -> Option<String> {
        let acquired_date = lot.acquired_at.parse::<DateTime<Utc>>().ok()?;
        let holding_period_days = (Utc::now() - acquired_date).num_days();

        let days_to_long_term = self.jurisdiction.holding_period_days - holding_period_days;

        if days_to_long_term > 0 && days_to_long_term <= 60 {
            Some(format!(
                "Consider waiting {} days for long-term capital gains treatment",
                days_to_long_term
            ))
        } else if holding_period_days < 30 {
            Some("Short holding period - consider waiting for better tax treatment".to_string())
        } else {
            None
        }
    }

    pub fn estimate_year_end_harvesting_potential(
        &self,
        open_lots: &[TaxLot],
        current_prices: &std::collections::HashMap<String, f64>,
        ytd_realized_gains: f64,
    ) -> f64 {
        let mut total_harvestable_losses = 0.0;

        for lot in open_lots {
            if let Some(current_price) = current_prices.get(&lot.symbol) {
                let current_value = lot.amount * current_price;
                let unrealized_loss = current_value - lot.cost_basis;

                if unrealized_loss < 0.0 && unrealized_loss.abs() >= self.threshold_usd {
                    total_harvestable_losses += unrealized_loss.abs();
                }
            }
        }

        // Calculate how much can offset gains
        let offsettable = total_harvestable_losses.min(ytd_realized_gains);
        let tax_rate = self.jurisdiction.short_term_rate;

        offsettable * tax_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::collections::HashMap;

    #[test]
    fn test_generate_recommendations() {
        let jurisdiction = TaxJurisdiction::us_federal();
        let harvester = TaxLossHarvester::new(jurisdiction, 100.0);

        let lot = TaxLot {
            id: "test-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 15000.0,
            price_per_unit: 150.0,
            acquired_at: (Utc::now() - Duration::days(100)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        };

        let mut current_prices = HashMap::new();
        current_prices.insert("SOL".to_string(), 130.0);

        let recent_purchases = HashMap::new();

        let recommendations =
            harvester.generate_recommendations(&[lot], &current_prices, &recent_purchases);

        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].unrealized_loss > 0.0);
        assert!(recommendations[0].tax_savings > 0.0);
    }

    #[test]
    fn test_no_recommendation_for_gains() {
        let jurisdiction = TaxJurisdiction::us_federal();
        let harvester = TaxLossHarvester::new(jurisdiction, 100.0);

        let lot = TaxLot {
            id: "test-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 10000.0,
            price_per_unit: 100.0,
            acquired_at: (Utc::now() - Duration::days(100)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        };

        let mut current_prices = HashMap::new();
        current_prices.insert("SOL".to_string(), 150.0);

        let recent_purchases = HashMap::new();

        let recommendations =
            harvester.generate_recommendations(&[lot], &current_prices, &recent_purchases);

        assert_eq!(recommendations.len(), 0);
    }
}
