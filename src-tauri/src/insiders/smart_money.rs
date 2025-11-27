use super::types::*;
use chrono::{DateTime, Duration, Utc};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct SmartMoneyDetector {
    pool: SqlitePool,
}

impl SmartMoneyDetector {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    pub async fn classify_wallet(
        &self,
        wallet_address: &str,
    ) -> Result<SmartMoneyClassification, String> {
        let metrics = self.calculate_wallet_metrics(wallet_address).await?;

        let mut score = 0.0;
        let mut reasons = Vec::new();

        if metrics.total_trades >= 100 {
            score += 25.0;
            reasons.push("High trade volume");
        } else if metrics.total_trades >= 50 {
            score += 15.0;
            reasons.push("Moderate trade volume");
        }

        if metrics.win_rate >= 0.70 {
            score += 40.0;
            reasons.push("Excellent win rate (>70%)");
        } else if metrics.win_rate >= 0.60 {
            score += 30.0;
            reasons.push("Good win rate (>60%)");
        } else if metrics.win_rate >= 0.50 {
            score += 15.0;
        }

        if metrics.avg_profit_per_trade > 1000.0 {
            score += 20.0;
            reasons.push("High average profit");
        } else if metrics.avg_profit_per_trade > 500.0 {
            score += 10.0;
        }

        if let Some(sharpe) = metrics.sharpe_ratio {
            if sharpe > 2.0 {
                score += 15.0;
                reasons.push("Excellent risk-adjusted returns");
            } else if sharpe > 1.0 {
                score += 10.0;
            }
        }

        let is_smart_money =
            metrics.total_trades >= 100 && metrics.win_rate >= 0.60 && score >= 60.0;

        let reason = if is_smart_money {
            format!("Smart Money: {}", reasons.join(", "))
        } else {
            "Does not meet smart money criteria".to_string()
        };

        Ok(SmartMoneyClassification {
            wallet_address: wallet_address.to_string(),
            is_smart_money,
            score,
            reason,
            metrics,
        })
    }

    async fn calculate_wallet_metrics(
        &self,
        wallet_address: &str,
    ) -> Result<SmartMoneyMetrics, String> {
        let trades = self.get_wallet_trades(wallet_address).await?;

        if trades.is_empty() {
            return Ok(SmartMoneyMetrics {
                total_trades: 0,
                win_rate: 0.0,
                avg_profit_per_trade: 0.0,
                sharpe_ratio: None,
                max_drawdown: None,
            });
        }

        let mut winning_trades = 0;
        let mut total_pnl = 0.0;
        let mut returns = Vec::new();

        let mut token_positions: HashMap<String, Vec<TradeInfo>> = HashMap::new();

        for activity in &trades {
            let token_key = activity.output_mint.clone().unwrap_or_default();
            token_positions
                .entry(token_key)
                .or_insert_with(Vec::new)
                .push(TradeInfo {
                    action: activity.action_type.clone(),
                    amount_usd: activity.amount_usd.unwrap_or(0.0),
                    price: activity.price.unwrap_or(0.0),
                    timestamp: activity.timestamp,
                });
        }

        for (_, position_trades) in token_positions.iter() {
            let pnl = self.calculate_position_pnl(position_trades);
            if pnl > 0.0 {
                winning_trades += 1;
            }
            total_pnl += pnl;

            if position_trades.len() >= 2 {
                let return_pct = pnl / position_trades.first().unwrap().amount_usd.max(1.0);
                returns.push(return_pct);
            }
        }

        let total_trades = token_positions.len() as i64;
        let win_rate = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let avg_profit_per_trade = if total_trades > 0 {
            total_pnl / total_trades as f64
        } else {
            0.0
        };

        let sharpe_ratio = if returns.len() >= 5 {
            Some(self.calculate_sharpe_ratio(&returns))
        } else {
            None
        };

        let max_drawdown = if !returns.is_empty() {
            Some(self.calculate_max_drawdown(&returns))
        } else {
            None
        };

        Ok(SmartMoneyMetrics {
            total_trades,
            win_rate,
            avg_profit_per_trade,
            sharpe_ratio,
            max_drawdown,
        })
    }

