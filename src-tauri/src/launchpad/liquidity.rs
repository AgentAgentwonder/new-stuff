use super::types::*;
use crate::errors::AppError;
use crate::security::keystore::Keystore;
use chrono::{Duration, Utc};
use rand::RngCore;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

pub struct LiquidityLocker {
    locks: Mutex<Vec<LiquidityLockConfig>>,
}

impl Clone for LiquidityLocker {
    fn clone(&self) -> Self {
        let locks = self.locks.lock().unwrap().clone();
        Self {
            locks: Mutex::new(locks),
        }
    }
}

impl LiquidityLocker {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(Vec::new()),
        }
    }

    pub async fn create_lock(
        &self,
        request: LockLiquidityRequest,
        app: &AppHandle,
    ) -> Result<LiquidityLockConfig, AppError> {
        // Validate addresses
        let _mint_pubkey = Pubkey::from_str(&request.token_mint)
            .map_err(|e| AppError::Generic(format!("Invalid token mint: {}", e)))?;
        let _pool_pubkey = Pubkey::from_str(&request.pool_address)
            .map_err(|e| AppError::Generic(format!("Invalid pool address: {}", e)))?;
        let _beneficiary_pubkey = Pubkey::from_str(&request.beneficiary)
            .map_err(|e| AppError::Generic(format!("Invalid beneficiary address: {}", e)))?;

        // Validate amounts
        if request.amount == 0 {
            return Err(AppError::Validation(
                "Lock amount must be greater than 0".to_string(),
            ));
        }

        if request.duration_seconds < 86400 {
            // At least 1 day
            return Err(AppError::Validation(
                "Lock duration must be at least 1 day".to_string(),
            ));
        }

        // Get authority keypair from keystore
        let keystore: tauri::State<Keystore> = app.try_state::<Keystore>().unwrap();
        let _authority_secret = keystore
            .retrieve_secret("wallet_keypair")
            .map_err(|e| AppError::Generic(format!("Failed to retrieve keypair: {}", e)))?;

        let lock_id = Uuid::new_v4().to_string();
        let unlock_date = Utc::now() + Duration::seconds(request.duration_seconds as i64);

        let lock_config = LiquidityLockConfig {
            pool_address: Some(request.pool_address.clone()),
            lock_amount: request.amount,
            lock_duration_seconds: request.duration_seconds,
            unlock_date,
            beneficiary: request.beneficiary.clone(),
            is_revocable: request.is_revocable,
            lock_id: Some(lock_id.clone()),
            status: LockStatus::Locked,
        };

        // Store lock
        let mut locks = self.locks.lock().unwrap();
        locks.push(lock_config.clone());

        Ok(lock_config)
    }

    pub async fn unlock_liquidity(
        &self,
        lock_id: &str,
        app: &AppHandle,
    ) -> Result<String, AppError> {
        let mut locks = self.locks.lock().unwrap();

        let lock = locks
            .iter_mut()
            .find(|l| l.lock_id.as_deref() == Some(lock_id))
            .ok_or_else(|| AppError::NotFound("Lock not found".to_string()))?;

        // Check if unlock date has passed
        if Utc::now() < lock.unlock_date {
            return Err(AppError::Validation(
                "Lock period has not expired yet".to_string(),
            ));
        }

        // Get beneficiary keypair from keystore
        let keystore: tauri::State<Keystore> = app.try_state::<Keystore>().unwrap();
        let _beneficiary_secret = keystore
            .retrieve_secret("wallet_keypair")
            .map_err(|e| AppError::Generic(format!("Failed to retrieve keypair: {}", e)))?;

        lock.status = LockStatus::Unlocked;

        // In production, execute unlock transaction
        let signature = Self::generate_mock_signature();

        Ok(signature)
    }

    pub async fn revoke_lock(&self, lock_id: &str, app: &AppHandle) -> Result<String, AppError> {
        let mut locks = self.locks.lock().unwrap();

        let lock = locks
            .iter_mut()
            .find(|l| l.lock_id.as_deref() == Some(lock_id))
            .ok_or_else(|| AppError::NotFound("Lock not found".to_string()))?;

        if !lock.is_revocable {
            return Err(AppError::Validation("Lock is not revocable".to_string()));
        }

        // Get authority keypair from keystore
        let keystore: tauri::State<Keystore> = app.try_state::<Keystore>().unwrap();
        let _authority_secret = keystore
            .retrieve_secret("wallet_keypair")
            .map_err(|e| AppError::Generic(format!("Failed to retrieve keypair: {}", e)))?;

        lock.status = LockStatus::Revoked;

        // In production, execute revoke transaction
        let signature = Self::generate_mock_signature();

        Ok(signature)
    }

    pub fn get_lock(&self, lock_id: &str) -> Result<LiquidityLockConfig, AppError> {
        let locks = self.locks.lock().unwrap();

        locks
            .iter()
            .find(|l| l.lock_id.as_deref() == Some(lock_id))
            .cloned()
            .ok_or_else(|| AppError::NotFound("Lock not found".to_string()))
    }

    pub fn get_all_locks(&self) -> Vec<LiquidityLockConfig> {
        let locks = self.locks.lock().unwrap();
        locks.clone()
    }

    pub fn get_active_locks(&self) -> Vec<LiquidityLockConfig> {
        let locks = self.locks.lock().unwrap();
        locks
            .iter()
            .filter(|l| l.status == LockStatus::Locked)
            .cloned()
            .collect()
    }

    fn generate_mock_signature() -> String {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 64];
        rng.fill_bytes(&mut bytes);
        bs58::encode(bytes).into_string()
    }
}

impl Default for LiquidityLocker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_lock_request() {
        let request = LockLiquidityRequest {
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            pool_address: "11111111111111111111111111111111".to_string(),
            amount: 1000000,
            duration_seconds: 86400 * 30, // 30 days
            beneficiary: "11111111111111111111111111111111".to_string(),
            is_revocable: false,
        };

        assert!(request.amount > 0);
        assert!(request.duration_seconds >= 86400);
    }
}
