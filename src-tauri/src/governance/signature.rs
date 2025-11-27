use super::types::{VoteChoice, VoteSignatureRequest, VoteSignatureResponse};
use crate::errors::AppError;
use serde_json::json;

pub fn create_vote_message(
    proposal_id: &str,
    vote_choice: &VoteChoice,
    voter: &str,
    timestamp: i64,
) -> String {
    let choice_str = match vote_choice {
        VoteChoice::Yes => "yes",
        VoteChoice::No => "no",
        VoteChoice::Abstain => "abstain",
    };

    format!(
        "Vote on Proposal\n\nProposal ID: {}\nVote: {}\nVoter: {}\nTimestamp: {}",
        proposal_id, choice_str, voter, timestamp
    )
}

pub fn verify_vote_signature(
    request: &VoteSignatureRequest,
    response: &VoteSignatureResponse,
) -> Result<bool, AppError> {
    if response.public_key != request.wallet_address {
        return Ok(false);
    }

    let time_diff = (response.timestamp - request.timestamp).abs();
    if time_diff > 300 {
        return Ok(false);
    }

    if response.signature.is_empty() {
        return Ok(false);
    }

    Ok(true)
}

pub fn prepare_transaction_data(
    proposal_id: &str,
    vote_choice: &VoteChoice,
    voting_power: f64,
) -> Result<serde_json::Value, AppError> {
    Ok(json!({
        "proposalId": proposal_id,
        "voteChoice": match vote_choice {
            VoteChoice::Yes => "yes",
            VoteChoice::No => "no",
            VoteChoice::Abstain => "abstain",
        },
        "votingPower": voting_power,
        "timestamp": chrono::Utc::now().timestamp(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vote_message() {
        let message = create_vote_message("prop-123", &VoteChoice::Yes, "wallet-456", 1234567890);
        assert!(message.contains("Proposal ID: prop-123"));
        assert!(message.contains("Vote: yes"));
        assert!(message.contains("Voter: wallet-456"));
    }

    #[test]
    fn test_verify_vote_signature_valid() {
        let now = chrono::Utc::now().timestamp();
        let request = VoteSignatureRequest {
            proposal_id: "prop-123".to_string(),
            vote_choice: VoteChoice::Yes,
            wallet_address: "wallet-addr".to_string(),
            message: "test message".to_string(),
            timestamp: now,
        };

        let response = VoteSignatureResponse {
            signature: "valid-signature".to_string(),
            public_key: "wallet-addr".to_string(),
            timestamp: now + 10,
        };

        let result = verify_vote_signature(&request, &response);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_vote_signature_wrong_wallet() {
        let now = chrono::Utc::now().timestamp();
        let request = VoteSignatureRequest {
            proposal_id: "prop-123".to_string(),
            vote_choice: VoteChoice::Yes,
            wallet_address: "wallet-addr".to_string(),
            message: "test message".to_string(),
            timestamp: now,
        };

        let response = VoteSignatureResponse {
            signature: "valid-signature".to_string(),
            public_key: "different-wallet".to_string(),
            timestamp: now + 10,
        };

        let result = verify_vote_signature(&request, &response);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_vote_signature_expired() {
        let now = chrono::Utc::now().timestamp();
        let request = VoteSignatureRequest {
            proposal_id: "prop-123".to_string(),
            vote_choice: VoteChoice::Yes,
            wallet_address: "wallet-addr".to_string(),
            message: "test message".to_string(),
            timestamp: now,
        };

        let response = VoteSignatureResponse {
            signature: "valid-signature".to_string(),
            public_key: "wallet-addr".to_string(),
            timestamp: now + 400,
        };

        let result = verify_vote_signature(&request, &response);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
