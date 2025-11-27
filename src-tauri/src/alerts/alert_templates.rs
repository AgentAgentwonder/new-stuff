use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use tauri::{AppHandle, State};

use super::price_alerts::{AlertError, CompoundCondition, NotificationChannel};

const ALERT_TEMPLATES_DB_FILE: &str = "alert_templates.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub compound_condition: CompoundCondition,
    pub notification_channels: Vec<NotificationChannel>,
    pub cooldown_minutes: i32,
    pub is_builtin: bool,
    pub version: i32,
    pub category: String, // "price", "volume", "volatility", "custom"
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateRequest {
    pub name: String,
    pub description: String,
    pub compound_condition: CompoundCondition,
    pub notification_channels: Vec<NotificationChannel>,
    pub cooldown_minutes: i32,
    pub category: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub compound_condition: Option<CompoundCondition>,
    pub notification_channels: Option<Vec<NotificationChannel>>,
    pub cooldown_minutes: Option<i32>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateExport {
    pub template: AlertTemplate,
    pub exported_at: String,
    pub export_version: String,
}

pub struct AlertTemplateManager {
    pool: Pool<Sqlite>,
}

impl AlertTemplateManager {
    pub async fn new(app: &AppHandle) -> Result<Self, AlertError> {
        let db_path = alert_templates_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self { pool };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), AlertError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alert_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                compound_condition TEXT NOT NULL,
                notification_channels TEXT NOT NULL,
                cooldown_minutes INTEGER NOT NULL,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                version INTEGER NOT NULL DEFAULT 1,
                category TEXT NOT NULL,
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
            CREATE INDEX IF NOT EXISTS idx_templates_category ON alert_templates(category);
            CREATE INDEX IF NOT EXISTS idx_templates_builtin ON alert_templates(is_builtin);
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Insert builtin templates
        self.insert_builtin_templates().await?;

        Ok(())
    }

    async fn insert_builtin_templates(&self) -> Result<(), AlertError> {
        use super::price_alerts::{AlertCondition, AlertConditionType, LogicalOperator};

        let builtin_templates = vec![
            (
                "Price Breakout Above",
                "Alert when price breaks above a threshold",
                CompoundCondition {
                    conditions: vec![AlertCondition {
                        condition_type: AlertConditionType::Above,
                        value: 0.0,
                        timeframe_minutes: None,
                    }],
                    operator: LogicalOperator::And,
                },
                "price",
                vec!["breakout", "bullish"],
            ),
            (
                "Price Breakdown Below",
                "Alert when price breaks below a threshold",
                CompoundCondition {
                    conditions: vec![AlertCondition {
                        condition_type: AlertConditionType::Below,
                        value: 0.0,
                        timeframe_minutes: None,
                    }],
                    operator: LogicalOperator::And,
                },
                "price",
                vec!["breakdown", "bearish"],
            ),
            (
                "High Volume Spike",
                "Alert on significant volume increase",
                CompoundCondition {
                    conditions: vec![AlertCondition {
                        condition_type: AlertConditionType::VolumeSpike,
                        value: 1000000.0,
                        timeframe_minutes: None,
                    }],
                    operator: LogicalOperator::And,
                },
                "volume",
                vec!["volume", "activity"],
            ),
            (
                "Volatile Price Movement",
                "Alert on price change of 10% or more",
                CompoundCondition {
                    conditions: vec![AlertCondition {
                        condition_type: AlertConditionType::PercentChange,
                        value: 10.0,
                        timeframe_minutes: Some(60),
                    }],
                    operator: LogicalOperator::And,
                },
                "volatility",
                vec!["volatility", "swing"],
            ),
            (
                "Volume & Price Breakout",
                "Alert when both price and volume spike",
                CompoundCondition {
                    conditions: vec![
                        AlertCondition {
                            condition_type: AlertConditionType::Above,
                            value: 0.0,
                            timeframe_minutes: None,
                        },
                        AlertCondition {
                            condition_type: AlertConditionType::VolumeSpike,
                            value: 500000.0,
                            timeframe_minutes: None,
                        },
                    ],
                    operator: LogicalOperator::And,
                },
                "custom",
                vec!["breakout", "volume", "bullish"],
            ),
        ];

        for (name, desc, condition, category, tags) in builtin_templates {
            let exists = sqlx::query("SELECT id FROM alert_templates WHERE name = ?1 AND is_builtin = 1")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;

            if exists.is_none() {
                let id = uuid::Uuid::new_v4().to_string();
                let now = Utc::now().to_rfc3339();
                let condition_json = serde_json::to_string(&condition)?;
                let channels_json = serde_json::to_string(&vec![
                    NotificationChannel::InApp,
                    NotificationChannel::System,
                ])?;
                let tags_json = serde_json::to_string(&tags)?;

                sqlx::query(
                    r#"
                    INSERT INTO alert_templates (
                        id, name, description, compound_condition, notification_channels,
                        cooldown_minutes, is_builtin, version, category, tags,
                        created_at, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                    "#,
                )
                .bind(&id)
                .bind(name)
                .bind(desc)
                .bind(&condition_json)
                .bind(&channels_json)
                .bind(30) // default 30 min cooldown
                .bind(1) // is_builtin = true
                .bind(1) // version
                .bind(category)
                .bind(&tags_json)
                .bind(&now)
                .bind(&now)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn create_template(
        &self,
        req: CreateTemplateRequest,
    ) -> Result<AlertTemplate, AlertError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let compound_condition_json = serde_json::to_string(&req.compound_condition)?;
        let channels_json = serde_json::to_string(&req.notification_channels)?;
        let tags_json = serde_json::to_string(&req.tags)?;

        sqlx::query(
            r#"
            INSERT INTO alert_templates (
                id, name, description, compound_condition, notification_channels,
                cooldown_minutes, is_builtin, version, category, tags,
                created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&compound_condition_json)
        .bind(&channels_json)
        .bind(req.cooldown_minutes)
        .bind(0) // is_builtin = false
        .bind(1) // version
        .bind(&req.category)
        .bind(&tags_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(AlertTemplate {
            id,
            name: req.name,
            description: req.description,
            compound_condition: req.compound_condition,
            notification_channels: req.notification_channels,
            cooldown_minutes: req.cooldown_minutes,
            is_builtin: false,
            version: 1,
            category: req.category,
            tags: req.tags,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn list_templates(&self) -> Result<Vec<AlertTemplate>, AlertError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, compound_condition, notification_channels,
                   cooldown_minutes, is_builtin, version, category, tags,
                   created_at, updated_at
            FROM alert_templates
            ORDER BY is_builtin DESC, created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(self.row_to_template(row)?);
        }

        Ok(templates)
    }

    pub async fn get_template(&self, id: &str) -> Result<AlertTemplate, AlertError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, compound_condition, notification_channels,
                   cooldown_minutes, is_builtin, version, category, tags,
                   created_at, updated_at
            FROM alert_templates
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AlertError::NotFound(id.to_string()))?;

        self.row_to_template(row)
    }

    pub async fn update_template(
        &self,
        id: &str,
        req: UpdateTemplateRequest,
    ) -> Result<AlertTemplate, AlertError> {
        let mut template = self.get_template(id).await?;

        if template.is_builtin {
            return Err(AlertError::Internal(
                "Cannot update builtin templates".to_string(),
            ));
        }

        let now = Utc::now().to_rfc3339();
        template.version += 1;
        template.updated_at = now.clone();

        if let Some(name) = req.name {
            template.name = name;
        }
        if let Some(description) = req.description {
            template.description = description;
        }
        if let Some(compound_condition) = req.compound_condition {
            template.compound_condition = compound_condition;
        }
        if let Some(notification_channels) = req.notification_channels {
            template.notification_channels = notification_channels;
        }
        if let Some(cooldown_minutes) = req.cooldown_minutes {
            template.cooldown_minutes = cooldown_minutes;
        }
        if let Some(category) = req.category {
            template.category = category;
        }
        if let Some(tags) = req.tags {
            template.tags = tags;
        }

        let compound_condition_json = serde_json::to_string(&template.compound_condition)?;
        let channels_json = serde_json::to_string(&template.notification_channels)?;
        let tags_json = serde_json::to_string(&template.tags)?;

        sqlx::query(
            r#"
            UPDATE alert_templates
            SET name = ?1, description = ?2, compound_condition = ?3,
                notification_channels = ?4, cooldown_minutes = ?5, version = ?6,
                category = ?7, tags = ?8, updated_at = ?9
            WHERE id = ?10
            "#,
        )
        .bind(&template.name)
        .bind(&template.description)
        .bind(&compound_condition_json)
        .bind(&channels_json)
        .bind(template.cooldown_minutes)
        .bind(template.version)
        .bind(&template.category)
        .bind(&tags_json)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(template)
    }

    pub async fn delete_template(&self, id: &str) -> Result<(), AlertError> {
        let template = self.get_template(id).await?;

        if template.is_builtin {
            return Err(AlertError::Internal(
                "Cannot delete builtin templates".to_string(),
            ));
        }

        let result = sqlx::query("DELETE FROM alert_templates WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AlertError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn export_template(&self, id: &str) -> Result<TemplateExport, AlertError> {
        let template = self.get_template(id).await?;

        Ok(TemplateExport {
            template,
            exported_at: Utc::now().to_rfc3339(),
            export_version: "1.0.0".to_string(),
        })
    }

    pub async fn import_template(&self, export: TemplateExport) -> Result<AlertTemplate, AlertError> {
        let req = CreateTemplateRequest {
            name: export.template.name,
            description: export.template.description,
            compound_condition: export.template.compound_condition,
            notification_channels: export.template.notification_channels,
            cooldown_minutes: export.template.cooldown_minutes,
            category: export.template.category,
            tags: export.template.tags,
        };

        self.create_template(req).await
    }

    fn row_to_template(&self, row: sqlx::sqlite::SqliteRow) -> Result<AlertTemplate, AlertError> {
        let compound_condition_json: String = row.try_get("compound_condition")?;
        let compound_condition: CompoundCondition =
            serde_json::from_str(&compound_condition_json)?;

        let channels_json: String = row.try_get("notification_channels")?;
        let notification_channels: Vec<NotificationChannel> =
            serde_json::from_str(&channels_json)?;

        let tags_json: String = row.try_get("tags")?;
        let tags: Vec<String> = serde_json::from_str(&tags_json)?;

        let is_builtin_int: i32 = row.try_get("is_builtin")?;

        Ok(AlertTemplate {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            compound_condition,
            notification_channels,
            cooldown_minutes: row.try_get("cooldown_minutes")?,
            is_builtin: is_builtin_int == 1,
            version: row.try_get("version")?,
            category: row.try_get("category")?,
            tags,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

fn alert_templates_db_path(app: &AppHandle) -> Result<PathBuf, AlertError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .ok_or_else(|| AlertError::Internal("Unable to resolve app data directory".to_string()))?;

    std::fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join(ALERT_TEMPLATES_DB_FILE))
}

// Tauri commands
#[tauri::command]
pub async fn alert_template_create(
    manager: State<'_, crate::alerts::SharedAlertTemplateManager>,
    req: CreateTemplateRequest,
) -> Result<AlertTemplate, String> {
    let mgr = manager.read().await;
    mgr.create_template(req).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_template_list(
    manager: State<'_, crate::alerts::SharedAlertTemplateManager>,
) -> Result<Vec<AlertTemplate>, String> {
    let mgr = manager.read().await;
    mgr.list_templates().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_template_get(
    manager: State<'_, crate::alerts::SharedAlertTemplateManager>,
    id: String,
) -> Result<AlertTemplate, String> {
    let mgr = manager.read().await;
    mgr.get_template(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_template_update(
    manager: State<'_, crate::alerts::SharedAlertTemplateManager>,
    id: String,
    req: UpdateTemplateRequest,
) -> Result<AlertTemplate, String> {
    let mgr = manager.read().await;
    mgr.update_template(&id, req)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_template_delete(
    manager: State<'_, crate::alerts::SharedAlertTemplateManager>,
    id: String,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_template(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_template_export(
    manager: State<'_, crate::alerts::SharedAlertTemplateManager>,
    id: String,
) -> Result<TemplateExport, String> {
    let mgr = manager.read().await;
    mgr.export_template(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn alert_template_import(
    manager: State<'_, crate::alerts::SharedAlertTemplateManager>,
    export: TemplateExport,
) -> Result<AlertTemplate, String> {
    let mgr = manager.read().await;
    mgr.import_template(export).await.map_err(|e| e.to_string())
}
