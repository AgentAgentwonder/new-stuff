#[cfg(test)]
mod tests {
    use app_lib::ai::launch_predictor::{LaunchModel, LaunchPredictor, TokenFeatures};
    use chrono::Utc;
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

    async fn create_test_predictor() -> LaunchPredictor {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:?cache=shared")
            .await
            .unwrap();
        LaunchPredictor::with_pool(pool).await.unwrap()
    }

    fn create_mock_features(token_address: &str) -> TokenFeatures {
        TokenFeatures {
            token_address: token_address.to_string(),
            developer_reputation: 0.75,
            developer_launch_count: 5,
            developer_success_rate: 0.80,
            developer_category: "experienced".to_string(),
            contract_complexity: 0.45,
            proxy_pattern_detected: false,
            upgradeable_contract: false,
            liquidity_usd: 150_000.0,
            liquidity_ratio: 0.18,
            liquidity_change_24h: 8.5,
            initial_market_cap: 750_000.0,
            marketing_hype: 0.55,
            marketing_spend_usd: 12_000.0,
            social_followers_growth: 0.45,
            community_engagement: 0.72,
            influencer_sentiment: 0.60,
            security_audit_score: Some(0.85),
            dex_depth_score: 0.78,
            watchlist_interest: 0.65,
            retention_score: 0.70,
            launch_timestamp: Utc::now() - chrono::Duration::days(3),
            actual_outcome: None,
        }
    }

    fn create_risky_features(token_address: &str) -> TokenFeatures {
        TokenFeatures {
            token_address: token_address.to_string(),
            developer_reputation: 0.25,
            developer_launch_count: 1,
            developer_success_rate: 0.30,
            developer_category: "unproven".to_string(),
            contract_complexity: 0.85,
            proxy_pattern_detected: true,
            upgradeable_contract: true,
            liquidity_usd: 15_000.0,
            liquidity_ratio: 0.03,
            liquidity_change_24h: -12.5,
            initial_market_cap: 200_000.0,
            marketing_hype: 0.90,
            marketing_spend_usd: 1_500.0,
            social_followers_growth: 0.95,
            community_engagement: 0.25,
            influencer_sentiment: 0.20,
            security_audit_score: Some(0.30),
            dex_depth_score: 0.20,
            watchlist_interest: 0.15,
            retention_score: 0.25,
            launch_timestamp: Utc::now() - chrono::Duration::hours(6),
            actual_outcome: None,
        }
    }

    #[tokio::test]
    async fn test_model_prediction() {
        let model = LaunchModel::new();
        let features = create_mock_features("test_token_1");

        let prediction = model.predict(&features);

        assert_eq!(prediction.token_address, "test_token_1");
        assert!(prediction.success_probability >= 0.0 && prediction.success_probability <= 100.0);
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
        assert!(!prediction.feature_scores.is_empty());
        assert!(["Low", "Medium", "High", "Critical"].contains(&prediction.risk_level.as_str()));
    }

    #[tokio::test]
    async fn test_high_quality_token_prediction() {
        let model = LaunchModel::new();
        let features = create_mock_features("test_token_high_quality");

        let prediction = model.predict(&features);

        assert!(
            prediction.success_probability > 50.0,
            "High quality token should have >50% success probability, got: {}",
            prediction.success_probability
        );
        assert!(prediction.confidence > 0.5);
        assert!(prediction.early_warnings.len() < 2);
    }

    #[tokio::test]
    async fn test_risky_token_prediction() {
        let model = LaunchModel::new();
        let features = create_risky_features("test_token_risky");

        let prediction = model.predict(&features);

        assert!(
            prediction.success_probability < 50.0,
            "Risky token should have <50% success probability, got: {}",
            prediction.success_probability
        );
        assert!(!prediction.early_warnings.is_empty());
    }

    #[tokio::test]
    async fn test_early_warnings_generation() {
        let features = create_risky_features("test_token_warnings");
        let warnings = features.early_warnings();

        assert!(
            !warnings.is_empty(),
            "Risky token should generate early warnings"
        );

        let warning_types: Vec<String> = warnings.iter().map(|w| w.warning_type.clone()).collect();

        assert!(
            warning_types.contains(&"lowDeveloperReputation".to_string())
                || warning_types.contains(&"shallowLiquidity".to_string()),
            "Should detect low developer reputation or shallow liquidity"
        );
    }

    #[tokio::test]
    async fn test_predictor_persistence() {
        let predictor = create_test_predictor().await;
        let features = create_mock_features("test_token_persist");

        let prediction = predictor
            .predict("test_token_persist", features.clone())
            .await
            .unwrap();

        assert_eq!(prediction.token_address, "test_token_persist");

        let history = predictor
            .get_prediction_history("test_token_persist", 30)
            .await
            .unwrap();
        assert_eq!(history.predictions.len(), 1);
        assert_eq!(history.predictions[0].token_address, "test_token_persist");
    }

