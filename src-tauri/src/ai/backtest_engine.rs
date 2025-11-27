// Strategy Backtesting Engine
// Test trading strategies against historical data

use super::types::*;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct BacktestEngine {
    db: SqlitePool,
}

impl BacktestEngine {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Run backtest for a strategy
    pub async fn run_backtest(
        &self,
        strategy: StrategyConfig,
        token_mint: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        initial_capital: f64,
    ) -> AiResult<BacktestResult> {
        log::info!(
            "Running backtest for strategy '{}' on token {}",
            strategy.name,
            token_mint
        );

        // Validate strategy
        self.validate_strategy(&strategy)?;

        // TODO: Implement actual backtesting logic
        // This would involve:
        // 1. Fetching historical price data
        // 2. Simulating trades based on strategy rules
        // 3. Calculating performance metrics
        // 4. Tracking equity curve

        // For now, return mock results
        let backtest_id = Uuid::new_v4().to_string();
        let trades = self.generate_mock_trades(&backtest_id, token_mint, &start_date, &end_date);

        let winning_trades = trades.iter().filter(|t| t.pnl.unwrap_or(0.0) > 0.0).count() as i32;
        let losing_trades = trades.iter().filter(|t| t.pnl.unwrap_or(0.0) < 0.0).count() as i32;
        let total_trades = trades.len() as i32;

        let final_capital = initial_capital * 1.15; // Mock 15% return
        let total_return_percent = ((final_capital - initial_capital) / initial_capital) * 100.0;

        let result = BacktestResult {
            id: backtest_id.clone(),
            strategy_name: strategy.name.clone(),
            strategy_config: strategy,
            token_mint: Some(token_mint.to_string()),
            start_date,
            end_date,
            initial_capital,
            final_capital,
            total_return_percent,
            sharpe_ratio: Some(1.5),
            max_drawdown_percent: Some(-8.5),
            win_rate: Some((winning_trades as f64 / total_trades as f64) * 100.0),
            total_trades,
            winning_trades,
            losing_trades,
            avg_win: Some(5.2),
            avg_loss: Some(-3.1),
            trades: trades.clone(),
            equity_curve: self.generate_equity_curve(&trades, initial_capital),
            created_at: Utc::now(),
        };

        // Save to database
        self.save_backtest_result(&result).await?;

        Ok(result)
    }

    /// Get backtest history
    pub async fn get_backtest_history(
        &self,
        strategy_name: Option<String>,
    ) -> AiResult<Vec<BacktestSummary>> {
        let query = if let Some(name) = strategy_name {
            sqlx::query_as::<_, BacktestSummaryRow>(
                r#"
                SELECT id, strategy_name, token_mint, start_date, end_date,
                       total_return_percent, sharpe_ratio, max_drawdown_percent,
                       total_trades, win_rate, created_at
                FROM backtest_results
                WHERE strategy_name = ?
                ORDER BY created_at DESC
                "#
            )
            .bind(name)
        } else {
            sqlx::query_as::<_, BacktestSummaryRow>(
                r#"
                SELECT id, strategy_name, token_mint, start_date, end_date,
                       total_return_percent, sharpe_ratio, max_drawdown_percent,
                       total_trades, win_rate, created_at
                FROM backtest_results
                ORDER BY created_at DESC
                "#
            )
        };

        let results = query.fetch_all(&self.db).await?;

        Ok(results.into_iter().map(|row| row.into()).collect())
    }

