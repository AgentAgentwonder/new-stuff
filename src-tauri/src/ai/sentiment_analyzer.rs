// Sentiment Analysis Module
// Aggregates sentiment from Twitter, Reddit, Discord, and on-chain activity

use super::types::*;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

pub struct SentimentAnalyzer {
    db: SqlitePool,
}

impl SentimentAnalyzer {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Get aggregated sentiment analysis for a token
    pub async fn analyze_token_sentiment(
        &self,
        token_mint: &str,
        timeframe_hours: Option<u64>,
    ) -> AiResult<SentimentAnalysis> {
        let timeframe = timeframe_hours.unwrap_or(24);

        // Fetch recent sentiment scores from database
        let scores = self.get_recent_scores(token_mint, timeframe).await?;

        if scores.is_empty() {
            return Err(AiError::InsufficientData(
                "No sentiment data available for this token".to_string(),
            ));
        }

        // Calculate weighted average sentiment
        let (overall_sentiment, overall_confidence) = self.calculate_weighted_sentiment(&scores);

        // Determine trend
        let trend = self.calculate_sentiment_trend(&scores);

        Ok(SentimentAnalysis {
            token_mint: token_mint.to_string(),
            token_symbol: None, // TODO: Fetch from token registry
            overall_sentiment,
            overall_confidence,
            sources: scores,
            trend,
            analyzed_at: Utc::now(),
        })
    }

    /// Get sentiment trend over time
    pub async fn get_sentiment_trend(
        &self,
        token_mint: &str,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> AiResult<Vec<SentimentDataPoint>> {
        let scores = sqlx::query_as::<_, SentimentScoreRow>(
            r#"
            SELECT
                sentiment_score,
                confidence,
                positive_mentions + negative_mentions + neutral_mentions as total_mentions,
                timestamp
            FROM sentiment_scores
            WHERE token_mint = ?
            AND timestamp BETWEEN ? AND ?
            ORDER BY timestamp ASC
            "#
        )
        .bind(token_mint)
        .bind(start_time.to_rfc3339())
        .bind(end_time.to_rfc3339())
        .fetch_all(&self.db)
        .await?;

        Ok(scores
            .into_iter()
            .map(|row| SentimentDataPoint {
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.timestamp)
                    .unwrap_or_else(|_| chrono::DateTime::<Utc>::default().into())
                    .with_timezone(&Utc),
                sentiment_score: row.sentiment_score,
                confidence: row.confidence,
                sample_size: row.total_mentions.unwrap_or(0),
            })
            .collect())
    }

    /// Refresh sentiment data for a token (fetch from APIs)
    pub async fn refresh_sentiment(&self, token_mint: &str) -> AiResult<()> {
        // TODO: Implement actual API fetching
        // For now, this is a placeholder that would:
        // 1. Fetch from Twitter API
        // 2. Fetch from Reddit API
        // 3. Fetch on-chain metrics
        // 4. Aggregate and store results

        log::info!("Refreshing sentiment data for token: {}", token_mint);

        // Mock implementation - generate sample data
        self.generate_mock_sentiment(token_mint).await?;

        Ok(())
    }

    // Private helper methods

