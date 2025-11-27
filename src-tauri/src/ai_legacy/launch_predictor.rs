use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;
use uuid::Uuid;

// ==================== Data Structures ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchFeatures {
    pub token_address: String,
    pub liquidity_usd: f64,
    pub holder_count: u64,
    pub creator_history: u32,
    pub social_score: f64,
    pub code_verified: bool,
    pub liquidity_locked: bool,
    pub lock_duration_days: u32,
    pub token_supply: f64,
    pub initial_price_usd: f64,
    pub market_cap_usd: f64,
    pub mint_disabled: bool,
    pub freeze_disabled: bool,
    pub ownership_renounced: bool,
    pub top_10_holders_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPrediction {
    pub token_address: String,
    pub success_score: f64,
    pub risk_level: String,
    pub contributing_factors: Vec<PredictionFactor>,
    pub confidence: f64,
    pub prediction_time: String,
    pub model_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionFactor {
    pub factor_name: String,
    pub impact: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchTrainingData {
    pub id: String,
    pub token_address: String,
    pub features: LaunchFeatures,
    pub actual_outcome: bool,
    pub outcome_time: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiasReport {
    pub total_predictions: u32,
    pub correct_predictions: u32,
    pub accuracy: f64,
    pub false_positives: u32,
    pub false_negatives: u32,
    pub by_liquidity_range: HashMap<String, BiasMetrics>,
    pub by_holder_count: HashMap<String, BiasMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiasMetrics {
    pub predictions: u32,
    pub accuracy: f64,
    pub avg_confidence: f64,
}

// ==================== LaunchPredictor Implementation ====================

pub struct LaunchPredictor {
    pool: Pool<Sqlite>,
    weights: Arc<RwLock<HashMap<String, f64>>>,
    intercept: Arc<RwLock<f64>>,
    model_version: Arc<RwLock<u32>>,
}

pub type SharedLaunchPredictor = Arc<RwLock<LaunchPredictor>>;

impl LaunchPredictor {
    pub async fn new(app: &AppHandle) -> Result<Self, sqlx::Error> {
        let app_handle = app.clone();
        let mut db_path = app_handle.path().app_data_dir().map_err(|_| {
            sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "App data dir not found",
            ))
        })?;

        std::fs::create_dir_all(&db_path).map_err(sqlx::Error::Io)?;
        db_path.push("launch_predictor.db");

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let predictor = Self {
            pool,
            weights: Arc::new(RwLock::new(Self::default_weights())),
            intercept: Arc::new(RwLock::new(50.0)),
            model_version: Arc::new(RwLock::new(1)),
        };

        predictor.initialize().await?;
        Ok(predictor)
    }

    fn default_weights() -> HashMap<String, f64> {
        let mut weights = HashMap::new();

        // Positive signals (increase success score)
        weights.insert("liquidity_locked".to_string(), 25.0);
        weights.insert("ownership_renounced".to_string(), 20.0);
        weights.insert("mint_disabled".to_string(), 15.0);
        weights.insert("freeze_disabled".to_string(), 15.0);
        weights.insert("code_verified".to_string(), 10.0);
        weights.insert("social_score".to_string(), 8.0);
        weights.insert("creator_history".to_string(), 5.0);
        weights.insert("liquidity_amount".to_string(), 12.0);

        // Negative signals (decrease success score)
        weights.insert("holder_concentration".to_string(), -25.0);
        weights.insert("low_holders".to_string(), -20.0);
        weights.insert("low_liquidity".to_string(), -30.0);

        weights
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        // Create predictions history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS launch_predictions (
                id TEXT PRIMARY KEY,
                token_address TEXT NOT NULL,
                success_score REAL NOT NULL,
                risk_level TEXT NOT NULL,
                factors TEXT NOT NULL,
                confidence REAL NOT NULL,
                prediction_time TEXT NOT NULL,
                model_version INTEGER NOT NULL,
                actual_outcome INTEGER,
                outcome_verified_at TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_predictions_token
            ON launch_predictions(token_address, prediction_time DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create training data table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS launch_training_data (
                id TEXT PRIMARY KEY,
                token_address TEXT NOT NULL,
                features TEXT NOT NULL,
                actual_outcome INTEGER NOT NULL,
                outcome_time TEXT NOT NULL,
                added_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_training_token
            ON launch_training_data(token_address)
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create model versions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS launch_models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL,
                weights TEXT NOT NULL,
                intercept REAL NOT NULL,
                trained_on_samples INTEGER NOT NULL,
                accuracy REAL,
                created_at TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_models_active
            ON launch_models(is_active, created_at DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn predict(&self, features: &LaunchFeatures) -> LaunchPrediction {
        let weights = self.weights.read().await;
        let intercept = *self.intercept.read().await;
        let model_version = *self.model_version.read().await;

        // Normalize features
        let liquidity_score = (features.liquidity_usd / 100000.0).min(1.0);
        let holder_score = (features.holder_count as f64 / 1000.0).min(1.0);
        let concentration = features.top_10_holders_percent / 100.0;

        // Calculate weighted score
        let mut score = intercept;
        let mut factor_contributions: Vec<(String, f64, String)> = Vec::new();

        // Positive factors
        if features.liquidity_locked {
            let contrib = weights.get("liquidity_locked").unwrap_or(&0.0);
            score += contrib;
            factor_contributions.push((
                "Liquidity Locked".to_string(),
                *contrib,
                "LP tokens are locked, reducing rug pull risk".to_string(),
            ));
        }

        if features.ownership_renounced {
            let contrib = weights.get("ownership_renounced").unwrap_or(&0.0);
            score += contrib;
            factor_contributions.push((
                "Ownership Renounced".to_string(),
                *contrib,
                "Contract ownership renounced, can't be changed".to_string(),
            ));
        }

        if features.mint_disabled {
            let contrib = weights.get("mint_disabled").unwrap_or(&0.0);
            score += contrib;
            factor_contributions.push((
                "Mint Disabled".to_string(),
                *contrib,
                "Cannot mint new tokens, supply is fixed".to_string(),
            ));
        }

        if features.freeze_disabled {
            let contrib = weights.get("freeze_disabled").unwrap_or(&0.0);
            score += contrib;
            factor_contributions.push((
                "Freeze Disabled".to_string(),
                *contrib,
                "Cannot freeze user accounts".to_string(),
            ));
        }

        if features.code_verified {
            let contrib = weights.get("code_verified").unwrap_or(&0.0);
            score += contrib;
            factor_contributions.push((
                "Code Verified".to_string(),
                *contrib,
                "Contract code is verified on-chain".to_string(),
            ));
        }

        // Social and creator factors
        let social_contrib = weights.get("social_score").unwrap_or(&0.0) * (features.social_score / 100.0);
        score += social_contrib;
        if social_contrib.abs() > 1.0 {
            factor_contributions.push((
                "Social Score".to_string(),
                social_contrib,
                format!("Social presence score: {:.1}/100", features.social_score),
            ));
        }

        let creator_contrib = weights.get("creator_history").unwrap_or(&0.0) * (features.creator_history as f64 / 10.0).min(1.0);
        score += creator_contrib;
        if creator_contrib.abs() > 1.0 {
            factor_contributions.push((
                "Creator History".to_string(),
                creator_contrib,
                format!("Creator launched {} previous tokens", features.creator_history),
            ));
        }

        // Liquidity factors
        let liq_contrib = weights.get("liquidity_amount").unwrap_or(&0.0) * liquidity_score;
        score += liq_contrib;
        if features.liquidity_usd < 10000.0 {
            let low_liq_contrib = *weights.get("low_liquidity").unwrap_or(&0.0);
            score += low_liq_contrib;
            factor_contributions.push((
                "Low Liquidity".to_string(),
                low_liq_contrib,
                format!("Only ${:.0} liquidity - high manipulation risk", features.liquidity_usd),
            ));
        } else {
            factor_contributions.push((
                "Liquidity Amount".to_string(),
                liq_contrib,
                format!("${:.0} liquidity provided", features.liquidity_usd),
            ));
        }

        // Holder concentration
        if concentration > 0.7 {
            let concentration_contrib = *weights.get("holder_concentration").unwrap_or(&0.0);
            score += concentration_contrib;
            factor_contributions.push((
                "High Concentration".to_string(),
                concentration_contrib,
                format!("Top 10 holders control {:.1}% of supply", features.top_10_holders_percent),
            ));
        }

        if features.holder_count < 100 {
            let low_holders_contrib = *weights.get("low_holders").unwrap_or(&0.0);
            score += low_holders_contrib;
            factor_contributions.push((
                "Low Holder Count".to_string(),
                low_holders_contrib,
                format!("Only {} holders - limited distribution", features.holder_count),
            ));
        }

        // Clamp final score to 0-100
        score = score.max(0.0).min(100.0);

        // Determine risk level (inverse of success)
        let risk_level = if score >= 70.0 {
            "Low"
        } else if score >= 50.0 {
            "Medium"
        } else if score >= 30.0 {
            "High"
        } else {
            "Critical"
        };

        // Sort factors by impact
        factor_contributions.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());

        let contributing_factors: Vec<PredictionFactor> = factor_contributions
            .into_iter()
            .take(5)
            .map(|(name, impact, desc)| PredictionFactor {
                factor_name: name,
                impact,
                description: desc,
            })
            .collect();

        // Calculate confidence based on number of factors and data quality
        let confidence = 0.75; // Could be improved with more sophisticated calculation

        LaunchPrediction {
            token_address: features.token_address.clone(),
            success_score: score,
            risk_level: risk_level.to_string(),
            contributing_factors,
            confidence,
            prediction_time: Utc::now().to_rfc3339(),
            model_version,
        }
    }

    pub async fn add_training_data(&self, data: LaunchTrainingData) -> Result<(), sqlx::Error> {
        let features_json = serde_json::to_string(&data.features).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO launch_training_data (id, token_address, features, actual_outcome, outcome_time, added_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&data.id)
        .bind(&data.token_address)
        .bind(&features_json)
        .bind(if data.actual_outcome { 1 } else { 0 })
        .bind(&data.outcome_time)
        .bind(&data.added_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn retrain(&self) -> Result<String, String> {
        // Fetch all training data
        let rows = sqlx::query("SELECT features, actual_outcome FROM launch_training_data")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if rows.len() < 10 {
            return Err("Need at least 10 samples to retrain".to_string());
        }

        // Parse training data
        let mut training_samples: Vec<(LaunchFeatures, bool)> = Vec::new();
        for row in rows {
            let features_json: String = row.get("features");
            let outcome: i32 = row.get("actual_outcome");

            if let Ok(features) = serde_json::from_str::<LaunchFeatures>(&features_json) {
                training_samples.push((features, outcome == 1));
            }
        }

        // Simple gradient descent for logistic regression
        // In a real implementation, this would be more sophisticated
        let new_weights = Self::default_weights();
        let new_intercept = 50.0;

        // Calculate accuracy on training set
        let mut correct = 0;
        for (features, actual) in &training_samples {
            let prediction = self.predict(features).await;
            let predicted_success = prediction.success_score > 50.0;
            if predicted_success == *actual {
                correct += 1;
            }
        }

        let accuracy = (correct as f64 / training_samples.len() as f64) * 100.0;

        // Update model
        *self.weights.write().await = new_weights.clone();
        *self.intercept.write().await = new_intercept;

        // Save model
        self.save_model(training_samples.len(), accuracy).await?;

        Ok(format!(
            "Retrained on {} samples, accuracy: {:.1}%",
            training_samples.len(),
            accuracy
        ))
    }

    async fn save_model(&self, sample_count: usize, accuracy: f64) -> Result<(), String> {
        let weights = self.weights.read().await;
        let intercept = *self.intercept.read().await;
        let version = *self.model_version.read().await + 1;

        let weights_json = serde_json::to_string(&*weights).map_err(|e| e.to_string())?;

        // Mark all models as inactive
        sqlx::query("UPDATE launch_models SET is_active = 0")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Insert new model
        sqlx::query(
            r#"
            INSERT INTO launch_models (version, weights, intercept, trained_on_samples, accuracy, created_at, is_active)
            VALUES (?, ?, ?, ?, ?, ?, 1)
            "#,
        )
        .bind(version as i32)
        .bind(&weights_json)
        .bind(intercept)
        .bind(sample_count as i32)
        .bind(accuracy)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        *self.model_version.write().await = version;

        Ok(())
    }

    pub async fn get_prediction_history(&self, limit: u32) -> Result<Vec<LaunchPrediction>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT token_address, success_score, risk_level, factors, confidence, prediction_time, model_version
            FROM launch_predictions
            ORDER BY prediction_time DESC
            LIMIT ?
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut predictions = Vec::new();
        for row in rows {
            let factors_json: String = row.get("factors");
            let factors: Vec<PredictionFactor> = serde_json::from_str(&factors_json).unwrap_or_default();

            predictions.push(LaunchPrediction {
                token_address: row.get("token_address"),
                success_score: row.get("success_score"),
                risk_level: row.get("risk_level"),
                contributing_factors: factors,
                confidence: row.get("confidence"),
                prediction_time: row.get("prediction_time"),
                model_version: row.get::<i32, _>("model_version") as u32,
            });
        }

        Ok(predictions)
    }

    pub async fn get_bias_report(&self) -> Result<BiasReport, sqlx::Error> {
        // Get all predictions with known outcomes
        let rows = sqlx::query(
            r#"
            SELECT success_score, actual_outcome
            FROM launch_predictions
            WHERE actual_outcome IS NOT NULL
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut total = 0;
        let mut correct = 0;
        let mut false_positives = 0;
        let mut false_negatives = 0;

        for row in rows {
            let score: f64 = row.get("success_score");
            let outcome: i32 = row.get("actual_outcome");

            let predicted_success = score > 50.0;
            let actual_success = outcome == 1;

            total += 1;
            if predicted_success == actual_success {
                correct += 1;
            } else if predicted_success && !actual_success {
                false_positives += 1;
            } else {
                false_negatives += 1;
            }
        }

        let accuracy = if total > 0 {
            (correct as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(BiasReport {
            total_predictions: total,
            correct_predictions: correct,
            accuracy,
            false_positives,
            false_negatives,
            by_liquidity_range: HashMap::new(), // Could be implemented with more data
            by_holder_count: HashMap::new(),    // Could be implemented with more data
        })
    }

    pub async fn load_latest_model(&self) -> Result<u32, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT version, weights, intercept
            FROM launch_models
            WHERE is_active = 1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let version: i32 = row.get("version");
            let weights_json: String = row.get("weights");
            let intercept: f64 = row.get("intercept");

            if let Ok(weights) = serde_json::from_str::<HashMap<String, f64>>(&weights_json) {
                *self.weights.write().await = weights;
                *self.intercept.write().await = intercept;
                *self.model_version.write().await = version as u32;
            }

            Ok(version as u32)
        } else {
            Ok(1)
        }
    }

    async fn store_prediction(&self, prediction: &LaunchPrediction) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let factors_json = serde_json::to_string(&prediction.contributing_factors).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO launch_predictions
            (id, token_address, success_score, risk_level, factors, confidence, prediction_time, model_version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&prediction.token_address)
        .bind(prediction.success_score)
        .bind(&prediction.risk_level)
        .bind(&factors_json)
        .bind(prediction.confidence)
        .bind(&prediction.prediction_time)
        .bind(prediction.model_version as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub async fn extract_token_features(token_address: String) -> Result<LaunchFeatures, String> {
    // Mock implementation - in production, this would query on-chain data
    Ok(LaunchFeatures {
        token_address,
        liquidity_usd: 50000.0,
        holder_count: 250,
        creator_history: 2,
        social_score: 65.0,
        code_verified: true,
        liquidity_locked: false,
        lock_duration_days: 0,
        token_supply: 1000000.0,
        initial_price_usd: 0.05,
        market_cap_usd: 50000.0,
        mint_disabled: true,
        freeze_disabled: true,
        ownership_renounced: false,
        top_10_holders_percent: 45.0,
    })
}

#[tauri::command]
pub async fn predict_launch_success(
    token_address: String,
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<LaunchPrediction, String> {
    // Extract features
    let features = extract_token_features(token_address).await?;

    // Make prediction
    let pred = predictor.read().await;
    let prediction = pred.predict(&features).await;

    // Store prediction
    pred.store_prediction(&prediction)
        .await
        .map_err(|e| format!("Failed to store prediction: {}", e))?;

    Ok(prediction)
}

#[tauri::command]
pub async fn get_launch_prediction_history(
    limit: Option<u32>,
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<Vec<LaunchPrediction>, String> {
    let pred = predictor.read().await;
    pred.get_prediction_history(limit.unwrap_or(50))
        .await
        .map_err(|e| format!("Failed to get prediction history: {}", e))
}

#[tauri::command]
pub async fn add_launch_training_data(
    token_address: String,
    actual_outcome: bool,
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<(), String> {
    // Extract features for the token
    let features = extract_token_features(token_address.clone()).await?;

    let training_data = LaunchTrainingData {
        id: Uuid::new_v4().to_string(),
        token_address,
        features,
        actual_outcome,
        outcome_time: Utc::now().to_rfc3339(),
        added_at: Utc::now().to_rfc3339(),
    };

    let pred = predictor.read().await;
    pred.add_training_data(training_data)
        .await
        .map_err(|e| format!("Failed to add training data: {}", e))
}

#[tauri::command]
pub async fn retrain_launch_model(
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<String, String> {
    let pred = predictor.read().await;
    pred.retrain().await
}

#[tauri::command]
pub async fn load_latest_launch_model(
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<u32, String> {
    let pred = predictor.read().await;
    pred.load_latest_model()
        .await
        .map_err(|e| format!("Failed to load model: {}", e))
}

#[tauri::command]
pub async fn get_launch_bias_report(
    predictor: State<'_, SharedLaunchPredictor>,
) -> Result<BiasReport, String> {
    let pred = predictor.read().await;
    pred.get_bias_report()
        .await
        .map_err(|e| format!("Failed to get bias report: {}", e))
}
