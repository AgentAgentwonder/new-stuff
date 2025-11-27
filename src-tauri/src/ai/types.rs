// AI Trading Features - Shared Types
// Types used across sentiment analysis, predictions, and backtesting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Sentiment Analysis Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentimentScore {
    pub id: String,
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub sentiment_score: f64,  // -1.0 to 1.0
    pub confidence: f64,        // 0.0 to 1.0
    pub source: SentimentSource,
    pub sample_size: Option<i32>,
    pub positive_mentions: i32,
    pub negative_mentions: i32,
    pub neutral_mentions: i32,
    pub timestamp: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SentimentSource {
    Twitter,
    Reddit,
    Discord,
    #[serde(rename = "onchain")]
    OnChain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentimentAnalysis {
    pub token_mint: String,
    pub token_symbol: Option<String>,
    pub overall_sentiment: f64,
    pub overall_confidence: f64,
    pub sources: Vec<SentimentScore>,
    pub trend: SentimentTrend,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SentimentTrend {
    Rising,
    Falling,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentimentDataPoint {
    pub timestamp: DateTime<Utc>,
    pub sentiment_score: f64,
    pub confidence: f64,
    pub sample_size: i32,
}

// ============================================================================
// Price Prediction Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricePrediction {
    pub id: String,
    pub token_mint: String,
    pub prediction_timestamp: DateTime<Utc>,
    pub target_timestamp: DateTime<Utc>,
    pub predicted_price: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
    pub actual_price: Option<f64>,
    pub model_version: String,
    pub features: PredictionFeatures,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionFeatures {
    pub price_history: Vec<f64>,
    pub volume_history: Vec<f64>,
    pub sentiment_score: Option<f64>,
    pub social_mentions: Option<i32>,
    pub wallet_activity: Option<WalletActivityMetrics>,
    pub tvl_change: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletActivityMetrics {
    pub unique_traders: i32,
    pub buy_pressure: f64,
    pub sell_pressure: f64,
    pub whale_activity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPerformanceMetrics {
    pub model_version: String,
    pub token_mint: Option<String>,
    pub timeframe: String,
    pub mae: f64,
    pub rmse: f64,
    pub accuracy_percent: f64,
    pub total_predictions: i32,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestResults {
    pub predictions: Vec<PricePrediction>,
    pub accuracy_metrics: ModelPerformanceMetrics,
    pub confidence_calibration: f64,
}

// ============================================================================
// Strategy Backtesting Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyConfig {
    pub name: String,
    pub rules: StrategyRules,
    pub position_sizing: PositionSizing,
    pub risk_management: RiskManagement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyRules {
    pub entry: ConditionGroup,
    pub exit: ConditionGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionGroup {
    pub conditions: Vec<Condition>,
    pub logic: LogicOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    pub indicator: String,
    pub operator: Operator,
    pub value: f64,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogicOperator {
    And,
    Or,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operator {
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = "==")]
    Equal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSizing {
    pub sizing_type: PositionSizingType,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionSizingType {
    FixedPercent,
    FixedAmount,
    KellyFraction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskManagement {
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub max_open_positions: Option<i32>,
    pub trailing_stop: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestResult {
    pub id: String,
    pub strategy_name: String,
    pub strategy_config: StrategyConfig,
    pub token_mint: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_capital: f64,
    pub final_capital: f64,
    pub total_return_percent: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown_percent: Option<f64>,
    pub win_rate: Option<f64>,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub avg_win: Option<f64>,
    pub avg_loss: Option<f64>,
    pub trades: Vec<BacktestTrade>,
    pub equity_curve: Vec<EquityPoint>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestTrade {
    pub id: String,
    pub backtest_id: String,
    pub token_mint: String,
    pub entry_time: DateTime<Utc>,
    pub entry_price: f64,
    pub exit_time: Option<DateTime<Utc>>,
    pub exit_price: Option<f64>,
    pub position_size: f64,
    pub pnl: Option<f64>,
    pub pnl_percent: Option<f64>,
    pub exit_reason: Option<ExitReason>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitReason {
    StopLoss,
    TakeProfit,
    Signal,
    EndOfTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity: f64,
    pub drawdown_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BacktestSummary {
    pub id: String,
    pub strategy_name: String,
    pub token_mint: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_return_percent: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown_percent: Option<f64>,
    pub total_trades: i32,
    pub win_rate: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyComparison {
    pub strategies: Vec<BacktestSummary>,
    pub best_return: String,
    pub best_sharpe: String,
    pub lowest_drawdown: String,
    pub highest_win_rate: String,
}

// ============================================================================
// Technical Indicators (for backtesting)
// ============================================================================

#[derive(Debug, Clone)]
pub struct IndicatorValues {
    pub rsi: Option<f64>,
    pub macd: Option<MacdValues>,
    pub bollinger: Option<BollingerValues>,
    pub moving_average_20: Option<f64>,
    pub moving_average_50: Option<f64>,
    pub volume_sma: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct MacdValues {
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone)]
pub struct BollingerValues {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("sentiment analysis failed: {0}")]
    SentimentAnalysis(String),

    #[error("prediction failed: {0}")]
    Prediction(String),

    #[error("backtest failed: {0}")]
    Backtest(String),

    #[error("invalid strategy configuration: {0}")]
    InvalidStrategy(String),

    #[error("insufficient data: {0}")]
    InsufficientData(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type AiResult<T> = Result<T, AiError>;
