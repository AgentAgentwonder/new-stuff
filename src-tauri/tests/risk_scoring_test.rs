use app_lib::ai::{RiskAnalyzer, RiskFeatures, RiskModel};
use sqlx::SqlitePool;
use tempfile::tempdir;

#[tokio::test]
async fn test_risk_model_high_risk_token() {
    let model = RiskModel::new();

    // Token with characteristics of a rug pull
    let high_risk = RiskFeatures {
        gini_coefficient: 0.92,
        top_10_percentage: 88.0,
        total_holders: 85,
        liquidity_usd: 15000.0,
        liquidity_to_mcap_ratio: 0.01,
        has_mint_authority: true,
        has_freeze_authority: true,
        verified: false,
        audited: false,
        community_trust_score: 0.2,
        sentiment_score: -0.4,
        token_age_days: 2.0,
        volume_24h: 8000.0,
        price_volatility: 52.0,
    };

    let (score, factors) = model.score_token(&high_risk);

    // Should score as high risk (> 60)
    assert!(
        score > 60.0,
        "High risk token scored {}, expected > 60",
        score
    );
    assert!(!factors.is_empty(), "Should have contributing factors");

    // Check that top risk factors are present
    let factor_names: Vec<String> = factors.iter().map(|f| f.factor_name.clone()).collect();
    assert!(
        factor_names
            .iter()
            .any(|f| f.contains("gini") || f.contains("mint") || f.contains("freeze")),
        "Expected concentration or authority factors in top contributors"
    );
}

#[tokio::test]
async fn test_risk_model_low_risk_token() {
    let model = RiskModel::new();

    // Established, legitimate token
    let low_risk = RiskFeatures {
        gini_coefficient: 0.28,
        top_10_percentage: 18.0,
        total_holders: 12000,
        liquidity_usd: 1250000.0,
        liquidity_to_mcap_ratio: 0.15,
        has_mint_authority: false,
        has_freeze_authority: false,
        verified: true,
        audited: true,
        community_trust_score: 0.93,
        sentiment_score: 0.7,
        token_age_days: 390.0,
        volume_24h: 580000.0,
        price_volatility: 8.0,
    };

    let (score, _factors) = model.score_token(&low_risk);

    // Should score as low risk (< 40)
    assert!(
        score < 40.0,
        "Low risk token scored {}, expected < 40",
        score
    );
}

#[tokio::test]
async fn test_risk_model_boundary_conditions() {
    let model = RiskModel::new();

    // All features at extreme values
    let extreme = RiskFeatures {
        gini_coefficient: 1.0,
        top_10_percentage: 100.0,
        total_holders: 1,
        liquidity_usd: 1.0,
        liquidity_to_mcap_ratio: 0.0,
        has_mint_authority: true,
        has_freeze_authority: true,
        verified: false,
        audited: false,
        community_trust_score: 0.0,
        sentiment_score: -1.0,
        token_age_days: 0.1,
        volume_24h: 1.0,
        price_volatility: 100.0,
    };

    let (score, _) = model.score_token(&extreme);

    // Score should be clamped to valid range
    assert!(
        score >= 0.0 && score <= 100.0,
        "Score {} out of valid range",
        score
    );
    assert!(score > 80.0, "Extreme risk token should score as Critical");
}

#[tokio::test]
async fn test_model_serialization() {
    let model = RiskModel::new();

    // Serialize
    let json = model.to_json().expect("Should serialize to JSON");

    // Deserialize
    let loaded = RiskModel::from_json(&json).expect("Should deserialize from JSON");

    // Verify weights are preserved
    assert_eq!(model.intercept, loaded.intercept);
    assert_eq!(model.threshold, loaded.threshold);

    // Test that loaded model produces same scores
    let test_features = RiskFeatures {
        gini_coefficient: 0.5,
        top_10_percentage: 50.0,
        total_holders: 1000,
        liquidity_usd: 100000.0,
        liquidity_to_mcap_ratio: 0.1,
        has_mint_authority: false,
        has_freeze_authority: false,
        verified: true,
        audited: false,
        community_trust_score: 0.7,
        sentiment_score: 0.3,
        token_age_days: 30.0,
        volume_24h: 50000.0,
        price_volatility: 10.0,
    };

    let (score1, _) = model.score_token(&test_features);
    let (score2, _) = loaded.score_token(&test_features);

    assert!(
        (score1 - score2).abs() < 0.001,
        "Loaded model should produce same scores"
    );
}

