use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

use super::sentiment_engine::SentimentSnapshot;
use super::trend_engine::TrendRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeReading {
    pub token: String,
    pub fomo_score: f32,
    pub fomo_level: String,
    pub fud_score: f32,
    pub fud_level: String,
    pub drivers: HashMap<String, f32>,
    pub updated_at: i64,
}

pub struct GaugeEngine;

impl GaugeEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn update_gauges(
        &self,
        pool: &SqlitePool,
        snapshots: &[SentimentSnapshot],
        trends: &[TrendRecord],
    ) -> Result<Vec<GaugeReading>, sqlx::Error> {
        let mut readings = Vec::new();
        let trend_map: HashMap<(&str, i64), &TrendRecord> = trends
            .iter()
            .map(|t| ((t.token.as_str(), t.window_minutes), t))
            .collect();

        for snapshot in snapshots {
            let trend_key = (snapshot.token.as_str(), 60);
            let trend = trend_map.get(&trend_key).copied();
            let acceleration = trend.map(|t| t.acceleration).unwrap_or(0.0);

            let positive_sentiment = ((snapshot.avg_score + 1.0) / 2.0).clamp(0.0, 1.0);
            let negative_sentiment = 1.0 - positive_sentiment;
            let momentum_norm = ((snapshot.momentum + 1.0) / 2.0).clamp(0.0, 1.0);
            let acceleration_norm = ((acceleration + 1.0) / 2.0).clamp(0.0, 1.0);
            let whale_correlation = 0.5;

            let fomo_score = (0.45 * positive_sentiment
                + 0.3 * momentum_norm
                + 0.15 * acceleration_norm
                + 0.1 * whale_correlation)
                .clamp(0.0, 1.0);

            let fud_score = (0.45 * negative_sentiment
                + 0.3 * (1.0 - momentum_norm)
                + 0.15 * (1.0 - acceleration_norm)
                + 0.1 * (1.0 - whale_correlation))
                .clamp(0.0, 1.0);

            let fomo_level = Self::score_to_level(fomo_score);
            let fud_level = Self::score_to_level(fud_score);

            let mut drivers = HashMap::new();
            drivers.insert("positive_sentiment".to_string(), positive_sentiment);
            drivers.insert("momentum".to_string(), momentum_norm);
            drivers.insert("acceleration".to_string(), acceleration_norm);
            drivers.insert("whale_correlation".to_string(), whale_correlation);

            let now = Utc::now().timestamp();
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO social_gauges 
                (token, fomo_score, fomo_level, fud_score, fud_level, drivers, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
            )
            .bind(&snapshot.token)
            .bind(fomo_score)
            .bind(&fomo_level)
            .bind(fud_score)
            .bind(&fud_level)
            .bind(serde_json::to_string(&drivers).unwrap_or_else(|_| "{}".to_string()))
            .bind(now)
            .execute(pool)
            .await?;

            readings.push(GaugeReading {
                token: snapshot.token.clone(),
                fomo_score,
                fomo_level,
                fud_score,
                fud_level,
                drivers,
                updated_at: now,
            });
        }

        Ok(readings)
    }

    pub async fn fetch_gauges(
        &self,
        pool: &SqlitePool,
        token: Option<&str>,
    ) -> Result<Vec<GaugeReading>, sqlx::Error> {
        let rows = if let Some(tok) = token {
            sqlx::query(
                "SELECT token, fomo_score, fomo_level, fud_score, fud_level, drivers, updated_at \
                 FROM social_gauges WHERE token = ?1",
            )
            .bind(tok)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query(
                "SELECT token, fomo_score, fomo_level, fud_score, fud_level, drivers, updated_at \
                 FROM social_gauges ORDER BY updated_at DESC",
            )
            .fetch_all(pool)
            .await?
        };

        let mut readings = Vec::new();
        for row in rows {
            let drivers_json: String = row.try_get("drivers").unwrap_or_default();
            let drivers: HashMap<String, f32> =
                serde_json::from_str(&drivers_json).unwrap_or_default();

            readings.push(GaugeReading {
                token: row.try_get("token").unwrap_or_default(),
                fomo_score: row.try_get("fomo_score").unwrap_or(0.0),
                fomo_level: row.try_get("fomo_level").unwrap_or_default(),
                fud_score: row.try_get("fud_score").unwrap_or(0.0),
                fud_level: row.try_get("fud_level").unwrap_or_default(),
                drivers,
                updated_at: row.try_get("updated_at").unwrap_or(0),
            });
        }

        Ok(readings)
    }

    fn score_to_level(score: f32) -> String {
        if score >= 0.7 {
            "high".to_string()
        } else if score >= 0.4 {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }
}
