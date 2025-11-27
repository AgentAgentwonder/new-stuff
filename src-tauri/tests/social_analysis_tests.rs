use app_lib::sentiment::analyze_sentiment;
use app_lib::social::analysis::*;
use app_lib::social::{SocialCache, SocialPost};
use chrono::Utc;
use tempfile::tempdir;

#[tokio::test]
async fn test_sentiment_engine_positive() {
    let engine = SentimentEngine::new();
    let result =
        engine.analyze_text("This is amazing! Great bullish project going to the moon! 🚀");

    assert!(
        result.score > 0.2,
        "Score should be positive: {}",
        result.score
    );
    assert_eq!(result.label, "positive");
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_sentiment_engine_negative() {
    let engine = SentimentEngine::new();
    let result = engine.analyze_text("Terrible scam! Dump this crash immediately! 📉");

    assert!(
        result.score < -0.2,
        "Score should be negative: {}",
        result.score
    );
    assert_eq!(result.label, "negative");
    assert!(result.confidence > 0.0);
}

#[tokio::test]
async fn test_sentiment_engine_neutral() {
    let engine = SentimentEngine::new();
    let result = engine.analyze_text("The price is stable today.");

    assert_eq!(result.label, "neutral");
}

#[tokio::test]
async fn test_sentiment_negation_handling() {
    let engine = SentimentEngine::new();
    let positive = engine.analyze_text("This is great");
    let negated = engine.analyze_text("This is not great");

    assert!(
        positive.score > negated.score,
        "Negation should reduce positive sentiment"
    );
}

#[tokio::test]
async fn test_sentiment_emoji_boost() {
    let engine = SentimentEngine::new();
    let without_emoji = engine.analyze_text("Let's go");
    let with_emoji = engine.analyze_text("Let's go 🚀🚀🚀");

    assert!(
        with_emoji.score > without_emoji.score,
        "Emojis should boost sentiment"
    );
}

#[tokio::test]
async fn test_sentiment_amplifiers() {
    let engine = SentimentEngine::new();
    let normal = engine.analyze_text("This is good");
    let amplified = engine.analyze_text("This is very good");

    assert!(
        amplified.score > normal.score,
        "Amplifiers should increase sentiment"
    );
}

#[tokio::test]
async fn test_sentiment_snapshot_computation() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let engine = SentimentEngine::new();
    let token = "TEST_TOKEN";
    let now = Utc::now().timestamp();

    let posts = vec![
        create_test_post("1", "Great project! Bullish!", now),
        create_test_post("2", "Amazing opportunity 🚀", now),
        create_test_post("3", "Love this token", now),
    ];

    let items: Vec<(String, SocialPost)> = posts
        .iter()
        .map(|p| (token.to_string(), p.clone()))
        .collect();

    engine
        .batch_analyze_posts(&items, cache.pool())
        .await
        .expect("Batch analysis should succeed");

    let snapshot = engine
        .compute_sentiment_snapshot(cache.pool(), token, None)
        .await
        .expect("Snapshot computation should succeed");

    assert_eq!(snapshot.token, token);
    assert_eq!(snapshot.mention_count, 3);
    assert!(snapshot.avg_score > 0.0, "Average score should be positive");
    assert_eq!(snapshot.dominant_label, "positive");
    assert_eq!(snapshot.positive_mentions, 3);
}

#[tokio::test]
async fn test_sentiment_snapshot_with_mixed_sentiment() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let engine = SentimentEngine::new();
    let token = "MIXED_TOKEN";
    let now = Utc::now().timestamp();

    let posts = vec![
        create_test_post("1", "Great project!", now),
        create_test_post("2", "Terrible scam", now),
        create_test_post("3", "Price update", now),
    ];

    let items: Vec<(String, SocialPost)> = posts
        .iter()
        .map(|p| (token.to_string(), p.clone()))
        .collect();

    engine
        .batch_analyze_posts(&items, cache.pool())
        .await
        .expect("Batch analysis should succeed");

    let snapshot = engine
        .compute_sentiment_snapshot(cache.pool(), token, None)
        .await
        .expect("Snapshot computation should succeed");

    assert_eq!(snapshot.mention_count, 3);
    assert_eq!(snapshot.positive_mentions, 1);
    assert_eq!(snapshot.negative_mentions, 1);
    assert_eq!(snapshot.neutral_mentions, 1);
}

