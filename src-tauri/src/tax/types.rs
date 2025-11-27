use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxJurisdiction {
    pub code: String,
    pub name: String,
    pub short_term_rate: f64,
    pub long_term_rate: f64,
    pub holding_period_days: i64,
    pub wash_sale_period_days: i64,
    pub capital_loss_limit: Option<f64>,
    pub tax_year_start: String, // MM-DD format
    pub requires_reporting_threshold: Option<f64>,
    pub supports_like_kind_exchange: bool,
    pub crypto_specific_rules: HashMap<String, serde_json::Value>,
}

impl Default for TaxJurisdiction {
    fn default() -> Self {
        Self::us_federal()
    }
}

impl TaxJurisdiction {
    pub fn us_federal() -> Self {
        let mut crypto_rules = HashMap::new();
        crypto_rules.insert(
            "staking_income_taxable".to_string(),
            serde_json::json!(true),
        );
        crypto_rules.insert("defi_yield_taxable".to_string(), serde_json::json!(true));
        crypto_rules.insert("airdrop_taxable".to_string(), serde_json::json!(true));

        Self {
            code: "US".to_string(),
            name: "United States - Federal".to_string(),
            short_term_rate: 0.37,
            long_term_rate: 0.20,
            holding_period_days: 365,
            wash_sale_period_days: 30,
            capital_loss_limit: Some(3000.0),
            tax_year_start: "01-01".to_string(),
            requires_reporting_threshold: Some(10.0),
            supports_like_kind_exchange: false,
            crypto_specific_rules: crypto_rules,
        }
    }

    pub fn uk() -> Self {
        let mut crypto_rules = HashMap::new();
        crypto_rules.insert(
            "capital_gains_allowance".to_string(),
            serde_json::json!(12300),
        );

        Self {
            code: "UK".to_string(),
            name: "United Kingdom".to_string(),
            short_term_rate: 0.20,
            long_term_rate: 0.20,
            holding_period_days: 0,
            wash_sale_period_days: 30,
            capital_loss_limit: None,
            tax_year_start: "04-06".to_string(),
            requires_reporting_threshold: None,
            supports_like_kind_exchange: false,
            crypto_specific_rules: crypto_rules,
        }
    }

    pub fn germany() -> Self {
        Self {
            code: "DE".to_string(),
            name: "Germany".to_string(),
            short_term_rate: 0.45,
            long_term_rate: 0.0,
            holding_period_days: 365,
            wash_sale_period_days: 0,
            capital_loss_limit: None,
            tax_year_start: "01-01".to_string(),
            requires_reporting_threshold: Some(600.0),
            supports_like_kind_exchange: false,
            crypto_specific_rules: HashMap::new(),
        }
    }

