use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Datelike, Duration, Utc};
use serde::Deserialize;
use tauri::State;

use super::types::{LotStrategy, TaxLossHarvestingSuggestion, TaxLot, TaxReport};

#[derive(Debug)]
pub struct TaxLotsState {
    lots: Vec<TaxLot>,
    strategy: LotStrategy,
}

impl Default for TaxLotsState {
    fn default() -> Self {
        let mut lots = Vec::new();

        lots.push(TaxLot {
            id: "lot-sol-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 150.0,
            cost_basis: 18000.0,
            price_per_unit: 120.0,
            acquired_at: (Utc::now() - Duration::days(410)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        });

        lots.push(TaxLot {
            id: "lot-sol-2".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 15000.0,
            price_per_unit: 150.0,
            acquired_at: (Utc::now() - Duration::days(240)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        });

        lots.push(TaxLot {
            id: "lot-sol-3".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 70.0,
            cost_basis: 10500.0,
            price_per_unit: 150.0,
            acquired_at: (Utc::now() - Duration::days(75)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        });

        lots.push(TaxLot {
            id: "lot-btc-1".to_string(),
            symbol: "BTC".to_string(),
            mint: "11111111111111111111111111111111".to_string(),
            amount: 1.8,
            cost_basis: 72000.0,
            price_per_unit: 40000.0,
            acquired_at: (Utc::now() - Duration::days(500)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        });

        lots.push(TaxLot {
            id: "lot-btc-2".to_string(),
            symbol: "BTC".to_string(),
            mint: "11111111111111111111111111111111".to_string(),
            amount: 0.8,
            cost_basis: 40000.0,
            price_per_unit: 50000.0,
            acquired_at: (Utc::now() - Duration::days(150)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        });

        lots.push(TaxLot {
            id: "lot-eth-1".to_string(),
            symbol: "ETH".to_string(),
            mint: "22222222222222222222222222222222".to_string(),
            amount: 25.0,
            cost_basis: 60000.0,
            price_per_unit: 2400.0,
            acquired_at: (Utc::now() - Duration::days(380)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        });

        lots.push(TaxLot {
            id: "lot-eth-2".to_string(),
            symbol: "ETH".to_string(),
            mint: "22222222222222222222222222222222".to_string(),
            amount: 10.0,
            cost_basis: 28000.0,
            price_per_unit: 2800.0,
            acquired_at: (Utc::now() - Duration::days(90)).to_rfc3339(),
            disposed_amount: None,
            disposed_at: None,
            realized_gain: None,
        });

        lots.push(TaxLot {
            id: "lot-sol-4-disposed".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 50.0,
            cost_basis: 6000.0,
            price_per_unit: 120.0,
            acquired_at: (Utc::now() - Duration::days(600)).to_rfc3339(),
            disposed_amount: Some(50.0),
            disposed_at: Some((Utc::now() - Duration::days(30)).to_rfc3339()),
            realized_gain: Some(2750.0),
        });

        Self {
            lots,
            strategy: LotStrategy::FIFO,
        }
    }
}

impl TaxLotsState {
    pub fn open_lots(&self) -> Vec<TaxLot> {
        self.lots
            .iter()
            .filter(|l| l.disposed_at.is_none())
            .cloned()
            .collect()
    }

    pub fn all_lots(&self) -> Vec<TaxLot> {
        self.lots.clone()
    }

    pub fn add_lot(&mut self, lot: TaxLot) {
        self.lots.push(lot);
    }

    fn set_strategy(&mut self, strategy: LotStrategy) {
        self.strategy = strategy;
    }

    fn dispose(&mut self, lot_id: &str, amount: f64, sale_price: f64) -> Result<TaxLot, String> {
        let lot = self
            .lots
            .iter_mut()
            .find(|l| l.id == lot_id)
            .ok_or_else(|| "Lot not found".to_string())?;

        if lot.disposed_at.is_some() {
            return Err("Lot already disposed".to_string());
        }

        if amount > lot.amount {
            return Err("Disposal amount exceeds lot amount".to_string());
        }

        let cost_per_unit = lot.cost_basis / lot.amount;
        let proceeds = amount * sale_price;
        let cost = amount * cost_per_unit;
        let realized = proceeds - cost;

        lot.disposed_amount = Some(amount);
        lot.disposed_at = Some(Utc::now().to_rfc3339());
        lot.realized_gain = Some(realized);

        Ok(lot.clone())
    }

    fn report(&self, tax_year: i32) -> TaxReport {
        let disposed_in_year: Vec<TaxLot> = self
            .all_lots()
            .into_iter()
            .filter(|lot| {
                lot.disposed_at
                    .as_ref()
                    .and_then(|s| parse_datetime(s).ok())
                    .map(|dt| dt.year() == tax_year)
                    .unwrap_or(false)
            })
            .collect();

        let mut total_gains = 0.0;
        let mut total_losses = 0.0;
        let mut short_term_gains = 0.0;
        let mut long_term_gains = 0.0;

        for lot in disposed_in_year.iter() {
            let realized = lot.realized_gain.unwrap_or(0.0);
            if realized > 0.0 {
                total_gains += realized;
            } else {
                total_losses += realized.abs();
            }

            let days = days_between(&lot.acquired_at, lot.disposed_at.as_deref());
            if is_long_term(days) {
                long_term_gains += realized;
            } else {
                short_term_gains += realized;
            }
        }

        let net = total_gains - total_losses;

        TaxReport {
            tax_year,
            lots: disposed_in_year,
            total_realized_gains: total_gains,
            total_realized_losses: total_losses,
            net_gain_loss: net,
            short_term_gains,
            long_term_gains,
            strategy: self.strategy.clone(),
            generated_at: Utc::now().to_rfc3339(),
        }
    }

    fn export(&self, tax_year: i32, format: &str) -> Result<String, String> {
        let disposed_in_year: Vec<TaxLot> = self
            .all_lots()
            .into_iter()
            .filter(|lot| {
                lot.disposed_at
                    .as_ref()
                    .and_then(|s| parse_datetime(s).ok())
                    .map(|dt| dt.year() == tax_year)
                    .unwrap_or(false)
            })
            .collect();

        match format {
            "turbotax" => {
                export_turbotax_format(&disposed_in_year, tax_year, self.strategy.clone())
            }
            "cointracker" => {
                export_cointracker_format(&disposed_in_year, tax_year, self.strategy.clone())
            }
            "csv" => export_csv_format(&disposed_in_year, tax_year, self.strategy.clone()),
            other => Err(format!("Unsupported export format: {}", other)),
        }
    }

    fn harvesting_suggestions(&self) -> Vec<TaxLossHarvestingSuggestion> {
        let open_lots = self.open_lots();
        let mut suggestions = Vec::new();

        let mock_prices: HashMap<&str, f64> = [("SOL", 175.4), ("BTC", 64000.0), ("ETH", 3400.0)]
            .iter()
            .cloned()
            .collect();

        for lot in open_lots.iter() {
            let current_price = mock_prices.get(lot.symbol.as_str()).cloned().unwrap_or(0.0);
            let current_value = lot.amount * current_price;
            let unrealized = current_value - lot.cost_basis;

            if unrealized >= 0.0 {
                continue;
            }

            let days_held = days_between(&lot.acquired_at, None);
            let tax_rate = if is_long_term(days_held) { 0.15 } else { 0.30 };
            let potential_savings = unrealized.abs() * tax_rate;

            suggestions.push(TaxLossHarvestingSuggestion {
                symbol: lot.symbol.clone(),
                mint: lot.mint.clone(),
                lot: lot.clone(),
                current_price,
                unrealized_loss: unrealized.abs(),
                potential_tax_savings: potential_savings,
                days_held,
            });
        }

        suggestions.sort_by(|a, b| {
            b.potential_tax_savings
                .partial_cmp(&a.potential_tax_savings)
                .unwrap()
        });

        suggestions
    }
}

pub type SharedTaxLotsState = Mutex<TaxLotsState>;

#[derive(Debug, Deserialize)]
pub struct DisposeLotInput {
    #[serde(rename = "lotId")]
    pub lot_id: String,
    pub amount: f64,
    #[serde(rename = "salePrice")]
    pub sale_price: f64,
}

#[tauri::command]
pub fn get_tax_lots(state: State<'_, SharedTaxLotsState>) -> Result<Vec<TaxLot>, String> {
    state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())
        .map(|guard| guard.all_lots())
}

#[tauri::command]
pub fn get_open_tax_lots(state: State<'_, SharedTaxLotsState>) -> Result<Vec<TaxLot>, String> {
    state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())
        .map(|guard| guard.open_lots())
}

#[tauri::command]
pub fn set_tax_lot_strategy(
    strategy: LotStrategy,
    state: State<'_, SharedTaxLotsState>,
) -> Result<LotStrategy, String> {
    state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())
        .map(|mut guard| {
            guard.set_strategy(strategy.clone());
            strategy
        })
}

#[tauri::command]
pub fn get_tax_lot_strategy(state: State<'_, SharedTaxLotsState>) -> Result<LotStrategy, String> {
    state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())
        .map(|guard| guard.strategy.clone())
}

#[tauri::command]
pub fn dispose_tax_lot(
    input: DisposeLotInput,
    state: State<'_, SharedTaxLotsState>,
) -> Result<TaxLot, String> {
    let mut guard = state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())?;
    guard.dispose(&input.lot_id, input.amount, input.sale_price)
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    s.parse::<DateTime<Utc>>()
}

fn days_between(acquired: &str, disposed_or_now: Option<&str>) -> i64 {
    let acquired_dt = parse_datetime(acquired).unwrap_or_else(|_| Utc::now());
    let disposed_dt = disposed_or_now
        .and_then(|d| parse_datetime(d).ok())
        .unwrap_or_else(Utc::now);
    (disposed_dt - acquired_dt).num_days()
}

fn is_long_term(days: i64) -> bool {
    days > 365
}

#[derive(Debug, Deserialize)]
pub struct TaxReportParams {
    #[serde(rename = "taxYear")]
    pub tax_year: i32,
}

#[tauri::command]
pub fn generate_tax_report(
    params: TaxReportParams,
    state: State<'_, SharedTaxLotsState>,
) -> Result<TaxReport, String> {
    state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())
        .map(|guard| guard.report(params.tax_year))
}

