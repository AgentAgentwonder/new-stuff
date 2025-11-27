use httpmock::prelude::*;
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn test_reddit_client_normalizes_posts() {
    use app_lib::social::models::SocialFetchResult;
    use app_lib::social::reddit::RedditClient;

    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/r/CryptoCurrency/new.json")
            .query_param("limit", "10");
        then.status(200)
            .header("content-type", "application/json")
            .header("x-ratelimit-limit", "600")
            .header("x-ratelimit-remaining", "598")
            .header("x-ratelimit-reset", "60")
            .json_body(json!({
                "data": {
                    "children": [
                        {
                            "data": {
                                "id": "abc123",
                                "title": "Bitcoin going to the moon!",
                                "selftext": "",
                                "author": "crypto_fan",
                                "created_utc": 1672531200.0,
                                "score": 42,
                                "num_comments": 10,
                                "subreddit": "CryptoCurrency"
                            }
                        },
                        {
                            "data": {
                                "id": "def456",
                                "title": "Market crash incoming",
                                "selftext": "This is terrible news",
                                "author": "bear_hunter",
                                "created_utc": 1672531300.0,
                                "score": 15,
                                "num_comments": 5,
                                "subreddit": "CryptoCurrency"
                            }
                        }
                    ]
                }
            }));
    });

    let client =
        RedditClient::with_base_url(server.base_url()).expect("Failed to create reddit client");
    let result: SocialFetchResult = client
        .fetch_subreddit_posts("CryptoCurrency", None, Some(10))
        .await
        .expect("Reddit fetch should succeed");

    assert_eq!(result.posts.len(), 2);
    assert_eq!(result.posts[0].source, "reddit/r/CryptoCurrency");
    assert_eq!(result.metadata.result_count, 2);
    assert_eq!(result.metadata.rate_limit.limit, Some(600));
    assert_eq!(result.metadata.rate_limit.remaining, Some(598));

    mock.assert();
}

#[tokio::test]
async fn test_twitter_rate_limiter_blocks_excessive_requests() {
    use app_lib::social::twitter::{TwitterClient, TwitterError};

    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(GET).path("/tweets/search/recent");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "data": [],
                "meta": { "result_count": 0 }
            }));
    });

    let client = TwitterClient::with_base_url_and_limit(server.base_url(), 10)
        .expect("Failed to create Twitter client");

    let mut successful_requests = 0;
    let mut blocked_requests = 0;

    for _ in 0..15 {
        match client
            .search_tweets("bitcoin", "fake_token", Some(10))
            .await
        {
            Ok(_) => successful_requests += 1,
            Err(TwitterError::RateLimitExceeded) => blocked_requests += 1,
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    assert!(
        successful_requests <= 10,
        "Rate limiter allowed too many requests: {}",
        successful_requests
    );
    assert!(
        blocked_requests > 0,
        "Rate limiter did not block any requests"
    );
}

#[tokio::test]
async fn test_cache_stores_and_retrieves_posts() {
    use app_lib::social::cache::SocialCache;
    use app_lib::social::models::{SentimentResult, SocialPost};
    use chrono::Utc;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let posts = vec![
        SocialPost {
            id: "post_1".to_string(),
            text: "Bitcoin is doing great!".to_string(),
            source: "reddit".to_string(),
            author: "user1".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: SentimentResult {
                score: 0.8,
                label: "positive".to_string(),
                confidence: 0.9,
            },
            engagement: 100,
        },
        SocialPost {
            id: "post_2".to_string(),
            text: "Market looks bearish".to_string(),
            source: "twitter".to_string(),
            author: "user2".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: SentimentResult {
                score: -0.6,
                label: "negative".to_string(),
                confidence: 0.85,
            },
            engagement: 50,
        },
    ];

    cache
        .store_posts(&posts, Some("SOL_TOKEN_ADDRESS"))
        .await
        .expect("Failed to store posts");

    let retrieved = cache
        .get_cached_posts(None, Some("SOL_TOKEN_ADDRESS"), Some(10))
        .await
        .expect("Failed to retrieve posts");

    assert_eq!(retrieved.len(), 2);
    assert_eq!(retrieved[0].id, "post_1");
    assert_eq!(retrieved[1].id, "post_2");
}

#[tokio::test]
async fn test_mention_aggregates_calculation() {
    use app_lib::social::cache::SocialCache;
    use app_lib::social::models::{SentimentResult, SocialPost};
    use chrono::Utc;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let posts = vec![
        SocialPost {
            id: "post_1".to_string(),
            text: "Positive news".to_string(),
            source: "reddit".to_string(),
            author: "user1".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: SentimentResult {
                score: 0.7,
                label: "positive".to_string(),
                confidence: 0.9,
            },
            engagement: 100,
        },
        SocialPost {
            id: "post_2".to_string(),
            text: "Negative outlook".to_string(),
            source: "reddit".to_string(),
            author: "user2".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: SentimentResult {
                score: -0.5,
                label: "negative".to_string(),
                confidence: 0.8,
            },
            engagement: 50,
        },
        SocialPost {
            id: "post_3".to_string(),
            text: "Neutral statement".to_string(),
            source: "reddit".to_string(),
            author: "user3".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: SentimentResult {
                score: 0.1,
                label: "neutral".to_string(),
                confidence: 0.6,
            },
            engagement: 25,
        },
    ];

    cache
        .store_posts(&posts, Some("TEST_TOKEN"))
        .await
        .expect("Failed to store posts");

    let aggregates = cache
        .get_mention_aggregates(Some("TEST_TOKEN"))
        .await
        .expect("Failed to get aggregates");

    assert_eq!(aggregates.len(), 1);
    let agg = &aggregates[0];
    assert_eq!(agg.token, "TEST_TOKEN");
    assert_eq!(agg.mention_count, 3);
    assert_eq!(agg.positive_count, 1);
    assert_eq!(agg.negative_count, 1);
    assert_eq!(agg.neutral_count, 1);
}

#[tokio::test]
async fn test_trend_snapshot_creation() {
    use app_lib::social::cache::SocialCache;
    use app_lib::social::models::{SentimentResult, SocialPost};
    use chrono::Utc;

    let temp_dir = tempdir().expect("Failed to create temp directory");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let posts = vec![
        SocialPost {
            id: "post_1".to_string(),
            text: "Great project!".to_string(),
            source: "twitter".to_string(),
            author: "user1".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: SentimentResult {
                score: 0.8,
                label: "positive".to_string(),
                confidence: 0.9,
            },
            engagement: 200,
        },
        SocialPost {
            id: "post_2".to_string(),
            text: "Looks promising".to_string(),
            source: "twitter".to_string(),
            author: "user2".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: SentimentResult {
                score: 0.6,
                label: "positive".to_string(),
                confidence: 0.85,
            },
            engagement: 150,
        },
    ];

    cache
        .store_posts(&posts, Some("TREND_TOKEN"))
        .await
        .expect("Failed to store posts");

    cache
        .create_trend_snapshot("TREND_TOKEN", "twitter")
        .await
        .expect("Failed to create trend snapshot");

    let snapshots = cache
        .get_trend_snapshots("TREND_TOKEN", Some(24))
        .await
        .expect("Failed to get snapshots");

    assert_eq!(snapshots.len(), 1);
    let snapshot = &snapshots[0];
    assert_eq!(snapshot.token, "TREND_TOKEN");
    assert_eq!(snapshot.mention_count, 2);
    assert_eq!(snapshot.engagement_total, 350);
    assert!(snapshot.sentiment_score > 0.0);
}
