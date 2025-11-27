use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, SqlitePool};
use std::collections::HashMap;

use crate::social::models::SocialPost;

pub const DEFAULT_WINDOWS: [i64; 3] = [15, 60, 1440];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendRecord {
    pub token: String,
    pub window_minutes: i64,
    pub mentions: i32,
    pub velocity: f32,
    pub acceleration: f32,
    pub volume_spike: f32,
    pub sentiment_avg: f32,
    pub engagement_total: i64,
    pub updated_at: i64,
}

pub struct TrendEngine {
    windows: Vec<i64>,
}

impl TrendEngine {
    pub fn new(windows: Vec<i64>) -> Self {
        Self { windows }
    }

    pub async fn update_trends(
        &self,
        pool: &SqlitePool,
        token: &str,
    ) -> Result<Vec<TrendRecord>, sqlx::Error> {
        let mut updates = Vec::new();

        for window in &self.windows {
            let cutoff = Utc::now().timestamp() - (window * 60);
            let posts = sqlx::query(
                "SELECT post_data, timestamp FROM social_posts WHERE token = ?1 AND timestamp >= ?2",
            )
            .bind(token)
            .bind(cutoff)
            .fetch_all(pool)
            .await?;

            let mut mentions = 0i32;
            let mut engagement_total = 0i64;

            let mut posts_by_id: HashMap<String, SocialPost> = HashMap::new();
            for row in posts {
                let data: String = row.try_get("post_data")?;
                let post: SocialPost = serde_json::from_str(&data)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
                engagement_total += post.engagement as i64;
                mentions += 1;
                posts_by_id.insert(post.id.clone(), post);
            }

            let sentiment_rows = sqlx::query(
                "SELECT score FROM sentiment_scores WHERE token = ?1 AND timestamp >= ?2",
            )
            .bind(token)
            .bind(cutoff)
            .fetch_all(pool)
            .await?;

            let mut sentiment_total = 0.0;
            for row in sentiment_rows {
                let score: f32 = row.try_get("score")?;
                sentiment_total += score;
            }

            let sentiment_avg = if mentions > 0 {
                sentiment_total / mentions as f32
            } else {
                0.0
            };

            let velocity = if *window > 0 {
                mentions as f32 / *window as f32
            } else {
                0.0
            };

            let previous = sqlx::query(
                "SELECT mentions, velocity FROM social_trends WHERE token = ?1 AND window_minutes = ?2",
            )
            .bind(token)
            .bind(window)
            .fetch_optional(pool)
            .await?;

            let (prev_mentions, prev_velocity) = if let Some(row) = previous {
                (
                    row.try_get::<i32, _>("mentions")?,
                    row.try_get::<f32, _>("velocity")?,
                )
            } else {
                (0, 0.0)
            };

            let acceleration = velocity - prev_velocity;
            let volume_spike = if prev_mentions > 0 {
                (mentions as f32 / prev_mentions as f32).min(10.0)
            } else if mentions > 0 {
                2.0
            } else {
                0.0
            };

            let now = Utc::now().timestamp();
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO social_trends 
                (token, window_minutes, mentions, velocity, acceleration, volume_spike, sentiment_avg, engagement_total, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )
            .bind(token)
            .bind(window)
            .bind(mentions)
            .bind(velocity)
            .bind(acceleration)
            .bind(volume_spike)
            .bind(sentiment_avg)
            .bind(engagement_total)
            .bind(now)
            .execute(pool)
            .await?;

            updates.push(TrendRecord {
                token: token.to_string(),
                window_minutes: *window,
                mentions,
                velocity,
                acceleration,
                volume_spike,
                sentiment_avg,
                engagement_total,
                updated_at: now,
            });
        }

        Ok(updates)
    }

    pub async fn fetch_trends(
        &self,
        pool: &SqlitePool,
        token: Option<&str>,
        window: Option<i64>,
    ) -> Result<Vec<TrendRecord>, sqlx::Error> {
        let mut query = String::from(
            "SELECT token, window_minutes, mentions, velocity, acceleration, volume_spike, sentiment_avg, engagement_total, updated_at FROM social_trends",
        );
        let mut params: Vec<String> = Vec::new();
        let mut binds: Vec<i64> = Vec::new();

        if token.is_some() || window.is_some() {
            query.push_str(" WHERE");
        }

        if let Some(tok) = token {
            query.push_str(" token = ?1");
            params.push(tok.to_string());
        }

        if let Some(win) = window {
            if token.is_some() {
                query.push_str(" AND window_minutes = ?2");
            } else {
                query.push_str(" window_minutes = ?1");
            }
            binds.push(win);
        }

        query.push_str(" ORDER BY updated_at DESC");

        let mut sql_query = sqlx::query(&query);

        for (idx, value) in params.iter().enumerate() {
            sql_query = sql_query.bind(value);
            if idx == 0 && window.is_some() {
                sql_query = sql_query.bind(window.unwrap());
            }
        }

        if token.is_none() && window.is_some() {
            sql_query = sql_query.bind(window.unwrap());
        }

        let rows = sql_query.fetch_all(pool).await?;

        let mut records = Vec::new();
        for row in rows {
            records.push(TrendRecord {
                token: row.try_get("token")?,
                window_minutes: row.try_get("window_minutes")?,
                mentions: row.try_get("mentions")?,
                velocity: row.try_get("velocity")?,
                acceleration: row.try_get("acceleration")?,
                volume_spike: row.try_get("volume_spike")?,
                sentiment_avg: row.try_get("sentiment_avg")?,
                engagement_total: row.try_get("engagement_total")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(records)
    }
}
