use super::storage::HistoricalDataPoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioHolding {
    pub symbol: String,
    pub quantity: f64,
    pub average_entry_price: f64,
    pub first_purchase_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub timestamp: i64,
    pub holdings: Vec<PortfolioHolding>,
    pub cash_balance: f64,
    pub total_value: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationAction {
    pub timestamp: i64,
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionType {
    Buy {
        symbol: String,
        quantity: f64,
        price: f64,
    },
    Sell {
        symbol: String,
        quantity: f64,
        price: f64,
    },
    Rebalance {
        target_allocations: HashMap<String, f64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub start_time: i64,
    pub end_time: i64,
    pub initial_capital: f64,
    pub commission_rate: f64,
    pub slippage_rate: f64,
    pub actions: Vec<SimulationAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub config: SimulationConfig,
    pub snapshots: Vec<PortfolioSnapshot>,
    pub final_value: f64,
    pub total_return: f64,
    pub total_return_percent: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub num_trades: u32,
    pub total_fees: f64,
}

#[derive(Debug)]
pub struct PortfolioSimulator {
    holdings: HashMap<String, PortfolioHolding>,
    cash_balance: f64,
    realized_pnl: f64,
    total_fees: f64,
    num_trades: u32,
    commission_rate: f64,
    slippage_rate: f64,
    snapshots: Vec<PortfolioSnapshot>,
    peak_value: f64,
}

impl PortfolioSimulator {
    pub fn new(initial_capital: f64, commission_rate: f64, slippage_rate: f64) -> Self {
        Self {
            holdings: HashMap::new(),
            cash_balance: initial_capital,
            realized_pnl: 0.0,
            total_fees: 0.0,
            num_trades: 0,
            commission_rate,
            slippage_rate,
            snapshots: Vec::new(),
            peak_value: initial_capital,
        }
    }

    pub fn import_holdings(&mut self, holdings: Vec<PortfolioHolding>) {
        for holding in holdings {
            self.holdings.insert(holding.symbol.clone(), holding);
        }
    }

    pub fn execute_buy(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
        timestamp: i64,
    ) -> Result<(), String> {
        // Apply slippage (negative for buys)
        let execution_price = price * (1.0 + self.slippage_rate);
        let cost = quantity * execution_price;
        let commission = cost * self.commission_rate;
        let total_cost = cost + commission;

        if total_cost > self.cash_balance {
            return Err("Insufficient funds".to_string());
        }

        self.cash_balance -= total_cost;
        self.total_fees += commission;
        self.num_trades += 1;

        // Update or create holding
        if let Some(holding) = self.holdings.get_mut(symbol) {
            let total_quantity = holding.quantity + quantity;
            let total_cost = (holding.quantity * holding.average_entry_price) + cost;
            holding.average_entry_price = total_cost / total_quantity;
            holding.quantity = total_quantity;
        } else {
            self.holdings.insert(
                symbol.to_string(),
                PortfolioHolding {
                    symbol: symbol.to_string(),
                    quantity,
                    average_entry_price: execution_price,
                    first_purchase_time: timestamp,
                },
            );
        }

        Ok(())
    }

    pub fn execute_sell(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
        _timestamp: i64,
    ) -> Result<(), String> {
        let holding = self
            .holdings
            .get_mut(symbol)
            .ok_or("No holding found for symbol")?;

        if holding.quantity < quantity {
            return Err("Insufficient quantity to sell".to_string());
        }

        // Apply slippage (positive for sells)
        let execution_price = price * (1.0 - self.slippage_rate);
        let proceeds = quantity * execution_price;
        let commission = proceeds * self.commission_rate;
        let net_proceeds = proceeds - commission;

        self.cash_balance += net_proceeds;
        self.total_fees += commission;
        self.num_trades += 1;

        // Calculate realized PnL
        let cost_basis = quantity * holding.average_entry_price;
        let pnl = proceeds - cost_basis - commission;
        self.realized_pnl += pnl;

        // Update holding
        holding.quantity -= quantity;
        if holding.quantity <= 0.0 {
            self.holdings.remove(symbol);
        }

        Ok(())
    }

    pub fn take_snapshot(&mut self, timestamp: i64, current_prices: &HashMap<String, f64>) {
        let mut total_value = self.cash_balance;
        let mut unrealized_pnl = 0.0;

        let holdings: Vec<PortfolioHolding> = self
            .holdings
            .values()
            .map(|h| {
                if let Some(&price) = current_prices.get(&h.symbol) {
                    let market_value = h.quantity * price;
                    let cost_basis = h.quantity * h.average_entry_price;
                    total_value += market_value;
                    unrealized_pnl += market_value - cost_basis;
                }
                h.clone()
            })
            .collect();

        if total_value > self.peak_value {
            self.peak_value = total_value;
        }

        self.snapshots.push(PortfolioSnapshot {
            timestamp,
            holdings,
            cash_balance: self.cash_balance,
            total_value,
            unrealized_pnl,
            realized_pnl: self.realized_pnl,
        });
    }

    pub fn metrics(&self, initial_capital: f64) -> SimulationMetrics {
        let final_value = self
            .snapshots
            .last()
            .map(|s| s.total_value)
            .unwrap_or(initial_capital);

        let total_return = final_value - initial_capital;
        let total_return_percent = (total_return / initial_capital) * 100.0;

        // Calculate max drawdown
        let mut max_drawdown = 0.0_f64;
        let mut peak = initial_capital;

        for snapshot in &self.snapshots {
            if snapshot.total_value > peak {
                peak = snapshot.total_value;
            }
            let drawdown = ((peak - snapshot.total_value) / peak) * 100.0;
            max_drawdown = max_drawdown.max(drawdown);
        }

        // Calculate Sharpe ratio (simplified)
        let returns: Vec<f64> = self
            .snapshots
            .windows(2)
            .map(|w| (w[1].total_value - w[0].total_value) / w[0].total_value)
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
            (mean_return * 252.0_f64.sqrt()) / std_dev
        } else {
            0.0
        };

        SimulationMetrics {
            final_value,
            total_return,
            total_return_percent,
            max_drawdown,
            sharpe_ratio,
            num_trades: self.num_trades,
            total_fees: self.total_fees,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub final_value: f64,
    pub total_return: f64,
    pub total_return_percent: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub num_trades: u32,
    pub total_fees: f64,
}

pub fn run_simulation(
    config: SimulationConfig,
    historical_data: &HashMap<String, Vec<HistoricalDataPoint>>,
) -> Result<SimulationResult, String> {
    let mut simulator = PortfolioSimulator::new(
        config.initial_capital,
        config.commission_rate,
        config.slippage_rate,
    );

    // Build a timeline of all events
    let mut timeline: Vec<(i64, SimulationAction)> = config
        .actions
        .clone()
        .into_iter()
        .map(|action| (action.timestamp, action))
        .collect();

    timeline.sort_by_key(|(timestamp, _)| *timestamp);

    // Get all unique timestamps from historical data
    let mut data_timestamps = std::collections::BTreeSet::new();
    for data_points in historical_data.values() {
        for point in data_points {
            if point.timestamp >= config.start_time && point.timestamp <= config.end_time {
                data_timestamps.insert(point.timestamp);
            }
        }
    }

    let mut action_idx = 0;

    for &timestamp in &data_timestamps {
        // Build current price map
        let mut current_prices = HashMap::new();
        for (symbol, data_points) in historical_data {
            if let Some(point) = data_points.iter().find(|p| p.timestamp == timestamp) {
                current_prices.insert(symbol.clone(), point.close);
            }
        }

        // Execute any pending actions at this timestamp
        while action_idx < timeline.len() && timeline[action_idx].0 <= timestamp {
            let (_, action) = &timeline[action_idx];

            match &action.action_type {
                ActionType::Buy {
                    symbol,
                    quantity,
                    price,
                } => {
                    let _ = simulator.execute_buy(symbol, *quantity, *price, timestamp);
                }
                ActionType::Sell {
                    symbol,
                    quantity,
                    price,
                } => {
                    let _ = simulator.execute_sell(symbol, *quantity, *price, timestamp);
                }
                ActionType::Rebalance { target_allocations } => {
                    // Calculate current portfolio value
                    let mut total_value = simulator.cash_balance;
                    for (symbol, holding) in &simulator.holdings {
                        if let Some(&price) = current_prices.get(symbol) {
                            total_value += holding.quantity * price;
                        }
                    }

                    // Rebalance to target allocations
                    for (symbol, &target_percent) in target_allocations {
                        let target_value = total_value * target_percent;

                        if let Some(&price) = current_prices.get(symbol) {
                            let current_value = simulator
                                .holdings
                                .get(symbol)
                                .map(|h| h.quantity * price)
                                .unwrap_or(0.0);

                            let diff_value = target_value - current_value;

                            if diff_value > 0.0 {
                                // Buy more
                                let quantity = diff_value / price;
                                let _ = simulator.execute_buy(symbol, quantity, price, timestamp);
                            } else if diff_value < 0.0 {
                                // Sell some
                                let quantity = diff_value.abs() / price;
                                let _ = simulator.execute_sell(symbol, quantity, price, timestamp);
                            }
                        }
                    }
                }
            }

            action_idx += 1;
        }

        // Take snapshot at this timestamp
        simulator.take_snapshot(timestamp, &current_prices);
    }

    let metrics = simulator.metrics(config.initial_capital);
    let snapshots = simulator.snapshots.clone();

    Ok(SimulationResult {
        config: config.clone(),
        snapshots,
        final_value: metrics.final_value,
        total_return: metrics.total_return,
        total_return_percent: metrics.total_return_percent,
        max_drawdown: metrics.max_drawdown,
        sharpe_ratio: metrics.sharpe_ratio,
        num_trades: metrics.num_trades,
        total_fees: metrics.total_fees,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data(price_start: f64, price_end: f64) -> Vec<HistoricalDataPoint> {
        vec![
            HistoricalDataPoint {
                timestamp: 1,
                open: price_start,
                high: price_start,
                low: price_start,
                close: price_start,
                volume: 1000.0,
            },
            HistoricalDataPoint {
                timestamp: 2,
                open: price_end,
                high: price_end,
                low: price_end,
                close: price_end,
                volume: 1200.0,
            },
        ]
    }

    #[test]
    fn test_run_simulation_buy_and_hold() {
        let config = SimulationConfig {
            start_time: 1,
            end_time: 2,
            initial_capital: 1000.0,
            commission_rate: 0.0,
            slippage_rate: 0.0,
            actions: vec![SimulationAction {
                timestamp: 1,
                action_type: ActionType::Buy {
                    symbol: "SOL".to_string(),
                    quantity: 5.0,
                    price: 100.0,
                },
            }],
        };

        let mut datasets = HashMap::new();
        datasets.insert("SOL".to_string(), sample_data(100.0, 120.0));

        let result = run_simulation(config, &datasets).expect("simulation should succeed");

        assert_eq!(result.num_trades, 1);
        assert!(result.total_return > 0.0);
        assert!(result.final_value > result.config.initial_capital);
    }

    #[test]
    fn test_run_simulation_no_actions() {
        let config = SimulationConfig {
            start_time: 1,
            end_time: 2,
            initial_capital: 5000.0,
            commission_rate: 0.0,
            slippage_rate: 0.0,
            actions: vec![],
        };

        let mut datasets = HashMap::new();
        datasets.insert("SOL".to_string(), sample_data(100.0, 120.0));

        let result = run_simulation(config, &datasets).expect("simulation should succeed");

        assert_eq!(result.final_value, 5000.0);
        assert_eq!(result.num_trades, 0);
        assert_eq!(result.total_fees, 0.0);
    }
}
