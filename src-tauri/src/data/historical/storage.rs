use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataSet {
    pub symbol: String,
    pub interval: String, // 1m, 5m, 15m, 1h, 4h, 1d
    pub data: Vec<HistoricalDataPoint>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub timestamp: i64,
    pub symbol: String,
    pub bids: Vec<(f64, f64)>, // price, quantity
    pub asks: Vec<(f64, f64)>,
}

pub struct HistoricalStorage {
    pool: Pool<Sqlite>,
}

impl HistoricalStorage {
    pub async fn new(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let storage = Self { pool };
        storage.initialize().await?;

        Ok(storage)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        // Create historical_prices table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS historical_prices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                interval TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                open REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                close REAL NOT NULL,
                volume REAL NOT NULL,
                fetched_at TEXT NOT NULL,
                UNIQUE(symbol, interval, timestamp)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_prices_symbol_interval ON historical_prices(symbol, interval);
            CREATE INDEX IF NOT EXISTS idx_prices_timestamp ON historical_prices(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create historical_orderbooks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS historical_orderbooks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                bids TEXT NOT NULL,
                asks TEXT NOT NULL,
                fetched_at TEXT NOT NULL,
                UNIQUE(symbol, timestamp)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_orderbooks_symbol ON historical_orderbooks(symbol);
            CREATE INDEX IF NOT EXISTS idx_orderbooks_timestamp ON historical_orderbooks(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create data_cache_metadata table for tracking fetched ranges
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS data_cache_metadata (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol TEXT NOT NULL,
                interval TEXT NOT NULL,
                start_timestamp INTEGER NOT NULL,
                end_timestamp INTEGER NOT NULL,
                fetched_at TEXT NOT NULL,
                UNIQUE(symbol, interval, start_timestamp, end_timestamp)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn store_price_data(
        &self,
        symbol: &str,
        interval: &str,
        data: &[HistoricalDataPoint],
    ) -> Result<(), sqlx::Error> {
        if data.is_empty() {
            return Ok(());
        }

        let fetched_at = Utc::now().to_rfc3339();

        for point in data {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO historical_prices
                (symbol, interval, timestamp, open, high, low, close, volume, fetched_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )
            .bind(symbol)
            .bind(interval)
            .bind(point.timestamp)
            .bind(point.open)
            .bind(point.high)
            .bind(point.low)
            .bind(point.close)
            .bind(point.volume)
            .bind(&fetched_at)
            .execute(&self.pool)
            .await?;
        }

        // Update metadata
        if let (Some(first), Some(last)) = (data.first(), data.last()) {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO data_cache_metadata
                (symbol, interval, start_timestamp, end_timestamp, fetched_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .bind(symbol)
            .bind(interval)
            .bind(first.timestamp)
            .bind(last.timestamp)
            .bind(&fetched_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_price_data(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<HistoricalDataPoint>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, f64, f64, f64, f64, f64)>(
            r#"
            SELECT timestamp, open, high, low, close, volume
            FROM historical_prices
            WHERE symbol = ?1 AND interval = ?2
              AND timestamp >= ?3 AND timestamp <= ?4
            ORDER BY timestamp ASC
            "#,
        )
        .bind(symbol)
        .bind(interval)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;

        let data = rows
            .into_iter()
            .map(
                |(timestamp, open, high, low, close, volume)| HistoricalDataPoint {
                    timestamp,
                    open,
                    high,
                    low,
                    close,
                    volume,
                },
            )
            .collect();

        Ok(data)
    }

    pub async fn check_data_coverage(
        &self,
        symbol: &str,
        interval: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM data_cache_metadata
            WHERE symbol = ?1 AND interval = ?2
              AND start_timestamp <= ?3 AND end_timestamp >= ?4
            "#,
        )
        .bind(symbol)
        .bind(interval)
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }

    pub async fn store_orderbook_snapshot(
        &self,
        snapshot: &OrderBookSnapshot,
    ) -> Result<(), sqlx::Error> {
        let bids_json = serde_json::to_string(&snapshot.bids).unwrap_or_default();
        let asks_json = serde_json::to_string(&snapshot.asks).unwrap_or_default();
        let fetched_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO historical_orderbooks
            (symbol, timestamp, bids, asks, fetched_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&snapshot.symbol)
        .bind(snapshot.timestamp)
        .bind(&bids_json)
        .bind(&asks_json)
        .bind(&fetched_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_orderbook_snapshots(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<OrderBookSnapshot>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, String, String)>(
            r#"
            SELECT timestamp, bids, asks
            FROM historical_orderbooks
            WHERE symbol = ?1 AND timestamp >= ?2 AND timestamp <= ?3
            ORDER BY timestamp ASC
            "#,
        )
        .bind(symbol)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;

        let mut snapshots = Vec::new();
        for (timestamp, bids_json, asks_json) in rows {
            let bids: Vec<(f64, f64)> = serde_json::from_str(&bids_json).unwrap_or_default();
            let asks: Vec<(f64, f64)> = serde_json::from_str(&asks_json).unwrap_or_default();

            snapshots.push(OrderBookSnapshot {
                timestamp,
                symbol: symbol.to_string(),
                bids,
                asks,
            });
        }

        Ok(snapshots)
    }

    pub async fn get_cache_stats(&self, symbol: &str) -> Result<HashMap<String, u64>, sqlx::Error> {
        let mut stats = HashMap::new();

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM historical_prices WHERE symbol = ?1")
                .bind(symbol)
                .fetch_one(&self.pool)
                .await?;
        stats.insert("price_points".to_string(), count.0 as u64);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM historical_orderbooks WHERE symbol = ?1")
                .bind(symbol)
                .fetch_one(&self.pool)
                .await?;
        stats.insert("orderbook_snapshots".to_string(), count.0 as u64);

        Ok(stats)
    }

    pub async fn clear_old_data(&self, days: i64) -> Result<u64, sqlx::Error> {
        let cutoff_time = (Utc::now() - chrono::Duration::days(days)).timestamp();

        let result = sqlx::query("DELETE FROM historical_prices WHERE timestamp < ?1")
            .bind(cutoff_time)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM historical_orderbooks WHERE timestamp < ?1")
            .bind(cutoff_time)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
