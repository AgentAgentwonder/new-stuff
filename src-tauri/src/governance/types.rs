use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DAOPlatform {
    Realms,
    Tribeca,
    Squads,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    Draft,
    Active,
    Succeeded,
    Defeated,
    Queued,
    Executed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DAOMembership {
    pub dao_id: String,
    pub dao_name: String,
    pub dao_address: String,
    pub platform: DAOPlatform,
    pub wallet_address: String,
    pub governance_token: String,
    pub token_balance: f64,
    pub voting_power: f64,
    pub delegated_to: Option<String>,
    pub delegated_from: Vec<String>,
    pub joined_at: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceProposal {
    pub proposal_id: String,
    pub dao_id: String,
    pub dao_name: String,
    pub platform: DAOPlatform,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub status: ProposalStatus,
    pub created_at: i64,
    pub voting_starts_at: i64,
    pub voting_ends_at: i64,
    pub execution_eta: Option<i64>,
    pub yes_votes: f64,
    pub no_votes: f64,
    pub abstain_votes: f64,
    pub quorum_required: f64,
    pub threshold_percent: f64,
    pub instructions: Vec<ProposalInstruction>,
    pub discussion_url: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalInstruction {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub data: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteRecord {
    pub vote_id: String,
    pub proposal_id: String,
    pub voter: String,
    pub vote_choice: VoteChoice,
    pub voting_power: f64,
    pub timestamp: i64,
    pub transaction_signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegationRecord {
    pub delegation_id: String,
    pub dao_id: String,
    pub delegator: String,
    pub delegate: String,
    pub voting_power: f64,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalImpactAnalysis {
    pub proposal_id: String,
    pub historical_similarity_score: f64,
    pub similar_proposals: Vec<SimilarProposal>,
    pub predicted_outcome: ProposalStatus,
    pub confidence: f64,
    pub risk_factors: Vec<String>,
    pub potential_impacts: HashMap<String, String>,
    pub recommended_vote: Option<VoteChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarProposal {
    pub proposal_id: String,
    pub title: String,
    pub similarity_score: f64,
    pub outcome: ProposalStatus,
    pub final_yes_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalReminder {
    pub reminder_id: String,
    pub proposal_id: String,
    pub wallet_address: String,
    pub remind_at: i64,
    pub notification_sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSignatureRequest {
    pub proposal_id: String,
    pub vote_choice: VoteChoice,
    pub wallet_address: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoteSignatureResponse {
    pub signature: String,
    pub public_key: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceSummary {
    pub total_daos: usize,
    pub active_memberships: usize,
    pub total_voting_power: f64,
    pub active_proposals: usize,
    pub pending_votes: usize,
    pub upcoming_deadlines: Vec<UpcomingDeadline>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpcomingDeadline {
    pub proposal_id: String,
    pub dao_name: String,
    pub title: String,
    pub ends_at: i64,
    pub time_remaining_hours: i64,
    pub has_voted: bool,
}