    async fn get_wallet_trades(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<WalletActivityRecord>, String> {
        let cutoff = Utc::now() - Duration::days(90);

        sqlx::query_as::<_, WalletActivityRecord>(
            r#"
            SELECT * FROM wallet_activities
            WHERE wallet_address = ?1
            AND (action_type = 'buy' OR action_type = 'sell')
            AND timestamp >= ?2
            ORDER BY timestamp ASC
            "#,
        )
        .bind(wallet_address)
        .bind(cutoff.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch wallet trades: {e}"))
    }

    fn calculate_position_pnl(&self, trades: &[TradeInfo]) -> f64 {
        let mut cost_basis = 0.0;
        let mut quantity = 0.0;
        let mut realized_pnl = 0.0;

        for trade in trades {
            match trade.action.as_str() {
                "buy" => {
                    cost_basis += trade.amount_usd;
                    quantity += trade.amount_usd / trade.price.max(0.01);
                }
                "sell" => {
                    if quantity > 0.0 {
                        let sell_value = trade.amount_usd;
                        let avg_cost = cost_basis / quantity;
                        let sold_quantity = sell_value / trade.price.max(0.01);
                        let pnl = sell_value - (sold_quantity * avg_cost);
                        realized_pnl += pnl;

                        quantity -= sold_quantity;
                        if quantity > 0.0 {
                            cost_basis -= sold_quantity * avg_cost;
                        } else {
                            cost_basis = 0.0;
                            quantity = 0.0;
                        }
                    }
                }
                _ => {}
            }
        }

        realized_pnl
    }

    fn calculate_sharpe_ratio(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            0.0
        } else {
            mean / std_dev
        }
    }

    fn calculate_max_drawdown(&self, returns: &[f64]) -> f64 {
        let mut peak = 0.0;
        let mut max_drawdown = 0.0;
        let mut cumulative = 0.0;

        for ret in returns {
            cumulative += ret;
            if cumulative > peak {
                peak = cumulative;
            }
            let drawdown = peak - cumulative;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        max_drawdown
    }

    pub async fn update_smart_money_wallet(
        &self,
        classification: &SmartMoneyClassification,
        label: Option<&str>,
    ) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();

        let existing = sqlx::query(
            "SELECT id, first_seen, label FROM smart_money_wallets WHERE wallet_address = ?1",
        )
        .bind(&classification.wallet_address)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to load smart money wallet: {e}"))?;

        let (id, first_seen, existing_label) = if let Some(row) = existing {
            let id: String = row
                .try_get("id")
                .map_err(|e| format!("Failed to read smart money wallet id: {e}"))?;
            let first_seen: String = row
                .try_get("first_seen")
                .map_err(|e| format!("Failed to read smart money wallet first_seen: {e}"))?;
            let label: Option<String> = row.try_get("label").ok();
            (id, first_seen, label)
        } else {
            (Uuid::new_v4().to_string(), now.clone(), None)
        };

        let final_label = label
            .map(|s| s.to_string())
            .or(existing_label)
            .unwrap_or_default();

        let winning_trades =
            (classification.metrics.total_trades as f64 * classification.metrics.win_rate)
                .round()
                .clamp(0.0, classification.metrics.total_trades as f64) as i64;
        let losing_trades = classification.metrics.total_trades - winning_trades;
        let total_pnl = classification.metrics.avg_profit_per_trade
            * classification.metrics.total_trades as f64;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO smart_money_wallets (
                id, wallet_address, label, total_trades, winning_trades, losing_trades,
                win_rate, total_pnl, avg_hold_time_hours, smart_money_score,
                is_smart_money, first_seen, last_updated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(&id)
        .bind(&classification.wallet_address)
        .bind(&final_label)
        .bind(classification.metrics.total_trades)
        .bind(winning_trades)
        .bind(losing_trades)
        .bind(classification.metrics.win_rate)
        .bind(total_pnl)
        .bind(0.0)
        .bind(classification.score)
        .bind(if classification.is_smart_money { 1 } else { 0 })
        .bind(&first_seen)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update smart money wallet: {e}"))?;

        Ok(())
    }

    pub async fn is_smart_money(&self, wallet_address: &str) -> Result<bool, String> {
        let result =
            sqlx::query("SELECT is_smart_money FROM smart_money_wallets WHERE wallet_address = ?1")
                .bind(wallet_address)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| format!("Failed to check smart money status: {e}"))?;

        Ok(result
            .map(|row| row.try_get::<i64, _>("is_smart_money").unwrap_or(0) == 1)
            .unwrap_or(false))
    }

    pub async fn get_smart_money_wallets(&self) -> Result<Vec<SmartMoneyWallet>, String> {
        sqlx::query_as::<_, SmartMoneyWallet>(
            "SELECT * FROM smart_money_wallets WHERE is_smart_money = 1 ORDER BY smart_money_score DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch smart money wallets: {e}"))
    }

