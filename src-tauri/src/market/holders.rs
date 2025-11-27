use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

const HOLDERS_DB_FILE: &str = "holders.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderInfo {
    pub address: String,
    pub balance: f64,
    pub percentage: f64,
    pub is_known_wallet: bool,
    pub wallet_label: Option<String>,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderDistribution {
    pub token_address: String,
    pub total_holders: u64,
    pub top_holders: Vec<HolderInfo>,
    pub gini_coefficient: f64,
    pub concentration_risk: String,
    pub top_10_percentage: f64,
    pub top_50_percentage: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderTrend {
    pub timestamp: String,
    pub holder_count: u64,
    pub new_holders: u32,
    pub existing_holders: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeTransfer {
    pub id: String,
    pub token_address: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub percentage_of_supply: f64,
    pub timestamp: String,
    pub transaction_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: f64,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub update_authority: Option<String>,
    pub creation_date: String,
    pub creator: String,
    pub logo_uri: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
    pub description: Option<String>,
    pub token_program: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationStatus {
    pub verified: bool,
    pub verified_on_solana_explorer: bool,
    pub audit_status: String,
    pub audit_provider: Option<String>,
    pub audit_date: Option<String>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub community_votes: CommunityVotes,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vulnerability {
    pub severity: String,
    pub description: String,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityVotes {
    pub upvotes: u32,
    pub downvotes: u32,
    pub total_votes: u32,
    pub trust_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderDataExport {
    pub token_address: String,
    pub exported_at: String,
    pub distribution: HolderDistribution,
    pub trends: Vec<HolderTrend>,
    pub large_transfers: Vec<LargeTransfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataSnapshot {
    pub token_address: String,
    pub exported_at: String,
    pub metadata: TokenMetadata,
    pub verification: VerificationStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum HolderError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("token not found: {0}")]
    NotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Clone)]
pub struct HolderAnalyzer {
    pool: Pool<Sqlite>,
}

pub type SharedHolderAnalyzer = Arc<RwLock<HolderAnalyzer>>;

impl HolderAnalyzer {
    pub async fn new(app: &AppHandle) -> Result<Self, HolderError> {
        let db_path = holder_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let analyzer = Self { pool };
        analyzer.initialize().await?;
        Ok(analyzer)
    }

    pub fn with_pool(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    async fn initialize(&self) -> Result<(), HolderError> {
        // Create holders table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS holders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_address TEXT NOT NULL,
                holder_address TEXT NOT NULL,
                balance REAL NOT NULL,
                percentage REAL NOT NULL,
                is_known_wallet INTEGER NOT NULL DEFAULT 0,
                wallet_label TEXT,
                updated_at TEXT NOT NULL,
                UNIQUE(token_address, holder_address)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create holder trends table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS holder_trends (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_address TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                holder_count INTEGER NOT NULL,
                new_holders INTEGER NOT NULL,
                existing_holders INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create large transfers table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS large_transfers (
                id TEXT PRIMARY KEY,
                token_address TEXT NOT NULL,
                from_address TEXT NOT NULL,
                to_address TEXT NOT NULL,
                amount REAL NOT NULL,
                percentage_of_supply REAL NOT NULL,
                timestamp TEXT NOT NULL,
                transaction_signature TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create token metadata table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS token_metadata (
                address TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                decimals INTEGER NOT NULL,
                total_supply REAL NOT NULL,
                mint_authority TEXT,
                freeze_authority TEXT,
                update_authority TEXT,
                creation_date TEXT NOT NULL,
                creator TEXT NOT NULL,
                logo_uri TEXT,
                website TEXT,
                twitter TEXT,
                telegram TEXT,
                discord TEXT,
                description TEXT,
                token_program TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create verification status table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS verification_status (
                token_address TEXT PRIMARY KEY,
                verified INTEGER NOT NULL DEFAULT 0,
                verified_on_solana_explorer INTEGER NOT NULL DEFAULT 0,
                audit_status TEXT NOT NULL,
                audit_provider TEXT,
                audit_date TEXT,
                risk_score REAL NOT NULL DEFAULT 0.5,
                upvotes INTEGER NOT NULL DEFAULT 0,
                downvotes INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create vulnerabilities table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vulnerabilities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                token_address TEXT NOT NULL,
                severity TEXT NOT NULL,
                description TEXT NOT NULL,
                discovered_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_holders_token 
            ON holders(token_address);
            CREATE INDEX IF NOT EXISTS idx_holder_trends_token 
            ON holder_trends(token_address, timestamp);
            CREATE INDEX IF NOT EXISTS idx_large_transfers_token 
            ON large_transfers(token_address, timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Calculate Gini coefficient for holder distribution
    pub fn calculate_gini_coefficient(&self, balances: &[f64]) -> f64 {
        if balances.is_empty() {
            return 0.0;
        }

        let mut sorted = balances.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len() as f64;
        let sum_of_absolute_differences: f64 = sorted
            .iter()
            .enumerate()
            .flat_map(|(i, &xi)| sorted.iter().skip(i + 1).map(move |&xj| (xi - xj).abs()))
            .sum();

        let mean = sorted.iter().sum::<f64>() / n;
        if mean == 0.0 {
            return 0.0;
        }

        sum_of_absolute_differences / (2.0 * n * n * mean)
    }

    pub async fn get_holder_distribution(
        &self,
        token_address: &str,
    ) -> Result<HolderDistribution, HolderError> {
        // In production, this would fetch from Solana RPC or indexer
        // For now, we'll generate mock data with realistic distribution

        // Generate mock holder data
        let mut holders = self.generate_mock_holders(token_address);

        // Calculate percentages
        let total_balance: f64 = holders.iter().map(|h| h.balance).sum();
        for holder in &mut holders {
            holder.percentage = (holder.balance / total_balance) * 100.0;
        }

        // Sort by balance descending
        holders.sort_by(|a, b| {
            b.balance
                .partial_cmp(&a.balance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (i, holder) in holders.iter_mut().enumerate() {
            holder.rank = (i + 1) as u32;
        }

        let total_holders = holders.len() as u64;
        let top_10_percentage: f64 = holders.iter().take(10).map(|h| h.percentage).sum();
        let top_50_percentage: f64 = holders.iter().take(50).map(|h| h.percentage).sum();

        let balances: Vec<f64> = holders.iter().map(|h| h.balance).collect();
        let gini = self.calculate_gini_coefficient(&balances);

        let concentration_risk = if gini > 0.8 {
            "Critical".to_string()
        } else if gini > 0.6 {
            "High".to_string()
        } else if gini > 0.4 {
            "Medium".to_string()
        } else {
            "Low".to_string()
        };

        Ok(HolderDistribution {
            token_address: token_address.to_string(),
            total_holders,
            top_holders: holders.into_iter().take(100).collect(),
            gini_coefficient: gini,
            concentration_risk,
            top_10_percentage,
            top_50_percentage,
            updated_at: Utc::now().to_rfc3339(),
        })
    }

    fn generate_mock_holders(&self, _token_address: &str) -> Vec<HolderInfo> {
        // Known wallets for identification
        let known_wallets = vec![
            (
                "DeFi Protocol Treasury",
                "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
            ),
            (
                "Team Vesting",
                "3xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
            ),
            (
                "Marketing Fund",
                "4xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
            ),
            (
                "Liquidity Pool",
                "5xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
            ),
            (
                "Exchange Cold Wallet",
                "6xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
            ),
        ];

        let mut holders = Vec::new();
        let num_holders = rand::random_range(500..2000);

        // Top holders (whale distribution)
        for i in 0..20 {
            let is_known = i < 5;
            let balance = if i < 5 {
                rand::random_range(5000000.0..15000000.0)
            } else {
                rand::random_range(500000.0..5000000.0)
            };

            let (is_known_wallet, wallet_label, address) = if is_known {
                let (label, addr) = &known_wallets[i];
                (true, Some(label.to_string()), addr.to_string())
            } else {
                (
                    false,
                    None,
                    format!("{}xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", i),
                )
            };

            holders.push(HolderInfo {
                address,
                balance,
                percentage: 0.0, // Will be calculated
                is_known_wallet,
                wallet_label,
                rank: 0, // Will be assigned
            });
        }

        // Medium holders
        for i in 20..200 {
            holders.push(HolderInfo {
                address: format!("{}xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", i),
                balance: rand::random_range(10000.0..500000.0),
                percentage: 0.0,
                is_known_wallet: false,
                wallet_label: None,
                rank: 0,
            });
        }

        // Small holders (long tail)
        for i in 200..num_holders {
            holders.push(HolderInfo {
                address: format!("{}xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", i),
                balance: rand::random_range(1.0..10000.0),
                percentage: 0.0,
                is_known_wallet: false,
                wallet_label: None,
                rank: 0,
            });
        }

        holders
    }

    pub async fn get_holder_trends(
        &self,
        token_address: &str,
        days: u32,
    ) -> Result<Vec<HolderTrend>, HolderError> {
        let mut trends = Vec::new();
        let mut base_holders = 500u64;

        for day in (0..days).rev() {
            let timestamp = Utc::now() - chrono::Duration::days(day as i64);
            let new_holders = rand::random_range(10..100);
            let existing_holders = base_holders as u32;

            base_holders += new_holders as u64;

            trends.push(HolderTrend {
                timestamp: timestamp.to_rfc3339(),
                holder_count: base_holders,
                new_holders,
                existing_holders,
            });
        }

        Ok(trends)
    }

    pub async fn get_large_transfers(
        &self,
        token_address: &str,
        days: u32,
    ) -> Result<Vec<LargeTransfer>, HolderError> {
        let mut transfers = Vec::new();
        let num_transfers = rand::random_range(3..10);

        for i in 0..num_transfers {
            let days_ago = rand::random_range(0..days);
            let timestamp = Utc::now() - chrono::Duration::days(days_ago as i64);
            let amount = rand::random_range(100000.0..5000000.0);
            let percentage = rand::random_range(0.1..5.0);

            transfers.push(LargeTransfer {
                id: uuid::Uuid::new_v4().to_string(),
                token_address: token_address.to_string(),
                from_address: format!("From{}xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", i),
                to_address: format!("To{}xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", i),
                amount,
                percentage_of_supply: percentage,
                timestamp: timestamp.to_rfc3339(),
                transaction_signature: format!("{}signature123456789", i),
            });
        }

        transfers.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(transfers)
    }

    pub async fn get_token_metadata(
        &self,
        token_address: &str,
    ) -> Result<TokenMetadata, HolderError> {
        // In production, fetch from Solana RPC
        // For now, return mock data

        Ok(TokenMetadata {
            address: token_address.to_string(),
            name: "Example Token".to_string(),
            symbol: "EXT".to_string(),
            decimals: 9,
            total_supply: 100_000_000.0,
            mint_authority: Some("AuthxQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string()),
            freeze_authority: None,
            update_authority: Some("UpdatexQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string()),
            creation_date: "2024-01-15T10:30:00Z".to_string(),
            creator: "CreatorxQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string(),
            logo_uri: Some("https://example.com/logo.png".to_string()),
            website: Some("https://example-token.com".to_string()),
            twitter: Some("https://twitter.com/exampletoken".to_string()),
            telegram: Some("https://t.me/exampletoken".to_string()),
            discord: Some("https://discord.gg/exampletoken".to_string()),
            description: Some("A revolutionary token for decentralized trading".to_string()),
            token_program: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
        })
    }

    pub async fn get_verification_status(
        &self,
        token_address: &str,
    ) -> Result<VerificationStatus, HolderError> {
        let verified = rand::random_bool(0.6);
        let has_audit = rand::random_bool(0.4);
        let num_vulnerabilities = if rand::random_bool(0.3) {
            rand::random_range(1..3)
        } else {
            0
        };

        let mut vulnerabilities = Vec::new();
        for i in 0..num_vulnerabilities {
            let severity = match i % 3 {
                0 => "Low",
                1 => "Medium",
                _ => "High",
            };
            vulnerabilities.push(Vulnerability {
                severity: severity.to_string(),
                description: format!(
                    "Potential {} severity issue detected in smart contract",
                    severity
                ),
                discovered_at: (Utc::now() - chrono::Duration::days(30)).to_rfc3339(),
            });
        }

        let upvotes = rand::random_range(50..500);
        let downvotes = rand::random_range(10..100);
        let total_votes = upvotes + downvotes;
        let trust_score = upvotes as f64 / total_votes as f64;

        let risk_score = if !verified {
            0.7
        } else if num_vulnerabilities > 0 {
            0.5
        } else {
            0.2
        };

        Ok(VerificationStatus {
            verified,
            verified_on_solana_explorer: verified && (rand::random::<f64>() < 0.8),
            audit_status: if has_audit {
                "Audited".to_string()
            } else {
                "Not Audited".to_string()
            },
            audit_provider: if has_audit {
                Some("CertiK".to_string())
            } else {
                None
            },
            audit_date: if has_audit {
                Some("2024-06-15".to_string())
            } else {
                None
            },
            vulnerabilities,
            community_votes: CommunityVotes {
                upvotes,
                downvotes,
                total_votes,
                trust_score,
            },
            risk_score,
        })
    }

    pub async fn export_holder_data(
        &self,
        token_address: &str,
        days: u32,
    ) -> Result<HolderDataExport, HolderError> {
        let distribution = self.get_holder_distribution(token_address).await?;
        let trends = self.get_holder_trends(token_address, days).await?;
        let large_transfers = self.get_large_transfers(token_address, days).await?;

        Ok(HolderDataExport {
            token_address: token_address.to_string(),
            exported_at: Utc::now().to_rfc3339(),
            distribution,
            trends,
            large_transfers,
        })
    }

    pub async fn export_metadata_snapshot(
        &self,
        token_address: &str,
    ) -> Result<MetadataSnapshot, HolderError> {
        let metadata = self.get_token_metadata(token_address).await?;
        let verification = self.get_verification_status(token_address).await?;

        Ok(MetadataSnapshot {
            token_address: token_address.to_string(),
            exported_at: Utc::now().to_rfc3339(),
            metadata,
            verification,
        })
    }
}

fn holder_db_path(app: &AppHandle) -> Result<PathBuf, HolderError> {
    let mut path = app.path().app_data_dir().map_err(|err| {
        HolderError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;

    std::fs::create_dir_all(&path).map_err(|e| {
        HolderError::Internal(format!("Failed to create app data directory: {}", e))
    })?;

    path.push(HOLDERS_DB_FILE);
    Ok(path)
}

// Tauri commands
#[tauri::command]
pub async fn get_holder_distribution(
    token_address: String,
    analyzer: State<'_, SharedHolderAnalyzer>,
) -> Result<HolderDistribution, String> {
    let analyzer = analyzer.read().await;
    analyzer
        .get_holder_distribution(&token_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_holder_trends(
    token_address: String,
    days: u32,
    analyzer: State<'_, SharedHolderAnalyzer>,
) -> Result<Vec<HolderTrend>, String> {
    let analyzer = analyzer.read().await;
    analyzer
        .get_holder_trends(&token_address, days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_large_transfers(
    token_address: String,
    days: u32,
    analyzer: State<'_, SharedHolderAnalyzer>,
) -> Result<Vec<LargeTransfer>, String> {
    let analyzer = analyzer.read().await;
    analyzer
        .get_large_transfers(&token_address, days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_token_metadata(
    token_address: String,
    analyzer: State<'_, SharedHolderAnalyzer>,
) -> Result<TokenMetadata, String> {
    let analyzer = analyzer.read().await;
    analyzer
        .get_token_metadata(&token_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_verification_status(
    token_address: String,
    analyzer: State<'_, SharedHolderAnalyzer>,
) -> Result<VerificationStatus, String> {
    let analyzer = analyzer.read().await;
    analyzer
        .get_verification_status(&token_address)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_holder_data(
    token_address: String,
    days: u32,
    analyzer: State<'_, SharedHolderAnalyzer>,
) -> Result<HolderDataExport, String> {
    let analyzer = analyzer.read().await;
    analyzer
        .export_holder_data(&token_address, days)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_metadata_snapshot(
    token_address: String,
    analyzer: State<'_, SharedHolderAnalyzer>,
) -> Result<MetadataSnapshot, String> {
    let analyzer = analyzer.read().await;
    analyzer
        .export_metadata_snapshot(&token_address)
        .await
        .map_err(|e| e.to_string())
}