#[tokio::test]
async fn test_trend_engine_basic() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let token = "TRENDING_TOKEN";
    let now = Utc::now().timestamp();

    let posts = vec![
        create_test_post("1", "Post 1", now - 1000),
        create_test_post("2", "Post 2", now - 500),
        create_test_post("3", "Post 3", now - 100),
    ];

    cache
        .store_posts(&posts, Some(token))
        .await
        .expect("Failed to store posts");

    let sentiment_engine = SentimentEngine::new();
    let items: Vec<(String, SocialPost)> = posts
        .iter()
        .map(|p| (token.to_string(), p.clone()))
        .collect();

    sentiment_engine
        .batch_analyze_posts(&items, cache.pool())
        .await
        .expect("Batch analysis should succeed");

    let trend_engine = TrendEngine::new(vec![60]);
    let trends = trend_engine
        .update_trends(cache.pool(), token)
        .await
        .expect("Trend update should succeed");

    assert!(!trends.is_empty());
    let trend = &trends[0];
    assert_eq!(trend.token, token);
    assert_eq!(trend.window_minutes, 60);
    assert!(trend.mentions > 0);
}

#[tokio::test]
async fn test_trend_engine_volume_spike() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let token = "SPIKE_TOKEN";
    let now = Utc::now().timestamp();

    let posts = vec![create_test_post("1", "Post 1", now - 1000)];

    cache
        .store_posts(&posts, Some(token))
        .await
        .expect("Failed to store posts");

    let sentiment_engine = SentimentEngine::new();
    let items: Vec<(String, SocialPost)> = posts
        .iter()
        .map(|p| (token.to_string(), p.clone()))
        .collect();

    sentiment_engine
        .batch_analyze_posts(&items, cache.pool())
        .await
        .expect("Batch analysis should succeed");

    let trend_engine = TrendEngine::new(vec![60]);
    let _first_update = trend_engine
        .update_trends(cache.pool(), token)
        .await
        .expect("First trend update should succeed");

    let spike_posts = vec![
        create_test_post("2", "Post 2", now),
        create_test_post("3", "Post 3", now),
        create_test_post("4", "Post 4", now),
        create_test_post("5", "Post 5", now),
    ];

    cache
        .store_posts(&spike_posts, Some(token))
        .await
        .expect("Failed to store spike posts");

    let items2: Vec<(String, SocialPost)> = spike_posts
        .iter()
        .map(|p| (token.to_string(), p.clone()))
        .collect();

    sentiment_engine
        .batch_analyze_posts(&items2, cache.pool())
        .await
        .expect("Batch analysis should succeed");

    let trends = trend_engine
        .update_trends(cache.pool(), token)
        .await
        .expect("Second trend update should succeed");

    let trend = &trends[0];
    assert!(trend.volume_spike > 1.0, "Volume spike should be detected");
    assert!(trend.mentions > 1);
}

#[tokio::test]
async fn test_influencer_scoring() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let token = "INFLUENCER_TOKEN";
    let now = Utc::now().timestamp();

    let posts = vec![
        SocialPost {
            id: "1".to_string(),
            text: "Great project!".to_string(),
            source: "twitter".to_string(),
            author: "big_influencer".to_string(),
            timestamp: now,
            sentiment: analyze_sentiment("Great project!"),
            engagement: 10000,
        },
        SocialPost {
            id: "2".to_string(),
            text: "Amazing!".to_string(),
            source: "twitter".to_string(),
            author: "big_influencer".to_string(),
            timestamp: now,
            sentiment: analyze_sentiment("Amazing!"),
            engagement: 8000,
        },
        SocialPost {
            id: "3".to_string(),
            text: "Nice!".to_string(),
            source: "twitter".to_string(),
            author: "small_user".to_string(),
            timestamp: now,
            sentiment: analyze_sentiment("Nice!"),
            engagement: 10,
        },
    ];

    cache
        .store_posts(&posts, Some(token))
        .await
        .expect("Failed to store posts");

    let sentiment_engine = SentimentEngine::new();
    let items: Vec<(String, SocialPost)> = posts
        .iter()
        .map(|p| (token.to_string(), p.clone()))
        .collect();

    sentiment_engine
        .batch_analyze_posts(&items, cache.pool())
        .await
        .expect("Batch analysis should succeed");

    let snapshot = sentiment_engine
        .compute_sentiment_snapshot(cache.pool(), token, None)
        .await
        .expect("Snapshot should succeed");

    let influencer_engine = InfluencerEngine::default();
    let scores = influencer_engine
        .compute_influencer_scores(cache.pool(), token, 86400, snapshot.avg_score)
        .await
        .expect("Influencer scoring should succeed");

    assert_eq!(scores.len(), 2, "Should have 2 influencers");

    let big_influencer = scores
        .iter()
        .find(|s| s.influencer == "big_influencer")
        .expect("big_influencer should be scored");

    let small_user = scores
        .iter()
        .find(|s| s.influencer == "small_user")
        .expect("small_user should be scored");

    assert!(
        big_influencer.impact_score > small_user.impact_score,
        "Big influencer should have higher impact score"
    );
    assert!(
        big_influencer.engagement_score > small_user.engagement_score,
        "Big influencer should have higher engagement score"
    );
}

