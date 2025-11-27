use serde::{Deserialize, Serialize};

/// Sentiment analysis result for a piece of text
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SentimentResult {
    pub score: f32,
    pub label: String,
    pub confidence: f32,
}

/// Social media post with sentiment analysis
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SocialPost {
    pub id: String,
    pub text: String,
    pub source: String,
    pub author: String,
    pub timestamp: i64,
    pub sentiment: SentimentResult,
    pub engagement: i32,
}

/// Rate limit information from social platforms
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateLimitInfo {
    pub limit: Option<i32>,
    pub remaining: Option<i32>,
    pub used: Option<i32>,
    pub reset_after_seconds: Option<i64>,
}

/// Metadata for social data fetch operations
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchMetadata {
    pub source: String,
    pub query: String,
    pub fetched_at: i64,
    pub result_count: usize,
    pub rate_limit: RateLimitInfo,
}

/// Result of a social data fetch operation combining posts and metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SocialFetchResult {
    pub posts: Vec<SocialPost>,
    pub metadata: FetchMetadata,
}
