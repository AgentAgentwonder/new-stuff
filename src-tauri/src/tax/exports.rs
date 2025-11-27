use super::types::{
    CoinTrackerExport, CoinTrackerSummary, CoinTrackerTransaction, KoinlyExport, KoinlySummary,
    KoinlyTransaction, TaxExportFormat,
};
use crate::portfolio::TaxLot;
use chrono::Utc;

pub struct TaxExportService;

impl TaxExportService {
    pub fn new() -> Self {
        Self
    }

    pub fn export_cointracker(
        &self,
        tax_year: i32,
        lots: &[TaxLot],
    ) -> Result<TaxExportFormat, String> {
        let mut transactions = Vec::new();
        let mut total_gains = 0.0;
        let mut total_losses = 0.0;

        for lot in lots {
            let amount = lot.disposed_amount.unwrap_or(0.0);
            if amount == 0.0 {
                continue;
            }

            let acquired_date = lot
                .acquired_at
                .split('T')
                .next()
                .unwrap_or_default()
                .to_string();

            let disposed_date = lot
                .disposed_at
                .as_deref()
                .and_then(|d| d.split('T').next())
                .unwrap_or_default()
                .to_string();

            let unit_cost_basis = lot.cost_basis / lot.amount;
            let cost_basis = unit_cost_basis * amount;
            let realized_gain = lot.realized_gain.unwrap_or(0.0);

            if realized_gain >= 0.0 {
                total_gains += realized_gain;
            } else {
                total_losses += realized_gain.abs();
            }

            // Buy record
            transactions.push(CoinTrackerTransaction {
                date: acquired_date,
                received_quantity: lot.amount,
                received_currency: lot.symbol.clone(),
                sent_quantity: cost_basis,
                sent_currency: "USD".to_string(),
                fee_amount: 0.0,
                fee_currency: "USD".to_string(),
                tag: "Buy".to_string(),
                description: format!("Purchase of {}", lot.symbol),
            });

            // Sell record
            transactions.push(CoinTrackerTransaction {
                date: disposed_date,
                received_quantity: 0.0,
                received_currency: lot.symbol.clone(),
                sent_quantity: amount,
                sent_currency: lot.symbol.clone(),
                fee_amount: 0.0,
                fee_currency: "USD".to_string(),
                tag: "Sell".to_string(),
                description: format!(
                    "Sale of {} with gain/loss ${:.2}",
                    lot.symbol, realized_gain
                ),
            });
        }

        let summary = CoinTrackerSummary {
            total_transactions: transactions.len(),
            total_gains,
            total_losses,
            net_gain_loss: total_gains - total_losses,
        };

        let data = serde_json::to_string_pretty(&CoinTrackerExport {
            transactions,
            summary,
        })
        .map_err(|e| format!("Failed to serialize CoinTracker export: {e}"))?;

        Ok(TaxExportFormat {
            format: "cointracker".to_string(),
            version: "1.0".to_string(),
            generated_at: Utc::now(),
            jurisdiction: "Global".to_string(),
            tax_year,
            data,
            filename: format!("cointracker_export_{}.json", tax_year),
        })
    }

    pub fn export_koinly(&self, tax_year: i32, lots: &[TaxLot]) -> Result<TaxExportFormat, String> {
        let mut transactions = Vec::new();
        let mut cost_basis_total = 0.0;
        let mut proceeds_total = 0.0;

        for lot in lots {
            let amount = lot.disposed_amount.unwrap_or(0.0);
            if amount == 0.0 {
                continue;
            }

            let acquired_date = lot
                .acquired_at
                .split('T')
                .next()
                .unwrap_or_default()
                .to_string();

            let disposed_date = lot
                .disposed_at
                .as_deref()
                .and_then(|d| d.split('T').next())
                .unwrap_or_default()
                .to_string();

            let unit_cost_basis = lot.cost_basis / lot.amount;
            let cost_basis = unit_cost_basis * amount;
            let realized_gain = lot.realized_gain.unwrap_or(0.0);
            let proceeds = cost_basis + realized_gain;

            cost_basis_total += cost_basis;
            proceeds_total += proceeds;

            transactions.push(KoinlyTransaction {
                date: acquired_date.clone(),
                sent_amount: cost_basis,
                sent_currency: "USD".to_string(),
                received_amount: lot.amount,
                received_currency: lot.symbol.clone(),
                fee_amount: 0.0,
                fee_currency: "USD".to_string(),
                net_worth_amount: cost_basis,
                net_worth_currency: "USD".to_string(),
                label: "buy".to_string(),
                description: format!("Purchase of {}", lot.symbol),
                tx_hash: format!("{}-buy-{}", lot.id, acquired_date),
            });

            transactions.push(KoinlyTransaction {
                date: disposed_date.clone(),
                sent_amount: amount,
                sent_currency: lot.symbol.clone(),
                received_amount: proceeds,
                received_currency: "USD".to_string(),
                fee_amount: 0.0,
                fee_currency: "USD".to_string(),
                net_worth_amount: proceeds,
                net_worth_currency: "USD".to_string(),
                label: "sell".to_string(),
                description: format!("Sale of {}", lot.symbol),
                tx_hash: format!("{}-sell-{}", lot.id, disposed_date),
            });
        }

        let summary = KoinlySummary {
            total_transactions: transactions.len(),
            cost_basis: cost_basis_total,
            proceeds: proceeds_total,
            capital_gains: proceeds_total - cost_basis_total,
        };

        let data = serde_json::to_string_pretty(&KoinlyExport {
            transactions,
            summary,
        })
        .map_err(|e| format!("Failed to serialize Koinly export: {e}"))?;

        Ok(TaxExportFormat {
            format: "koinly".to_string(),
            version: "1.0".to_string(),
            generated_at: Utc::now(),
            jurisdiction: "Global".to_string(),
            tax_year,
            data,
            filename: format!("koinly_export_{}.json", tax_year),
        })
    }