#[tokio::test]
async fn test_fomo_fud_gauges() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let token = "GAUGE_TOKEN";
    let now = Utc::now().timestamp();

    let positive_posts = vec![
        create_test_post("1", "Amazing project! 🚀🚀🚀", now),
        create_test_post("2", "Going to the moon! Bullish!", now),
        create_test_post("3", "Best investment ever!", now),
    ];

    cache
        .store_posts(&positive_posts, Some(token))
        .await
        .expect("Failed to store posts");

    let sentiment_engine = SentimentEngine::new();
    let items: Vec<(String, SocialPost)> = positive_posts
        .iter()
        .map(|p| (token.to_string(), p.clone()))
        .collect();

    sentiment_engine
        .batch_analyze_posts(&items, cache.pool())
        .await
        .expect("Batch analysis should succeed");

    let snapshot = sentiment_engine
        .compute_sentiment_snapshot(cache.pool(), token, None)
        .await
        .expect("Snapshot should succeed");

    let trend_engine = TrendEngine::new(vec![60]);
    let trends = trend_engine
        .update_trends(cache.pool(), token)
        .await
        .expect("Trend update should succeed");

    let gauge_engine = GaugeEngine::new();
    let gauges = gauge_engine
        .update_gauges(cache.pool(), &[snapshot], &trends)
        .await
        .expect("Gauge update should succeed");

    assert_eq!(gauges.len(), 1);
    let gauge = &gauges[0];
    assert_eq!(gauge.token, token);
    assert!(gauge.fomo_score > 0.0, "FOMO score should be positive");
    assert!(
        gauge.fud_score < gauge.fomo_score,
        "FUD should be lower than FOMO for positive sentiment"
    );
    assert!(["low", "medium", "high"].contains(&gauge.fomo_level.as_str()));
    assert!(["low", "medium", "high"].contains(&gauge.fud_level.as_str()));
}

#[tokio::test]
async fn test_full_analysis_service() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let cache = SocialCache::new(temp_dir.path().to_path_buf())
        .await
        .expect("Failed to create cache");

    let token = "FULL_ANALYSIS_TOKEN";
    let now = Utc::now().timestamp();

    let posts = vec![
        create_test_post("1", "Great project! Bullish! 🚀", now - 1000),
        create_test_post("2", "Love this token", now - 500),
        create_test_post("3", "To the moon!", now - 100),
    ];

    cache
        .store_posts(&posts, Some(token))
        .await
        .expect("Failed to store posts");

    let mut service = SocialAnalysisService::new(cache);
    service
        .initialize()
        .await
        .expect("Service initialization should succeed");

    let summary = service
        .run_full_analysis(token)
        .await
        .expect("Full analysis should succeed");

    assert_eq!(summary.sentiments_analyzed, 3);
    assert!(summary.trends_updated > 0);
    assert!(summary.influencers_scored > 0);
    assert!(summary.gauges_computed > 0);

    let snapshot = service
        .get_sentiment_snapshot(token)
        .await
        .expect("Getting snapshot should succeed");

    assert!(snapshot.is_some(), "Snapshot should exist");
    let snapshot = snapshot.unwrap();
    assert_eq!(snapshot.token, token);
    assert!(
        snapshot.avg_score > 0.0,
        "Average sentiment should be positive"
    );

    let trends = service
        .get_token_trends(token)
        .await
        .expect("Getting trends should succeed");

    assert!(!trends.is_empty(), "Trends should exist");

    let gauges = service
        .get_fomo_fud_gauges(Some(token))
        .await
        .expect("Getting gauges should succeed");

    assert!(!gauges.is_empty(), "Gauges should exist");
}

fn create_test_post(id: &str, text: &str, timestamp: i64) -> SocialPost {
    SocialPost {
        id: id.to_string(),
        text: text.to_string(),
        source: "twitter".to_string(),
        author: format!("user_{}", id),
        timestamp,
        sentiment: analyze_sentiment(text),
        engagement: 100,
    }
}
