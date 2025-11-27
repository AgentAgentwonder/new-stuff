use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use uuid::Uuid;

// Squads Protocol Program ID (mainnet-beta)
pub const SQUADS_PROGRAM_ID: &str = "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
    Cancelled,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Pending => write!(f, "pending"),
            ProposalStatus::Approved => write!(f, "approved"),
            ProposalStatus::Executed => write!(f, "executed"),
            ProposalStatus::Rejected => write!(f, "rejected"),
            ProposalStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl FromStr for ProposalStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ProposalStatus::Pending),
            "approved" => Ok(ProposalStatus::Approved),
            "executed" => Ok(ProposalStatus::Executed),
            "rejected" => Ok(ProposalStatus::Rejected),
            "cancelled" => Ok(ProposalStatus::Cancelled),
            _ => Err(anyhow!("Invalid proposal status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultisigWallet {
    pub id: String,
    pub name: String,
    pub address: String,
    pub threshold: u32,
    pub members: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultisigProposal {
    pub id: String,
    pub wallet_id: String,
    pub transaction_data: String,
    pub status: ProposalStatus,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
    pub signatures: Vec<ProposalSignature>,
    pub executed_at: Option<DateTime<Utc>>,
    pub tx_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalSignature {
    pub id: String,
    pub proposal_id: String,
    pub signer: String,
    pub signature: String,
    pub signed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMultisigRequest {
    pub name: String,
    pub members: Vec<String>,
    pub threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProposalRequest {
    pub wallet_id: String,
    pub transaction_data: String,
    pub description: Option<String>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignProposalRequest {
    pub proposal_id: String,
    pub signer: String,
    pub signature: String,
}

pub struct MultisigDatabase {
    pool: Pool<Sqlite>,
}

impl MultisigDatabase {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: MultisigDatabase failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for MultisigDatabase");
                eprintln!("MultisigDatabase using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        let db = Self { pool };
        db.initialize().await?;

        Ok(db)
    }

    async fn initialize(&self) -> Result<()> {
        // Create multisig_wallets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multisig_wallets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                address TEXT NOT NULL,
                threshold INTEGER NOT NULL,
                members TEXT NOT NULL,
                created_at TEXT NOT NULL,
                balance REAL NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create multisig_proposals table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multisig_proposals (
                id TEXT PRIMARY KEY,
                wallet_id TEXT NOT NULL,
                transaction_data TEXT NOT NULL,
                status TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at TEXT NOT NULL,
                description TEXT,
                executed_at TEXT,
                tx_signature TEXT,
                FOREIGN KEY (wallet_id) REFERENCES multisig_wallets(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create multisig_signatures table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multisig_signatures (
                id TEXT PRIMARY KEY,
                proposal_id TEXT NOT NULL,
                signer TEXT NOT NULL,
                signature TEXT NOT NULL,
                signed_at TEXT NOT NULL,
                FOREIGN KEY (proposal_id) REFERENCES multisig_proposals(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_multisig_proposals_wallet ON multisig_proposals(wallet_id);
            CREATE INDEX IF NOT EXISTS idx_multisig_proposals_status ON multisig_proposals(status);
            CREATE INDEX IF NOT EXISTS idx_multisig_signatures_proposal ON multisig_signatures(proposal_id);
            CREATE INDEX IF NOT EXISTS idx_multisig_signatures_signer ON multisig_signatures(signer);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_wallet(&self, request: CreateMultisigRequest) -> Result<MultisigWallet> {
        // Generate a deterministic multisig address based on members and threshold
        let multisig_address = self.derive_multisig_address(&request.members, request.threshold)?;

        let wallet = MultisigWallet {
            id: format!("multisig_{}", Uuid::new_v4()),
            name: request.name,
            address: multisig_address,
            threshold: request.threshold,
            members: request.members.clone(),
            created_at: Utc::now(),
            balance: 0.0,
        };

        let members_json = serde_json::to_string(&request.members)?;

        sqlx::query(
            r#"
            INSERT INTO multisig_wallets (id, name, address, threshold, members, created_at, balance)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&wallet.id)
        .bind(&wallet.name)
        .bind(&wallet.address)
        .bind(wallet.threshold as i64)
        .bind(&members_json)
        .bind(wallet.created_at.to_rfc3339())
        .bind(wallet.balance)
        .execute(&self.pool)
        .await?;

        Ok(wallet)
    }

    pub async fn get_wallet(&self, wallet_id: &str) -> Result<Option<MultisigWallet>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, address, threshold, members, created_at, balance
            FROM multisig_wallets
            WHERE id = ?1
            "#,
        )
        .bind(wallet_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let members_json: String = row.try_get("members")?;
            let members: Vec<String> = serde_json::from_str(&members_json)?;

            let wallet = MultisigWallet {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                address: row.try_get("address")?,
                threshold: row.try_get::<i64, _>("threshold")? as u32,
                members,
                created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
                balance: row.try_get("balance")?,
            };

            Ok(Some(wallet))
        } else {
            Ok(None)
        }
    }

    pub async fn list_wallets(&self) -> Result<Vec<MultisigWallet>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, address, threshold, members, created_at, balance
            FROM multisig_wallets
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut wallets = Vec::new();
        for row in rows {
            let members_json: String = row.try_get("members")?;
            let members: Vec<String> = serde_json::from_str(&members_json)?;

            let wallet = MultisigWallet {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                address: row.try_get("address")?,
                threshold: row.try_get::<i64, _>("threshold")? as u32,
                members,
                created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
                balance: row.try_get("balance")?,
            };

            wallets.push(wallet);
        }

        Ok(wallets)
    }

    pub async fn create_proposal(
        &self,
        request: CreateProposalRequest,
    ) -> Result<MultisigProposal> {
        let proposal = MultisigProposal {
            id: format!("proposal_{}", Uuid::new_v4()),
            wallet_id: request.wallet_id,
            transaction_data: request.transaction_data,
            status: ProposalStatus::Pending,
            created_by: request.created_by,
            created_at: Utc::now(),
            description: request.description,
            signatures: vec![],
            executed_at: None,
            tx_signature: None,
        };

        sqlx::query(
            r#"
            INSERT INTO multisig_proposals (id, wallet_id, transaction_data, status, created_by, created_at, description, executed_at, tx_signature)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&proposal.id)
        .bind(&proposal.wallet_id)
        .bind(&proposal.transaction_data)
        .bind(proposal.status.to_string())
        .bind(&proposal.created_by)
        .bind(proposal.created_at.to_rfc3339())
        .bind(&proposal.description)
        .bind(proposal.executed_at.map(|dt| dt.to_rfc3339()))
        .bind(&proposal.tx_signature)
        .execute(&self.pool)
        .await?;

        Ok(proposal)
    }

    pub async fn get_proposal(&self, proposal_id: &str) -> Result<Option<MultisigProposal>> {
        let row = sqlx::query(
            r#"
            SELECT id, wallet_id, transaction_data, status, created_by, created_at, description, executed_at, tx_signature
            FROM multisig_proposals
            WHERE id = ?1
            "#,
        )
        .bind(proposal_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let proposal = MultisigProposal {
                id: row.try_get("id")?,
                wallet_id: row.try_get("wallet_id")?,
                transaction_data: row.try_get("transaction_data")?,
                status: ProposalStatus::from_str(&row.try_get::<String, _>("status")?)?,
                created_by: row.try_get("created_by")?,
                created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
                description: row.try_get("description")?,
                signatures: self
                    .get_proposal_signatures(&row.try_get::<String, _>("id")?)
                    .await?,
                executed_at: row
                    .try_get::<Option<String>, _>("executed_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                tx_signature: row.try_get("tx_signature")?,
            };

            Ok(Some(proposal))
        } else {
            Ok(None)
        }
    }

    pub async fn list_proposals(
        &self,
        wallet_id: &str,
        status_filter: Option<String>,
    ) -> Result<Vec<MultisigProposal>> {
        let query_builder = if let Some(ref status) = status_filter {
            let status_filter_query = r#"
                SELECT id, wallet_id, transaction_data, status, created_by, created_at, description, executed_at, tx_signature
                FROM multisig_proposals
                WHERE wallet_id = ?1 AND status = ?2
                ORDER BY created_at DESC
            "#;
            sqlx::query(status_filter_query)
                .bind(wallet_id)
                .bind(status)
        } else {
            let query = r#"
            SELECT id, wallet_id, transaction_data, status, created_by, created_at, description, executed_at, tx_signature
            FROM multisig_proposals
            WHERE wallet_id = ?1
            ORDER BY created_at DESC
            "#;
            sqlx::query(query).bind(wallet_id)
        };

        let rows = query_builder.fetch_all(&self.pool).await?;

        let mut proposals = Vec::new();
        for row in rows {
            let proposal_id: String = row.try_get("id")?;
            let signatures = self.get_proposal_signatures(&proposal_id).await?;

            let proposal = MultisigProposal {
                id: proposal_id,
                wallet_id: row.try_get("wallet_id")?,
                transaction_data: row.try_get("transaction_data")?,
                status: ProposalStatus::from_str(&row.try_get::<String, _>("status")?)?,
                created_by: row.try_get("created_by")?,
                created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
                description: row.try_get("description")?,
                signatures,
                executed_at: row
                    .try_get::<Option<String>, _>("executed_at")?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                tx_signature: row.try_get("tx_signature")?,
            };

            proposals.push(proposal);
        }

        Ok(proposals)
    }

    pub async fn add_signature(&self, request: SignProposalRequest) -> Result<ProposalSignature> {
        let signature = ProposalSignature {
            id: format!("sig_{}", Uuid::new_v4()),
            proposal_id: request.proposal_id.clone(),
            signer: request.signer,
            signature: request.signature,
            signed_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO multisig_signatures (id, proposal_id, signer, signature, signed_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&signature.id)
        .bind(&signature.proposal_id)
        .bind(&signature.signer)
        .bind(&signature.signature)
        .bind(signature.signed_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        // Check if threshold is met
        let proposal = self.get_proposal(&request.proposal_id).await?;
        if let Some(proposal) = proposal {
            let wallet = self.get_wallet(&proposal.wallet_id).await?;
            if let Some(wallet) = wallet {
                let signatures = self.get_proposal_signatures(&request.proposal_id).await?;
                if signatures.len() >= wallet.threshold as usize {
                    // Update proposal status to approved
                    sqlx::query(
                        r#"
                        UPDATE multisig_proposals
                        SET status = ?1
                        WHERE id = ?2
                        "#,
                    )
                    .bind(ProposalStatus::Approved.to_string())
                    .bind(&request.proposal_id)
                    .execute(&self.pool)
                    .await?;
                }
            }
        }

        Ok(signature)
    }

    pub async fn get_proposal_signatures(
        &self,
        proposal_id: &str,
    ) -> Result<Vec<ProposalSignature>> {
        let rows = sqlx::query(
            r#"
            SELECT id, proposal_id, signer, signature, signed_at
            FROM multisig_signatures
            WHERE proposal_id = ?1
            ORDER BY signed_at ASC
            "#,
        )
        .bind(proposal_id)
        .fetch_all(&self.pool)
        .await?;

        let mut signatures = Vec::new();
        for row in rows {
            let signature = ProposalSignature {
                id: row.try_get("id")?,
                proposal_id: row.try_get("proposal_id")?,
                signer: row.try_get("signer")?,
                signature: row.try_get("signature")?,
                signed_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("signed_at")?)
                    .map(|dt| dt.with_timezone(&Utc))?,
            };
            signatures.push(signature);
        }

        Ok(signatures)
    }

    pub async fn execute_proposal(&self, proposal_id: &str, tx_signature: String) -> Result<()> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE multisig_proposals
            SET status = ?1, executed_at = ?2, tx_signature = ?3
            WHERE id = ?4
            "#,
        )
        .bind(ProposalStatus::Executed.to_string())
        .bind(now.to_rfc3339())
        .bind(tx_signature)
        .bind(proposal_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn cancel_proposal(&self, proposal_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE multisig_proposals
            SET status = ?1
            WHERE id = ?2
            "#,
        )
        .bind(ProposalStatus::Cancelled.to_string())
        .bind(proposal_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn derive_multisig_address(&self, members: &[String], threshold: u32) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(b"multisig_v1");
        hasher.update(&threshold.to_le_bytes());

        for member in members {
            hasher.update(member.as_bytes());
        }

        let hash = hasher.finalize();
        let address_bytes = &hash[..32];
        Ok(bs58::encode(address_bytes).into_string())
    }
}

pub type SharedMultisigDatabase = Arc<RwLock<MultisigDatabase>>;

// Tauri Commands

#[tauri::command]
pub async fn create_multisig_wallet(
    request: CreateMultisigRequest,
    db: State<'_, SharedMultisigDatabase>,
) -> Result<MultisigWallet, String> {
    // Validate request
    if request.members.is_empty() {
        return Err("At least one member is required".to_string());
    }

    if request.threshold == 0 || request.threshold > request.members.len() as u32 {
        return Err(format!(
            "Threshold must be between 1 and {}",
            request.members.len()
        ));
    }

    let db_guard = db.read().await;
    db_guard
        .create_wallet(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_multisig_wallets(
    db: State<'_, SharedMultisigDatabase>,
) -> Result<Vec<MultisigWallet>, String> {
    let db_guard = db.read().await;
    db_guard.list_wallets().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_multisig_wallet(
    wallet_id: String,
    db: State<'_, SharedMultisigDatabase>,
) -> Result<Option<MultisigWallet>, String> {
    let db_guard = db.read().await;
    db_guard
        .get_wallet(&wallet_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_proposal(
    request: CreateProposalRequest,
    db: State<'_, SharedMultisigDatabase>,
) -> Result<MultisigProposal, String> {
    let db_guard = db.read().await;

    // Verify wallet exists
    let wallet = db_guard
        .get_wallet(&request.wallet_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Wallet not found".to_string())?;

    // Verify creator is a member
    if !wallet.members.contains(&request.created_by) {
        return Err("Only wallet members can create proposals".to_string());
    }

    db_guard
        .create_proposal(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_proposals(
    wallet_id: String,
    status_filter: Option<String>,
    db: State<'_, SharedMultisigDatabase>,
) -> Result<Vec<MultisigProposal>, String> {
    let db_guard = db.read().await;
    db_guard
        .list_proposals(&wallet_id, status_filter)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sign_proposal(
    request: SignProposalRequest,
    db: State<'_, SharedMultisigDatabase>,
) -> Result<ProposalSignature, String> {
    let db_guard = db.read().await;

    // Verify proposal exists
    let proposal = db_guard
        .get_proposal(&request.proposal_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Proposal not found".to_string())?;

    // Verify proposal is pending or approved
    if proposal.status != ProposalStatus::Pending && proposal.status != ProposalStatus::Approved {
        return Err("Proposal is not in a signable state".to_string());
    }

    // Verify wallet exists
    let wallet = db_guard
        .get_wallet(&proposal.wallet_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Wallet not found".to_string())?;

    // Verify signer is a member
    if !wallet.members.contains(&request.signer) {
        return Err("Only wallet members can sign proposals".to_string());
    }

    // Check if already signed
    if proposal
        .signatures
        .iter()
        .any(|sig| sig.signer == request.signer)
    {
        return Err("You have already signed this proposal".to_string());
    }

    db_guard
        .add_signature(request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_proposal(
    proposal_id: String,
    db: State<'_, SharedMultisigDatabase>,
) -> Result<String, String> {
    let db_guard = db.read().await;

    // Verify proposal exists and is approved
    let proposal = db_guard
        .get_proposal(&proposal_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Proposal not found".to_string())?;

    if proposal.status != ProposalStatus::Approved {
        return Err("Proposal is not approved for execution".to_string());
    }

    // Verify threshold is met
    let wallet = db_guard
        .get_wallet(&proposal.wallet_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Wallet not found".to_string())?;

    if proposal.signatures.len() < wallet.threshold as usize {
        return Err(format!(
            "Insufficient signatures: {} of {} required",
            proposal.signatures.len(),
            wallet.threshold
        ));
    }

    // Simulate transaction execution
    // In a real implementation, this would execute the transaction on Solana
    let tx_signature = format!("tx_{}", Uuid::new_v4());

    db_guard
        .execute_proposal(&proposal_id, tx_signature.clone())
        .await
        .map_err(|e| e.to_string())?;

    Ok(tx_signature)
}

#[tauri::command]
pub async fn cancel_proposal(
    proposal_id: String,
    user_address: String,
    db: State<'_, SharedMultisigDatabase>,
) -> Result<(), String> {
    let db_guard = db.read().await;

    // Verify proposal exists
    let proposal = db_guard
        .get_proposal(&proposal_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Proposal not found".to_string())?;

    // Only creator can cancel
    if proposal.created_by != user_address {
        return Err("Only the proposal creator can cancel it".to_string());
    }

    // Can only cancel pending proposals
    if proposal.status != ProposalStatus::Pending {
        return Err("Can only cancel pending proposals".to_string());
    }

    db_guard
        .cancel_proposal(&proposal_id)
        .await
        .map_err(|e| e.to_string())
}
