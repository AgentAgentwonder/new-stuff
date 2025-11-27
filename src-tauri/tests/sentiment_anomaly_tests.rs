use app_lib::anomalies::{AnomalyDetector, PriceData, TransactionData};
use app_lib::sentiment::{analyze_sentiment, SentimentManager, SocialPost};
use chrono::Utc;

#[test]
fn test_sentiment_scoring_range() {
    let positive = analyze_sentiment("This is amazing! Great bullish project going to the moon!");
    assert!(positive.score >= -1.0 && positive.score <= 1.0);
    assert!(positive.score > 0.0);

    let negative = analyze_sentiment("Terrible scam! Dump this crash immediately!");
    assert!(negative.score >= -1.0 && negative.score <= 1.0);
    assert!(negative.score < 0.0);

    let neutral = analyze_sentiment("The price is stable today.");
    assert!(neutral.score >= -1.0 && neutral.score <= 1.0);
}

#[test]
fn test_sentiment_classification() {
    let positive = analyze_sentiment("Excellent project with great fundamentals!");
    assert_eq!(positive.label, "positive");

    let negative = analyze_sentiment("This is a terrible scam, avoid it!");
    assert_eq!(negative.label, "negative");

    let neutral = analyze_sentiment("Price update for today.");
    assert_eq!(neutral.label, "neutral");
}

#[test]
fn test_sentiment_confidence() {
    let strong_positive = analyze_sentiment("Excellent! Amazing! Great! Bullish! Moon!");
    assert!(strong_positive.confidence > 0.5);

    let weak = analyze_sentiment("Maybe okay.");
    assert!(weak.confidence < 0.3);
}

#[test]
fn test_sentiment_manager_tracking() {
    let mut manager = SentimentManager::new();
    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    let posts = vec![
        SocialPost {
            id: "1".to_string(),
            text: "Great project!".to_string(),
            source: "twitter".to_string(),
            author: "user1".to_string(),
            timestamp: now,
            sentiment: analyze_sentiment("Great project!"),
            engagement: 100,
        },
        SocialPost {
            id: "2".to_string(),
            text: "Bullish on this!".to_string(),
            source: "reddit".to_string(),
            author: "user2".to_string(),
            timestamp: now,
            sentiment: analyze_sentiment("Bullish on this!"),
            engagement: 200,
        },
    ];

    manager.add_sentiment_data(token.clone(), posts);

    let sentiment = manager.get_token_sentiment(&token).unwrap();
    assert_eq!(sentiment.total_mentions, 2);
    assert!(sentiment.positive_count > 0);
    assert_eq!(sentiment.label, "positive");
}

#[test]
fn test_sentiment_trend_tracking() {
    let mut manager = SentimentManager::new();
    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    for i in 0..5 {
        let posts = vec![SocialPost {
            id: format!("{}", i),
            text: "Great project!".to_string(),
            source: "twitter".to_string(),
            author: format!("user{}", i),
            timestamp: now + i * 3600,
            sentiment: analyze_sentiment("Great project!"),
            engagement: 100,
        }];
        manager.add_sentiment_data(token.clone(), posts);
    }

    let sentiment = manager.get_token_sentiment(&token).unwrap();
    assert_eq!(sentiment.trend.len(), 5);
}

#[test]
fn test_sentiment_alert_thresholds() {
    let mut manager = SentimentManager::new();
    manager.alert_config.positive_threshold = 0.3;
    manager.alert_config.negative_threshold = -0.3;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    let positive_posts = vec![SocialPost {
        id: "1".to_string(),
        text: "Amazing! Excellent! Great! Bullish! Moon!".to_string(),
        source: "twitter".to_string(),
        author: "user1".to_string(),
        timestamp: now,
        sentiment: analyze_sentiment("Amazing! Excellent! Great! Bullish! Moon!"),
        engagement: 100,
    }];

    manager.add_sentiment_data(token.clone(), positive_posts);
    let alerts = manager.get_alerts(Some(&token));
    assert!(
        !alerts.is_empty(),
        "Should generate alert for high positive sentiment"
    );
}

#[test]
fn test_zscore_anomaly_threshold() {
    let mut detector = AnomalyDetector::new();
    detector.config.min_data_points = 10;
    detector.config.zscore_threshold = 3.0;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    for i in 0..10 {
        let data = PriceData {
            timestamp: now + i * 60,
            price: 100.0,
            volume: 1000.0,
        };
        detector.add_price_data(token.clone(), data);
    }

    let anomaly_data = PriceData {
        timestamp: now + 600,
        price: 400.0,
        volume: 1000.0,
    };
    detector.add_price_data(token.clone(), anomaly_data);

    let anomalies = detector.get_anomalies(Some(&token), Some("price_zscore"));
    assert!(!anomalies.is_empty(), "Should detect Z-score anomaly");
}

