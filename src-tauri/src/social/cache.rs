use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::{fs, path::PathBuf};

use super::models::SocialPost;
use uuid::Uuid;

const SOCIAL_DB_FILE: &str = "social_intel.db";

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPost {
    pub id: String,
    pub post_data: String,
    pub source: String,
    pub token: Option<String>,
    pub timestamp: i64,
    pub cached_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionAggregate {
    pub token: String,
    pub source: String,
    pub mention_count: i32,
    pub positive_count: i32,
    pub negative_count: i32,
    pub neutral_count: i32,
    pub avg_sentiment: f32,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSnapshot {
    pub id: String,
    pub token: String,
    pub source: String,
    pub snapshot_time: i64,
    pub mention_count: i32,
    pub sentiment_score: f32,
    pub engagement_total: i64,
}

#[derive(Clone)]
pub struct SocialCache {
    pool: Pool<Sqlite>,
}

impl SocialCache {
    pub async fn new(app_data_dir: PathBuf) -> Result<Self, CacheError> {
        let _ = fs::create_dir_all(&app_data_dir);

        let db_path = app_data_dir.join(SOCIAL_DB_FILE);
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: SocialCache failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for SocialCache");
                eprintln!("SocialCache using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        let cache = Self { pool };
        cache.initialize().await?;
        Ok(cache)
    }

    async fn initialize(&self) -> Result<(), CacheError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS social_posts (
                id TEXT PRIMARY KEY,
                post_data TEXT NOT NULL,
                source TEXT NOT NULL,
                token TEXT,
                timestamp INTEGER NOT NULL,
                cached_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mention_aggregates (
                token TEXT NOT NULL,
                source TEXT NOT NULL,
                mention_count INTEGER NOT NULL DEFAULT 0,
                positive_count INTEGER NOT NULL DEFAULT 0,
                negative_count INTEGER NOT NULL DEFAULT 0,
                neutral_count INTEGER NOT NULL DEFAULT 0,
                avg_sentiment REAL NOT NULL DEFAULT 0.0,
                last_updated INTEGER NOT NULL,
                PRIMARY KEY (token, source)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trend_snapshots (
                id TEXT PRIMARY KEY,
                token TEXT NOT NULL,
                source TEXT NOT NULL,
                snapshot_time INTEGER NOT NULL,
                mention_count INTEGER NOT NULL,
                sentiment_score REAL NOT NULL,
                engagement_total INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_posts_source ON social_posts(source);
            CREATE INDEX IF NOT EXISTS idx_posts_token ON social_posts(token);
            CREATE INDEX IF NOT EXISTS idx_posts_timestamp ON social_posts(timestamp);
            CREATE INDEX IF NOT EXISTS idx_mentions_token ON mention_aggregates(token);
            CREATE INDEX IF NOT EXISTS idx_trends_token_time ON trend_snapshots(token, snapshot_time);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create tables for sentiment analysis and derived metrics
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sentiment_lexicon (
                term TEXT PRIMARY KEY,
                weight REAL NOT NULL,
                category TEXT,
                is_negation INTEGER NOT NULL DEFAULT 0,
                metadata TEXT,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sentiment_scores (
                post_id TEXT PRIMARY KEY,
                token TEXT,
                source TEXT,
                timestamp INTEGER NOT NULL,
                score REAL NOT NULL,
                label TEXT NOT NULL,
                confidence REAL NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sentiment_snapshots (
                token TEXT PRIMARY KEY,
                avg_score REAL NOT NULL,
                momentum REAL NOT NULL,
                mention_count INTEGER NOT NULL,
                positive_mentions INTEGER NOT NULL,
                negative_mentions INTEGER NOT NULL,
                neutral_mentions INTEGER NOT NULL,
                confidence REAL NOT NULL,
                dominant_label TEXT NOT NULL,
                last_post_timestamp INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS social_trends (
                token TEXT NOT NULL,
                window_minutes INTEGER NOT NULL,
                mentions INTEGER NOT NULL,
                velocity REAL NOT NULL,
                acceleration REAL NOT NULL,
                volume_spike REAL NOT NULL,
                sentiment_avg REAL NOT NULL,
                engagement_total INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (token, window_minutes)
            );
            CREATE TABLE IF NOT EXISTS social_influencer_scores (
                influencer TEXT PRIMARY KEY,
                follower_score REAL NOT NULL,
                engagement_score REAL NOT NULL,
                accuracy_score REAL NOT NULL,
                impact_score REAL NOT NULL,
                sample_size INTEGER NOT NULL,
                tokens TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS social_gauges (
                token TEXT PRIMARY KEY,
                fomo_score REAL NOT NULL,
                fomo_level TEXT NOT NULL,
                fud_score REAL NOT NULL,
                fud_level TEXT NOT NULL,
                drivers TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sentiment_scores_token_time ON sentiment_scores(token, timestamp);
            CREATE INDEX IF NOT EXISTS idx_sentiment_scores_label ON sentiment_scores(label);
            CREATE INDEX IF NOT EXISTS idx_social_trends_token ON social_trends(token);
            CREATE INDEX IF NOT EXISTS idx_social_trends_updated ON social_trends(updated_at);
            CREATE INDEX IF NOT EXISTS idx_social_influencer_scores_impact ON social_influencer_scores(impact_score);
            CREATE INDEX IF NOT EXISTS idx_social_gauges_token ON social_gauges(token);
            CREATE INDEX IF NOT EXISTS idx_sentiment_lexicon_category ON sentiment_lexicon(category);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn store_posts(
        &self,
        posts: &[SocialPost],
        token: Option<&str>,
    ) -> Result<(), CacheError> {
        let now = Utc::now().timestamp();

        for post in posts {
            let post_json = serde_json::to_string(post)?;

            sqlx::query(
                r#"
                INSERT OR REPLACE INTO social_posts (id, post_data, source, token, timestamp, cached_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
            )
            .bind(&post.id)
            .bind(&post_json)
            .bind(&post.source)
            .bind(token)
            .bind(post.timestamp)
            .bind(now)
            .execute(&self.pool)
            .await?;
        }

        if let Some(token_addr) = token {
            self.update_mention_aggregates(token_addr, posts).await?;
        }

        Ok(())
    }

    pub async fn get_cached_posts(
        &self,
        source: Option<&str>,
        token: Option<&str>,
        limit: Option<i32>,
    ) -> Result<Vec<SocialPost>, CacheError> {
        let limit = limit.unwrap_or(100);

        let query = match (source, token) {
            (Some(src), Some(tok)) => {
                sqlx::query(
                    "SELECT post_data FROM social_posts WHERE source = ?1 AND token = ?2 ORDER BY timestamp DESC LIMIT ?3"
                )
                .bind(src)
                .bind(tok)
                .bind(limit)
            }
            (Some(src), None) => {
                sqlx::query(
                    "SELECT post_data FROM social_posts WHERE source = ?1 ORDER BY timestamp DESC LIMIT ?2"
                )
                .bind(src)
                .bind(limit)
            }
            (None, Some(tok)) => {
                sqlx::query(
                    "SELECT post_data FROM social_posts WHERE token = ?1 ORDER BY timestamp DESC LIMIT ?2"
                )
                .bind(tok)
                .bind(limit)
            }
            (None, None) => {
                sqlx::query(
                    "SELECT post_data FROM social_posts ORDER BY timestamp DESC LIMIT ?1"
                )
                .bind(limit)
            }
        };

        let rows = query.fetch_all(&self.pool).await?;

        let mut posts = Vec::new();
        for row in rows {
            let post_data: String = row.try_get("post_data")?;
            let post: SocialPost = serde_json::from_str(&post_data)?;
            posts.push(post);
        }

        Ok(posts)
    }

    async fn update_mention_aggregates(
        &self,
        token: &str,
        posts: &[SocialPost],
    ) -> Result<(), CacheError> {
        let mut positive = 0;
        let mut negative = 0;
        let mut neutral = 0;
        let mut total_sentiment = 0.0;

        for post in posts {
            match post.sentiment.label.as_str() {
                "positive" => positive += 1,
                "negative" => negative += 1,
                _ => neutral += 1,
            }
            total_sentiment += post.sentiment.score;
        }

        let avg_sentiment = if !posts.is_empty() {
            total_sentiment / posts.len() as f32
        } else {
            0.0
        };

        let source = posts
            .first()
            .map(|p| p.source.as_str())
            .unwrap_or("unknown");
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO mention_aggregates (token, source, mention_count, positive_count, negative_count, neutral_count, avg_sentiment, last_updated)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(token, source) DO UPDATE SET
                mention_count = mention_count + ?3,
                positive_count = positive_count + ?4,
                negative_count = negative_count + ?5,
                neutral_count = neutral_count + ?6,
                avg_sentiment = (?7 + avg_sentiment) / 2.0,
                last_updated = ?8
            "#,
        )
        .bind(token)
        .bind(source)
        .bind(posts.len() as i32)
        .bind(positive)
        .bind(negative)
        .bind(neutral)
        .bind(avg_sentiment)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_mention_aggregates(
        &self,
        token: Option<&str>,
    ) -> Result<Vec<MentionAggregate>, CacheError> {
        let query = if let Some(tok) = token {
            sqlx::query(
                "SELECT * FROM mention_aggregates WHERE token = ?1 ORDER BY last_updated DESC",
            )
            .bind(tok)
        } else {
            sqlx::query("SELECT * FROM mention_aggregates ORDER BY last_updated DESC LIMIT 100")
        };

        let rows = query.fetch_all(&self.pool).await?;

        let mut aggregates = Vec::new();
        for row in rows {
            aggregates.push(MentionAggregate {
                token: row.try_get("token")?,
                source: row.try_get("source")?,
                mention_count: row.try_get("mention_count")?,
                positive_count: row.try_get("positive_count")?,
                negative_count: row.try_get("negative_count")?,
                neutral_count: row.try_get("neutral_count")?,
                avg_sentiment: row.try_get("avg_sentiment")?,
                last_updated: row.try_get("last_updated")?,
            });
        }

        Ok(aggregates)
    }

    pub async fn create_trend_snapshot(&self, token: &str, source: &str) -> Result<(), CacheError> {
        let posts = self
            .get_cached_posts(Some(source), Some(token), Some(1000))
            .await?;

        if posts.is_empty() {
            return Ok(());
        }

        let mention_count = posts.len() as i32;
        let total_sentiment: f32 = posts.iter().map(|p| p.sentiment.score).sum();
        let avg_sentiment = total_sentiment / mention_count as f32;
        let engagement_total: i64 = posts.iter().map(|p| p.engagement as i64).sum();
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO trend_snapshots (id, token, source, snapshot_time, mention_count, sentiment_score, engagement_total)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(token)
        .bind(source)
        .bind(now)
        .bind(mention_count)
        .bind(avg_sentiment)
        .bind(engagement_total)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_trend_snapshots(
        &self,
        token: &str,
        hours: Option<i64>,
    ) -> Result<Vec<TrendSnapshot>, CacheError> {
        let hours = hours.unwrap_or(24);
        let cutoff = Utc::now().timestamp() - (hours * 3600);

        let rows = sqlx::query(
            "SELECT * FROM trend_snapshots WHERE token = ?1 AND snapshot_time >= ?2 ORDER BY snapshot_time ASC"
        )
        .bind(token)
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;

        let mut snapshots = Vec::new();
        for row in rows {
            snapshots.push(TrendSnapshot {
                id: row.try_get("id")?,
                token: row.try_get("token")?,
                source: row.try_get("source")?,
                snapshot_time: row.try_get("snapshot_time")?,
                mention_count: row.try_get("mention_count")?,
                sentiment_score: row.try_get("sentiment_score")?,
                engagement_total: row.try_get("engagement_total")?,
            });
        }

        Ok(snapshots)
    }

    pub async fn cleanup_old_posts(&self, days: i64) -> Result<i64, CacheError> {
        let cutoff = Utc::now().timestamp() - (days * 86400);

        let result = sqlx::query("DELETE FROM social_posts WHERE timestamp < ?1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i64)
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
