use crate::trading::types::{Order, OrderStatus, OrderType};
use chrono::Utc;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct OrderDatabase {
    pool: Pool<Sqlite>,
}

impl OrderDatabase {
    pub async fn new(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        let db = Self { pool };
        db.initialize().await?;

        Ok(db)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS orders (
                id TEXT PRIMARY KEY,
                order_type TEXT NOT NULL,
                side TEXT NOT NULL,
                status TEXT NOT NULL,
                input_mint TEXT NOT NULL,
                output_mint TEXT NOT NULL,
                input_symbol TEXT NOT NULL,
                output_symbol TEXT NOT NULL,
                amount REAL NOT NULL,
                filled_amount REAL NOT NULL DEFAULT 0,
                limit_price REAL,
                stop_price REAL,
                trailing_percent REAL,
                highest_price REAL,
                lowest_price REAL,
                linked_order_id TEXT,
                slippage_bps INTEGER NOT NULL,
                priority_fee_micro_lamports INTEGER NOT NULL,
                wallet_address TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                triggered_at TEXT,
                tx_signature TEXT,
                error_message TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
            CREATE INDEX IF NOT EXISTS idx_orders_wallet ON orders(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);
            CREATE INDEX IF NOT EXISTS idx_orders_linked ON orders(linked_order_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_order(&self, order: &Order) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO orders (
                id, order_type, side, status, input_mint, output_mint,
                input_symbol, output_symbol, amount, filled_amount,
                limit_price, stop_price, trailing_percent,
                highest_price, lowest_price, linked_order_id,
                slippage_bps, priority_fee_micro_lamports, wallet_address,
                created_at, updated_at, triggered_at, tx_signature, error_message
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19,
                ?20, ?21, ?22, ?23, ?24
            )
            "#,
        )
        .bind(&order.id)
        .bind(order.order_type.to_string())
        .bind(order.side.to_string())
        .bind(order.status.to_string())
        .bind(&order.input_mint)
        .bind(&order.output_mint)
        .bind(&order.input_symbol)
        .bind(&order.output_symbol)
        .bind(order.amount)
        .bind(order.filled_amount)
        .bind(order.limit_price)
        .bind(order.stop_price)
        .bind(order.trailing_percent)
        .bind(order.highest_price)
        .bind(order.lowest_price)
        .bind(&order.linked_order_id)
        .bind(order.slippage_bps)
        .bind(order.priority_fee_micro_lamports)
        .bind(&order.wallet_address)
        .bind(order.created_at.to_rfc3339())
        .bind(order.updated_at.to_rfc3339())
        .bind(order.triggered_at.map(|t| t.to_rfc3339()))
        .bind(&order.tx_signature)
        .bind(&order.error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_order(&self, id: &str) -> Result<Option<Order>, sqlx::Error> {
        let order = sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(order)
    }

    pub async fn get_active_orders(&self, wallet_address: &str) -> Result<Vec<Order>, sqlx::Error> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders 
            WHERE wallet_address = ?1 
            AND status IN ('pending', 'partially_filled')
            ORDER BY created_at DESC
            "#,
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }

    pub async fn get_all_active_orders(&self) -> Result<Vec<Order>, sqlx::Error> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders 
            WHERE status IN ('pending', 'partially_filled')
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }

    pub async fn get_order_history(
        &self,
        wallet_address: &str,
        limit: i64,
    ) -> Result<Vec<Order>, sqlx::Error> {
        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT * FROM orders 
            WHERE wallet_address = ?1 
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )
        .bind(wallet_address)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(orders)
    }

    pub async fn update_order_status(
        &self,
        id: &str,
        status: OrderStatus,
        error_message: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE orders 
            SET status = ?1, updated_at = ?2, error_message = ?3
            WHERE id = ?4
            "#,
        )
        .bind(status.to_string())
        .bind(now)
        .bind(error_message)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_order_fill(
        &self,
        id: &str,
        filled_amount: f64,
        status: OrderStatus,
        tx_signature: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE orders 
            SET filled_amount = ?1, status = ?2, updated_at = ?3,
                triggered_at = ?4, tx_signature = ?5
            WHERE id = ?6
            "#,
        )
        .bind(filled_amount)
        .bind(status.to_string())
        .bind(&now)
        .bind(&now)
        .bind(tx_signature)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_trailing_stop(
        &self,
        id: &str,
        highest_price: Option<f64>,
        lowest_price: Option<f64>,
        stop_price: Option<f64>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE orders 
            SET highest_price = ?1, lowest_price = ?2, stop_price = ?3, updated_at = ?4
            WHERE id = ?5
            "#,
        )
        .bind(highest_price)
        .bind(lowest_price)
        .bind(stop_price)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn cancel_order(&self, id: &str) -> Result<(), sqlx::Error> {
        self.update_order_status(id, OrderStatus::Cancelled, None)
            .await
    }

    pub async fn cancel_linked_orders(&self, linked_id: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE orders 
            SET status = 'cancelled', updated_at = ?1
            WHERE linked_order_id = ?2 AND status IN ('pending', 'partially_filled')
            "#,
        )
        .bind(now)
        .bind(linked_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

pub type SharedOrderDatabase = Arc<RwLock<OrderDatabase>>;