#[test]
fn test_iqr_anomaly_detection() {
    let mut detector = AnomalyDetector::new();
    detector.config.min_data_points = 20;
    detector.config.iqr_multiplier = 1.5;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    for i in 0..25 {
        let price = if i < 24 { 100.0 } else { 300.0 };
        let data = PriceData {
            timestamp: now + i * 60,
            price,
            volume: 1000.0,
        };
        detector.add_price_data(token.clone(), data);
    }

    let anomalies = detector.get_anomalies(Some(&token), Some("price_iqr"));
    assert!(!anomalies.is_empty(), "Should detect IQR outlier");
}

#[test]
fn test_volume_spike_detection() {
    let mut detector = AnomalyDetector::new();
    detector.config.min_data_points = 10;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    for i in 0..10 {
        let data = PriceData {
            timestamp: now + i * 60,
            price: 100.0,
            volume: 1000.0,
        };
        detector.add_price_data(token.clone(), data);
    }

    let spike_data = PriceData {
        timestamp: now + 600,
        price: 100.0,
        volume: 20000.0,
    };
    detector.add_price_data(token.clone(), spike_data);

    let anomalies = detector.get_anomalies(Some(&token), Some("volume_spike"));
    assert!(!anomalies.is_empty(), "Should detect volume spike");
}

#[test]
fn test_wash_trading_pattern() {
    let mut detector = AnomalyDetector::new();
    detector.config.min_data_points = 10;
    detector.config.wash_trading_threshold = 0.6;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();
    let addr1 = "address1".to_string();
    let addr2 = "address2".to_string();

    for i in 0..15 {
        let from = if i % 2 == 0 {
            addr1.clone()
        } else {
            addr2.clone()
        };
        let to = if i % 2 == 0 {
            addr2.clone()
        } else {
            addr1.clone()
        };

        let data = TransactionData {
            timestamp: now + i * 60,
            from,
            to,
            amount: 100.0,
            price: 1.0,
        };
        detector.add_transaction_data(token.clone(), data);
    }

    let anomalies = detector.get_anomalies(Some(&token), Some("wash_trading"));
    assert!(!anomalies.is_empty(), "Should detect wash trading pattern");
}

#[test]
fn test_anomaly_severity_classification() {
    let mut detector = AnomalyDetector::new();
    detector.config.min_data_points = 10;
    detector.config.zscore_threshold = 3.0;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    for i in 0..10 {
        let data = PriceData {
            timestamp: now + i * 60,
            price: 100.0,
            volume: 1000.0,
        };
        detector.add_price_data(token.clone(), data);
    }

    let critical_anomaly = PriceData {
        timestamp: now + 600,
        price: 600.0,
        volume: 1000.0,
    };
    detector.add_price_data(token.clone(), critical_anomaly);

    let anomalies = detector.get_anomalies(Some(&token), None);
    let has_critical = anomalies
        .iter()
        .any(|a| a.severity == "critical" || a.severity == "high");
    assert!(
        has_critical,
        "Should classify severe anomalies appropriately"
    );
}

#[test]
fn test_anomaly_dismiss() {
    let mut detector = AnomalyDetector::new();
    detector.config.min_data_points = 5;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    for i in 0..10 {
        let data = PriceData {
            timestamp: now + i * 60,
            price: if i == 5 { 300.0 } else { 100.0 },
            volume: 1000.0,
        };
        detector.add_price_data(token.clone(), data);
    }

    let active = detector.get_active_anomalies();
    assert!(!active.is_empty());

    if let Some(first) = active.first() {
        detector.dismiss_anomaly(&first.id);
        let updated_active = detector.get_active_anomalies();
        assert_eq!(updated_active.len(), active.len() - 1);
    }
}

#[test]
fn test_anomaly_statistics() {
    let mut detector = AnomalyDetector::new();
    detector.config.min_data_points = 5;

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    for i in 0..10 {
        let data = PriceData {
            timestamp: now + i * 60,
            price: if i % 3 == 0 { 200.0 } else { 100.0 },
            volume: if i == 8 { 50000.0 } else { 1000.0 },
        };
        detector.add_price_data(token.clone(), data);
    }

    let stats = detector.get_statistics(&token);
    assert!(stats.is_some());
    let stats = stats.unwrap();
    assert!(stats.total_anomalies > 0);
    assert!(stats.by_type.len() > 0);
}

#[test]
fn test_alert_routing() {
    let mut manager = SentimentManager::new();
    manager.alert_config.notification_channels = vec!["in-app".to_string(), "email".to_string()];

    let token = "test_token".to_string();
    let now = Utc::now().timestamp();

    let posts = vec![SocialPost {
        id: "1".to_string(),
        text: "Amazing! Excellent! Great! Bullish! Moon! Rocket!".to_string(),
        source: "twitter".to_string(),
        author: "user1".to_string(),
        timestamp: now,
        sentiment: analyze_sentiment("Amazing! Excellent! Great! Bullish! Moon! Rocket!"),
        engagement: 100,
    }];

    manager.add_sentiment_data(token.clone(), posts);
    let alerts = manager.get_alerts(Some(&token));

    assert!(!alerts.is_empty());
    assert_eq!(manager.alert_config.notification_channels.len(), 2);
}
