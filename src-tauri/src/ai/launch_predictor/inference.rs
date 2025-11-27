use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::ai::launch_predictor::{LaunchPredictor, SharedLaunchPredictor, TokenFeatures};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictLaunchRequest {
    pub token_address: String,
    pub features: TokenFeatures,
}

#[tauri::command]
pub async fn predict_launch_success(
    request: PredictLaunchRequest,
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<super::LaunchPrediction, String> {
    let pred = predictor.read().await;
    pred.predict(&request.token_address, request.features)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_launch_prediction_history(
    token_address: String,
    days: Option<u32>,
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<super::PredictionHistory, String> {
    let pred = predictor.read().await;
    pred.get_prediction_history(&token_address, days.unwrap_or(30))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_launch_training_data(
    token_address: String,
    features: TokenFeatures,
    actual_outcome: f64,
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<(), String> {
    let pred = predictor.read().await;
    pred.add_training_data(&token_address, features, actual_outcome)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn retrain_launch_model(
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<super::ModelMetrics, String> {
    let pred = predictor.read().await;
    pred.retrain_model().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_latest_launch_model(
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<(), String> {
    let pred = predictor.read().await;
    pred.load_latest_model().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_launch_bias_report(
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<super::LaunchBiasReport, String> {
    let pred = predictor.read().await;
    pred.generate_bias_report().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_token_features(token_address: String) -> Result<TokenFeatures, String> {
    Ok(TokenFeatures {
        token_address,
        developer_reputation: 0.65,
        developer_launch_count: 3,
        developer_success_rate: 0.70,
        developer_category: "experienced".to_string(),
        contract_complexity: 0.45,
        proxy_pattern_detected: false,
        upgradeable_contract: false,
        liquidity_usd: 120_000.0,
        liquidity_ratio: 0.15,
        liquidity_change_24h: 5.2,
        initial_market_cap: 500_000.0,
        marketing_hype: 0.60,
        marketing_spend_usd: 8_000.0,
        social_followers_growth: 0.40,
        community_engagement: 0.68,
        influencer_sentiment: 0.55,
        security_audit_score: Some(0.80),
        dex_depth_score: 0.72,
        watchlist_interest: 0.58,
        retention_score: 0.75,
        launch_timestamp: Utc::now() - chrono::Duration::days(2),
        actual_outcome: None,
    })
}