#[tauri::command]
pub fn export_tax_report(
    params: TaxReportParams,
    format: String,
    state: State<'_, SharedTaxLotsState>,
) -> Result<String, String> {
    state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())
        .and_then(|guard| guard.export(params.tax_year, &format))
}

fn export_turbotax_format(
    lots: &[TaxLot],
    tax_year: i32,
    strategy: LotStrategy,
) -> Result<String, String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "TurboTax Tax Report {} (Strategy: {:?})",
        tax_year, strategy
    ));
    lines.push(String::new());
    lines
        .push("Description,Date Acquired,Date Sold,Proceeds,Cost Basis,Gain/Loss,Term".to_string());

    for lot in lots {
        let acquired_date = lot.acquired_at.split('T').next().unwrap_or("");
        let sold_date = lot
            .disposed_at
            .as_ref()
            .and_then(|s| s.split('T').next())
            .unwrap_or("");
        let disposed_amount = lot.disposed_amount.unwrap_or(0.0);
        let unit_cost = lot.cost_basis / lot.amount;
        let cost = disposed_amount * unit_cost;
        let proceeds =
            disposed_amount * (lot.realized_gain.unwrap_or(0.0) + cost) / disposed_amount.max(1.0);
        let gain = lot.realized_gain.unwrap_or(0.0);

        let days = days_between(&lot.acquired_at, lot.disposed_at.as_deref());
        let term = if is_long_term(days) {
            "Long-Term"
        } else {
            "Short-Term"
        };

        lines.push(format!(
            "{} {},\"{}\",\"{}\",{:.2},{:.2},{:.2},{}",
            lot.symbol, lot.id, acquired_date, sold_date, proceeds, cost, gain, term
        ));
    }

    Ok(lines.join("\n"))
}

