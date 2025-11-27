use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};

use crate::security::keystore::Keystore;
use crate::sentiment::analyze_sentiment;

use super::models::{FetchMetadata, RateLimitInfo, SocialFetchResult, SocialPost};

const TWITTER_API_BASE: &str = "https://api.twitter.com/2";
const KEY_TWITTER_BEARER: &str = "twitter_bearer_token";
const RATE_LIMIT_WINDOW_SECS: u64 = 900;
const MAX_REQUESTS_PER_WINDOW: usize = 450;

#[derive(Debug, Deserialize)]
struct TwitterSearchResponse {
    data: Option<Vec<TwitterTweet>>,
    meta: TwitterSearchMeta,
}

#[derive(Debug, Deserialize)]
struct TwitterTweet {
    id: String,
    text: String,
    author_id: Option<String>,
    created_at: Option<String>,
    public_metrics: Option<TwitterMetrics>,
}

#[derive(Debug, Deserialize)]
struct TwitterMetrics {
    retweet_count: i32,
    reply_count: i32,
    like_count: i32,
    quote_count: i32,
}

#[derive(Debug, Deserialize)]
struct TwitterSearchMeta {
    result_count: i32,
    newest_id: Option<String>,
    oldest_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TwitterError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("rate limit exceeded")]
    RateLimitExceeded,
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("bearer token not configured")]
    TokenNotConfigured,
    #[error("parse error: {0}")]
    Parse(String),
}

pub struct TwitterClient {
    client: Client,
    rate_limiter: Arc<Semaphore>,
    last_reset: Arc<RwLock<std::time::Instant>>,
    base_url: String,
    max_requests_per_window: usize,
}

impl TwitterClient {
    pub fn new() -> Result<Self, TwitterError> {
        Self::with_base_url_and_limit(TWITTER_API_BASE, MAX_REQUESTS_PER_WINDOW)
    }

    pub fn with_base_url_and_limit(
        base_url: impl Into<String>,
        max_requests_per_window: usize,
    ) -> Result<Self, TwitterError> {
        let max_requests_per_window = max_requests_per_window.max(1);
        let client = Client::builder()
            .user_agent("eclipse-market-pro/0.1.0")
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            client,
            rate_limiter: Arc::new(Semaphore::new(max_requests_per_window)),
            last_reset: Arc::new(RwLock::new(std::time::Instant::now())),
            base_url: base_url.into(),
            max_requests_per_window,
        })
    }

    async fn check_rate_limit(&self) -> Result<(), TwitterError> {
        let mut last_reset = self.last_reset.write().await;
        let elapsed = last_reset.elapsed();

        if elapsed.as_secs() >= RATE_LIMIT_WINDOW_SECS {
            *last_reset = std::time::Instant::now();
            let available_permits = self.rate_limiter.available_permits();
            if available_permits < self.max_requests_per_window {
                self.rate_limiter
                    .add_permits(self.max_requests_per_window - available_permits);
            }
        }

        Ok(())
    }

    async fn acquire_permit(&self) -> Result<tokio::sync::SemaphorePermit<'_>, TwitterError> {
        self.check_rate_limit().await?;

        match self.rate_limiter.try_acquire() {
            Ok(permit) => Ok(permit),
            Err(_) => Err(TwitterError::RateLimitExceeded),
        }
    }

    pub async fn search_tweets(
        &self,
        query: &str,
        bearer_token: &str,
        max_results: Option<u32>,
    ) -> Result<SocialFetchResult, TwitterError> {
        let _permit = self.acquire_permit().await?;

        let max_results = max_results.unwrap_or(10).min(100);

        let url = format!("{}/tweets/search/recent", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(bearer_token)
            .query(&[
                ("query", query),
                ("max_results", &max_results.to_string()),
                ("tweet.fields", "created_at,public_metrics,author_id"),
            ])
            .send()
            .await?;

        let rate_limit = extract_rate_limit_info(&response);

        if response.status().as_u16() == 429 {
            return Err(TwitterError::RateLimitExceeded);
        }

        if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown auth error".to_string());
            return Err(TwitterError::AuthenticationFailed(error_text));
        }

        let twitter_response: TwitterSearchResponse = response
            .json()
            .await
            .map_err(|e| TwitterError::Parse(e.to_string()))?;

        let posts = twitter_response
            .data
            .unwrap_or_default()
            .into_iter()
            .map(normalize_twitter_post)
            .collect::<Vec<_>>();

        let result_count = posts.len();
        let now = Utc::now().timestamp();

        Ok(SocialFetchResult {
            posts,
            metadata: FetchMetadata {
                source: "twitter".to_string(),
                query: query.to_string(),
                fetched_at: now,
                result_count,
                rate_limit,
            },
        })
    }

    pub async fn search_user_tweets(
        &self,
        username: &str,
        bearer_token: &str,
        max_results: Option<u32>,
    ) -> Result<SocialFetchResult, TwitterError> {
        let query = format!("from:{}", username);
        self.search_tweets(&query, bearer_token, max_results).await
    }

    pub fn get_bearer_token_from_keystore(keystore: &Keystore) -> Result<String, TwitterError> {
        let data = keystore
            .retrieve_secret(KEY_TWITTER_BEARER)
            .map_err(|_| TwitterError::TokenNotConfigured)?;

        String::from_utf8(data.to_vec())
            .map_err(|e| TwitterError::Parse(format!("Invalid UTF-8 in bearer token: {}", e)))
    }

    pub fn save_bearer_token_to_keystore(
        keystore: &Keystore,
        token: &str,
    ) -> Result<(), TwitterError> {
        keystore
            .store_secret(KEY_TWITTER_BEARER, token.as_bytes())
            .map_err(|e| TwitterError::Parse(format!("Failed to store bearer token: {}", e)))
    }
}

fn normalize_twitter_post(tweet: TwitterTweet) -> SocialPost {
    let sentiment = analyze_sentiment(&tweet.text);

    let engagement = tweet
        .public_metrics
        .as_ref()
        .map(|m| m.retweet_count + m.reply_count + m.like_count + m.quote_count)
        .unwrap_or(0);

    let timestamp = tweet
        .created_at
        .as_ref()
        .and_then(|dt| chrono::DateTime::parse_from_rfc3339(dt).ok())
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| Utc::now().timestamp());

    SocialPost {
        id: format!("twitter_{}", tweet.id),
        text: tweet.text,
        source: "twitter".to_string(),
        author: tweet.author_id.unwrap_or_else(|| "unknown".to_string()),
        timestamp,
        sentiment,
        engagement,
    }
}

fn extract_rate_limit_info(response: &reqwest::Response) -> RateLimitInfo {
    let headers = response.headers();

    let limit = headers
        .get("x-rate-limit-limit")
        .or_else(|| headers.get("x-ratelimit-limit"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok());

    let remaining = headers
        .get("x-rate-limit-remaining")
        .or_else(|| headers.get("x-ratelimit-remaining"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok());

    let reset = headers
        .get("x-rate-limit-reset")
        .or_else(|| headers.get("x-ratelimit-reset"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let reset_after_seconds = reset.map(|reset_timestamp| {
        let now = Utc::now().timestamp();
        (reset_timestamp - now).max(0)
    });

    RateLimitInfo {
        limit,
        remaining,
        used: limit.and_then(|l| remaining.map(|r| l - r)),
        reset_after_seconds,
    }
}
