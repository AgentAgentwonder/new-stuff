use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::collections::HashMap;

use crate::social::models::{SentimentResult, SocialPost};

const DEFAULT_POSITIVE_WORDS: &[&str] = &[
    "good",
    "great",
    "excellent",
    "bullish",
    "moon",
    "pump",
    "profit",
    "gain",
    "win",
    "rocket",
    "buy",
    "up",
    "rise",
    "surge",
    "strong",
    "positive",
    "growth",
    "success",
    "amazing",
    "best",
    "love",
    "high",
    "rally",
    "boom",
    "outstanding",
    "superb",
    "breakthrough",
    "opportunity",
    "optimistic",
    "confident",
    "winning",
];

const DEFAULT_NEGATIVE_WORDS: &[&str] = &[
    "bad",
    "terrible",
    "bearish",
    "dump",
    "crash",
    "loss",
    "scam",
    "rug",
    "fail",
    "dead",
    "sell",
    "down",
    "drop",
    "fall",
    "weak",
    "negative",
    "decline",
    "worst",
    "hate",
    "low",
    "collapse",
    "panic",
    "disaster",
    "fraud",
    "broken",
    "disappointing",
    "risky",
    "warning",
    "avoid",
    "concern",
];

const NEGATION_WORDS: &[&str] = &[
    "not", "no", "never", "don't", "didn't", "won't", "can't", "cannot", "isn't", "aren't",
];

const POSITIVE_EMOJIS: &[&str] = &["🚀", "💎", "🌙", "📈", "💰", "🔥", "👍", "✅", "💪", "🎉"];
const NEGATIVE_EMOJIS: &[&str] = &["📉", "💀", "😢", "😡", "⚠️", "❌", "👎", "🚨", "⛔"];