fn export_cointracker_format(
    lots: &[TaxLot],
    tax_year: i32,
    strategy: LotStrategy,
) -> Result<String, String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "CoinTracker Tax Report {} (Strategy: {:?})",
        tax_year, strategy
    ));
    lines.push(String::new());
    lines.push("Date,Type,Asset,Amount,Price,Fee,Total".to_string());

    for lot in lots {
        let disposed_amount = lot.disposed_amount.unwrap_or(0.0);
        let unit_cost = lot.cost_basis / lot.amount;
        let cost = disposed_amount * unit_cost;
        let proceeds =
            disposed_amount * (lot.realized_gain.unwrap_or(0.0) + cost) / disposed_amount.max(1.0);

        let acquired_date = lot.acquired_at.split('T').next().unwrap_or("");
        let sold_date = lot
            .disposed_at
            .as_ref()
            .and_then(|s| s.split('T').next())
            .unwrap_or("");

        lines.push(format!(
            "{},Buy,{},{:.6},{:.2},0.00,{:.2}",
            acquired_date, lot.symbol, lot.amount, lot.price_per_unit, cost
        ));

        lines.push(format!(
            "{},Sell,{},{:.6},{:.2},0.00,{:.2}",
            sold_date,
            lot.symbol,
            disposed_amount,
            proceeds / disposed_amount.max(1.0),
            proceeds
        ));
    }

    Ok(lines.join("\n"))
}

