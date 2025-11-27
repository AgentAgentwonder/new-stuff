use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingStock {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change_24h: f64,
    pub percent_change_24h: f64,
    pub volume: f64,
    pub volume_change_24h: f64,
    pub unusual_volume: bool,
    pub market_cap: Option<f64>,
    pub avg_volume: f64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopMover {
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub percent_change: f64,
    pub volume: f64,
    pub market_cap: Option<f64>,
    pub direction: MoverDirection,
    pub session: TradingSession,
    pub technical_indicators: TechnicalIndicators,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MoverDirection {
    Gainer,
    Loser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradingSession {
    Regular,
    PreMarket,
    AfterHours,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicalIndicators {
    pub rsi: Option<f64>,
    pub macd: Option<String>,
    pub volume_ratio: f64,
    pub momentum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewIPO {
    pub symbol: String,
    pub name: String,
    pub ipo_date: String,
    pub offer_price: f64,
    pub current_price: Option<f64>,
    pub percent_change: Option<f64>,
    pub shares_offered: Option<f64>,
    pub market_cap: Option<f64>,
    pub exchange: String,
    pub status: IPOStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IPOStatus {
    Upcoming,
    Today,
    Recent,
    Filed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsEvent {
    pub symbol: String,
    pub name: String,
    pub date: String,
    pub time: EarningsTime,
    pub fiscal_quarter: String,
    pub estimate_eps: Option<f64>,
    pub actual_eps: Option<f64>,
    pub surprise_percent: Option<f64>,
    pub historical_reaction: Option<HistoricalReaction>,
    pub has_alert: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EarningsTime {
    BeforeMarket,
    AfterMarket,
    DuringMarket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalReaction {
    pub avg_move_percent: f64,
    pub last_reaction_percent: f64,
    pub beat_miss_ratio: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockNews {
    pub id: String,
    pub symbol: String,
    pub title: String,
    pub summary: String,
    pub ai_summary: Option<String>,
    pub url: String,
    pub source: String,
    pub published_at: String,
    pub sentiment: Sentiment,
    pub impact_level: ImpactLevel,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sentiment {
    Bullish,
    Neutral,
    Bearish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalHolding {
    pub symbol: String,
    pub institution_name: String,
    pub shares: f64,
    pub value: f64,
    pub percent_of_portfolio: f64,
    pub change_shares: f64,
    pub change_percent: f64,
    pub quarter: String,
    pub is_whale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderActivity {
    pub symbol: String,
    pub insider_name: String,
    pub insider_title: String,
    pub transaction_type: TransactionType,
    pub shares: f64,
    pub price: f64,
    pub value: f64,
    pub transaction_date: String,
    pub filing_date: String,
    pub is_significant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Buy,
    Sell,
    Option,
    Gift,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAlert {
    pub id: String,
    pub symbol: String,
    pub alert_type: StockAlertType,
    pub message: String,
    pub triggered_at: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StockAlertType {
    EarningsUpcoming,
    UnusualVolume,
    SignificantMove,
    WhaleActivity,
    InsiderActivity,
    NewIPO,
}
