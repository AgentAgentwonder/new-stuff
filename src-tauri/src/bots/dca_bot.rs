use crate::api::jupiter::{
    jupiter_quote, PriorityFeeConfig, QuoteCommandInput, QuoteResult, SwapMode,
};
use crate::utils::{OptionalRfc3339DateTime, Rfc3339DateTime};
use chrono::{DateTime, NaiveDateTime, Utc};
use cron::Schedule;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{OnceCell, RwLock};
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcaConfig {
    pub id: String,
    pub name: String,
    pub wallet_address: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub input_decimals: i32,
    pub output_decimals: i32,
    pub amount_per_execution: f64,
    pub total_budget: f64,
    pub spent_amount: f64,
    pub schedule_cron: String,
    pub slippage_bps: i32,
    pub priority_fee_micro_lamports: i32,
    pub max_price_impact_pct: f64,
    pub daily_spend_cap: Option<f64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_execution: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_execution: Option<DateTime<Utc>>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for DcaConfig {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(DcaConfig {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            wallet_address: row.try_get("wallet_address")?,
            input_mint: row.try_get("input_mint")?,
            output_mint: row.try_get("output_mint")?,
            input_symbol: row.try_get("input_symbol")?,
            output_symbol: row.try_get("output_symbol")?,
            input_decimals: row.try_get("input_decimals")?,
            output_decimals: row.try_get("output_decimals")?,
            amount_per_execution: row.try_get("amount_per_execution")?,
            total_budget: row.try_get("total_budget")?,
            spent_amount: row.try_get("spent_amount")?,
            schedule_cron: row.try_get("schedule_cron")?,
            slippage_bps: row.try_get("slippage_bps")?,
            priority_fee_micro_lamports: row.try_get("priority_fee_micro_lamports")?,
            max_price_impact_pct: row.try_get("max_price_impact_pct")?,
            daily_spend_cap: row.try_get("daily_spend_cap")?,
            is_active: row.try_get("is_active")?,
            created_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("created_at")?)?.into(),
            updated_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("updated_at")?)?.into(),
            last_execution: OptionalRfc3339DateTime::try_from(
                row.try_get::<Option<String>, _>("last_execution")?,
            )?
            .into(),
            next_execution: OptionalRfc3339DateTime::try_from(
                row.try_get::<Option<String>, _>("next_execution")?,
            )?
            .into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcaExecution {
    pub id: String,
    pub dca_config_id: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub price: f64,
    pub total_cost: f64,
    pub executed_at: DateTime<Utc>,
    pub status: String,
    pub error_message: Option<String>,
    pub tx_signature: Option<String>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for DcaExecution {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(DcaExecution {
            id: row.try_get("id")?,
            dca_config_id: row.try_get("dca_config_id")?,
            input_amount: row.try_get("input_amount")?,
            output_amount: row.try_get("output_amount")?,
            price: row.try_get("price")?,
            total_cost: row.try_get("total_cost")?,
            executed_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("executed_at")?)?.into(),
            status: row.try_get("status")?,
            error_message: row.try_get("error_message")?,
            tx_signature: row.try_get("tx_signature")?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDcaRequest {
    pub name: String,
    pub wallet_address: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub input_decimals: i32,
    pub output_decimals: i32,
    pub amount_per_execution: f64,
    pub total_budget: f64,
    pub schedule_cron: String,
    pub slippage_bps: i32,
    pub priority_fee_micro_lamports: i32,
    pub max_price_impact_pct: f64,
    pub daily_spend_cap: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DcaPerformance {
    pub total_invested: f64,
    pub total_acquired: f64,
    pub average_price: f64,
    pub execution_count: i64,
    pub success_count: i64,
    pub success_rate: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
    pub remaining_budget: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DcaExecutionEvent {
    pub dca_id: String,
    pub name: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub input_amount: f64,
    pub output_amount: f64,
    pub price: f64,
    pub status: String,
    pub tx_signature: Option<String>,
}

#[derive(Debug)]
struct DcaExecutionSummary {
    executions: i64,
    successes: i64,
    invested: f64,
    acquired: f64,
}

pub struct DcaDatabase {
    pool: Pool<Sqlite>,
}

impl DcaDatabase {
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
            CREATE TABLE IF NOT EXISTS dca_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                wallet_address TEXT NOT NULL,
                input_mint TEXT NOT NULL,
                output_mint TEXT NOT NULL,
                input_symbol TEXT NOT NULL,
                output_symbol TEXT NOT NULL,
                input_decimals INTEGER NOT NULL,
                output_decimals INTEGER NOT NULL,
                amount_per_execution REAL NOT NULL,
                total_budget REAL NOT NULL,
                spent_amount REAL NOT NULL DEFAULT 0,
                schedule_cron TEXT NOT NULL,
                slippage_bps INTEGER NOT NULL,
                priority_fee_micro_lamports INTEGER NOT NULL,
                max_price_impact_pct REAL NOT NULL,
                daily_spend_cap REAL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_execution TEXT,
                next_execution TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dca_executions (
                id TEXT PRIMARY KEY,
                dca_config_id TEXT NOT NULL,
                input_amount REAL NOT NULL,
                output_amount REAL NOT NULL,
                price REAL NOT NULL,
                total_cost REAL NOT NULL,
                executed_at TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                tx_signature TEXT,
                FOREIGN KEY (dca_config_id) REFERENCES dca_configs(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_dca_configs_active ON dca_configs(is_active);
            CREATE INDEX IF NOT EXISTS idx_dca_configs_wallet ON dca_configs(wallet_address);
            CREATE INDEX IF NOT EXISTS idx_dca_configs_next_execution ON dca_configs(next_execution);
            CREATE INDEX IF NOT EXISTS idx_dca_exec_config ON dca_executions(dca_config_id);
            CREATE INDEX IF NOT EXISTS idx_dca_exec_time ON dca_executions(executed_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_config(&self, config: &DcaConfig) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO dca_configs (
                id, name, wallet_address, input_mint, output_mint,
                input_symbol, output_symbol, input_decimals, output_decimals,
                amount_per_execution, total_budget, spent_amount, schedule_cron,
                slippage_bps, priority_fee_micro_lamports, max_price_impact_pct,
                daily_spend_cap, is_active, created_at, updated_at, last_execution, next_execution
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13,
                ?14, ?15, ?16,
                ?17, ?18, ?19, ?20, ?21, ?22
            )
            "#,
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.wallet_address)
        .bind(&config.input_mint)
        .bind(&config.output_mint)
        .bind(&config.input_symbol)
        .bind(&config.output_symbol)
        .bind(config.input_decimals)
        .bind(config.output_decimals)
        .bind(config.amount_per_execution)
        .bind(config.total_budget)
        .bind(config.spent_amount)
        .bind(&config.schedule_cron)
        .bind(config.slippage_bps)
        .bind(config.priority_fee_micro_lamports)
        .bind(config.max_price_impact_pct)
        .bind(config.daily_spend_cap)
        .bind(if config.is_active { 1 } else { 0 })
        .bind(config.created_at.to_rfc3339())
        .bind(config.updated_at.to_rfc3339())
        .bind(config.last_execution.map(|t| t.to_rfc3339()))
        .bind(config.next_execution.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_config(&self, id: &str) -> Result<Option<DcaConfig>, sqlx::Error> {
        sqlx::query_as::<_, DcaConfig>("SELECT * FROM dca_configs WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_configs(&self, wallet_address: &str) -> Result<Vec<DcaConfig>, sqlx::Error> {
        sqlx::query_as::<_, DcaConfig>(
            "SELECT * FROM dca_configs WHERE wallet_address = ?1 ORDER BY created_at DESC",
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_active_configs(&self) -> Result<Vec<DcaConfig>, sqlx::Error> {
        sqlx::query_as::<_, DcaConfig>("SELECT * FROM dca_configs WHERE is_active = 1")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_due_configs(
        &self,
        reference: DateTime<Utc>,
    ) -> Result<Vec<DcaConfig>, sqlx::Error> {
        sqlx::query_as::<_, DcaConfig>(
            "SELECT * FROM dca_configs WHERE is_active = 1 AND next_execution IS NOT NULL AND next_execution <= ?1",
        )
        .bind(reference.to_rfc3339())
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_config_status(&self, id: &str, is_active: bool) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query("UPDATE dca_configs SET is_active = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(if is_active { 1 } else { 0 })
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_execution_window(
        &self,
        id: &str,
        last_execution: DateTime<Utc>,
        next_execution: Option<DateTime<Utc>>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE dca_configs SET last_execution = ?1, next_execution = ?2, updated_at = ?3 WHERE id = ?4",
        )
        .bind(last_execution.to_rfc3339())
        .bind(next_execution.map(|dt| dt.to_rfc3339()))
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_spent_amount(
        &self,
        id: &str,
        spent_amount: f64,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query("UPDATE dca_configs SET spent_amount = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(spent_amount)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_config(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM dca_configs WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn record_execution(&self, execution: &DcaExecution) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO dca_executions (
                id, dca_config_id, input_amount, output_amount, price, total_cost,
                executed_at, status, error_message, tx_signature
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(&execution.id)
        .bind(&execution.dca_config_id)
        .bind(execution.input_amount)
        .bind(execution.output_amount)
        .bind(execution.price)
        .bind(execution.total_cost)
        .bind(execution.executed_at.to_rfc3339())
        .bind(&execution.status)
        .bind(&execution.error_message)
        .bind(&execution.tx_signature)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_executions(&self, dca_id: &str) -> Result<Vec<DcaExecution>, sqlx::Error> {
        sqlx::query_as::<_, DcaExecution>(
            "SELECT * FROM dca_executions WHERE dca_config_id = ?1 ORDER BY executed_at DESC",
        )
        .bind(dca_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn execution_summary(
        &self,
        dca_id: &str,
    ) -> Result<DcaExecutionSummary, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as executions,
                SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as successes,
                COALESCE(SUM(total_cost), 0) as invested,
                COALESCE(SUM(output_amount), 0) as acquired
            FROM dca_executions
            WHERE dca_config_id = ?1
            "#,
        )
        .bind(dca_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DcaExecutionSummary {
            executions: row.try_get("executions")?,
            successes: row.try_get("successes")?,
            invested: row.try_get("invested")?,
            acquired: row.try_get("acquired")?,
        })
    }

    pub async fn spend_since(
        &self,
        dca_id: &str,
        since: DateTime<Utc>,
    ) -> Result<f64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(total_cost), 0) as spent
            FROM dca_executions
            WHERE dca_config_id = ?1
              AND status = 'success'
              AND executed_at >= ?2
            "#,
        )
        .bind(dca_id)
        .bind(since.to_rfc3339())
        .fetch_one(&self.pool)
        .await?;

        row.try_get("spent")
    }
}

pub type SharedDcaDatabase = Arc<RwLock<DcaDatabase>>;

pub struct DcaManager {
    db: SharedDcaDatabase,
    app_handle: AppHandle,
    schedules: Arc<RwLock<HashMap<String, Schedule>>>,
}

impl DcaManager {
    pub fn new(db: SharedDcaDatabase, app_handle: AppHandle) -> Self {
        Self {
            db,
            app_handle,
            schedules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_dca(&self, request: CreateDcaRequest) -> Result<DcaConfig, String> {
        if request.amount_per_execution <= 0.0 {
            return Err("Amount per execution must be greater than zero".into());
        }
        if request.total_budget < request.amount_per_execution {
            return Err(
                "Total budget must be greater than or equal to amount per execution".into(),
            );
        }
        if request.slippage_bps < 0 {
            return Err("Slippage bps must be non-negative".into());
        }
        if request.max_price_impact_pct <= 0.0 {
            return Err("Max price impact must be greater than zero".into());
        }

        let schedule = Schedule::from_str(&request.schedule_cron)
            .map_err(|e| format!("Invalid cron expression: {e}"))?;
        let next_execution = schedule
            .upcoming(Utc)
            .next()
            .ok_or_else(|| "Unable to determine next execution time".to_string())?;

        let config = DcaConfig {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            wallet_address: request.wallet_address,
            input_mint: request.input_mint,
            output_mint: request.output_mint,
            input_symbol: request.input_symbol,
            output_symbol: request.output_symbol,
            input_decimals: request.input_decimals,
            output_decimals: request.output_decimals,
            amount_per_execution: request.amount_per_execution,
            total_budget: request.total_budget,
            spent_amount: 0.0,
            schedule_cron: request.schedule_cron.clone(),
            slippage_bps: request.slippage_bps,
            priority_fee_micro_lamports: request.priority_fee_micro_lamports,
            max_price_impact_pct: request.max_price_impact_pct,
            daily_spend_cap: request.daily_spend_cap,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_execution: None,
            next_execution: Some(next_execution),
        };

        self.db
            .write()
            .await
            .create_config(&config)
            .await
            .map_err(|e| format!("Failed to persist DCA config: {e}"))?;

        self.schedules
            .write()
            .await
            .insert(config.id.clone(), schedule);

        Ok(config)
    }

    pub async fn get_dca(&self, id: &str) -> Result<DcaConfig, String> {
        self.db
            .read()
            .await
            .get_config(id)
            .await
            .map_err(|e| format!("Failed to load DCA config: {e}"))?
            .ok_or_else(|| "DCA config not found".to_string())
    }

    pub async fn list_dcas(&self, wallet_address: &str) -> Result<Vec<DcaConfig>, String> {
        self.db
            .read()
            .await
            .list_configs(wallet_address)
            .await
            .map_err(|e| format!("Failed to list DCA configs: {e}"))
    }

    pub async fn pause_dca(&self, id: &str) -> Result<(), String> {
        self.db
            .write()
            .await
            .update_config_status(id, false)
            .await
            .map_err(|e| format!("Failed to pause DCA: {e}"))?;

        self.schedules.write().await.remove(id);
        Ok(())
    }

    pub async fn resume_dca(&self, id: &str) -> Result<DcaConfig, String> {
        let mut config = self.get_dca(id).await?;
        let schedule = Schedule::from_str(&config.schedule_cron)
            .map_err(|e| format!("Invalid stored cron expression: {e}"))?;

        let next_execution = schedule
            .upcoming(Utc)
            .next()
            .ok_or_else(|| "Unable to determine next execution time".to_string())?;

        self.db
            .write()
            .await
            .update_config_status(id, true)
            .await
            .map_err(|e| format!("Failed to resume DCA: {e}"))?;

        self.db
            .write()
            .await
            .update_execution_window(id, Utc::now(), Some(next_execution))
            .await
            .map_err(|e| format!("Failed to update execution window: {e}"))?;

        config.is_active = true;
        config.next_execution = Some(next_execution);
        config.last_execution = Some(Utc::now());

        self.schedules
            .write()
            .await
            .insert(config.id.clone(), schedule);

        Ok(config)
    }

    pub async fn delete_dca(&self, id: &str) -> Result<(), String> {
        self.db
            .write()
            .await
            .delete_config(id)
            .await
            .map_err(|e| format!("Failed to delete DCA config: {e}"))?;

        self.schedules.write().await.remove(id);
        Ok(())
    }

    pub async fn executions(&self, id: &str) -> Result<Vec<DcaExecution>, String> {
        self.db
            .read()
            .await
            .get_executions(id)
            .await
            .map_err(|e| format!("Failed to fetch execution history: {e}"))
    }

    pub async fn performance(&self, id: &str) -> Result<DcaPerformance, String> {
        let config = self.get_dca(id).await?;
        let summary = self
            .db
            .read()
            .await
            .execution_summary(id)
            .await
            .map_err(|e| format!("Failed to compute execution summary: {e}"))?;

        let average_price = if summary.acquired > 0.0 {
            summary.invested / summary.acquired
        } else {
            0.0
        };

        let success_rate = if summary.executions > 0 {
            (summary.successes as f64 / summary.executions as f64) * 100.0
        } else {
            0.0
        };

        Ok(DcaPerformance {
            total_invested: summary.invested,
            total_acquired: summary.acquired,
            average_price,
            execution_count: summary.executions,
            success_count: summary.successes,
            success_rate,
            last_execution: config.last_execution,
            next_execution: config.next_execution,
            remaining_budget: (config.total_budget - config.spent_amount).max(0.0),
        })
    }

    pub async fn initialize_schedules(&self) -> Result<(), String> {
        let configs = self
            .db
            .read()
            .await
            .get_active_configs()
            .await
            .map_err(|e| format!("Failed to load active DCA configs: {e}"))?;

        let mut schedules = self.schedules.write().await;
        schedules.clear();
        for config in configs {
            if let Ok(schedule) = Schedule::from_str(&config.schedule_cron) {
                schedules.insert(config.id.clone(), schedule);
            }
        }

        Ok(())
    }

    pub async fn check_and_execute(&self) -> Result<(), String> {
        let due_configs = self
            .db
            .read()
            .await
            .get_due_configs(Utc::now())
            .await
            .map_err(|e| format!("Failed to fetch due DCA configs: {e}"))?;

        for config in due_configs {
            if let Err(err) = self.execute_config(&config).await {
                eprintln!("Failed to run DCA {}: {}", config.id, err);
            }
        }

        Ok(())
    }

    async fn execute_config(&self, config: &DcaConfig) -> Result<(), String> {
        if config.spent_amount + config.amount_per_execution > config.total_budget {
            self.db
                .write()
                .await
                .update_config_status(&config.id, false)
                .await
                .ok();

            self.log_execution(
                config,
                0.0,
                0.0,
                0.0,
                "skipped",
                Some("Total budget exceeded".into()),
                None,
            )
            .await?;
            self.schedule_next(config, None).await?;
            return Err("Total budget exceeded".into());
        }

        if let Some(cap) = config.daily_spend_cap {
            let start_of_day = start_of_day_utc(Utc::now());
            let spent_today = self
                .db
                .read()
                .await
                .spend_since(&config.id, start_of_day)
                .await
                .map_err(|e| format!("Failed to compute daily spend: {e}"))?;

            if spent_today + config.amount_per_execution > cap {
                self.log_execution(
                    config,
                    0.0,
                    0.0,
                    0.0,
                    "skipped",
                    Some("Daily spend cap reached".into()),
                    None,
                )
                .await?;
                self.schedule_next(config, None).await?;
                return Ok(());
            }
        }

        let amount_in_units = to_base_units(config.amount_per_execution, config.input_decimals)?;

        let quote_input = QuoteCommandInput {
            input_mint: config.input_mint.clone(),
            output_mint: config.output_mint.clone(),
            amount: amount_in_units,
            slippage_bps: Some(config.slippage_bps as u16),
            swap_mode: Some(SwapMode::ExactIn),
            platform_fee_bps: None,
            only_direct_routes: None,
            referral_account: None,
            as_legacy_transaction: None,
            priority_fee_config: Some(PriorityFeeConfig {
                compute_unit_price_micro_lamports: Some(config.priority_fee_micro_lamports as u64),
                auto_multiplier: None,
            }),
        };

        let quote_result: QuoteResult = jupiter_quote(quote_input)
            .await
            .map_err(|e| format!("Failed to fetch quote: {e}"))?;

        let price_impact_pct = quote_result.quote.price_impact_pct * 100.0;
        if price_impact_pct > config.max_price_impact_pct {
            self.log_execution(
                config,
                0.0,
                0.0,
                0.0,
                "skipped",
                Some(format!(
                    "Price impact {}% exceeds configured maximum of {}%",
                    price_impact_pct, config.max_price_impact_pct
                )),
                None,
            )
            .await?;
            self.schedule_next(config, Some(&quote_result)).await?;
            return Ok(());
        }

        let output_amount = parse_amount(&quote_result.quote.output_amount, config.output_decimals);
        let input_amount = config.amount_per_execution;
        let price = if output_amount > 0.0 {
            input_amount / output_amount
        } else {
            0.0
        };

        let execution_time = Utc::now();

        self.log_execution(
            config,
            input_amount,
            output_amount,
            price,
            "success",
            None,
            Some(format!("simulated_{}", Uuid::new_v4())),
        )
        .await?;

        let new_spent = config.spent_amount + input_amount;
        self.db
            .write()
            .await
            .update_spent_amount(&config.id, new_spent)
            .await
            .map_err(|e| format!("Failed to update spent amount: {e}"))?;

        self.schedule_next(config, Some(&quote_result)).await?;
        self.emit_execution_event(
            config,
            input_amount,
            output_amount,
            price,
            "success",
            execution_time,
        );

        Ok(())
    }

    async fn log_execution(
        &self,
        config: &DcaConfig,
        input_amount: f64,
        output_amount: f64,
        price: f64,
        status: &str,
        error_message: Option<String>,
        tx_signature: Option<String>,
    ) -> Result<(), String> {
        let execution = DcaExecution {
            id: Uuid::new_v4().to_string(),
            dca_config_id: config.id.clone(),
            input_amount,
            output_amount,
            price,
            total_cost: input_amount,
            executed_at: Utc::now(),
            status: status.to_string(),
            error_message,
            tx_signature,
        };

        self.db
            .write()
            .await
            .record_execution(&execution)
            .await
            .map_err(|e| format!("Failed to persist execution log: {e}"))
    }

    async fn schedule_next(
        &self,
        config: &DcaConfig,
        quote_result: Option<&QuoteResult>,
    ) -> Result<(), String> {
        let next_execution = self
            .schedules
            .read()
            .await
            .get(&config.id)
            .cloned()
            .and_then(|schedule| schedule.upcoming(Utc).next());

        let last_execution_time = quote_result
            .and_then(|_| Some(Utc::now()))
            .unwrap_or_else(Utc::now);

        self.db
            .write()
            .await
            .update_execution_window(&config.id, last_execution_time, next_execution)
            .await
            .map_err(|e| format!("Failed to update execution schedule: {e}"))
    }

    fn emit_execution_event(
        &self,
        config: &DcaConfig,
        input_amount: f64,
        output_amount: f64,
        price: f64,
        status: &str,
        _timestamp: DateTime<Utc>,
    ) {
        let event = DcaExecutionEvent {
            dca_id: config.id.clone(),
            name: config.name.clone(),
            input_symbol: config.input_symbol.clone(),
            output_symbol: config.output_symbol.clone(),
            input_amount,
            output_amount,
            price,
            status: status.to_string(),
            tx_signature: None,
        };

        let _ = self.app_handle.emit("dca_execution", event);
    }

    pub async fn start_monitoring(manager: Arc<Self>) {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            if let Err(err) = manager.check_and_execute().await {
                eprintln!("Error running DCA scheduler: {err}");
            }
        }
    }
}

fn to_base_units(amount: f64, decimals: i32) -> Result<u64, String> {
    if amount < 0.0 {
        return Err("Amount cannot be negative".into());
    }

    let factor = 10f64.powi(decimals);
    let value = (amount * factor).round();

    if value > u64::MAX as f64 {
        return Err("Amount is too large".into());
    }

    Ok(value as u64)
}

fn parse_amount(raw: &str, decimals: i32) -> f64 {
    raw.parse::<f64>().unwrap_or_default() / 10f64.powi(decimals)
}

fn start_of_day_utc(now: DateTime<Utc>) -> DateTime<Utc> {
    let date = now.date_naive();
    let midnight: NaiveDateTime = date.and_hms_opt(0, 0, 0).unwrap();
    DateTime::<Utc>::from_naive_utc_and_offset(midnight, Utc)
}

pub fn preview_next_execution(cron: &str, after: DateTime<Utc>) -> Result<DateTime<Utc>, String> {
    let schedule = Schedule::from_str(cron).map_err(|e| format!("Invalid cron expression: {e}"))?;
    schedule
        .after(&after)
        .next()
        .ok_or_else(|| "Unable to determine next execution".to_string())
}

pub type SharedDcaManager = Arc<DcaManager>;

pub struct DcaState {
    pub db: SharedDcaDatabase,
    pub manager: SharedDcaManager,
}

static DCA_STATE: OnceCell<DcaState> = OnceCell::const_new();

pub async fn init_dca(app_handle: &AppHandle) -> Result<(), String> {
    if DCA_STATE.get().is_some() {
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

    let db = DcaDatabase::new(db_path)
        .await
        .map_err(|e| format!("Failed to initialize DCA database: {e}"))?;

    let shared_db = Arc::new(RwLock::new(db));
    let manager = Arc::new(DcaManager::new(shared_db.clone(), app_handle.clone()));
    manager.initialize_schedules().await?;

    let handle_for_task = app_handle.clone();
    let manager_for_task = manager.clone();
    tauri::async_runtime::spawn(async move {
        DcaManager::start_monitoring(manager_for_task).await;
        let _ = handle_for_task.emit("dca_scheduler_stopped", "Scheduler stopped");
    });

    DCA_STATE
        .set(DcaState {
            db: shared_db,
            manager: manager.clone(),
        })
        .map_err(|_| "DCA state already initialized".to_string())?;

    Ok(())
}

fn require_state<'a>() -> Result<&'a DcaState, String> {
    DCA_STATE
        .get()
        .ok_or_else(|| "DCA module not initialized".to_string())
}

#[tauri::command]
pub async fn dca_init(handle: AppHandle) -> Result<(), String> {
    init_dca(&handle).await
}

#[tauri::command]
pub async fn dca_create(request: CreateDcaRequest) -> Result<DcaConfig, String> {
    let state = require_state()?;
    state.manager.create_dca(request).await
}

#[tauri::command]
pub async fn dca_list(wallet_address: String) -> Result<Vec<DcaConfig>, String> {
    let state = require_state()?;
    state.manager.list_dcas(&wallet_address).await
}

#[tauri::command]
pub async fn dca_get(id: String) -> Result<DcaConfig, String> {
    let state = require_state()?;
    state.manager.get_dca(&id).await
}

#[tauri::command]
pub async fn dca_pause(id: String) -> Result<(), String> {
    let state = require_state()?;
    state.manager.pause_dca(&id).await
}

#[tauri::command]
pub async fn dca_resume(id: String) -> Result<DcaConfig, String> {
    let state = require_state()?;
    state.manager.resume_dca(&id).await
}

#[tauri::command]
pub async fn dca_delete(id: String) -> Result<(), String> {
    let state = require_state()?;
    state.manager.delete_dca(&id).await
}

#[tauri::command]
pub async fn dca_history(id: String) -> Result<Vec<DcaExecution>, String> {
    let state = require_state()?;
    state.manager.executions(&id).await
}

#[tauri::command]
pub async fn dca_performance(id: String) -> Result<DcaPerformance, String> {
    let state = require_state()?;
    state.manager.performance(&id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_next_execution_daily() {
        let cron = "0 0 12 * * *"; // Every day at noon UTC
        let now = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let next = preview_next_execution(cron, now).unwrap();
        assert!(next > now);
        assert_eq!(next.hour(), 12);
    }

    #[test]
    fn test_preview_next_execution_invalid() {
        let cron = "invalid cron";
        let now = Utc::now();
        let result = preview_next_execution(cron, now);
        assert!(result.is_err());
    }
}
