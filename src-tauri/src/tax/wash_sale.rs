use super::types::{WashSaleSeverity, WashSaleWarning};
use crate::portfolio::TaxLot;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

pub struct WashSaleDetector {
    wash_sale_period_days: i64,
}

impl WashSaleDetector {
    pub fn new(wash_sale_period_days: i64) -> Self {
        Self {
            wash_sale_period_days,
        }
    }

    pub fn detect_wash_sales(
        &self,
        lots: &[TaxLot],
        recent_transactions: &[(String, DateTime<Utc>, f64, String)], // (asset, date, amount, type)
    ) -> Vec<WashSaleWarning> {
        let mut warnings = Vec::new();

        for lot in lots {
            if lot.disposed_at.is_none() {
                continue;
            }

            let disposed_at = match lot
                .disposed_at
                .as_ref()
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            {
                Some(dt) => dt,
                None => continue,
            };

            // Only check if it was a loss
            if let Some(realized_gain) = lot.realized_gain {
                if realized_gain >= 0.0 {
                    continue;
                }

                let wash_sale_start = disposed_at - Duration::days(self.wash_sale_period_days);
                let wash_sale_end = disposed_at + Duration::days(self.wash_sale_period_days);

                // Check for repurchases of the same asset within wash sale window
                for (asset, trans_date, amount, trans_type) in recent_transactions {
                    if asset != &lot.symbol {
                        continue;
                    }

                    if trans_type != "BUY" && trans_type != "RECEIVE" {
                        continue;
                    }

                    if *trans_date < disposed_at {
                        continue;
                    }

                    if *trans_date >= wash_sale_start && *trans_date <= wash_sale_end {
                        let loss_amount = realized_gain.abs();
                        let disposed_amount = lot.disposed_amount.unwrap_or(lot.amount);

                        let disallowed_loss = if *amount >= disposed_amount {
                            loss_amount
                        } else {
                            loss_amount * (*amount / disposed_amount)
                        };

                        let severity =
                            self.calculate_severity(disallowed_loss, *trans_date, disposed_at);

                        warnings.push(WashSaleWarning {
                            asset: lot.symbol.clone(),
                            mint_address: lot.mint.clone(),
                            sale_date: disposed_at,
                            sale_amount: disposed_amount,
                            loss_amount,
                            disallowed_loss,
                            repurchase_date: *trans_date,
                            repurchase_amount: *amount,
                            wash_sale_period_start: wash_sale_start,
                            wash_sale_period_end: wash_sale_end,
                            severity,
                            recommendation: self.generate_recommendation(
                                &lot.symbol,
                                disallowed_loss,
                                *trans_date,
                                wash_sale_end,
                            ),
                        });
                    }
                }
            }
        }

        warnings.sort_by(|a, b| {
            b.disallowed_loss
                .partial_cmp(&a.disallowed_loss)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        warnings
    }

    pub fn check_potential_wash_sale(
        &self,
        asset: &str,
        sale_date: DateTime<Utc>,
        recent_purchases: &[(DateTime<Utc>, f64)],
    ) -> bool {
        let wash_sale_start = sale_date - Duration::days(self.wash_sale_period_days);
        let wash_sale_end = sale_date + Duration::days(self.wash_sale_period_days);

        for (purchase_date, _amount) in recent_purchases {
            if *purchase_date >= wash_sale_start && *purchase_date <= wash_sale_end {
                return true;
            }
        }

        false
    }

    pub fn calculate_safe_date(&self, sale_date: DateTime<Utc>) -> DateTime<Utc> {
        sale_date + Duration::days(self.wash_sale_period_days + 1)
    }

    fn calculate_severity(
        &self,
        disallowed_loss: f64,
        repurchase_date: DateTime<Utc>,
        sale_date: DateTime<Utc>,
    ) -> WashSaleSeverity {
        let days_diff = (repurchase_date - sale_date).num_days();

        if disallowed_loss > 10000.0 || days_diff <= 1 {
            WashSaleSeverity::High
        } else if disallowed_loss > 1000.0 || days_diff <= 7 {
            WashSaleSeverity::Medium
        } else {
            WashSaleSeverity::Low
        }
    }

    fn generate_recommendation(
        &self,
        asset: &str,
        disallowed_loss: f64,
        repurchase_date: DateTime<Utc>,
        wash_sale_end: DateTime<Utc>,
    ) -> String {
        let days_until_safe = (wash_sale_end - repurchase_date).num_days();

        if disallowed_loss > 10000.0 {
            format!(
                "Critical: ${:.2} loss disallowed for {}. Consider selling and waiting {} days before repurchasing, or switch to a similar but different asset.",
                disallowed_loss, asset, days_until_safe
            )
        } else if disallowed_loss > 1000.0 {
            format!(
                "Warning: ${:.2} loss disallowed for {}. To avoid wash sales, wait {} days before repurchasing or consider alternative similar assets.",
                disallowed_loss, asset, days_until_safe
            )
        } else {
            format!(
                "Minor wash sale detected for {} (${:.2} disallowed). The cost basis will be adjusted to the new lot.",
                asset, disallowed_loss
            )
        }
    }

    pub fn suggest_alternatives(&self, asset: &str) -> Vec<String> {
        // Suggest similar but different assets to avoid wash sales
        let alternatives: HashMap<&str, Vec<&str>> = [
            ("SOL", vec!["mSOL", "stSOL", "jitoSOL"]),
            ("BTC", vec!["WBTC", "renBTC", "ETH"]),
            ("ETH", vec!["stETH", "rETH", "BTC"]),
            ("USDC", vec!["USDT", "DAI", "BUSD"]),
        ]
        .iter()
        .cloned()
        .collect();

        alternatives
            .get(asset)
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_detect_wash_sale() {
        let detector = WashSaleDetector::new(30);

        let lot = TaxLot {
            id: "test-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 15000.0,
            price_per_unit: 150.0,
            acquired_at: (Utc::now() - Duration::days(100)).to_rfc3339(),
            disposed_amount: Some(100.0),
            disposed_at: Some((Utc::now() - Duration::days(10)).to_rfc3339()),
            realized_gain: Some(-2000.0),
        };

        let transactions = vec![(
            "SOL".to_string(),
            Utc::now() - Duration::days(5),
            100.0,
            "BUY".to_string(),
        )];

        let warnings = detector.detect_wash_sales(&[lot], &transactions);

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].disallowed_loss, 2000.0);
    }

    #[test]
    fn test_no_wash_sale_after_period() {
        let detector = WashSaleDetector::new(30);

        let lot = TaxLot {
            id: "test-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 15000.0,
            price_per_unit: 150.0,
            acquired_at: (Utc::now() - Duration::days(100)).to_rfc3339(),
            disposed_amount: Some(100.0),
            disposed_at: Some((Utc::now() - Duration::days(40)).to_rfc3339()),
            realized_gain: Some(-2000.0),
        };

        let transactions = vec![(
            "SOL".to_string(),
            Utc::now() - Duration::days(5),
            100.0,
            "BUY".to_string(),
        )];

        let warnings = detector.detect_wash_sales(&[lot], &transactions);

        assert_eq!(warnings.len(), 0);
    }

    #[test]
    fn test_suggest_alternatives() {
        let detector = WashSaleDetector::new(30);

        let alternatives = detector.suggest_alternatives("SOL");
        assert!(!alternatives.is_empty());
        assert!(alternatives.contains(&"mSOL".to_string()));
    }
}