    async fn get_recent_scores(
        &self,
        token_mint: &str,
        hours: u64,
    ) -> AiResult<Vec<SentimentScore>> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours as i64);

        let scores = sqlx::query_as::<_, SentimentScoreRow>(
            r#"
            SELECT *
            FROM sentiment_scores
            WHERE token_mint = ?
            AND timestamp >= ?
            ORDER BY timestamp DESC
            "#
        )
        .bind(token_mint)
        .bind(cutoff_time.to_rfc3339())
        .fetch_all(&self.db)
        .await?;

        Ok(scores.into_iter().map(|row| row.into()).collect())
    }

    fn calculate_weighted_sentiment(&self, scores: &[SentimentScore]) -> (f64, f64) {
        if scores.is_empty() {
            return (0.0, 0.0);
        }

        let mut weighted_sentiment = 0.0;
        let mut total_weight = 0.0;
        let mut confidence_sum = 0.0;

        for score in scores {
            // Weight by confidence and sample size
            let weight = score.confidence * (1.0 + (score.sample_size.unwrap_or(0) as f64).ln().max(0.0));
            weighted_sentiment += score.sentiment_score * weight;
            total_weight += weight;
            confidence_sum += score.confidence;
        }

        let overall_sentiment = if total_weight > 0.0 {
            weighted_sentiment / total_weight
        } else {
            0.0
        };

        let overall_confidence = confidence_sum / scores.len() as f64;

        (overall_sentiment, overall_confidence)
    }

    fn calculate_sentiment_trend(&self, scores: &[SentimentScore]) -> SentimentTrend {
        if scores.len() < 2 {
            return SentimentTrend::Stable;
        }

        // Sort by timestamp (oldest to newest)
        let mut sorted_scores = scores.to_vec();
        sorted_scores.sort_by_key(|s| s.timestamp);

        // Calculate trend using linear regression slope
        let n = sorted_scores.len() as f64;
        let x_values: Vec<f64> = (0..sorted_scores.len()).map(|i| i as f64).collect();
        let y_values: Vec<f64> = sorted_scores.iter().map(|s| s.sentiment_score).collect();

        let x_mean = x_values.iter().sum::<f64>() / n;
        let y_mean = y_values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..sorted_scores.len() {
            numerator += (x_values[i] - x_mean) * (y_values[i] - y_mean);
            denominator += (x_values[i] - x_mean).powi(2);
        }

        let slope = if denominator != 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        // Calculate volatility
        let variance = y_values.iter().map(|&y| (y - y_mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();

        // Classify trend
        if std_dev > 0.3 {
            SentimentTrend::Volatile
        } else if slope > 0.05 {
            SentimentTrend::Rising
        } else if slope < -0.05 {
            SentimentTrend::Falling
        } else {
            SentimentTrend::Stable
        }
    }

    async fn generate_mock_sentiment(&self, token_mint: &str) -> AiResult<()> {
        let sources = vec![
            SentimentSource::Twitter,
            SentimentSource::Reddit,
            SentimentSource::OnChain,
        ];

        for source in sources {
            let id = Uuid::new_v4().to_string();
            let sentiment_score = rand::random_range(-1.0..1.0);
            let confidence = rand::random_range(0.5..0.95);
            let positive = rand::random_range(10..100);
            let negative = rand::random_range(5..50);
            let neutral = rand::random_range(20..80);
            let timestamp = Utc::now();

            sqlx::query(
                r#"
                INSERT INTO sentiment_scores (
                    id, token_mint, sentiment_score, confidence, source,
                    sample_size, positive_mentions, negative_mentions, neutral_mentions,
                    timestamp, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&id)
            .bind(token_mint)
            .bind(sentiment_score)
            .bind(confidence)
            .bind(source_to_string(source))
            .bind(positive + negative + neutral)
            .bind(positive)
            .bind(negative)
            .bind(neutral)
            .bind(timestamp.to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }
}

// Helper structs for database queries

#[derive(sqlx::FromRow)]
struct SentimentScoreRow {
    id: String,
    token_mint: String,
    token_symbol: Option<String>,
    sentiment_score: f64,
    confidence: f64,
    source: String,
    sample_size: Option<i32>,
    positive_mentions: i32,
    negative_mentions: i32,
    neutral_mentions: i32,
    timestamp: String,
    created_at: String,
    total_mentions: Option<i32>,
}

impl From<SentimentScoreRow> for SentimentScore {
    fn from(row: SentimentScoreRow) -> Self {
        Self {
            id: row.id,
            token_mint: row.token_mint,
            token_symbol: row.token_symbol,
            sentiment_score: row.sentiment_score,
            confidence: row.confidence,
            source: string_to_source(&row.source),
            sample_size: row.sample_size,
            positive_mentions: row.positive_mentions,
            negative_mentions: row.negative_mentions,
            neutral_mentions: row.neutral_mentions,
            timestamp: chrono::DateTime::parse_from_rfc3339(&row.timestamp)
                .unwrap()
                .with_timezone(&Utc),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap()
                .with_timezone(&Utc),
        }
    }
}

fn string_to_source(s: &str) -> SentimentSource {
    match s {
        "twitter" => SentimentSource::Twitter,
        "reddit" => SentimentSource::Reddit,
        "discord" => SentimentSource::Discord,
        "onchain" => SentimentSource::OnChain,
        _ => SentimentSource::Twitter,
    }
}

fn source_to_string(source: SentimentSource) -> &'static str {
    match source {
        SentimentSource::Twitter => "twitter",
        SentimentSource::Reddit => "reddit",
        SentimentSource::Discord => "discord",
        SentimentSource::OnChain => "onchain",
    }
}
