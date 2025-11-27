use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use crate::social::models::{SentimentResult, SocialPost};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenSentiment {
    pub token: String,
    pub token_address: String,
    pub current_score: f32,
    pub label: String,
    pub confidence: f32,
    pub trend: Vec<SentimentDataPoint>,
    pub sample_posts: Vec<SocialPost>,
    pub total_mentions: i32,
    pub positive_count: i32,
    pub negative_count: i32,
    pub neutral_count: i32,
    pub last_updated: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SentimentDataPoint {
    pub timestamp: i64,
    pub score: f32,
    pub mentions: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SentimentAlert {
    pub id: String,
    pub token: String,
    pub token_address: String,
    pub alert_type: String,
    pub message: String,
    pub score: f32,
    pub threshold: f32,
    pub timestamp: i64,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SentimentAlertConfig {
    pub enabled: bool,
    pub positive_threshold: f32,
    pub negative_threshold: f32,
    pub spike_threshold: f32,
    pub notification_channels: Vec<String>,
}

pub type SharedSentimentManager = Arc<RwLock<SentimentManager>>;

pub struct SentimentManager {
    token_sentiments: HashMap<String, TokenSentiment>,
    alerts: Vec<SentimentAlert>,
    alert_config: SentimentAlertConfig,
}

impl SentimentManager {
    pub fn new() -> Self {
        Self {
            token_sentiments: HashMap::new(),
            alerts: Vec::new(),
            alert_config: SentimentAlertConfig {
                enabled: true,
                positive_threshold: 0.7,
                negative_threshold: -0.7,
                spike_threshold: 0.5,
                notification_channels: vec!["in-app".to_string()],
            },
        }
    }

    pub fn add_sentiment_data(&mut self, token_address: String, posts: Vec<SocialPost>) {
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut neutral_count = 0;
        let mut total_score = 0.0;

        for post in &posts {
            match post.sentiment.label.as_str() {
                "positive" => positive_count += 1,
                "negative" => negative_count += 1,
                _ => neutral_count += 1,
            }
            total_score += post.sentiment.score;
        }

        let total = posts.len() as f32;
        let avg_score = if total > 0.0 {
            total_score / total
        } else {
            0.0
        };

        let label = if avg_score > 0.2 {
            "positive".to_string()
        } else if avg_score < -0.2 {
            "negative".to_string()
        } else {
            "neutral".to_string()
        };

        let confidence = avg_score.abs().min(1.0);

        // Get or create token sentiment
        let token_sentiment = self
            .token_sentiments
            .entry(token_address.clone())
            .or_insert_with(|| TokenSentiment {
                token: token_address.clone(),
                token_address: token_address.clone(),
                current_score: 0.0,
                label: "neutral".to_string(),
                confidence: 0.0,
                trend: Vec::new(),
                sample_posts: Vec::new(),
                total_mentions: 0,
                positive_count: 0,
                negative_count: 0,
                neutral_count: 0,
                last_updated: 0,
            });

        let now = Utc::now().timestamp();

        // Update trend data
        token_sentiment.trend.push(SentimentDataPoint {
            timestamp: now,
            score: avg_score,
            mentions: posts.len() as i32,
        });

        // Keep only last 100 data points
        if token_sentiment.trend.len() > 100 {
            token_sentiment
                .trend
                .drain(0..token_sentiment.trend.len() - 100);
        }

        // Update current sentiment
        token_sentiment.current_score = avg_score;
        token_sentiment.label = label.clone();
        token_sentiment.confidence = confidence;
        token_sentiment.total_mentions += posts.len() as i32;
        token_sentiment.positive_count += positive_count;
        token_sentiment.negative_count += negative_count;
        token_sentiment.neutral_count += neutral_count;
        token_sentiment.last_updated = now;

        // Keep sample posts (up to 10 most recent)
        let mut all_posts = posts.clone();
        all_posts.extend(token_sentiment.sample_posts.clone());
        all_posts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        token_sentiment.sample_posts = all_posts.into_iter().take(10).collect();

        // Check for alerts
        self.check_sentiment_alerts(&token_address, avg_score, &label);
    }

    fn check_sentiment_alerts(&mut self, token_address: &str, score: f32, label: &str) {
        if !self.alert_config.enabled {
            return;
        }

        let now = Utc::now().timestamp();
        let mut should_alert = false;
        let mut alert_type = String::new();
        let mut message = String::new();
        let mut threshold = 0.0;

        // Check positive threshold
        if score >= self.alert_config.positive_threshold {
            should_alert = true;
            alert_type = "sentiment_positive_spike".to_string();
            message = format!(
                "Positive sentiment spike detected for token {}. Score: {:.2}",
                token_address, score
            );
            threshold = self.alert_config.positive_threshold;
        }
        // Check negative threshold
        else if score <= self.alert_config.negative_threshold {
            should_alert = true;
            alert_type = "sentiment_negative_spike".to_string();
            message = format!(
                "Negative sentiment spike detected for token {}. Score: {:.2}",
                token_address, score
            );
            threshold = self.alert_config.negative_threshold;
        }

        // Check for sudden change (spike detection)
        if let Some(token_sentiment) = self.token_sentiments.get(token_address) {
            if token_sentiment.trend.len() >= 2 {
                let prev_score = token_sentiment.trend[token_sentiment.trend.len() - 2].score;
                let change = (score - prev_score).abs();
                if change >= self.alert_config.spike_threshold {
                    should_alert = true;
                    alert_type = "sentiment_spike".to_string();
                    message = format!(
                        "Sudden sentiment change detected for token {}. Change: {:.2}",
                        token_address, change
                    );
                    threshold = self.alert_config.spike_threshold;
                }
            }
        }

        if should_alert {
            let alert = SentimentAlert {
                id: uuid::Uuid::new_v4().to_string(),
                token: token_address.to_string(),
                token_address: token_address.to_string(),
                alert_type,
                message,
                score,
                threshold,
                timestamp: now,
                is_active: true,
            };
            self.alerts.push(alert);

            // Keep only last 100 alerts
            if self.alerts.len() > 100 {
                self.alerts.drain(0..self.alerts.len() - 100);
            }
        }
    }

    pub fn get_token_sentiment(&self, token_address: &str) -> Option<TokenSentiment> {
        self.token_sentiments.get(token_address).cloned()
    }

    pub fn get_all_sentiments(&self) -> Vec<TokenSentiment> {
        self.token_sentiments.values().cloned().collect()
    }

    pub fn get_alerts(&self, token_address: Option<&str>) -> Vec<SentimentAlert> {
        match token_address {
            Some(addr) => self
                .alerts
                .iter()
                .filter(|a| a.token_address == addr)
                .cloned()
                .collect(),
            None => self.alerts.clone(),
        }
    }

    pub fn update_alert_config(&mut self, config: SentimentAlertConfig) {
        self.alert_config = config;
    }

    pub fn get_alert_config(&self) -> SentimentAlertConfig {
        self.alert_config.clone()
    }

    pub fn dismiss_alert(&mut self, alert_id: &str) {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.is_active = false;
        }
    }
}

// Simple sentiment analysis function (can be replaced with more sophisticated NLP)
pub fn analyze_sentiment(text: &str) -> SentimentResult {
    let positive_words = [
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
    ];
    let negative_words = [
        "bad", "terrible", "bearish", "dump", "crash", "loss", "scam", "rug", "fail", "dead",
        "sell", "down", "drop", "fall", "weak", "negative", "decline", "worst", "hate", "low",
        "collapse", "panic",
    ];

    let text_lower = text.to_lowercase();
    let mut score: f32 = 0.0;

    for word in positive_words.iter() {
        if text_lower.contains(word) {
            score += 0.1;
        }
    }
    for word in negative_words.iter() {
        if text_lower.contains(word) {
            score -= 0.1;
        }
    }

    // Normalize score to [-1, 1]
    let normalized_score = score.max(-1.0).min(1.0);

    SentimentResult {
        score: normalized_score,
        label: if normalized_score > 0.2 {
            "positive".to_string()
        } else if normalized_score < -0.2 {
            "negative".to_string()
        } else {
            "neutral".to_string()
        },
        confidence: normalized_score.abs().min(1.0),
    }
}

// Tauri commands
#[tauri::command]
pub async fn analyze_text_sentiment(text: String) -> Result<SentimentResult, String> {
    Ok(analyze_sentiment(&text))
}

#[tauri::command]
pub async fn get_token_sentiment(
    token_address: String,
    manager: tauri::State<'_, SharedSentimentManager>,
) -> Result<Option<TokenSentiment>, String> {
    let mgr = manager.read().await;
    Ok(mgr.get_token_sentiment(&token_address))
}

#[tauri::command]
pub async fn get_all_token_sentiments(
    manager: tauri::State<'_, SharedSentimentManager>,
) -> Result<Vec<TokenSentiment>, String> {
    let mgr = manager.read().await;
    Ok(mgr.get_all_sentiments())
}

#[tauri::command]
pub async fn ingest_social_data(
    token_address: String,
    posts: Vec<SocialPost>,
    manager: tauri::State<'_, SharedSentimentManager>,
) -> Result<(), String> {
    let mut mgr = manager.write().await;
    mgr.add_sentiment_data(token_address, posts);
    Ok(())
}

#[tauri::command]
pub async fn get_sentiment_alerts(
    token_address: Option<String>,
    manager: tauri::State<'_, SharedSentimentManager>,
) -> Result<Vec<SentimentAlert>, String> {
    let mgr = manager.read().await;
    Ok(mgr.get_alerts(token_address.as_deref()))
}

#[tauri::command]
pub async fn update_sentiment_alert_config(
    config: SentimentAlertConfig,
    manager: tauri::State<'_, SharedSentimentManager>,
) -> Result<(), String> {
    let mut mgr = manager.write().await;
    mgr.update_alert_config(config);
    Ok(())
}

#[tauri::command]
pub async fn get_sentiment_alert_config(
    manager: tauri::State<'_, SharedSentimentManager>,
) -> Result<SentimentAlertConfig, String> {
    let mgr = manager.read().await;
    Ok(mgr.get_alert_config())
}

#[tauri::command]
pub async fn dismiss_sentiment_alert(
    alert_id: String,
    manager: tauri::State<'_, SharedSentimentManager>,
) -> Result<(), String> {
    let mut mgr = manager.write().await;
    mgr.dismiss_alert(&alert_id);
    Ok(())
}

// Mock function to simulate social media data ingestion
#[tauri::command]
pub async fn fetch_social_mentions(token_address: String) -> Result<Vec<SocialPost>, String> {
    // In a real implementation, this would call Twitter/Reddit APIs
    // For now, return mock data
    let now = Utc::now().timestamp();

    let sample_texts = vec![
        "This token is amazing! Great project with strong fundamentals.",
        "Love the recent updates. Bullish on this one!",
        "Not sure about this. Price action looks weak.",
        "Terrible performance lately. Might be time to sell.",
        "Just bought some more. Going to the moon! 🚀",
        "This is a scam. Don't invest!",
        "Good entry point here. Strong support levels.",
        "The team delivered on their roadmap. Very impressed.",
    ];

    let posts: Vec<SocialPost> = sample_texts
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let sentiment = analyze_sentiment(text);
            SocialPost {
                id: format!("post_{}", i),
                text: text.to_string(),
                source: if i % 2 == 0 { "twitter" } else { "reddit" }.to_string(),
                author: format!("user_{}", i),
                timestamp: now - (i as i64 * 3600),
                sentiment,
                engagement: (100 + i * 50) as i32,
            }
        })
        .collect();

    Ok(posts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_sentiment() {
        let result = analyze_sentiment("This coin is great! Bullish and going to the moon!");
        assert!(result.score > 0.0);
        assert_eq!(result.label, "positive");
    }

    #[test]
    fn test_negative_sentiment() {
        let result = analyze_sentiment("This is a terrible scam. Avoid at all costs!");
        assert!(result.score < 0.0);
        assert_eq!(result.label, "negative");
    }

    #[test]
    fn test_neutral_sentiment() {
        let result = analyze_sentiment("The price is stable today.");
        assert_eq!(result.label, "neutral");
    }

    #[test]
    fn test_sentiment_manager_add_data() {
        let mut manager = SentimentManager::new();
        let token_address = "test_token".to_string();

        let posts = vec![SocialPost {
            id: "1".to_string(),
            text: "Great project!".to_string(),
            source: "twitter".to_string(),
            author: "user1".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: analyze_sentiment("Great project!"),
            engagement: 100,
        }];

        manager.add_sentiment_data(token_address.clone(), posts);

        let sentiment = manager.get_token_sentiment(&token_address);
        assert!(sentiment.is_some());
        let sentiment = sentiment.unwrap();
        assert!(sentiment.current_score > 0.0);
        assert_eq!(sentiment.label, "positive");
    }

    #[test]
    fn test_sentiment_alert_generation() {
        let mut manager = SentimentManager::new();
        manager.alert_config.positive_threshold = 0.5;

        let token_address = "test_token".to_string();
        let posts = vec![SocialPost {
            id: "1".to_string(),
            text: "Amazing! Great! Excellent! Bullish! Moon! Rocket!".to_string(),
            source: "twitter".to_string(),
            author: "user1".to_string(),
            timestamp: Utc::now().timestamp(),
            sentiment: analyze_sentiment("Amazing! Great! Excellent! Bullish! Moon! Rocket!"),
            engagement: 100,
        }];

        manager.add_sentiment_data(token_address.clone(), posts);

        let alerts = manager.get_alerts(Some(&token_address));
        assert!(!alerts.is_empty());
    }
}