    pub async fn get_consensus(
        &self,
        time_window_hours: i64,
    ) -> Result<Vec<SmartMoneyConsensus>, String> {
        let cutoff = Utc::now() - Duration::hours(time_window_hours);

        let rows = sqlx::query(
            r#"
            SELECT 
                wa.output_mint as token_mint,
                wa.output_symbol as token_symbol,
                wa.action_type as action,
                COUNT(DISTINCT wa.wallet_address) as smart_wallets_count,
                SUM(wa.amount_usd) as total_volume_usd,
                AVG(wa.price) as avg_price,
                MIN(wa.timestamp) as first_seen,
                MAX(wa.timestamp) as last_updated
            FROM wallet_activities wa
            INNER JOIN smart_money_wallets smw ON wa.wallet_address = smw.wallet_address
            WHERE smw.is_smart_money = 1
            AND wa.timestamp >= ?1
            AND (wa.action_type = 'buy' OR wa.action_type = 'sell')
            AND wa.output_mint IS NOT NULL
            GROUP BY wa.output_mint, wa.action_type
            HAVING COUNT(DISTINCT wa.wallet_address) >= 3
            ORDER BY smart_wallets_count DESC, total_volume_usd DESC
            LIMIT 20
            "#,
        )
        .bind(cutoff.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch consensus: {e}"))?;

        let mut consensus_list = Vec::new();
        for row in rows {
            let smart_wallets_count: i64 = row.try_get("smart_wallets_count").unwrap_or(0);
            let total_volume: f64 = row.try_get("total_volume_usd").unwrap_or(0.0);
            let consensus_strength = (smart_wallets_count as f64 / 10.0).min(1.0) * 50.0
                + (total_volume / 100000.0).min(1.0) * 50.0;

            consensus_list.push(SmartMoneyConsensus {
                token_mint: row.try_get("token_mint").unwrap_or_default(),
                token_symbol: row.try_get("token_symbol").ok(),
                action: row.try_get("action").unwrap_or_default(),
                smart_wallets_count,
                total_volume_usd: total_volume,
                avg_price: row.try_get("avg_price").unwrap_or(0.0),
                consensus_strength,
                first_seen: row
                    .try_get::<String, _>("first_seen")
                    .ok()
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|| Utc::now()),
                last_updated: row
                    .try_get::<String, _>("last_updated")
                    .ok()
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|| Utc::now()),
            });
        }

        Ok(consensus_list)
    }

    pub async fn get_sentiment_comparison(
        &self,
        token_mint: &str,
    ) -> Result<SentimentComparison, String> {
        let cutoff = Utc::now() - Duration::hours(24);

        let smart_row = sqlx::query(
            r#"
            SELECT 
                SUM(CASE WHEN action_type = 'buy' THEN amount_usd ELSE 0 END) as buy_volume,
                SUM(CASE WHEN action_type = 'sell' THEN amount_usd ELSE 0 END) as sell_volume
            FROM wallet_activities wa
            INNER JOIN smart_money_wallets smw ON wa.wallet_address = smw.wallet_address
            WHERE smw.is_smart_money = 1
            AND wa.output_mint = ?1
            AND wa.timestamp >= ?2
            "#,
        )
        .bind(token_mint)
        .bind(cutoff.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch smart money sentiment: {e}"))?;

        let retail_row = sqlx::query(
            r#"
            SELECT 
                SUM(CASE WHEN action_type = 'buy' THEN amount_usd ELSE 0 END) as buy_volume,
                SUM(CASE WHEN action_type = 'sell' THEN amount_usd ELSE 0 END) as sell_volume
            FROM wallet_activities wa
            LEFT JOIN smart_money_wallets smw ON wa.wallet_address = smw.wallet_address
            WHERE (smw.is_smart_money IS NULL OR smw.is_smart_money = 0)
            AND wa.output_mint = ?1
            AND wa.timestamp >= ?2
            "#,
        )
        .bind(token_mint)
        .bind(cutoff.to_rfc3339())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch retail sentiment: {e}"))?;

        let smart_buy: f64 = smart_row.try_get("buy_volume").unwrap_or(0.0);
        let smart_sell: f64 = smart_row.try_get("sell_volume").unwrap_or(0.0);
        let retail_buy: f64 = retail_row.try_get("buy_volume").unwrap_or(0.0);
        let retail_sell: f64 = retail_row.try_get("sell_volume").unwrap_or(0.0);

        let smart_total = smart_buy + smart_sell;
        let retail_total = retail_buy + retail_sell;

        let smart_sentiment = if smart_total > 0.0 {
            (smart_buy - smart_sell) / smart_total
        } else {
            0.0
        };

        let retail_sentiment = if retail_total > 0.0 {
            (retail_buy - retail_sell) / retail_total
        } else {
            0.0
        };

        let divergence = smart_sentiment - retail_sentiment;

        Ok(SentimentComparison {
            token_mint: token_mint.to_string(),
            token_symbol: None,
            smart_money_sentiment: smart_sentiment,
            retail_sentiment,
            divergence,
            smart_money_volume: smart_total,
            retail_volume: retail_total,
            timestamp: Utc::now(),
        })
    }
}

#[derive(Debug, Clone)]
struct TradeInfo {
    action: String,
    amount_usd: f64,
    price: f64,
    timestamp: DateTime<Utc>,
}
