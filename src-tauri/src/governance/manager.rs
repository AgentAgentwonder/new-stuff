use super::types::*;
use crate::errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedGovernanceManager = Arc<RwLock<GovernanceManager>>;

pub struct GovernanceManager {
    memberships: HashMap<String, Vec<DAOMembership>>,
    proposals: HashMap<String, Vec<GovernanceProposal>>,
    votes: HashMap<String, VoteRecord>,
    delegations: HashMap<String, Vec<DelegationRecord>>,
    reminders: HashMap<String, Vec<ProposalReminder>>,
}

impl GovernanceManager {
    pub fn new() -> Self {
        Self {
            memberships: HashMap::new(),
            proposals: HashMap::new(),
            votes: HashMap::new(),
            delegations: HashMap::new(),
            reminders: HashMap::new(),
        }
    }

    pub async fn sync_memberships(
        &mut self,
        wallet_address: &str,
    ) -> Result<Vec<DAOMembership>, AppError> {
        let memberships = self.fetch_dao_memberships(wallet_address).await?;
        self.memberships
            .insert(wallet_address.to_string(), memberships.clone());
        Ok(memberships)
    }

    pub async fn get_memberships(&self, wallet_address: &str) -> Vec<DAOMembership> {
        self.memberships
            .get(wallet_address)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn sync_proposals(
        &mut self,
        dao_id: &str,
    ) -> Result<Vec<GovernanceProposal>, AppError> {
        let proposals = self.fetch_dao_proposals(dao_id).await?;
        self.proposals.insert(dao_id.to_string(), proposals.clone());
        Ok(proposals)
    }

    pub async fn get_proposals(&self, dao_id: &str) -> Vec<GovernanceProposal> {
        self.proposals.get(dao_id).cloned().unwrap_or_default()
    }

    pub async fn get_all_active_proposals(&self, wallet_address: &str) -> Vec<GovernanceProposal> {
        let memberships = self.get_memberships(wallet_address).await;
        let mut all_proposals = Vec::new();

        for membership in memberships {
            if let Some(proposals) = self.proposals.get(&membership.dao_id) {
                all_proposals.extend(
                    proposals
                        .iter()
                        .filter(|p| p.status == ProposalStatus::Active)
                        .cloned(),
                );
            }
        }

        all_proposals
    }

    pub async fn get_voting_power(&self, wallet_address: &str, dao_id: &str) -> f64 {
        let memberships = self.get_memberships(wallet_address).await;
        memberships
            .iter()
            .find(|m| m.dao_id == dao_id)
            .map(|m| m.voting_power)
            .unwrap_or(0.0)
    }

    pub async fn submit_vote(
        &mut self,
        proposal_id: String,
        voter: String,
        vote_choice: VoteChoice,
        voting_power: f64,
        signature: String,
    ) -> Result<VoteRecord, AppError> {
        let vote = VoteRecord {
            vote_id: uuid::Uuid::new_v4().to_string(),
            proposal_id: proposal_id.clone(),
            voter: voter.clone(),
            vote_choice: vote_choice.clone(),
            voting_power,
            timestamp: chrono::Utc::now().timestamp(),
            transaction_signature: Some(signature),
        };

        self.votes.insert(vote.vote_id.clone(), vote.clone());

        if let Some(dao_proposals) = self
            .proposals
            .values_mut()
            .find(|proposals| proposals.iter().any(|p| p.proposal_id == proposal_id))
        {
            if let Some(proposal) = dao_proposals
                .iter_mut()
                .find(|p| p.proposal_id == proposal_id)
            {
                match vote_choice {
                    VoteChoice::Yes => proposal.yes_votes += voting_power,
                    VoteChoice::No => proposal.no_votes += voting_power,
                    VoteChoice::Abstain => proposal.abstain_votes += voting_power,
                }
            }
        }

        Ok(vote)
    }

    pub async fn delegate_votes(
        &mut self,
        dao_id: String,
        delegator: String,
        delegate: String,
        voting_power: f64,
        expires_at: Option<i64>,
    ) -> Result<DelegationRecord, AppError> {
        let delegation = DelegationRecord {
            delegation_id: uuid::Uuid::new_v4().to_string(),
            dao_id: dao_id.clone(),
            delegator: delegator.clone(),
            delegate: delegate.clone(),
            voting_power,
            created_at: chrono::Utc::now().timestamp(),
            expires_at,
            is_active: true,
        };

        self.delegations
            .entry(delegator.clone())
            .or_default()
            .push(delegation.clone());

        Ok(delegation)
    }

    pub async fn revoke_delegation(
        &mut self,
        delegation_id: &str,
        wallet_address: &str,
    ) -> Result<(), AppError> {
        if let Some(delegations) = self.delegations.get_mut(wallet_address) {
            if let Some(delegation) = delegations
                .iter_mut()
                .find(|d| d.delegation_id == delegation_id)
            {
                delegation.is_active = false;
            }
        }
        Ok(())
    }

    pub async fn get_delegations(&self, wallet_address: &str) -> Vec<DelegationRecord> {
        self.delegations
            .get(wallet_address)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn analyze_proposal_impact(
        &self,
        proposal_id: &str,
    ) -> Result<ProposalImpactAnalysis, AppError> {
        let proposals_flat: Vec<&GovernanceProposal> = self.proposals.values().flatten().collect();

        let current_proposal = proposals_flat
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .ok_or_else(|| AppError::NotFound("Proposal not found".to_string()))?;

        let similar_proposals = self.find_similar_proposals(current_proposal, &proposals_flat);

        let succeeded_count = similar_proposals
            .iter()
            .filter(|sp| sp.outcome == ProposalStatus::Succeeded)
            .count();

        let confidence = if similar_proposals.is_empty() {
            0.5
        } else {
            succeeded_count as f64 / similar_proposals.len() as f64
        };

        let predicted_outcome = if confidence > 0.6 {
            ProposalStatus::Succeeded
        } else {
            ProposalStatus::Defeated
        };

        let recommended_vote = if confidence > 0.7 {
            Some(VoteChoice::Yes)
        } else if confidence < 0.3 {
            Some(VoteChoice::No)
        } else {
            None
        };

        let mut risk_factors = Vec::new();
        if current_proposal.quorum_required > 0.0 {
            let total_votes = current_proposal.yes_votes
                + current_proposal.no_votes
                + current_proposal.abstain_votes;
            let participation = (total_votes / current_proposal.quorum_required) * 100.0;
            if participation < 50.0 {
                risk_factors.push(format!("Low participation rate: {:.1}%", participation));
            }
        }

        if current_proposal.instructions.len() > 5 {
            risk_factors.push("Complex proposal with multiple instructions".to_string());
        }

        let mut potential_impacts = HashMap::new();
        potential_impacts.insert(
            "treasury".to_string(),
            "May affect DAO treasury balance".to_string(),
        );
        potential_impacts.insert(
            "governance".to_string(),
            "Changes to governance parameters".to_string(),
        );

        Ok(ProposalImpactAnalysis {
            proposal_id: proposal_id.to_string(),
            historical_similarity_score: if similar_proposals.is_empty() {
                0.0
            } else {
                similar_proposals
                    .iter()
                    .map(|sp| sp.similarity_score)
                    .sum::<f64>()
                    / similar_proposals.len() as f64
            },
            similar_proposals,
            predicted_outcome,
            confidence,
            risk_factors,
            potential_impacts,
            recommended_vote,
        })
    }

    pub async fn create_reminder(
        &mut self,
        proposal_id: String,
        wallet_address: String,
        remind_at: i64,
    ) -> Result<ProposalReminder, AppError> {
        let reminder = ProposalReminder {
            reminder_id: uuid::Uuid::new_v4().to_string(),
            proposal_id: proposal_id.clone(),
            wallet_address: wallet_address.clone(),
            remind_at,
            notification_sent: false,
        };

        self.reminders
            .entry(wallet_address)
            .or_default()
            .push(reminder.clone());

        Ok(reminder)
    }

    pub async fn get_upcoming_deadlines(&self, wallet_address: &str) -> Vec<UpcomingDeadline> {
        let memberships = self.get_memberships(wallet_address).await;
        let mut deadlines = Vec::new();

        for membership in memberships {
            if let Some(proposals) = self.proposals.get(&membership.dao_id) {
                for proposal in proposals {
                    if proposal.status == ProposalStatus::Active {
                        let now = chrono::Utc::now().timestamp();
                        let time_remaining = (proposal.voting_ends_at - now) / 3600;

                        if time_remaining > 0 {
                            let has_voted = self.votes.values().any(|v| {
                                v.proposal_id == proposal.proposal_id && v.voter == wallet_address
                            });

                            deadlines.push(UpcomingDeadline {
                                proposal_id: proposal.proposal_id.clone(),
                                dao_name: proposal.dao_name.clone(),
                                title: proposal.title.clone(),
                                ends_at: proposal.voting_ends_at,
                                time_remaining_hours: time_remaining,
                                has_voted,
                            });
                        }
                    }
                }
            }
        }

        deadlines.sort_by_key(|d| d.ends_at);
        deadlines
    }

    pub async fn get_governance_summary(&self, wallet_address: &str) -> GovernanceSummary {
        let memberships = self.get_memberships(wallet_address).await;
        let active_memberships = memberships.iter().filter(|m| m.is_active).count();
        let total_voting_power: f64 = memberships.iter().map(|m| m.voting_power).sum();

        let active_proposals = self.get_all_active_proposals(wallet_address).await.len();

        let upcoming_deadlines = self.get_upcoming_deadlines(wallet_address).await;
        let pending_votes = upcoming_deadlines.iter().filter(|d| !d.has_voted).count();

        GovernanceSummary {
            total_daos: memberships.len(),
            active_memberships,
            total_voting_power,
            active_proposals,
            pending_votes,
            upcoming_deadlines: upcoming_deadlines.into_iter().take(5).collect(),
        }
    }

    async fn fetch_dao_memberships(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<DAOMembership>, AppError> {
        let mock_memberships = vec![
            DAOMembership {
                dao_id: "realms-marinade-dao".to_string(),
                dao_name: "Marinade DAO".to_string(),
                dao_address: "GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw".to_string(),
                platform: DAOPlatform::Realms,
                wallet_address: wallet_address.to_string(),
                governance_token: "MNDEFzGvMt87ueuHvVU9VcTqsAP5b3fTGPsHuuPA5ey".to_string(),
                token_balance: 15000.0,
                voting_power: 15000.0,
                delegated_to: None,
                delegated_from: vec![],
                joined_at: chrono::Utc::now().timestamp() - 2592000,
                is_active: true,
            },
            DAOMembership {
                dao_id: "realms-mango-dao".to_string(),
                dao_name: "Mango DAO".to_string(),
                dao_address: "DPiH3H3c7t47BMxqTxLsuPQpEC6Kne8GA9VXbxpnZxFE".to_string(),
                platform: DAOPlatform::Realms,
                wallet_address: wallet_address.to_string(),
                governance_token: "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac".to_string(),
                token_balance: 8500.0,
                voting_power: 8500.0,
                delegated_to: None,
                delegated_from: vec![],
                joined_at: chrono::Utc::now().timestamp() - 1296000,
                is_active: true,
            },
        ];

        Ok(mock_memberships)
    }

    async fn fetch_dao_proposals(&self, dao_id: &str) -> Result<Vec<GovernanceProposal>, AppError> {
        let now = chrono::Utc::now().timestamp();

        let mock_proposals = match dao_id {
            "realms-marinade-dao" => vec![
                GovernanceProposal {
                    proposal_id: "marinade-prop-12".to_string(),
                    dao_id: dao_id.to_string(),
                    dao_name: "Marinade DAO".to_string(),
                    platform: DAOPlatform::Realms,
                    title: "Update Validator Onboarding Criteria".to_string(),
                    description: "Proposal to update the criteria for new validators joining the stake pool to ensure higher quality and performance standards.".to_string(),
                    proposer: "marinade_team".to_string(),
                    status: ProposalStatus::Active,
                    created_at: now - 86400,
                    voting_starts_at: now - 43200,
                    voting_ends_at: now + 172800,
                    execution_eta: Some(now + 259200),
                    yes_votes: 45000.0,
                    no_votes: 12000.0,
                    abstain_votes: 3000.0,
                    quorum_required: 100000.0,
                    threshold_percent: 60.0,
                    instructions: vec![
                        ProposalInstruction {
                            program_id: "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD".to_string(),
                            accounts: vec!["validator_registry".to_string()],
                            data: "update_criteria".to_string(),
                            description: "Update validator registry criteria".to_string(),
                        },
                    ],
                    discussion_url: Some("https://forum.marinade.finance/t/proposal-12".to_string()),
                    tags: vec!["validators".to_string(), "governance".to_string()],
                },
            ],
            "realms-mango-dao" => vec![
                GovernanceProposal {
                    proposal_id: "mango-prop-8".to_string(),
                    dao_id: dao_id.to_string(),
                    dao_name: "Mango DAO".to_string(),
                    platform: DAOPlatform::Realms,
                    title: "Treasury Diversification Strategy".to_string(),
                    description: "Allocate 20% of treasury funds to diversified DeFi protocols to generate yield while maintaining safety.".to_string(),
                    proposer: "mango_council".to_string(),
                    status: ProposalStatus::Active,
                    created_at: now - 129600,
                    voting_starts_at: now - 86400,
                    voting_ends_at: now + 86400,
                    execution_eta: Some(now + 259200),
                    yes_votes: 75000.0,
                    no_votes: 25000.0,
                    abstain_votes: 5000.0,
                    quorum_required: 150000.0,
                    threshold_percent: 66.0,
                    instructions: vec![
                        ProposalInstruction {
                            program_id: "4MangoMjqJ2firMokCjjGgoK8d4MXcrgL7XJaL3w6fVg".to_string(),
                            accounts: vec!["treasury".to_string(), "target_protocol".to_string()],
                            data: "diversify_treasury".to_string(),
                            description: "Transfer funds from treasury to diversification targets".to_string(),
                        },
                    ],
                    discussion_url: Some("https://dao.mango.markets/dao/MNGO/proposal/8".to_string()),
                    tags: vec!["treasury".to_string(), "defi".to_string()],
                },
            ],
            _ => vec![],
        };

        Ok(mock_proposals)
    }

    fn find_similar_proposals(
        &self,
        target: &GovernanceProposal,
        all_proposals: &[&GovernanceProposal],
    ) -> Vec<SimilarProposal> {
        let mut similar = Vec::new();

        for proposal in all_proposals {
            if proposal.proposal_id == target.proposal_id {
                continue;
            }

            let mut similarity_score = 0.0;

            if proposal.dao_id == target.dao_id {
                similarity_score += 0.3;
            }

            let common_tags = target
                .tags
                .iter()
                .filter(|t| proposal.tags.contains(t))
                .count();
            similarity_score += (common_tags as f64 / target.tags.len().max(1) as f64) * 0.4;

            if proposal
                .title
                .to_lowercase()
                .split_whitespace()
                .any(|word| {
                    target
                        .title
                        .to_lowercase()
                        .split_whitespace()
                        .any(|tw| tw == word)
                })
            {
                similarity_score += 0.3;
            }

            if similarity_score > 0.4 {
                let total_votes = proposal.yes_votes + proposal.no_votes + proposal.abstain_votes;
                let final_yes_percent = if total_votes > 0.0 {
                    (proposal.yes_votes / total_votes) * 100.0
                } else {
                    0.0
                };

                similar.push(SimilarProposal {
                    proposal_id: proposal.proposal_id.clone(),
                    title: proposal.title.clone(),
                    similarity_score,
                    outcome: proposal.status.clone(),
                    final_yes_percent,
                });
            }
        }

        similar.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        similar.into_iter().take(5).collect()
    }

    pub fn find_dao_id_by_proposal(&self, proposal_id: &str) -> Option<String> {
        for (dao_id, proposals) in &self.proposals {
            if proposals
                .iter()
                .any(|proposal| proposal.proposal_id == proposal_id)
            {
                return Some(dao_id.clone());
            }
        }
        None
    }
}

impl Default for GovernanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_memberships() {
        let mut manager = GovernanceManager::new();
        let wallet = "test-wallet";

        let memberships = manager.sync_memberships(wallet).await.unwrap();
        assert_eq!(memberships.len(), 2);

        let stored = manager.get_memberships(wallet).await;
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].wallet_address, wallet);
    }

