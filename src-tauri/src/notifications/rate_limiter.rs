use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use tokio::sync::Mutex;

use super::types::{ChatServiceType, NotificationError, RateLimitStatus};

const TELEGRAM_MAX_PER_MINUTE: i32 = 30; // Conservative limit per bot per minute
const SLACK_MAX_PER_MINUTE: i32 = 60; // Slack incoming webhooks allow ~1 msg/sec
const DISCORD_MAX_PER_MINUTE: i32 = 60; // Discord webhooks are rate limited server-side

#[derive(Debug, Clone)]
struct RateLimitEntry {
    service_type: ChatServiceType,
    config_id: String,
    count: i32,
    max_per_minute: i32,
    window_start: DateTime<Utc>,
}

impl RateLimitEntry {
    fn new(service_type: ChatServiceType, config_id: String, max_per_minute: i32) -> Self {
        Self {
            service_type,
            config_id,
            count: 0,
            max_per_minute,
            window_start: Utc::now(),
        }
    }

    fn reset_if_needed(&mut self, now: DateTime<Utc>) {
        if now - self.window_start >= ChronoDuration::seconds(60) {
            self.window_start = now;
            self.count = 0;
        }
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn is_rate_limited(&self) -> bool {
        self.count >= self.max_per_minute
    }

    fn status(&self) -> RateLimitStatus {
        RateLimitStatus {
            service_type: self.service_type.clone(),
            config_id: self.config_id.clone(),
            current_count: self.count,
            max_per_minute: self.max_per_minute,
            reset_at: (self.window_start + ChronoDuration::seconds(60)).to_rfc3339(),
        }
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    inner: Mutex<HashMap<String, RateLimitEntry>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    fn key(service_type: &ChatServiceType, config_id: &str) -> String {
        format!("{}:{}", service_type.as_str(), config_id)
    }

    fn max_per_minute(service_type: &ChatServiceType) -> i32 {
        match service_type {
            ChatServiceType::Telegram => TELEGRAM_MAX_PER_MINUTE,
            ChatServiceType::Slack => SLACK_MAX_PER_MINUTE,
            ChatServiceType::Discord => DISCORD_MAX_PER_MINUTE,
        }
    }

    pub async fn acquire(
        &self,
        service_type: &ChatServiceType,
        config_id: &str,
    ) -> Result<(), NotificationError> {
        let key = Self::key(service_type, config_id);
        let mut guard = self.inner.lock().await;
        let entry = guard.entry(key.clone()).or_insert_with(|| {
            RateLimitEntry::new(
                service_type.clone(),
                config_id.to_string(),
                Self::max_per_minute(service_type),
            )
        });

        let now = Utc::now();
        entry.reset_if_needed(now);

        if entry.is_rate_limited() {
            return Err(NotificationError::RateLimited(format!(
                "{}:{} hit rate limit ({} per minute)",
                service_type.as_str(),
                config_id,
                entry.max_per_minute
            )));
        }

        entry.increment();
        Ok(())
    }

    pub async fn register_failure(&self, service_type: &ChatServiceType, config_id: &str) {
        let key = Self::key(service_type, config_id);
        let mut guard = self.inner.lock().await;
        if let Some(entry) = guard.get_mut(&key) {
            if entry.count > 0 {
                entry.count -= 1; // roll back count when delivery fails
            }
        }
    }

    pub async fn get_statuses(&self) -> Vec<RateLimitStatus> {
        let guard = self.inner.lock().await;
        guard.values().map(|entry| entry.status()).collect()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
