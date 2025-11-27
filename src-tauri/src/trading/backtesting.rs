use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub strategy_id: String,
    pub symbol: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub start_date: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub end_date: DateTime<Utc>,
    pub initial_capital: f64,
    pub commission_rate: f64,  // percentage
    pub slippage_rate: f64,    // percentage
    pub data_interval: String, // 1m, 5m, 15m, 1h, 4h, 1d
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub side: String, // buy or sell
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub value: f64,
    pub commission: f64,
    pub slippage: f64,
    pub signal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub total_return: f64,
    pub total_return_percent: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub max_drawdown_percent: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub average_trade_duration: i64, // milliseconds
    pub exposure_time: f64,          // percentage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub equity: f64,
    pub drawdown: f64,
    pub drawdown_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub id: String,
    pub config: BacktestConfig,
    pub metrics: BacktestMetrics,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<EquityPoint>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub started_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub completed_at: DateTime<Utc>,
    pub duration: i64, // milliseconds
}

#[derive(Debug, Clone)]
pub struct HistoricalData {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug)]
pub struct BacktestEngine {
    config: BacktestConfig,
    equity: f64,
    peak_equity: f64,
    current_position: Option<Position>,
    trades: Vec<Trade>,
    equity_curve: Vec<EquityPoint>,
}

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    entry_time: DateTime<Utc>,
}

impl BacktestEngine {
    pub fn new(config: BacktestConfig) -> Self {
        let initial_capital = config.initial_capital;
        Self {
            config,
            equity: initial_capital,
            peak_equity: initial_capital,
            current_position: None,
            trades: Vec::new(),
            equity_curve: Vec::new(),
        }
    }

    pub fn execute_buy(&mut self, timestamp: DateTime<Utc>, price: f64, signal: Option<String>) {
        if self.current_position.is_some() {
            return; // Already in a position
        }

        // Apply slippage
        let execution_price = price * (1.0 + self.config.slippage_rate / 100.0);

        // Calculate quantity based on available capital
        let gross_value = self.equity * 0.95; // Use 95% of capital
        let commission = gross_value * (self.config.commission_rate / 100.0);
        let net_value = gross_value - commission;
        let quantity = net_value / execution_price;
        let slippage_cost = quantity * (execution_price - price);

        self.current_position = Some(Position {
            symbol: self.config.symbol.clone(),
            quantity,
            entry_price: execution_price,
            entry_time: timestamp,
        });

        self.trades.push(Trade {
            timestamp,
            side: "buy".to_string(),
            symbol: self.config.symbol.clone(),
            price: execution_price,
            quantity,
            value: quantity * execution_price,
            commission,
            slippage: slippage_cost,
            signal,
        });

        self.equity -= quantity * execution_price + commission;
    }

    pub fn execute_sell(&mut self, timestamp: DateTime<Utc>, price: f64, signal: Option<String>) {
        let position = match &self.current_position {
            Some(p) => p.clone(),
            None => return, // No position to close
        };

        // Apply slippage (negative for sells)
        let execution_price = price * (1.0 - self.config.slippage_rate / 100.0);

        let gross_proceeds = position.quantity * execution_price;
        let commission = gross_proceeds * (self.config.commission_rate / 100.0);
        let net_proceeds = gross_proceeds - commission;
        let slippage_cost = position.quantity * (price - execution_price);

        self.trades.push(Trade {
            timestamp,
            side: "sell".to_string(),
            symbol: self.config.symbol.clone(),
            price: execution_price,
            quantity: position.quantity,
            value: gross_proceeds,
            commission,
            slippage: slippage_cost,
            signal,
        });

        self.equity += net_proceeds;
        self.current_position = None;
    }

    pub fn update_equity_curve(&mut self, timestamp: DateTime<Utc>, current_price: f64) {
        let position_value = self
            .current_position
            .as_ref()
            .map(|p| p.quantity * current_price)
            .unwrap_or(0.0);

        let total_equity = self.equity + position_value;

        if total_equity > self.peak_equity {
            self.peak_equity = total_equity;
        }

        let drawdown = self.peak_equity - total_equity;
        let drawdown_percent = if self.peak_equity > 0.0 {
            (drawdown / self.peak_equity) * 100.0
        } else {
            0.0
        };

        self.equity_curve.push(EquityPoint {
            timestamp,
            equity: total_equity,
            drawdown,
            drawdown_percent,
        });
    }

