// Feature Flags System
// Runtime feature flag checking and management

use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct FeatureFlags {
    db: SqlitePool,
    cache: RwLock<HashMap<String, bool>>,
}

impl FeatureFlags {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Check if a feature is enabled
    pub async fn is_enabled(&self, feature_name: &str) -> bool {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(&enabled) = cache.get(feature_name) {
                return enabled;
            }
        }

        // Query database
        match self.fetch_flag(feature_name).await {
            Ok(enabled) => {
                // Update cache
                let mut cache = self.cache.write().await;
                cache.insert(feature_name.to_string(), enabled);
                enabled
            }
            Err(e) => {
                log::warn!("Failed to fetch feature flag '{}': {}", feature_name, e);
                false // Default to disabled on error
            }
        }
    }

    /// Enable a feature flag
    pub async fn enable_feature(&self, feature_name: &str) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE feature_flags
            SET enabled = 1, updated_at = datetime('now')
            WHERE feature_name = ?
            "#
        )
        .bind(feature_name)
        .execute(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        // Clear cache
        let mut cache = self.cache.write().await;
        cache.remove(feature_name);

        log::info!("Feature '{}' enabled", feature_name);
        Ok(())
    }

    /// Disable a feature flag
    pub async fn disable_feature(&self, feature_name: &str) -> Result<(), String> {
        sqlx::query(
            r#"
            UPDATE feature_flags
            SET enabled = 0, updated_at = datetime('now')
            WHERE feature_name = ?
            "#
        )
        .bind(feature_name)
        .execute(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        // Clear cache
        let mut cache = self.cache.write().await;
        cache.remove(feature_name);

        log::info!("Feature '{}' disabled", feature_name);
        Ok(())
    }

    /// Get all feature flags
    pub async fn get_all_flags(&self) -> Result<Vec<FeatureFlag>, String> {
        let flags = sqlx::query_as::<_, FeatureFlagRow>(
            r#"
            SELECT feature_name, enabled, description, rollout_percentage
            FROM feature_flags
            ORDER BY feature_name
            "#
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        Ok(flags.into_iter().map(|row| row.into()).collect())
    }

    async fn fetch_flag(&self, feature_name: &str) -> Result<bool, sqlx::Error> {
        let row: (i32,) = sqlx::query_as(
            r#"
            SELECT enabled FROM feature_flags WHERE feature_name = ?
            "#
        )
        .bind(feature_name)
        .fetch_one(&self.db)
        .await?;

        Ok(row.0 == 1)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub rollout_percentage: i32,
}

#[derive(sqlx::FromRow)]
struct FeatureFlagRow {
    feature_name: String,
    enabled: i32,
    description: Option<String>,
    rollout_percentage: i32,
}

impl From<FeatureFlagRow> for FeatureFlag {
    fn from(row: FeatureFlagRow) -> Self {
        Self {
            name: row.feature_name,
            enabled: row.enabled == 1,
            description: row.description,
            rollout_percentage: row.rollout_percentage,
        }
    }
}

// Tauri commands

#[tauri::command]
pub async fn get_feature_flags(
    flags: tauri::State<'_, FeatureFlags>,
) -> Result<Vec<FeatureFlag>, String> {
    flags.get_all_flags().await
}

#[tauri::command]
pub async fn enable_feature_flag(
    feature_name: String,
    flags: tauri::State<'_, FeatureFlags>,
) -> Result<(), String> {
    flags.enable_feature(&feature_name).await
}

#[tauri::command]
pub async fn disable_feature_flag(
    feature_name: String,
    flags: tauri::State<'_, FeatureFlags>,
) -> Result<(), String> {
    flags.disable_feature(&feature_name).await
}

#[tauri::command]
pub async fn is_feature_enabled(
    feature_name: String,
    flags: tauri::State<'_, FeatureFlags>,
) -> Result<bool, String> {
    Ok(flags.is_enabled(&feature_name).await)
}
