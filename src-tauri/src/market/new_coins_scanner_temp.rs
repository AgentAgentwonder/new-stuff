use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::RwLock;

const NEW_COINS_DB_FILE: &str = "new_coins.db";
const SCAN_INTERVAL_SECS: u64 = 300; // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCoin {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub logo_uri: Option<String>,
    pub created_at: String,
    pub liquidity: f64,
    pub mint_authority_revoked: bool,
    pub freeze_authority_revoked: bool,
    pub holder_count: i64,
    pub top_holder_percent: f64,
    pub creator_wallet: String,
    pub creator_reputation_score: f64,
    pub safety_score: i64,
    pub is_spam: bool,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafetyAnalysis {
    pub is_safe: bool,
    pub score: f64,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}




#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyReport {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub safety_score: i64,
    pub checks: SafetyChecks,
    pub liquidity_info: LiquidityInfo,
    pub holder_info: HolderInfo,
    pub creator_info: CreatorInfo,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyChecks {
    pub mint_authority_revoked: bool,
    pub freeze_authority_revoked: bool,
    pub has_minimum_liquidity: bool,
    pub holder_distribution_healthy: bool,
    pub creator_reputation_good: bool,
    pub not_flagged_as_spam: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityInfo {
    pub total_liquidity: f64,
    pub pool_address: Option<String>,
    pub liquidity_locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderInfo {
    pub holder_count: i64,
    pub top_holder_percent: f64,
    pub top_10_holders_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorInfo {
    pub wallet_address: String,
    pub reputation_score: f64,
    pub previous_tokens_created: i64,
    pub suspicious_activity: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum NewCoinsScannerError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct NewCoinsScanner {
    pool: Pool<Sqlite>,
    app_handle: Option<AppHandle>,
}

impl NewCoinsScanner {
    pub async fn new(app: &AppHandle) -> Result<Self, NewCoinsScannerError> {
        let db_path = get_new_coins_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let scanner = Self {
            pool,
            app_handle: Some(app.clone()),
        };

        scanner.initialize().await?;
        Ok(scanner)
    }

    async fn initialize(&self) -> Result<(), NewCoinsScannerError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS new_coins (
                address TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                logo_uri TEXT,
                created_at TEXT NOT NULL,
                liquidity REAL NOT NULL,
                mint_authority_revoked INTEGER NOT NULL,
                freeze_authority_revoked INTEGER NOT NULL,
                holder_count INTEGER NOT NULL,
                top_holder_percent REAL NOT NULL,
                creator_wallet TEXT NOT NULL,
                creator_reputation_score REAL NOT NULL,
                safety_score INTEGER NOT NULL,
                is_spam INTEGER NOT NULL,
                detected_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_new_coins_created ON new_coins(created_at);
            CREATE INDEX IF NOT EXISTS idx_new_coins_detected ON new_coins(detected_at);
            CREATE INDEX IF NOT EXISTS idx_new_coins_safety ON new_coins(safety_score);
            CREATE INDEX IF NOT EXISTS idx_new_coins_spam ON new_coins(is_spam);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn scan_for_new_tokens(&self) -> Result<Vec<NewCoin>, NewCoinsScannerError> {
        // Mock implementation - In production, this would:
        // 1. Query Solana blockchain for new token mint accounts
        // 2. Filter by age (<24 hours)
        // 3. Fetch token metadata
        // 4. Check liquidity pools
        // 5. Analyze holder distribution
        // 6. Check mint/freeze authorities

        let mock_coins = self.generate_mock_new_coins().await?;

        // Store new coins in database
        for coin in &mock_coins {
            self.store_coin(coin).await?;
        }

        // Emit event for high-safety coins
        if let Some(app) = &self.app_handle {
            for coin in &mock_coins {
                if coin.safety_score >= 70 && !coin.is_spam {
                    let _ = app.emit("new-coin-detected", coin);
                }
            }
        }

        Ok(mock_coins)
    }

    async fn generate_mock_new_coins(&self) -> Result<Vec<NewCoin>, NewCoinsScannerError> {
        use rand::Rng;
        let now = Utc::now();

        let mock_data = vec![
            ("MOON", "Moon Rocket", 85, false),
            ("DOGE2", "Doge 2.0", 45, true),
            ("SAFE", "Safe Token", 92, false),
            ("SCAM", "Scammy Coin", 15, true),
            ("GEM", "Hidden Gem", 78, false),
        ];

        let mut coins = Vec::new();

        for (idx, (symbol, name, base_safety, is_spam)) in mock_data.iter().enumerate() {
            let age_hours = rand::random_range(0..24);
            let created_at = (now - ChronoDuration::hours(age_hours)).to_rfc3339();

            let mint_revoked = base_safety >= &50;
            let freeze_revoked = base_safety >= &60;
            let liquidity = if is_spam {
                rand::random_range(500.0..1500.0)
            } else {
                rand::random_range(5000.0..50000.0)
            };
            let holder_count = if is_spam {
                rand::random_range(5..50)
            } else {
                rand::random_range(100..1000)
            };
            let top_holder_percent = if is_spam {
                rand::random_range(60.0..95.0)
            } else {
                rand::random_range(5.0..25.0)
            };
            let creator_reputation = if is_spam {
                rand::random_range(0.0..0.3)
            } else {
                rand::random_range(0.6..0.95)
            };

            let coin = NewCoin {
                address: format!("{}mock{}", symbol, idx),
                symbol: symbol.to_string(),
                name: name.to_string(),
                logo_uri: None,
                created_at,
                liquidity,
                mint_authority_revoked: mint_revoked,
                freeze_authority_revoked: freeze_revoked,
                holder_count,
                top_holder_percent,
                creator_wallet: format!("Creator{}MockWallet", idx),
                creator_reputation_score: creator_reputation,
                safety_score: *base_safety,
                is_spam: *is_spam,
                detected_at: now.to_rfc3339(),
            };

            coins.push(coin);
        }

        Ok(coins)
    }

    async fn store_coin(&self, coin: &NewCoin) -> Result<(), NewCoinsScannerError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO new_coins (
                address, symbol, name, logo_uri, created_at, liquidity,
                mint_authority_revoked, freeze_authority_revoked,
                holder_count, top_holder_percent, creator_wallet,
                creator_reputation_score, safety_score, is_spam, detected_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
            )
            "#,
        )
        .bind(&coin.address)
        .bind(&coin.symbol)
        .bind(&coin.name)
        .bind(&coin.logo_uri)
        .bind(&coin.created_at)
        .bind(coin.liquidity)
        .bind(coin.mint_authority_revoked as i32)
        .bind(coin.freeze_authority_revoked as i32)
        .bind(coin.holder_count)
        .bind(coin.top_holder_percent)
        .bind(&coin.creator_wallet)
        .bind(coin.creator_reputation_score)
        .bind(coin.safety_score)
        .bind(coin.is_spam as i32)
        .bind(&coin.detected_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_new_coins(
        &self,
        hours: Option<i64>,
        min_safety_score: Option<i64>,
    ) -> Result<Vec<NewCoin>, NewCoinsScannerError> {
        let hours = hours.unwrap_or(24);
        let min_safety = min_safety_score.unwrap_or(0);
        let cutoff_time = (Utc::now() - ChronoDuration::hours(hours)).to_rfc3339();

        let rows = sqlx::query(
            r#"
            SELECT * FROM new_coins 
            WHERE created_at >= ?1 
            AND safety_score >= ?2 
            AND is_spam = 0
            ORDER BY created_at DESC
            "#,
        )
        .bind(cutoff_time)
        .bind(min_safety)
        .fetch_all(&self.pool)
        .await?;

        let coins = rows
            .into_iter()
            .map(|row| NewCoin {
                address: row.get("address"),
                symbol: row.get("symbol"),
                name: row.get("name"),
                logo_uri: row.get("logo_uri"),
                created_at: row.get("created_at"),
                liquidity: row.get("liquidity"),
                mint_authority_revoked: row.get::<i32, _>("mint_authority_revoked") != 0,
                freeze_authority_revoked: row.get::<i32, _>("freeze_authority_revoked") != 0,
                holder_count: row.get("holder_count"),
                top_holder_percent: row.get("top_holder_percent"),
                creator_wallet: row.get("creator_wallet"),
                creator_reputation_score: row.get("creator_reputation_score"),
                safety_score: row.get("safety_score"),
                is_spam: row.get::<i32, _>("is_spam") != 0,
                detected_at: row.get("detected_at"),
            })
            .collect();

        Ok(coins)
    }

    pub async fn get_safety_report(
        &self,
        token_address: &str,
    ) -> Result<SafetyReport, NewCoinsScannerError> {
        let row = sqlx::query("SELECT * FROM new_coins WHERE address = ?1")
            .bind(token_address)
            .fetch_optional(&self.pool)
            .await?;

        let coin = row.ok_or_else(|| {
            NewCoinsScannerError::Internal(format!("Token {} not found", token_address))
        })?;

        let mint_revoked = coin.get::<i32, _>("mint_authority_revoked") != 0;
        let freeze_revoked = coin.get::<i32, _>("freeze_authority_revoked") != 0;
        let liquidity: f64 = coin.get("liquidity");
        let holder_count: i64 = coin.get("holder_count");
        let top_holder_percent: f64 = coin.get("top_holder_percent");
        let creator_reputation: f64 = coin.get("creator_reputation_score");
        let safety_score: i64 = coin.get("safety_score");
        let is_spam = coin.get::<i32, _>("is_spam") != 0;

        let checks = SafetyChecks {
            mint_authority_revoked: mint_revoked,
            freeze_authority_revoked: freeze_revoked,
            has_minimum_liquidity: liquidity >= 1000.0,
            holder_distribution_healthy: top_holder_percent < 50.0,
            creator_reputation_good: creator_reputation >= 0.5,
            not_flagged_as_spam: !is_spam,
        };

        let liquidity_info = LiquidityInfo {
            total_liquidity: liquidity,
            pool_address: None,
            liquidity_locked: false, // Mock data
        };

        let holder_info = HolderInfo {
            holder_count,
            top_holder_percent,
            top_10_holders_percent: top_holder_percent * 2.5, // Mock calculation
        };

        let creator_info = CreatorInfo {
            wallet_address: coin.get("creator_wallet"),
            reputation_score: creator_reputation,
            previous_tokens_created: 0, // Mock data
            suspicious_activity: creator_reputation < 0.3,
        };

        let recommendation = if safety_score >= 80 {
            "Safe - Low risk for investment".to_string()
        } else if safety_score >= 50 {
            "Moderate - Exercise caution, do your own research".to_string()
        } else {
            "High Risk - Not recommended, likely scam".to_string()
        };

        Ok(SafetyReport {
            address: coin.get("address"),
            symbol: coin.get("symbol"),
            name: coin.get("name"),
            safety_score,
            checks,
            liquidity_info,
            holder_info,
            creator_info,
            recommendation,
        })
    }

    pub async fn cleanup_old_coins(&self, days: i64) -> Result<(), NewCoinsScannerError> {
        let cutoff_time = (Utc::now() - ChronoDuration::days(days)).to_rfc3339();

        sqlx::query("DELETE FROM new_coins WHERE detected_at < ?1")
            .bind(cutoff_time)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

pub type SharedNewCoinsScanner = Arc<RwLock<NewCoinsScanner>>;

pub fn start_new_coins_scanner(scanner: SharedNewCoinsScanner) {
    tauri::async_runtime::spawn(async move {
        loop {
            {
                let scanner_guard = scanner.read().await;
                if let Err(e) = scanner_guard.scan_for_new_tokens().await {
                    eprintln!("Failed to scan for new tokens: {}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(SCAN_INTERVAL_SECS)).await;
        }
    });
}

fn get_new_coins_db_path(app: &AppHandle) -> Result<PathBuf, NewCoinsScannerError> {
    let mut path = app.path().app_data_dir().map_err(|err| {
        NewCoinsScannerError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;

    std::fs::create_dir_all(&path)?;
    path.push(NEW_COINS_DB_FILE);
    Ok(path)
}

// Tauri Commands
#[tauri::command]
pub async fn get_new_coins(
    scanner: tauri::State<'_, SharedNewCoinsScanner>,
    hours: Option<i64>,
    min_safety_score: Option<i64>,
) -> Result<Vec<NewCoin>, String> {
    let scanner = scanner.read().await;
    scanner
        .get_new_coins(hours, min_safety_score)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_coin_safety_report(
    scanner: tauri::State<'_, SharedNewCoinsScanner>,
    token_address: String,
) -> Result<SafetyReport, String> {
    let scanner = scanner.read().await;
    scanner
        .get_safety_report(&token_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_for_new_coins(
    scanner: tauri::State<'_, SharedNewCoinsScanner>,
) -> Result<Vec<NewCoin>, String> {
    let scanner = scanner.read().await;
    scanner
        .scan_for_new_tokens()
        .await
        .map_err(|e| e.to_string())
}

