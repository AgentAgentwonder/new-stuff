use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

use crate::security::keystore::Keystore;

use super::cache::{MentionAggregate, SocialCache, TrendSnapshot};
use super::models::{SocialFetchResult, SocialPost};
use super::reddit::RedditClient;
use super::twitter::TwitterClient;
use super::SocialError;

pub type SharedSocialDataService = Arc<RwLock<SocialDataService>>;

pub struct SocialDataService {
    reddit_client: RedditClient,
    twitter_client: TwitterClient,
    cache: SocialCache,
}

impl SocialDataService {
    pub async fn new(app: &AppHandle) -> Result<Self, SocialError> {
        let reddit_client = RedditClient::new().map_err(SocialError::from)?;
        let twitter_client = TwitterClient::new().map_err(SocialError::from)?;

        let mut data_dir = app.path().app_data_dir().map_err(|err| {
            SocialError::Internal("Failed to resolve app data directory".to_string())
        })?;

        std::fs::create_dir_all(&data_dir).map_err(|e| {
            SocialError::Internal(format!("Failed to ensure app data directory exists: {e}"))
        })?;

        data_dir.push("social");
        std::fs::create_dir_all(&data_dir).map_err(|e| {
            SocialError::Internal(format!("Failed to create social data directory: {e}"))
        })?;

        let cache = SocialCache::new(data_dir).await?;

        Ok(Self {
            reddit_client,
            twitter_client,
            cache,
        })
    }

    pub async fn fetch_reddit(
        &self,
        subreddit: &str,
        query: Option<&str>,
        limit: Option<u32>,
        token: Option<&str>,
    ) -> Result<SocialFetchResult, SocialError> {
        let result = self
            .reddit_client
            .fetch_subreddit_posts(subreddit, query, limit)
            .await?;

        self.cache.store_posts(&result.posts, token).await?;

        Ok(result)
    }

    pub async fn search_reddit_mentions(
        &self,
        subreddits: &[&str],
        keyword: &str,
        limit: Option<u32>,
        token: Option<&str>,
    ) -> Result<Vec<SocialFetchResult>, SocialError> {
        let mut aggregated = Vec::new();

        for result in self
            .reddit_client
            .search_mentions(subreddits, keyword, limit)
            .await?
        {
            self.cache.store_posts(&result.posts, token).await?;
            aggregated.push(result);
        }

        Ok(aggregated)
    }

    pub async fn fetch_twitter(
        &self,
        query: &str,
        max_results: Option<u32>,
        token_address: Option<&str>,
        bearer_override: Option<&str>,
        keystore: Option<&Keystore>,
    ) -> Result<SocialFetchResult, SocialError> {
        let bearer_token = self.resolve_bearer_token(bearer_override, keystore)?;
        let result = self
            .twitter_client
            .search_tweets(query, &bearer_token, max_results)
            .await?;

        self.cache.store_posts(&result.posts, token_address).await?;

        Ok(result)
    }

    pub async fn fetch_twitter_user(
        &self,
        username: &str,
        max_results: Option<u32>,
        token_address: Option<&str>,
        bearer_override: Option<&str>,
        keystore: Option<&Keystore>,
    ) -> Result<SocialFetchResult, SocialError> {
        let bearer_token = self.resolve_bearer_token(bearer_override, keystore)?;
        let result = self
            .twitter_client
            .search_user_tweets(username, &bearer_token, max_results)
            .await?;

        self.cache.store_posts(&result.posts, token_address).await?;

        Ok(result)
    }

    fn resolve_bearer_token(
        &self,
        override_token: Option<&str>,
        keystore: Option<&Keystore>,
    ) -> Result<String, SocialError> {
        if let Some(token) = override_token {
            return Ok(token.to_string());
        }

        let store = keystore.ok_or_else(|| {
            SocialError::Internal(
                "Keystore state unavailable for Twitter authentication".to_string(),
            )
        })?;

        TwitterClient::get_bearer_token_from_keystore(store).map_err(SocialError::from)
    }

    pub fn set_twitter_bearer_token(
        &self,
        keystore: &Keystore,
        token: &str,
    ) -> Result<(), SocialError> {
        TwitterClient::save_bearer_token_to_keystore(keystore, token).map_err(SocialError::from)
    }

    pub async fn get_cached_posts(
        &self,
        source: Option<&str>,
        token: Option<&str>,
        limit: Option<i32>,
    ) -> Result<Vec<SocialPost>, SocialError> {
        self.cache
            .get_cached_posts(source, token, limit)
            .await
            .map_err(Into::into)
    }

    pub async fn get_mention_aggregates(
        &self,
        token: Option<&str>,
    ) -> Result<Vec<MentionAggregate>, SocialError> {
        self.cache
            .get_mention_aggregates(token)
            .await
            .map_err(Into::into)
    }

    pub async fn get_trend_snapshots(
        &self,
        token: &str,
        hours: Option<i64>,
    ) -> Result<Vec<TrendSnapshot>, SocialError> {
        self.cache
            .get_trend_snapshots(token, hours)
            .await
            .map_err(Into::into)
    }

    pub async fn create_trend_snapshot(
        &self,
        token: &str,
        source: &str,
    ) -> Result<(), SocialError> {
        self.cache
            .create_trend_snapshot(token, source)
            .await
            .map_err(Into::into)
    }

    pub async fn cleanup_old_posts(&self, days: i64) -> Result<i64, SocialError> {
        self.cache.cleanup_old_posts(days).await.map_err(Into::into)
    }

    pub async fn schedule_refresh(
        &self,
        token: &str,
        interval_minutes: u64,
    ) -> Result<(), SocialError> {
        tracing::info!(
            token,
            interval = interval_minutes,
            "Simulated scheduling of social data refresh"
        );
        Ok(())
    }
}