    pub fn export_csv(&self, tax_year: i32, lots: &[TaxLot]) -> Result<TaxExportFormat, String> {
        let mut rows = Vec::new();
        rows.push("Date Acquired,Date Sold,Asset,Amount,Cost Basis,Proceeds,Gain/Loss".to_string());

        for lot in lots {
            let amount = lot.disposed_amount.unwrap_or(0.0);
            if amount == 0.0 {
                continue;
            }

            let acquired_date = lot
                .acquired_at
                .split('T')
                .next()
                .unwrap_or_default()
                .to_string();

            let disposed_date = lot
                .disposed_at
                .as_deref()
                .and_then(|d| d.split('T').next())
                .unwrap_or_default()
                .to_string();

            let unit_cost_basis = lot.cost_basis / lot.amount;
            let cost_basis = unit_cost_basis * amount;
            let realized_gain = lot.realized_gain.unwrap_or(0.0);
            let proceeds = cost_basis + realized_gain;

            rows.push(format!(
                "{},{},{},{:.4},{:.2},{:.2},{:.2}",
                acquired_date,
                disposed_date,
                lot.symbol,
                amount,
                cost_basis,
                proceeds,
                realized_gain
            ));
        }

        let csv_data = rows.join("\n");

        Ok(TaxExportFormat {
            format: "csv".to_string(),
            version: "1.0".to_string(),
            generated_at: Utc::now(),
            jurisdiction: "Global".to_string(),
            tax_year,
            data: csv_data,
            filename: format!("tax_report_{}.csv", tax_year),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_lot() -> TaxLot {
        TaxLot {
            id: "test-1".to_string(),
            symbol: "SOL".to_string(),
            mint: "So11111111111111111111111111111111111111112".to_string(),
            amount: 100.0,
            cost_basis: 15000.0,
            price_per_unit: 150.0,
            acquired_at: (Utc::now() - Duration::days(400)).to_rfc3339(),
            disposed_amount: Some(100.0),
            disposed_at: Some(Utc::now().to_rfc3339()),
            realized_gain: Some(2500.0),
        }
    }

    #[test]
    fn test_cointracker_export() {
        let export_service = TaxExportService::new();
        let lot = create_test_lot();
        let export = export_service.export_cointracker(2024, &[lot]);

        assert!(export.is_ok());
        let export = export.unwrap();
        assert_eq!(export.format, "cointracker");
        assert!(export.data.contains("Purchase of SOL"));
    }

    #[test]
    fn test_koinly_export() {
        let export_service = TaxExportService::new();
        let lot = create_test_lot();
        let export = export_service.export_koinly(2024, &[lot]);

        assert!(export.is_ok());
        let export = export.unwrap();
        assert_eq!(export.format, "koinly");
        assert!(export.data.contains("Sale of SOL"));
    }

    #[test]
    fn test_csv_export() {
        let export_service = TaxExportService::new();
        let lot = create_test_lot();
        let export = export_service.export_csv(2024, &[lot]);

        assert!(export.is_ok());
        let export = export.unwrap();
        assert_eq!(export.format, "csv");
        assert!(export.data.contains("Purchase of SOL"));
    }
}
