use crate::logger::{LogLevel, SharedLogger};
use crate::recovery::{ErrorRecoveryManager, RecoveryPlan};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCategory {
    Compilation,
    Runtime,
    Network,
    Database,
    FileSystem,
    Memory,
    Logic,
    User,
    ExternalService,
    Unknown,
}

impl ErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::Compilation => "COMPILATION",
            ErrorCategory::Runtime => "RUNTIME",
            ErrorCategory::Network => "NETWORK",
            ErrorCategory::Database => "DATABASE",
            ErrorCategory::FileSystem => "FILE_SYSTEM",
            ErrorCategory::Memory => "MEMORY",
            ErrorCategory::Logic => "LOGIC",
            ErrorCategory::User => "USER",
            ErrorCategory::ExternalService => "EXTERNAL_SERVICE",
            ErrorCategory::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorStats {
    pub total_errors: usize,
    pub errors_by_category: HashMap<String, usize>,
    pub errors_by_code: HashMap<String, usize>,
    pub auto_fixed: usize,
    pub manually_fixed: usize,
    pub recovery_success_rate: f64,
}

pub type SharedRuntimeHandler = Arc<RuntimeHandler>;

#[derive(Clone)]
pub struct RuntimeHandler {
    logger: SharedLogger,
    recovery_manager: Arc<ErrorRecoveryManager>,
    error_counts: Arc<RwLock<HashMap<String, usize>>>,
    recovery_plans: Arc<RwLock<HashMap<String, RecoveryPlan>>>,
}

impl RuntimeHandler {
    pub fn new(logger: SharedLogger) -> Self {
        Self {
            logger,
            recovery_manager: Arc::new(ErrorRecoveryManager::new()),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            recovery_plans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn handle_error(
        &self,
        error_code: &str,
        category: ErrorCategory,
        message: &str,
        details: Option<serde_json::Value>,
    ) -> Result<String, String> {
        self.log_error(error_code, category.clone(), message, details.clone());

        self.classify_error(error_code, &category);

        if let Some(recovery_plan) = self.get_recovery_plan(error_code) {
            self.execute_recovery_plan(error_code, &recovery_plan)?;
            Ok("Recovery successful".to_string())
        } else {
            Err(format!("No recovery plan for error: {}", error_code))
        }
    }

    fn log_error(
        &self,
        error_code: &str,
        category: ErrorCategory,
        message: &str,
        details: Option<serde_json::Value>,
    ) {
        let mut counts = self.error_counts.write();
        *counts.entry(error_code.to_string()).or_insert(0) += 1;

        let error_details = serde_json::json!({
            "error_code": error_code,
            "category": category.as_str(),
            "count": counts.get(error_code).unwrap_or(&0),
            "details": details,
        });

        self.logger.log(
            LogLevel::Error,
            message,
            Some(&format!("{:?}", category)),
            Some(error_details),
            None,
        );
    }

    fn classify_error(&self, error_code: &str, category: &ErrorCategory) {
        match category {
            ErrorCategory::Network => {
                self.add_network_recovery_plan(error_code);
            }
            ErrorCategory::Database => {
                self.add_database_recovery_plan(error_code);
            }
            ErrorCategory::FileSystem => {
                self.add_filesystem_recovery_plan(error_code);
            }
            _ => {}
        }
    }

    fn add_network_recovery_plan(&self, error_code: &str) {
        let plan = RecoveryPlan {
            retry_strategy: Some(crate::recovery::RetryStrategy {
                max_attempts: 3,
                backoff_ms: 1000,
                multiplier: 2.0,
            }),
            fallback: Some(crate::recovery::FallbackPlan {
                primary: "Primary API".to_string(),
                fallback: "Fallback API".to_string(),
                description: Some("Switch to fallback API endpoint".to_string()),
            }),
            state_recovery: None,
            data_recovery: None,
        };
        self.recovery_plans
            .write()
            .insert(error_code.to_string(), plan);
    }

    fn add_database_recovery_plan(&self, error_code: &str) {
        let plan = RecoveryPlan {
            retry_strategy: Some(crate::recovery::RetryStrategy {
                max_attempts: 5,
                backoff_ms: 500,
                multiplier: 1.5,
            }),
            fallback: None,
            state_recovery: Some(crate::recovery::StateRecoveryPlan {
                description: "Reconnect to database".to_string(),
                actions: vec![
                    "Close connection".to_string(),
                    "Reopen connection".to_string(),
                ],
            }),
            data_recovery: None,
        };
        self.recovery_plans
            .write()
            .insert(error_code.to_string(), plan);
    }

    fn add_filesystem_recovery_plan(&self, error_code: &str) {
        let plan = RecoveryPlan {
            retry_strategy: Some(crate::recovery::RetryStrategy {
                max_attempts: 3,
                backoff_ms: 300,
                multiplier: 1.0,
            }),
            fallback: None,
            state_recovery: None,
            data_recovery: Some(crate::recovery::DataRecoveryPlan {
                source: "Backup".to_string(),
                action: "Restore from backup".to_string(),
                fallback: Some("Use default values".to_string()),
            }),
        };
        self.recovery_plans
            .write()
            .insert(error_code.to_string(), plan);
    }

    fn get_recovery_plan(&self, error_code: &str) -> Option<RecoveryPlan> {
        self.recovery_plans.read().get(error_code).cloned()
    }

    fn execute_recovery_plan(&self, error_code: &str, plan: &RecoveryPlan) -> Result<(), String> {
        self.logger.info(
            &format!("Executing recovery plan for error: {}", error_code),
            Some(serde_json::json!({ "plan": plan })),
        );

        if let Some(strategy) = &plan.retry_strategy {
            self.recovery_manager.record_attempt(
                error_code,
                "retry",
                true,
                Some("Retry strategy executed".to_string()),
                None,
            );
        }

        Ok(())
    }

    pub fn get_error_stats(&self) -> ErrorStats {
        let counts = self.error_counts.read();
        let total_errors: usize = counts.values().sum();

        let errors_by_code: HashMap<String, usize> = counts.clone();

        ErrorStats {
            total_errors,
            errors_by_category: HashMap::new(),
            errors_by_code,
            auto_fixed: 0,
            manually_fixed: 0,
            recovery_success_rate: 0.0,
        }
    }
}
