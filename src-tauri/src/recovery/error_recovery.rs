use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryAttempt {
    pub error_code: String,
    pub recovery_type: String,
    pub success: bool,
    pub attempts: u32,
    pub last_attempt: DateTime<Utc>,
    pub message: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryPlan {
    pub retry_strategy: Option<RetryStrategy>,
    pub fallback: Option<FallbackPlan>,
    pub state_recovery: Option<StateRecoveryPlan>,
    pub data_recovery: Option<DataRecoveryPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryStrategy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FallbackPlan {
    pub primary: String,
    pub fallback: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateRecoveryPlan {
    pub description: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRecoveryPlan {
    pub source: String,
    pub action: String,
    pub fallback: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ErrorRecoveryManager {
    attempts: Arc<RwLock<HashMap<String, RecoveryAttempt>>>,
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn record_attempt(
        &self,
        error_code: &str,
        recovery_type: &str,
        success: bool,
        message: Option<String>,
        metadata: Option<serde_json::Value>,
    ) {
        let key = format!("{}:{}", error_code, recovery_type);
        let mut attempts = self.attempts.write();

        let current_attempts = {
            attempts
                .get(&key)
                .map(|attempt| attempt.attempts + 1)
                .unwrap_or(1)
        };

        attempts.insert(
            key,
            RecoveryAttempt {
                error_code: error_code.to_string(),
                recovery_type: recovery_type.to_string(),
                success,
                attempts: current_attempts,
                last_attempt: Utc::now(),
                message,
                metadata,
            },
        );
    }

    pub fn get_attempt(&self, error_code: &str, recovery_type: &str) -> Option<RecoveryAttempt> {
        let key = format!("{}:{}", error_code, recovery_type);
        self.attempts.read().get(&key).cloned()
    }

    pub fn attempt_is_exhausted(&self, error_code: &str, strategy: &RetryStrategy) -> bool {
        let attempts = self.attempts.read();
        if let Some(attempt) = attempts.get(error_code) {
            attempt.attempts >= strategy.max_attempts
        } else {
            false
        }
    }

    pub async fn exponential_backoff(retry: &RetryStrategy, attempt: u32) {
        let backoff = retry.backoff_ms as f64 * retry.multiplier.powi(attempt as i32 - 1);
        tokio::time::sleep(std::time::Duration::from_millis(backoff as u64)).await;
    }
}
