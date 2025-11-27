use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};

use crate::social::models::SocialPost;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluencerScore {
    pub influencer: String,
    pub follower_score: f32,
    pub engagement_score: f32,
    pub accuracy_score: f32,
    pub impact_score: f32,
    pub sample_size: i32,
    pub tokens: Vec<String>,
    pub updated_at: i64,
}

struct InfluencerStats {
    engagement_total: i64,
    sentiment_sum: f32,
    post_count: i32,
    tokens: HashSet<String>,
}

pub struct InfluencerEngine {
    follower_weight: f32,
    engagement_weight: f32,
    accuracy_weight: f32,
}

impl InfluencerEngine {
    pub fn new(follower_weight: f32, engagement_weight: f32, accuracy_weight: f32) -> Self {
        let total = follower_weight + engagement_weight + accuracy_weight;
        Self {
            follower_weight: follower_weight / total,
            engagement_weight: engagement_weight / total,
            accuracy_weight: accuracy_weight / total,
        }
    }

    pub fn default() -> Self {
        Self::new(0.35, 0.4, 0.25)
    }

    pub async fn compute_influencer_scores(
        &self,
        pool: &SqlitePool,
        token: &str,
        lookback_secs: i64,
        snapshot_avg: f32,
    ) -> Result<Vec<InfluencerScore>, sqlx::Error> {
        let cutoff = Utc::now().timestamp() - lookback_secs;
        let rows = sqlx::query(
            r#"
            SELECT p.post_data, s.score FROM social_posts p
            JOIN sentiment_scores s ON s.post_id = p.id
            WHERE p.token = ?1 AND p.timestamp >= ?2
            "#,
        )
        .bind(token)
        .bind(cutoff)
        .fetch_all(pool)
        .await?;

        let mut influencer_data: HashMap<String, InfluencerStats> = HashMap::new();

        for row in rows {
            let data: String = row.try_get("post_data")?;
            let score: f32 = row.try_get("score")?;
            let post: SocialPost = serde_json::from_str(&data)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let entry = influencer_data
                .entry(post.author.clone())
                .or_insert_with(|| InfluencerStats {
                    engagement_total: 0,
                    sentiment_sum: 0.0,
                    post_count: 0,
                    tokens: HashSet::new(),
                });
            entry.engagement_total += post.engagement as i64;
            entry.sentiment_sum += score;
            entry.post_count += 1;
            entry.tokens.insert(token.to_string());
        }

        let max_engagement = influencer_data
            .values()
            .map(|s| s.engagement_total)
            .max()
            .unwrap_or(1)
            .max(1) as f32;

        let mut scores = Vec::new();

        for (influencer, stats) in influencer_data {
            let sample_size = stats.post_count;
            let normalized_engagement = stats.engagement_total as f32 / max_engagement;

            let follower_score =
                self.normalize_follower_score(stats.engagement_total as f32, max_engagement);
            let engagement_score = normalized_engagement.min(1.0).max(0.0);

            let avg_sentiment = if stats.post_count > 0 {
                stats.sentiment_sum / stats.post_count as f32
            } else {
                0.0
            };

            let accuracy_score =
                (1.0 - ((avg_sentiment - snapshot_avg).abs() / 2.0)).clamp(0.0, 1.0);

            let impact_score = (follower_score * self.follower_weight)
                + (engagement_score * self.engagement_weight)
                + (accuracy_score * self.accuracy_weight);

            let tokens: Vec<String> = stats.tokens.iter().cloned().collect();
            let now = Utc::now().timestamp();

            sqlx::query(
                r#"
                INSERT OR REPLACE INTO social_influencer_scores 
                (influencer, follower_score, engagement_score, accuracy_score, impact_score, sample_size, tokens, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
            )
            .bind(&influencer)
            .bind(follower_score)
            .bind(engagement_score)
            .bind(accuracy_score)
            .bind(impact_score)
            .bind(sample_size)
            .bind(serde_json::to_string(&tokens).unwrap_or_else(|_| "[]".to_string()))
            .bind(now)
            .execute(pool)
            .await?;

            scores.push(InfluencerScore {
                influencer,
                follower_score,
                engagement_score,
                accuracy_score,
                impact_score,
                sample_size,
                tokens,
                updated_at: now,
            });
        }

        Ok(scores)
    }

    pub async fn fetch_influencer_scores(
        &self,
        pool: &SqlitePool,
        token: Option<&str>,
        min_impact: Option<f32>,
    ) -> Result<Vec<InfluencerScore>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT influencer, follower_score, engagement_score, accuracy_score, impact_score, sample_size, tokens, updated_at
            FROM social_influencer_scores
            "#,
        );

        let mut where_clauses = Vec::new();

        if token.is_some() {
            where_clauses.push("tokens LIKE ?1");
        }

        if min_impact.is_some() {
            where_clauses.push(if token.is_some() {
                "impact_score >= ?2"
            } else {
                "impact_score >= ?1"
            });
        }

        if !where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&where_clauses.join(" AND "));
        }

        query.push_str(" ORDER BY impact_score DESC");

        let mut sql_query = sqlx::query(&query);

        if let Some(tok) = token {
            sql_query = sql_query.bind(format!("%{}%", tok));
        }

        if let Some(impact) = min_impact {
            sql_query = sql_query.bind(impact);
        }

        let rows = sql_query.fetch_all(pool).await?;

        let mut scores = Vec::new();
        for row in rows {
            let tokens_json: String = row.try_get("tokens")?;
            let tokens: Vec<String> = serde_json::from_str(&tokens_json).unwrap_or_default();

            scores.push(InfluencerScore {
                influencer: row.try_get("influencer")?,
                follower_score: row.try_get("follower_score")?,
                engagement_score: row.try_get("engagement_score")?,
                accuracy_score: row.try_get("accuracy_score")?,
                impact_score: row.try_get("impact_score")?,
                sample_size: row.try_get("sample_size")?,
                tokens,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(scores)
    }

    fn normalize_follower_score(&self, engagement: f32, max_engagement: f32) -> f32 {
        let adjusted_max = max_engagement.max(1.0);
        (engagement.ln_1p() / adjusted_max.ln_1p()).clamp(0.0, 1.0)
    }
}
