use super::types::{CapitalGain, TaxJurisdiction, TaxProjection};
use crate::portfolio::TaxLot;
use chrono::{DateTime, Utc};

pub struct TaxCalculator {
    jurisdiction: TaxJurisdiction,
}

impl TaxCalculator {
    pub fn new(jurisdiction: TaxJurisdiction) -> Self {
        Self { jurisdiction }
    }

    pub fn calculate_capital_gain(
        &self,
        lot: &TaxLot,
        sale_price: f64,
        sale_amount: f64,
        sale_date: DateTime<Utc>,
    ) -> Result<CapitalGain, String> {
        let acquired_date = lot
            .acquired_at
            .parse::<DateTime<Utc>>()
            .map_err(|e| format!("Invalid acquired date: {}", e))?;

        let holding_period_days = (sale_date - acquired_date).num_days();
        let is_long_term = holding_period_days >= self.jurisdiction.holding_period_days;

        let unit_cost_basis = lot.cost_basis / lot.amount;
        let cost_basis = unit_cost_basis * sale_amount;
        let proceeds = sale_price * sale_amount;
        let gain_loss = proceeds - cost_basis;

        let tax_rate = if is_long_term {
            self.jurisdiction.long_term_rate
        } else {
            self.jurisdiction.short_term_rate
        };

        let tax_owed = if gain_loss > 0.0 {
            gain_loss * tax_rate
        } else {
            0.0
        };

        Ok(CapitalGain {
            asset: lot.symbol.clone(),
            mint_address: lot.mint.clone(),
            amount: sale_amount,
            cost_basis,
            proceeds,
            gain_loss,
            is_long_term,
            acquired_date,
            disposed_date: sale_date,
            holding_period_days,
            tax_rate,
            tax_owed,
        })
    }

    pub fn calculate_projection(
        &self,
        realized_gains: Vec<CapitalGain>,
        unrealized_positions: Vec<(f64, f64)>, // (unrealized gain, unrealized loss)
        carryforward_losses: f64,
        tax_year: i32,
    ) -> TaxProjection {
        let mut total_short_term_gains = 0.0;
        let mut total_long_term_gains = 0.0;
        let mut total_short_term_losses = 0.0;
        let mut total_long_term_losses = 0.0;

        for gain in realized_gains {
            if gain.is_long_term {
                if gain.gain_loss > 0.0 {
                    total_long_term_gains += gain.gain_loss;
                } else {
                    total_long_term_losses += gain.gain_loss.abs();
                }
            } else {
                if gain.gain_loss > 0.0 {
                    total_short_term_gains += gain.gain_loss;
                } else {
                    total_short_term_losses += gain.gain_loss.abs();
                }
            }
        }

        let net_short_term = total_short_term_gains - total_short_term_losses;
        let net_long_term = total_long_term_gains - total_long_term_losses;

        let mut total_net = net_short_term + net_long_term - carryforward_losses;

        // Apply capital loss limit if applicable
        if let Some(loss_limit) = self.jurisdiction.capital_loss_limit {
            if total_net < 0.0 && total_net.abs() > loss_limit {
                total_net = -loss_limit;
            }
        }

        let total_tax_owed = if total_net > 0.0 {
            let short_term_tax = if net_short_term > 0.0 {
                net_short_term * self.jurisdiction.short_term_rate
            } else {
                0.0
            };

            let long_term_tax = if net_long_term > 0.0 {
                net_long_term * self.jurisdiction.long_term_rate
            } else {
                0.0
            };

            short_term_tax + long_term_tax
        } else {
            0.0
        };

        let total_gains = total_short_term_gains + total_long_term_gains;
        let effective_tax_rate = if total_gains > 0.0 {
            total_tax_owed / total_gains
        } else {
            0.0
        };

        let mut total_unrealized_gains = 0.0;
        let mut total_unrealized_losses = 0.0;

        for (gain, loss) in &unrealized_positions {
            total_unrealized_gains += gain;
            total_unrealized_losses += loss;
        }

        let potential_savings = self.calculate_harvesting_savings(total_unrealized_losses);

        TaxProjection {
            tax_year,
            jurisdiction: self.jurisdiction.name.clone(),
            total_short_term_gains,
            total_long_term_gains,
            total_short_term_losses,
            total_long_term_losses,
            net_short_term,
            net_long_term,
            total_tax_owed,
            effective_tax_rate,
            potential_savings_from_harvesting: potential_savings,
            unrealized_gains: total_unrealized_gains,
            unrealized_losses: total_unrealized_losses,
            carryforward_losses,
            generated_at: Utc::now(),
        }
    }

    pub fn calculate_harvesting_savings(&self, unrealized_losses: f64) -> f64 {
        if unrealized_losses <= 0.0 {
            return 0.0;
        }

        // Apply capital loss limit if applicable
        let deductible_loss = if let Some(loss_limit) = self.jurisdiction.capital_loss_limit {
            unrealized_losses.min(loss_limit)
        } else {
            unrealized_losses
        };

        // Use the higher short-term rate for conservative estimate
        deductible_loss * self.jurisdiction.short_term_rate
    }

    pub fn estimate_quarterly_tax(&self, ytd_gains: f64) -> f64 {
        if ytd_gains <= 0.0 {
            return 0.0;
        }

        // Simplified quarterly estimate
        ytd_gains * self.jurisdiction.short_term_rate * 0.25
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_short_term_gain() {
        let jurisdiction = TaxJurisdiction::us_federal();
        let calculator = TaxCalculator::new(jurisdiction);

        let lot = TaxLot {
            id: "test-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 10000.0,
            price_per_unit: 100.0,
            acquired_at: (Utc::now() - Duration::days(180)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        };

        let sale_date = Utc::now();
        let result = calculator.calculate_capital_gain(&lot, 150.0, 100.0, sale_date);

        assert!(result.is_ok());
        let gain = result.unwrap();
        assert!(!gain.is_long_term);
        assert_eq!(gain.proceeds, 15000.0);
        assert_eq!(gain.cost_basis, 10000.0);
        assert_eq!(gain.gain_loss, 5000.0);
    }

    #[test]
    fn test_long_term_gain() {
        let jurisdiction = TaxJurisdiction::us_federal();
        let calculator = TaxCalculator::new(jurisdiction);

        let lot = TaxLot {
            id: "test-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 10000.0,
            price_per_unit: 100.0,
            acquired_at: (Utc::now() - Duration::days(400)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        };

        let sale_date = Utc::now();
        let result = calculator.calculate_capital_gain(&lot, 150.0, 100.0, sale_date);

        assert!(result.is_ok());
        let gain = result.unwrap();
        assert!(gain.is_long_term);
        assert_eq!(gain.tax_rate, 0.20);
    }
}
