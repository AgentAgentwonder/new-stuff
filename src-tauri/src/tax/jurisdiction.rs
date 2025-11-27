use super::types::TaxJurisdiction;
use crate::security::keystore::Keystore;
use serde_json;

const JURISDICTION_KEY_PREFIX: &str = "tax_jurisdiction_";

pub struct JurisdictionManager;

impl JurisdictionManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_available_jurisdictions() -> Vec<TaxJurisdiction> {
        vec![
            TaxJurisdiction::us_federal(),
            TaxJurisdiction::uk(),
            TaxJurisdiction::germany(),
            TaxJurisdiction::australia(),
        ]
    }

    pub fn save_jurisdiction(
        &self,
        keystore: &Keystore,
        user_id: &str,
        jurisdiction: &TaxJurisdiction,
    ) -> Result<(), String> {
        let key = format!("{}{}", JURISDICTION_KEY_PREFIX, user_id);
        let serialized = serde_json::to_vec(jurisdiction)
            .map_err(|e| format!("Failed to serialize jurisdiction: {e}"))?;

        keystore
            .store_secret(&key, &serialized)
            .map_err(|e| format!("Failed to store jurisdiction: {e}"))?;

        Ok(())
    }

    pub fn load_jurisdiction(
        &self,
        keystore: &Keystore,
        user_id: &str,
    ) -> Result<TaxJurisdiction, String> {
        let key = format!("{}{}", JURISDICTION_KEY_PREFIX, user_id);

        match keystore.retrieve_secret(&key) {
            Ok(data) => {
                let jurisdiction = serde_json::from_slice(&data)
                    .map_err(|e| format!("Failed to deserialize jurisdiction: {e}"))?;
                Ok(jurisdiction)
            }
            Err(_) => {
                // Return default if not found
                Ok(TaxJurisdiction::default())
            }
        }
    }

    pub fn delete_jurisdiction(&self, keystore: &Keystore, user_id: &str) -> Result<(), String> {
        let key = format!("{}{}", JURISDICTION_KEY_PREFIX, user_id);

        keystore
            .remove_secret(&key)
            .map_err(|e| format!("Failed to delete jurisdiction: {e}"))?;

        Ok(())
    }

    pub fn get_jurisdiction_by_code(code: &str) -> Option<TaxJurisdiction> {
        match code {
            "US" => Some(TaxJurisdiction::us_federal()),
            "UK" => Some(TaxJurisdiction::uk()),
            "DE" => Some(TaxJurisdiction::germany()),
            "AU" => Some(TaxJurisdiction::australia()),
            _ => None,
        }
    }

    pub fn validate_jurisdiction(&self, jurisdiction: &TaxJurisdiction) -> Result<(), String> {
        if jurisdiction.code.is_empty() {
            return Err("Jurisdiction code cannot be empty".to_string());
        }

        if jurisdiction.name.is_empty() {
            return Err("Jurisdiction name cannot be empty".to_string());
        }

        if jurisdiction.short_term_rate < 0.0 || jurisdiction.short_term_rate > 1.0 {
            return Err("Short term rate must be between 0 and 1".to_string());
        }

        if jurisdiction.long_term_rate < 0.0 || jurisdiction.long_term_rate > 1.0 {
            return Err("Long term rate must be between 0 and 1".to_string());
        }

        if jurisdiction.holding_period_days < 0 {
            return Err("Holding period days cannot be negative".to_string());
        }

        if jurisdiction.wash_sale_period_days < 0 {
            return Err("Wash sale period days cannot be negative".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_jurisdictions() {
        let jurisdictions = JurisdictionManager::get_available_jurisdictions();
        assert!(!jurisdictions.is_empty());
        assert!(jurisdictions.iter().any(|j| j.code == "US"));
    }

    #[test]
    fn test_get_jurisdiction_by_code() {
        let us = JurisdictionManager::get_jurisdiction_by_code("US");
        assert!(us.is_some());
        assert_eq!(us.unwrap().code, "US");

        let invalid = JurisdictionManager::get_jurisdiction_by_code("XX");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_validate_jurisdiction() {
        let manager = JurisdictionManager::new();
        let valid = TaxJurisdiction::us_federal();
        assert!(manager.validate_jurisdiction(&valid).is_ok());

        let mut invalid = TaxJurisdiction::us_federal();
        invalid.short_term_rate = 1.5;
        assert!(manager.validate_jurisdiction(&invalid).is_err());
    }
}
