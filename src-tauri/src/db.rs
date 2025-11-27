use duckdb::Connection;
use sqlx::{SqlitePool, Sqlite};
use std::path::Path;

pub struct MarketDB {
    conn: Connection
}

impl MarketDB {
    pub fn new(path: &Path) -> Self {
        let conn = Connection::open(path).unwrap();
        Self { conn }
    }

    pub fn initialize(&self) {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS trades (
                id INTEGER PRIMARY KEY,
                timestamp DATETIME,
                pair TEXT,
                amount REAL,
                price REAL
            )"
        ).unwrap();
    }
}

pub async fn create_sqlite_db(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
        }
    }

    let db_url = format!("sqlite:{}?mode=rwc", path.display());
    SqlitePool::connect(&db_url).await
}

pub async fn initialize_conversation_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            context TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_messages_conversation
        ON messages(conversation_id, timestamp);
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS usage_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            requests_count INTEGER NOT NULL,
            tokens_used INTEGER NOT NULL,
            timestamp TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_usage_user_time
        ON usage_stats(user_id, timestamp DESC);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
