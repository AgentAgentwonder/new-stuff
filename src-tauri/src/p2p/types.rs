use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OfferType {
    Buy,
    Sell,
}

impl std::fmt::Display for OfferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfferType::Buy => write!(f, "buy"),
            OfferType::Sell => write!(f, "sell"),
        }
    }
}

impl FromStr for OfferType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buy" => Ok(OfferType::Buy),
            "sell" => Ok(OfferType::Sell),
            _ => Err(anyhow::anyhow!("Invalid offer type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EscrowState {
    Created,
    Funded,
    Confirmed,
    Released,
    Disputed,
    Cancelled,
    Refunded,
    Completed,
}

impl std::fmt::Display for EscrowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EscrowState::Created => write!(f, "created"),
            EscrowState::Funded => write!(f, "funded"),
            EscrowState::Confirmed => write!(f, "confirmed"),
            EscrowState::Released => write!(f, "released"),
            EscrowState::Disputed => write!(f, "disputed"),
            EscrowState::Cancelled => write!(f, "cancelled"),
            EscrowState::Refunded => write!(f, "refunded"),
            EscrowState::Completed => write!(f, "completed"),
        }
    }
}

impl FromStr for EscrowState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "created" => Ok(EscrowState::Created),
            "funded" => Ok(EscrowState::Funded),
            "confirmed" => Ok(EscrowState::Confirmed),
            "released" => Ok(EscrowState::Released),
            "disputed" => Ok(EscrowState::Disputed),
            "cancelled" => Ok(EscrowState::Cancelled),
            "refunded" => Ok(EscrowState::Refunded),
            "completed" => Ok(EscrowState::Completed),
            _ => Err(anyhow::anyhow!("Invalid escrow state: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Evidence,
    Resolved,
    Escalated,
}

impl std::fmt::Display for DisputeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisputeStatus::Open => write!(f, "open"),
            DisputeStatus::UnderReview => write!(f, "under_review"),
            DisputeStatus::Evidence => write!(f, "evidence"),
            DisputeStatus::Resolved => write!(f, "resolved"),
            DisputeStatus::Escalated => write!(f, "escalated"),
        }
    }
}

impl FromStr for DisputeStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(DisputeStatus::Open),
            "under_review" => Ok(DisputeStatus::UnderReview),
            "evidence" => Ok(DisputeStatus::Evidence),
            "resolved" => Ok(DisputeStatus::Resolved),
            "escalated" => Ok(DisputeStatus::Escalated),
            _ => Err(anyhow::anyhow!("Invalid dispute status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2POffer {
    pub id: String,
    pub creator: String,
    pub offer_type: OfferType,
    pub token_address: String,
    pub token_symbol: String,
    pub amount: f64,
    pub price: f64,
    pub fiat_currency: String,
    pub payment_methods: Vec<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub terms: Option<String>,
    pub time_limit: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub completed_trades: i32,
    pub reputation_required: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Escrow {
    pub id: String,
    pub offer_id: String,
    pub buyer: String,
    pub seller: String,
    pub amount: f64,
    pub token_address: String,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub state: EscrowState,
    pub multisig_address: Option<String>,
    pub escrow_pubkey: Option<String>,
    pub created_at: DateTime<Utc>,
    pub funded_at: Option<DateTime<Utc>>,
    pub released_at: Option<DateTime<Utc>>,
    pub timeout_at: DateTime<Utc>,
    pub arbitrators: Vec<String>,
    pub fee_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dispute {
    pub id: String,
    pub escrow_id: String,
    pub filed_by: String,
    pub reason: String,
    pub description: String,
    pub evidence: Vec<DisputeEvidence>,
    pub status: DisputeStatus,
    pub arbitrator: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
    pub votes: Vec<DisputeVote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisputeEvidence {
    pub id: String,
    pub dispute_id: String,
    pub submitted_by: String,
    pub evidence_type: String,
    pub content: String,
    pub attachments: Vec<String>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisputeVote {
    pub id: String,
    pub dispute_id: String,
    pub voter: String,
    pub vote: String,
    pub comment: Option<String>,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub escrow_id: String,
    pub sender: String,
    pub recipient: String,
    pub message: String,
    pub encrypted: bool,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraderProfile {
    pub address: String,
    pub username: Option<String>,
    pub reputation_score: f64,
    pub total_trades: i32,
    pub successful_trades: i32,
    pub cancelled_trades: i32,
    pub disputed_trades: i32,
    pub avg_completion_time: i64,
    pub first_trade_at: Option<DateTime<Utc>>,
    pub last_trade_at: Option<DateTime<Utc>>,
    pub verified: bool,
    pub verification_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplianceCheck {
    pub passed: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub risk_level: String,
    pub checks_performed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOfferRequest {
    pub creator: String,
    pub offer_type: OfferType,
    pub token_address: String,
    pub token_symbol: String,
    pub amount: f64,
    pub price: f64,
    pub fiat_currency: String,
    pub payment_methods: Vec<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub terms: Option<String>,
    pub time_limit: i32,
    pub reputation_required: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEscrowRequest {
    pub offer_id: String,
    pub buyer: String,
    pub seller: String,
    pub amount: f64,
    pub fiat_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDisputeRequest {
    pub escrow_id: String,
    pub filed_by: String,
    pub reason: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitEvidenceRequest {
    pub dispute_id: String,
    pub submitted_by: String,
    pub evidence_type: String,
    pub content: String,
    pub attachments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub escrow_id: String,
    pub sender: String,
    pub recipient: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PStats {
    pub total_offers: i64,
    pub active_offers: i64,
    pub total_escrows: i64,
    pub active_escrows: i64,
    pub completed_escrows: i64,
    pub total_disputes: i64,
    pub open_disputes: i64,
    pub total_volume: f64,
    pub avg_completion_time: i64,
}
