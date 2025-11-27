use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use sysinfo::CpuExt;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::RwLock;

const ALERTS_DB_FILE: &str = "price_alerts.db";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertConditionType {
    Above,
    Below,
    PercentChange,
    VolumeSpike,
}

impl AlertConditionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertConditionType::Above => "above",
            AlertConditionType::Below => "below",
            AlertConditionType::PercentChange => "percent_change",
            AlertConditionType::VolumeSpike => "volume_spike",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "above" => Some(AlertConditionType::Above),
            "below" => Some(AlertConditionType::Below),
            "percent_change" => Some(AlertConditionType::PercentChange),
            "volume_spike" => Some(AlertConditionType::VolumeSpike),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperator {
    And,
    Or,
}

impl LogicalOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogicalOperator::And => "and",
            LogicalOperator::Or => "or",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "and" => Some(LogicalOperator::And),
            "or" => Some(LogicalOperator::Or),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertCondition {
    pub condition_type: AlertConditionType,
    pub value: f64,
    pub timeframe_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompoundCondition {
    pub conditions: Vec<AlertCondition>,
    pub operator: LogicalOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertState {
    Active,
    Triggered,
    Cooldown,
    Disabled,
}

impl AlertState {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertState::Active => "active",
            AlertState::Triggered => "triggered",
            AlertState::Cooldown => "cooldown",
            AlertState::Disabled => "disabled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(AlertState::Active),
            "triggered" => Some(AlertState::Triggered),
            "cooldown" => Some(AlertState::Cooldown),
            "disabled" => Some(AlertState::Disabled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannel {
    InApp,
    System,
    Email,
    Webhook,
    Telegram,
    Slack,
    Discord,
}

impl NotificationChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationChannel::InApp => "in_app",
            NotificationChannel::System => "system",
            NotificationChannel::Email => "email",
            NotificationChannel::Webhook => "webhook",
            NotificationChannel::Telegram => "telegram",
            NotificationChannel::Slack => "slack",
            NotificationChannel::Discord => "discord",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "in_app" => Some(NotificationChannel::InApp),
            "system" => Some(NotificationChannel::System),
            "email" => Some(NotificationChannel::Email),
            "webhook" => Some(NotificationChannel::Webhook),
            "telegram" => Some(NotificationChannel::Telegram),
            "slack" => Some(NotificationChannel::Slack),
            "discord" => Some(NotificationChannel::Discord),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceAlert {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub mint: String,
    pub watchlist_id: Option<String>,
    pub compound_condition: CompoundCondition,
    pub notification_channels: Vec<NotificationChannel>,
    pub cooldown_minutes: i32,
    pub state: AlertState,
    pub last_triggered_at: Option<String>,
    pub cooldown_until: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertRequest {
    pub name: String,
    pub symbol: String,
    pub mint: String,
    pub watchlist_id: Option<String>,
    pub compound_condition: CompoundCondition,
    pub notification_channels: Vec<NotificationChannel>,
    pub cooldown_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAlertRequest {
    pub name: Option<String>,
    pub compound_condition: Option<CompoundCondition>,
    pub notification_channels: Option<Vec<NotificationChannel>>,
    pub cooldown_minutes: Option<i32>,
    pub state: Option<AlertState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertTestResult {
    pub alert_id: String,
    pub would_trigger: bool,
    pub conditions_met: Vec<bool>,
    pub current_price: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertTriggerEvent {
    pub alert_id: String,
    pub alert_name: String,
    pub symbol: String,
    pub current_price: f64,
    pub conditions_met: String,
    pub triggered_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AlertError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("alert not found: {0}")]
    NotFound(String),
    #[error("alert in cooldown until: {0}")]
    InCooldown(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Clone)]
pub struct AlertManager {
    pool: Pool<Sqlite>,
    app_handle: AppHandle,
}

pub type SharedAlertManager = Arc<RwLock<AlertManager>>;

impl AlertManager {
    pub async fn new(app: &AppHandle) -> Result<Self, AlertError> {
        let db_path = alerts_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self {
            pool,
            app_handle: app.clone(),
        };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), AlertError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS price_alerts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                mint TEXT NOT NULL,
                watchlist_id TEXT,
                compound_condition TEXT NOT NULL,
                notification_channels TEXT NOT NULL,
                cooldown_minutes INTEGER NOT NULL,
                state TEXT NOT NULL,
                last_triggered_at TEXT,
                cooldown_until TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_alerts_symbol ON price_alerts(symbol);
            CREATE INDEX IF NOT EXISTS idx_alerts_state ON price_alerts(state);
            CREATE INDEX IF NOT EXISTS idx_alerts_watchlist ON price_alerts(watchlist_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_alert(&self, req: CreateAlertRequest) -> Result<PriceAlert, AlertError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let compound_condition_json = serde_json::to_string(&req.compound_condition)?;
        let channels_json = serde_json::to_string(&req.notification_channels)?;

        sqlx::query(
            r#"
            INSERT INTO price_alerts (
                id, name, symbol, mint, watchlist_id, compound_condition,
                notification_channels, cooldown_minutes, state,
                last_triggered_at, cooldown_until, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.symbol)
        .bind(&req.mint)
        .bind(&req.watchlist_id)
        .bind(&compound_condition_json)
        .bind(&channels_json)
        .bind(req.cooldown_minutes)
        .bind(AlertState::Active.as_str())
        .bind::<Option<String>>(None)
        .bind::<Option<String>>(None)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(PriceAlert {
            id,
            name: req.name,
            symbol: req.symbol,
            mint: req.mint,
            watchlist_id: req.watchlist_id,
            compound_condition: req.compound_condition,
            notification_channels: req.notification_channels,
            cooldown_minutes: req.cooldown_minutes,
            state: AlertState::Active,
            last_triggered_at: None,
            cooldown_until: None,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn list_alerts(&self) -> Result<Vec<PriceAlert>, AlertError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, symbol, mint, watchlist_id, compound_condition,
                   notification_channels, cooldown_minutes, state,
                   last_triggered_at, cooldown_until, created_at, updated_at
            FROM price_alerts
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(self.row_to_alert(row)?);
        }

        Ok(alerts)
    }

    pub async fn get_alert(&self, id: &str) -> Result<PriceAlert, AlertError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, symbol, mint, watchlist_id, compound_condition,
                   notification_channels, cooldown_minutes, state,
                   last_triggered_at, cooldown_until, created_at, updated_at
            FROM price_alerts
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AlertError::NotFound(id.to_string()))?;

        self.row_to_alert(row)
    }

    pub async fn update_alert(
        &self,
        id: &str,
        req: UpdateAlertRequest,
    ) -> Result<PriceAlert, AlertError> {
        let mut alert = self.get_alert(id).await?;
        let now = Utc::now().to_rfc3339();

        if let Some(name) = req.name {
            alert.name = name;
        }
        if let Some(compound_condition) = req.compound_condition {
            alert.compound_condition = compound_condition;
        }
        if let Some(notification_channels) = req.notification_channels {
            alert.notification_channels = notification_channels;
        }
        if let Some(cooldown_minutes) = req.cooldown_minutes {
            alert.cooldown_minutes = cooldown_minutes;
        }
        if let Some(state) = req.state {
            alert.state = state;
        }

        alert.updated_at = now.clone();

        let compound_condition_json = serde_json::to_string(&alert.compound_condition)?;
        let channels_json = serde_json::to_string(&alert.notification_channels)?;

        sqlx::query(
            r#"
            UPDATE price_alerts
            SET name = ?1, compound_condition = ?2, notification_channels = ?3,
                cooldown_minutes = ?4, state = ?5, updated_at = ?6
            WHERE id = ?7
            "#,
        )
        .bind(&alert.name)
        .bind(&compound_condition_json)
        .bind(&channels_json)
        .bind(alert.cooldown_minutes)
        .bind(alert.state.as_str())
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(alert)
    }

    pub async fn delete_alert(&self, id: &str) -> Result<(), AlertError> {
        let result = sqlx::query("DELETE FROM price_alerts WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AlertError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn test_alert(
        &self,
        id: &str,
        current_price: f64,
        price_24h_ago: Option<f64>,
        volume_24h: Option<f64>,
    ) -> Result<AlertTestResult, AlertError> {
        let alert = self.get_alert(id).await?;

        let (would_trigger, conditions_met, message) = self.evaluate_conditions(
            &alert.compound_condition,
            current_price,
            price_24h_ago,
            volume_24h,
        );

        Ok(AlertTestResult {
            alert_id: alert.id,
            would_trigger,
            conditions_met,
            current_price,
            message,
        })
    }

    pub async fn check_and_trigger_alerts(
        &self,
        symbol: &str,
        current_price: f64,
        price_24h_ago: Option<f64>,
        volume_24h: Option<f64>,
    ) -> Result<Vec<String>, AlertError> {
        let now = Utc::now();
        let rows = sqlx::query(
            r#"
            SELECT id, name, symbol, mint, watchlist_id, compound_condition,
                   notification_channels, cooldown_minutes, state,
                   last_triggered_at, cooldown_until, created_at, updated_at
            FROM price_alerts
            WHERE symbol = ?1 AND state = ?2
            "#,
        )
        .bind(symbol)
        .bind(AlertState::Active.as_str())
        .fetch_all(&self.pool)
        .await?;

        let mut triggered_alerts = Vec::new();

        for row in rows {
            let alert = self.row_to_alert(row)?;

            if let Some(cooldown_until_str) = &alert.cooldown_until {
                if let Ok(cooldown_until) = DateTime::parse_from_rfc3339(cooldown_until_str) {
                    if now < cooldown_until.with_timezone(&Utc) {
                        continue;
                    }
                }
            }

            let (would_trigger, conditions_met, message) = self.evaluate_conditions(
                &alert.compound_condition,
                current_price,
                price_24h_ago,
                volume_24h,
            );

            if would_trigger {
                self.trigger_alert(&alert, current_price, &message).await?;
                triggered_alerts.push(alert.id.clone());
            }
        }

        Ok(triggered_alerts)
    }

    async fn trigger_alert(
        &self,
        alert: &PriceAlert,
        current_price: f64,
        message: &str,
    ) -> Result<(), AlertError> {
        let now = Utc::now();
        let cooldown_until = now + Duration::minutes(alert.cooldown_minutes as i64);

        sqlx::query(
            r#"
            UPDATE price_alerts
            SET state = ?1, last_triggered_at = ?2, cooldown_until = ?3, updated_at = ?4
            WHERE id = ?5
            "#,
        )
        .bind(AlertState::Cooldown.as_str())
        .bind(now.to_rfc3339())
        .bind(cooldown_until.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(&alert.id)
        .execute(&self.pool)
        .await?;

        let event = AlertTriggerEvent {
            alert_id: alert.id.clone(),
            alert_name: alert.name.clone(),
            symbol: alert.symbol.clone(),
            current_price,
            conditions_met: message.to_string(),
            triggered_at: now.to_rfc3339(),
        };

        self.app_handle
            .emit("alert_triggered", event)
            .map_err(|e| AlertError::Internal(format!("Failed to emit event: {}", e)))?;

        Ok(())
    }

    pub async fn reset_cooldowns(&self) -> Result<usize, AlertError> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE price_alerts
            SET state = ?1, cooldown_until = NULL, updated_at = ?2
            WHERE state = ?3 AND (cooldown_until IS NULL OR cooldown_until <= ?4)
            "#,
        )
        .bind(AlertState::Active.as_str())
        .bind(&now)
        .bind(AlertState::Cooldown.as_str())
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    fn evaluate_conditions(
        &self,
        compound: &CompoundCondition,
        current_price: f64,
        price_24h_ago: Option<f64>,
        volume_24h: Option<f64>,
    ) -> (bool, Vec<bool>, String) {
        let mut results = Vec::new();
        let mut messages = Vec::new();

        for condition in &compound.conditions {
            let (met, msg) = match condition.condition_type {
                AlertConditionType::Above => {
                    let met = current_price > condition.value;
                    let msg = format!(
                        "Price {} threshold ${:.2}",
                        if met { "above" } else { "not above" },
                        condition.value
                    );
                    (met, msg)
                }
                AlertConditionType::Below => {
                    let met = current_price < condition.value;
                    let msg = format!(
                        "Price {} threshold ${:.2}",
                        if met { "below" } else { "not below" },
                        condition.value
                    );
                    (met, msg)
                }
                AlertConditionType::PercentChange => {
                    if let Some(price_24h) = price_24h_ago {
                        let percent_change = ((current_price - price_24h) / price_24h) * 100.0;
                        let met = percent_change.abs() >= condition.value;
                        let msg = format!(
                            "Price change {:.2}% {} threshold {:.2}%",
                            percent_change,
                            if met { "exceeds" } else { "below" },
                            condition.value
                        );
                        (met, msg)
                    } else {
                        (false, "Insufficient price history".to_string())
                    }
                }
                AlertConditionType::VolumeSpike => {
                    if let Some(volume) = volume_24h {
                        let met = volume >= condition.value;
                        let msg = format!(
                            "Volume ${:.0} {} threshold ${:.0}",
                            volume,
                            if met { "exceeds" } else { "below" },
                            condition.value
                        );
                        (met, msg)
                    } else {
                        (false, "Volume data unavailable".to_string())
                    }
                }
            };

            results.push(met);
            messages.push(msg);
        }

        let would_trigger = match compound.operator {
            LogicalOperator::And => results.iter().all(|&x| x),
            LogicalOperator::Or => results.iter().any(|&x| x),
        };

        let message = messages.join("; ");

        (would_trigger, results, message)
    }

    fn row_to_alert(&self, row: sqlx::sqlite::SqliteRow) -> Result<PriceAlert, AlertError> {
        let compound_condition_json: String = row.try_get("compound_condition")?;
        let compound_condition: CompoundCondition = serde_json::from_str(&compound_condition_json)?;

        let channels_json: String = row.try_get("notification_channels")?;
        let notification_channels: Vec<NotificationChannel> = serde_json::from_str(&channels_json)?;

        let state_str: String = row.try_get("state")?;
        let state = AlertState::from_str(&state_str)
            .ok_or_else(|| AlertError::Internal(format!("Invalid state: {}", state_str)))?;

        Ok(PriceAlert {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            symbol: row.try_get("symbol")?,
            mint: row.try_get("mint")?,
            watchlist_id: row.try_get("watchlist_id")?,
            compound_condition,
            notification_channels,
            cooldown_minutes: row.try_get("cooldown_minutes")?,
            state,
            last_triggered_at: row.try_get("last_triggered_at")?,
            cooldown_until: row.try_get("cooldown_until")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

fn alerts_db_path(app: &AppHandle) -> Result<PathBuf, AlertError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AlertError::Internal(format!("Unable to resolve app data directory: {}", e)))?;

    std::fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join(ALERTS_DB_FILE))
}

// Tauri commands
#[tauri::command]
pub async fn alert_create(
    manager: State<'_, SharedAlertManager>,
    req: CreateAlertRequest,
) -> Result<PriceAlert, String> {
    let mgr = manager.read().await;
    mgr.create_alert(req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_list(manager: State<'_, SharedAlertManager>) -> Result<Vec<PriceAlert>, String> {
    let mgr = manager.read().await;
    mgr.list_alerts().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_get(
    manager: State<'_, SharedAlertManager>,
    id: String,
) -> Result<PriceAlert, String> {
    let mgr = manager.read().await;
    mgr.get_alert(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_update(
    manager: State<'_, SharedAlertManager>,
    id: String,
    req: UpdateAlertRequest,
) -> Result<PriceAlert, String> {
    let mgr = manager.read().await;
    mgr.update_alert(&id, req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_delete(
    manager: State<'_, SharedAlertManager>,
    id: String,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_alert(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_test(
    manager: State<'_, SharedAlertManager>,
    id: String,
    current_price: f64,
    price_24h_ago: Option<f64>,
    volume_24h: Option<f64>,
) -> Result<AlertTestResult, String> {
    let mgr = manager.read().await;
    mgr.test_alert(&id, current_price, price_24h_ago, volume_24h)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_check_triggers(
    manager: State<'_, SharedAlertManager>,
    symbol: String,
    current_price: f64,
    price_24h_ago: Option<f64>,
    volume_24h: Option<f64>,
) -> Result<Vec<String>, String> {
    let mgr = manager.read().await;
    mgr.check_and_trigger_alerts(&symbol, current_price, price_24h_ago, volume_24h)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_reset_cooldowns(
    manager: State<'_, SharedAlertManager>,
) -> Result<usize, String> {
    let mgr = manager.read().await;
    mgr.reset_cooldowns().await.map_err(|e| e.to_string())
}
