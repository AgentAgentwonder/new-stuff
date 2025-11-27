use super::{manager::SharedGovernanceManager, signature, types::*};
use crate::errors::AppError;
use tauri::State;

#[tauri::command]
pub async fn sync_governance_memberships(
    wallet_address: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<Vec<DAOMembership>, String> {
    let mut guard = manager.write().await;
    guard
        .sync_memberships(&wallet_address)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_governance_memberships(
    wallet_address: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<Vec<DAOMembership>, String> {
    let guard = manager.read().await;
    Ok(guard.get_memberships(&wallet_address).await)
}

#[tauri::command]
pub async fn sync_governance_proposals(
    dao_id: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<Vec<GovernanceProposal>, String> {
    let mut guard = manager.write().await;
    guard
        .sync_proposals(&dao_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_dao_governance_proposals(
    dao_id: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<Vec<GovernanceProposal>, String> {
    let guard = manager.read().await;
    Ok(guard.get_proposals(&dao_id).await)
}

#[tauri::command]
pub async fn get_all_active_governance_proposals(
    wallet_address: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<Vec<GovernanceProposal>, String> {
    let guard = manager.read().await;
    Ok(guard.get_all_active_proposals(&wallet_address).await)
}

#[tauri::command]
pub async fn get_wallet_voting_power(
    wallet_address: String,
    dao_id: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<f64, String> {
    let guard = manager.read().await;
    Ok(guard.get_voting_power(&wallet_address, &dao_id).await)
}

#[tauri::command]
pub async fn submit_signed_vote(
    proposal_id: String,
    wallet_address: String,
    vote_choice: VoteChoice,
    signature: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<VoteRecord, String> {
    let voting_power = {
        let guard = manager.read().await;
        guard.get_voting_power(&wallet_address, &proposal_id).await
    };

    let mut guard = manager.write().await;
    guard
        .submit_vote(
            proposal_id,
            wallet_address,
            vote_choice,
            voting_power,
            signature,
        )
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn delegate_governance_votes(
    dao_id: String,
    delegator: String,
    delegate: String,
    voting_power: f64,
    expires_at: Option<i64>,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<DelegationRecord, String> {
    let mut guard = manager.write().await;
    guard
        .delegate_votes(dao_id, delegator, delegate, voting_power, expires_at)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn revoke_governance_delegation(
    delegation_id: String,
    wallet_address: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<(), String> {
    let mut guard = manager.write().await;
    guard
        .revoke_delegation(&delegation_id, &wallet_address)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_governance_delegations(
    wallet_address: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<Vec<DelegationRecord>, String> {
    let guard = manager.read().await;
    Ok(guard.get_delegations(&wallet_address).await)
}

#[tauri::command]
pub async fn analyze_governance_proposal(
    proposal_id: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<ProposalImpactAnalysis, String> {
    let guard = manager.read().await;
    guard
        .analyze_proposal_impact(&proposal_id)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn create_governance_reminder(
    proposal_id: String,
    wallet_address: String,
    remind_at: i64,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<ProposalReminder, String> {
    let mut guard = manager.write().await;
    guard
        .create_reminder(proposal_id, wallet_address, remind_at)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn get_governance_summary(
    wallet_address: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<GovernanceSummary, String> {
    let guard = manager.read().await;
    Ok(guard.get_governance_summary(&wallet_address).await)
}

#[tauri::command]
pub async fn get_governance_deadlines(
    wallet_address: String,
    manager: State<'_, SharedGovernanceManager>,
) -> Result<Vec<UpcomingDeadline>, String> {
    let guard = manager.read().await;
    Ok(guard.get_upcoming_deadlines(&wallet_address).await)
}

#[tauri::command]
pub async fn prepare_vote_signature(
    proposal_id: String,
    vote_choice: VoteChoice,
    wallet_address: String,
) -> Result<VoteSignatureRequest, String> {
    let timestamp = chrono::Utc::now().timestamp();
    let message =
        signature::create_vote_message(&proposal_id, &vote_choice, &wallet_address, timestamp);

    Ok(VoteSignatureRequest {
        proposal_id,
        vote_choice,
        wallet_address,
        message,
        timestamp,
    })
}

#[tauri::command]
pub async fn verify_vote_signature(
    request: VoteSignatureRequest,
    response: VoteSignatureResponse,
) -> Result<bool, String> {
    signature::verify_vote_signature(&request, &response).map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn prepare_vote_transaction(
    proposal_id: String,
    vote_choice: VoteChoice,
    voting_power: f64,
) -> Result<serde_json::Value, String> {
    signature::prepare_transaction_data(&proposal_id, &vote_choice, voting_power)
        .map_err(|err| err.to_string())
}
