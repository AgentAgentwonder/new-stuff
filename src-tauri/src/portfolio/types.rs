use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub mint: String,
    pub amount: f64,
    #[serde(rename = "currentPrice")]
    pub current_price: f64,
    #[serde(rename = "avgEntryPrice")]
    pub avg_entry_price: f64,
    #[serde(rename = "totalValue")]
    pub total_value: f64,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: f64,
    #[serde(rename = "unrealizedPnlPercent")]
    pub unrealized_pnl_percent: f64,
    pub allocation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    #[serde(rename = "totalValue")]
    pub total_value: f64,
    #[serde(rename = "dailyPnl")]
    pub daily_pnl: f64,
    #[serde(rename = "dailyPnlPercent")]
    pub daily_pnl_percent: f64,
    #[serde(rename = "weeklyPnl")]
    pub weekly_pnl: f64,
    #[serde(rename = "weeklyPnlPercent")]
    pub weekly_pnl_percent: f64,
    #[serde(rename = "monthlyPnl")]
    pub monthly_pnl: f64,
    #[serde(rename = "monthlyPnlPercent")]
    pub monthly_pnl_percent: f64,
    #[serde(rename = "allTimePnl")]
    pub all_time_pnl: f64,
    #[serde(rename = "allTimePnlPercent")]
    pub all_time_pnl_percent: f64,
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: f64,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: f64,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationTarget {
    pub symbol: String,
    #[serde(rename = "targetPercent")]
    pub target_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceProfile {
    pub id: String,
    pub name: String,
    pub targets: Vec<AllocationTarget>,
    #[serde(rename = "deviationTriggerPercent")]
    pub deviation_trigger_percent: f64,
    #[serde(rename = "timeIntervalHours")]
    pub time_interval_hours: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceAction {
    pub symbol: String,
    pub mint: String,
    #[serde(rename = "currentPercent")]
    pub current_percent: f64,
    #[serde(rename = "targetPercent")]
    pub target_percent: f64,
    pub deviation: f64,
    pub action: String,
    pub amount: f64,
    #[serde(rename = "estimatedValue")]
    pub estimated_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceHistory {
    pub id: String,
    #[serde(rename = "profileId")]
    pub profile_id: String,
    pub actions: Vec<RebalanceAction>,
    #[serde(rename = "triggerType")]
    pub trigger_type: String,
    pub executed: bool,
    #[serde(rename = "executedAt")]
    pub executed_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLot {
    pub id: String,
    pub symbol: String,
    pub mint: String,
    pub amount: f64,
    #[serde(rename = "costBasis")]
    pub cost_basis: f64,
    #[serde(rename = "pricePerUnit")]
    pub price_per_unit: f64,
    #[serde(rename = "acquiredAt")]
    pub acquired_at: String,
    #[serde(rename = "disposedAmount")]
    pub disposed_amount: Option<f64>,
    #[serde(rename = "disposedAt")]
    pub disposed_at: Option<String>,
    #[serde(rename = "realizedGain")]
    pub realized_gain: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LotStrategy {
    FIFO,
    LIFO,
    HIFO,
    SPECIFIC,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxReport {
    #[serde(rename = "taxYear")]
    pub tax_year: i32,
    pub lots: Vec<TaxLot>,
    #[serde(rename = "totalRealizedGains")]
    pub total_realized_gains: f64,
    #[serde(rename = "totalRealizedLosses")]
    pub total_realized_losses: f64,
    #[serde(rename = "netGainLoss")]
    pub net_gain_loss: f64,
    #[serde(rename = "shortTermGains")]
    pub short_term_gains: f64,
    #[serde(rename = "longTermGains")]
    pub long_term_gains: f64,
    pub strategy: LotStrategy,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLossHarvestingSuggestion {
    pub symbol: String,
    pub mint: String,
    pub lot: TaxLot,
    #[serde(rename = "currentPrice")]
    pub current_price: f64,
    #[serde(rename = "unrealizedLoss")]
    pub unrealized_loss: f64,
    #[serde(rename = "potentialTaxSavings")]
    pub potential_tax_savings: f64,
    #[serde(rename = "daysHeld")]
    pub days_held: i64,
}
