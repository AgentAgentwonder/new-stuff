use chrono::{Datelike, NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    Above,
    Below,
    PercentChange,
    VolumeSpike,
    WhaleTransaction,
    TimeWindow,
    MarketCap,
    Liquidity,
    TradingVolume,
    PriceRange,
    Volatility,
    TrendChange,
}

impl ConditionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionType::Above => "above",
            ConditionType::Below => "below",
            ConditionType::PercentChange => "percent_change",
            ConditionType::VolumeSpike => "volume_spike",
            ConditionType::WhaleTransaction => "whale_transaction",
            ConditionType::TimeWindow => "time_window",
            ConditionType::MarketCap => "market_cap",
            ConditionType::Liquidity => "liquidity",
            ConditionType::TradingVolume => "trading_volume",
            ConditionType::PriceRange => "price_range",
            ConditionType::Volatility => "volatility",
            ConditionType::TrendChange => "trend_change",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "above" | "price_above" => Some(ConditionType::Above),
            "below" | "price_below" => Some(ConditionType::Below),
            "percent_change" => Some(ConditionType::PercentChange),
            "volume_spike" => Some(ConditionType::VolumeSpike),
            "whale_transaction" | "whale_transfer" => Some(ConditionType::WhaleTransaction),
            "time_window" => Some(ConditionType::TimeWindow),
            "market_cap" => Some(ConditionType::MarketCap),
            "liquidity" => Some(ConditionType::Liquidity),
            "trading_volume" | "volume" => Some(ConditionType::TradingVolume),
            "price_range" => Some(ConditionType::PriceRange),
            "volatility" => Some(ConditionType::Volatility),
            "trend_change" | "momentum_shift" => Some(ConditionType::TrendChange),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConditionParameters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeframe_minutes: Option<i32>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whale_threshold_usd: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub days_of_week: Option<Vec<u8>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_operator: Option<ComparisonOperator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    pub condition_type: ConditionType,

    #[serde(default)]
    pub parameters: ConditionParameters,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Greater,
    GreaterOrEqual,
    Less,
    LessOrEqual,
    Equal,
    Between,
}

