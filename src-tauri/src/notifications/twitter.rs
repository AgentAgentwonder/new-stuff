use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::security::keystore::Keystore;

const TWITTER_DB_FILE: &str = "twitter_integration.db";
const KEY_TWITTER_CONFIG: &str = "twitter_api_credentials";
const TWITTER_API_BASE: &str = "https://api.twitter.com/2";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterConfig {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_secret: String,
    pub bearer_token: String,
    pub enabled: bool,
    pub auto_tweet_enabled: bool,
    pub sentiment_tracking_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterSentimentKeyword {
    pub id: String,
    pub keyword: String,
    pub category: String,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterInfluencer {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub follower_count: Option<i64>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterSentimentData {
    pub id: String,
    pub keyword: String,
    pub sentiment_score: f64,
    pub positive_count: i32,
    pub neutral_count: i32,
    pub negative_count: i32,
    pub total_mentions: i32,
    pub trending: bool,
    pub fetched_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoTweetConfig {
    pub milestone_tweets: bool,
    pub price_alert_tweets: bool,
    pub portfolio_updates: bool,
    pub consent_given: bool,
    pub consent_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweetRecord {
    pub id: String,
    pub tweet_id: Option<String>,
    pub content: String,
    pub tweet_type: String,
    pub status: TweetStatus,
    pub error: Option<String>,
    pub posted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TweetStatus {
    Pending,
    Posted,
    Failed,
}

impl TweetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TweetStatus::Pending => "pending",
            TweetStatus::Posted => "posted",
            TweetStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TweetStatus::Pending),
            "posted" => Some(TweetStatus::Posted),
            "failed" => Some(TweetStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterStats {
    pub total_tweets_posted: i64,
    pub total_sentiment_checks: i64,
    pub tracked_keywords: i64,
    pub tracked_influencers: i64,
    pub average_sentiment_score: f64,
    pub last_24h_tweets: i64,
    pub last_sentiment_check: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TwitterError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("configuration not found")]
    ConfigNotFound,
    #[error("consent not given")]
    ConsentNotGiven,
    #[error("twitter api error: {0}")]
    TwitterApi(String),
    #[error("internal error: {0}")]
    Internal(String),
}

// Internal Twitter API response types
#[derive(Debug, Deserialize)]
struct TwitterSearchResponse {
    data: Option<Vec<TwitterTweet>>,
    meta: TwitterSearchMeta,
}

#[derive(Debug, Deserialize)]
struct TwitterTweet {
    id: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct TwitterSearchMeta {
    result_count: i32,
}

#[derive(Debug, Serialize)]
struct TwitterPostRequest {
    text: String,
}

#[derive(Debug, Deserialize)]
struct TwitterPostResponse {
    data: TwitterTweetData,
}

#[derive(Debug, Deserialize)]
struct TwitterTweetData {
    id: String,
    text: String,
}

#[derive(Clone)]
pub struct TwitterManager {
    pool: Pool<Sqlite>,
    app_handle: AppHandle,
    client: Client,
}

pub type SharedTwitterManager = Arc<RwLock<TwitterManager>>;

impl TwitterManager {
    pub async fn new(app: &AppHandle) -> Result<Self, TwitterError> {
        let db_path = twitter_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self {
            pool,
            app_handle: app.clone(),
            client: Client::new(),
        };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), TwitterError> {
        // Sentiment keywords table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twitter_keywords (
                id TEXT PRIMARY KEY,
                keyword TEXT NOT NULL,
                category TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Influencers table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twitter_influencers (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                follower_count INTEGER,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Sentiment data table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twitter_sentiment_data (
                id TEXT PRIMARY KEY,
                keyword TEXT NOT NULL,
                sentiment_score REAL NOT NULL,
                positive_count INTEGER NOT NULL,
                neutral_count INTEGER NOT NULL,
                negative_count INTEGER NOT NULL,
                total_mentions INTEGER NOT NULL,
                trending INTEGER NOT NULL DEFAULT 0,
                fetched_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Tweet records table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twitter_tweet_records (
                id TEXT PRIMARY KEY,
                tweet_id TEXT,
                content TEXT NOT NULL,
                tweet_type TEXT NOT NULL,
                status TEXT NOT NULL,
                error TEXT,
                posted_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sentiment_keyword ON twitter_sentiment_data(keyword);
            CREATE INDEX IF NOT EXISTS idx_sentiment_fetched ON twitter_sentiment_data(fetched_at);
            CREATE INDEX IF NOT EXISTS idx_tweets_status ON twitter_tweet_records(status);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_config(
        &self,
        config: TwitterConfig,
        keystore: &Keystore,
    ) -> Result<(), TwitterError> {
        let serialized = serde_json::to_vec(&config)?;
        keystore
            .store_secret(KEY_TWITTER_CONFIG, &serialized)
            .map_err(|e| TwitterError::Internal(format!("Failed to store config: {}", e)))?;
        Ok(())
    }

    pub async fn get_config(&self, keystore: &Keystore) -> Result<TwitterConfig, TwitterError> {
        let data = keystore
            .retrieve_secret(KEY_TWITTER_CONFIG)
            .map_err(|_| TwitterError::ConfigNotFound)?;
        let config: TwitterConfig = serde_json::from_slice(&data)?;
        Ok(config)
    }

    pub async fn delete_config(&self, keystore: &Keystore) -> Result<(), TwitterError> {
        keystore
            .remove_secret(KEY_TWITTER_CONFIG)
            .map_err(|e| TwitterError::Internal(format!("Failed to delete config: {}", e)))?;
        Ok(())
    }

    pub async fn test_connection(&self, config: &TwitterConfig) -> Result<String, TwitterError> {
        // Test by fetching user info
        let url = format!("{}/users/me", TWITTER_API_BASE);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&config.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok("Twitter API connection successful".to_string())
        } else {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(TwitterError::TwitterApi(error))
        }
    }

    // Keyword management
    pub async fn add_keyword(
        &self,
        keyword: String,
        category: String,
    ) -> Result<TwitterSentimentKeyword, TwitterError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO twitter_keywords (id, keyword, category, enabled, created_at)
            VALUES (?1, ?2, ?3, 1, ?4)
            "#,
        )
        .bind(&id)
        .bind(&keyword)
        .bind(&category)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(TwitterSentimentKeyword {
            id,
            keyword,
            category,
            enabled: true,
            created_at: now,
        })
    }

    pub async fn list_keywords(&self) -> Result<Vec<TwitterSentimentKeyword>, TwitterError> {
        let rows = sqlx::query(
            "SELECT id, keyword, category, enabled, created_at FROM twitter_keywords ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut keywords = Vec::new();
        for row in rows {
            let enabled: i32 = row.try_get("enabled")?;
            keywords.push(TwitterSentimentKeyword {
                id: row.try_get("id")?,
                keyword: row.try_get("keyword")?,
                category: row.try_get("category")?,
                enabled: enabled != 0,
                created_at: row.try_get("created_at")?,
            });
        }

        Ok(keywords)
    }

    pub async fn remove_keyword(&self, id: &str) -> Result<(), TwitterError> {
        sqlx::query("DELETE FROM twitter_keywords WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Influencer management
    pub async fn add_influencer(
        &self,
        username: String,
        display_name: String,
    ) -> Result<TwitterInfluencer, TwitterError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO twitter_influencers (id, username, display_name, enabled, created_at)
            VALUES (?1, ?2, ?3, 1, ?4)
            "#,
        )
        .bind(&id)
        .bind(&username)
        .bind(&display_name)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(TwitterInfluencer {
            id,
            username,
            display_name,
            follower_count: None,
            enabled: true,
            created_at: now,
        })
    }

    pub async fn list_influencers(&self) -> Result<Vec<TwitterInfluencer>, TwitterError> {
        let rows = sqlx::query(
            "SELECT id, username, display_name, follower_count, enabled, created_at FROM twitter_influencers ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut influencers = Vec::new();
        for row in rows {
            let enabled: i32 = row.try_get("enabled")?;
            influencers.push(TwitterInfluencer {
                id: row.try_get("id")?,
                username: row.try_get("username")?,
                display_name: row.try_get("display_name")?,
                follower_count: row.try_get("follower_count")?,
                enabled: enabled != 0,
                created_at: row.try_get("created_at")?,
            });
        }

        Ok(influencers)
    }

    pub async fn remove_influencer(&self, id: &str) -> Result<(), TwitterError> {
        sqlx::query("DELETE FROM twitter_influencers WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Sentiment fetching
    pub async fn fetch_sentiment(
        &self,
        keyword: &str,
        config: &TwitterConfig,
    ) -> Result<TwitterSentimentData, TwitterError> {
        if !config.sentiment_tracking_enabled {
            return Err(TwitterError::Internal(
                "Sentiment tracking is disabled".to_string(),
            ));
        }

        let url = format!("{}/tweets/search/recent", TWITTER_API_BASE);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&config.bearer_token)
            .query(&[
                ("query", keyword),
                ("max_results", "100"),
                ("tweet.fields", "created_at,public_metrics"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TwitterError::TwitterApi(error));
        }

        let search_result: TwitterSearchResponse = response.json().await?;

        // Simple sentiment analysis (in real implementation, use ML model)
        let tweets = search_result.data.unwrap_or_default();
        let total_mentions = tweets.len() as i32;

        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut neutral_count = 0;

        for tweet in &tweets {
            let sentiment = self.analyze_tweet_sentiment(&tweet.text);
            match sentiment {
                s if s > 0.2 => positive_count += 1,
                s if s < -0.2 => negative_count += 1,
                _ => neutral_count += 1,
            }
        }

        let sentiment_score = if total_mentions > 0 {
            ((positive_count as f64 - negative_count as f64) / total_mentions as f64) * 100.0
        } else {
            0.0
        };

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO twitter_sentiment_data (
                id, keyword, sentiment_score, positive_count, neutral_count,
                negative_count, total_mentions, trending, fetched_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&id)
        .bind(keyword)
        .bind(sentiment_score)
        .bind(positive_count)
        .bind(neutral_count)
        .bind(negative_count)
        .bind(total_mentions)
        .bind(total_mentions > 50)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(TwitterSentimentData {
            id,
            keyword: keyword.to_string(),
            sentiment_score,
            positive_count,
            neutral_count,
            negative_count,
            total_mentions,
            trending: total_mentions > 50,
            fetched_at: now,
        })
    }

    fn analyze_tweet_sentiment(&self, text: &str) -> f64 {
        // Simple keyword-based sentiment analysis
        let positive_keywords = [
            "bullish",
            "moon",
            "great",
            "excellent",
            "amazing",
            "love",
            "best",
            "win",
            "profit",
            "gain",
        ];
        let negative_keywords = [
            "bearish", "dump", "bad", "worst", "terrible", "hate", "loss", "crash", "scam", "rug",
        ];

        let text_lower = text.to_lowercase();
        let positive = positive_keywords
            .iter()
            .filter(|&kw| text_lower.contains(kw))
            .count() as f64;
        let negative = negative_keywords
            .iter()
            .filter(|&kw| text_lower.contains(kw))
            .count() as f64;

        if positive + negative > 0.0 {
            (positive - negative) / (positive + negative)
        } else {
            0.0
        }
    }

    pub async fn get_sentiment_history(
        &self,
        keyword: &str,
        limit: i32,
    ) -> Result<Vec<TwitterSentimentData>, TwitterError> {
        let rows = sqlx::query(
            r#"
            SELECT id, keyword, sentiment_score, positive_count, neutral_count,
                   negative_count, total_mentions, trending, fetched_at
            FROM twitter_sentiment_data
            WHERE keyword = ?1
            ORDER BY fetched_at DESC
            LIMIT ?2
            "#,
        )
        .bind(keyword)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut data = Vec::new();
        for row in rows {
            let trending: i32 = row.try_get("trending")?;
            data.push(TwitterSentimentData {
                id: row.try_get("id")?,
                keyword: row.try_get("keyword")?,
                sentiment_score: row.try_get("sentiment_score")?,
                positive_count: row.try_get("positive_count")?,
                neutral_count: row.try_get("neutral_count")?,
                negative_count: row.try_get("negative_count")?,
                total_mentions: row.try_get("total_mentions")?,
                trending: trending != 0,
                fetched_at: row.try_get("fetched_at")?,
            });
        }

        Ok(data)
    }

    // Auto-tweet functionality
    pub async fn post_tweet(
        &self,
        content: String,
        tweet_type: String,
        config: &TwitterConfig,
        auto_tweet_config: &AutoTweetConfig,
    ) -> Result<TweetRecord, TwitterError> {
        if !config.auto_tweet_enabled {
            return Err(TwitterError::Internal("Auto-tweet is disabled".to_string()));
        }

        if !auto_tweet_config.consent_given {
            return Err(TwitterError::ConsentNotGiven);
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Post to Twitter
        let url = format!("{}/tweets", TWITTER_API_BASE);
        let post_data = TwitterPostRequest {
            text: content.clone(),
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&config.bearer_token)
            .json(&post_data)
            .send()
            .await?;

        let (status, tweet_id, error) = if response.status().is_success() {
            let post_response: TwitterPostResponse = response.json().await?;
            (TweetStatus::Posted, Some(post_response.data.id), None)
        } else {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            (TweetStatus::Failed, None, Some(error_msg))
        };

        sqlx::query(
            r#"
            INSERT INTO twitter_tweet_records (id, tweet_id, content, tweet_type, status, error, posted_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&id)
        .bind(&tweet_id)
        .bind(&content)
        .bind(&tweet_type)
        .bind(status.as_str())
        .bind(&error)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(TweetRecord {
            id,
            tweet_id,
            content,
            tweet_type,
            status,
            error,
            posted_at: now,
        })
    }

    pub async fn get_tweet_history(&self, limit: i32) -> Result<Vec<TweetRecord>, TwitterError> {
        let rows = sqlx::query(
            r#"
            SELECT id, tweet_id, content, tweet_type, status, error, posted_at
            FROM twitter_tweet_records
            ORDER BY posted_at DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut records = Vec::new();
        for row in rows {
            let status_str: String = row.try_get("status")?;
            let status = TweetStatus::from_str(&status_str).unwrap_or(TweetStatus::Failed);

            records.push(TweetRecord {
                id: row.try_get("id")?,
                tweet_id: row.try_get("tweet_id")?,
                content: row.try_get("content")?,
                tweet_type: row.try_get("tweet_type")?,
                status,
                error: row.try_get("error")?,
                posted_at: row.try_get("posted_at")?,
            });
        }

        Ok(records)
    }

    pub async fn get_stats(&self) -> Result<TwitterStats, TwitterError> {
        let total_tweets = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM twitter_tweet_records WHERE status = 'posted'",
        )
        .fetch_one(&self.pool)
        .await?;

        let total_sentiment =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM twitter_sentiment_data")
                .fetch_one(&self.pool)
                .await?;

        let tracked_keywords =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM twitter_keywords WHERE enabled = 1")
                .fetch_one(&self.pool)
                .await?;

        let tracked_influencers = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM twitter_influencers WHERE enabled = 1",
        )
        .fetch_one(&self.pool)
        .await?;

        let avg_sentiment = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT AVG(sentiment_score) FROM twitter_sentiment_data WHERE datetime(fetched_at) > datetime('now', '-7 days')"
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0.0);

        let last_24h_tweets = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM twitter_tweet_records WHERE status = 'posted' AND datetime(posted_at) > datetime('now', '-1 day')"
        )
        .fetch_one(&self.pool)
        .await?;

        let last_sentiment_check = sqlx::query_scalar::<_, Option<String>>(
            "SELECT fetched_at FROM twitter_sentiment_data ORDER BY fetched_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(TwitterStats {
            total_tweets_posted: total_tweets,
            total_sentiment_checks: total_sentiment,
            tracked_keywords,
            tracked_influencers,
            average_sentiment_score: avg_sentiment,
            last_24h_tweets,
            last_sentiment_check,
        })
    }
}

fn twitter_db_path(app: &AppHandle) -> Result<PathBuf, TwitterError> {
    let app_dir = app.path().app_data_dir().map_err(|err| {
        TwitterError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;

    std::fs::create_dir_all(&app_dir).map_err(|e| {
        TwitterError::Internal(format!("Failed to create app data directory: {}", e))
    })?;

    Ok(app_dir.join(TWITTER_DB_FILE))
}

// Tauri Commands
#[tauri::command]
pub async fn twitter_save_config(
    config: TwitterConfig,
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<String, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .save_config(config, &keystore)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Twitter configuration saved successfully".to_string())
}

#[tauri::command]
pub async fn twitter_get_config(
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<TwitterConfig, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .get_config(&keystore)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_delete_config(
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<String, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .delete_config(&keystore)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Twitter configuration deleted successfully".to_string())
}

#[tauri::command]
pub async fn twitter_test_connection(
    config: TwitterConfig,
    app: AppHandle,
) -> Result<String, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .test_connection(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_add_keyword(
    keyword: String,
    category: String,
    app: AppHandle,
) -> Result<TwitterSentimentKeyword, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .add_keyword(keyword, category)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_list_keywords(app: AppHandle) -> Result<Vec<TwitterSentimentKeyword>, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager.list_keywords().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_remove_keyword(id: String, app: AppHandle) -> Result<String, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .remove_keyword(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Keyword removed successfully".to_string())
}

#[tauri::command]
pub async fn twitter_add_influencer(
    username: String,
    display_name: String,
    app: AppHandle,
) -> Result<TwitterInfluencer, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .add_influencer(username, display_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_list_influencers(app: AppHandle) -> Result<Vec<TwitterInfluencer>, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager.list_influencers().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_remove_influencer(id: String, app: AppHandle) -> Result<String, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .remove_influencer(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Influencer removed successfully".to_string())
}

#[tauri::command]
pub async fn twitter_fetch_sentiment(
    keyword: String,
    keystore: State<'_, Keystore>,
    app: AppHandle,
) -> Result<TwitterSentimentData, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    let config = manager
        .get_config(&keystore)
        .await
        .map_err(|e| e.to_string())?;

    manager
        .fetch_sentiment(&keyword, &config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_get_sentiment_history(
    keyword: String,
    limit: i32,
    app: AppHandle,
) -> Result<Vec<TwitterSentimentData>, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .get_sentiment_history(&keyword, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_get_stats(app: AppHandle) -> Result<TwitterStats, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager.get_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn twitter_get_tweet_history(
    limit: i32,
    app: AppHandle,
) -> Result<Vec<TweetRecord>, String> {
    let manager = TwitterManager::new(&app).await.map_err(|e| e.to_string())?;

    manager
        .get_tweet_history(limit)
        .await
        .map_err(|e| e.to_string())
}