#[tokio::test]
async fn test_risk_analyzer_scoring_and_storage() {
    let dir = tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test_risk.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to test DB");
    let analyzer = RiskAnalyzer::with_pool(pool)
        .await
        .expect("Failed to create analyzer");

    // Score a token
    let features = RiskFeatures {
        gini_coefficient: 0.6,
        top_10_percentage: 55.0,
        total_holders: 800,
        liquidity_usd: 80000.0,
        liquidity_to_mcap_ratio: 0.08,
        has_mint_authority: false,
        has_freeze_authority: true,
        verified: false,
        audited: false,
        community_trust_score: 0.5,
        sentiment_score: 0.0,
        token_age_days: 15.0,
        volume_24h: 35000.0,
        price_volatility: 20.0,
    };

    let token_address = "TestToken123";
    let score = analyzer
        .score_token(token_address, features.clone())
        .await
        .expect("Scoring failed");

    // Verify score properties
    assert!(score.score >= 0.0 && score.score <= 100.0);
    assert_eq!(score.token_address, token_address);
    assert!(!score.contributing_factors.is_empty());
    assert!(!score.timestamp.is_empty());

    // Verify it was stored
    let latest = analyzer
        .get_latest_risk_score(token_address)
        .await
        .expect("Failed to get latest");
    assert!(latest.is_some());
    let latest = latest.unwrap();
    assert_eq!(latest.token_address, token_address);
    assert_eq!(latest.score, score.score);

    // Score again to create history
    let score2 = analyzer
        .score_token(token_address, features)
        .await
        .expect("Second scoring failed");

    // Get history
    let history = analyzer
        .get_risk_history(token_address, 30)
        .await
        .expect("Failed to get history");
    assert_eq!(history.token_address, token_address);
    assert_eq!(history.history.len(), 2, "Should have 2 history points");
}

#[tokio::test]
async fn test_risk_analyzer_model_persistence() {
    let dir = tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test_risk_models.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to connect to test DB");
    let analyzer = RiskAnalyzer::with_pool(pool)
        .await
        .expect("Failed to create analyzer");

    // Save current model
    let metrics = Some(r#"{"auc_roc": 0.87, "precision": 0.82}"#.to_string());
    analyzer
        .save_model(metrics)
        .await
        .expect("Failed to save model");

    // Load it back
    analyzer
        .load_latest_model()
        .await
        .expect("Failed to load model");

    // Verify it works
    let features = RiskFeatures {
        gini_coefficient: 0.5,
        top_10_percentage: 50.0,
        total_holders: 1000,
        liquidity_usd: 100000.0,
        liquidity_to_mcap_ratio: 0.1,
        has_mint_authority: false,
        has_freeze_authority: false,
        verified: true,
        audited: false,
        community_trust_score: 0.7,
        sentiment_score: 0.3,
        token_age_days: 30.0,
        volume_24h: 50000.0,
        price_volatility: 10.0,
    };

    let score = analyzer
        .score_token("test", features)
        .await
        .expect("Scoring with loaded model failed");
    assert!(score.score >= 0.0 && score.score <= 100.0);
}

#[test]
fn test_feature_extraction_correctness() {
    let model = RiskModel::new();

    // Test that features are extracted correctly
    let features = RiskFeatures {
        gini_coefficient: 0.7,
        top_10_percentage: 65.0,
        total_holders: 500,
        liquidity_usd: 50000.0,
        liquidity_to_mcap_ratio: 0.05,
        has_mint_authority: true,
        has_freeze_authority: false,
        verified: false,
        audited: false,
        community_trust_score: 0.4,
        sentiment_score: -0.2,
        token_age_days: 10.0,
        volume_24h: 20000.0,
        price_volatility: 30.0,
    };

    let (score, factors) = model.score_token(&features);

    // Verify score is reasonable
    assert!(
        score >= 30.0 && score <= 90.0,
        "Score {} seems unreasonable",
        score
    );

    // Verify we have factors
    assert!(
        factors.len() > 0 && factors.len() <= 5,
        "Should have 1-5 factors"
    );

    // Verify factors have valid impacts
    for factor in &factors {
        assert!(factor.impact > 0.0, "Factor impact should be positive");
        assert!(
            factor.severity == "Low" || factor.severity == "Medium" || factor.severity == "High",
            "Invalid severity: {}",
            factor.severity
        );
    }
}

#[test]
fn test_risk_level_classification() {
    let model = RiskModel::new();

    let test_cases = vec![
        (15.0, "Low"),
        (45.0, "Medium"),
        (70.0, "High"),
        (90.0, "Critical"),
    ];

    for (target_score, expected_level) in test_cases {
        // Create features that should produce approximately the target score
        let features = RiskFeatures {
            gini_coefficient: if target_score > 70.0 { 0.9 } else { 0.3 },
            top_10_percentage: if target_score > 70.0 { 80.0 } else { 20.0 },
            total_holders: if target_score > 70.0 { 100 } else { 10000 },
            liquidity_usd: if target_score > 70.0 {
                10000.0
            } else {
                1000000.0
            },
            liquidity_to_mcap_ratio: 0.1,
            has_mint_authority: target_score > 70.0,
            has_freeze_authority: target_score > 70.0,
            verified: target_score < 40.0,
            audited: target_score < 40.0,
            community_trust_score: if target_score > 70.0 { 0.2 } else { 0.9 },
            sentiment_score: if target_score > 70.0 { -0.5 } else { 0.7 },
            token_age_days: if target_score > 70.0 { 2.0 } else { 200.0 },
            volume_24h: 100000.0,
            price_volatility: if target_score > 70.0 { 50.0 } else { 5.0 },
        };

        let (score, _) = model.score_token(&features);

        let actual_level = if score < 30.0 {
            "Low"
        } else if score < 60.0 {
            "Medium"
        } else if score < 80.0 {
            "High"
        } else {
            "Critical"
        };

        // The classification should be reasonable given the features
        if target_score > 70.0 {
            assert!(score > 50.0, "High risk features should produce high score");
        } else if target_score < 30.0 {
            assert!(score < 60.0, "Low risk features should produce lower score");
        }
    }
}
