use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use tauri::{AppHandle, State};

use super::price_alerts::{AlertError, CompoundCondition, NotificationChannel};

const ALERT_HISTORY_DB_FILE: &str = "alert_history.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertHistoryEntry {
    pub id: String,
    pub alert_id: String,
    pub alert_name: String,
    pub symbol: String,
    pub mint: String,
    pub compound_condition: CompoundCondition,
    pub triggered_price: f64,
    pub conditions_met: String,
    pub notification_channels: Vec<NotificationChannel>,
    pub triggered_at: String,
    pub bookmarked: bool,
    pub outcome_notes: Option<String>,
    pub price_after_1h: Option<f64>,
    pub price_after_4h: Option<f64>,
    pub price_after_24h: Option<f64>,
    pub outcome_type: Option<String>, // "profit", "loss", "neutral", "pending"
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertHistoryFilter {
    pub symbol: Option<String>,
    pub alert_name: Option<String>,
    pub bookmarked_only: Option<bool>,
    pub outcome_type: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertHistoryStats {
    pub total_alerts: i32,
    pub bookmarked_count: i32,
    pub profit_count: i32,
    pub loss_count: i32,
    pub neutral_count: i32,
    pub pending_count: i32,
    pub avg_price_change_1h: f64,
    pub avg_price_change_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAlertHistoryRequest {
    pub bookmarked: Option<bool>,
    pub outcome_notes: Option<String>,
    pub price_after_1h: Option<f64>,
    pub price_after_4h: Option<f64>,
    pub price_after_24h: Option<f64>,
    pub outcome_type: Option<String>,
}

pub struct AlertHistoryManager {
    pool: Pool<Sqlite>,
}

impl AlertHistoryManager {
    pub async fn new(app: &AppHandle) -> Result<Self, AlertError> {
        let db_path = alert_history_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self { pool };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), AlertError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alert_history (
                id TEXT PRIMARY KEY,
                alert_id TEXT NOT NULL,
                alert_name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                mint TEXT NOT NULL,
                compound_condition TEXT NOT NULL,
                triggered_price REAL NOT NULL,
                conditions_met TEXT NOT NULL,
                notification_channels TEXT NOT NULL,
                triggered_at TEXT NOT NULL,
                bookmarked INTEGER NOT NULL DEFAULT 0,
                outcome_notes TEXT,
                price_after_1h REAL,
                price_after_4h REAL,
                price_after_24h REAL,
                outcome_type TEXT,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_history_alert_id ON alert_history(alert_id);
            CREATE INDEX IF NOT EXISTS idx_history_symbol ON alert_history(symbol);
            CREATE INDEX IF NOT EXISTS idx_history_triggered_at ON alert_history(triggered_at);
            CREATE INDEX IF NOT EXISTS idx_history_bookmarked ON alert_history(bookmarked);
            CREATE INDEX IF NOT EXISTS idx_history_outcome_type ON alert_history(outcome_type);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_triggered_alert(
        &self,
        alert_id: &str,
        alert_name: &str,
        symbol: &str,
        mint: &str,
        compound_condition: &CompoundCondition,
        triggered_price: f64,
        conditions_met: &str,
        notification_channels: &[NotificationChannel],
    ) -> Result<AlertHistoryEntry, AlertError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let compound_condition_json = serde_json::to_string(compound_condition)?;
        let channels_json = serde_json::to_string(notification_channels)?;

        sqlx::query(
            r#"
            INSERT INTO alert_history (
                id, alert_id, alert_name, symbol, mint, compound_condition,
                triggered_price, conditions_met, notification_channels,
                triggered_at, bookmarked, outcome_notes, price_after_1h,
                price_after_4h, price_after_24h, outcome_type, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            "#,
        )
        .bind(&id)
        .bind(alert_id)
        .bind(alert_name)
        .bind(symbol)
        .bind(mint)
        .bind(&compound_condition_json)
        .bind(triggered_price)
        .bind(conditions_met)
        .bind(&channels_json)
        .bind(&now)
        .bind(0) // bookmarked = false
        .bind::<Option<String>>(None) // outcome_notes
        .bind::<Option<f64>>(None) // price_after_1h
        .bind::<Option<f64>>(None) // price_after_4h
        .bind::<Option<f64>>(None) // price_after_24h
        .bind::<Option<String>>(Some("pending".to_string())) // outcome_type
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(AlertHistoryEntry {
            id,
            alert_id: alert_id.to_string(),
            alert_name: alert_name.to_string(),
            symbol: symbol.to_string(),
            mint: mint.to_string(),
            compound_condition: compound_condition.clone(),
            triggered_price,
            conditions_met: conditions_met.to_string(),
            notification_channels: notification_channels.to_vec(),
            triggered_at: now.clone(),
            bookmarked: false,
            outcome_notes: None,
            price_after_1h: None,
            price_after_4h: None,
            price_after_24h: None,
            outcome_type: Some("pending".to_string()),
            created_at: now,
        })
    }

    pub async fn list_history(
        &self,
        filter: AlertHistoryFilter,
    ) -> Result<Vec<AlertHistoryEntry>, AlertError> {
        let mut query = String::from(
            r#"
            SELECT id, alert_id, alert_name, symbol, mint, compound_condition,
                   triggered_price, conditions_met, notification_channels,
                   triggered_at, bookmarked, outcome_notes, price_after_1h,
                   price_after_4h, price_after_24h, outcome_type, created_at
            FROM alert_history
            WHERE 1=1
            "#,
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(symbol) = &filter.symbol {
            query.push_str(&format!(" AND symbol = ?{}", params.len() + 1));
            params.push(symbol.clone());
        }

        if let Some(alert_name) = &filter.alert_name {
            query.push_str(&format!(" AND alert_name LIKE ?{}", params.len() + 1));
            params.push(format!("%{}%", alert_name));
        }

        if let Some(true) = filter.bookmarked_only {
            query.push_str(" AND bookmarked = 1");
        }

        if let Some(outcome_type) = &filter.outcome_type {
            query.push_str(&format!(" AND outcome_type = ?{}", params.len() + 1));
            params.push(outcome_type.clone());
        }

        if let Some(from_date) = &filter.from_date {
            query.push_str(&format!(" AND triggered_at >= ?{}", params.len() + 1));
            params.push(from_date.clone());
        }

        if let Some(to_date) = &filter.to_date {
            query.push_str(&format!(" AND triggered_at <= ?{}", params.len() + 1));
            params.push(to_date.clone());
        }

        query.push_str(" ORDER BY triggered_at DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut sql_query = sqlx::query(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(self.row_to_history_entry(row)?);
        }

        Ok(entries)
    }

    pub async fn get_history_entry(&self, id: &str) -> Result<AlertHistoryEntry, AlertError> {
        let row = sqlx::query(
            r#"
            SELECT id, alert_id, alert_name, symbol, mint, compound_condition,
                   triggered_price, conditions_met, notification_channels,
                   triggered_at, bookmarked, outcome_notes, price_after_1h,
                   price_after_4h, price_after_24h, outcome_type, created_at
            FROM alert_history
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AlertError::NotFound(id.to_string()))?;

        self.row_to_history_entry(row)
    }

    pub async fn update_history_entry(
        &self,
        id: &str,
        req: UpdateAlertHistoryRequest,
    ) -> Result<AlertHistoryEntry, AlertError> {
        let mut entry = self.get_history_entry(id).await?;

        if let Some(bookmarked) = req.bookmarked {
            entry.bookmarked = bookmarked;
        }
        if let Some(outcome_notes) = req.outcome_notes {
            entry.outcome_notes = Some(outcome_notes);
        }
        if let Some(price_after_1h) = req.price_after_1h {
            entry.price_after_1h = Some(price_after_1h);
        }
        if let Some(price_after_4h) = req.price_after_4h {
            entry.price_after_4h = Some(price_after_4h);
        }
        if let Some(price_after_24h) = req.price_after_24h {
            entry.price_after_24h = Some(price_after_24h);
        }
        if let Some(outcome_type) = req.outcome_type {
            entry.outcome_type = Some(outcome_type);
        }

        sqlx::query(
            r#"
            UPDATE alert_history
            SET bookmarked = ?1, outcome_notes = ?2, price_after_1h = ?3,
                price_after_4h = ?4, price_after_24h = ?5, outcome_type = ?6
            WHERE id = ?7
            "#,
        )
        .bind(if entry.bookmarked { 1 } else { 0 })
        .bind(&entry.outcome_notes)
        .bind(entry.price_after_1h)
        .bind(entry.price_after_4h)
        .bind(entry.price_after_24h)
        .bind(&entry.outcome_type)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(entry)
    }

    pub async fn delete_history_entry(&self, id: &str) -> Result<(), AlertError> {
        let result = sqlx::query("DELETE FROM alert_history WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AlertError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn get_statistics(&self) -> Result<AlertHistoryStats, AlertError> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_alerts,
                SUM(CASE WHEN bookmarked = 1 THEN 1 ELSE 0 END) as bookmarked_count,
                SUM(CASE WHEN outcome_type = 'profit' THEN 1 ELSE 0 END) as profit_count,
                SUM(CASE WHEN outcome_type = 'loss' THEN 1 ELSE 0 END) as loss_count,
                SUM(CASE WHEN outcome_type = 'neutral' THEN 1 ELSE 0 END) as neutral_count,
                SUM(CASE WHEN outcome_type = 'pending' THEN 1 ELSE 0 END) as pending_count,
                AVG(CASE WHEN price_after_1h IS NOT NULL THEN (price_after_1h - triggered_price) / triggered_price * 100 ELSE NULL END) as avg_change_1h,
                AVG(CASE WHEN price_after_24h IS NOT NULL THEN (price_after_24h - triggered_price) / triggered_price * 100 ELSE NULL END) as avg_change_24h
            FROM alert_history
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AlertHistoryStats {
            total_alerts: row.try_get("total_alerts")?,
            bookmarked_count: row.try_get("bookmarked_count")?,
            profit_count: row.try_get("profit_count")?,
            loss_count: row.try_get("loss_count")?,
            neutral_count: row.try_get("neutral_count")?,
            pending_count: row.try_get("pending_count")?,
            avg_price_change_1h: row.try_get("avg_change_1h").unwrap_or(0.0),
            avg_price_change_24h: row.try_get("avg_change_24h").unwrap_or(0.0),
        })
    }

    pub async fn export_to_csv(&self, filter: AlertHistoryFilter) -> Result<String, AlertError> {
        let entries = self.list_history(filter).await?;

        let mut csv = String::from("ID,Alert Name,Symbol,Triggered Price,Triggered At,Bookmarked,Outcome Type,Outcome Notes,Price After 1h,Price After 4h,Price After 24h\n");

        for entry in entries {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                entry.id,
                entry.alert_name,
                entry.symbol,
                entry.triggered_price,
                entry.triggered_at,
                entry.bookmarked,
                entry.outcome_type.unwrap_or_default(),
                entry.outcome_notes.unwrap_or_default().replace(',', ";"),
                entry.price_after_1h.map(|p| p.to_string()).unwrap_or_default(),
                entry.price_after_4h.map(|p| p.to_string()).unwrap_or_default(),
                entry.price_after_24h.map(|p| p.to_string()).unwrap_or_default(),
            ));
        }

        Ok(csv)
    }

    fn row_to_history_entry(
        &self,
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<AlertHistoryEntry, AlertError> {
        let compound_condition_json: String = row.try_get("compound_condition")?;
        let compound_condition: CompoundCondition =
            serde_json::from_str(&compound_condition_json)?;

        let channels_json: String = row.try_get("notification_channels")?;
        let notification_channels: Vec<NotificationChannel> =
            serde_json::from_str(&channels_json)?;

        let bookmarked_int: i32 = row.try_get("bookmarked")?;

        Ok(AlertHistoryEntry {
            id: row.try_get("id")?,
            alert_id: row.try_get("alert_id")?,
            alert_name: row.try_get("alert_name")?,
            symbol: row.try_get("symbol")?,
            mint: row.try_get("mint")?,
            compound_condition,
            triggered_price: row.try_get("triggered_price")?,
            conditions_met: row.try_get("conditions_met")?,
            notification_channels,
            triggered_at: row.try_get("triggered_at")?,
            bookmarked: bookmarked_int == 1,
            outcome_notes: row.try_get("outcome_notes")?,
            price_after_1h: row.try_get("price_after_1h")?,
            price_after_4h: row.try_get("price_after_4h")?,
            price_after_24h: row.try_get("price_after_24h")?,
            outcome_type: row.try_get("outcome_type")?,
            created_at: row.try_get("created_at")?,
        })
    }
}

fn alert_history_db_path(app: &AppHandle) -> Result<PathBuf, AlertError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .ok_or_else(|| AlertError::Internal("Unable to resolve app data directory".to_string()))?;

    std::fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join(ALERT_HISTORY_DB_FILE))
}

// Tauri commands
#[tauri::command]
pub async fn alert_history_list(
    manager: State<'_, crate::alerts::SharedAlertHistoryManager>,
    filter: AlertHistoryFilter,
) -> Result<Vec<AlertHistoryEntry>, String> {
    let mgr = manager.read().await;
    mgr.list_history(filter).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_history_get(
    manager: State<'_, crate::alerts::SharedAlertHistoryManager>,
    id: String,
) -> Result<AlertHistoryEntry, String> {
    let mgr = manager.read().await;
    mgr.get_history_entry(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_history_update(
    manager: State<'_, crate::alerts::SharedAlertHistoryManager>,
    id: String,
    req: UpdateAlertHistoryRequest,
) -> Result<AlertHistoryEntry, String> {
    let mgr = manager.read().await;
    mgr.update_history_entry(&id, req)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_history_delete(
    manager: State<'_, crate::alerts::SharedAlertHistoryManager>,
    id: String,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_history_entry(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_history_stats(
    manager: State<'_, crate::alerts::SharedAlertHistoryManager>,
) -> Result<AlertHistoryStats, String> {
    let mgr = manager.read().await;
    mgr.get_statistics().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_history_export_csv(
    manager: State<'_, crate::alerts::SharedAlertHistoryManager>,
    filter: AlertHistoryFilter,
) -> Result<String, String> {
    let mgr = manager.read().await;
    mgr.export_to_csv(filter).await.map_err(|e| e.to_string())
}
