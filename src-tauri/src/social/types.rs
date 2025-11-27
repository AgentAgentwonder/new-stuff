// Social Trading Types
// Strategy marketplace, trader profiles, and leaderboards

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraderProfile {
    pub id: String,
    pub wallet_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub total_trades: i32,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum SocialError {
    #[error("social error: {0}")]
    General(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("reddit error: {0}")]
    Reddit(#[from] crate::social::reddit::RedditError),

    #[error("twitter error: {0}")]
    Twitter(#[from] crate::social::twitter::TwitterError),

    #[error("cache error: {0}")]
    Cache(#[from] crate::social::cache::CacheError),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type SocialResult<T> = Result<T, SocialError>;
