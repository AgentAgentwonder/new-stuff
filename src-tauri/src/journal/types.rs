use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalEntry {
    pub id: String,
    pub timestamp: i64,
    pub trade_id: Option<String>,
    pub entry_type: EntryType,
    pub strategy_tags: Vec<String>,
    pub emotions: EmotionTracking,
    pub notes: String,
    pub market_conditions: MarketConditions,
    pub confidence_level: f32,
    pub position_size: Option<f32>,
    pub entry_price: Option<f32>,
    pub exit_price: Option<f32>,
    pub outcome: Option<TradeOutcome>,
    pub lessons_learned: Option<String>,
    pub attachments: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    PreTrade,
    InTrade,
    PostTrade,
    Reflection,
    Goal,
    Mistake,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmotionTracking {
    pub primary_emotion: Emotion,
    pub intensity: f32,
    pub secondary_emotions: Vec<Emotion>,
    pub stress_level: f32,
    pub clarity_level: f32,
    pub fomo_level: f32,
    pub revenge_trading: bool,
    pub discipline_score: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Emotion {
    Confident,
    Anxious,
    Excited,
    Fearful,
    Greedy,
    Patient,
    Impatient,
    Calm,
    Stressed,
    Euphoric,
    Regretful,
    Neutral,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MarketConditions {
    pub trend: MarketTrend,
    pub volatility: Volatility,
    pub volume: VolumeLevel,
    pub news_sentiment: f32,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MarketTrend {
    StrongBullish,
    Bullish,
    Neutral,
    Bearish,
    StrongBearish,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Volatility {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VolumeLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TradeOutcome {
    pub pnl: f32,
    pub pnl_percent: f32,
    pub success: bool,
    pub followed_plan: bool,
    pub risk_reward_ratio: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct JournalFilters {
    pub date_range: Option<DateRange>,
    pub entry_types: Option<Vec<EntryType>>,
    pub strategy_tags: Option<Vec<String>>,
    pub emotions: Option<Vec<Emotion>>,
    pub min_confidence: Option<f32>,
    pub max_confidence: Option<f32>,
    pub outcome_success: Option<bool>,
    pub search_query: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DateRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WeeklyReport {
    pub id: String,
    pub week_start: i64,
    pub week_end: i64,
    pub total_entries: usize,
    pub trades_taken: usize,
    pub trades_won: usize,
    pub trades_lost: usize,
    pub win_rate: f32,
    pub total_pnl: f32,
    pub average_confidence: f32,
    pub emotion_breakdown: EmotionBreakdown,
    pub discipline_metrics: DisciplineMetrics,
    pub pattern_insights: Vec<PatternInsight>,
    pub strategy_performance: Vec<StrategyPerformance>,
    pub psychological_insights: PsychologicalInsights,
    pub recommendations: Vec<String>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmotionBreakdown {
    pub emotion_counts: std::collections::HashMap<String, usize>,
    pub average_stress: f32,
    pub average_clarity: f32,
    pub average_fomo: f32,
    pub revenge_trading_instances: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisciplineMetrics {
    pub average_discipline_score: f32,
    pub plan_adherence_rate: f32,
    pub impulsive_trades: usize,
    pub patient_trades: usize,
    pub stop_loss_adherence: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PatternInsight {
    pub pattern_type: String,
    pub description: String,
    pub frequency: usize,
    pub impact_on_performance: f32,
    pub recommendation: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyPerformance {
    pub strategy_tag: String,
    pub trades_count: usize,
    pub win_rate: f32,
    pub average_pnl: f32,
    pub average_confidence: f32,
    pub common_emotions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PsychologicalInsights {
    pub dominant_emotions: Vec<String>,
    pub stress_correlation_with_loss: f32,
    pub confidence_correlation_with_win: f32,
    pub fomo_impact: f32,
    pub best_mental_state: String,
    pub worst_mental_state: String,
    pub cognitive_biases_detected: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BehavioralAnalytics {
    pub total_entries: usize,
    pub consistency_score: f32,
    pub emotional_volatility: f32,
    pub discipline_trend: Vec<DisciplineTrendPoint>,
    pub win_rate_by_emotion: std::collections::HashMap<String, f32>,
    pub best_trading_hours: Vec<usize>,
    pub cognitive_biases: Vec<CognitiveBias>,
    pub growth_indicators: GrowthIndicators,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisciplineTrendPoint {
    pub timestamp: i64,
    pub score: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CognitiveBias {
    pub bias_type: BiasType,
    pub severity: f32,
    pub instances: usize,
    pub description: String,
    pub mitigation_strategy: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BiasType {
    ConfirmationBias,
    AnchoringBias,
    RecencyBias,
    OverconfidenceBias,
    LossAversion,
    GamblersFallacy,
    HerdMentality,
    SunkCostFallacy,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrowthIndicators {
    pub improvement_rate: f32,
    pub consistency_improvement: f32,
    pub emotional_control_improvement: f32,
    pub strategy_refinement_score: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JournalStats {
    pub total_entries: usize,
    pub entries_this_week: usize,
    pub entries_this_month: usize,
    pub total_trades_logged: usize,
    pub average_entries_per_week: f32,
    pub most_used_strategies: Vec<StrategyUsage>,
    pub most_common_emotions: Vec<EmotionUsage>,
    pub overall_discipline_score: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyUsage {
    pub tag: String,
    pub count: usize,
    pub win_rate: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmotionUsage {
    pub emotion: String,
    pub count: usize,
    pub percentage: f32,
}
