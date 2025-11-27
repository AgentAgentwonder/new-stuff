pub mod features;
pub mod inference;
pub mod model;
pub mod training;

pub use features::*;
pub use inference::*;
pub use model::*;
pub use training::*;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPrediction {
    pub token_address: String,
    pub success_probability: f64,
    pub risk_level: String,
    pub confidence: f64,
    pub predicted_peak_timeframe: Option<String>,
    pub feature_scores: Vec<FeatureScore>,
    pub early_warnings: Vec<EarlyWarning>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureScore {
    pub feature_name: String,
    pub value: f64,
    pub importance: f64,
    pub impact: String, // "Positive", "Negative", "Neutral"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarlyWarning {
    pub warning_type: String,
    pub severity: String, // "Low", "Medium", "High", "Critical"
    pub message: String,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionHistory {
    pub token_address: String,
    pub predictions: Vec<LaunchPrediction>,
}

#[derive(Clone)]
pub struct LaunchPredictor {
    pool: Pool<Sqlite>,
    model: Arc<RwLock<LaunchModel>>,
}

pub type SharedLaunchPredictor = Arc<RwLock<LaunchPredictor>>;

impl LaunchPredictor {
    pub async fn new(app: &AppHandle) -> Result<Self, sqlx::Error> {
        let mut db_path = app.path().app_data_dir().map_err(|_| {
            sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "App data dir not found",
            ))
        })?;

        std::fs::create_dir_all(&db_path).map_err(sqlx::Error::Io)?;
        db_path.push("launch_predictor.db");

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        Self::with_pool(pool).await
    }

    pub async fn with_pool(pool: Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let model = Arc::new(RwLock::new(LaunchModel::new()));
        let predictor = Self { pool, model };
        predictor.initialize().await?;
        Ok(predictor)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        // Create predictions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS launch_predictions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_address TEXT NOT NULL,
                success_probability REAL NOT NULL,
                risk_level TEXT NOT NULL,
                confidence REAL NOT NULL,
                predicted_peak_timeframe TEXT,
                feature_scores TEXT NOT NULL,
                early_warnings TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create index
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_predictions_token_timestamp 
            ON launch_predictions(token_address, timestamp DESC);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create features table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS token_features (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_address TEXT NOT NULL,
                features TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create models table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS launch_models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL,
                model_data TEXT NOT NULL,
                metrics TEXT,
                training_date TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create training data table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS training_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_address TEXT NOT NULL,
                features TEXT NOT NULL,
                actual_outcome REAL NOT NULL,
                outcome_date TEXT NOT NULL,
                added_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn predict(
        &self,
        token_address: &str,
        features: TokenFeatures,
    ) -> Result<LaunchPrediction, sqlx::Error> {
        let model = self.model.read().await;
        let prediction = model.predict(&features);

        // Store in database
        let feature_scores_json =
            serde_json::to_string(&prediction.feature_scores).unwrap_or_default();
        let early_warnings_json =
            serde_json::to_string(&prediction.early_warnings).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO launch_predictions 
            (token_address, success_probability, risk_level, confidence, 
             predicted_peak_timeframe, feature_scores, early_warnings, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&prediction.token_address)
        .bind(prediction.success_probability)
        .bind(&prediction.risk_level)
        .bind(prediction.confidence)
        .bind(&prediction.predicted_peak_timeframe)
        .bind(&feature_scores_json)
        .bind(&early_warnings_json)
        .bind(&prediction.timestamp)
        .execute(&self.pool)
        .await?;

        // Store features
        let features_json = serde_json::to_string(&features).unwrap_or_default();
        sqlx::query(
            r#"
            INSERT INTO token_features (token_address, features, timestamp)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(token_address)
        .bind(&features_json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(prediction)
    }

    pub async fn get_prediction_history(
        &self,
        token_address: &str,
        days: u32,
    ) -> Result<PredictionHistory, sqlx::Error> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let rows = sqlx::query(
            r#"
            SELECT * FROM launch_predictions
            WHERE token_address = ? AND timestamp >= ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(token_address)
        .bind(&cutoff_str)
        .fetch_all(&self.pool)
        .await?;

        let mut predictions = Vec::new();
        for row in rows {
            let feature_scores_json: String = row.try_get("feature_scores")?;
            let early_warnings_json: String = row.try_get("early_warnings")?;

            let feature_scores: Vec<FeatureScore> =
                serde_json::from_str(&feature_scores_json).unwrap_or_default();
            let early_warnings: Vec<EarlyWarning> =
                serde_json::from_str(&early_warnings_json).unwrap_or_default();

            predictions.push(LaunchPrediction {
                token_address: row.try_get("token_address")?,
                success_probability: row.try_get("success_probability")?,
                risk_level: row.try_get("risk_level")?,
                confidence: row.try_get("confidence")?,
                predicted_peak_timeframe: row.try_get("predicted_peak_timeframe")?,
                feature_scores,
                early_warnings,
                timestamp: row.try_get("timestamp")?,
            });
        }

        Ok(PredictionHistory {
            token_address: token_address.to_string(),
            predictions,
        })
    }

    pub async fn add_training_data(
        &self,
        token_address: &str,
        features: TokenFeatures,
        actual_outcome: f64,
    ) -> Result<(), sqlx::Error> {
        let features_json = serde_json::to_string(&features).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO training_data 
            (token_address, features, actual_outcome, outcome_date, added_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(token_address)
        .bind(&features_json)
        .bind(actual_outcome)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn retrain_model(&self) -> Result<ModelMetrics, sqlx::Error> {
        // Load training data
        let rows = sqlx::query("SELECT features, actual_outcome FROM training_data")
            .fetch_all(&self.pool)
            .await?;

        let mut training_samples = Vec::new();
        for row in rows {
            let features_json: String = row.try_get("features")?;
            let outcome: f64 = row.try_get("actual_outcome")?;

            if let Ok(features) = serde_json::from_str::<TokenFeatures>(&features_json) {
                training_samples.push((features, outcome));
            }
        }

        // Train new model
        let mut model = self.model.write().await;
        let metrics = model.train(training_samples);

        // Save model
        let model_json = model.to_json().map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;

        let metrics_json = serde_json::to_string(&metrics).unwrap_or_default();

        // Mark all existing models as inactive
        sqlx::query("UPDATE launch_models SET is_active = 0")
            .execute(&self.pool)
            .await?;

        // Insert new model
        sqlx::query(
            r#"
            INSERT INTO launch_models (version, model_data, metrics, training_date, is_active)
            VALUES (
                (SELECT COALESCE(MAX(version), 0) + 1 FROM launch_models),
                ?, ?, ?, 1
            )
            "#,
        )
        .bind(&model_json)
        .bind(&metrics_json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(metrics)
    }

    pub async fn generate_bias_report(&self) -> Result<LaunchBiasReport, sqlx::Error> {
        let rows = sqlx::query("SELECT features, actual_outcome FROM training_data")
            .fetch_all(&self.pool)
            .await?;

        let mut segment_counts: HashMap<String, (usize, usize)> = HashMap::new();
        let mut global_total = 0usize;
        let mut global_success = 0usize;

        for row in rows {
            let features_json: String = row.try_get("features")?;
            let outcome: f64 = row.try_get("actual_outcome")?;

            if let Ok(features) = serde_json::from_str::<TokenFeatures>(&features_json) {
                let segment = features.developer_category.clone();
                let entry = segment_counts.entry(segment).or_insert((0, 0));
                entry.0 += 1;
                if outcome >= 0.5 {
                    entry.1 += 1;
                    global_success += 1;
                }
                global_total += 1;
            }
        }

        let global_rate = if global_total > 0 {
            global_success as f64 / global_total as f64
        } else {
            0.0
        };

        let mut metrics = Vec::new();
        let mut flagged_segments = Vec::new();

        for (segment, (total, success)) in segment_counts {
            let rate = if total > 0 {
                success as f64 / total as f64
            } else {
                0.0
            };
            let delta = rate - global_rate;
            let adverse = total >= 5 && delta < -0.15;
            if adverse {
                flagged_segments.push(segment.clone());
            }
            metrics.push(LaunchBiasMetric {
                segment,
                sample_size: total,
                success_rate: rate,
                delta_from_global: delta,
                adverse_impact: adverse,
            });
        }

        metrics.sort_by(|a, b| b.sample_size.cmp(&a.sample_size));

        Ok(LaunchBiasReport {
            generated_at: Utc::now().to_rfc3339(),
            global_success_rate: global_rate,
            metrics,
            flagged_segments,
        })
    }

    pub async fn load_latest_model(&self) -> Result<(), sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT model_data FROM launch_models
            WHERE is_active = 1
            ORDER BY training_date DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let model_json: String = row.try_get("model_data")?;
            let loaded_model = LaunchModel::from_json(&model_json).map_err(|e| {
                sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })?;

            let mut model = self.model.write().await;
            *model = loaded_model;
        }

        Ok(())
    }
}
