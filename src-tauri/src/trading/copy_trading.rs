use crate::utils::Rfc3339DateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Row, Sqlite, SqlitePool};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{OnceCell, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTradeConfig {
    pub id: String,
    pub name: String,
    pub wallet_address: String,
    pub source_wallet: String,
    pub allocation_percentage: f64,
    pub multiplier: f64,
    pub min_trade_amount: Option<f64>,
    pub max_trade_amount: Option<f64>,
    pub delay_seconds: i32,
    pub token_whitelist: Option<String>,
    pub token_blacklist: Option<String>,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
    pub max_daily_trades: Option<i32>,
    pub max_total_loss: Option<f64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for CopyTradeConfig {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(CopyTradeConfig {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            wallet_address: row.try_get("wallet_address")?,
            source_wallet: row.try_get("source_wallet")?,
            allocation_percentage: row.try_get("allocation_percentage")?,
            multiplier: row.try_get("multiplier")?,
            min_trade_amount: row.try_get("min_trade_amount")?,
            max_trade_amount: row.try_get("max_trade_amount")?,
            delay_seconds: row.try_get("delay_seconds")?,
            token_whitelist: row.try_get("token_whitelist")?,
            token_blacklist: row.try_get("token_blacklist")?,
            stop_loss_percentage: row.try_get("stop_loss_percentage")?,
            take_profit_percentage: row.try_get("take_profit_percentage")?,
            max_daily_trades: row.try_get("max_daily_trades")?,
            max_total_loss: row.try_get("max_total_loss")?,
            is_active: row.try_get("is_active")?,
            created_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("created_at")?)?.into(),
            updated_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("updated_at")?)?.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyTradeExecution {
    pub id: String,
    pub config_id: String,
    pub source_tx_signature: String,
    pub copied_tx_signature: Option<String>,
    pub source_amount: f64,
    pub copied_amount: f64,
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub price: f64,
    pub pnl: f64,
    pub executed_at: DateTime<Utc>,
    pub status: String,
    pub error_message: Option<String>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for CopyTradeExecution {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(CopyTradeExecution {
            id: row.try_get("id")?,
            config_id: row.try_get("config_id")?,
            source_tx_signature: row.try_get("source_tx_signature")?,
            copied_tx_signature: row.try_get("copied_tx_signature")?,
            source_amount: row.try_get("source_amount")?,
            copied_amount: row.try_get("copied_amount")?,
            input_mint: row.try_get("input_mint")?,
            output_mint: row.try_get("output_mint")?,
            input_symbol: row.try_get("input_symbol")?,
            output_symbol: row.try_get("output_symbol")?,
            price: row.try_get("price")?,
            pnl: row.try_get("pnl")?,
            executed_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("executed_at")?)?.into(),
            status: row.try_get("status")?,
            error_message: row.try_get("error_message")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCopyTradeRequest {
    pub name: String,
    pub wallet_address: String,
    pub source_wallet: String,
    pub allocation_percentage: f64,
    pub multiplier: f64,
    pub min_trade_amount: Option<f64>,
    pub max_trade_amount: Option<f64>,
    pub delay_seconds: i32,
    pub token_whitelist: Option<Vec<String>>,
    pub token_blacklist: Option<Vec<String>>,
    pub stop_loss_percentage: Option<f64>,
    pub take_profit_percentage: Option<f64>,
    pub max_daily_trades: Option<i32>,
    pub max_total_loss: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopyTradePerformance {
    pub total_trades: i64,
    pub successful_trades: i64,
    pub failed_trades: i64,
    pub total_volume: f64,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub avg_trade_size: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletActivity {
    pub wallet: String,
    pub tx_signature: String,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub amount: f64,
    pub performance_pct: Option<f64>,
    pub pnl: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopyTradeEvent {
    pub config_id: String,
    pub name: String,
    pub source_wallet: String,
    pub amount: f64,
    pub symbol: String,
    pub status: String,
    pub tx_signature: Option<String>,
}

#[derive(Debug)]
struct CopyTradeStats {
    total_trades: i64,
    successful: i64,
    total_volume: f64,
    total_pnl: f64,
}

pub struct CopyTradeDatabase {
    pool: Pool<Sqlite>,
}

impl CopyTradeDatabase {
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
            CREATE TABLE IF NOT EXISTS copy_trade_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                wallet_address TEXT NOT NULL,
                source_wallet TEXT NOT NULL,
                allocation_percentage REAL NOT NULL,
                multiplier REAL NOT NULL,
                min_trade_amount REAL,
                max_trade_amount REAL,
                delay_seconds INTEGER NOT NULL,
                token_whitelist TEXT,
                token_blacklist TEXT,
                stop_loss_percentage REAL,
                take_profit_percentage REAL,
                max_daily_trades INTEGER,
                max_total_loss REAL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS copy_trade_executions (
                id TEXT PRIMARY KEY,
                config_id TEXT NOT NULL,
                source_tx_signature TEXT NOT NULL,
                copied_tx_signature TEXT,
                source_amount REAL NOT NULL,
                copied_amount REAL NOT NULL,
                input_mint TEXT NOT NULL,
                output_mint TEXT NOT NULL,
                input_symbol TEXT NOT NULL,
                output_symbol TEXT NOT NULL,
                price REAL NOT NULL,
                pnl REAL NOT NULL,
                executed_at TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                FOREIGN KEY (config_id) REFERENCES copy_trade_configs(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_copy_trade_configs_active ON copy_trade_configs(is_active);
            CREATE INDEX IF NOT EXISTS idx_copy_trade_configs_wallet ON copy_trade_configs(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_copy_trade_configs_source ON copy_trade_configs(source_wallet);
            CREATE INDEX IF NOT EXISTS idx_copy_trade_exec_config ON copy_trade_executions(config_id);
            CREATE INDEX IF NOT EXISTS idx_copy_trade_exec_time ON copy_trade_executions(executed_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_config(&self, config: &CopyTradeConfig) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO copy_trade_configs (
                id, name, wallet_address, source_wallet, allocation_percentage, multiplier,
                min_trade_amount, max_trade_amount, delay_seconds, token_whitelist, token_blacklist,
                stop_loss_percentage, take_profit_percentage, max_daily_trades, max_total_loss,
                is_active, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14, ?15,
                ?16, ?17, ?18
            )
            "#,
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.wallet_address)
        .bind(&config.source_wallet)
        .bind(config.allocation_percentage)
        .bind(config.multiplier)
        .bind(config.min_trade_amount)
        .bind(config.max_trade_amount)
        .bind(config.delay_seconds)
        .bind(&config.token_whitelist)
        .bind(&config.token_blacklist)
        .bind(config.stop_loss_percentage)
        .bind(config.take_profit_percentage)
        .bind(config.max_daily_trades)
        .bind(config.max_total_loss)
        .bind(if config.is_active { 1 } else { 0 })
        .bind(config.created_at.to_rfc3339())
        .bind(config.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_config(&self, id: &str) -> Result<Option<CopyTradeConfig>, sqlx::Error> {
        sqlx::query_as::<_, CopyTradeConfig>("SELECT * FROM copy_trade_configs WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_configs(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<CopyTradeConfig>, sqlx::Error> {
        sqlx::query_as::<_, CopyTradeConfig>(
            "SELECT * FROM copy_trade_configs WHERE wallet_address = ?1 ORDER BY created_at DESC",
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_active_configs(&self) -> Result<Vec<CopyTradeConfig>, sqlx::Error> {
        sqlx::query_as::<_, CopyTradeConfig>("SELECT * FROM copy_trade_configs WHERE is_active = 1")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn update_config_status(&self, id: &str, is_active: bool) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE copy_trade_configs SET is_active = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(if is_active { 1 } else { 0 })
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_config(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM copy_trade_configs WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_execution(
        &self,
        execution: &CopyTradeExecution,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO copy_trade_executions (
                id, config_id, source_tx_signature, copied_tx_signature,
                source_amount, copied_amount, input_mint, output_mint,
                input_symbol, output_symbol, price, pnl, executed_at, status, error_message
            ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7, ?8,
                ?9, ?10, ?11, ?12, ?13, ?14, ?15
            )
            "#,
        )
        .bind(&execution.id)
        .bind(&execution.config_id)
        .bind(&execution.source_tx_signature)
        .bind(&execution.copied_tx_signature)
        .bind(execution.source_amount)
        .bind(execution.copied_amount)
        .bind(&execution.input_mint)
        .bind(&execution.output_mint)
        .bind(&execution.input_symbol)
        .bind(&execution.output_symbol)
        .bind(execution.price)
        .bind(execution.pnl)
        .bind(execution.executed_at.to_rfc3339())
        .bind(&execution.status)
        .bind(&execution.error_message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_executions(
        &self,
        config_id: &str,
    ) -> Result<Vec<CopyTradeExecution>, sqlx::Error> {
        sqlx::query_as::<_, CopyTradeExecution>(
            "SELECT * FROM copy_trade_executions WHERE config_id = ?1 ORDER BY executed_at DESC",
        )
        .bind(config_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn daily_trade_count(&self, config_id: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM copy_trade_executions
            WHERE config_id = ?1
              AND date(executed_at) = date('now')
            "#,
        )
        .bind(config_id)
        .fetch_one(&self.pool)
        .await?;
        row.try_get("count")
    }

    pub async fn stats(&self, config_id: &str) -> Result<CopyTradeStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as successful,
                COALESCE(SUM(copied_amount), 0) as volume,
                COALESCE(SUM(pnl), 0) as pnl
            FROM copy_trade_executions
            WHERE config_id = ?1
            "#,
        )
        .bind(config_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CopyTradeStats {
            total_trades: row.try_get("total")?,
            successful: row.try_get("successful")?,
            total_volume: row.try_get("volume")?,
            total_pnl: row.try_get("pnl")?,
        })
    }

    pub async fn total_pnl(&self, config_id: &str) -> Result<f64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(pnl), 0) as pnl
            FROM copy_trade_executions
            WHERE config_id = ?1
            "#,
        )
        .bind(config_id)
        .fetch_one(&self.pool)
        .await?;

        row.try_get("pnl")
    }
}

pub type SharedCopyTradeDatabase = Arc<RwLock<CopyTradeDatabase>>;

pub struct CopyTradeManager {
    db: SharedCopyTradeDatabase,
    app_handle: AppHandle,
    monitored_wallets: Arc<RwLock<HashSet<String>>>,
    processed_transactions: Arc<RwLock<HashSet<String>>>,
}

impl CopyTradeManager {
    pub fn new(db: SharedCopyTradeDatabase, app_handle: AppHandle) -> Self {
        Self {
            db,
            app_handle,
            monitored_wallets: Arc::new(RwLock::new(HashSet::new())),
            processed_transactions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn create_copy_trade(
        &self,
        request: CreateCopyTradeRequest,
    ) -> Result<CopyTradeConfig, String> {
        if !(0.0..=100.0).contains(&request.allocation_percentage) {
            return Err("Allocation percentage must be between 0 and 100".into());
        }
        if request.multiplier <= 0.0 {
            return Err("Multiplier must be greater than zero".into());
        }

        let config = CopyTradeConfig {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            wallet_address: request.wallet_address,
            source_wallet: request.source_wallet.clone(),
            allocation_percentage: request.allocation_percentage,
            multiplier: request.multiplier,
            min_trade_amount: request.min_trade_amount,
            max_trade_amount: request.max_trade_amount,
            delay_seconds: request.delay_seconds,
            token_whitelist: request.token_whitelist.map(|list| list.join(",")),
            token_blacklist: request.token_blacklist.map(|list| list.join(",")),
            stop_loss_percentage: request.stop_loss_percentage,
            take_profit_percentage: request.take_profit_percentage,
            max_daily_trades: request.max_daily_trades,
            max_total_loss: request.max_total_loss,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.db
            .write()
            .await
            .create_config(&config)
            .await
            .map_err(|e| format!("Failed to create copy trade config: {e}"))?;

        self.monitored_wallets
            .write()
            .await
            .insert(request.source_wallet);

        Ok(config)
    }

    pub async fn get_copy_trade(&self, id: &str) -> Result<CopyTradeConfig, String> {
        self.db
            .read()
            .await
            .get_config(id)
            .await
            .map_err(|e| format!("Failed to load copy trade config: {e}"))?
            .ok_or_else(|| "Copy trade config not found".to_string())
    }

    pub async fn list_copy_trades(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<CopyTradeConfig>, String> {
        self.db
            .read()
            .await
            .list_configs(wallet_address)
            .await
            .map_err(|e| format!("Failed to list copy trades: {e}"))
    }

    pub async fn pause_copy_trade(&self, id: &str) -> Result<(), String> {
        self.db
            .write()
            .await
            .update_config_status(id, false)
            .await
            .map_err(|e| format!("Failed to pause copy trade: {e}"))
    }

    pub async fn resume_copy_trade(&self, id: &str) -> Result<CopyTradeConfig, String> {
        self.db
            .write()
            .await
            .update_config_status(id, true)
            .await
            .map_err(|e| format!("Failed to resume copy trade: {e}"))?;
        self.get_copy_trade(id).await
    }

    pub async fn delete_copy_trade(&self, id: &str) -> Result<(), String> {
        self.db
            .write()
            .await
            .delete_config(id)
            .await
            .map_err(|e| format!("Failed to delete copy trade: {e}"))
    }

    pub async fn get_executions(&self, id: &str) -> Result<Vec<CopyTradeExecution>, String> {
        self.db
            .read()
            .await
            .get_executions(id)
            .await
            .map_err(|e| format!("Failed to fetch executions: {e}"))
    }

    pub async fn get_performance(&self, id: &str) -> Result<CopyTradePerformance, String> {
        let stats = self
            .db
            .read()
            .await
            .stats(id)
            .await
            .map_err(|e| format!("Failed to compute performance: {e}"))?;

        let failed = stats.total_trades - stats.successful;
        let win_rate = if stats.total_trades > 0 {
            (stats.successful as f64 / stats.total_trades as f64) * 100.0
        } else {
            0.0
        };
        let avg_trade = if stats.total_trades > 0 {
            stats.total_volume / stats.total_trades as f64
        } else {
            0.0
        };

        Ok(CopyTradePerformance {
            total_trades: stats.total_trades,
            successful_trades: stats.successful,
            failed_trades: failed,
            total_volume: stats.total_volume,
            total_pnl: stats.total_pnl,
            win_rate,
            avg_trade_size: avg_trade,
        })
    }

    pub async fn process_wallet_activity(&self, activity: WalletActivity) -> Result<(), String> {
        if !self
            .monitored_wallets
            .read()
            .await
            .contains(&activity.wallet)
        {
            return Ok(());
        }

        {
            let mut seen = self.processed_transactions.write().await;
            if seen.contains(&activity.tx_signature) {
                return Ok(());
            }
            seen.insert(activity.tx_signature.clone());
            if seen.len() > 50_000 {
                // Prevent unbounded growth
                seen.clear();
            }
        }

        let configs = self
            .db
            .read()
            .await
            .get_active_configs()
            .await
            .map_err(|e| format!("Failed to load copy trade configs: {e}"))?;

        for config in configs {
            if config.source_wallet != activity.wallet {
                continue;
            }

            match self.should_copy_trade(&config, &activity).await? {
                TradeDecision::Stop(reason) => {
                    self.db
                        .write()
                        .await
                        .update_config_status(&config.id, false)
                        .await
                        .ok();
                    self.log_execution(
                        &config,
                        &activity,
                        0.0,
                        "stopped",
                        Some(reason.clone()),
                        None,
                    )
                    .await
                    .ok();
                }
                TradeDecision::Skip(reason) => {
                    self.log_execution(&config, &activity, 0.0, "skipped", Some(reason), None)
                        .await
                        .ok();
                }
                TradeDecision::Proceed => {
                    if let Err(err) = self.execute_copy_trade(&config, &activity).await {
                        eprintln!("Failed to execute copy trade: {err}");
                        self.log_execution(&config, &activity, 0.0, "error", Some(err), None)
                            .await
                            .ok();
                    }
                }
            }
        }

        Ok(())
    }

    async fn execute_copy_trade(
        &self,
        config: &CopyTradeConfig,
        activity: &WalletActivity,
    ) -> Result<(), String> {
        if config.delay_seconds > 0 {
            tokio::time::sleep(Duration::from_secs(config.delay_seconds as u64)).await;
        }

        let base_amount = activity.amount * (config.allocation_percentage / 100.0);
        let copied_amount = base_amount * config.multiplier;

        if let Some(min_amount) = config.min_trade_amount {
            if copied_amount < min_amount {
                return Err("Trade amount below minimum threshold".into());
            }
        }

        if let Some(max_amount) = config.max_trade_amount {
            if copied_amount > max_amount {
                return Err("Trade amount exceeds maximum threshold".into());
            }
        }

        let pnl = activity.pnl.unwrap_or_default()
            * (config.allocation_percentage / 100.0)
            * config.multiplier;

        let execution = CopyTradeExecution {
            id: Uuid::new_v4().to_string(),
            config_id: config.id.clone(),
            source_tx_signature: activity.tx_signature.clone(),
            copied_tx_signature: Some(format!("simulated_{}", Uuid::new_v4())),
            source_amount: activity.amount,
            copied_amount,
            input_mint: activity.input_mint.clone(),
            output_mint: activity.output_mint.clone(),
            input_symbol: activity.input_symbol.clone(),
            output_symbol: activity.output_symbol.clone(),
            price: if copied_amount > 0.0 {
                activity.amount / copied_amount
            } else {
                0.0
            },
            pnl,
            executed_at: Utc::now(),
            status: "success".into(),
            error_message: None,
        };

        self.db
            .write()
            .await
            .create_execution(&execution)
            .await
            .map_err(|e| format!("Failed to record execution: {e}"))?;

        self.emit_execution_event(config, &execution);

        Ok(())
    }

    async fn should_copy_trade(
        &self,
        config: &CopyTradeConfig,
        activity: &WalletActivity,
    ) -> Result<TradeDecision, String> {
        let allocation_amount =
            activity.amount * (config.allocation_percentage / 100.0) * config.multiplier;

        let daily_trade_count = if config.max_daily_trades.is_some() {
            Some(
                self.db
                    .read()
                    .await
                    .daily_trade_count(&config.id)
                    .await
                    .map_err(|e| format!("Failed to fetch daily trade count: {e}"))?,
            )
        } else {
            None
        };

        let total_pnl = if config.max_total_loss.is_some() {
            Some(
                self.db
                    .read()
                    .await
                    .total_pnl(&config.id)
                    .await
                    .map_err(|e| format!("Failed to fetch total PnL: {e}"))?,
            )
        } else {
            None
        };

        Ok(evaluate_trade_decision(
            config,
            activity,
            allocation_amount,
            daily_trade_count,
            total_pnl,
        ))
    }

    async fn log_execution(
        &self,
        config: &CopyTradeConfig,
        activity: &WalletActivity,
        copied_amount: f64,
        status: &str,
        error: Option<String>,
        tx_signature: Option<String>,
    ) -> Result<(), String> {
        let execution = CopyTradeExecution {
            id: Uuid::new_v4().to_string(),
            config_id: config.id.clone(),
            source_tx_signature: activity.tx_signature.clone(),
            copied_tx_signature: tx_signature,
            source_amount: activity.amount,
            copied_amount,
            input_mint: activity.input_mint.clone(),
            output_mint: activity.output_mint.clone(),
            input_symbol: activity.input_symbol.clone(),
            output_symbol: activity.output_symbol.clone(),
            price: if copied_amount > 0.0 {
                activity.amount / copied_amount
            } else {
                0.0
            },
            pnl: activity.pnl.unwrap_or_default()
                * (config.allocation_percentage / 100.0)
                * config.multiplier,
            executed_at: Utc::now(),
            status: status.to_string(),
            error_message: error,
        };

        self.db
            .write()
            .await
            .create_execution(&execution)
            .await
            .map_err(|e| format!("Failed to record execution: {e}"))
    }

    fn emit_execution_event(&self, config: &CopyTradeConfig, execution: &CopyTradeExecution) {
        let event = CopyTradeEvent {
            config_id: config.id.clone(),
            name: config.name.clone(),
            source_wallet: config.source_wallet.clone(),
            amount: execution.copied_amount,
            symbol: execution.output_symbol.clone(),
            status: execution.status.clone(),
            tx_signature: execution.copied_tx_signature.clone(),
        };

        let _ = self.app_handle.emit("copy_trade_execution", event);
    }

    pub async fn initialize_monitored_wallets(&self) -> Result<(), String> {
        let configs = self
            .db
            .read()
            .await
            .get_active_configs()
            .await
            .map_err(|e| format!("Failed to load copy trade configs: {e}"))?;

        let mut wallets = self.monitored_wallets.write().await;
        wallets.clear();
        wallets.extend(configs.into_iter().map(|cfg| cfg.source_wallet));
        Ok(())
    }

    pub async fn followed_wallets(&self) -> Vec<String> {
        self.monitored_wallets
            .read()
            .await
            .iter()
            .cloned()
            .collect()
    }

    pub async fn start_monitoring(manager: Arc<Self>) {
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await;
            if let Err(err) = manager.initialize_monitored_wallets().await {
                eprintln!("Failed to refresh monitored wallets: {err}");
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TradeDecision {
    Proceed,
    Skip(String),
    Stop(String),
}

fn evaluate_trade_decision(
    config: &CopyTradeConfig,
    activity: &WalletActivity,
    allocation_amount: f64,
    daily_trade_count: Option<i64>,
    total_pnl: Option<f64>,
) -> TradeDecision {
    if let Some(list) = &config.token_whitelist {
        let set: HashSet<&str> = list.split(',').collect();
        if !set.contains(activity.output_mint.as_str()) {
            return TradeDecision::Skip("Token not in whitelist".into());
        }
    }

    if let Some(list) = &config.token_blacklist {
        let set: HashSet<&str> = list.split(',').collect();
        if set.contains(activity.output_mint.as_str()) {
            return TradeDecision::Skip("Token is blacklisted".into());
        }
    }

    if let Some(performance) = activity.performance_pct {
        if let Some(stop_loss) = config.stop_loss_percentage {
            if performance <= -stop_loss {
                return TradeDecision::Stop(format!("Stop loss triggered at {:.2}%", performance));
            }
        }

        if let Some(take_profit) = config.take_profit_percentage {
            if performance >= take_profit {
                return TradeDecision::Stop(format!(
                    "Take profit triggered at {:.2}%",
                    performance
                ));
            }
        }
    }

    if let (Some(max_trades), Some(count)) = (config.max_daily_trades, daily_trade_count) {
        if count >= max_trades as i64 {
            return TradeDecision::Skip("Daily trade limit reached".into());
        }
    }

    if let (Some(max_loss), Some(pnl)) = (config.max_total_loss, total_pnl) {
        if pnl <= -max_loss {
            return TradeDecision::Stop("Maximum loss threshold reached".into());
        }
    }

    if let Some(min_amount) = config.min_trade_amount {
        if allocation_amount < min_amount {
            return TradeDecision::Skip("Trade below minimum amount".into());
        }
    }

    if let Some(max_amount) = config.max_trade_amount {
        if allocation_amount > max_amount {
            return TradeDecision::Skip("Trade exceeds maximum amount".into());
        }
    }

    TradeDecision::Proceed
}

pub struct CopyTradingState {
    pub db: SharedCopyTradeDatabase,
    pub manager: Arc<CopyTradeManager>,
}

static COPY_TRADING_STATE: OnceCell<CopyTradingState> = OnceCell::const_new();

pub async fn init_copy_trading(app_handle: &AppHandle) -> Result<(), String> {
    if COPY_TRADING_STATE.get().is_some() {
        return Ok(());
    }

    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Unable to resolve app data directory: {}", e))?;
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {e}"))?;

    let mut db_path = PathBuf::from(&app_dir);
    db_path.push("automation.db");

    let db = CopyTradeDatabase::new(db_path)
        .await
        .map_err(|e| format!("Failed to initialize copy trading database: {e}"))?;

    let shared_db = Arc::new(RwLock::new(db));
    let manager = Arc::new(CopyTradeManager::new(shared_db.clone(), app_handle.clone()));
    manager.initialize_monitored_wallets().await?;

    let handle_for_task = app_handle.clone();
    let manager_for_task = manager.clone();
    tauri::async_runtime::spawn(async move {
        CopyTradeManager::start_monitoring(manager_for_task).await;
        let _ = handle_for_task.emit(
            "copy_trading_monitor_stopped",
            "Copy trading monitor stopped",
        );
    });

    COPY_TRADING_STATE
        .set(CopyTradingState {
            db: shared_db,
            manager: manager.clone(),
        })
        .map_err(|_| "Copy trading state already initialized".to_string())?;

    Ok(())
}

fn require_state<'a>() -> Result<&'a CopyTradingState, String> {
    COPY_TRADING_STATE
        .get()
        .ok_or_else(|| "Copy trading module not initialized".to_string())
}

#[tauri::command]
pub async fn copy_trading_init(handle: AppHandle) -> Result<(), String> {
    init_copy_trading(&handle).await
}

#[tauri::command]
pub async fn copy_trading_create(
    request: CreateCopyTradeRequest,
) -> Result<CopyTradeConfig, String> {
    let state = require_state()?;
    state.manager.create_copy_trade(request).await
}

#[tauri::command]
pub async fn copy_trading_list(wallet_address: String) -> Result<Vec<CopyTradeConfig>, String> {
    let state = require_state()?;
    state.manager.list_copy_trades(&wallet_address).await
}

#[tauri::command]
pub async fn copy_trading_get(id: String) -> Result<CopyTradeConfig, String> {
    let state = require_state()?;
    state.manager.get_copy_trade(&id).await
}

#[tauri::command]
pub async fn copy_trading_pause(id: String) -> Result<(), String> {
    let state = require_state()?;
    state.manager.pause_copy_trade(&id).await
}

#[tauri::command]
pub async fn copy_trading_resume(id: String) -> Result<CopyTradeConfig, String> {
    let state = require_state()?;
    state.manager.resume_copy_trade(&id).await
}

#[tauri::command]
pub async fn copy_trading_delete(id: String) -> Result<(), String> {
    let state = require_state()?;
    state.manager.delete_copy_trade(&id).await
}

#[tauri::command]
pub async fn copy_trading_history(id: String) -> Result<Vec<CopyTradeExecution>, String> {
    let state = require_state()?;
    state.manager.get_executions(&id).await
}

#[tauri::command]
pub async fn copy_trading_performance(id: String) -> Result<CopyTradePerformance, String> {
    let state = require_state()?;
    state.manager.get_performance(&id).await
}

#[tauri::command]
pub async fn copy_trading_process_activity(activity: WalletActivity) -> Result<(), String> {
    let state = require_state()?;
    state.manager.process_wallet_activity(activity).await
}

#[tauri::command]
pub async fn copy_trading_followed_wallets() -> Result<Vec<String>, String> {
    let state = require_state()?;
    Ok(state.manager.followed_wallets().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> CopyTradeConfig {
        CopyTradeConfig {
            id: "cfg".into(),
            name: "Test".into(),
            wallet_address: "wallet".into(),
            source_wallet: "source".into(),
            allocation_percentage: 50.0,
            multiplier: 1.0,
            min_trade_amount: Some(10.0),
            max_trade_amount: Some(1000.0),
            delay_seconds: 0,
            token_whitelist: None,
            token_blacklist: None,
            stop_loss_percentage: Some(5.0),
            take_profit_percentage: Some(20.0),
            max_daily_trades: Some(3),
            max_total_loss: Some(500.0),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_activity(performance: Option<f64>) -> WalletActivity {
        WalletActivity {
            wallet: "source".into(),
            tx_signature: "sig".into(),
            timestamp: Utc::now(),
            action: "buy".into(),
            input_mint: "mint1".into(),
            output_mint: "mint2".into(),
            input_symbol: "IN".into(),
            output_symbol: "OUT".into(),
            amount: 100.0,
            performance_pct: performance,
            pnl: Some(-60.0),
        }
    }

    #[test]
    fn test_stop_loss_triggers_stop_decision() {
        let config = sample_config();
        let activity = sample_activity(Some(-6.0));
        let allocation =
            activity.amount * (config.allocation_percentage / 100.0) * config.multiplier;

        let decision = evaluate_trade_decision(&config, &activity, allocation, Some(0), Some(0.0));
        assert!(matches!(decision, TradeDecision::Stop(_)));
    }

    #[test]
    fn test_min_amount_skip() {
        let mut config = sample_config();
        config.min_trade_amount = Some(60.0);
        let activity = sample_activity(Some(2.0));
        let allocation =
            activity.amount * (config.allocation_percentage / 100.0) * config.multiplier;

        let decision = evaluate_trade_decision(&config, &activity, allocation, None, None);
        assert!(matches!(decision, TradeDecision::Skip(_)));
    }
}