fn export_csv_format(
    lots: &[TaxLot],
    tax_year: i32,
    strategy: LotStrategy,
) -> Result<String, String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Tax Report {} (Strategy: {:?})",
        tax_year, strategy
    ));
    lines.push(String::new());
    lines.push(
        "Lot ID,Symbol,Acquired Date,Disposed Date,Amount,Cost Basis,Sale Price,Realized Gain"
            .to_string(),
    );

    for lot in lots {
        let acquired_date = lot.acquired_at.split('T').next().unwrap_or("");
        let disposed_date = lot
            .disposed_at
            .as_ref()
            .and_then(|s| s.split('T').next())
            .unwrap_or("");
        let disposed_amount = lot.disposed_amount.unwrap_or(0.0);
        let unit_cost = lot.cost_basis / lot.amount;
        let cost = disposed_amount * unit_cost;
        let proceeds =
            disposed_amount * (lot.realized_gain.unwrap_or(0.0) + cost) / disposed_amount.max(1.0);

        lines.push(format!(
            "{},{},{},{},{:.6},{:.2},{:.2},{:.2}",
            lot.id,
            lot.symbol,
            acquired_date,
            disposed_date,
            disposed_amount,
            cost,
            proceeds / disposed_amount.max(1.0),
            lot.realized_gain.unwrap_or(0.0)
        ));
    }

    Ok(lines.join("\n"))
}

#[tauri::command]
pub fn get_tax_loss_harvesting_suggestions(
    state: State<'_, SharedTaxLotsState>,
) -> Result<Vec<TaxLossHarvestingSuggestion>, String> {
    state
        .lock()
        .map_err(|_| "Tax lots unavailable".to_string())
        .map(|guard| guard.harvesting_suggestions())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_strategy_selects_oldest_lots() {
        let state = TaxLotsState::default();
        let lots = state.open_lots();

        let sol_lots: Vec<_> = lots.iter().filter(|l| l.symbol == "SOL").collect();

        assert!(sol_lots.len() >= 2);

        let first = parse_datetime(&sol_lots[0].acquired_at).unwrap();
        let second = parse_datetime(&sol_lots[1].acquired_at).unwrap();

        assert!(first <= second);
    }

    #[test]
    fn dispose_lot_calculates_realized_gain() {
        let mut state = TaxLotsState::default();
        let open_lots = state.open_lots();
        let lot = open_lots
            .iter()
            .find(|l| l.symbol == "SOL")
            .unwrap()
            .clone();

        let cost_per_unit = lot.cost_basis / lot.amount;
        let sale_price = 180.0;
        let dispose_amount = lot.amount / 2.0;

        let expected_cost = dispose_amount * cost_per_unit;
        let expected_proceeds = dispose_amount * sale_price;
        let expected_gain = expected_proceeds - expected_cost;

        let result = state.dispose(&lot.id, dispose_amount, sale_price).unwrap();

        assert_eq!(result.disposed_amount, Some(dispose_amount));
        assert!(result.disposed_at.is_some());
        let realized = result.realized_gain.unwrap();
        assert!((realized - expected_gain).abs() < 0.01);
    }

    #[test]
    fn tax_report_separates_short_and_long_term() {
        let mut state = TaxLotsState::default();

        let now = Utc::now();
        let year = now.year();

        let short_term_lot = TaxLot {
            id: "short".to_string(),
            symbol: "TEST".to_string(),
            mint: "test-mint".to_string(),
            amount: 10.0,
            cost_basis: 1000.0,
            price_per_unit: 100.0,
            acquired_at: (now - Duration::days(180)).to_rfc3339(),
            disposed_amount: Some(10.0),
            disposed_at: Some(now.to_rfc3339()),
            realized_gain: Some(200.0),
        };

        let long_term_lot = TaxLot {
            id: "long".to_string(),
            symbol: "TEST".to_string(),
            mint: "test-mint".to_string(),
            amount: 10.0,
            cost_basis: 1000.0,
            price_per_unit: 100.0,
            acquired_at: (now - Duration::days(400)).to_rfc3339(),
            disposed_amount: Some(10.0),
            disposed_at: Some(now.to_rfc3339()),
            realized_gain: Some(300.0),
        };

        state.add_lot(short_term_lot);
        state.add_lot(long_term_lot);

        let report = state.report(year);

        assert_eq!(report.short_term_gains, 200.0);
        assert_eq!(report.long_term_gains, 300.0);
    }

    #[test]
    fn tax_loss_harvesting_detects_losses() {
        let state = TaxLotsState::default();
        let suggestions = state.harvesting_suggestions();

        for suggestion in suggestions {
            assert!(suggestion.unrealized_loss > 0.0);
            assert!(suggestion.potential_tax_savings > 0.0);
            assert!(suggestion.current_price < suggestion.lot.price_per_unit);
        }
    }
}
