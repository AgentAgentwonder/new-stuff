use super::storage::HistoricalDataPoint;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualRequest {
    pub symbol: String,
    pub quantity: f64,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualPoint {
    pub timestamp: i64,
    pub price: f64,
    pub value: f64,
    pub percent_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    pub symbol: String,
    pub quantity: f64,
    pub start_price: f64,
    pub end_price: f64,
    pub start_value: f64,
    pub final_value: f64,
    pub absolute_return: f64,
    pub percent_return: f64,
    pub max_drawdown: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub points: Vec<CounterfactualPoint>,
}

pub fn compute_hold_counterfactual(
    request: CounterfactualRequest,
    data: &[HistoricalDataPoint],
) -> Option<CounterfactualResult> {
    if data.is_empty() {
        return None;
    }

    let mut points = Vec::new();
    let mut peak_value = f64::MIN;
    let mut max_drawdown = 0.0;
    let mut returns = Vec::new();

    let mut previous_value = None;

    for point in data {
        if point.timestamp < request.start_time || point.timestamp > request.end_time {
            continue;
        }

        let value = point.close * request.quantity;
        let start_value = data
            .iter()
            .find(|p| p.timestamp == request.start_time)
            .unwrap_or(point)
            .close
            * request.quantity;

        let percent_change = if start_value != 0.0 {
            ((value - start_value) / start_value) * 100.0
        } else {
            0.0
        };

        if value > peak_value {
            peak_value = value;
        }

        let drawdown = if peak_value > 0.0 {
            ((peak_value - value) / peak_value) * 100.0
        } else {
            0.0
        };

        if drawdown > max_drawdown {
            max_drawdown = drawdown;
        }

        if let Some(prev_value) = previous_value {
            if prev_value > 0.0 {
                returns.push((value - prev_value) / prev_value);
            }
        }
        previous_value = Some(value);

        points.push(CounterfactualPoint {
            timestamp: point.timestamp,
            price: point.close,
            value,
            percent_change,
        });
    }

    if points.is_empty() {
        return None;
    }

    let start_price = points.first()?.price;
    let end_price = points.last()?.price;
    let start_value = points.first()?.value;
    let final_value = points.last()?.value;
    let absolute_return = final_value - start_value;
    let percent_return = if start_value != 0.0 {
        (absolute_return / start_value) * 100.0
    } else {
        0.0
    };

    let duration_days = ((request.end_time - request.start_time) as f64 / 86_400.0).max(0.0);
    let annualized_return = if duration_days > 0.0 {
        ((final_value / start_value).powf(365.25 / duration_days) - 1.0) * 100.0
    } else {
        0.0
    };

    let mean_return = if !returns.is_empty() {
        returns.iter().copied().sum::<f64>() / returns.len() as f64
    } else {
        0.0
    };

    let variance = if returns.len() > 1 {
        returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / (returns.len() - 1) as f64
    } else {
        0.0
    };

    let volatility = variance.sqrt() * (returns.len() as f64).sqrt();

    Some(CounterfactualResult {
        symbol: request.symbol,
        quantity: request.quantity,
        start_price,
        end_price,
        start_value,
        final_value,
        absolute_return,
        percent_return,
        max_drawdown,
        annualized_return,
        volatility,
        points,
    })
}
