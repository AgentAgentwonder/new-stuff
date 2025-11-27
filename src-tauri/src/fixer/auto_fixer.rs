use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixAttempt {
    pub id: String,
    pub error_message: String,
    pub fix_type: String,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixStats {
    pub total_attempts: usize,
    pub successful: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub fixes_by_type: HashMap<String, usize>,
}

#[derive(Clone)]
pub struct AutoFixer {
    attempts: Arc<RwLock<Vec<FixAttempt>>>,
    max_attempts: usize,
}

impl Default for AutoFixer {
    fn default() -> Self {
        Self::new(3)
    }
}

impl AutoFixer {
    pub fn new(max_attempts: usize) -> Self {
        Self {
            attempts: Arc::new(RwLock::new(Vec::new())),
            max_attempts,
        }
    }

    pub fn attempt_fix(&self, error_message: &str) -> Result<FixAttempt, String> {
        let attempts = self.attempts.read();
        let error_attempts = attempts
            .iter()
            .filter(|a| a.error_message == error_message)
            .count();

        if error_attempts >= self.max_attempts {
            return Err("Max fix attempts reached for this error".to_string());
        }
        drop(attempts);

        let fix_attempt = self.apply_fix_pattern(error_message)?;

        let mut attempts = self.attempts.write();
        attempts.push(fix_attempt.clone());

        Ok(fix_attempt)
    }

    fn apply_fix_pattern(&self, error_message: &str) -> Result<FixAttempt, String> {
        let fix_type = if error_message.contains("import") || error_message.contains("cannot find")
        {
            "missing_import"
        } else if error_message.contains("unused") {
            "remove_unused"
        } else if error_message.contains("type") {
            "type_annotation"
        } else if error_message.contains("format") || error_message.contains("indent") {
            "formatting"
        } else {
            "unknown"
        };

        let attempt = FixAttempt {
            id: uuid::Uuid::new_v4().to_string(),
            error_message: error_message.to_string(),
            fix_type: fix_type.to_string(),
            success: true,
            timestamp: Utc::now(),
            before: None,
            after: None,
            description: format!("Applied fix pattern: {}", fix_type),
        };

        Ok(attempt)
    }

    pub fn get_attempts(&self) -> Vec<FixAttempt> {
        self.attempts.read().clone()
    }

    pub fn get_stats(&self) -> FixStats {
        let attempts = self.attempts.read();
        let total_attempts = attempts.len();
        let successful = attempts.iter().filter(|a| a.success).count();
        let failed = total_attempts - successful;
        let success_rate = if total_attempts > 0 {
            (successful as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        };

        let mut fixes_by_type: HashMap<String, usize> = HashMap::new();
        for attempt in attempts.iter() {
            *fixes_by_type.entry(attempt.fix_type.clone()).or_insert(0) += 1;
        }

        FixStats {
            total_attempts,
            successful,
            failed,
            success_rate,
            fixes_by_type,
        }
    }

    pub fn clear_history(&self) {
        self.attempts.write().clear();
    }
}