    #[tokio::test]
    async fn test_sync_proposals_and_summary() {
        let mut manager = GovernanceManager::new();
        let wallet = "test-wallet";

        manager.sync_memberships(wallet).await.unwrap();

        for membership in manager.get_memberships(wallet).await {
            manager.sync_proposals(&membership.dao_id).await.unwrap();
        }

        let summary = manager.get_governance_summary(wallet).await;
        assert_eq!(summary.total_daos, 2);
        assert!(summary.total_voting_power > 0.0);
        assert!(summary.active_proposals > 0);
    }

    #[tokio::test]
    async fn test_delegation_flow() {
        let mut manager = GovernanceManager::new();
        let wallet = "delegator-wallet";
        manager.sync_memberships(wallet).await.unwrap();

        let delegation = manager
            .delegate_votes(
                "realms-marinade-dao".to_string(),
                wallet.to_string(),
                "delegate-wallet".to_string(),
                5000.0,
                None,
            )
            .await
            .unwrap();

        assert_eq!(delegation.delegator, wallet);

        let delegations = manager.get_delegations(wallet).await;
        assert_eq!(delegations.len(), 1);

        manager
            .revoke_delegation(&delegation.delegation_id, wallet)
            .await
            .unwrap();

        let delegations = manager.get_delegations(wallet).await;
        assert!(!delegations[0].is_active);
    }
}