    pub fn calculate_metrics(&self) -> BacktestMetrics {
        let initial_capital = self.config.initial_capital;
        let final_equity = self
            .equity_curve
            .last()
            .map(|e| e.equity)
            .unwrap_or(initial_capital);

        let total_return = final_equity - initial_capital;
        let total_return_percent = (total_return / initial_capital) * 100.0;

        // Calculate annualized return
        let duration_days = (self.config.end_date - self.config.start_date).num_days() as f64;
        let years = duration_days / 365.25;
        let annualized_return = if years > 0.0 {
            (((final_equity / initial_capital).powf(1.0 / years)) - 1.0) * 100.0
        } else {
            0.0
        };

        // Calculate trade statistics
        let winning_trades: Vec<&Trade> = self
            .trades
            .windows(2)
            .filter_map(|w| {
                if w[0].side == "buy" && w[1].side == "sell" {
                    let pnl = (w[1].price - w[0].price) * w[0].quantity
                        - w[0].commission
                        - w[1].commission;
                    if pnl > 0.0 {
                        Some(&w[1])
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let losing_trades: Vec<&Trade> = self
            .trades
            .windows(2)
            .filter_map(|w| {
                if w[0].side == "buy" && w[1].side == "sell" {
                    let pnl = (w[1].price - w[0].price) * w[0].quantity
                        - w[0].commission
                        - w[1].commission;
                    if pnl < 0.0 {
                        Some(&w[1])
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let total_win = self.trades.windows(2).fold(0.0, |acc, w| {
            if w[0].side == "buy" && w[1].side == "sell" {
                let pnl =
                    (w[1].price - w[0].price) * w[0].quantity - w[0].commission - w[1].commission;
                if pnl > 0.0 {
                    acc + pnl
                } else {
                    acc
                }
            } else {
                acc
            }
        });

        let total_loss = self.trades.windows(2).fold(0.0, |acc, w| {
            if w[0].side == "buy" && w[1].side == "sell" {
                let pnl =
                    (w[1].price - w[0].price) * w[0].quantity - w[0].commission - w[1].commission;
                if pnl < 0.0 {
                    acc + pnl.abs()
                } else {
                    acc
                }
            } else {
                acc
            }
        });

        let round_trips = self.trades.len() / 2;
        let win_rate = if round_trips > 0 {
            (winning_trades.len() as f64 / round_trips as f64) * 100.0
        } else {
            0.0
        };

        let average_win = if !winning_trades.is_empty() {
            total_win / winning_trades.len() as f64
        } else {
            0.0
        };

        let average_loss = if !losing_trades.is_empty() {
            total_loss / losing_trades.len() as f64
        } else {
            0.0
        };

        let profit_factor = if total_loss > 0.0 {
            total_win / total_loss
        } else {
            if total_win > 0.0 {
                f64::INFINITY
            } else {
                0.0
            }
        };

        // Calculate Sharpe ratio (simplified)
        let returns: Vec<f64> = self
            .equity_curve
            .windows(2)
            .map(|w| (w[1].equity - w[0].equity) / w[0].equity)
            .collect();

        let mean_return = if !returns.is_empty() {
            returns.iter().sum::<f64>() / returns.len() as f64
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

        let std_dev = variance.sqrt();
        let sharpe_ratio = if std_dev > 0.0 {
            (mean_return * 252.0_f64.sqrt()) / std_dev // Annualized
        } else {
            0.0
        };

        // Calculate Sortino ratio (only downside deviation)
        let downside_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();
        let downside_variance = if downside_returns.len() > 1 {
            downside_returns.iter().map(|r| r.powi(2)).sum::<f64>()
                / (downside_returns.len() - 1) as f64
        } else {
            0.0
        };

        let downside_dev = downside_variance.sqrt();
        let sortino_ratio = if downside_dev > 0.0 {
            (mean_return * 252.0_f64.sqrt()) / downside_dev
        } else {
            0.0
        };

        let max_drawdown = self
            .equity_curve
            .iter()
            .map(|e| e.drawdown)
            .fold(0.0, f64::max);
        let max_drawdown_percent = self
            .equity_curve
            .iter()
            .map(|e| e.drawdown_percent)
            .fold(0.0, f64::max);

        // Calculate average trade duration
        let trade_durations: Vec<i64> = self
            .trades
            .windows(2)
            .filter_map(|w| {
                if w[0].side == "buy" && w[1].side == "sell" {
                    Some((w[1].timestamp - w[0].timestamp).num_milliseconds())
                } else {
                    None
                }
            })
            .collect();

        let average_trade_duration = if !trade_durations.is_empty() {
            trade_durations.iter().sum::<i64>() / trade_durations.len() as i64
        } else {
            0
        };

        // Calculate exposure time
        let total_time = (self.config.end_date - self.config.start_date).num_milliseconds();
        let time_in_market: i64 = trade_durations.iter().sum();
        let exposure_time = if total_time > 0 {
            (time_in_market as f64 / total_time as f64) * 100.0
        } else {
            0.0
        };

        BacktestMetrics {
            total_return,
            total_return_percent,
            annualized_return,
            sharpe_ratio,
            sortino_ratio,
            max_drawdown,
            max_drawdown_percent,
            win_rate,
            profit_factor,
            total_trades: round_trips as u32,
            winning_trades: winning_trades.len() as u32,
            losing_trades: losing_trades.len() as u32,
            average_win,
            average_loss,
            largest_win: if !winning_trades.is_empty() {
                total_win / winning_trades.len() as f64
            } else {
                0.0
            },
            largest_loss: if !losing_trades.is_empty() {
                total_loss / losing_trades.len() as f64
            } else {
                0.0
            },
            average_trade_duration,
            exposure_time,
        }
    }

    pub fn finalize(self) -> BacktestResult {
        let metrics = self.calculate_metrics();
        let completed_at = Utc::now();
        let duration = (completed_at - self.config.start_date).num_milliseconds();

        BacktestResult {
            id: Uuid::new_v4().to_string(),
            config: self.config.clone(),
            metrics,
            trades: self.trades,
            equity_curve: self.equity_curve,
            started_at: self.config.start_date,
            completed_at,
            duration,
        }
    }
}

// Generate mock historical data for testing
pub fn generate_mock_historical_data(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    interval_minutes: i64,
    initial_price: f64,
) -> Vec<HistoricalData> {
    let mut data = Vec::new();
    let mut current_time = start;
    let mut price = initial_price;
    let volatility = 0.02; // 2% volatility

    while current_time <= end {
        let change = (rand::random::<f64>() - 0.5) * volatility;
        price *= 1.0 + change;

        let high = price * (1.0 + rand::random::<f64>() * 0.01);
        let low = price * (1.0 - rand::random::<f64>() * 0.01);
        let volume = 1000000.0 + rand::random::<f64>() * 5000000.0;

        data.push(HistoricalData {
            timestamp: current_time,
            open: price,
            high,
            low,
            close: price,
            volume,
        });

        current_time = current_time + chrono::Duration::minutes(interval_minutes);
    }

    data
}

#[tauri::command]
pub async fn backtest_run(config: BacktestConfig) -> Result<BacktestResult, String> {
    // In a real implementation, fetch historical data from database or API
    let interval_minutes = match config.data_interval.as_str() {
        "1m" => 1,
        "5m" => 5,
        "15m" => 15,
        "1h" => 60,
        "4h" => 240,
        "1d" => 1440,
        _ => 60,
    };

    let historical_data = generate_mock_historical_data(
        config.start_date,
        config.end_date,
        interval_minutes,
        100.0, // Initial price
    );

    let mut engine = BacktestEngine::new(config);

    // Simple strategy for demonstration: Moving average crossover
    let short_period = 10;
    let long_period = 30;

    for (i, data) in historical_data.iter().enumerate() {
        if i < long_period {
            engine.update_equity_curve(data.timestamp, data.close);
            continue;
        }

        // Calculate moving averages
        let short_ma: f64 = historical_data[i - short_period..i]
            .iter()
            .map(|d| d.close)
            .sum::<f64>()
            / short_period as f64;

        let long_ma: f64 = historical_data[i - long_period..i]
            .iter()
            .map(|d| d.close)
            .sum::<f64>()
            / long_period as f64;

        let prev_short_ma: f64 = historical_data[i - short_period - 1..i - 1]
            .iter()
            .map(|d| d.close)
            .sum::<f64>()
            / short_period as f64;

        let prev_long_ma: f64 = historical_data[i - long_period - 1..i - 1]
            .iter()
            .map(|d| d.close)
            .sum::<f64>()
            / long_period as f64;

        // Buy signal: short MA crosses above long MA
        if prev_short_ma <= prev_long_ma && short_ma > long_ma {
            engine.execute_buy(data.timestamp, data.close, Some("MA_CROSS_UP".to_string()));
        }

        // Sell signal: short MA crosses below long MA
        if prev_short_ma >= prev_long_ma && short_ma < long_ma {
            engine.execute_sell(
                data.timestamp,
                data.close,
                Some("MA_CROSS_DOWN".to_string()),
            );
        }

        engine.update_equity_curve(data.timestamp, data.close);
    }

    // Close any open positions at the end
    if engine.current_position.is_some() {
        let last_data = historical_data.last().unwrap();
        engine.execute_sell(
            last_data.timestamp,
            last_data.close,
            Some("END_OF_PERIOD".to_string()),
        );
    }

    Ok(engine.finalize())
}

// Simple random number generation for mock data
mod rand {
    use std::cell::Cell;

    thread_local! {
        static RNG_STATE: Cell<u64> = Cell::new(123456789);
    }

    pub fn random<T: From<f64>>() -> T {
        RNG_STATE.with(|state| {
            let mut x = state.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            state.set(x);
            T::from((x as f64) / (u64::MAX as f64))
        })
    }
}
