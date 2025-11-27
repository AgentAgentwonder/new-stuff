use crate::utils::Rfc3339DateTime;
use chrono::{DateTime, Utc};
use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: String,
    pub wallet_address: String,
    pub token_mint: String,
    pub token_symbol: String,
    pub side: String, // "buy" or "sell"
    pub amount: f64,
    pub price: f64,
    pub total_value: f64,
    pub fee: f64,
    pub tx_signature: String,
    pub timestamp: DateTime<Utc>,
    pub pnl: Option<f64>,
    pub hold_duration_seconds: Option<i64>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Trade {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Trade {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            token_mint: row.try_get("token_mint")?,
            token_symbol: row.try_get("token_symbol")?,
            side: row.try_get("side")?,
            amount: row.try_get("amount")?,
            price: row.try_get("price")?,
            total_value: row.try_get("total_value")?,
            fee: row.try_get("fee")?,
            tx_signature: row.try_get("tx_signature")?,
            timestamp: Rfc3339DateTime::try_from(row.try_get::<String, _>("timestamp")?)?.into(),
            pnl: row.try_get("pnl")?,
            hold_duration_seconds: row.try_get("hold_duration_seconds")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceScore {
    pub id: i64,
    pub wallet_address: String,
    pub score: f64, // 0-100
    pub win_rate: f64,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub net_pnl: f64,
    pub avg_profit_per_trade: f64,
    pub avg_loss_per_trade: f64,
    pub profit_factor: f64, // total_profit / abs(total_loss)
    pub sharpe_ratio: f64,
    pub consistency_score: f64, // Based on variance in returns
    pub avg_hold_duration_seconds: f64,
    pub best_trade_pnl: f64,
    pub worst_trade_pnl: f64,
    pub calculated_at: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for PerformanceScore {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(PerformanceScore {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            score: row.try_get("score")?,
            win_rate: row.try_get("win_rate")?,
            total_trades: row.try_get("total_trades")?,
            winning_trades: row.try_get("winning_trades")?,
            losing_trades: row.try_get("losing_trades")?,
            total_profit: row.try_get("total_profit")?,
            total_loss: row.try_get("total_loss")?,
            net_pnl: row.try_get("net_pnl")?,
            avg_profit_per_trade: row.try_get("avg_profit_per_trade")?,
            avg_loss_per_trade: row.try_get("avg_loss_per_trade")?,
            profit_factor: row.try_get("profit_factor")?,
            sharpe_ratio: row.try_get("sharpe_ratio")?,
            consistency_score: row.try_get("consistency_score")?,
            avg_hold_duration_seconds: row.try_get("avg_hold_duration_seconds")?,
            best_trade_pnl: row.try_get("best_trade_pnl")?,
            worst_trade_pnl: row.try_get("worst_trade_pnl")?,
            calculated_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("calculated_at")?)?.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPerformance {
    pub token_mint: String,
    pub token_symbol: String,
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub win_rate: f64,
    pub net_pnl: f64,
    pub total_volume: f64,
    pub avg_hold_duration_seconds: f64,
    pub best_trade_pnl: f64,
    pub worst_trade_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingAnalysis {
    pub hour_of_day: i32,
    pub day_of_week: i32,
    pub trades_count: i64,
    pub avg_pnl: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BestWorstTrades {
    pub best_trades: Vec<Trade>,
    pub worst_trades: Vec<Trade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkComparison {
    pub wallet_score: f64,
    pub market_avg_score: f64,
    pub percentile: f64,
    pub rank: i64,
    pub total_wallets: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreAlert {
    pub id: i64,
    pub wallet_address: String,
    pub old_score: f64,
    pub new_score: f64,
    pub change_percent: f64,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordTradeRequest {
    pub wallet_address: String,
    pub token_mint: String,
    pub token_symbol: String,
    pub side: String,
    pub amount: f64,
    pub price: f64,
    pub fee: f64,
    pub tx_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletPerformanceData {
    pub score: PerformanceScore,
    pub score_history: Vec<PerformanceScore>,
    pub token_performance: Vec<TokenPerformance>,
    pub timing_analysis: Vec<TimingAnalysis>,
    pub best_worst: BestWorstTrades,
    pub benchmark: Option<BenchmarkComparison>,
}

pub struct PerformanceDatabase {
    pool: Pool<Sqlite>,
}

impl PerformanceDatabase {
    pub async fn new(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: PerformanceDatabase failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for PerformanceDatabase");
                eprintln!("PerformanceDatabase using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        let db = Self { pool };
        db.initialize().await?;

        Ok(db)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id TEXT PRIMARY KEY,
                wallet_address TEXT NOT NULL,
                token_mint TEXT NOT NULL,
                token_symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                amount REAL NOT NULL,
                price REAL NOT NULL,
                total_value REAL NOT NULL,
                fee REAL NOT NULL,
                tx_signature TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                pnl REAL,
                hold_duration_seconds INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_trades_wallet ON trades(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_trades_token ON trades(token_mint);
            CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS performance_scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_address TEXT NOT NULL,
                score REAL NOT NULL,
                win_rate REAL NOT NULL,
                total_trades INTEGER NOT NULL,
                winning_trades INTEGER NOT NULL,
                losing_trades INTEGER NOT NULL,
                total_profit REAL NOT NULL,
                total_loss REAL NOT NULL,
                net_pnl REAL NOT NULL,
                avg_profit_per_trade REAL NOT NULL,
                avg_loss_per_trade REAL NOT NULL,
                profit_factor REAL NOT NULL,
                sharpe_ratio REAL NOT NULL,
                consistency_score REAL NOT NULL,
                avg_hold_duration_seconds REAL NOT NULL,
                best_trade_pnl REAL NOT NULL,
                worst_trade_pnl REAL NOT NULL,
                calculated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_scores_wallet ON performance_scores(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_scores_calculated ON performance_scores(calculated_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS score_alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                wallet_address TEXT NOT NULL,
                old_score REAL NOT NULL,
                new_score REAL NOT NULL,
                change_percent REAL NOT NULL,
                reason TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_alerts_wallet ON score_alerts(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_alerts_created ON score_alerts(created_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_trade(&self, request: RecordTradeRequest) -> Result<Trade, sqlx::Error> {
        let id = format!("trade_{}", uuid::Uuid::new_v4());
        let timestamp = Utc::now();
        let total_value = request.amount * request.price;

        // Calculate PnL for sell trades by finding matching buy
        let (pnl, hold_duration) = if request.side == "sell" {
            self.calculate_pnl(
                &request.wallet_address,
                &request.token_mint,
                request.price,
                request.amount,
                &timestamp,
            )
            .await?
        } else {
            (None, None)
        };

        let trade = Trade {
            id: id.clone(),
            wallet_address: request.wallet_address,
            token_mint: request.token_mint,
            token_symbol: request.token_symbol,
            side: request.side,
            amount: request.amount,
            price: request.price,
            total_value,
            fee: request.fee,
            tx_signature: request.tx_signature,
            timestamp,
            pnl,
            hold_duration_seconds: hold_duration,
        };

        sqlx::query(
            r#"
            INSERT INTO trades (
                id, wallet_address, token_mint, token_symbol, side,
                amount, price, total_value, fee, tx_signature, timestamp,
                pnl, hold_duration_seconds
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
            )
            "#,
        )
        .bind(&trade.id)
        .bind(&trade.wallet_address)
        .bind(&trade.token_mint)
        .bind(&trade.token_symbol)
        .bind(&trade.side)
        .bind(trade.amount)
        .bind(trade.price)
        .bind(trade.total_value)
        .bind(trade.fee)
        .bind(&trade.tx_signature)
        .bind(trade.timestamp.to_rfc3339())
        .bind(trade.pnl)
        .bind(trade.hold_duration_seconds)
        .execute(&self.pool)
        .await?;

        Ok(trade)
    }

    async fn calculate_pnl(
        &self,
        wallet_address: &str,
        token_mint: &str,
        sell_price: f64,
        sell_amount: f64,
        sell_timestamp: &DateTime<Utc>,
    ) -> Result<(Option<f64>, Option<i64>), sqlx::Error> {
        let buy_trade = sqlx::query_as::<_, Trade>(
            r#"
            SELECT * FROM trades
            WHERE wallet_address = ?1 
            AND token_mint = ?2 
            AND side = 'buy'
            AND timestamp < ?3
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(wallet_address)
        .bind(token_mint)
        .bind(sell_timestamp.to_rfc3339())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(buy) = buy_trade {
            let pnl = (sell_price - buy.price) * sell_amount;
            let duration = sell_timestamp
                .signed_duration_since(buy.timestamp)
                .num_seconds();
            Ok((Some(pnl), Some(duration)))
        } else {
            Ok((None, None))
        }
    }

    pub async fn calculate_performance_score(
        &self,
        wallet_address: &str,
    ) -> Result<PerformanceScore, sqlx::Error> {
        let trades = sqlx::query_as::<_, Trade>(
            r#"
            SELECT * FROM trades
            WHERE wallet_address = ?1
            ORDER BY timestamp ASC
            "#,
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        let completed_trades: Vec<&Trade> = trades.iter().filter(|t| t.pnl.is_some()).collect();

        if completed_trades.is_empty() {
            return Ok(self.default_score(wallet_address));
        }

        let total_trades = completed_trades.len() as i64;
        let winning_trades = completed_trades
            .iter()
            .filter(|t| t.pnl.unwrap_or(0.0) > 0.0)
            .count() as i64;
        let losing_trades = completed_trades
            .iter()
            .filter(|t| t.pnl.unwrap_or(0.0) < 0.0)
            .count() as i64;
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        let profits: Vec<f64> = completed_trades
            .iter()
            .filter_map(|t| {
                let pnl = t.pnl.unwrap_or(0.0);
                if pnl > 0.0 {
                    Some(pnl)
                } else {
                    None
                }
            })
            .collect();

        let losses: Vec<f64> = completed_trades
            .iter()
            .filter_map(|t| {
                let pnl = t.pnl.unwrap_or(0.0);
                if pnl < 0.0 {
                    Some(pnl)
                } else {
                    None
                }
            })
            .collect();

        let total_profit = profits.iter().sum::<f64>();
        let total_loss = losses.iter().sum::<f64>();
        let net_pnl = total_profit + total_loss;

        let avg_profit_per_trade = if !profits.is_empty() {
            total_profit / profits.len() as f64
        } else {
            0.0
        };

        let avg_loss_per_trade = if !losses.is_empty() {
            total_loss / losses.len() as f64
        } else {
            0.0
        };

        let profit_factor = if total_loss != 0.0 {
            total_profit / total_loss.abs()
        } else if total_profit > 0.0 {
            10.0 // Max profit factor when no losses
        } else {
            0.0
        };

        let all_pnls: Vec<f64> = completed_trades.iter().filter_map(|t| t.pnl).collect();
        let sharpe_ratio = self.calculate_sharpe_ratio(&all_pnls);
        let consistency_score = self.calculate_consistency_score(&all_pnls);

        let avg_hold_duration = if !completed_trades.is_empty() {
            completed_trades
                .iter()
                .filter_map(|t| t.hold_duration_seconds)
                .sum::<i64>() as f64
                / completed_trades.len() as f64
        } else {
            0.0
        };

        let best_trade_pnl = all_pnls.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let worst_trade_pnl = all_pnls.iter().cloned().fold(f64::INFINITY, f64::min);

        // Calculate overall score (0-100)
        let score =
            self.calculate_overall_score(win_rate, profit_factor, sharpe_ratio, consistency_score);

        let performance = PerformanceScore {
            id: 0,
            wallet_address: wallet_address.to_string(),
            score,
            win_rate,
            total_trades,
            winning_trades,
            losing_trades,
            total_profit,
            total_loss,
            net_pnl,
            avg_profit_per_trade,
            avg_loss_per_trade,
            profit_factor,
            sharpe_ratio,
            consistency_score,
            avg_hold_duration_seconds: avg_hold_duration,
            best_trade_pnl,
            worst_trade_pnl,
            calculated_at: Utc::now(),
        };

        self.save_performance_score(&performance).await?;
        self.check_and_create_alert(wallet_address, score).await?;

        Ok(performance)
    }

    fn default_score(&self, wallet_address: &str) -> PerformanceScore {
        PerformanceScore {
            id: 0,
            wallet_address: wallet_address.to_string(),
            score: 50.0,
            win_rate: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_profit: 0.0,
            total_loss: 0.0,
            net_pnl: 0.0,
            avg_profit_per_trade: 0.0,
            avg_loss_per_trade: 0.0,
            profit_factor: 0.0,
            sharpe_ratio: 0.0,
            consistency_score: 0.0,
            avg_hold_duration_seconds: 0.0,
            best_trade_pnl: 0.0,
            worst_trade_pnl: 0.0,
            calculated_at: Utc::now(),
        }
    }

    fn calculate_sharpe_ratio(&self, pnls: &[f64]) -> f64 {
        if pnls.len() < 2 {
            return 0.0;
        }

        let mean = pnls.iter().sum::<f64>() / pnls.len() as f64;
        let variance =
            pnls.iter().map(|pnl| (pnl - mean).powi(2)).sum::<f64>() / (pnls.len() - 1) as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        // Annualized Sharpe ratio (assuming risk-free rate of 0)
        let sharpe = mean / std_dev * (252.0_f64).sqrt();
        sharpe.max(-5.0).min(5.0) // Clamp between -5 and 5
    }

    fn calculate_consistency_score(&self, pnls: &[f64]) -> f64 {
        if pnls.len() < 2 {
            return 50.0;
        }

        let mean = pnls.iter().sum::<f64>() / pnls.len() as f64;
        let variance =
            pnls.iter().map(|pnl| (pnl - mean).powi(2)).sum::<f64>() / (pnls.len() - 1) as f64;
        let std_dev = variance.sqrt();

        let coefficient_of_variation = if mean.abs() > 0.0 {
            (std_dev / mean.abs()).abs()
        } else {
            10.0
        };

        let consistency = 100.0 / (1.0 + coefficient_of_variation);
        consistency.max(0.0).min(100.0)
    }

    fn calculate_overall_score(
        &self,
        win_rate: f64,
        profit_factor: f64,
        sharpe_ratio: f64,
        consistency_score: f64,
    ) -> f64 {
        let win_rate_component = (win_rate / 100.0) * 30.0;

        let profit_factor_normalized = if profit_factor > 3.0 {
            1.0
        } else {
            profit_factor / 3.0
        };
        let profit_component = profit_factor_normalized * 30.0;

        let sharpe_normalized = ((sharpe_ratio + 5.0) / 10.0).max(0.0).min(1.0);
        let sharpe_component = sharpe_normalized * 20.0;

        let consistency_component = (consistency_score / 100.0) * 20.0;

        let score =
            win_rate_component + profit_component + sharpe_component + consistency_component;
        score.max(0.0).min(100.0)
    }

    async fn save_performance_score(&self, score: &PerformanceScore) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO performance_scores (
                wallet_address, score, win_rate, total_trades, winning_trades, losing_trades,
                total_profit, total_loss, net_pnl, avg_profit_per_trade, avg_loss_per_trade,
                profit_factor, sharpe_ratio, consistency_score, avg_hold_duration_seconds,
                best_trade_pnl, worst_trade_pnl, calculated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
            )
            "#,
        )
        .bind(&score.wallet_address)
        .bind(score.score)
        .bind(score.win_rate)
        .bind(score.total_trades)
        .bind(score.winning_trades)
        .bind(score.losing_trades)
        .bind(score.total_profit)
        .bind(score.total_loss)
        .bind(score.net_pnl)
        .bind(score.avg_profit_per_trade)
        .bind(score.avg_loss_per_trade)
        .bind(score.profit_factor)
        .bind(score.sharpe_ratio)
        .bind(score.consistency_score)
        .bind(score.avg_hold_duration_seconds)
        .bind(score.best_trade_pnl)
        .bind(score.worst_trade_pnl)
        .bind(score.calculated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn check_and_create_alert(
        &self,
        wallet_address: &str,
        new_score: f64,
    ) -> Result<(), sqlx::Error> {
        let previous_score = sqlx::query_as::<_, PerformanceScore>(
            r#"
            SELECT * FROM performance_scores
            WHERE wallet_address = ?1
            ORDER BY calculated_at DESC
            LIMIT 1 OFFSET 1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(prev) = previous_score {
            let change_percent = ((new_score - prev.score) / prev.score) * 100.0;

            if change_percent.abs() >= 10.0 {
                let reason = if change_percent > 0.0 {
                    format!("Score increased by {:.1}%", change_percent)
                } else {
                    format!("Score decreased by {:.1}%", change_percent.abs())
                };

                sqlx::query(
                    r#"
                    INSERT INTO score_alerts (
                        wallet_address, old_score, new_score, change_percent, reason, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                    "#,
                )
                .bind(wallet_address)
                .bind(prev.score)
                .bind(new_score)
                .bind(change_percent)
                .bind(reason)
                .bind(Utc::now().to_rfc3339())
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn get_score_history(
        &self,
        wallet_address: &str,
        limit: i64,
    ) -> Result<Vec<PerformanceScore>, sqlx::Error> {
        sqlx::query_as::<_, PerformanceScore>(
            r#"
            SELECT * FROM performance_scores
            WHERE wallet_address = ?1
            ORDER BY calculated_at DESC
            LIMIT ?2
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_token_performance(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<TokenPerformance>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                token_mint,
                token_symbol,
                COUNT(*) as total_trades,
                SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END) as winning_trades,
                SUM(CASE WHEN pnl < 0 THEN 1 ELSE 0 END) as losing_trades,
                COALESCE(SUM(pnl), 0) as net_pnl,
                SUM(total_value) as total_volume,
                AVG(hold_duration_seconds) as avg_hold_duration,
                MAX(pnl) as best_trade_pnl,
                MIN(pnl) as worst_trade_pnl
            FROM trades
            WHERE wallet_address = ?1 AND pnl IS NOT NULL
            GROUP BY token_mint, token_symbol
            ORDER BY net_pnl DESC
            "#,
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        let mut performances = Vec::new();
        for row in rows {
            let total_trades: i64 = row.get("total_trades");
            let winning_trades: i64 = row.get("winning_trades");
            let win_rate = if total_trades > 0 {
                (winning_trades as f64 / total_trades as f64) * 100.0
            } else {
                0.0
            };

            performances.push(TokenPerformance {
                token_mint: row.get("token_mint"),
                token_symbol: row.get("token_symbol"),
                total_trades,
                winning_trades,
                losing_trades: row.get("losing_trades"),
                win_rate,
                net_pnl: row.get("net_pnl"),
                total_volume: row.get("total_volume"),
                avg_hold_duration_seconds: row
                    .get::<Option<f64>, _>("avg_hold_duration")
                    .unwrap_or(0.0),
                best_trade_pnl: row.get::<Option<f64>, _>("best_trade_pnl").unwrap_or(0.0),
                worst_trade_pnl: row.get::<Option<f64>, _>("worst_trade_pnl").unwrap_or(0.0),
            });
        }

        Ok(performances)
    }

    pub async fn get_best_worst_trades(
        &self,
        wallet_address: &str,
        limit: i64,
    ) -> Result<BestWorstTrades, sqlx::Error> {
        let best_trades = sqlx::query_as::<_, Trade>(
            r#"
            SELECT * FROM trades
            WHERE wallet_address = ?1 AND pnl IS NOT NULL
            ORDER BY pnl DESC
            LIMIT ?2
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let worst_trades = sqlx::query_as::<_, Trade>(
            r#"
            SELECT * FROM trades
            WHERE wallet_address = ?1 AND pnl IS NOT NULL
            ORDER BY pnl ASC
            LIMIT ?2
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(BestWorstTrades {
            best_trades,
            worst_trades,
        })
    }

    pub async fn get_timing_analysis(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<TimingAnalysis>, sqlx::Error> {
        // This requires extracting hour and day from timestamp
        // SQLite doesn't have built-in date functions, so we'll do this in application code
        let trades = sqlx::query_as::<_, Trade>(
            r#"
            SELECT * FROM trades
            WHERE wallet_address = ?1 AND pnl IS NOT NULL
            "#,
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        let mut timing_map: HashMap<(i32, i32), Vec<f64>> = HashMap::new();

        for trade in trades {
            let hour = trade.timestamp.hour() as i32;
            let day = trade.timestamp.weekday().num_days_from_monday() as i32;

            timing_map
                .entry((hour, day))
                .or_insert_with(Vec::new)
                .push(trade.pnl.unwrap_or(0.0));
        }

        let mut results = Vec::new();
        for ((hour, day), pnls) in timing_map {
            let trades_count = pnls.len() as i64;
            let avg_pnl = pnls.iter().sum::<f64>() / trades_count as f64;
            let winning = pnls.iter().filter(|&&p| p > 0.0).count() as f64;
            let win_rate = (winning / trades_count as f64) * 100.0;

            results.push(TimingAnalysis {
                hour_of_day: hour,
                day_of_week: day,
                trades_count,
                avg_pnl,
                win_rate,
            });
        }

        Ok(results)
    }

    pub async fn get_benchmark_comparison(
        &self,
        wallet_address: &str,
    ) -> Result<Option<BenchmarkComparison>, sqlx::Error> {
        let wallet_score = sqlx::query_as::<_, PerformanceScore>(
            r#"
            SELECT * FROM performance_scores
            WHERE wallet_address = ?1
            ORDER BY calculated_at DESC
            LIMIT 1
            "#,
        )
        .bind(wallet_address)
        .fetch_optional(&self.pool)
        .await?;

        if wallet_score.is_none() {
            return Ok(None);
        }

        let wallet_score = wallet_score.unwrap();

        let all_scores = sqlx::query(
            r#"
            SELECT wallet_address, score FROM (
                SELECT wallet_address, score, calculated_at,
                       ROW_NUMBER() OVER (PARTITION BY wallet_address ORDER BY calculated_at DESC) as rn
                FROM performance_scores
            )
            WHERE rn = 1
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        if all_scores.is_empty() {
            return Ok(None);
        }

        let scores: Vec<f64> = all_scores.iter().map(|r| r.get("score")).collect();
        let total_wallets = scores.len() as i64;
        let market_avg_score = scores.iter().sum::<f64>() / total_wallets as f64;

        let better_scores = scores.iter().filter(|&&s| s > wallet_score.score).count() as i64;
        let rank = better_scores + 1;
        let percentile = ((total_wallets - rank) as f64 / total_wallets as f64) * 100.0;

        Ok(Some(BenchmarkComparison {
            wallet_score: wallet_score.score,
            market_avg_score,
            percentile,
            rank,
            total_wallets,
        }))
    }

    pub async fn get_score_alerts(
        &self,
        wallet_address: &str,
        limit: i64,
    ) -> Result<Vec<ScoreAlert>, sqlx::Error> {
        sqlx::query_as::<_, ScoreAlert>(
            r#"
            SELECT * FROM score_alerts
            WHERE wallet_address = ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_wallet_performance(
        &self,
        wallet_address: &str,
    ) -> Result<WalletPerformanceData, sqlx::Error> {
        let score = self.calculate_performance_score(wallet_address).await?;
        let score_history = self.get_score_history(wallet_address, 30).await?;
        let token_performance = self.get_token_performance(wallet_address).await?;
        let timing_analysis = self.get_timing_analysis(wallet_address).await?;
        let best_worst = self.get_best_worst_trades(wallet_address, 5).await?;
        let benchmark = self.get_benchmark_comparison(wallet_address).await?;

        Ok(WalletPerformanceData {
            score,
            score_history,
            token_performance,
            timing_analysis,
            best_worst,
            benchmark,
        })
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for ScoreAlert {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            old_score: row.try_get("old_score")?,
            new_score: row.try_get("new_score")?,
            change_percent: row.try_get("change_percent")?,
            reason: row.try_get("reason")?,
            created_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("created_at")?)?.into(),
        })
    }
}

pub type SharedPerformanceDatabase = Arc<RwLock<PerformanceDatabase>>;

#[tauri::command]
pub async fn record_trade(
    request: RecordTradeRequest,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<Trade, String> {
    let db = db.read().await;
    db.record_trade(request).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn calculate_wallet_performance(
    wallet_address: String,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<PerformanceScore, String> {
    let db = db.read().await;
    db.calculate_performance_score(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_wallet_performance_data(
    wallet_address: String,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<WalletPerformanceData, String> {
    let db = db.read().await;
    db.get_wallet_performance(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_performance_score_history(
    wallet_address: String,
    limit: i64,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<Vec<PerformanceScore>, String> {
    let db = db.read().await;
    db.get_score_history(&wallet_address, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_token_performance_breakdown(
    wallet_address: String,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<Vec<TokenPerformance>, String> {
    let db = db.read().await;
    db.get_token_performance(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_timing_analysis_data(
    wallet_address: String,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<Vec<TimingAnalysis>, String> {
    let db = db.read().await;
    db.get_timing_analysis(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_best_worst_trades_data(
    wallet_address: String,
    limit: i64,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<BestWorstTrades, String> {
    let db = db.read().await;
    db.get_best_worst_trades(&wallet_address, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_benchmark_comparison_data(
    wallet_address: String,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<Option<BenchmarkComparison>, String> {
    let db = db.read().await;
    db.get_benchmark_comparison(&wallet_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_performance_alerts(
    wallet_address: String,
    limit: i64,
    db: State<'_, SharedPerformanceDatabase>,
) -> Result<Vec<ScoreAlert>, String> {
    let db = db.read().await;
    db.get_score_alerts(&wallet_address, limit)
        .await
        .map_err(|e| e.to_string())
}
