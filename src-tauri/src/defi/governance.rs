use crate::defi::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceVote {
    pub proposal_id: String,
    pub wallet: String,
    pub choice: VoteChoice,
    pub voting_power: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

#[derive(Clone)]
pub struct GovernanceHub {
    proposals: Arc<RwLock<Vec<GovernanceProposal>>>,
    votes: Arc<RwLock<HashMap<String, Vec<GovernanceVote>>>>,
}

impl Default for GovernanceHub {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernanceHub {
    pub fn new() -> Self {
        let proposals = Arc::new(RwLock::new(Self::generate_mock_proposals()));
        let votes = Arc::new(RwLock::new(HashMap::new()));
        Self { proposals, votes }
    }

    pub async fn list_proposals(&self) -> Vec<GovernanceProposal> {
        let guard = self.proposals.read().await;
        guard.clone()
    }

    pub async fn get_proposal(&self, id: &str) -> Option<GovernanceProposal> {
        let guard = self.proposals.read().await;
        guard.iter().find(|proposal| proposal.id == id).cloned()
    }

    pub async fn submit_vote(
        &self,
        proposal_id: &str,
        wallet: &str,
        choice: VoteChoice,
        voting_power: f64,
    ) -> Result<GovernanceProposal, String> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals
            .iter_mut()
            .find(|p| p.id == proposal_id)
            .ok_or_else(|| "Proposal not found".to_string())?;

        let vote = GovernanceVote {
            proposal_id: proposal_id.to_string(),
            wallet: wallet.to_string(),
            choice: choice.clone(),
            voting_power,
            timestamp: chrono::Utc::now().timestamp(),
        };

        {
            let mut votes = self.votes.write().await;
            let entry = votes.entry(proposal_id.to_string()).or_default();
            entry.push(vote);
        }

        match choice {
            VoteChoice::For => proposal.votes_for += voting_power,
            VoteChoice::Against => proposal.votes_against += voting_power,
            VoteChoice::Abstain => proposal.votes_abstain += voting_power,
        }

        Ok(proposal.clone())
    }

    pub async fn get_votes(&self, proposal_id: &str) -> Vec<GovernanceVote> {
        let votes = self.votes.read().await;
        votes.get(proposal_id).cloned().unwrap_or_else(Vec::new)
    }

    pub async fn get_participation_rate(&self, proposal_id: &str) -> f64 {
        let proposal = match self.get_proposal(proposal_id).await {
            Some(p) => p,
            None => return 0.0,
        };
        let total_votes = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;
        if proposal.quorum <= 0.0 {
            0.0
        } else {
            total_votes / proposal.quorum * 100.0
        }
    }

    fn generate_mock_proposals() -> Vec<GovernanceProposal> {
        vec![
            GovernanceProposal {
                id: "solend-proposal-1".to_string(),
                protocol: Protocol::Solend,
                title: "Adjust USDC Borrow Rates".to_string(),
                description: "Proposal to adjust the borrow rates for the USDC pool to maintain optimal utilization.".to_string(),
                proposer: "Solend DAO".to_string(),
                start_time: chrono::Utc::now().timestamp() - 3600,
                end_time: chrono::Utc::now().timestamp() + 86400,
                votes_for: 1_250_000.0,
                votes_against: 320_000.0,
                votes_abstain: 50_000.0,
                status: ProposalStatus::Active,
                quorum: 2_000_000.0,
                execution_eta: Some(chrono::Utc::now().timestamp() + 172800),
            },
            GovernanceProposal {
                id: "marginfi-proposal-7".to_string(),
                protocol: Protocol::MarginFi,
                title: "Launch New Risk Tier".to_string(),
                description: "Introduce a new risk tier for upcoming volatile assets with lower collateral factors.".to_string(),
                proposer: "MarginFi Core".to_string(),
                start_time: chrono::Utc::now().timestamp() - 10800,
                end_time: chrono::Utc::now().timestamp() + 43200,
                votes_for: 850_000.0,
                votes_against: 120_000.0,
                votes_abstain: 40_000.0,
                status: ProposalStatus::Active,
                quorum: 1_500_000.0,
                execution_eta: None,
            },
            GovernanceProposal {
                id: "kamino-proposal-3".to_string(),
                protocol: Protocol::Kamino,
                title: "Enable Auto-Compounder".to_string(),
                description: "Activate automated compounding for the SOL-USDC vault with dynamic fees.".to_string(),
                proposer: "Kamino Council".to_string(),
                start_time: chrono::Utc::now().timestamp() - 7200,
                end_time: chrono::Utc::now().timestamp() + 21600,
                votes_for: 420_000.0,
                votes_against: 35_000.0,
                votes_abstain: 12_000.0,
                status: ProposalStatus::Active,
                quorum: 600_000.0,
                execution_eta: None,
            },
        ]
    }
}

#[tauri::command]
pub async fn get_governance_proposals() -> Result<Vec<GovernanceProposal>, String> {
    Ok(GovernanceHub::new().list_proposals().await)
}

#[tauri::command]
pub async fn vote_on_proposal(
    proposal_id: String,
    wallet: String,
    choice: VoteChoice,
    voting_power: f64,
) -> Result<GovernanceProposal, String> {
    GovernanceHub::new()
        .submit_vote(&proposal_id, &wallet, choice, voting_power)
        .await
}

#[tauri::command]
pub async fn get_governance_participation(proposal_id: String) -> Result<f64, String> {
    Ok(GovernanceHub::new()
        .get_participation_rate(&proposal_id)
        .await)
}