const AMPLIFIERS: &[&str] = &[
    "very",
    "extremely",
    "really",
    "super",
    "absolutely",
    "totally",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconEntry {
    pub term: String,
    pub weight: f32,
    pub category: Option<String>,
    pub is_negation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSnapshot {
    pub token: String,
    pub avg_score: f32,
    pub momentum: f32,
    pub mention_count: i32,
    pub positive_mentions: i32,
    pub negative_mentions: i32,
    pub neutral_mentions: i32,
    pub confidence: f32,
    pub dominant_label: String,
    pub last_post_timestamp: i64,
    pub updated_at: i64,
}

pub struct SentimentEngine {
    positive_lexicon: HashMap<String, f32>,
    negative_lexicon: HashMap<String, f32>,
    negation_words: Vec<String>,
}

impl SentimentEngine {
    pub fn new() -> Self {
        let mut positive_lexicon = HashMap::new();
        for word in DEFAULT_POSITIVE_WORDS {
            positive_lexicon.insert(word.to_string(), 0.15);
        }
        for emoji in POSITIVE_EMOJIS {
            positive_lexicon.insert(emoji.to_string(), 0.25);
        }

        let mut negative_lexicon = HashMap::new();
        for word in DEFAULT_NEGATIVE_WORDS {
            negative_lexicon.insert(word.to_string(), 0.15);
        }
        for emoji in NEGATIVE_EMOJIS {
            negative_lexicon.insert(emoji.to_string(), 0.25);
        }

        let negation_words = NEGATION_WORDS.iter().map(|s| s.to_string()).collect();

        Self {
            positive_lexicon,
            negative_lexicon,
            negation_words,
        }
    }

    pub async fn load_lexicon_from_db(&mut self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        let rows = sqlx::query("SELECT term, weight, category, is_negation FROM sentiment_lexicon")
            .fetch_all(pool)
            .await?;

        for row in rows {
            let term: String = row.try_get("term")?;
            let weight: f32 = row.try_get("weight")?;
            let category: Option<String> = row.try_get("category").ok();
            let is_negation: i32 = row.try_get("is_negation")?;

            if is_negation != 0 {
                if !self.negation_words.contains(&term) {
                    self.negation_words.push(term.clone());
                }
            } else if category.as_deref() == Some("positive") {
                self.positive_lexicon.insert(term, weight);
            } else if category.as_deref() == Some("negative") {
                self.negative_lexicon.insert(term, weight);
            }
        }

        Ok(())
    }

    pub async fn save_lexicon_entry(
        &self,
        pool: &Pool<Sqlite>,
        entry: &LexiconEntry,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO sentiment_lexicon 
            (term, weight, category, is_negation, metadata, updated_at)
            VALUES (?1, ?2, ?3, ?4, NULL, ?5)
            "#,
        )
        .bind(&entry.term)
        .bind(entry.weight)
        .bind(&entry.category)
        .bind(if entry.is_negation { 1 } else { 0 })
        .bind(now)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub fn analyze_text(&self, text: &str) -> SentimentResult {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();

        let mut score = 0.0;
        let mut matches = 0;

        for (i, word) in words.iter().enumerate() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());

            let mut amplifier = 1.0;
            if i > 0 {
                let prev = words[i - 1].trim_matches(|c: char| !c.is_alphanumeric());
                if AMPLIFIERS.contains(&prev) {
                    amplifier = 1.5;
                }
            }

            let is_negated = if i > 0 {
                let prev = words[i - 1].trim_matches(|c: char| !c.is_alphanumeric());
                self.negation_words.contains(&prev.to_string())
            } else {
                false
            };

            if let Some(&weight) = self.positive_lexicon.get(cleaned) {
                let contribution = weight * amplifier;
                score += if is_negated {
                    -contribution
                } else {
                    contribution
                };
                matches += 1;
            }

            if let Some(&weight) = self.negative_lexicon.get(cleaned) {
                let contribution = weight * amplifier;
                score -= if is_negated {
                    -contribution
                } else {
                    contribution
                };
                matches += 1;
            }
        }

        for emoji in POSITIVE_EMOJIS {
            if text.contains(emoji) {
                score += 0.25;
                matches += 1;
            }
        }

        for emoji in NEGATIVE_EMOJIS {
            if text.contains(emoji) {
                score -= 0.25;
                matches += 1;
            }
        }

        let hashtag_count = text.matches('#').count();
        if hashtag_count > 3 {
            score += 0.1;
        }

        let normalized_score = score.max(-1.0).min(1.0);

        let label = if normalized_score > 0.2 {
            "positive".to_string()
        } else if normalized_score < -0.2 {
            "negative".to_string()
        } else {
            "neutral".to_string()
        };

        let confidence = if matches > 0 {
            normalized_score.abs().min(0.95)
        } else {
            0.1
        };

        SentimentResult {
            score: normalized_score,
            label,
            confidence,
        }
    }

    pub async fn batch_analyze_posts(
        &self,
        items: &[(String, SocialPost)],
        pool: &Pool<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        for (token, post) in items {
            let result = self.analyze_text(&post.text);

            sqlx::query(
                r#"
                INSERT OR REPLACE INTO sentiment_scores 
                (post_id, token, source, timestamp, score, label, confidence)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
            )
            .bind(&post.id)
            .bind(Some(token))
            .bind(&post.source)
            .bind(post.timestamp)
            .bind(result.score)
            .bind(&result.label)
            .bind(result.confidence)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_sentiment_snapshot(
        &self,
        pool: &Pool<Sqlite>,
        token: &str,
    ) -> Result<Option<SentimentSnapshot>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT avg_score, momentum, mention_count, positive_mentions, negative_mentions,
                   neutral_mentions, confidence, dominant_label, last_post_timestamp, updated_at
            FROM sentiment_snapshots WHERE token = ?1
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| SentimentSnapshot {
            token: token.to_string(),
            avg_score: r.try_get("avg_score").unwrap_or(0.0),
            momentum: r.try_get("momentum").unwrap_or(0.0),
            mention_count: r.try_get("mention_count").unwrap_or(0),
            positive_mentions: r.try_get("positive_mentions").unwrap_or(0),
            negative_mentions: r.try_get("negative_mentions").unwrap_or(0),
            neutral_mentions: r.try_get("neutral_mentions").unwrap_or(0),
            confidence: r.try_get("confidence").unwrap_or(0.0),
            dominant_label: r
                .try_get("dominant_label")
                .unwrap_or_else(|_| "neutral".to_string()),
            last_post_timestamp: r.try_get("last_post_timestamp").unwrap_or(0),
            updated_at: r.try_get("updated_at").unwrap_or(0),
        }))
    }

    pub async fn compute_sentiment_snapshot(
        &self,
        pool: &Pool<Sqlite>,
        token: &str,
        since_timestamp: Option<i64>,
    ) -> Result<SentimentSnapshot, sqlx::Error> {
        let cutoff = since_timestamp.unwrap_or_else(|| Utc::now().timestamp() - 86400);

        let rows = sqlx::query(
            "SELECT score, label, timestamp FROM sentiment_scores WHERE token = ?1 AND timestamp > ?2 ORDER BY timestamp DESC"
        )
        .bind(token)
        .bind(cutoff)
        .fetch_all(pool)
        .await?;

        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut neutral_count = 0;
        let mut total_score = 0.0;
        let mut last_timestamp = 0i64;
        let mut scores_vec: Vec<f32> = Vec::new();

        for row in &rows {
            let score: f32 = row.try_get("score").unwrap_or(0.0);
            let label: String = row
                .try_get("label")
                .unwrap_or_else(|_| "neutral".to_string());
            let timestamp: i64 = row.try_get("timestamp").unwrap_or(0);

            match label.as_str() {
                "positive" => positive_count += 1,
                "negative" => negative_count += 1,
                _ => neutral_count += 1,
            }

            total_score += score;
            scores_vec.push(score);
            if timestamp > last_timestamp {
                last_timestamp = timestamp;
            }
        }

        let mention_count = rows.len() as i32;
        let avg_score = if mention_count > 0 {
            total_score / mention_count as f32
        } else {
            0.0
        };

        let momentum = if scores_vec.len() >= 2 {
            let half = scores_vec.len() / 2;
            let recent_avg: f32 = scores_vec[0..half].iter().sum::<f32>() / half as f32;
            let older_avg: f32 =
                scores_vec[half..].iter().sum::<f32>() / (scores_vec.len() - half) as f32;
            recent_avg - older_avg
        } else {
            0.0
        };

        let dominant_label = if positive_count > negative_count && positive_count > neutral_count {
            "positive".to_string()
        } else if negative_count > positive_count && negative_count > neutral_count {
            "negative".to_string()
        } else {
            "neutral".to_string()
        };

        let confidence = if mention_count > 0 {
            let majority = positive_count.max(negative_count).max(neutral_count) as f32;
            (majority / mention_count as f32).min(0.95)
        } else {
            0.1
        };

        let now = Utc::now().timestamp();
        let snapshot = SentimentSnapshot {
            token: token.to_string(),
            avg_score,
            momentum,
            mention_count,
            positive_mentions: positive_count,
            negative_mentions: negative_count,
            neutral_mentions: neutral_count,
            confidence,
            dominant_label: dominant_label.clone(),
            last_post_timestamp: last_timestamp,
            updated_at: now,
        };

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO sentiment_snapshots
            (token, avg_score, momentum, mention_count, positive_mentions, negative_mentions,
             neutral_mentions, confidence, dominant_label, last_post_timestamp, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(token)
        .bind(avg_score)
        .bind(momentum)
        .bind(mention_count)
        .bind(positive_count)
        .bind(negative_count)
        .bind(neutral_count)
        .bind(confidence)
        .bind(&dominant_label)
        .bind(last_timestamp)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_sentiment() {
        let engine = SentimentEngine::new();
        let result =
            engine.analyze_text("This is amazing! Great bullish project going to the moon! 🚀");
        assert!(result.score > 0.2);
        assert_eq!(result.label, "positive");
    }

    #[test]
    fn test_negative_sentiment() {
        let engine = SentimentEngine::new();
        let result = engine.analyze_text("Terrible scam! Dump this crash immediately! 📉");
        assert!(result.score < -0.2);
        assert_eq!(result.label, "negative");
    }

    #[test]
    fn test_neutral_sentiment() {
        let engine = SentimentEngine::new();
        let result = engine.analyze_text("The price is stable today.");
        assert_eq!(result.label, "neutral");
    }

    #[test]
    fn test_negation_handling() {
        let engine = SentimentEngine::new();
        let positive = engine.analyze_text("This is great");
        let negated = engine.analyze_text("This is not great");
        assert!(positive.score > negated.score);
    }

    #[test]
    fn test_emoji_sentiment() {
        let engine = SentimentEngine::new();
        let result = engine.analyze_text("Let's go 🚀🚀🚀");
        assert!(result.score > 0.0);
    }

    #[test]
    fn test_amplifiers() {
        let engine = SentimentEngine::new();
        let normal = engine.analyze_text("This is good");
        let amplified = engine.analyze_text("This is very good");
        assert!(amplified.score > normal.score);
    }
}
