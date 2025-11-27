use crate::errors::AppError;
use crate::security::keystore::Keystore;
use chrono::Utc;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use uuid::Uuid;
use zeroize::Zeroizing;

const LAUNCH_KEY_PREFIX: &str = "launchpad::key::";

pub struct LaunchpadKeyManager {
    cache: Mutex<HashMap<String, Zeroizing<Vec<u8>>>>,
}

impl LaunchpadKeyManager {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_ephemeral_key(&self, app: &AppHandle) -> Result<KeyDescriptor, AppError> {
        let mut key = vec![0u8; 64];
        let mut rng = OsRng;
        rng.fill_bytes(&mut key);

        let key_id = Uuid::new_v4().to_string();
        let keystore: tauri::State<Keystore> = app.try_state::<Keystore>().unwrap();
        keystore
            .store_secret(&format!("{}{}", LAUNCH_KEY_PREFIX, key_id), &key)
            .map_err(|e| AppError::Generic(format!("Failed to store key: {}", e)))?;

        self.cache
            .lock()
            .map_err(|_| AppError::Generic("Failed to access key cache".to_string()))?
            .insert(key_id.clone(), Zeroizing::new(key));

        Ok(KeyDescriptor {
            key_id,
            created_at: Utc::now(),
            permanent: false,
        })
    }

    pub fn get_key(&self, key_id: &str, app: &AppHandle) -> Result<Zeroizing<Vec<u8>>, AppError> {
        if let Ok(cache) = self.cache.lock() {
            if let Some(secret) = cache.get(key_id) {
                return Ok(secret.clone());
            }
        }

        let keystore: tauri::State<Keystore> = app.try_state::<Keystore>().unwrap();
        let secret = keystore
            .retrieve_secret(&format!("{}{}", LAUNCH_KEY_PREFIX, key_id))
            .map_err(|e| AppError::Generic(format!("Failed to retrieve key: {}", e)))?;

        Ok(secret)
    }

    pub fn revoke_key(&self, key_id: &str, app: &AppHandle) -> Result<(), AppError> {
        let keystore: tauri::State<Keystore> = app.state::<Keystore>();
        keystore
            .remove_secret(&format!("{}{}", LAUNCH_KEY_PREFIX, key_id))
            .map_err(|e| AppError::Generic(format!("Failed to remove key: {}", e)))?;

        if let Ok(mut cache) = self.cache.lock() {
            cache.remove(key_id);
        }

        Ok(())
    }

    pub fn simulate_transaction(
        &self,
        instructions: &[SimulatedInstruction],
    ) -> TransactionSimulationResult {
        let mut compute_units: u64 = 0;
        let mut warnings = Vec::new();

        for instruction in instructions {
            compute_units += instruction.estimated_compute_units;
            if instruction.risk_score > 70 {
                warnings.push(format!(
                    "Instruction '{}' has elevated risk score: {}",
                    instruction.name, instruction.risk_score
                ));
            }
        }

        let fee_estimate = (compute_units / 1000).max(5_000); // Minimum fee

        TransactionSimulationResult {
            success: warnings.is_empty(),
            compute_units,
            fee_estimate,
            logs: instructions.iter().map(|i| i.log_message.clone()).collect(),
            warnings,
        }
    }
}

impl Default for LaunchpadKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyDescriptor {
    pub key_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub permanent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulatedInstruction {
    pub name: String,
    pub estimated_compute_units: u64,
    pub risk_score: u8,
    pub log_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSimulationResult {
    pub success: bool,
    pub compute_units: u64,
    pub fee_estimate: u64,
    pub logs: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_result() {
        let key_manager = LaunchpadKeyManager::new();

        let instructions = vec![SimulatedInstruction {
            name: "InitializeMint".to_string(),
            estimated_compute_units: 120_000,
            risk_score: 30,
            log_message: "Mint initialized".to_string(),
        }];

        let result = key_manager.simulate_transaction(&instructions);
        assert!(result.success);
        assert!(result.compute_units > 0);
    }
}