    pub fn australia() -> Self {
        Self {
            code: "AU".to_string(),
            name: "Australia".to_string(),
            short_term_rate: 0.45,
            long_term_rate: 0.225,
            holding_period_days: 365,
            wash_sale_period_days: 0,
            capital_loss_limit: None,
            tax_year_start: "07-01".to_string(),
            requires_reporting_threshold: None,
            supports_like_kind_exchange: false,
            crypto_specific_rules: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapitalGain {
    pub asset: String,
    pub mint_address: String,
    pub amount: f64,
    pub cost_basis: f64,
    pub proceeds: f64,
    pub gain_loss: f64,
    pub is_long_term: bool,
    pub acquired_date: DateTime<Utc>,
    pub disposed_date: DateTime<Utc>,
    pub holding_period_days: i64,
    pub tax_rate: f64,
    pub tax_owed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WashSaleWarning {
    pub asset: String,
    pub mint_address: String,
    pub sale_date: DateTime<Utc>,
    pub sale_amount: f64,
    pub loss_amount: f64,
    pub disallowed_loss: f64,
    pub repurchase_date: DateTime<Utc>,
    pub repurchase_amount: f64,
    pub wash_sale_period_start: DateTime<Utc>,
    pub wash_sale_period_end: DateTime<Utc>,
    pub severity: WashSaleSeverity,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WashSaleSeverity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxProjection {
    pub tax_year: i32,
    pub jurisdiction: String,
    pub total_short_term_gains: f64,
    pub total_long_term_gains: f64,
    pub total_short_term_losses: f64,
    pub total_long_term_losses: f64,
    pub net_short_term: f64,
    pub net_long_term: f64,
    pub total_tax_owed: f64,
    pub effective_tax_rate: f64,
    pub potential_savings_from_harvesting: f64,
    pub unrealized_gains: f64,
    pub unrealized_losses: f64,
    pub carryforward_losses: f64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarvestingRecommendation {
    pub id: String,
    pub asset: String,
    pub mint_address: String,
    pub lot_id: String,
    pub current_price: f64,
    pub cost_basis: f64,
    pub unrealized_loss: f64,
    pub amount: f64,
    pub holding_period_days: i64,
    pub tax_savings: f64,
    pub priority: RecommendationPriority,
    pub reason: String,
    pub wash_sale_risk: bool,
    pub alternative_assets: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxExportFormat {
    pub format: String,
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub jurisdiction: String,
    pub tax_year: i32,
    pub data: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinTrackerExport {
    pub transactions: Vec<CoinTrackerTransaction>,
    pub summary: CoinTrackerSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinTrackerTransaction {
    pub date: String,
    pub received_quantity: f64,
    pub received_currency: String,
    pub sent_quantity: f64,
    pub sent_currency: String,
    pub fee_amount: f64,
    pub fee_currency: String,
    pub tag: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinTrackerSummary {
    pub total_transactions: usize,
    pub total_gains: f64,
    pub total_losses: f64,
    pub net_gain_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KoinlyExport {
    pub transactions: Vec<KoinlyTransaction>,
    pub summary: KoinlySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KoinlyTransaction {
    pub date: String,
    pub sent_amount: f64,
    pub sent_currency: String,
    pub received_amount: f64,
    pub received_currency: String,
    pub fee_amount: f64,
    pub fee_currency: String,
    pub net_worth_amount: f64,
    pub net_worth_currency: String,
    pub label: String,
    pub description: String,
    pub tx_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KoinlySummary {
    pub total_transactions: usize,
    pub cost_basis: f64,
    pub proceeds: f64,
    pub capital_gains: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxAlert {
    pub id: String,
    pub alert_type: TaxAlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub asset: Option<String>,
    pub action_required: bool,
    pub action_deadline: Option<DateTime<Utc>>,
    pub recommendations: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaxAlertType {
    WashSale,
    LargeGain,
    LargeLoss,
    HarvestingOpportunity,
    YearEndDeadline,
    TaxRateChange,
    JurisdictionChange,
    MissingCostBasis,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxSettings {
    pub jurisdiction: TaxJurisdiction,
    pub tax_year: i32,
    pub default_cost_basis_method: String,
    pub enable_wash_sale_detection: bool,
    pub enable_tax_loss_harvesting: bool,
    pub harvesting_threshold_usd: f64,
    pub year_end_reminder_days: i32,
    pub custom_tax_rates: Option<CustomTaxRates>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTaxRates {
    pub short_term_rate: Option<f64>,
    pub long_term_rate: Option<f64>,
    pub state_tax_rate: Option<f64>,
}

impl Default for TaxSettings {
    fn default() -> Self {
        Self {
            jurisdiction: TaxJurisdiction::default(),
            tax_year: Utc::now().year(),
            default_cost_basis_method: "FIFO".to_string(),
            enable_wash_sale_detection: true,
            enable_tax_loss_harvesting: true,
            harvesting_threshold_usd: 100.0,
            year_end_reminder_days: 30,
            custom_tax_rates: None,
        }
    }
}
