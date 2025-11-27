use super::drift_adapter::{generate_mock_drift_predictions, DriftAdapter, DriftPrediction};
use super::polymarket_adapter::{
    generate_mock_polymarket_markets, PolymarketAdapter, PolymarketMarket,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// Normalized prediction market structure
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PredictionMarket {
    pub id: String,
    pub source: String, // "polymarket" | "drift"
    pub title: String,
    pub description: String,
    pub category: String,
    pub outcomes: Vec<String>,
    pub outcome_prices: Vec<f64>, // Probabilities (0.0 to 1.0)
    pub volume_24h: f64,
    pub total_volume: f64,
    pub liquidity: f64,
    pub created_at: Option<i64>,
    pub end_date: Option<i64>,
    pub resolved: bool,
    pub winning_outcome: Option<usize>,
    pub tags: Vec<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomPrediction {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: String,
    pub outcomes: Vec<String>,
    pub user_prediction: Vec<f64>, // User's probability predictions
    pub confidence: f64,           // 0.0 to 1.0
    pub created_at: i64,
    pub updated_at: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PredictionPerformance {
    pub prediction_id: String,
    pub user_id: String,
    pub initial_prediction: Vec<f64>,
    pub actual_outcome: Option<usize>,
    pub accuracy_score: Option<f64>,
    pub brier_score: Option<f64>,
    pub log_score: Option<f64>,
    pub market_comparison: Option<f64>, // How user compared to market consensus
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConsensusData {
    pub market_id: String,
    pub outcomes: Vec<String>,
    pub polymarket_odds: Option<Vec<f64>>,
    pub drift_odds: Option<Vec<f64>>,
    pub aggregated_odds: Vec<f64>,
    pub volume_weighted_odds: Vec<f64>,
    pub variance: f64,
    pub agreement_score: f64, // 0.0 to 1.0, how much markets agree
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioComparison {
    pub user_id: String,
    pub total_predictions: usize,
    pub correct_predictions: usize,
    pub accuracy_rate: f64,
    pub avg_brier_score: f64,
    pub avg_confidence: f64,
    pub market_accuracy_rate: f64, // Community average
    pub percentile_rank: f64,
    pub best_category: Option<String>,
    pub worst_category: Option<String>,
    pub recent_performance: Vec<f64>, // Last N accuracy scores
}

pub struct PredictionMarketService {
    polymarket_adapter: PolymarketAdapter,
    drift_adapter: DriftAdapter,
    custom_predictions: Arc<RwLock<Vec<CustomPrediction>>>,
    performances: Arc<RwLock<Vec<PredictionPerformance>>>,
}

impl PredictionMarketService {
    pub fn new() -> Self {
        Self {
            polymarket_adapter: PolymarketAdapter::new(),
            drift_adapter: DriftAdapter::new(),
            custom_predictions: Arc::new(RwLock::new(Vec::new())),
            performances: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Normalize Polymarket markets to common format
    fn normalize_polymarket(market: &PolymarketMarket) -> PredictionMarket {
        let end_timestamp = market
            .end_date
            .as_ref()
            .and_then(|date| chrono::DateTime::parse_from_rfc3339(date).ok())
            .map(|dt| dt.timestamp());

        PredictionMarket {
            id: format!("polymarket_{}", market.condition_id),
            source: "polymarket".to_string(),
            title: market.question.clone(),
            description: market.description.clone().unwrap_or_default(),
            category: market
                .tags
                .first()
                .cloned()
                .unwrap_or_else(|| "General".to_string()),
            outcomes: market.outcomes.clone(),
            outcome_prices: market.outcome_prices.clone(),
            volume_24h: market.volume,
            total_volume: market.volume,
            liquidity: market.liquidity,
            created_at: None,
            end_date: end_timestamp,
            resolved: market.closed,
            winning_outcome: None,
            tags: market.tags.clone(),
            image_url: market.image.clone(),
        }
    }

    // Normalize Drift predictions to common format
    fn normalize_drift(prediction: &DriftPrediction) -> PredictionMarket {
        PredictionMarket {
            id: format!("drift_{}", prediction.id),
            source: "drift".to_string(),
            title: prediction.title.clone(),
            description: prediction.description.clone(),
            category: prediction.category.clone(),
            outcomes: prediction.outcomes.clone(),
            outcome_prices: prediction.outcome_prices.clone(),
            volume_24h: prediction.total_volume,
            total_volume: prediction.total_volume,
            liquidity: prediction.total_volume * 0.3, // Estimate
            created_at: None,
            end_date: prediction.resolution_time,
            resolved: prediction.resolved,
            winning_outcome: prediction.winning_outcome,
            tags: vec![prediction.category.clone()],
            image_url: None,
        }
    }

    pub async fn fetch_all_markets(&self, use_mock: bool) -> Result<Vec<PredictionMarket>, String> {
        let mut all_markets = Vec::new();

        // Fetch from Polymarket
        let polymarket_markets = if use_mock {
            generate_mock_polymarket_markets()
        } else {
            self.polymarket_adapter
                .fetch_markets()
                .await
                .unwrap_or_else(|_| generate_mock_polymarket_markets())
        };

        for market in polymarket_markets {
            all_markets.push(Self::normalize_polymarket(&market));
        }

        // Fetch from Drift
        let drift_predictions = if use_mock {
            generate_mock_drift_predictions()
        } else {
            self.drift_adapter
                .fetch_predictions()
                .await
                .unwrap_or_else(|_| generate_mock_drift_predictions())
        };

        for prediction in drift_predictions {
            all_markets.push(Self::normalize_drift(&prediction));
        }

        Ok(all_markets)
    }

    pub async fn search_markets(
        &self,
        query: &str,
        use_mock: bool,
    ) -> Result<Vec<PredictionMarket>, String> {
        let all_markets = self.fetch_all_markets(use_mock).await?;

        let query_lower = query.to_lowercase();
        let filtered: Vec<PredictionMarket> = all_markets
            .into_iter()
            .filter(|market| {
                market.title.to_lowercase().contains(&query_lower)
                    || market.description.to_lowercase().contains(&query_lower)
                    || market.category.to_lowercase().contains(&query_lower)
                    || market
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(filtered)
    }

    pub async fn create_custom_prediction(
        &self,
        prediction: CustomPrediction,
    ) -> Result<CustomPrediction, String> {
        let mut predictions = self.custom_predictions.write().await;
        predictions.push(prediction.clone());
        Ok(prediction)
    }

    pub async fn get_custom_predictions(
        &self,
        user_id: &str,
    ) -> Result<Vec<CustomPrediction>, String> {
        let predictions = self.custom_predictions.read().await;
        Ok(predictions
            .iter()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect())
    }

    pub async fn update_custom_prediction(
        &self,
        id: &str,
        updated: CustomPrediction,
    ) -> Result<CustomPrediction, String> {
        let mut predictions = self.custom_predictions.write().await;
        if let Some(pred) = predictions.iter_mut().find(|p| p.id == id) {
            *pred = updated.clone();
            Ok(updated)
        } else {
            Err("Prediction not found".to_string())
        }
    }

    pub async fn record_performance(
        &self,
        performance: PredictionPerformance,
    ) -> Result<(), String> {
        let mut performances = self.performances.write().await;
        performances.push(performance);
        Ok(())
    }

    pub async fn get_portfolio_comparison(
        &self,
        user_id: &str,
    ) -> Result<PortfolioComparison, String> {
        let performances = self.performances.read().await;
        let user_performances: Vec<&PredictionPerformance> = performances
            .iter()
            .filter(|p| p.user_id == user_id)
            .collect();

        if user_performances.is_empty() {
            return Ok(PortfolioComparison {
                user_id: user_id.to_string(),
                total_predictions: 0,
                correct_predictions: 0,
                accuracy_rate: 0.0,
                avg_brier_score: 0.0,
                avg_confidence: 0.0,
                market_accuracy_rate: 0.65, // Mock baseline
                percentile_rank: 0.5,
                best_category: None,
                worst_category: None,
                recent_performance: Vec::new(),
            });
        }

        let total = user_performances.len();
        let correct = user_performances
            .iter()
            .filter(|p| p.accuracy_score.unwrap_or(0.0) > 0.7)
            .count();

        let accuracy_rate = correct as f64 / total as f64;

        let avg_brier = user_performances
            .iter()
            .filter_map(|p| p.brier_score)
            .sum::<f64>()
            / total as f64;

        let recent: Vec<f64> = user_performances
            .iter()
            .rev()
            .take(10)
            .filter_map(|p| p.accuracy_score)
            .collect();

        Ok(PortfolioComparison {
            user_id: user_id.to_string(),
            total_predictions: total,
            correct_predictions: correct,
            accuracy_rate,
            avg_brier_score: avg_brier,
            avg_confidence: 0.75, // Mock
            market_accuracy_rate: 0.65,
            percentile_rank: if accuracy_rate > 0.65 { 0.75 } else { 0.45 },
            best_category: Some("Crypto Price".to_string()),
            worst_category: Some("Politics".to_string()),
            recent_performance: recent,
        })
    }

    pub async fn calculate_consensus(
        &self,
        market_id: &str,
        use_mock: bool,
    ) -> Result<ConsensusData, String> {
        // For now, return mock consensus data
        // In production, this would aggregate data from multiple sources

        let markets = self.fetch_all_markets(use_mock).await?;
        let market = markets
            .iter()
            .find(|m| m.id == market_id)
            .ok_or_else(|| "Market not found".to_string())?;

        let polymarket_odds = if market.source == "polymarket" {
            Some(market.outcome_prices.clone())
        } else {
            None
        };

        let drift_odds = if market.source == "drift" {
            Some(market.outcome_prices.clone())
        } else {
            None
        };

        let aggregated_odds = market.outcome_prices.clone();

        // Calculate variance between sources
        let variance = 0.05; // Mock
        let agreement_score = 0.9; // Mock

        Ok(ConsensusData {
            market_id: market_id.to_string(),
            outcomes: market.outcomes.clone(),
            polymarket_odds,
            drift_odds,
            aggregated_odds,
            volume_weighted_odds: market.outcome_prices.clone(),
            variance,
            agreement_score,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
}

pub type SharedPredictionMarketService = Arc<RwLock<PredictionMarketService>>;

// Tauri commands

#[tauri::command]
pub async fn get_prediction_markets(
    use_mock: bool,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<Vec<PredictionMarket>, String> {
    let svc = service.read().await;
    svc.fetch_all_markets(use_mock).await
}

#[tauri::command]
pub async fn search_prediction_markets(
    query: String,
    use_mock: bool,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<Vec<PredictionMarket>, String> {
    let svc = service.read().await;
    svc.search_markets(&query, use_mock).await
}

#[tauri::command]
pub async fn create_custom_prediction(
    prediction: CustomPrediction,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<CustomPrediction, String> {
    let svc = service.read().await;
    svc.create_custom_prediction(prediction).await
}

#[tauri::command]
pub async fn get_custom_predictions(
    user_id: String,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<Vec<CustomPrediction>, String> {
    let svc = service.read().await;
    svc.get_custom_predictions(&user_id).await
}

#[tauri::command]
pub async fn update_custom_prediction(
    id: String,
    prediction: CustomPrediction,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<CustomPrediction, String> {
    let svc = service.read().await;
    svc.update_custom_prediction(&id, prediction).await
}

#[tauri::command]
pub async fn get_portfolio_comparison(
    user_id: String,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<PortfolioComparison, String> {
    let svc = service.read().await;
    svc.get_portfolio_comparison(&user_id).await
}

#[tauri::command]
pub async fn get_consensus_data(
    market_id: String,
    use_mock: bool,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<ConsensusData, String> {
    let svc = service.read().await;
    svc.calculate_consensus(&market_id, use_mock).await
}

#[tauri::command]
pub async fn record_prediction_performance(
    performance: PredictionPerformance,
    service: tauri::State<'_, SharedPredictionMarketService>,
) -> Result<(), String> {
    let svc = service.read().await;
    svc.record_performance(performance).await
}
