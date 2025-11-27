use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::Row;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::gauges::{GaugeEngine, GaugeReading};
use super::influencer::{InfluencerEngine, InfluencerScore};
use super::sentiment_engine::{SentimentEngine, SentimentSnapshot};
use super::trend_engine::{TrendEngine, TrendRecord, DEFAULT_WINDOWS};
use crate::social::cache::SocialCache;
use crate::social::models::SocialPost;

pub type SharedSocialAnalysisService = Arc<RwLock<SocialAnalysisService>>;

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub sentiments_analyzed: usize,
    pub trends_updated: usize,
    pub influencers_scored: usize,
    pub gauges_computed: usize,
}

pub struct SocialAnalysisService {
    sentiment_engine: SentimentEngine,
    trend_engine: TrendEngine,
    influencer_engine: InfluencerEngine,
    gauge_engine: GaugeEngine,
    cache: SocialCache,
}

impl SocialAnalysisService {
    pub fn new(cache: SocialCache) -> Self {
        let sentiment_engine = SentimentEngine::new();
        let trend_engine = TrendEngine::new(DEFAULT_WINDOWS.to_vec());
        let influencer_engine = InfluencerEngine::default();
        let gauge_engine = GaugeEngine::new();

        Self {
            sentiment_engine,
            trend_engine,
            influencer_engine,
            gauge_engine,
            cache,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), AnalysisError> {
        let pool = self.cache.pool();
        self.sentiment_engine.load_lexicon_from_db(pool).await?;
        Ok(())
    }

    pub async fn list_tokens(&self) -> Result<Vec<String>, AnalysisError> {
        let rows = sqlx::query("SELECT DISTINCT token FROM social_posts WHERE token IS NOT NULL")
            .fetch_all(self.cache.pool())
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| r.try_get::<String, _>("token").ok())
            .collect())
    }

    async fn fetch_pending_posts(
        &self,
        token: &str,
    ) -> Result<Vec<(String, SocialPost)>, AnalysisError> {
        let rows = sqlx::query(
            r#"
            SELECT p.post_data FROM social_posts p
            LEFT JOIN sentiment_scores s ON s.post_id = p.id
            WHERE p.token = ?1 AND s.post_id IS NULL
            ORDER BY p.timestamp ASC
            "#,
        )
        .bind(token)
        .fetch_all(self.cache.pool())
        .await?;

        let mut items = Vec::new();
        for row in rows {
            let data: String = row.try_get("post_data")?;
            let post: SocialPost = serde_json::from_str(&data)?;
            items.push((token.to_string(), post));
        }
        Ok(items)
    }

    pub async fn run_full_analysis(
        &mut self,
        token: &str,
    ) -> Result<AnalysisSummary, AnalysisError> {
        let pool = self.cache.pool();

        let pending = self.fetch_pending_posts(token).await?;
        if !pending.is_empty() {
            self.sentiment_engine
                .batch_analyze_posts(&pending, pool)
                .await?;
        }

        let snapshot = self
            .sentiment_engine
            .compute_sentiment_snapshot(pool, token, None)
            .await?;

        let trends = self.trend_engine.update_trends(pool, token).await?;

        let influencers = self
            .influencer_engine
            .compute_influencer_scores(pool, token, 86400, snapshot.avg_score)
            .await?;

        let gauges = self
            .gauge_engine
            .update_gauges(pool, &[snapshot.clone()], &trends)
            .await?;

        Ok(AnalysisSummary {
            sentiments_analyzed: pending.len(),
            trends_updated: trends.len(),
            influencers_scored: influencers.len(),
            gauges_computed: gauges.len(),
        })
    }

    pub async fn run_analysis_for_tokens(
        &mut self,
        tokens: &[String],
    ) -> Result<AnalysisSummary, AnalysisError> {
        let mut total = AnalysisSummary {
            sentiments_analyzed: 0,
            trends_updated: 0,
            influencers_scored: 0,
            gauges_computed: 0,
        };

        for token in tokens {
            let summary = self.run_full_analysis(token).await?;
            total.sentiments_analyzed += summary.sentiments_analyzed;
            total.trends_updated += summary.trends_updated;
            total.influencers_scored += summary.influencers_scored;
            total.gauges_computed += summary.gauges_computed;
        }

        Ok(total)
    }

    pub async fn run_analysis_all(&mut self) -> Result<AnalysisSummary, AnalysisError> {
        let tokens = self.list_tokens().await?;
        self.run_analysis_for_tokens(&tokens).await
    }

    pub async fn get_sentiment_snapshot(
        &self,
        token: &str,
    ) -> Result<Option<SentimentSnapshot>, AnalysisError> {
        let pool = self.cache.pool();
        Ok(self
            .sentiment_engine
            .get_sentiment_snapshot(pool, token)
            .await?)
    }

    pub async fn get_sentiment_snapshots(
        &self,
        token: Option<&str>,
    ) -> Result<Vec<SentimentSnapshot>, AnalysisError> {
        let pool = self.cache.pool();

        if let Some(tok) = token {
            if let Some(snapshot) = self
                .sentiment_engine
                .get_sentiment_snapshot(pool, tok)
                .await?
            {
                Ok(vec![snapshot])
            } else {
                Ok(Vec::new())
            }
        } else {
            let rows = sqlx::query(
                r#"
                SELECT token FROM sentiment_snapshots ORDER BY updated_at DESC
                "#,
            )
            .fetch_all(pool)
            .await?;

            let mut snapshots = Vec::new();
            for row in rows {
                let tok: String = row.try_get("token")?;
                if let Some(snapshot) = self
                    .sentiment_engine
                    .get_sentiment_snapshot(pool, &tok)
                    .await?
                {
                    snapshots.push(snapshot);
                }
            }
            Ok(snapshots)
        }
    }

    pub async fn get_trending_tokens(
        &self,
        window: Option<i64>,
    ) -> Result<Vec<TrendRecord>, AnalysisError> {
        let pool = self.cache.pool();
        Ok(self.trend_engine.fetch_trends(pool, None, window).await?)
    }

    pub async fn get_token_trends(&self, token: &str) -> Result<Vec<TrendRecord>, AnalysisError> {
        let pool = self.cache.pool();
        Ok(self
            .trend_engine
            .fetch_trends(pool, Some(token), None)
            .await?)
    }

    pub async fn get_influencer_scores(
        &self,
        token: Option<&str>,
        min_impact: Option<f32>,
    ) -> Result<Vec<InfluencerScore>, AnalysisError> {
        let pool = self.cache.pool();
        Ok(self
            .influencer_engine
            .fetch_influencer_scores(pool, token, min_impact)
            .await?)
    }

    pub async fn get_fomo_fud_gauges(
        &self,
        token: Option<&str>,
    ) -> Result<Vec<GaugeReading>, AnalysisError> {
        let pool = self.cache.pool();
        Ok(self.gauge_engine.fetch_gauges(pool, token).await?)
    }
}
