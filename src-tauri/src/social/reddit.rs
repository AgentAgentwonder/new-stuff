use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::sentiment::analyze_sentiment;

use super::models::{FetchMetadata, RateLimitInfo, SocialFetchResult, SocialPost};

const REDDIT_BASE_URL: &str = "https://www.reddit.com";
const USER_AGENT: &str = "eclipse-market-pro:v0.1.0";

#[derive(Debug, Deserialize)]
struct RedditResponse {
    data: RedditListingData,
}

#[derive(Debug, Deserialize)]
struct RedditListingData {
    children: Vec<RedditChild>,
    after: Option<String>,
    before: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPost,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    id: String,
    title: String,
    selftext: String,
    author: String,
    created_utc: f64,
    score: i32,
    num_comments: i32,
    subreddit: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RedditError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("rate limit exceeded")]
    RateLimitExceeded,
    #[error("parse error: {0}")]
    Parse(String),
}

pub struct RedditClient {
    client: Client,
    base_url: String,
}

impl RedditClient {
    pub fn new() -> Result<Self, RedditError> {
        Self::with_base_url(REDDIT_BASE_URL)
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, RedditError> {
        let client = Self::build_client()?;
        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    fn build_client() -> Result<Client, RedditError> {
        Ok(Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(10))
            .build()?)
    }

    pub async fn fetch_subreddit_posts(
        &self,
        subreddit: &str,
        query: Option<&str>,
        limit: Option<u32>,
    ) -> Result<SocialFetchResult, RedditError> {
        let limit = limit.unwrap_or(25).min(100);

        let url = match query {
            Some(q) => {
                format!(
                    "{}/r/{}/search.json?q={}&restrict_sr=1&limit={}&sort=new",
                    self.base_url,
                    subreddit,
                    urlencoding::encode(q),
                    limit
                )
            }
            None => {
                format!("{}/r/{}/new.json?limit={}", self.base_url, subreddit, limit)
            }
        };

        let response = self.client.get(&url).send().await?;

        let rate_limit = extract_rate_limit_info(&response);

        if response.status().as_u16() == 429 {
            return Err(RedditError::RateLimitExceeded);
        }

        let reddit_response: RedditResponse = response
            .json()
            .await
            .map_err(|e| RedditError::Parse(e.to_string()))?;

        let posts = reddit_response
            .data
            .children
            .into_iter()
            .map(|child| normalize_reddit_post(child.data))
            .collect::<Vec<_>>();

        let result_count = posts.len();
        let now = Utc::now().timestamp();

        Ok(SocialFetchResult {
            posts,
            metadata: FetchMetadata {
                source: format!("reddit:/r/{}", subreddit),
                query: query.unwrap_or("").to_string(),
                fetched_at: now,
                result_count,
                rate_limit,
            },
        })
    }

    pub async fn search_mentions(
        &self,
        subreddits: &[&str],
        keyword: &str,
        limit: Option<u32>,
    ) -> Result<Vec<SocialFetchResult>, RedditError> {
        let mut results = Vec::new();

        for subreddit in subreddits {
            match self
                .fetch_subreddit_posts(subreddit, Some(keyword), limit)
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!("Failed to fetch from r/{}: {}", subreddit, e);
                }
            }
        }

        Ok(results)
    }
}

fn normalize_reddit_post(post: RedditPost) -> SocialPost {
    let text = if post.selftext.is_empty() {
        post.title.clone()
    } else {
        format!("{} {}", post.title, post.selftext)
    };

    let sentiment = analyze_sentiment(&text);
    let engagement = post.score + post.num_comments;

    SocialPost {
        id: format!("reddit_{}", post.id),
        text,
        source: format!("reddit/r/{}", post.subreddit),
        author: post.author,
        timestamp: post.created_utc as i64,
        sentiment,
        engagement,
    }
}

fn extract_rate_limit_info(response: &reqwest::Response) -> RateLimitInfo {
    let headers = response.headers();

    let limit = headers
        .get("x-ratelimit-limit")
        .or_else(|| headers.get("ratelimit-limit"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok());

    let remaining = headers
        .get("x-ratelimit-remaining")
        .or_else(|| headers.get("ratelimit-remaining"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok());

    let used = headers
        .get("x-ratelimit-used")
        .or_else(|| headers.get("ratelimit-used"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i32>().ok());

    let reset_after = headers
        .get("x-ratelimit-reset")
        .or_else(|| headers.get("ratelimit-reset"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<f64>().ok())
        .map(|v| v.ceil() as i64);

    RateLimitInfo {
        limit,
        remaining,
        used,
        reset_after_seconds: reset_after,
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
