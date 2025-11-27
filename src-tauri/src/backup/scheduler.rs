use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

const SCHEDULER_CONFIG_FILE: &str = "backup_schedule.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackupFrequency {
    Manual,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSchedule {
    pub enabled: bool,
    pub frequency: BackupFrequency,
    pub time_of_day: String,      // HH:MM format
    pub day_of_week: Option<u8>,  // 0-6 for weekly backups
    pub day_of_month: Option<u8>, // 1-31 for monthly backups
    pub last_backup: Option<DateTime<Utc>>,
    pub next_backup: Option<DateTime<Utc>>,
    pub auto_delete_old: bool,
    pub keep_last_n_backups: usize,
}

impl Default for BackupSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: BackupFrequency::Manual,
            time_of_day: "03:00".to_string(),
            day_of_week: None,
            day_of_month: None,
            last_backup: None,
            next_backup: None,
            auto_delete_old: true,
            keep_last_n_backups: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    pub running: bool,
    pub last_backup: Option<DateTime<Utc>>,
    pub next_backup: Option<DateTime<Utc>>,
    pub last_backup_size: Option<u64>,
    pub last_error: Option<String>,
    pub total_backups: usize,
}

impl Default for BackupStatus {
    fn default() -> Self {
        Self {
            running: false,
            last_backup: None,
            next_backup: None,
            last_backup_size: None,
            last_error: None,
            total_backups: 0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("invalid schedule configuration: {0}")]
    InvalidConfig(String),
}

pub type SharedBackupScheduler = Arc<RwLock<BackupScheduler>>;

pub struct BackupScheduler {
    app_handle: AppHandle,
    schedule: BackupSchedule,
    status: BackupStatus,
}

impl BackupScheduler {
    pub fn new(app: &AppHandle) -> Result<Self, SchedulerError> {
        let mut scheduler = Self {
            app_handle: app.clone(),
            schedule: BackupSchedule::default(),
            status: BackupStatus::default(),
        };

        // Try to load existing schedule
        if let Ok(schedule) = scheduler.load_schedule() {
            scheduler.schedule = schedule;
            scheduler.calculate_next_backup();
        }

        Ok(scheduler)
    }

    fn config_path(&self) -> Result<PathBuf, SchedulerError> {
        let mut path = self.app_handle.path().app_data_dir().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::NotFound, format!("App data directory not found: {}", e))
        })?;

        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        path.push(SCHEDULER_CONFIG_FILE);
        Ok(path)
    }

    fn load_schedule(&self) -> Result<BackupSchedule, SchedulerError> {
        let path = self.config_path()?;
        if !path.exists() {
            return Ok(BackupSchedule::default());
        }

        let data = fs::read_to_string(&path)?;
        let schedule: BackupSchedule = serde_json::from_str(&data)?;
        Ok(schedule)
    }

    fn save_schedule(&self) -> Result<(), SchedulerError> {
        let path = self.config_path()?;
        let json = serde_json::to_string_pretty(&self.schedule)?;
        fs::write(&path, json)?;
        Ok(())
    }

    pub fn get_schedule(&self) -> BackupSchedule {
        self.schedule.clone()
    }

    pub fn get_status(&self) -> BackupStatus {
        self.status.clone()
    }

    pub fn update_schedule(&mut self, schedule: BackupSchedule) -> Result<(), SchedulerError> {
        self.validate_schedule(&schedule)?;
        self.schedule = schedule;
        self.calculate_next_backup();
        self.save_schedule()?;
        Ok(())
    }

    fn validate_schedule(&self, schedule: &BackupSchedule) -> Result<(), SchedulerError> {
        // Validate time format
        let parts: Vec<&str> = schedule.time_of_day.split(':').collect();
        if parts.len() != 2 {
            return Err(SchedulerError::InvalidConfig(
                "Time must be in HH:MM format".to_string(),
            ));
        }

        let hour: u8 = parts[0]
            .parse()
            .map_err(|_| SchedulerError::InvalidConfig("Invalid hour".to_string()))?;
        let minute: u8 = parts[1]
            .parse()
            .map_err(|_| SchedulerError::InvalidConfig("Invalid minute".to_string()))?;

        if hour > 23 || minute > 59 {
            return Err(SchedulerError::InvalidConfig(
                "Invalid time values".to_string(),
            ));
        }

        // Validate day of week for weekly backups
        if schedule.frequency == BackupFrequency::Weekly {
            if let Some(day) = schedule.day_of_week {
                if day > 6 {
                    return Err(SchedulerError::InvalidConfig(
                        "Day of week must be 0-6".to_string(),
                    ));
                }
            } else {
                return Err(SchedulerError::InvalidConfig(
                    "Day of week required for weekly backups".to_string(),
                ));
            }
        }

        // Validate day of month for monthly backups
        if schedule.frequency == BackupFrequency::Monthly {
            if let Some(day) = schedule.day_of_month {
                if day < 1 || day > 31 {
                    return Err(SchedulerError::InvalidConfig(
                        "Day of month must be 1-31".to_string(),
                    ));
                }
            } else {
                return Err(SchedulerError::InvalidConfig(
                    "Day of month required for monthly backups".to_string(),
                ));
            }
        }

        if schedule.keep_last_n_backups == 0 {
            return Err(SchedulerError::InvalidConfig(
                "Must keep at least 1 backup".to_string(),
            ));
        }

        Ok(())
    }

    fn calculate_next_backup(&mut self) {
        if !self.schedule.enabled || self.schedule.frequency == BackupFrequency::Manual {
            self.status.next_backup = None;
            self.schedule.next_backup = None;
            return;
        }

        let now = Utc::now();
        let next = match self.schedule.frequency {
            BackupFrequency::Daily => {
                // Calculate next daily backup
                let next = now + chrono::Duration::days(1);
                Some(next)
            }
            BackupFrequency::Weekly => {
                // Calculate next weekly backup
                let next = now + chrono::Duration::weeks(1);
                Some(next)
            }
            BackupFrequency::Monthly => {
                // Calculate next monthly backup
                let next = now + chrono::Duration::days(30); // Approximate
                Some(next)
            }
            BackupFrequency::Manual => None,
        };

        self.schedule.next_backup = next;
        self.status.next_backup = next;
    }

    pub fn mark_backup_complete(&mut self, size: u64) -> Result<(), SchedulerError> {
        let now = Utc::now();
        self.schedule.last_backup = Some(now);
        self.status.last_backup = Some(now);
        self.status.last_backup_size = Some(size);
        self.status.last_error = None;
        self.status.total_backups += 1;
        self.status.running = false;

        self.calculate_next_backup();
        self.save_schedule()?;

        Ok(())
    }

    pub fn mark_backup_failed(&mut self, error: String) -> Result<(), SchedulerError> {
        self.status.last_error = Some(error);
        self.status.running = false;
        self.save_schedule()?;
        Ok(())
    }

    pub fn start_backup(&mut self) {
        self.status.running = true;
        self.status.last_error = None;
    }

    pub fn should_backup_now(&self) -> bool {
        if !self.schedule.enabled || self.status.running {
            return false;
        }

        if self.schedule.frequency == BackupFrequency::Manual {
            return false;
        }

        if let Some(next_backup) = self.schedule.next_backup {
            Utc::now() >= next_backup
        } else {
            false
        }
    }
}
