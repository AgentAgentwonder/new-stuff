mod calculator;
mod exports;
mod harvesting;
mod jurisdiction;
pub mod types;
mod wash_sale;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::portfolio::{SharedTaxLotsState, TaxLot, TaxReportParams};
use crate::security::keystore::Keystore;
use tauri::State;

use calculator::TaxCalculator;
use exports::TaxExportService;
use harvesting::TaxLossHarvester;
use jurisdiction::JurisdictionManager;
use types::{
    HarvestingRecommendation, TaxAlert, TaxAlertType, TaxExportFormat, TaxJurisdiction,
    TaxProjection, TaxSettings, WashSaleWarning,
};
use wash_sale::WashSaleDetector;

#[derive(Default)]
pub struct TaxPlanningEngine {
    pub jurisdiction: TaxJurisdiction,
    pub settings: TaxSettings,
    pub recent_transactions: Vec<(String, chrono::DateTime<Utc>, f64, String)>,
    pub current_prices: HashMap<String, f64>,
    pub carryforward_losses: f64,
    pub ytd_realized_gains: f64,
}

pub type SharedTaxPlanningEngine = Arc<RwLock<TaxPlanningEngine>>;

impl TaxPlanningEngine {
    pub fn new(jurisdiction: TaxJurisdiction) -> Self {
        let mut engine = Self {
            jurisdiction: jurisdiction.clone(),
            settings: TaxSettings {
                jurisdiction,
                ..Default::default()
            },
            recent_transactions: Vec::new(),
            current_prices: HashMap::new(),
            carryforward_losses: 0.0,
            ytd_realized_gains: 0.0,
        };

        // Populate with mock price data
        engine.current_prices.insert("SOL".to_string(), 170.5);
        engine.current_prices.insert("BTC".to_string(), 64000.0);
        engine.current_prices.insert("ETH".to_string(), 3200.0);
        engine.current_prices.insert("JUP".to_string(), 1.98);

        engine
    }

