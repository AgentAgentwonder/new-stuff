use super::actions::Action;
use super::conditions::{MarketData, WhaleActivity};
use super::dry_run::{execute_rule_with_dry_run, DryRunResult, DryRunSimulator};
use super::rule_engine::{AlertRule, Permission, RuleExecutionResult, RuleNode, SharedAccess};
use crate::alerts::logic::serialization::{deserialize_rule_from_json, serialize_rule_to_json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

const SMART_ALERTS_DB_FILE: &str = "smart_alerts.db";

#[derive(Debug, thiserror::Error)]
pub enum SmartAlertError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("rule serialization error: {0}")]
    RuleSerialization(#[from] crate::alerts::logic::serialization::SerializationError),

    #[error("rule not found: {0}")]
    NotFound(String),

    #[error("permission denied")]
    PermissionDenied,

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSmartRuleRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub rule_tree: RuleNode,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub team_id: Option<String>,
    #[serde(default)]
    pub shared_with: Vec<SharedAccess>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSmartRuleRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub rule_tree: Option<RuleNode>,
    #[serde(default)]
    pub actions: Option<Vec<Action>>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub symbol: Option<Option<String>>,
    #[serde(default)]
    pub owner_id: Option<Option<String>>,
    #[serde(default)]
    pub team_id: Option<Option<String>>,
    #[serde(default)]
    pub shared_with: Option<Vec<SharedAccess>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SmartRuleFilter {
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub team_id: Option<String>,
    #[serde(default)]
    pub include_disabled: bool,
    #[serde(default)]
    pub tag: Option<String>,
}

#[derive(Clone)]
pub struct SmartAlertManager {
    pool: Pool<Sqlite>,
}

pub type SharedSmartAlertManager = Arc<RwLock<SmartAlertManager>>;

impl SmartAlertManager {
    pub async fn new(app: &AppHandle) -> Result<Self, SmartAlertError> {
        let db_path = smart_alerts_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self { pool };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), SmartAlertError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS smart_alerts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                rule_json TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                symbol TEXT,
                owner_id TEXT,
                team_id TEXT,
                shared_with TEXT NOT NULL,
                tags TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_smart_alerts_owner ON smart_alerts(owner_id);
            CREATE INDEX IF NOT EXISTS idx_smart_alerts_team ON smart_alerts(team_id);
            CREATE INDEX IF NOT EXISTS idx_smart_alerts_enabled ON smart_alerts(enabled);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_rule(
        &self,
        mut req: CreateSmartRuleRequest,
    ) -> Result<AlertRule, SmartAlertError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        if req.actions.iter().any(|action| action.validate().is_err()) {
            return Err(SmartAlertError::InvalidRequest(
                "One or more actions failed validation".to_string(),
            ));
        }

        let rule = AlertRule {
            id: id.clone(),
            name: req.name.clone(),
            description: req.description.take(),
            rule_tree: req.rule_tree,
            actions: req.actions,
            enabled: req.enabled,
            symbol: req.symbol.take(),
            owner_id: req.owner_id.take(),
            team_id: req.team_id.take(),
            shared_with: req.shared_with,
            tags: req.tags,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.persist_rule(&rule).await?;

        Ok(rule)
    }

    pub async fn update_rule(
        &self,
        id: &str,
        req: UpdateSmartRuleRequest,
    ) -> Result<AlertRule, SmartAlertError> {
        let mut rule = self.get_rule(id).await?;
        let now = Utc::now().to_rfc3339();

        if let Some(name) = req.name {
            rule.name = name;
        }
        if let Some(description) = req.description {
            rule.description = description;
        }
        if let Some(rule_tree) = req.rule_tree {
            rule.rule_tree = rule_tree;
        }
        if let Some(actions) = req.actions {
            for action in &actions {
                action
                    .validate()
                    .map_err(|err| SmartAlertError::InvalidRequest(err))?;
            }
            rule.actions = actions;
        }
        if let Some(enabled) = req.enabled {
            rule.enabled = enabled;
        }
        if let Some(symbol) = req.symbol {
            rule.symbol = symbol;
        }
        if let Some(owner_id) = req.owner_id {
            rule.owner_id = owner_id;
        }
        if let Some(team_id) = req.team_id {
            rule.team_id = team_id;
        }
        if let Some(shared_with) = req.shared_with {
            rule.shared_with = shared_with;
        }
        if let Some(tags) = req.tags {
            rule.tags = tags;
        }

        rule.updated_at = now;

        self.persist_rule(&rule).await?;
        Ok(rule)
    }

    pub async fn delete_rule(&self, id: &str) -> Result<(), SmartAlertError> {
        let result = sqlx::query("DELETE FROM smart_alerts WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(SmartAlertError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn list_rules(
        &self,
        filter: Option<SmartRuleFilter>,
    ) -> Result<Vec<AlertRule>, SmartAlertError> {
        let mut query = String::from(
            "SELECT id, name, description, rule_json, enabled, symbol, owner_id, team_id, shared_with, tags, created_at, updated_at FROM smart_alerts",
        );
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(filter) = filter {
            if let Some(owner_id) = filter.owner_id {
                conditions.push("owner_id = ?".to_string());
                params.push(owner_id);
            }
            if let Some(team_id) = filter.team_id {
                conditions.push("team_id = ?".to_string());
                params.push(team_id);
            }
            if !filter.include_disabled {
                conditions.push("enabled = 1".to_string());
            }
            if let Some(tag) = filter.tag {
                conditions.push("tags LIKE ?".to_string());
                params.push(format!("%{}%", tag));
            }
        } else {
            conditions.push("enabled = 1".to_string());
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY updated_at DESC");

        let mut sql_query = sqlx::query(&query);
        for value in params {
            sql_query = sql_query.bind(value);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;
        let mut rules = Vec::new();
        for row in rows {
            rules.push(self.row_to_rule(row)?);
        }

        Ok(rules)
    }

    pub async fn get_rule(&self, id: &str) -> Result<AlertRule, SmartAlertError> {
        let row = sqlx::query(
            "SELECT id, name, description, rule_json, enabled, symbol, owner_id, team_id, shared_with, tags, created_at, updated_at FROM smart_alerts WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or_else(|| SmartAlertError::NotFound(id.to_string()))?;
        self.row_to_rule(row)
    }

    pub async fn dry_run(
        &self,
        id: &str,
        market_data: MarketData,
        whale_activity: Option<WhaleActivity>,
    ) -> Result<DryRunResult, SmartAlertError> {
        let rule = self.get_rule(id).await?;
        Ok(DryRunSimulator::simulate_rule(
            &rule,
            &market_data,
            &whale_activity,
        ))
    }

    pub async fn execute(
        &self,
        id: &str,
        market_data: MarketData,
        whale_activity: Option<WhaleActivity>,
        dry_run: bool,
    ) -> Result<RuleExecutionResult, SmartAlertError> {
        let rule = self.get_rule(id).await?;
        Ok(execute_rule_with_dry_run(
            &rule,
            &market_data,
            &whale_activity,
            dry_run,
        ))
    }

    async fn persist_rule(&self, rule: &AlertRule) -> Result<(), SmartAlertError> {
        let rule_json = serialize_rule_to_json(rule)?;
        let shared_with_json = serde_json::to_string(&rule.shared_with)?;
        let tags_json = serde_json::to_string(&rule.tags)?;

        sqlx::query(
            r#"
            INSERT INTO smart_alerts (
                id, name, description, rule_json, enabled, symbol,
                owner_id, team_id, shared_with, tags, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                rule_json = excluded.rule_json,
                enabled = excluded.enabled,
                symbol = excluded.symbol,
                owner_id = excluded.owner_id,
                team_id = excluded.team_id,
                shared_with = excluded.shared_with,
                tags = excluded.tags,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule_json)
        .bind(rule.enabled)
        .bind(&rule.symbol)
        .bind(&rule.owner_id)
        .bind(&rule.team_id)
        .bind(&shared_with_json)
        .bind(&tags_json)
        .bind(&rule.created_at)
        .bind(&rule.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn row_to_rule(&self, row: sqlx::sqlite::SqliteRow) -> Result<AlertRule, SmartAlertError> {
        let rule_json: String = row.try_get("rule_json")?;
        let mut rule = deserialize_rule_from_json(&rule_json)?;

        rule.id = row.try_get("id")?;
        rule.name = row.try_get("name")?;
        rule.description = row.try_get("description")?;
        rule.enabled = row.try_get::<i64, _>("enabled")? == 1;
        rule.symbol = row.try_get("symbol")?;
        rule.owner_id = row.try_get("owner_id")?;
        rule.team_id = row.try_get("team_id")?;

        let shared_with_json: String = row.try_get("shared_with")?;
        rule.shared_with = serde_json::from_str(&shared_with_json)?;

        let tags_json: String = row.try_get("tags")?;
        rule.tags = serde_json::from_str(&tags_json)?;

        rule.created_at = row.try_get("created_at")?;
        rule.updated_at = row.try_get("updated_at")?;

        Ok(rule)
    }
}

fn smart_alerts_db_path(app: &AppHandle) -> Result<PathBuf, SmartAlertError> {
    let app_handle = app.clone();
    let mut app_data_dir = app_handle.path().app_data_dir().map_err(|err| {
        SmartAlertError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;

    std::fs::create_dir_all(&app_data_dir)?;
    app_data_dir.push(SMART_ALERTS_DB_FILE);
    Ok(app_data_dir)
}

// Tauri Commands

#[tauri::command]
pub async fn smart_alert_create_rule(
    manager: State<'_, SharedSmartAlertManager>,
    req: CreateSmartRuleRequest,
) -> Result<AlertRule, String> {
    let mgr = manager.write().await;
    mgr.create_rule(req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn smart_alert_update_rule(
    manager: State<'_, SharedSmartAlertManager>,
    id: String,
    req: UpdateSmartRuleRequest,
) -> Result<AlertRule, String> {
    let mgr = manager.write().await;
    mgr.update_rule(&id, req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn smart_alert_delete_rule(
    manager: State<'_, SharedSmartAlertManager>,
    id: String,
) -> Result<(), String> {
    let mgr = manager.write().await;
    mgr.delete_rule(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn smart_alert_list_rules(
    manager: State<'_, SharedSmartAlertManager>,
    filter: Option<SmartRuleFilter>,
) -> Result<Vec<AlertRule>, String> {
    let mgr = manager.read().await;
    mgr.list_rules(filter).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn smart_alert_get_rule(
    manager: State<'_, SharedSmartAlertManager>,
    id: String,
) -> Result<AlertRule, String> {
    let mgr = manager.read().await;
    mgr.get_rule(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn smart_alert_dry_run(
    manager: State<'_, SharedSmartAlertManager>,
    id: String,
    market_data: MarketData,
    whale_activity: Option<WhaleActivity>,
) -> Result<DryRunResult, String> {
    let mgr = manager.read().await;
    mgr.dry_run(&id, market_data, whale_activity)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn smart_alert_execute(
    manager: State<'_, SharedSmartAlertManager>,
    id: String,
    market_data: MarketData,
    whale_activity: Option<WhaleActivity>,
    dry_run: bool,
) -> Result<RuleExecutionResult, String> {
    let mgr = manager.read().await;
    mgr.execute(&id, market_data, whale_activity, dry_run)
        .await
        .map_err(|e| e.to_string())
}
