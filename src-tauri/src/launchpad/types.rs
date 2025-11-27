use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenLaunchConfig {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub description: String,
    pub image_url: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
    pub creator_address: String,
    pub mint_authority_enabled: bool,
    pub freeze_authority_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: LaunchStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LaunchStatus {
    Draft,
    Configured,
    Simulated,
    Launched,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityLockConfig {
    pub pool_address: Option<String>,
    pub lock_amount: u64,
    pub lock_duration_seconds: u64,
    pub unlock_date: DateTime<Utc>,
    pub beneficiary: String,
    pub is_revocable: bool,
    pub lock_id: Option<String>,
    pub status: LockStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LockStatus {
    Pending,
    Locked,
    Unlocked,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VestingSchedule {
    pub id: String,
    pub token_mint: String,
    pub beneficiary: String,
    pub total_amount: u64,
    pub start_date: DateTime<Utc>,
    pub cliff_duration_seconds: Option<u64>,
    pub vesting_duration_seconds: u64,
    pub vesting_type: VestingType,
    pub released_amount: u64,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VestingType {
    Linear,
    Staged,
    Cliff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VestingStage {
    pub percentage: u8,
    pub unlock_date: DateTime<Utc>,
    pub released: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirdropConfig {
    pub id: String,
    pub token_mint: String,
    pub total_recipients: u32,
    pub total_amount: u64,
    pub recipients: Vec<AirdropRecipient>,
    pub merkle_root: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub claim_type: ClaimType,
    pub status: AirdropStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClaimType {
    Immediate,
    MerkleTree,
    Vested,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AirdropStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirdropRecipient {
    pub address: String,
    pub amount: u64,
    pub claimed: bool,
    pub claim_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketingAsset {
    pub id: String,
    pub launch_id: String,
    pub asset_type: AssetType,
    pub title: String,
    pub content: String,
    pub image_url: Option<String>,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub published: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Announcement,
    Banner,
    Tweet,
    Article,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSimulation {
    pub success: bool,
    pub compute_units: u64,
    pub fee_estimate: u64,
    pub logs: Vec<String>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchSafetyCheck {
    pub passed: bool,
    pub security_score: u8,
    pub risk_level: String,
    pub checks: Vec<SafetyCheckResult>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyCheckResult {
    pub check_name: String,
    pub passed: bool,
    pub severity: String,
    pub message: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionMetrics {
    pub token_mint: String,
    pub total_distributed: u64,
    pub total_recipients: u32,
    pub successful_transfers: u32,
    pub failed_transfers: u32,
    pub vesting_active: u32,
    pub vesting_completed: u32,
    pub liquidity_locked_amount: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenRequest {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub mint_authority_enabled: bool,
    pub freeze_authority_enabled: bool,
    pub metadata: TokenMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    pub description: String,
    pub image_url: Option<String>,
    pub website: Option<String>,
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTokenResponse {
    pub mint_address: String,
    pub transaction_signature: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockLiquidityRequest {
    pub token_mint: String,
    pub pool_address: String,
    pub amount: u64,
    pub duration_seconds: u64,
    pub beneficiary: String,
    pub is_revocable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVestingRequest {
    pub token_mint: String,
    pub beneficiary: String,
    pub total_amount: u64,
    pub start_date: DateTime<Utc>,
    pub cliff_duration_seconds: Option<u64>,
    pub vesting_duration_seconds: u64,
    pub vesting_type: VestingType,
    pub stages: Option<Vec<VestingStage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAirdropRequest {
    pub token_mint: String,
    pub recipients: Vec<AirdropRecipient>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub claim_type: ClaimType,
}