    pub fn generate_mock_transactions(&mut self) {
        if !self.recent_transactions.is_empty() {
            return;
        }

        let now = Utc::now();
        self.recent_transactions = vec![
            (
                "SOL".to_string(),
                now - chrono::Duration::days(5),
                120.0,
                "BUY".to_string(),
            ),
            (
                "SOL".to_string(),
                now - chrono::Duration::days(12),
                80.0,
                "SELL".to_string(),
            ),
            (
                "ETH".to_string(),
                now - chrono::Duration::days(8),
                15.0,
                "BUY".to_string(),
            ),
            (
                "BTC".to_string(),
                now - chrono::Duration::days(18),
                0.25,
                "BUY".to_string(),
            ),
        ];
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCenterSummary {
    pub projection: TaxProjection,
    pub wash_sale_warnings: Vec<WashSaleWarning>,
    pub harvesting_recommendations: Vec<HarvestingRecommendation>,
    pub alerts: Vec<TaxAlert>,
    pub available_jurisdictions: Vec<TaxJurisdiction>,
    pub settings: TaxSettings,
}

#[tauri::command]
pub async fn get_tax_center_summary(
    tax_year: Option<i32>,
    engine: State<'_, SharedTaxPlanningEngine>,
    tax_lot_state: State<'_, SharedTaxLotsState>,
) -> Result<TaxCenterSummary, String> {
    let tax_year = tax_year.unwrap_or_else(|| Utc::now().year());

    let engine = engine.read().await;
    let jurisdiction = engine.settings.jurisdiction.clone();
    let mut calculator = TaxCalculator::new(jurisdiction.clone());
    let wash_sale_detector = WashSaleDetector::new(jurisdiction.wash_sale_period_days);
    let harvester = TaxLossHarvester::new(
        jurisdiction.clone(),
        engine.settings.harvesting_threshold_usd,
    );
    let export_service = TaxExportService::new();

    drop(export_service); // Not used directly but ensures module initialization

    let lots = tax_lot_state
        .lock()
        .map_err(|_| "Unable to access tax lots".to_string())?
        .all_lots();

    let open_lots = tax_lot_state
        .lock()
        .map_err(|_| "Unable to access tax lots".to_string())?
        .open_lots();

    let mut realized_gains = Vec::new();

    for lot in lots.iter() {
        if let (Some(disposed_amount), Some(disposed_at), Some(_realized_gain)) = (
            lot.disposed_amount,
            lot.disposed_at.clone(),
            lot.realized_gain,
        ) {
            let sale_date = disposed_at
                .parse::<chrono::DateTime<Utc>>()
                .map_err(|e| format!("Invalid disposed date: {e}"))?;

            let sale_price = if disposed_amount > 0.0 {
                let cost_per_unit = lot.cost_basis / lot.amount;
                let realized = lot.realized_gain.unwrap_or(0.0);
                (cost_per_unit * disposed_amount + realized) / disposed_amount
            } else {
                lot.price_per_unit
            };

            realized_gains.push(calculator.calculate_capital_gain(
                lot,
                sale_price,
                disposed_amount,
                sale_date,
            )?);
        }
    }

    let mut unrealized = Vec::new();
    let mut current_prices = engine.current_prices.clone();
    if current_prices.is_empty() {
        current_prices.insert("SOL".to_string(), 170.5);
    }

    for lot in open_lots.iter() {
        let price = current_prices
            .get(&lot.symbol)
            .cloned()
            .unwrap_or(lot.price_per_unit);
        let current_value = price * lot.amount;
        let gain_loss = current_value - lot.cost_basis;
        if gain_loss >= 0.0 {
            unrealized.push((gain_loss, 0.0));
        } else {
            unrealized.push((0.0, gain_loss.abs()));
        }
    }

    let projection = calculator.calculate_projection(
        realized_gains,
        unrealized,
        engine.carryforward_losses,
        tax_year,
    );

    let warnings = wash_sale_detector.detect_wash_sales(&lots, &engine.recent_transactions);

    let mut recent_purchases: HashMap<String, Vec<(chrono::DateTime<Utc>, f64)>> = HashMap::new();
    for (asset, date, amount, txn_type) in engine.recent_transactions.iter() {
        if txn_type == "BUY" || txn_type == "RECEIVE" {
            recent_purchases
                .entry(asset.clone())
                .or_default()
                .push((*date, *amount));
        }
    }

    let recommendations =
        harvester.generate_recommendations(&open_lots, &engine.current_prices, &recent_purchases);

    let alerts = build_tax_alerts(&projection, &warnings, &recommendations);
    let available_jurisdictions = JurisdictionManager::get_available_jurisdictions();

    Ok(TaxCenterSummary {
        projection,
        wash_sale_warnings: warnings,
        harvesting_recommendations: recommendations,
        alerts,
        available_jurisdictions,
        settings: engine.settings.clone(),
    })
}

#[tauri::command]
pub async fn update_tax_settings(
    settings: TaxSettings,
    engine: State<'_, SharedTaxPlanningEngine>,
    keystore: State<'_, Keystore>,
) -> Result<TaxSettings, String> {
    let mut engine = engine.write().await;
    let manager = JurisdictionManager::new();
    manager.validate_jurisdiction(&settings.jurisdiction)?;

    manager.save_jurisdiction(&*keystore, "default", &settings.jurisdiction)?;
    engine.settings = settings.clone();

    Ok(settings)
}

#[tauri::command]
pub async fn export_tax_center_report(
    format: String,
    params: TaxReportParams,
    tax_lot_state: State<'_, SharedTaxLotsState>,
) -> Result<TaxExportFormat, String> {
    let export_service = TaxExportService::new();
    let lots_state = tax_lot_state
        .lock()
        .map_err(|_| "Unable to access tax lots".to_string())?;
    let lots = lots_state
        .all_lots()
        .into_iter()
        .filter(|lot| {
            lot.disposed_at
                .as_ref()
                .and_then(|d| d.parse::<chrono::DateTime<Utc>>().ok())
                .map(|dt| dt.year() == params.tax_year)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    match format.as_str() {
        "cointracker" => export_service.export_cointracker(params.tax_year, &lots),
        "koinly" => export_service.export_koinly(params.tax_year, &lots),
        "csv" => export_service.export_csv(params.tax_year, &lots),
        other => Err(format!("Unsupported export format: {}", other)),
    }
}

fn build_tax_alerts(
    projection: &TaxProjection,
    warnings: &[WashSaleWarning],
    recommendations: &[HarvestingRecommendation],
) -> Vec<TaxAlert> {
    let mut alerts = Vec::new();

    if projection.total_tax_owed > 5000.0 {
        alerts.push(TaxAlert {
            id: Uuid::new_v4().to_string(),
            alert_type: TaxAlertType::LargeGain,
            severity: types::AlertSeverity::High,
            title: "High tax liability projected".to_string(),
            message: format!(
                "Projected tax liability is ${:.2} with effective rate {:.2}%. Consider harvesting losses.",
                projection.total_tax_owed,
                projection.effective_tax_rate * 100.0
            ),
            asset: None,
            action_required: true,
            action_deadline: Some(Utc::now() + chrono::Duration::days(30)),
            recommendations: vec![
                "Review tax-loss harvesting opportunities".to_string(),
                "Consider estimated tax payments if applicable".to_string(),
            ],
            created_at: Utc::now(),
            dismissed: false,
        });
    }

    for warning in warnings.iter() {
        alerts.push(TaxAlert {
            id: Uuid::new_v4().to_string(),
            alert_type: TaxAlertType::WashSale,
            severity: match warning.severity {
                types::WashSaleSeverity::High => types::AlertSeverity::Critical,
                types::WashSaleSeverity::Medium => types::AlertSeverity::High,
                types::WashSaleSeverity::Low => types::AlertSeverity::Medium,
            },
            title: format!("Wash sale detected: {}", warning.asset),
            message: format!(
                "${:.2} loss disallowed due to repurchase on {}. {}",
                warning.disallowed_loss,
                warning.repurchase_date.format("%Y-%m-%d"),
                warning.recommendation
            ),
            asset: Some(warning.asset.clone()),
            action_required: true,
            action_deadline: Some(warning.wash_sale_period_end),
            recommendations: vec![warning.recommendation.clone()],
            created_at: Utc::now(),
            dismissed: false,
        });
    }

    if recommendations.is_empty() {
        alerts.push(TaxAlert {
            id: Uuid::new_v4().to_string(),
            alert_type: TaxAlertType::YearEndDeadline,
            severity: types::AlertSeverity::Info,
            title: "No harvesting opportunities".to_string(),
            message: "No material tax-loss harvesting candidates found.".to_string(),
            asset: None,
            action_required: false,
            action_deadline: None,
            recommendations: vec![
                "Monitor portfolio for emerging losses".to_string(),
                "Review tax settings for accuracy".to_string(),
            ],
            created_at: Utc::now(),
            dismissed: false,
        });
    } else {
        for recommendation in recommendations.iter().take(3) {
            alerts.push(TaxAlert {
                id: Uuid::new_v4().to_string(),
                alert_type: TaxAlertType::HarvestingOpportunity,
                severity: match recommendation.priority {
                    types::RecommendationPriority::Critical => types::AlertSeverity::Critical,
                    types::RecommendationPriority::High => types::AlertSeverity::High,
                    types::RecommendationPriority::Medium => types::AlertSeverity::Medium,
                    types::RecommendationPriority::Low => types::AlertSeverity::Low,
                },
                title: format!("Harvesting opportunity: {}", recommendation.asset),
                message: format!(
                    "${:.2} unrealized loss with potential tax savings ${:.2}. {}",
                    recommendation.unrealized_loss,
                    recommendation.tax_savings,
                    recommendation.reason
                ),
                asset: Some(recommendation.asset.clone()),
                action_required: true,
                action_deadline: recommendation.expires_at,
                recommendations: vec![
                    "Review position sizing".to_string(),
                    "Consider alternative assets to avoid wash sales".to_string(),
                ],
                created_at: Utc::now(),
                dismissed: false,
            });
        }
    }

    alerts
}

pub fn initialize_tax_engine(keystore: &Keystore) -> SharedTaxPlanningEngine {
    let manager = JurisdictionManager::new();
    let jurisdiction = manager
        .load_jurisdiction(keystore, "default")
        .unwrap_or_else(|_| TaxJurisdiction::default());
    let mut engine = TaxPlanningEngine::new(jurisdiction);
    engine.generate_mock_transactions();

    Arc::new(RwLock::new(engine))
}