impl Default for ComparisonOperator {
    fn default() -> Self {
        ComparisonOperator::Greater
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketData {
    pub symbol: String,
    pub current_price: f64,

    #[serde(default)]
    pub price_24h_ago: Option<f64>,

    #[serde(default)]
    pub volume_24h: Option<f64>,

    #[serde(default)]
    pub market_cap: Option<f64>,

    #[serde(default)]
    pub liquidity: Option<f64>,

    #[serde(default)]
    pub volatility: Option<f64>,

    #[serde(default)]
    pub price_change_percentage: Option<f64>,

    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WhaleActivity {
    pub transaction_signature: String,
    pub wallet_address: String,
    pub token_mint: String,
    pub amount: f64,
    pub usd_value: f64,
    pub transaction_type: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionEvaluationResult {
    pub condition_id: String,
    pub met: bool,
    pub message: String,
    pub confidence: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl Condition {
    pub fn condition_id(&self) -> String {
        self.id
            .clone()
            .unwrap_or_else(|| self.condition_type.as_str().to_string())
    }

    pub fn evaluate(
        &self,
        market_data: &MarketData,
        whale_activity: &Option<WhaleActivity>,
    ) -> ConditionEvaluationResult {
        match self.condition_type {
            ConditionType::Above => self.evaluate_price_above(market_data),
            ConditionType::Below => self.evaluate_price_below(market_data),
            ConditionType::PercentChange => self.evaluate_percent_change(market_data),
            ConditionType::VolumeSpike => self.evaluate_volume_spike(market_data),
            ConditionType::WhaleTransaction => self.evaluate_whale_transaction(whale_activity),
            ConditionType::TimeWindow => self.evaluate_time_window(),
            ConditionType::MarketCap => self.evaluate_market_cap(market_data),
            ConditionType::Liquidity => self.evaluate_liquidity(market_data),
            ConditionType::TradingVolume => self.evaluate_trading_volume(market_data),
            ConditionType::PriceRange => self.evaluate_price_range(market_data),
            ConditionType::Volatility => self.evaluate_volatility(market_data),
            ConditionType::TrendChange => self.evaluate_trend_change(market_data),
        }
    }

    fn evaluate_price_above(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        let threshold = self.parameters.threshold.unwrap_or(0.0);
        let met = market_data.current_price > threshold;

        ConditionEvaluationResult {
            condition_id: self.condition_id(),
            met,
            message: format!(
                "Price ${:.6} {} threshold ${:.6}",
                market_data.current_price,
                if met { "above" } else { "not above" },
                threshold
            ),
            confidence: 1.0,
            data: Some(serde_json::json!({
                "currentPrice": market_data.current_price,
                "threshold": threshold,
            })),
        }
    }

    fn evaluate_price_below(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        let threshold = self.parameters.threshold.unwrap_or(f64::MAX);
        let met = market_data.current_price < threshold;

        ConditionEvaluationResult {
            condition_id: self.condition_id(),
            met,
            message: format!(
                "Price ${:.6} {} threshold ${:.6}",
                market_data.current_price,
                if met { "below" } else { "not below" },
                threshold
            ),
            confidence: 1.0,
            data: Some(serde_json::json!({
                "currentPrice": market_data.current_price,
                "threshold": threshold,
            })),
        }
    }

    fn evaluate_percent_change(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        if let Some(price_24h) = market_data.price_24h_ago {
            let threshold = self.parameters.threshold.unwrap_or(0.0);
            let percent_change = ((market_data.current_price - price_24h) / price_24h) * 100.0;
            let met = percent_change.abs() >= threshold;

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "Price change {:.2}% {} threshold {:.2}%",
                    percent_change,
                    if met { "exceeds" } else { "below" },
                    threshold
                ),
                confidence: 1.0,
                data: Some(serde_json::json!({
                    "percentChange": percent_change,
                    "threshold": threshold,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "Insufficient price history".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }

    fn evaluate_volume_spike(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        if let Some(volume) = market_data.volume_24h {
            let threshold = self.parameters.threshold.unwrap_or(0.0);
            let met = volume >= threshold;

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "Volume ${:.0} {} threshold ${:.0}",
                    volume,
                    if met { "exceeds" } else { "below" },
                    threshold
                ),
                confidence: 1.0,
                data: Some(serde_json::json!({
                    "volume": volume,
                    "threshold": threshold,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "Volume data unavailable".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }

    fn evaluate_whale_transaction(
        &self,
        whale_activity: &Option<WhaleActivity>,
    ) -> ConditionEvaluationResult {
        if let Some(activity) = whale_activity {
            let threshold = self.parameters.whale_threshold_usd.unwrap_or(100_000.0);
            let met = activity.usd_value >= threshold;

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "Whale transaction ${:.2} {} threshold ${:.2}",
                    activity.usd_value,
                    if met { "exceeds" } else { "below" },
                    threshold
                ),
                confidence: 1.0,
                data: Some(serde_json::json!({
                    "walletAddress": activity.wallet_address,
                    "usdValue": activity.usd_value,
                    "transactionType": activity.transaction_type,
                    "threshold": threshold,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "No whale activity detected".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }

    fn evaluate_time_window(&self) -> ConditionEvaluationResult {
        let now = Utc::now();
        let current_time = now.time();
        let current_day = now.weekday().number_from_monday() as u8;

        let mut met = true;
        let mut messages = Vec::new();

        if let Some(days) = &self.parameters.days_of_week {
            if !days.contains(&current_day) {
                met = false;
                messages.push(format!("Current day {} not in allowed days", current_day));
            }
        }

        if let (Some(start_str), Some(end_str)) =
            (&self.parameters.start_time, &self.parameters.end_time)
        {
            if let (Ok(start), Ok(end)) = (
                NaiveTime::parse_from_str(start_str, "%H:%M"),
                NaiveTime::parse_from_str(end_str, "%H:%M"),
            ) {
                if current_time < start || current_time > end {
                    met = false;
                    messages.push(format!(
                        "Current time {} outside window {}-{}",
                        current_time.format("%H:%M"),
                        start,
                        end
                    ));
                }
            }
        }

        ConditionEvaluationResult {
            condition_id: self.condition_id(),
            met,
            message: if met {
                "Within time window".to_string()
            } else {
                messages.join("; ")
            },
            confidence: 1.0,
            data: Some(serde_json::json!({
                "currentTime": current_time.format("%H:%M").to_string(),
                "currentDay": current_day,
            })),
        }
    }

    fn evaluate_market_cap(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        if let Some(market_cap) = market_data.market_cap {
            let operator = self
                .parameters
                .comparison_operator
                .as_ref()
                .unwrap_or(&ComparisonOperator::Greater);

            let met = match operator {
                ComparisonOperator::Greater => {
                    market_cap > self.parameters.threshold.unwrap_or(0.0)
                }
                ComparisonOperator::GreaterOrEqual => {
                    market_cap >= self.parameters.threshold.unwrap_or(0.0)
                }
                ComparisonOperator::Less => {
                    market_cap < self.parameters.threshold.unwrap_or(f64::MAX)
                }
                ComparisonOperator::LessOrEqual => {
                    market_cap <= self.parameters.threshold.unwrap_or(f64::MAX)
                }
                ComparisonOperator::Equal => {
                    (market_cap - self.parameters.threshold.unwrap_or(0.0)).abs() < f64::EPSILON
                }
                ComparisonOperator::Between => {
                    let min = self.parameters.min_value.unwrap_or(0.0);
                    let max = self.parameters.max_value.unwrap_or(f64::MAX);
                    market_cap >= min && market_cap <= max
                }
            };

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "Market cap ${:.0} {} condition",
                    market_cap,
                    if met { "meets" } else { "does not meet" }
                ),
                confidence: 1.0,
                data: Some(serde_json::json!({
                    "marketCap": market_cap,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "Market cap data unavailable".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }

    fn evaluate_liquidity(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        if let Some(liquidity) = market_data.liquidity {
            let threshold = self.parameters.threshold.unwrap_or(0.0);
            let met = liquidity >= threshold;

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "Liquidity ${:.0} {} threshold ${:.0}",
                    liquidity,
                    if met { "above" } else { "below" },
                    threshold
                ),
                confidence: 1.0,
                data: Some(serde_json::json!({
                    "liquidity": liquidity,
                    "threshold": threshold,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "Liquidity data unavailable".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }

    fn evaluate_trading_volume(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        if let Some(volume) = market_data.volume_24h {
            let threshold = self.parameters.threshold.unwrap_or(0.0);
            let met = volume >= threshold;

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "24h volume ${:.0} {} threshold ${:.0}",
                    volume,
                    if met { "above" } else { "below" },
                    threshold
                ),
                confidence: 1.0,
                data: Some(serde_json::json!({
                    "volume": volume,
                    "threshold": threshold,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "Volume data unavailable".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }

    fn evaluate_price_range(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        let min = self.parameters.min_value.unwrap_or(0.0);
        let max = self.parameters.max_value.unwrap_or(f64::MAX);
        let met = market_data.current_price >= min && market_data.current_price <= max;

        ConditionEvaluationResult {
            condition_id: self.condition_id(),
            met,
            message: format!(
                "Price ${:.6} {} range ${:.6}-${:.6}",
                market_data.current_price,
                if met { "within" } else { "outside" },
                min,
                max
            ),
            confidence: 1.0,
            data: Some(serde_json::json!({
                "currentPrice": market_data.current_price,
                "minValue": min,
                "maxValue": max,
            })),
        }
    }

    fn evaluate_volatility(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        if let Some(volatility) = market_data.volatility {
            let threshold = self.parameters.threshold.unwrap_or(0.0);
            let met = volatility >= threshold;

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "Volatility {:.2}% {} threshold {:.2}%",
                    volatility,
                    if met { "above" } else { "below" },
                    threshold
                ),
                confidence: 1.0,
                data: Some(serde_json::json!({
                    "volatility": volatility,
                    "threshold": threshold,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "Volatility data unavailable".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }

    fn evaluate_trend_change(&self, market_data: &MarketData) -> ConditionEvaluationResult {
        if let Some(change_pct) = market_data.price_change_percentage {
            let threshold = self.parameters.threshold.unwrap_or(0.0);
            let met = change_pct.abs() >= threshold;

            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met,
                message: format!(
                    "Trend change {:.2}% {} threshold {:.2}%",
                    change_pct,
                    if met { "exceeds" } else { "below" },
                    threshold
                ),
                confidence: 0.85,
                data: Some(serde_json::json!({
                    "changePercentage": change_pct,
                    "threshold": threshold,
                })),
            }
        } else {
            ConditionEvaluationResult {
                condition_id: self.condition_id(),
                met: false,
                message: "Trend data unavailable".to_string(),
                confidence: 0.0,
                data: None,
            }
        }
    }
}