    /// Compare multiple strategies
    pub async fn compare_strategies(
        &self,
        backtest_ids: Vec<String>,
    ) -> AiResult<StrategyComparison> {
        let mut summaries = Vec::new();

        for id in &backtest_ids {
            if let Some(summary) = self.get_backtest_by_id(id).await? {
                summaries.push(summary);
            }
        }

        if summaries.is_empty() {
            return Err(AiError::InsufficientData(
                "No backtest results found".to_string(),
            ));
        }

        // Find best performers
        let best_return = summaries
            .iter()
            .max_by(|a, b| a.total_return_percent.partial_cmp(&b.total_return_percent).unwrap())
            .map(|s| s.id.clone())
            .unwrap_or_default();

        let best_sharpe = summaries
            .iter()
            .filter_map(|s| s.sharpe_ratio.map(|sr| (s, sr)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(s, _)| s.id.clone())
            .unwrap_or_default();

        let lowest_drawdown = summaries
            .iter()
            .filter_map(|s| s.max_drawdown_percent.map(|dd| (s, dd)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) // Max because drawdowns are negative
            .map(|(s, _)| s.id.clone())
            .unwrap_or_default();

        let highest_win_rate = summaries
            .iter()
            .filter_map(|s| s.win_rate.map(|wr| (s, wr)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(s, _)| s.id.clone())
            .unwrap_or_default();

        Ok(StrategyComparison {
            strategies: summaries,
            best_return,
            best_sharpe,
            lowest_drawdown,
            highest_win_rate,
        })
    }

    // Private helper methods

    fn validate_strategy(&self, strategy: &StrategyConfig) -> AiResult<()> {
        if strategy.name.is_empty() {
            return Err(AiError::InvalidStrategy("Strategy name cannot be empty".to_string()));
        }

        if strategy.rules.entry.conditions.is_empty() {
            return Err(AiError::InvalidStrategy("Entry conditions cannot be empty".to_string()));
        }

        if strategy.rules.exit.conditions.is_empty() {
            return Err(AiError::InvalidStrategy("Exit conditions cannot be empty".to_string()));
        }

        Ok(())
    }

    fn generate_mock_trades(
        &self,
        backtest_id: &str,
        token_mint: &str,
        start_date: &DateTime<Utc>,
        _end_date: &DateTime<Utc>,
    ) -> Vec<BacktestTrade> {
        let mut trades = Vec::new();

        for i in 0..10 {
            let entry_time = *start_date + chrono::Duration::days(i * 3);
            let exit_time = entry_time + chrono::Duration::days(2);
            let entry_price = 100.0 + rand::random_range(-10.0..10.0);
            let exit_price = entry_price * (1.0 + rand::random_range(-0.1..0.15));
            let position_size = 1000.0;
            let pnl = (exit_price - entry_price) * (position_size / entry_price);
            let pnl_percent = ((exit_price - entry_price) / entry_price) * 100.0;

            trades.push(BacktestTrade {
                id: Uuid::new_v4().to_string(),
                backtest_id: backtest_id.to_string(),
                token_mint: token_mint.to_string(),
                entry_time,
                entry_price,
                exit_time: Some(exit_time),
                exit_price: Some(exit_price),
                position_size,
                pnl: Some(pnl),
                pnl_percent: Some(pnl_percent),
                exit_reason: if pnl > 0.0 {
                    Some(ExitReason::TakeProfit)
                } else {
                    Some(ExitReason::StopLoss)
                },
            });
        }

        trades
    }

    fn generate_equity_curve(&self, trades: &[BacktestTrade], initial_capital: f64) -> Vec<EquityPoint> {
        let mut curve = Vec::new();
        let mut equity = initial_capital;
        let mut max_equity = equity;

        for trade in trades {
            if let (Some(exit_time), Some(pnl)) = (trade.exit_time, trade.pnl) {
                equity += pnl;
                max_equity = max_equity.max(equity);
                let drawdown_percent = ((equity - max_equity) / max_equity) * 100.0;

                curve.push(EquityPoint {
                    timestamp: exit_time,
                    equity,
                    drawdown_percent,
                });
            }
        }

        curve
    }

    async fn save_backtest_result(&self, result: &BacktestResult) -> AiResult<()> {
        let strategy_json = serde_json::to_string(&result.strategy_config)?;

        sqlx::query(
            r#"
            INSERT INTO backtest_results (
                id, strategy_name, strategy_config, token_mint,
                start_date, end_date, initial_capital, final_capital,
                total_return_percent, sharpe_ratio, max_drawdown_percent,
                win_rate, total_trades, winning_trades, losing_trades,
                avg_win, avg_loss, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&result.id)
        .bind(&result.strategy_name)
        .bind(&strategy_json)
        .bind(&result.token_mint)
        .bind(result.start_date.to_rfc3339())
        .bind(result.end_date.to_rfc3339())
        .bind(result.initial_capital)
        .bind(result.final_capital)
        .bind(result.total_return_percent)
        .bind(result.sharpe_ratio)
        .bind(result.max_drawdown_percent)
        .bind(result.win_rate)
        .bind(result.total_trades)
        .bind(result.winning_trades)
        .bind(result.losing_trades)
        .bind(result.avg_win)
        .bind(result.avg_loss)
        .bind(result.created_at.to_rfc3339())
        .execute(&self.db)
        .await?;

        // Save trades
        for trade in &result.trades {
            sqlx::query(
                r#"
                INSERT INTO backtest_trades (
                    id, backtest_id, token_mint, entry_time, entry_price,
                    exit_time, exit_price, position_size, pnl, pnl_percent, exit_reason
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&trade.id)
            .bind(&trade.backtest_id)
            .bind(&trade.token_mint)
            .bind(trade.entry_time.to_rfc3339())
            .bind(trade.entry_price)
            .bind(trade.exit_time.map(|t| t.to_rfc3339()))
            .bind(trade.exit_price)
            .bind(trade.position_size)
            .bind(trade.pnl)
            .bind(trade.pnl_percent)
            .bind(trade.exit_reason.map(|r| format!("{:?}", r).to_lowercase()))
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    async fn get_backtest_by_id(&self, id: &str) -> AiResult<Option<BacktestSummary>> {
        let row = sqlx::query_as::<_, BacktestSummaryRow>(
            r#"
            SELECT id, strategy_name, token_mint, start_date, end_date,
                   total_return_percent, sharpe_ratio, max_drawdown_percent,
                   total_trades, win_rate, created_at
            FROM backtest_results
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r| r.into()))
    }
}

// Database row types

#[derive(sqlx::FromRow)]
struct BacktestSummaryRow {
    id: String,
    strategy_name: String,
    token_mint: Option<String>,
    start_date: String,
    end_date: String,
    total_return_percent: f64,
    sharpe_ratio: Option<f64>,
    max_drawdown_percent: Option<f64>,
    total_trades: i32,
    win_rate: Option<f64>,
    created_at: String,
}

impl From<BacktestSummaryRow> for BacktestSummary {
    fn from(row: BacktestSummaryRow) -> Self {
        Self {
            id: row.id,
            strategy_name: row.strategy_name,
            token_mint: row.token_mint,
            start_date: DateTime::parse_from_rfc3339(&row.start_date)
                .unwrap()
                .with_timezone(&Utc),
            end_date: DateTime::parse_from_rfc3339(&row.end_date)
                .unwrap()
                .with_timezone(&Utc),
            total_return_percent: row.total_return_percent,
            sharpe_ratio: row.sharpe_ratio,
            max_drawdown_percent: row.max_drawdown_percent,
            total_trades: row.total_trades,
            win_rate: row.win_rate,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&Utc),
        }
    }
}
