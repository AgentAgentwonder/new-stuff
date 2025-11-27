use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

const REPUTATION_DB_FILE: &str = "reputation.db";

// Shared type for the reputation engine state
pub type SharedReputationEngine = Arc<RwLock<ReputationEngine>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReputationLevel {
    Excellent,
    Good,
    Neutral,
    Poor,
    Malicious,
}

impl ReputationLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 80.0 => ReputationLevel::Excellent,
            s if s >= 60.0 => ReputationLevel::Good,
            s if s >= 40.0 => ReputationLevel::Neutral,
            s if s >= 20.0 => ReputationLevel::Poor,
            _ => ReputationLevel::Malicious,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletReputation {
    pub address: String,
    pub trust_score: f64,
    pub reputation_level: ReputationLevel,
    pub vouches_received: i64,
    pub vouches_given: i64,
    pub is_blacklisted: bool,
    pub blacklist_reason: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub transaction_count: i64,
    pub total_volume: f64,
    pub age_days: i64,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenReputation {
    pub address: String,
    pub trust_score: f64,
    pub reputation_level: ReputationLevel,
    pub creator_address: String,
    pub creator_trust_score: f64,
    pub vouches_received: i64,
    pub is_blacklisted: bool,
    pub blacklist_reason: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub holder_count: i64,
    pub liquidity_score: f64,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VouchRecord {
    pub id: i64,
    pub voucher_address: String,
    pub target_address: String,
    pub target_type: String, // "wallet" or "token"
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistEntry {
    pub id: i64,
    pub address: String,
    pub entry_type: String, // "wallet" or "token"
    pub reason: String,
    pub reporter: Option<String>,
    pub source: String, // "community" or "automated" or "admin"
    pub timestamp: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReputationHistory {
    pub address: String,
    pub timestamp: DateTime<Utc>,
    pub trust_score: f64,
    pub event_type: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReputationReport {
    pub reporter_address: String,
    pub target_address: String,
    pub target_type: String,
    pub report_type: String, // "scam", "rugpull", "suspicious", "other"
    pub description: String,
    pub evidence: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReputationStats {
    pub total_wallets_tracked: i64,
    pub total_tokens_tracked: i64,
    pub total_vouches: i64,
    pub total_blacklisted: i64,
    pub recent_reports: i64,
    pub average_trust_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ReputationSettings {
    pub enabled: bool,
    pub auto_blacklist_threshold: f64,
    pub min_vouch_weight: f64,
    pub show_warnings: bool,
    pub share_data: bool, // Privacy setting for sharing reputation data
}

#[derive(Debug, thiserror::Error)]
pub enum ReputationError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}

pub struct ReputationEngine {
    pool: Pool<Sqlite>,
    settings: ReputationSettings,
}

impl ReputationEngine {
    pub async fn new(app_handle: &AppHandle) -> Result<Self, ReputationError> {
        let app_dir = app_handle.path().app_data_dir().map_err(|err| {
            ReputationError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Unable to resolve app data directory",
            ))
        })?;
        
        Self::new_with_path(&app_dir).await
    }
    
    // Helper method for testing that accepts a path directly
    pub async fn new_with_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ReputationError> {
        let app_dir = path.as_ref();
        std::fs::create_dir_all(app_dir)?;
        let db_path = app_dir.join(REPUTATION_DB_FILE);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        eprintln!("ReputationEngine attempting to create database at: {:?}", db_path);

        let pool = match SqlitePool::connect(&format!("sqlite:{}", db_path.display())).await {
            Ok(pool) => {
                eprintln!("ReputationEngine successfully connected to file database");
                pool
            }
            Err(e) => {
                eprintln!("Failed to open reputation database at {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database");
                
                // Create in-memory fallback pool
                let memory_pool = SqlitePool::connect("sqlite::memory:").await.map_err(|e| {
                    ReputationError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create in-memory database: {}", e),
                    ))
                })?;
                
                eprintln!("ReputationEngine successfully created in-memory database");
                memory_pool
            }
        };

        // Initialize database schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_reputation (
                address TEXT PRIMARY KEY NOT NULL,
                trust_score REAL NOT NULL DEFAULT 50.0,
                vouches_received INTEGER NOT NULL DEFAULT 0,
                vouches_given INTEGER NOT NULL DEFAULT 0,
                is_blacklisted INTEGER NOT NULL DEFAULT 0,
                blacklist_reason TEXT,
                first_seen TEXT NOT NULL,
                last_updated TEXT NOT NULL,
                transaction_count INTEGER NOT NULL DEFAULT 0,
                total_volume REAL NOT NULL DEFAULT 0.0,
                age_days INTEGER NOT NULL DEFAULT 0,
                risk_flags TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS token_reputation (
                address TEXT PRIMARY KEY NOT NULL,
                trust_score REAL NOT NULL DEFAULT 50.0,
                creator_address TEXT NOT NULL,
                creator_trust_score REAL NOT NULL DEFAULT 50.0,
                vouches_received INTEGER NOT NULL DEFAULT 0,
                is_blacklisted INTEGER NOT NULL DEFAULT 0,
                blacklist_reason TEXT,
                first_seen TEXT NOT NULL,
                last_updated TEXT NOT NULL,
                holder_count INTEGER NOT NULL DEFAULT 0,
                liquidity_score REAL NOT NULL DEFAULT 0.0,
                risk_flags TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vouches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                voucher_address TEXT NOT NULL,
                target_address TEXT NOT NULL,
                target_type TEXT NOT NULL,
                comment TEXT,
                timestamp TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                UNIQUE(voucher_address, target_address, target_type)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS blacklist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                address TEXT NOT NULL,
                entry_type TEXT NOT NULL,
                reason TEXT NOT NULL,
                reporter TEXT,
                source TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reputation_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                address TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                trust_score REAL NOT NULL,
                event_type TEXT NOT NULL,
                details TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reputation_reports (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                reporter_address TEXT NOT NULL,
                target_address TEXT NOT NULL,
                target_type TEXT NOT NULL,
                report_type TEXT NOT NULL,
                description TEXT NOT NULL,
                evidence TEXT,
                timestamp TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create indices for better query performance
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_wallet_trust ON wallet_reputation(trust_score)",
        )
        .execute(&pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_token_trust ON token_reputation(trust_score)")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_vouches_target ON vouches(target_address)")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_blacklist_address ON blacklist(address)")
            .execute(&pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_history_address ON reputation_history(address)",
        )
        .execute(&pool)
        .await?;

        let settings = ReputationSettings {
            enabled: true,
            auto_blacklist_threshold: 10.0,
            min_vouch_weight: 50.0,
            show_warnings: true,
            share_data: false,
        };

        Ok(Self { pool, settings })
    }

    // Wallet reputation methods
    pub async fn get_wallet_reputation(
        &self,
        address: &str,
    ) -> Result<WalletReputation, ReputationError> {
        let row = sqlx::query(
            r#"
            SELECT address, trust_score, vouches_received, vouches_given, is_blacklisted,
                   blacklist_reason, first_seen, last_updated, transaction_count, total_volume,
                   age_days, risk_flags
            FROM wallet_reputation
            WHERE address = ?
            "#,
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let trust_score: f64 = row.get("trust_score");
                let risk_flags_str: Option<String> = row.get("risk_flags");
                let risk_flags: Vec<String> = risk_flags_str
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default();

                Ok(WalletReputation {
                    address: row.get("address"),
                    trust_score,
                    reputation_level: ReputationLevel::from_score(trust_score),
                    vouches_received: row.get("vouches_received"),
                    vouches_given: row.get("vouches_given"),
                    is_blacklisted: row.get::<i64, _>("is_blacklisted") != 0,
                    blacklist_reason: row.get("blacklist_reason"),
                    first_seen: DateTime::parse_from_rfc3339(&row.get::<String, _>("first_seen"))
                        .unwrap()
                        .with_timezone(&Utc),
                    last_updated: DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("last_updated"),
                    )
                    .unwrap()
                    .with_timezone(&Utc),
                    transaction_count: row.get("transaction_count"),
                    total_volume: row.get("total_volume"),
                    age_days: row.get("age_days"),
                    risk_flags,
                })
            }
            None => {
                // Initialize new wallet reputation
                self.initialize_wallet_reputation(address).await?;
                // Return newly initialized reputation
                Ok(WalletReputation {
                    address: address.to_string(),
                    trust_score: 50.0, // Default neutral score
                    reputation_level: ReputationLevel::Neutral,
                    vouches_received: 0,
                    vouches_given: 0,
                    is_blacklisted: false,
                    blacklist_reason: None,
                    first_seen: Utc::now(),
                    last_updated: Utc::now(),
                    transaction_count: 0,
                    total_volume: 0.0,
                    age_days: 0,
                    risk_flags: vec![],
                })
            }
        }
    }

    async fn initialize_wallet_reputation(&self, address: &str) -> Result<(), ReputationError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO wallet_reputation (address, first_seen, last_updated)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(address)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.record_history(address, 50.0, "initialized", None)
            .await?;
        Ok(())
    }

    pub async fn update_wallet_behavior(
        &self,
        address: &str,
        transaction_count: Option<i64>,
        total_volume: Option<f64>,
        age_days: Option<i64>,
    ) -> Result<(), ReputationError> {
        let current = self.get_wallet_reputation(address).await?;

        let new_tx_count = transaction_count.unwrap_or(current.transaction_count);
        let new_volume = total_volume.unwrap_or(current.total_volume);
        let new_age = age_days.unwrap_or(current.age_days);

        // Calculate new trust score based on behavior
        let trust_score = self.calculate_wallet_trust_score(
            new_tx_count,
            new_volume,
            new_age,
            current.vouches_received,
        );

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE wallet_reputation
            SET transaction_count = ?, total_volume = ?, age_days = ?,
                trust_score = ?, last_updated = ?
            WHERE address = ?
            "#,
        )
        .bind(new_tx_count)
        .bind(new_volume)
        .bind(new_age)
        .bind(trust_score)
        .bind(&now)
        .bind(address)
        .execute(&self.pool)
        .await?;

        self.record_history(address, trust_score, "behavior_update", None)
            .await?;
        Ok(())
    }

    fn calculate_wallet_trust_score(
        &self,
        transaction_count: i64,
        total_volume: f64,
        age_days: i64,
        vouches: i64,
    ) -> f64 {
        let mut score = 50.0; // Base score

        // Age factor (max +20 points)
        score += (age_days as f64 / 365.0).min(1.0) * 20.0;

        // Transaction count factor (max +15 points)
        score += (transaction_count as f64 / 1000.0).min(1.0) * 15.0;

        // Volume factor (max +10 points)
        score += (total_volume / 1_000_000.0).min(1.0) * 10.0;

        // Vouch factor (max +5 points per vouch, up to +25)
        score += (vouches as f64 * 5.0).min(25.0);

        score.min(100.0).max(0.0)
    }

    // Token reputation methods
    pub async fn get_token_reputation(
        &self,
        address: &str,
    ) -> Result<TokenReputation, ReputationError> {
        let row = sqlx::query(
            r#"
            SELECT address, trust_score, creator_address, creator_trust_score, vouches_received,
                   is_blacklisted, blacklist_reason, first_seen, last_updated, holder_count,
                   liquidity_score, risk_flags
            FROM token_reputation
            WHERE address = ?
            "#,
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let trust_score: f64 = row.get("trust_score");
                let risk_flags_str: Option<String> = row.get("risk_flags");
                let risk_flags: Vec<String> = risk_flags_str
                    .map(|s| serde_json::from_str(&s).unwrap_or_default())
                    .unwrap_or_default();

                Ok(TokenReputation {
                    address: row.get("address"),
                    trust_score,
                    reputation_level: ReputationLevel::from_score(trust_score),
                    creator_address: row.get("creator_address"),
                    creator_trust_score: row.get("creator_trust_score"),
                    vouches_received: row.get("vouches_received"),
                    is_blacklisted: row.get::<i64, _>("is_blacklisted") != 0,
                    blacklist_reason: row.get("blacklist_reason"),
                    first_seen: DateTime::parse_from_rfc3339(&row.get::<String, _>("first_seen"))
                        .unwrap()
                        .with_timezone(&Utc),
                    last_updated: DateTime::parse_from_rfc3339(
                        &row.get::<String, _>("last_updated"),
                    )
                    .unwrap()
                    .with_timezone(&Utc),
                    holder_count: row.get("holder_count"),
                    liquidity_score: row.get("liquidity_score"),
                    risk_flags,
                })
            }
            None => Err(ReputationError::NotFound(format!(
                "Token not found: {}",
                address
            ))),
        }
    }

    pub async fn initialize_token_reputation(
        &self,
        address: &str,
        creator_address: &str,
    ) -> Result<(), ReputationError> {
        let creator_rep = self.get_wallet_reputation(creator_address).await?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO token_reputation (address, creator_address, creator_trust_score, first_seen, last_updated)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(address)
        .bind(creator_address)
        .bind(creator_rep.trust_score)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.record_history(address, 50.0, "token_created", None)
            .await?;
        Ok(())
    }

    pub async fn update_token_metrics(
        &self,
        address: &str,
        holder_count: Option<i64>,
        liquidity_score: Option<f64>,
    ) -> Result<(), ReputationError> {
        let current = self.get_token_reputation(address).await?;

        let new_holders = holder_count.unwrap_or(current.holder_count);
        let new_liquidity = liquidity_score.unwrap_or(current.liquidity_score);

        // Calculate new trust score
        let trust_score = self.calculate_token_trust_score(
            new_holders,
            new_liquidity,
            current.creator_trust_score,
            current.vouches_received,
        );

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE token_reputation
            SET holder_count = ?, liquidity_score = ?, trust_score = ?, last_updated = ?
            WHERE address = ?
            "#,
        )
        .bind(new_holders)
        .bind(new_liquidity)
        .bind(trust_score)
        .bind(&now)
        .bind(address)
        .execute(&self.pool)
        .await?;

        self.record_history(address, trust_score, "metrics_update", None)
            .await?;
        Ok(())
    }

    fn calculate_token_trust_score(
        &self,
        holder_count: i64,
        liquidity_score: f64,
        creator_trust: f64,
        vouches: i64,
    ) -> f64 {
        let mut score = 50.0; // Base score

        // Creator trust factor (max +25 points)
        score += (creator_trust / 100.0) * 25.0;

        // Holder count factor (max +20 points)
        score += (holder_count as f64 / 10000.0).min(1.0) * 20.0;

        // Liquidity factor (max +15 points)
        score += liquidity_score.min(1.0) * 15.0;

        // Vouch factor (max +5 points per vouch, up to +20)
        score += (vouches as f64 * 5.0).min(20.0);

        score.min(100.0).max(0.0)
    }

    // Vouching system
    pub async fn add_vouch(
        &self,
        voucher_address: &str,
        target_address: &str,
        target_type: &str,
        comment: Option<String>,
    ) -> Result<(), ReputationError> {
        // Check if voucher has sufficient reputation
        let voucher_rep = self.get_wallet_reputation(voucher_address).await?;
        if voucher_rep.trust_score < self.settings.min_vouch_weight {
            return Err(ReputationError::Unauthorized(
                "Insufficient reputation to vouch".to_string(),
            ));
        }

        let now = Utc::now().to_rfc3339();

        // Insert or update vouch
        sqlx::query(
            r#"
            INSERT INTO vouches (voucher_address, target_address, target_type, comment, timestamp)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(voucher_address, target_address, target_type)
            DO UPDATE SET comment = excluded.comment, timestamp = excluded.timestamp, is_active = 1
            "#,
        )
        .bind(voucher_address)
        .bind(target_address)
        .bind(target_type)
        .bind(&comment)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        // Update vouch counts
        if target_type == "wallet" {
            sqlx::query(
                "UPDATE wallet_reputation SET vouches_received = vouches_received + 1 WHERE address = ?"
            )
            .bind(target_address)
            .execute(&self.pool)
            .await?;
        } else if target_type == "token" {
            sqlx::query(
                "UPDATE token_reputation SET vouches_received = vouches_received + 1 WHERE address = ?"
            )
            .bind(target_address)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query(
            "UPDATE wallet_reputation SET vouches_given = vouches_given + 1 WHERE address = ?",
        )
        .bind(voucher_address)
        .execute(&self.pool)
        .await?;

        self.record_history(
            target_address,
            0.0,
            "vouch_received",
            Some(&format!("from: {}", voucher_address)),
        )
        .await?;
        Ok(())
    }

    pub async fn remove_vouch(
        &self,
        voucher_address: &str,
        target_address: &str,
        target_type: &str,
    ) -> Result<(), ReputationError> {
        sqlx::query(
            r#"
            UPDATE vouches
            SET is_active = 0
            WHERE voucher_address = ? AND target_address = ? AND target_type = ?
            "#,
        )
        .bind(voucher_address)
        .bind(target_address)
        .bind(target_type)
        .execute(&self.pool)
        .await?;

        // Update vouch counts
        if target_type == "wallet" {
            sqlx::query(
                "UPDATE wallet_reputation SET vouches_received = MAX(0, vouches_received - 1) WHERE address = ?"
            )
            .bind(target_address)
            .execute(&self.pool)
            .await?;
        } else if target_type == "token" {
            sqlx::query(
                "UPDATE token_reputation SET vouches_received = MAX(0, vouches_received - 1) WHERE address = ?"
            )
            .bind(target_address)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_vouches(
        &self,
        target_address: &str,
    ) -> Result<Vec<VouchRecord>, ReputationError> {
        let rows = sqlx::query(
            r#"
            SELECT id, voucher_address, target_address, target_type, comment, timestamp, is_active
            FROM vouches
            WHERE target_address = ? AND is_active = 1
            ORDER BY timestamp DESC
            "#,
        )
        .bind(target_address)
        .fetch_all(&self.pool)
        .await?;

        let vouches = rows
            .iter()
            .map(|row| VouchRecord {
                id: row.get("id"),
                voucher_address: row.get("voucher_address"),
                target_address: row.get("target_address"),
                target_type: row.get("target_type"),
                comment: row.get("comment"),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))
                    .unwrap()
                    .with_timezone(&Utc),
                is_active: row.get::<i64, _>("is_active") != 0,
            })
            .collect();

        Ok(vouches)
    }

    // Blacklist management
    pub async fn add_to_blacklist(
        &self,
        address: &str,
        entry_type: &str,
        reason: &str,
        reporter: Option<String>,
        source: &str,
    ) -> Result<(), ReputationError> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO blacklist (address, entry_type, reason, reporter, source, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(address)
        .bind(entry_type)
        .bind(reason)
        .bind(&reporter)
        .bind(source)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        // Update blacklist flag in reputation tables
        if entry_type == "wallet" {
            sqlx::query(
                "UPDATE wallet_reputation SET is_blacklisted = 1, blacklist_reason = ?, trust_score = 0 WHERE address = ?"
            )
            .bind(reason)
            .bind(address)
            .execute(&self.pool)
            .await?;
        } else if entry_type == "token" {
            sqlx::query(
                "UPDATE token_reputation SET is_blacklisted = 1, blacklist_reason = ?, trust_score = 0 WHERE address = ?"
            )
            .bind(reason)
            .bind(address)
            .execute(&self.pool)
            .await?;
        }

        self.record_history(address, 0.0, "blacklisted", Some(reason))
            .await?;
        Ok(())
    }

    pub async fn remove_from_blacklist(
        &self,
        address: &str,
        entry_type: &str,
    ) -> Result<(), ReputationError> {
        sqlx::query(
            r#"
            UPDATE blacklist
            SET is_active = 0
            WHERE address = ? AND entry_type = ?
            "#,
        )
        .bind(address)
        .bind(entry_type)
        .execute(&self.pool)
        .await?;

        // Update blacklist flag in reputation tables
        if entry_type == "wallet" {
            sqlx::query(
                "UPDATE wallet_reputation SET is_blacklisted = 0, blacklist_reason = NULL WHERE address = ?"
            )
            .bind(address)
            .execute(&self.pool)
            .await?;
        } else if entry_type == "token" {
            sqlx::query(
                "UPDATE token_reputation SET is_blacklisted = 0, blacklist_reason = NULL WHERE address = ?"
            )
            .bind(address)
            .execute(&self.pool)
            .await?;
        }

        self.record_history(address, 0.0, "removed_from_blacklist", None)
            .await?;
        Ok(())
    }

    pub async fn get_blacklist(
        &self,
        entry_type: Option<String>,
    ) -> Result<Vec<BlacklistEntry>, ReputationError> {
        let query = if let Some(t) = entry_type {
            sqlx::query(
                r#"
                SELECT id, address, entry_type, reason, reporter, source, timestamp, is_active
                FROM blacklist
                WHERE entry_type = ? AND is_active = 1
                ORDER BY timestamp DESC
                "#,
            )
            .bind(t)
        } else {
            sqlx::query(
                r#"
                SELECT id, address, entry_type, reason, reporter, source, timestamp, is_active
                FROM blacklist
                WHERE is_active = 1
                ORDER BY timestamp DESC
                "#,
            )
        };

        let rows = query.fetch_all(&self.pool).await?;

        let entries = rows
            .iter()
            .map(|row| BlacklistEntry {
                id: row.get("id"),
                address: row.get("address"),
                entry_type: row.get("entry_type"),
                reason: row.get("reason"),
                reporter: row.get("reporter"),
                source: row.get("source"),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))
                    .unwrap()
                    .with_timezone(&Utc),
                is_active: row.get::<i64, _>("is_active") != 0,
            })
            .collect();

        Ok(entries)
    }

    // Reporting system
    pub async fn submit_report(&self, report: ReputationReport) -> Result<(), ReputationError> {
        sqlx::query(
            r#"
            INSERT INTO reputation_reports (reporter_address, target_address, target_type, report_type, description, evidence, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&report.reporter_address)
        .bind(&report.target_address)
        .bind(&report.target_type)
        .bind(&report.report_type)
        .bind(&report.description)
        .bind(&report.evidence)
        .bind(report.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Check if we should auto-blacklist based on report count
        let report_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM reputation_reports WHERE target_address = ?")
                .bind(&report.target_address)
                .fetch_one(&self.pool)
                .await?;

        if report_count >= 10 {
            self.add_to_blacklist(
                &report.target_address,
                &report.target_type,
                "Multiple community reports",
                None,
                "automated",
            )
            .await?;
        }

        Ok(())
    }

    // History tracking
    async fn record_history(
        &self,
        address: &str,
        trust_score: f64,
        event_type: &str,
        details: Option<&str>,
    ) -> Result<(), ReputationError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO reputation_history (address, timestamp, trust_score, event_type, details)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(address)
        .bind(&now)
        .bind(trust_score)
        .bind(event_type)
        .bind(details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_reputation_history(
        &self,
        address: &str,
        limit: Option<i64>,
    ) -> Result<Vec<ReputationHistory>, ReputationError> {
        let limit = limit.unwrap_or(100);
        let rows = sqlx::query(
            r#"
            SELECT address, timestamp, trust_score, event_type, details
            FROM reputation_history
            WHERE address = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(address)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let history = rows
            .iter()
            .map(|row| ReputationHistory {
                address: row.get("address"),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<String, _>("timestamp"))
                    .unwrap()
                    .with_timezone(&Utc),
                trust_score: row.get("trust_score"),
                event_type: row.get("event_type"),
                details: row.get("details"),
            })
            .collect();

        Ok(history)
    }

    // Statistics
    pub async fn get_stats(&self) -> Result<ReputationStats, ReputationError> {
        let total_wallets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wallet_reputation")
            .fetch_one(&self.pool)
            .await?;

        let total_tokens: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM token_reputation")
            .fetch_one(&self.pool)
            .await?;

        let total_vouches: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM vouches WHERE is_active = 1")
                .fetch_one(&self.pool)
                .await?;

        let total_blacklisted: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM blacklist WHERE is_active = 1")
                .fetch_one(&self.pool)
                .await?;

        let recent_reports: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reputation_reports WHERE timestamp > datetime('now', '-7 days')",
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let avg_score: Option<f64> =
            sqlx::query_scalar("SELECT AVG(trust_score) FROM wallet_reputation")
                .fetch_one(&self.pool)
                .await
                .ok();

        Ok(ReputationStats {
            total_wallets_tracked: total_wallets,
            total_tokens_tracked: total_tokens,
            total_vouches,
            total_blacklisted,
            recent_reports,
            average_trust_score: avg_score.unwrap_or(50.0),
        })
    }

    // Settings
    pub fn get_settings(&self) -> &ReputationSettings {
        &self.settings
    }

    pub fn update_settings(&mut self, settings: ReputationSettings) {
        self.settings = settings;
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_wallet_reputation(
    address: String,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<WalletReputation, String> {
    let engine = engine.read().await;
    engine
        .get_wallet_reputation(&address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_token_reputation(
    address: String,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<TokenReputation, String> {
    let engine = engine.read().await;
    engine
        .get_token_reputation(&address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_wallet_behavior(
    address: String,
    transaction_count: Option<i64>,
    total_volume: Option<f64>,
    age_days: Option<i64>,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .update_wallet_behavior(&address, transaction_count, total_volume, age_days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn initialize_token_reputation(
    address: String,
    creator_address: String,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .initialize_token_reputation(&address, &creator_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_token_metrics(
    address: String,
    holder_count: Option<i64>,
    liquidity_score: Option<f64>,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .update_token_metrics(&address, holder_count, liquidity_score)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_vouch(
    voucher_address: String,
    target_address: String,
    target_type: String,
    comment: Option<String>,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .add_vouch(&voucher_address, &target_address, &target_type, comment)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_vouch(
    voucher_address: String,
    target_address: String,
    target_type: String,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .remove_vouch(&voucher_address, &target_address, &target_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_vouches(
    target_address: String,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<Vec<VouchRecord>, String> {
    let engine = engine.read().await;
    engine
        .get_vouches(&target_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_to_blacklist(
    address: String,
    entry_type: String,
    reason: String,
    reporter: Option<String>,
    source: String,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .add_to_blacklist(&address, &entry_type, &reason, reporter, &source)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_from_blacklist(
    address: String,
    entry_type: String,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .remove_from_blacklist(&address, &entry_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_blacklist(
    entry_type: Option<String>,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<Vec<BlacklistEntry>, String> {
    let engine = engine.read().await;
    engine
        .get_blacklist(entry_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn submit_reputation_report(
    report: ReputationReport,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let engine = engine.read().await;
    engine
        .submit_report(report)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reputation_history(
    address: String,
    limit: Option<i64>,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<Vec<ReputationHistory>, String> {
    let engine = engine.read().await;
    engine
        .get_reputation_history(&address, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reputation_stats(
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<ReputationStats, String> {
    let engine = engine.read().await;
    engine.get_stats().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reputation_settings(
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<ReputationSettings, String> {
    let engine = engine.read().await;
    Ok(engine.get_settings().clone())
}

#[tauri::command]
pub async fn update_reputation_settings(
    settings: ReputationSettings,
    engine: tauri::State<'_, SharedReputationEngine>,
) -> Result<(), String> {
    let mut engine = engine.write().await;
    engine.update_settings(settings);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_trust_score_calculation() {
        // Mock app handle would be needed for real test
        // This test demonstrates the scoring logic
        let engine = ReputationEngine {
            pool: todo!(),
            settings: ReputationSettings::default(),
        };

        // New wallet
        let score1 = engine.calculate_wallet_trust_score(0, 0.0, 0, 0);
        assert_eq!(score1, 50.0);

        // Established wallet
        let score2 = engine.calculate_wallet_trust_score(1000, 1_000_000.0, 365, 5);
        assert!(score2 > 80.0);

        // Heavily vouched wallet
        let score3 = engine.calculate_wallet_trust_score(500, 500_000.0, 180, 10);
        assert!(score3 > 85.0);
    }

    #[tokio::test]
    async fn test_token_trust_score_calculation() {
        let engine = ReputationEngine {
            pool: todo!(),
            settings: ReputationSettings::default(),
        };

        // New token with good creator
        let score1 = engine.calculate_token_trust_score(100, 0.5, 85.0, 0);
        assert!(score1 > 60.0);

        // Established token
        let score2 = engine.calculate_token_trust_score(10000, 0.9, 90.0, 5);
        assert!(score2 > 80.0);

        // Token with poor creator
        let score3 = engine.calculate_token_trust_score(100, 0.5, 20.0, 0);
        assert!(score3 < 60.0);
    }

    #[test]
    fn test_reputation_level_from_score() {
        assert_eq!(
            ReputationLevel::from_score(95.0),
            ReputationLevel::Excellent
        );
        assert_eq!(ReputationLevel::from_score(70.0), ReputationLevel::Good);
        assert_eq!(ReputationLevel::from_score(50.0), ReputationLevel::Neutral);
        assert_eq!(ReputationLevel::from_score(30.0), ReputationLevel::Poor);
        assert_eq!(
            ReputationLevel::from_score(10.0),
            ReputationLevel::Malicious
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_reputation_engine_fallback_to_memory() {
        // Test with an invalid path directly
        let result = ReputationEngine::new_with_path("/invalid/path/that/does/not/exist").await;
        
        // The engine should still initialize successfully with in-memory database
        assert!(result.is_ok(), "ReputationEngine should fallback to in-memory database");
        
        let engine = result.unwrap();
        
        // Test that the engine works by getting a wallet reputation
        let test_address = "test_address_12345";
        let reputation = engine.get_wallet_reputation(test_address).await;
        assert!(reputation.is_ok(), "Should be able to get wallet reputation from in-memory DB");
        
        let wallet_rep = reputation.unwrap();
        assert_eq!(wallet_rep.address, test_address);
        assert_eq!(wallet_rep.trust_score, 50.0); // Default score
    }
    
    #[tokio::test]
    async fn test_reputation_engine_normal_file_db() {
        // Test with a valid temporary directory
        let temp_dir = TempDir::new().unwrap();
        let result = ReputationEngine::new_with_path(temp_dir.path()).await;
        
        assert!(result.is_ok(), "ReputationEngine should work with valid file path");
        
        let engine = result.unwrap();
        
        // Test basic functionality
        let test_address = "test_file_db_address";
        let reputation = engine.get_wallet_reputation(test_address).await;
        assert!(reputation.is_ok(), "Should be able to get wallet reputation from file DB");
        
        let wallet_rep = reputation.unwrap();
        assert_eq!(wallet_rep.address, test_address);
        assert_eq!(wallet_rep.trust_score, 50.0); // Default score
    }
}
