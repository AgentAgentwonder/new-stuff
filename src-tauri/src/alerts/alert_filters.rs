use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::{AppHandle, State};

use super::price_alerts::{AlertError, CompoundCondition};

const ALERT_FILTERS_DB_FILE: &str = "alert_filters.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertFilter {
    pub id: String,
    pub name: String,
    pub min_volume: Option<f64>,
    pub min_liquidity: Option<f64>,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub spam_detection_enabled: bool,
    pub ml_threshold: f64,
    pub filtered_count: i64,
    pub last_applied_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFilterRequest {
    pub name: String,
    pub min_volume: Option<f64>,
    pub min_liquidity: Option<f64>,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub spam_detection_enabled: bool,
    pub ml_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFilterRequest {
    pub name: Option<String>,
    pub min_volume: Option<Option<f64>>,
    pub min_liquidity: Option<Option<f64>>,
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
    pub spam_detection_enabled: Option<bool>,
    pub ml_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterEvaluationResult {
    pub passed: bool,
    pub reasons: Vec<String>,
    pub applied_filter_id: Option<String>,
    pub spam_score: f64,
}

pub struct AlertFilterManager {
    pool: Pool<Sqlite>,
}

impl AlertFilterManager {
    pub async fn new(app: &AppHandle) -> Result<Self, AlertError> {
        let db_path = alert_filters_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self { pool };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), AlertError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alert_filters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                min_volume REAL,
                min_liquidity REAL,
                whitelist TEXT NOT NULL,
                blacklist TEXT NOT NULL,
                spam_detection_enabled INTEGER NOT NULL,
                ml_threshold REAL NOT NULL,
                filtered_count INTEGER NOT NULL DEFAULT 0,
                last_applied_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_filters_name ON alert_filters(name);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_filter(
        &self,
        req: CreateFilterRequest,
    ) -> Result<AlertFilter, AlertError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let whitelist_json = serde_json::to_string(&req.whitelist)?;
        let blacklist_json = serde_json::to_string(&req.blacklist)?;

        sqlx::query(
            r#"
            INSERT INTO alert_filters (
                id, name, min_volume, min_liquidity, whitelist, blacklist,
                spam_detection_enabled, ml_threshold, filtered_count, last_applied_at,
                created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, NULL, ?9, ?10)
            "#,
        )
        .bind(&id)
        .bind(&req.name)
        .bind(req.min_volume)
        .bind(req.min_liquidity)
        .bind(&whitelist_json)
        .bind(&blacklist_json)
        .bind(if req.spam_detection_enabled { 1 } else { 0 })
        .bind(req.ml_threshold)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(AlertFilter {
            id,
            name: req.name,
            min_volume: req.min_volume,
            min_liquidity: req.min_liquidity,
            whitelist: req.whitelist,
            blacklist: req.blacklist,
            spam_detection_enabled: req.spam_detection_enabled,
            ml_threshold: req.ml_threshold,
            filtered_count: 0,
            last_applied_at: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn list_filters(&self) -> Result<Vec<AlertFilter>, AlertError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, min_volume, min_liquidity, whitelist, blacklist,
                   spam_detection_enabled, ml_threshold, filtered_count,
                   last_applied_at, created_at, updated_at
            FROM alert_filters
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut filters = Vec::new();
        for row in rows {
            filters.push(self.row_to_filter(row)?);
        }

        Ok(filters)
    }

    pub async fn get_filter(&self, id: &str) -> Result<AlertFilter, AlertError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, min_volume, min_liquidity, whitelist, blacklist,
                   spam_detection_enabled, ml_threshold, filtered_count,
                   last_applied_at, created_at, updated_at
            FROM alert_filters
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AlertError::NotFound(id.to_string()))?;

        self.row_to_filter(row)
    }

    pub async fn update_filter(
        &self,
        id: &str,
        req: UpdateFilterRequest,
    ) -> Result<AlertFilter, AlertError> {
        let mut filter = self.get_filter(id).await?;
        let now = Utc::now().to_rfc3339();

        if let Some(name) = req.name {
            filter.name = name;
        }
        if let Some(min_volume) = req.min_volume {
            filter.min_volume = min_volume;
        }
        if let Some(min_liquidity) = req.min_liquidity {
            filter.min_liquidity = min_liquidity;
        }
        if let Some(whitelist) = req.whitelist {
            filter.whitelist = whitelist;
        }
        if let Some(blacklist) = req.blacklist {
            filter.blacklist = blacklist;
        }
        if let Some(spam_detection_enabled) = req.spam_detection_enabled {
            filter.spam_detection_enabled = spam_detection_enabled;
        }
        if let Some(ml_threshold) = req.ml_threshold {
            filter.ml_threshold = ml_threshold;
        }

        filter.updated_at = now.clone();

        let whitelist_json = serde_json::to_string(&filter.whitelist)?;
        let blacklist_json = serde_json::to_string(&filter.blacklist)?;

        sqlx::query(
            r#"
            UPDATE alert_filters
            SET name = ?1, min_volume = ?2, min_liquidity = ?3,
                whitelist = ?4, blacklist = ?5, spam_detection_enabled = ?6,
                ml_threshold = ?7, updated_at = ?8
            WHERE id = ?9
            "#,
        )
        .bind(&filter.name)
        .bind(filter.min_volume)
        .bind(filter.min_liquidity)
        .bind(&whitelist_json)
        .bind(&blacklist_json)
        .bind(if filter.spam_detection_enabled { 1 } else { 0 })
        .bind(filter.ml_threshold)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(filter)
    }

    pub async fn delete_filter(&self, id: &str) -> Result<(), AlertError> {
        let result = sqlx::query("DELETE FROM alert_filters WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AlertError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn evaluate_alert(
        &self,
        symbol: &str,
        compound_condition: &CompoundCondition,
        metadata: AlertFilterMetadata,
    ) -> Result<FilterEvaluationResult, AlertError> {
        let filters = self.list_filters().await?;

        for filter in filters {
            let mut reasons = Vec::new();
            let mut passed = true;

            if let Some(min_volume) = filter.min_volume {
                if let Some(volume) = metadata.volume_24h {
                    if volume < min_volume {
                        passed = false;
                        reasons.push(format!(
                            "Volume {:.0} below minimum {:.0}",
                            volume, min_volume
                        ));
                    }
                } else {
                    passed = false;
                    reasons.push("Volume data unavailable".to_string());
                }
            }

            if let Some(min_liquidity) = filter.min_liquidity {
                if let Some(liquidity) = metadata.liquidity_usd {
                    if liquidity < min_liquidity {
                        passed = false;
                        reasons.push(format!(
                            "Liquidity {:.0} below minimum {:.0}",
                            liquidity, min_liquidity
                        ));
                    }
                } else {
                    passed = false;
                    reasons.push("Liquidity data unavailable".to_string());
                }
            }

            if !filter.whitelist.is_empty() {
                let whitelist: HashSet<String> = filter
                    .whitelist
                    .iter()
                    .map(|s| s.to_ascii_lowercase())
                    .collect();
                if !whitelist.contains(&symbol.to_ascii_lowercase()) {
                    passed = false;
                    reasons.push("Symbol not in whitelist".to_string());
                }
            }

            if !filter.blacklist.is_empty() {
                let blacklist: HashSet<String> = filter
                    .blacklist
                    .iter()
                    .map(|s| s.to_ascii_lowercase())
                    .collect();
                if blacklist.contains(&symbol.to_ascii_lowercase()) {
                    passed = false;
                    reasons.push("Symbol in blacklist".to_string());
                }
            }

            let spam_score = if filter.spam_detection_enabled {
                spam_detection_heuristic(compound_condition, &metadata)
            } else {
                0.0
            };

            if spam_score > filter.ml_threshold {
                passed = false;
                reasons.push(format!(
                    "Spam score {:.2} exceeds threshold {:.2}",
                    spam_score, filter.ml_threshold
                ));
            }

            if !passed {
                self.increment_filtered_count(&filter.id).await?;
                return Ok(FilterEvaluationResult {
                    passed: false,
                    reasons,
                    applied_filter_id: Some(filter.id.clone()),
                    spam_score,
                });
            } else {
                self.update_last_applied(&filter.id).await?;
            }
        }

        Ok(FilterEvaluationResult {
            passed: true,
            reasons: Vec::new(),
            applied_filter_id: None,
            spam_score: 0.0,
        })
    }

    async fn increment_filtered_count(&self, id: &str) -> Result<(), AlertError> {
        sqlx::query(
            r#"
            UPDATE alert_filters
            SET filtered_count = filtered_count + 1,
                last_applied_at = ?1
            WHERE id = ?2
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_last_applied(&self, id: &str) -> Result<(), AlertError> {
        sqlx::query(
            r#"
            UPDATE alert_filters
            SET last_applied_at = ?1
            WHERE id = ?2
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn row_to_filter(&self, row: sqlx::sqlite::SqliteRow) -> Result<AlertFilter, AlertError> {
        let whitelist_json: String = row.try_get("whitelist")?;
        let whitelist: Vec<String> = serde_json::from_str(&whitelist_json)?;

        let blacklist_json: String = row.try_get("blacklist")?;
        let blacklist: Vec<String> = serde_json::from_str(&blacklist_json)?;

        Ok(AlertFilter {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            min_volume: row.try_get("min_volume")?,
            min_liquidity: row.try_get("min_liquidity")?,
            whitelist,
            blacklist,
            spam_detection_enabled: row.try_get::<i32, _>("spam_detection_enabled")? == 1,
            ml_threshold: row.try_get("ml_threshold")?,
            filtered_count: row.try_get("filtered_count")?,
            last_applied_at: row.try_get("last_applied_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertFilterMetadata {
    pub volume_24h: Option<f64>,
    pub liquidity_usd: Option<f64>,
    pub social_score: Option<f64>,
    pub sentiment_score: Option<f64>,
    pub volatility_score: Option<f64>,
}

fn alert_filters_db_path(app: &AppHandle) -> Result<PathBuf, AlertError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .ok_or_else(|| AlertError::Internal("Unable to resolve app data directory".to_string()))?;

    std::fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join(ALERT_FILTERS_DB_FILE))
}

fn spam_detection_heuristic(
    compound_condition: &CompoundCondition,
    metadata: &AlertFilterMetadata,
) -> f64 {
    let mut score = 0.0;

    // Basic heuristic: combine volatility, sentiment, and condition complexity
    if let Some(volatility) = metadata.volatility_score {
        score += volatility * 0.4;
    }
    if let Some(sentiment) = metadata.sentiment_score {
        score += (1.0 - sentiment) * 0.3; // low sentiment increases score
    }
    if let Some(social) = metadata.social_score {
        score += (1.0 - social) * 0.2;
    }

    // More complex conditions get lower score (less likely spam)
    let condition_complexity = compound_condition.conditions.len() as f64;
    if condition_complexity > 1.0 {
        score -= 0.1 * condition_complexity.min(3.0);
    }

    score.clamp(0.0, 1.0)
}

// Tauri commands
#[tauri::command]
pub async fn alert_filter_create(
    manager: State<'_, crate::alerts::SharedAlertFilterManager>,
    req: CreateFilterRequest,
) -> Result<AlertFilter, String> {
    let mgr = manager.read().await;
    mgr.create_filter(req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_filter_list(
    manager: State<'_, crate::alerts::SharedAlertFilterManager>,
) -> Result<Vec<AlertFilter>, String> {
    let mgr = manager.read().await;
    mgr.list_filters().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_filter_get(
    manager: State<'_, crate::alerts::SharedAlertFilterManager>,
    id: String,
) -> Result<AlertFilter, String> {
    let mgr = manager.read().await;
    mgr.get_filter(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_filter_update(
    manager: State<'_, crate::alerts::SharedAlertFilterManager>,
    id: String,
    req: UpdateFilterRequest,
) -> Result<AlertFilter, String> {
    let mgr = manager.read().await;
    mgr.update_filter(&id, req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_filter_delete(
    manager: State<'_, crate::alerts::SharedAlertFilterManager>,
    id: String,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_filter(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_filter_evaluate(
    manager: State<'_, crate::alerts::SharedAlertFilterManager>,
    symbol: String,
    compound_condition: CompoundCondition,
    metadata: AlertFilterMetadata,
) -> Result<FilterEvaluationResult, String> {
    let mgr = manager.read().await;
    mgr.evaluate_alert(&symbol, &compound_condition, metadata)
        .await
        .map_err(|e| e.to_string())
}