    #[tokio::test]
    async fn test_training_data_storage() {
        let predictor = create_test_predictor().await;
        let features = create_mock_features("test_token_training");

        predictor
            .add_training_data("test_token_training", features.clone(), 0.85)
            .await
            .unwrap();

        predictor
            .add_training_data(
                "test_token_training_2",
                create_risky_features("test_token_training_2"),
                0.15,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_model_retraining() {
        let predictor = create_test_predictor().await;

        for i in 0..10 {
            let features = if i % 2 == 0 {
                create_mock_features(&format!("train_token_{}", i))
            } else {
                create_risky_features(&format!("train_token_{}", i))
            };

            let outcome = if i % 2 == 0 { 0.85 } else { 0.20 };
            predictor
                .add_training_data(&format!("train_token_{}", i), features, outcome)
                .await
                .unwrap();
        }

        let metrics = predictor.retrain_model().await.unwrap();

        assert!(metrics.training_samples == 10);
        assert!(metrics.accuracy >= 0.0 && metrics.accuracy <= 1.0);
    }

    #[tokio::test]
    async fn test_feature_importance_ordering() {
        let model = LaunchModel::new();
        let features = create_mock_features("test_feature_importance");

        let prediction = model.predict(&features);

        assert!(!prediction.feature_scores.is_empty());

        let mut sorted_scores = prediction.feature_scores.clone();
        sorted_scores.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());

        assert!(sorted_scores[0].importance >= sorted_scores.last().unwrap().importance);
    }

    #[tokio::test]
    async fn test_bias_detection_low_liquidity() {
        let mut low_liquidity_features = create_mock_features("test_bias_liquidity");
        low_liquidity_features.liquidity_usd = 5_000.0;

        let warnings = low_liquidity_features.early_warnings();
        let has_liquidity_warning = warnings
            .iter()
            .any(|w| w.warning_type == "shallowLiquidity");

        assert!(
            has_liquidity_warning,
            "Should detect shallow liquidity as a bias risk"
        );
    }

    #[tokio::test]
    async fn test_bias_detection_marketing_mismatch() {
        let mut marketing_mismatch = create_mock_features("test_bias_marketing");
        marketing_mismatch.marketing_hype = 0.85;
        marketing_mismatch.community_engagement = 0.30;

        let warnings = marketing_mismatch.early_warnings();
        let has_marketing_warning = warnings
            .iter()
            .any(|w| w.warning_type == "marketingMismatch");

        assert!(
            has_marketing_warning,
            "Should detect marketing/engagement mismatch"
        );
    }

    #[tokio::test]
    async fn test_confidence_score_calculation() {
        let high_confidence_features = create_mock_features("test_confidence_high");
        let model = LaunchModel::new();
        let prediction = model.predict(&high_confidence_features);

        assert!(
            prediction.confidence > 0.6,
            "High-quality features should yield high confidence, got: {}",
            prediction.confidence
        );

        let low_confidence_features = create_risky_features("test_confidence_low");
        let low_prediction = model.predict(&low_confidence_features);

        assert!(
            low_prediction.confidence < prediction.confidence,
            "Risky features should yield lower confidence"
        );
    }

    #[tokio::test]
    async fn test_feature_vector_normalization() {
        let features = create_mock_features("test_normalization");
        let feature_vector = features.feature_vector();

        for (_key, value) in feature_vector.iter() {
            assert!(value.is_finite(), "All feature values should be finite");
            assert!(!value.is_nan(), "No feature value should be NaN");
        }
    }

    #[tokio::test]
    async fn test_model_serialization() {
        let model = LaunchModel::new();

        let json = model.to_json().unwrap();
        let deserialized = LaunchModel::from_json(&json).unwrap();

        assert_eq!(model.weights.len(), deserialized.weights.len());
        assert_eq!(model.intercept, deserialized.intercept);
    }

    #[tokio::test]
    async fn test_bias_report_generation() {
        let predictor = create_test_predictor().await;

        let mut experienced = create_mock_features("bias_exp_1");
        experienced.developer_category = "experienced".to_string();
        predictor
            .add_training_data("bias_exp_1", experienced, 0.9)
            .await
            .unwrap();
        let mut experienced_two = create_mock_features("bias_exp_2");
        experienced_two.developer_category = "experienced".to_string();
        predictor
            .add_training_data("bias_exp_2", experienced_two, 0.8)
            .await
            .unwrap();

        let mut studio = create_mock_features("bias_studio_1");
        studio.developer_category = "studio".to_string();
        predictor
            .add_training_data("bias_studio_1", studio, 0.6)
            .await
            .unwrap();
        let mut studio_two = create_mock_features("bias_studio_2");
        studio_two.developer_category = "studio".to_string();
        predictor
            .add_training_data("bias_studio_2", studio_two, 0.4)
            .await
            .unwrap();

        let mut unproven = create_risky_features("bias_unproven_1");
        unproven.developer_category = "unproven".to_string();
        predictor
            .add_training_data("bias_unproven_1", unproven, 0.2)
            .await
            .unwrap();
        let mut unproven_two = create_risky_features("bias_unproven_2");
        unproven_two.developer_category = "unproven".to_string();
        predictor
            .add_training_data("bias_unproven_2", unproven_two, 0.1)
            .await
            .unwrap();

        let report = predictor.generate_bias_report().await.unwrap();
        assert!(report.metrics.len() >= 3);
        assert!(report.global_success_rate > 0.0);
    }
}
