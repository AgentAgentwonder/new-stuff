// Security Enhancement Types
// Hardware wallets, transaction simulation, and audit logging

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: String,
    pub event_type: String,
    pub user_wallet: String,
    pub severity: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub risk_score: f64,
    pub risk_factors: Vec<String>,
    pub balance_changes: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("security error: {0}")]
    General(String),
    
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub type SecurityResult<T> = Result<T, SecurityError>;
