use super::types::*;
use crate::errors::AppError;
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

pub struct VestingManager {
    schedules: RwLock<HashMap<String, VestingSchedule>>, // id -> schedule
}

impl VestingManager {
    pub fn new() -> Self {
        Self {
            schedules: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_schedule(
        &self,
        request: CreateVestingRequest,
    ) -> Result<VestingSchedule, AppError> {
        self.validate_request(&request)?;

        let schedule_id = Uuid::new_v4().to_string();
        let schedule = VestingSchedule {
            id: schedule_id.clone(),
            token_mint: request.token_mint,
            beneficiary: request.beneficiary,
            total_amount: request.total_amount,
            start_date: request.start_date,
            cliff_duration_seconds: request.cliff_duration_seconds,
            vesting_duration_seconds: request.vesting_duration_seconds,
            vesting_type: request.vesting_type,
            released_amount: 0,
            revoked: false,
            created_at: Utc::now(),
        };

        self.schedules
            .write()
            .insert(schedule_id.clone(), schedule.clone());

        Ok(schedule)
    }

    pub fn release_tokens(
        &self,
        schedule_id: &str,
        amount: u64,
    ) -> Result<VestingSchedule, AppError> {
        if amount == 0 {
            return Err(AppError::Validation(
                "Release amount must be greater than 0".to_string(),
            ));
        }

        let mut schedules = self.schedules.write();
        let schedule = schedules
            .get_mut(schedule_id)
            .ok_or_else(|| AppError::NotFound("Vesting schedule not found".to_string()))?;

        if schedule.revoked {
            return Err(AppError::Validation(
                "Vesting schedule has been revoked".to_string(),
            ));
        }

        let releasable = self.releasable_amount(schedule);
        if amount > releasable {
            return Err(AppError::Validation(
                "Requested amount exceeds releasable tokens".to_string(),
            ));
        }

        schedule.released_amount = schedule.released_amount.saturating_add(amount);

        Ok(schedule.clone())
    }

    pub fn revoke_schedule(&self, schedule_id: &str) -> Result<VestingSchedule, AppError> {
        let mut schedules = self.schedules.write();
        let schedule = schedules
            .get_mut(schedule_id)
            .ok_or_else(|| AppError::NotFound("Vesting schedule not found".to_string()))?;

        schedule.revoked = true;

        Ok(schedule.clone())
    }

    pub fn get_schedule(&self, schedule_id: &str) -> Result<VestingSchedule, AppError> {
        self.schedules
            .read()
            .get(schedule_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Vesting schedule not found".to_string()))
    }

    pub fn get_schedules_for_mint(&self, mint: &str) -> Vec<VestingSchedule> {
        self.schedules
            .read()
            .values()
            .filter(|s| s.token_mint == mint)
            .cloned()
            .collect()
    }

    pub fn get_schedules_for_beneficiary(&self, beneficiary: &str) -> Vec<VestingSchedule> {
        self.schedules
            .read()
            .values()
            .filter(|s| s.beneficiary == beneficiary)
            .cloned()
            .collect()
    }

    fn validate_request(&self, request: &CreateVestingRequest) -> Result<(), AppError> {
        if request.total_amount == 0 {
            return Err(AppError::Validation(
                "Total vesting amount must be greater than 0".to_string(),
            ));
        }

        if request.vesting_duration_seconds == 0 {
            return Err(AppError::Validation(
                "Vesting duration must be greater than 0".to_string(),
            ));
        }

        if request.cliff_duration_seconds.is_some()
            && request.cliff_duration_seconds.unwrap() >= request.vesting_duration_seconds
        {
            return Err(AppError::Validation(
                "Cliff duration must be less than vesting duration".to_string(),
            ));
        }

        if let Some(stages) = &request.stages {
            if request.vesting_type != VestingType::Staged {
                return Err(AppError::Validation(
                    "Stages provided but vesting type is not staged".to_string(),
                ));
            }

            let total_percentage: u32 = stages.iter().map(|s| s.percentage as u32).sum();
            if total_percentage != 100 {
                return Err(AppError::Validation(
                    "Total stage percentages must equal 100".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn releasable_amount(&self, schedule: &VestingSchedule) -> u64 {
        let now = Utc::now();

        if schedule.revoked {
            return 0;
        }

        if now < schedule.start_date {
            return 0;
        }

        match schedule.vesting_type {
            VestingType::Cliff => self.releasable_cliff(schedule, now),
            VestingType::Linear => self.releasable_linear(schedule, now),
            VestingType::Staged => self.releasable_staged(schedule, now),
        }
    }

    fn releasable_cliff(&self, schedule: &VestingSchedule, now: DateTime<Utc>) -> u64 {
        if let Some(cliff) = schedule.cliff_duration_seconds {
            let cliff_date = schedule.start_date + Duration::seconds(cliff as i64);
            if now >= cliff_date {
                return schedule
                    .total_amount
                    .saturating_sub(schedule.released_amount);
            }
        }
        0
    }

    fn releasable_linear(&self, schedule: &VestingSchedule, now: DateTime<Utc>) -> u64 {
        let elapsed = now.signed_duration_since(schedule.start_date);
        if elapsed.num_seconds() <= 0 {
            return 0;
        }

        let total_duration = schedule.vesting_duration_seconds as i64;
        let elapsed_seconds = elapsed.num_seconds().min(total_duration);
        let vested =
            (schedule.total_amount as u128 * elapsed_seconds as u128) / total_duration as u128;

        vested
            .saturating_sub(schedule.released_amount as u128)
            .min(schedule.total_amount as u128) as u64
    }

    fn releasable_staged(&self, schedule: &VestingSchedule, now: DateTime<Utc>) -> u64 {
        // For staged vesting, we expect stages to be provided separately and tracked
        // For simplicity, release full amount after vesting_duration
        if now >= schedule.start_date + Duration::seconds(schedule.vesting_duration_seconds as i64)
        {
            return schedule
                .total_amount
                .saturating_sub(schedule.released_amount);
        }
        0
    }
}

impl Default for VestingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vesting_validation() {
        let manager = VestingManager::new();

        let request = CreateVestingRequest {
            token_mint: "So11111111111111111111111111111111111111112".to_string(),
            beneficiary: "11111111111111111111111111111111".to_string(),
            total_amount: 1_000_000,
            start_date: Utc::now(),
            cliff_duration_seconds: Some(86400),
            vesting_duration_seconds: 86400 * 30,
            vesting_type: VestingType::Linear,
            stages: None,
        };

        assert!(manager.validate_request(&request).is_ok());
    }
}
