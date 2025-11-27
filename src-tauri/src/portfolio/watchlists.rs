use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

const WATCHLIST_DB_FILE: &str = "watchlists.db";
const MAX_WATCHLISTS: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchlistItem {
    pub symbol: String,
    pub mint: String,
    pub position: i32,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Watchlist {
    pub id: String,
    pub name: String,
    pub items: Vec<WatchlistItem>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchlistStats {
    pub total_value: f64,
    pub change_24h: f64,
    pub change_24h_percent: f64,
    pub top_gainer: Option<String>,
    pub top_loser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchlistWithStats {
    #[serde(flatten)]
    pub watchlist: Watchlist,
    pub stats: Option<WatchlistStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWatchlistRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWatchlistRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddItemRequest {
    pub symbol: String,
    pub mint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderItemsRequest {
    pub items: Vec<ReorderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderItem {
    pub symbol: String,
    pub mint: String,
    pub position: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchlistError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("watchlist not found: {0}")]
    NotFound(String),
    #[error("maximum watchlists reached: {0}")]
    MaxWatchlistsReached(usize),
    #[error("duplicate item: {0}")]
    DuplicateItem(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Clone)]
pub struct WatchlistManager {
    pool: Pool<Sqlite>,
}

pub type SharedWatchlistManager = Arc<RwLock<WatchlistManager>>;

impl WatchlistManager {
    pub async fn new(app: &AppHandle) -> Result<Self, WatchlistError> {
        let db_path = watchlist_db_path(app)?;
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let manager = Self { pool };
        manager.initialize().await?;
        Ok(manager)
    }

    async fn initialize(&self) -> Result<(), WatchlistError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS watchlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS watchlist_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                watchlist_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                mint TEXT NOT NULL,
                position INTEGER NOT NULL,
                added_at TEXT NOT NULL,
                FOREIGN KEY (watchlist_id) REFERENCES watchlists(id) ON DELETE CASCADE,
                UNIQUE(watchlist_id, mint)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_watchlist_items_watchlist_id 
            ON watchlist_items(watchlist_id);
            CREATE INDEX IF NOT EXISTS idx_watchlist_items_position 
            ON watchlist_items(watchlist_id, position);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_watchlist(&self, name: String) -> Result<Watchlist, WatchlistError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM watchlists")
            .fetch_one(&self.pool)
            .await?;

        if count >= MAX_WATCHLISTS as i64 {
            return Err(WatchlistError::MaxWatchlistsReached(MAX_WATCHLISTS));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO watchlists (id, name, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&id)
        .bind(&name)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(Watchlist {
            id,
            name,
            items: vec![],
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn list_watchlists(&self) -> Result<Vec<Watchlist>, WatchlistError> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, created_at, updated_at
            FROM watchlists
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut watchlists = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let items = self.get_watchlist_items(&id).await?;

            watchlists.push(Watchlist {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                items,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(watchlists)
    }

    pub async fn get_watchlist(&self, id: &str) -> Result<Watchlist, WatchlistError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, created_at, updated_at
            FROM watchlists
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| WatchlistError::NotFound(id.to_string()))?;

        let items = self.get_watchlist_items(id).await?;

        Ok(Watchlist {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            items,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn update_watchlist(
        &self,
        id: &str,
        name: String,
    ) -> Result<Watchlist, WatchlistError> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            UPDATE watchlists
            SET name = ?1, updated_at = ?2
            WHERE id = ?3
            "#,
        )
        .bind(&name)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(WatchlistError::NotFound(id.to_string()));
        }

        self.get_watchlist(id).await
    }

    pub async fn delete_watchlist(&self, id: &str) -> Result<(), WatchlistError> {
        let result = sqlx::query("DELETE FROM watchlists WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(WatchlistError::NotFound(id.to_string()));
        }

        sqlx::query("DELETE FROM watchlist_items WHERE watchlist_id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_item(
        &self,
        watchlist_id: &str,
        symbol: String,
        mint: String,
    ) -> Result<Watchlist, WatchlistError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM watchlist_items WHERE watchlist_id = ?1 AND mint = ?2)",
        )
        .bind(watchlist_id)
        .bind(&mint)
        .fetch_one(&self.pool)
        .await?;

        if exists {
            return Err(WatchlistError::DuplicateItem(mint));
        }

        let max_position: Option<i32> =
            sqlx::query_scalar("SELECT MAX(position) FROM watchlist_items WHERE watchlist_id = ?1")
                .bind(watchlist_id)
                .fetch_one(&self.pool)
                .await?;

        let position = max_position.map(|p| p + 1).unwrap_or(0);
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO watchlist_items (watchlist_id, symbol, mint, position, added_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(watchlist_id)
        .bind(&symbol)
        .bind(&mint)
        .bind(position)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        sqlx::query("UPDATE watchlists SET updated_at = ?1 WHERE id = ?2")
            .bind(&now)
            .bind(watchlist_id)
            .execute(&self.pool)
            .await?;

        self.get_watchlist(watchlist_id).await
    }

    pub async fn remove_item(
        &self,
        watchlist_id: &str,
        mint: &str,
    ) -> Result<Watchlist, WatchlistError> {
        let result =
            sqlx::query("DELETE FROM watchlist_items WHERE watchlist_id = ?1 AND mint = ?2")
                .bind(watchlist_id)
                .bind(mint)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(WatchlistError::NotFound(format!(
                "Item {} in watchlist {}",
                mint, watchlist_id
            )));
        }

        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE watchlists SET updated_at = ?1 WHERE id = ?2")
            .bind(&now)
            .bind(watchlist_id)
            .execute(&self.pool)
            .await?;

        self.get_watchlist(watchlist_id).await
    }

    pub async fn reorder_items(
        &self,
        watchlist_id: &str,
        items: Vec<ReorderItem>,
    ) -> Result<Watchlist, WatchlistError> {
        let mut tx = self.pool.begin().await?;

        for item in items {
            sqlx::query(
                r#"
                UPDATE watchlist_items
                SET position = ?1
                WHERE watchlist_id = ?2 AND mint = ?3
                "#,
            )
            .bind(item.position)
            .bind(watchlist_id)
            .bind(&item.mint)
            .execute(&mut *tx)
            .await?;
        }

        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE watchlists SET updated_at = ?1 WHERE id = ?2")
            .bind(&now)
            .bind(watchlist_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        self.get_watchlist(watchlist_id).await
    }

    async fn get_watchlist_items(
        &self,
        watchlist_id: &str,
    ) -> Result<Vec<WatchlistItem>, WatchlistError> {
        let rows = sqlx::query(
            r#"
            SELECT symbol, mint, position, added_at
            FROM watchlist_items
            WHERE watchlist_id = ?1
            ORDER BY position ASC
            "#,
        )
        .bind(watchlist_id)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::new();
        for row in rows {
            items.push(WatchlistItem {
                symbol: row.try_get("symbol")?,
                mint: row.try_get("mint")?,
                position: row.try_get("position")?,
                added_at: row.try_get("added_at")?,
            });
        }

        Ok(items)
    }

    pub async fn export_watchlist(&self, id: &str) -> Result<String, WatchlistError> {
        let watchlist = self.get_watchlist(id).await?;
        let json = serde_json::to_string_pretty(&watchlist)?;
        Ok(json)
    }

    pub async fn import_watchlist(&self, data: String) -> Result<Watchlist, WatchlistError> {
        let mut watchlist: Watchlist = serde_json::from_str(&data)?;

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM watchlists")
            .fetch_one(&self.pool)
            .await?;

        if count >= MAX_WATCHLISTS as i64 {
            return Err(WatchlistError::MaxWatchlistsReached(MAX_WATCHLISTS));
        }

        let new_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO watchlists (id, name, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(&new_id)
        .bind(&watchlist.name)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        for item in &watchlist.items {
            sqlx::query(
                r#"
                INSERT INTO watchlist_items (watchlist_id, symbol, mint, position, added_at)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
            )
            .bind(&new_id)
            .bind(&item.symbol)
            .bind(&item.mint)
            .bind(item.position)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        }

        watchlist.id = new_id;
        watchlist.created_at = now.clone();
        watchlist.updated_at = now;

        Ok(watchlist)
    }
}

fn watchlist_db_path(app: &AppHandle) -> Result<PathBuf, WatchlistError> {
    let app_data_dir = app.path().app_data_dir().map_err(|err| {
        WatchlistError::Internal(format!("Unable to resolve app data directory: {err}"))
    })?;

    std::fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join(WATCHLIST_DB_FILE))
}

// Tauri commands
#[tauri::command]
pub async fn watchlist_create(
    manager: State<'_, SharedWatchlistManager>,
    name: String,
) -> Result<Watchlist, String> {
    let mgr = manager.read().await;
    mgr.create_watchlist(name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_list(
    manager: State<'_, SharedWatchlistManager>,
) -> Result<Vec<Watchlist>, String> {
    let mgr = manager.read().await;
    mgr.list_watchlists().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_get(
    manager: State<'_, SharedWatchlistManager>,
    id: String,
) -> Result<Watchlist, String> {
    let mgr = manager.read().await;
    mgr.get_watchlist(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_update(
    manager: State<'_, SharedWatchlistManager>,
    id: String,
    name: String,
) -> Result<Watchlist, String> {
    let mgr = manager.read().await;
    mgr.update_watchlist(&id, name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_delete(
    manager: State<'_, SharedWatchlistManager>,
    id: String,
) -> Result<(), String> {
    let mgr = manager.read().await;
    mgr.delete_watchlist(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_add_item(
    manager: State<'_, SharedWatchlistManager>,
    watchlist_id: String,
    symbol: String,
    mint: String,
) -> Result<Watchlist, String> {
    let mgr = manager.read().await;
    mgr.add_item(&watchlist_id, symbol, mint)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_remove_item(
    manager: State<'_, SharedWatchlistManager>,
    watchlist_id: String,
    mint: String,
) -> Result<Watchlist, String> {
    let mgr = manager.read().await;
    mgr.remove_item(&watchlist_id, &mint)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_reorder_items(
    manager: State<'_, SharedWatchlistManager>,
    watchlist_id: String,
    items: Vec<ReorderItem>,
) -> Result<Watchlist, String> {
    let mgr = manager.read().await;
    mgr.reorder_items(&watchlist_id, items)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_export(
    manager: State<'_, SharedWatchlistManager>,
    id: String,
) -> Result<String, String> {
    let mgr = manager.read().await;
    mgr.export_watchlist(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn watchlist_import(
    manager: State<'_, SharedWatchlistManager>,
    data: String,
) -> Result<Watchlist, String> {
    let mgr = manager.read().await;
    mgr.import_watchlist(data).await.map_err(|e| e.to_string())
}
