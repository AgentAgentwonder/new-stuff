use chrono::{NaiveTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use tauri::AppHandle;

use super::types::NotificationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DndSchedule {
    pub id: String,
    pub enabled: bool,
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub days_of_week: Vec<u8>, // 0 = Sunday, 6 = Saturday
    pub timezone: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDndScheduleRequest {
    pub start_time: String,
    pub end_time: String,
    pub days_of_week: Vec<u8>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDndScheduleRequest {
    pub enabled: Option<bool>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub days_of_week: Option<Vec<u8>>,
    pub timezone: Option<String>,
}

pub struct DndScheduler {
    pool: Pool<Sqlite>,
}

impl DndScheduler {
    pub async fn new(pool: Pool<Sqlite>) -> Result<Self, NotificationError> {
        let scheduler = Self { pool };
        scheduler.initialize().await?;
        Ok(scheduler)
    }

    async fn initialize(&self) -> Result<(), NotificationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dnd_schedules (
                id TEXT PRIMARY KEY,
                enabled INTEGER NOT NULL DEFAULT 1,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                days_of_week TEXT NOT NULL,
                timezone TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_schedule(
        &self,
        req: CreateDndScheduleRequest,
    ) -> Result<DndSchedule, NotificationError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let days_json = serde_json::to_string(&req.days_of_week)?;

        sqlx::query(
            r#"
            INSERT INTO dnd_schedules (id, enabled, start_time, end_time, days_of_week, timezone, created_at, updated_at)
            VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(&id)
        .bind(&req.start_time)
        .bind(&req.end_time)
        .bind(&days_json)
        .bind(&req.timezone)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(DndSchedule {
            id,
            enabled: true,
            start_time: req.start_time,
            end_time: req.end_time,
            days_of_week: req.days_of_week,
            timezone: req.timezone,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn list_schedules(&self) -> Result<Vec<DndSchedule>, NotificationError> {
        let rows = sqlx::query(
            r#"
            SELECT id, enabled, start_time, end_time, days_of_week, timezone, created_at, updated_at
            FROM dnd_schedules
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut schedules = Vec::new();
        for row in rows {
            schedules.push(self.row_to_schedule(row)?);
        }

        Ok(schedules)
    }

    pub async fn get_schedule(&self, id: &str) -> Result<DndSchedule, NotificationError> {
        let row = sqlx::query(
            r#"
            SELECT id, enabled, start_time, end_time, days_of_week, timezone, created_at, updated_at
            FROM dnd_schedules
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| NotificationError::Internal(format!("DND schedule not found: {}", id)))?;

        self.row_to_schedule(row)
    }

    pub async fn update_schedule(
        &self,
        id: &str,
        req: UpdateDndScheduleRequest,
    ) -> Result<DndSchedule, NotificationError> {
        let mut schedule = self.get_schedule(id).await?;
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(enabled) = req.enabled {
            schedule.enabled = enabled;
        }
        if let Some(start_time) = req.start_time {
            schedule.start_time = start_time;
        }
        if let Some(end_time) = req.end_time {
            schedule.end_time = end_time;
        }
        if let Some(days_of_week) = req.days_of_week {
            schedule.days_of_week = days_of_week;
        }
        if let Some(timezone) = req.timezone {
            schedule.timezone = timezone;
        }

        schedule.updated_at = now.clone();
        let days_json = serde_json::to_string(&schedule.days_of_week)?;

        sqlx::query(
            r#"
            UPDATE dnd_schedules
            SET enabled = ?1, start_time = ?2, end_time = ?3, days_of_week = ?4, timezone = ?5, updated_at = ?6
            WHERE id = ?7
            "#,
        )
        .bind(if schedule.enabled { 1 } else { 0 })
        .bind(&schedule.start_time)
        .bind(&schedule.end_time)
        .bind(&days_json)
        .bind(&schedule.timezone)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(schedule)
    }

    pub async fn delete_schedule(&self, id: &str) -> Result<(), NotificationError> {
        let result = sqlx::query("DELETE FROM dnd_schedules WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(NotificationError::Internal(format!("DND schedule not found: {}", id)));
        }

        Ok(())
    }

    pub async fn is_dnd_active(&self) -> Result<bool, NotificationError> {
        let schedules = self.list_schedules().await?;
        let now = Utc::now();

        for schedule in schedules {
            if !schedule.enabled {
                continue;
            }

            let weekday = now.weekday().num_days_from_sunday() as u8;
            if !schedule.days_of_week.contains(&weekday) {
                continue;
            }

            if let (Ok(start), Ok(end)) = (
                NaiveTime::parse_from_str(&schedule.start_time, "%H:%M"),
                NaiveTime::parse_from_str(&schedule.end_time, "%H:%M"),
            ) {
                let current_time = NaiveTime::from_hms_opt(
                    now.hour(),
                    now.minute(),
                    now.second(),
                ).unwrap();

                let in_range = if start < end {
                    current_time >= start && current_time <= end
                } else {
                    // Handles overnight schedules (e.g., 22:00 - 06:00)
                    current_time >= start || current_time <= end
                };

                if in_range {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn row_to_schedule(&self, row: sqlx::sqlite::SqliteRow) -> Result<DndSchedule, NotificationError> {
        let days_json: String = row.try_get("days_of_week")?;
        let days_of_week: Vec<u8> = serde_json::from_str(&days_json)?;
        let enabled_int: i32 = row.try_get("enabled")?;

        Ok(DndSchedule {
            id: row.try_get("id")?,
            enabled: enabled_int == 1,
            start_time: row.try_get("start_time")?,
            end_time: row.try_get("end_time")?,
            days_of_week,
            timezone: row.try_get("timezone")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
