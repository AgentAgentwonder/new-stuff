use super::types::*;
use crate::errors::AppError;
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

pub struct AirdropManager {
    airdrops: RwLock<HashMap<String, AirdropConfig>>, // id -> config
}

impl AirdropManager {
    pub fn new() -> Self {
        Self {
            airdrops: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_airdrop(&self, request: CreateAirdropRequest) -> Result<AirdropConfig, AppError> {
        self.validate_request(&request)?;

        let airdrop_id = Uuid::new_v4().to_string();
        let total_amount: u64 = request.recipients.iter().map(|r| r.amount).sum();
        let merkle_root = if request.claim_type == ClaimType::MerkleTree {
            Some(Self::generate_merkle_root(&request.recipients))
        } else {
            None
        };

        let airdrop = AirdropConfig {
            id: airdrop_id.clone(),
            token_mint: request.token_mint,
            total_recipients: request.recipients.len() as u32,
            total_amount,
            recipients: request.recipients,
            merkle_root,
            start_date: request.start_date,
            end_date: request.end_date,
            claim_type: request.claim_type,
            status: AirdropStatus::Pending,
            created_at: Utc::now(),
        };

        self.airdrops.write().insert(airdrop_id, airdrop.clone());

        Ok(airdrop)
    }

    pub fn activate_airdrop(&self, airdrop_id: &str) -> Result<AirdropConfig, AppError> {
        let mut airdrops = self.airdrops.write();
        let airdrop = airdrops
            .get_mut(airdrop_id)
            .ok_or_else(|| AppError::NotFound("Airdrop not found".to_string()))?;

        if airdrop.status != AirdropStatus::Pending {
            return Err(AppError::Validation(
                "Only pending airdrops can be activated".to_string(),
            ));
        }

        airdrop.status = AirdropStatus::Active;
        Ok(airdrop.clone())
    }

    pub fn claim_airdrop(
        &self,
        airdrop_id: &str,
        recipient_address: &str,
    ) -> Result<AirdropRecipient, AppError> {
        let mut airdrops = self.airdrops.write();
        let airdrop = airdrops
            .get_mut(airdrop_id)
            .ok_or_else(|| AppError::NotFound("Airdrop not found".to_string()))?;

        if airdrop.status != AirdropStatus::Active {
            return Err(AppError::Validation("Airdrop is not active".to_string()));
        }

        if let Some(end_date) = airdrop.end_date {
            if Utc::now() > end_date {
                return Err(AppError::Validation("Airdrop has ended".to_string()));
            }
        }

        let recipient = airdrop
            .recipients
            .iter_mut()
            .find(|r| r.address == recipient_address)
            .ok_or_else(|| AppError::NotFound("Recipient not found".to_string()))?;

        if recipient.claimed {
            return Err(AppError::Validation("Already claimed".to_string()));
        }

        recipient.claimed = true;
        recipient.claim_date = Some(Utc::now());

        Ok(recipient.clone())
    }

    pub fn cancel_airdrop(&self, airdrop_id: &str) -> Result<AirdropConfig, AppError> {
        let mut airdrops = self.airdrops.write();
        let airdrop = airdrops
            .get_mut(airdrop_id)
            .ok_or_else(|| AppError::NotFound("Airdrop not found".to_string()))?;

        airdrop.status = AirdropStatus::Cancelled;
        Ok(airdrop.clone())
    }

    pub fn get_airdrop(&self, airdrop_id: &str) -> Result<AirdropConfig, AppError> {
        self.airdrops
            .read()
            .get(airdrop_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Airdrop not found".to_string()))
    }

    pub fn get_airdrops_for_mint(&self, mint: &str) -> Vec<AirdropConfig> {
        self.airdrops
            .read()
            .values()
            .filter(|a| a.token_mint == mint)
            .cloned()
            .collect()
    }

    pub fn get_eligible_airdrop(&self, recipient_address: &str) -> Vec<(AirdropConfig, u64)> {
        self.airdrops
            .read()
            .values()
            .filter_map(|airdrop| {
                airdrop
                    .recipients
                    .iter()
                    .find(|r| r.address == recipient_address && !r.claimed)
                    .map(|r| (airdrop.clone(), r.amount))
            })
            .collect()
    }

    pub fn get_airdrop_metrics(&self, airdrop_id: &str) -> Result<AirdropMetrics, AppError> {
        let airdrops = self.airdrops.read();
        let airdrop = airdrops
            .get(airdrop_id)
            .ok_or_else(|| AppError::NotFound("Airdrop not found".to_string()))?;

        let claimed_count = airdrop.recipients.iter().filter(|r| r.claimed).count() as u32;
        let claimed_amount: u64 = airdrop
            .recipients
            .iter()
            .filter(|r| r.claimed)
            .map(|r| r.amount)
            .sum();

        Ok(AirdropMetrics {
            total_recipients: airdrop.total_recipients,
            claimed_recipients: claimed_count,
            unclaimed_recipients: airdrop.total_recipients - claimed_count,
            total_amount: airdrop.total_amount,
            claimed_amount,
            unclaimed_amount: airdrop.total_amount - claimed_amount,
        })
    }

    fn validate_request(&self, request: &CreateAirdropRequest) -> Result<(), AppError> {
        if request.recipients.is_empty() {
            return Err(AppError::Validation(
                "Recipients list cannot be empty".to_string(),
            ));
        }

        for recipient in &request.recipients {
            if recipient.amount == 0 {
                return Err(AppError::Validation(
                    "Recipient amount must be greater than 0".to_string(),
                ));
            }
        }

        if let Some(end_date) = request.end_date {
            if end_date <= request.start_date {
                return Err(AppError::Validation(
                    "End date must be after start date".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn generate_merkle_root(recipients: &[AirdropRecipient]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for recipient in recipients {
            hasher.update(recipient.address.as_bytes());
            hasher.update(recipient.amount.to_le_bytes());
        }
        let result = hasher.finalize();
        bs58::encode(result).into_string()
    }
}

impl Default for AirdropManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirdropMetrics {
    pub total_recipients: u32,
    pub claimed_recipients: u32,
    pub unclaimed_recipients: u32,
    pub total_amount: u64,
    pub claimed_amount: u64,
    pub unclaimed_amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airdrop_creation() {
        let manager = AirdropManager::new();

        let request = CreateAirdropRequest {
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            recipients: vec![
                AirdropRecipient {
                    address: "11111111111111111111111111111111".to_string(),
                    amount: 1000,
                    claimed: false,
                    claim_date: None,
                },
                AirdropRecipient {
                    address: "22222222222222222222222222222222".to_string(),
                    amount: 2000,
                    claimed: false,
                    claim_date: None,
                },
            ],
            start_date: Utc::now(),
            end_date: None,
            claim_type: ClaimType::Immediate,
        };

        let result = manager.create_airdrop(request);
        assert!(result.is_ok());

        let airdrop = result.unwrap();
        assert_eq!(airdrop.total_recipients, 2);
        assert_eq!(airdrop.total_amount, 3000);
    }
}
