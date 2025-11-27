use crate::trading::types::Order;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use zstd;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub age_threshold_days: i64,
    pub compression_level: i32,
    pub auto_compress: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            age_threshold_days: 7,
            compression_level: 3,
            auto_compress: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub total_uncompressed_bytes: i64,
    pub total_compressed_bytes: i64,
    pub compression_ratio: f64,
    pub num_compressed_records: i64,
    pub space_saved_mb: f64,
    pub last_compression_run: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct CompressedRecord {
    id: String,
    record_type: String,
    compressed_data: Vec<u8>,
    original_size: i64,
    compressed_size: i64,
    compressed_at: String,
    original_timestamp: String,
}

#[derive(Debug, Clone)]
struct DecompressedCacheEntry {
    data: Vec<u8>,
    cached_at: DateTime<Utc>,
}

pub struct CompressionManager {
    pool: Pool<Sqlite>,
    config: Arc<RwLock<CompressionConfig>>,
    decompression_cache: Arc<RwLock<HashMap<String, DecompressedCacheEntry>>>,
    stats_cache: Arc<RwLock<Option<(CompressionStats, DateTime<Utc>)>>>,
}

impl CompressionManager {
    pub async fn new(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self {
            pool,
            config: Arc::new(RwLock::new(CompressionConfig::default())),
            decompression_cache: Arc::new(RwLock::new(HashMap::new())),
            stats_cache: Arc::new(RwLock::new(None)),
        };

        manager.initialize().await?;

        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        // Create compressed_data table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS compressed_data (
                id TEXT PRIMARY KEY,
                record_type TEXT NOT NULL,
                compressed_data BLOB NOT NULL,
                original_size INTEGER NOT NULL,
                compressed_size INTEGER NOT NULL,
                compressed_at TEXT NOT NULL,
                original_timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_compressed_type ON compressed_data(record_type);
            CREATE INDEX IF NOT EXISTS idx_compressed_timestamp ON compressed_data(original_timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create compression_config table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS compression_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                enabled INTEGER NOT NULL DEFAULT 1,
                age_threshold_days INTEGER NOT NULL DEFAULT 7,
                compression_level INTEGER NOT NULL DEFAULT 3,
                auto_compress INTEGER NOT NULL DEFAULT 1,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create compression_log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS compression_log (
                id TEXT PRIMARY KEY,
                records_compressed INTEGER NOT NULL,
                space_saved_bytes INTEGER NOT NULL,
                compression_time_ms INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Initialize config if not exists
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO compression_config (id, enabled, age_threshold_days, compression_level, auto_compress, updated_at)
            VALUES (1, 1, 7, 3, 1, ?1)
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Load config from database
        self.load_config().await?;

        Ok(())
    }

    async fn load_config(&self) -> Result<(), sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT enabled, age_threshold_days, compression_level, auto_compress
            FROM compression_config
            WHERE id = 1
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let mut config = self.config.write().await;
        config.enabled = row.get::<i32, _>("enabled") != 0;
        config.age_threshold_days = row.get::<i64, _>("age_threshold_days");
        config.compression_level = row.get::<i32, _>("compression_level");
        config.auto_compress = row.get::<i32, _>("auto_compress") != 0;

        Ok(())
    }

    pub async fn get_config(&self) -> CompressionConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: CompressionConfig) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE compression_config
            SET enabled = ?1, age_threshold_days = ?2, compression_level = ?3, auto_compress = ?4, updated_at = ?5
            WHERE id = 1
            "#,
        )
        .bind(if new_config.enabled { 1 } else { 0 })
        .bind(new_config.age_threshold_days)
        .bind(new_config.compression_level)
        .bind(if new_config.auto_compress { 1 } else { 0 })
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        let mut config = self.config.write().await;
        *config = new_config;

        Ok(())
    }

    pub async fn compress_data(
        &self,
        data: &[u8],
        record_type: &str,
        record_id: &str,
        original_timestamp: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().await;

        if !config.enabled {
            return Ok(());
        }

        let compressed = zstd::encode_all(data, config.compression_level)?;
        let original_size = data.len() as i64;
        let compressed_size = compressed.len() as i64;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO compressed_data 
            (id, record_type, compressed_data, original_size, compressed_size, compressed_at, original_timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(record_id)
        .bind(record_type)
        .bind(&compressed)
        .bind(original_size)
        .bind(compressed_size)
        .bind(Utc::now().to_rfc3339())
        .bind(original_timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Invalidate stats cache
        let mut stats_cache = self.stats_cache.write().await;
        *stats_cache = None;

        Ok(())
    }

    pub async fn decompress_data(
        &self,
        record_id: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.decompression_cache.read().await;
            if let Some(entry) = cache.get(record_id) {
                let elapsed = Utc::now().signed_duration_since(entry.cached_at);
                if elapsed.num_seconds() < 300 {
                    return Ok(entry.data.clone());
                }
            }
        }

        // Fetch from database
        let record =
            sqlx::query_as::<_, CompressedRecord>("SELECT * FROM compressed_data WHERE id = ?1")
                .bind(record_id)
                .fetch_one(&self.pool)
                .await?;

        let decompressed = zstd::decode_all(&record.compressed_data[..])?;

        // Cache the decompressed data
        let mut cache = self.decompression_cache.write().await;
        cache.insert(
            record_id.to_string(),
            DecompressedCacheEntry {
                data: decompressed.clone(),
                cached_at: Utc::now(),
            },
        );

        // Limit cache size to 100 entries
        if cache.len() > 100 {
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        Ok(decompressed)
    }

    pub async fn compress_old_events(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let config = self.config.read().await.clone();

        if !config.enabled {
            return Ok(0);
        }

        let start_time = std::time::Instant::now();
        let threshold_date = Utc::now() - Duration::days(config.age_threshold_days);

        // Get old events that aren't compressed yet
        let old_events = sqlx::query(
            r#"
            SELECT id, event_data, timestamp
            FROM events
            WHERE timestamp < ?1
            AND id NOT IN (SELECT id FROM compressed_data WHERE record_type = 'event')
            LIMIT 1000
            "#,
        )
        .bind(threshold_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut compressed_count = 0;
        let mut space_saved = 0i64;

        for event in old_events {
            let event_id: String = event.get("id");
            let event_data: String = event.get("event_data");
            let timestamp_str: String = event.get("timestamp");

            let data = event_data.as_bytes();
            let original_size = data.len() as i64;

            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)?.with_timezone(&Utc);

            self.compress_data(data, "event", &event_id, timestamp)
                .await?;

            // Get compressed size
            let compressed =
                sqlx::query("SELECT compressed_size FROM compressed_data WHERE id = ?1")
                    .bind(&event_id)
                    .fetch_one(&self.pool)
                    .await?;

            space_saved += original_size - compressed.get::<i64, _>("compressed_size");
            compressed_count += 1;
        }

        let compression_time = start_time.elapsed().as_millis() as i64;

        // Log compression run
        if compressed_count > 0 {
            sqlx::query(
                r#"
                INSERT INTO compression_log (id, records_compressed, space_saved_bytes, compression_time_ms, timestamp)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(compressed_count)
            .bind(space_saved)
            .bind(compression_time)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;
        }

        Ok(compressed_count)
    }

    pub async fn compress_old_trades(&self) -> Result<i64, Box<dyn std::error::Error>> {
        let config = self.config.read().await.clone();

        if !config.enabled {
            return Ok(0);
        }

        let threshold_date = Utc::now() - Duration::days(30); // Compress trades older than 30 days

        // Check if orders table exists
        let table_exists = sqlx::query(
            r#"
            SELECT name FROM sqlite_master 
            WHERE type='table' AND name='orders'
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if table_exists.is_none() {
            return Ok(0);
        }

        // Get old closed orders that aren't compressed yet
        let old_orders = sqlx::query(
            r#"
            SELECT id, created_at
            FROM orders
            WHERE status IN ('filled', 'cancelled', 'failed')
            AND created_at < ?1
            AND id NOT IN (SELECT id FROM compressed_data WHERE record_type = 'trade')
            LIMIT 1000
            "#,
        )
        .bind(threshold_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut compressed_count = 0;

        for order in old_orders {
            let order_id: String = order.get("id");
            let created_at: String = order.get("created_at");

            // Get full order data
            let order_record = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = ?1")
                .bind(&order_id)
                .fetch_one(&self.pool)
                .await?;

            let order_data = serde_json::to_string(&order_record)?;
            let data = order_data.as_bytes();

            let timestamp = DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc);

            self.compress_data(data, "trade", &order_id, timestamp)
                .await?;
            compressed_count += 1;
        }

        Ok(compressed_count)
    }

    pub async fn get_compression_stats(
        &self,
    ) -> Result<CompressionStats, Box<dyn std::error::Error>> {
        // Check cache first (cache for 30 seconds)
        {
            let cache = self.stats_cache.read().await;
            if let Some((stats, cached_at)) = cache.as_ref() {
                let elapsed = Utc::now().signed_duration_since(*cached_at);
                if elapsed.num_seconds() < 30 {
                    return Ok(stats.clone());
                }
            }
        }

        let totals = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(original_size), 0) as total_original,
                COALESCE(SUM(compressed_size), 0) as total_compressed,
                COUNT(*) as num_records
            FROM compressed_data
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let last_run = sqlx::query(
            r#"
            SELECT timestamp
            FROM compression_log
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let total_uncompressed = totals.get::<i64, _>("total_original");
        let total_compressed = totals.get::<i64, _>("total_compressed");
        let num_records = totals.get::<i64, _>("num_records");

        let compression_ratio = if total_uncompressed > 0 {
            ((total_uncompressed - total_compressed) as f64 / total_uncompressed as f64) * 100.0
        } else {
            0.0
        };

        let stats = CompressionStats {
            total_uncompressed_bytes: total_uncompressed,
            total_compressed_bytes: total_compressed,
            compression_ratio,
            num_compressed_records: num_records,
            space_saved_mb: (total_uncompressed - total_compressed) as f64 / 1024.0 / 1024.0,
            last_compression_run: last_run.map(|r| r.get::<String, _>("timestamp")),
        };

        // Update cache
        let mut cache = self.stats_cache.write().await;
        *cache = Some((stats.clone(), Utc::now()));

        Ok(stats)
    }

    pub async fn cleanup_cache(&self) {
        let mut cache = self.decompression_cache.write().await;
        let now = Utc::now();

        cache.retain(|_, entry| {
            let elapsed = now.signed_duration_since(entry.cached_at);
            elapsed.num_seconds() < 300
        });
    }
}

pub type SharedCompressionManager = Arc<RwLock<CompressionManager>>;
