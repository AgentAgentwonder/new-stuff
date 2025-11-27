use crate::utils::Rfc3339DateTime;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite, SqlitePool};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::{OnceCell, RwLock};
use uuid::Uuid;

use crate::trading::types::{OrderSide, OrderType};

const DEFAULT_INITIAL_BALANCE: f64 = 10_000.0;
const MINIMUM_QUANTITY: f64 = 1e-9;

// ============================================================================
// Types and Structs
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAccount {
    pub id: String,
    pub balance: f64,
    pub initial_balance: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for PaperAccount {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(PaperAccount {
            id: row.try_get("id")?,
            balance: row.try_get("balance")?,
            initial_balance: row.try_get("initial_balance")?,
            created_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("created_at")?)?.into(),
            updated_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("updated_at")?)?.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTrade {
    pub id: String,
    pub account_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: f64,
    pub price: f64,
    pub trading_fee: f64,
    pub network_fee: f64,
    pub price_impact_fee: f64,
    pub fee: f64,
    pub slippage: f64,
    pub total_cost: f64,
    pub timestamp: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for PaperTrade {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(PaperTrade {
            id: row.try_get("id")?,
            account_id: row.try_get("account_id")?,
            symbol: row.try_get("symbol")?,
            side: row.try_get("side")?,
            order_type: row.try_get("order_type")?,
            quantity: row.try_get("quantity")?,
            price: row.try_get("price")?,
            trading_fee: row.try_get("trading_fee")?,
            network_fee: row.try_get("network_fee")?,
            price_impact_fee: row.try_get("price_impact_fee")?,
            fee: row.try_get("fee")?,
            slippage: row.try_get("slippage")?,
            total_cost: row.try_get("total_cost")?,
            timestamp: Rfc3339DateTime::try_from(row.try_get::<String, _>("timestamp")?)?.into(),
        })
    }
}

impl PaperTrade {
    pub fn fee_breakdown(&self) -> FeeBreakdown {
        FeeBreakdown {
            trading_fee: self.trading_fee,
            network_fee: self.network_fee,
            price_impact_fee: self.price_impact_fee,
            total_fee: self.fee,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPosition {
    pub id: String,
    pub account_id: String,
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub opened_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for PaperPosition {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(PaperPosition {
            id: row.try_get("id")?,
            account_id: row.try_get("account_id")?,
            symbol: row.try_get("symbol")?,
            quantity: row.try_get("quantity")?,
            entry_price: row.try_get("entry_price")?,
            current_price: row.try_get("current_price")?,
            unrealized_pnl: row.try_get("unrealized_pnl")?,
            opened_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("opened_at")?)?.into(),
            updated_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("updated_at")?)?.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePaperTradeRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub trading_fee: f64,
    pub network_fee: f64,
    pub price_impact_fee: f64,
    pub total_fee: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradeResult {
    pub trade: PaperTrade,
    pub account: PaperAccount,
    pub position: Option<PaperPosition>,
    pub fees: FeeBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPerformance {
    pub total_trades: i64,
    pub winning_trades: i64,
    pub losing_trades: i64,
    pub total_pnl: f64,
    pub total_fees: f64,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub current_balance: f64,
    pub initial_balance: f64,
    pub return_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageConfig {
    pub small_order_threshold: f64,  // $100
    pub medium_order_threshold: f64, // $1000
    pub small_slippage: f64,         // 0.1%
    pub medium_slippage: f64,        // 0.2%
    pub large_slippage: f64,         // 0.5%
    pub randomness_factor: f64,      // Variation factor
}

impl Default for SlippageConfig {
    fn default() -> Self {
        Self {
            small_order_threshold: 100.0,
            medium_order_threshold: 1000.0,
            small_slippage: 0.001,
            medium_slippage: 0.002,
            large_slippage: 0.005,
            randomness_factor: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeConfig {
    pub trading_fee_percentage: f64, // 0.1%
    pub network_fee: f64,            // $0.0005
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            trading_fee_percentage: 0.001,
            network_fee: 0.0005,
        }
    }
}

#[derive(Debug, Clone)]
struct PositionLot {
    quantity: f64,
    price: f64,
    fee_per_unit: f64,
}

// ============================================================================
// Database
// ============================================================================

pub struct PaperTradingDatabase {
    pool: Pool<Sqlite>,
}

impl PaperTradingDatabase {
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
            CREATE TABLE IF NOT EXISTS paper_accounts (
                id TEXT PRIMARY KEY,
                balance REAL NOT NULL,
                initial_balance REAL NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS paper_trades (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                side TEXT NOT NULL,
                order_type TEXT NOT NULL,
                quantity REAL NOT NULL,
                price REAL NOT NULL,
                trading_fee REAL NOT NULL,
                network_fee REAL NOT NULL,
                price_impact_fee REAL NOT NULL,
                fee REAL NOT NULL,
                slippage REAL NOT NULL,
                total_cost REAL NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (account_id) REFERENCES paper_accounts(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS paper_positions (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                symbol TEXT NOT NULL,
                quantity REAL NOT NULL,
                entry_price REAL NOT NULL,
                current_price REAL NOT NULL,
                unrealized_pnl REAL NOT NULL,
                opened_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (account_id) REFERENCES paper_accounts(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_paper_trades_account ON paper_trades(account_id);
            CREATE INDEX IF NOT EXISTS idx_paper_trades_timestamp ON paper_trades(timestamp);
            CREATE INDEX IF NOT EXISTS idx_paper_positions_account ON paper_positions(account_id);
            CREATE INDEX IF NOT EXISTS idx_paper_positions_symbol ON paper_positions(symbol);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_or_create_account(
        &self,
        initial_balance: f64,
    ) -> Result<PaperAccount, sqlx::Error> {
        let existing = sqlx::query_as::<_, PaperAccount>(
            "SELECT * FROM paper_accounts ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(account) = existing {
            return Ok(account);
        }

        let account = PaperAccount {
            id: Uuid::new_v4().to_string(),
            balance: initial_balance,
            initial_balance,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO paper_accounts (id, balance, initial_balance, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(&account.id)
        .bind(account.balance)
        .bind(account.initial_balance)
        .bind(account.created_at.to_rfc3339())
        .bind(account.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(account)
    }

    pub async fn get_account(&self) -> Result<Option<PaperAccount>, sqlx::Error> {
        sqlx::query_as::<_, PaperAccount>(
            "SELECT * FROM paper_accounts ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_balance(
        &self,
        account_id: &str,
        new_balance: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE paper_accounts
            SET balance = ?1, updated_at = ?2
            WHERE id = ?3
            "#,
        )
        .bind(new_balance)
        .bind(Utc::now().to_rfc3339())
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn reset_account(&self, initial_balance: f64) -> Result<PaperAccount, sqlx::Error> {
        sqlx::query("DELETE FROM paper_positions")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM paper_trades")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM paper_accounts")
            .execute(&self.pool)
            .await?;

        self.get_or_create_account(initial_balance).await
    }

    pub async fn create_trade(&self, trade: &PaperTrade) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO paper_trades (
                id, account_id, symbol, side, order_type, quantity,
                price, trading_fee, network_fee, price_impact_fee, fee,
                slippage, total_cost, timestamp
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6,
                ?7, ?8, ?9, ?10, ?11,
                ?12, ?13, ?14
            )
            "#,
        )
        .bind(&trade.id)
        .bind(&trade.account_id)
        .bind(&trade.symbol)
        .bind(&trade.side)
        .bind(&trade.order_type)
        .bind(trade.quantity)
        .bind(trade.price)
        .bind(trade.trading_fee)
        .bind(trade.network_fee)
        .bind(trade.price_impact_fee)
        .bind(trade.fee)
        .bind(trade.slippage)
        .bind(trade.total_cost)
        .bind(trade.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_trade_history(
        &self,
        account_id: &str,
    ) -> Result<Vec<PaperTrade>, sqlx::Error> {
        sqlx::query_as::<_, PaperTrade>(
            "SELECT * FROM paper_trades WHERE account_id = ?1 ORDER BY timestamp ASC",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_position(
        &self,
        account_id: &str,
        symbol: &str,
    ) -> Result<Option<PaperPosition>, sqlx::Error> {
        sqlx::query_as::<_, PaperPosition>(
            "SELECT * FROM paper_positions WHERE account_id = ?1 AND symbol = ?2",
        )
        .bind(account_id)
        .bind(symbol)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_all_positions(
        &self,
        account_id: &str,
    ) -> Result<Vec<PaperPosition>, sqlx::Error> {
        sqlx::query_as::<_, PaperPosition>("SELECT * FROM paper_positions WHERE account_id = ?1")
            .bind(account_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_position(&self, position: &PaperPosition) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO paper_positions (
                id, account_id, symbol, quantity, entry_price,
                current_price, unrealized_pnl, opened_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                quantity = excluded.quantity,
                entry_price = excluded.entry_price,
                current_price = excluded.current_price,
                unrealized_pnl = excluded.unrealized_pnl,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&position.id)
        .bind(&position.account_id)
        .bind(&position.symbol)
        .bind(position.quantity)
        .bind(position.entry_price)
        .bind(position.current_price)
        .bind(position.unrealized_pnl)
        .bind(position.opened_at.to_rfc3339())
        .bind(position.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_position(&self, position_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM paper_positions WHERE id = ?1")
            .bind(position_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_position_price(
        &self,
        position_id: &str,
        current_price: f64,
        unrealized_pnl: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE paper_positions
            SET current_price = ?1, unrealized_pnl = ?2, updated_at = ?3
            WHERE id = ?4
            "#,
        )
        .bind(current_price)
        .bind(unrealized_pnl)
        .bind(Utc::now().to_rfc3339())
        .bind(position_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_performance(&self, account_id: &str) -> Result<PaperPerformance, sqlx::Error> {
        let mut trades = self.get_trade_history(account_id).await?;
        let account =
            sqlx::query_as::<_, PaperAccount>("SELECT * FROM paper_accounts WHERE id = ?1")
                .bind(account_id)
                .fetch_one(&self.pool)
                .await?;

        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_win = 0.0_f64;
        let mut total_loss = 0.0_f64;
        let mut largest_win = 0.0_f64;
        let mut largest_loss = 0.0_f64;
        let mut total_fees = 0.0_f64;

        let mut lots: HashMap<String, VecDeque<PositionLot>> = HashMap::new();

        trades.sort_by_key(|t| t.timestamp);

        for trade in &trades {
            total_fees += trade.fee;
            let fee_per_unit = if trade.quantity.abs() > MINIMUM_QUANTITY {
                trade.fee / trade.quantity
            } else {
                0.0
            };

            match trade.side.as_str() {
                "buy" => {
                    let entry = lots
                        .entry(trade.symbol.clone())
                        .or_insert_with(VecDeque::new);
                    entry.push_back(PositionLot {
                        quantity: trade.quantity,
                        price: trade.price,
                        fee_per_unit: fee_per_unit,
                    });
                }
                "sell" => {
                    let entry = lots
                        .entry(trade.symbol.clone())
                        .or_insert_with(VecDeque::new);

                    let mut qty_remaining = trade.quantity;
                    let mut trade_pnl = 0.0;

                    while qty_remaining > MINIMUM_QUANTITY {
                        if let Some(front) = entry.front_mut() {
                            let matched_qty = front.quantity.min(qty_remaining);
                            let buy_fee = front.fee_per_unit * matched_qty;
                            let sell_fee = fee_per_unit * matched_qty;
                            trade_pnl +=
                                (trade.price - front.price) * matched_qty - sell_fee - buy_fee;

                            front.quantity -= matched_qty;
                            qty_remaining -= matched_qty;

                            if front.quantity <= MINIMUM_QUANTITY {
                                entry.pop_front();
                            }
                        } else {
                            // No matching long position; treat remaining as zero exposure
                            break;
                        }
                    }

                    if trade_pnl > 0.0 {
                        winning_trades += 1;
                        total_win += trade_pnl;
                        largest_win = largest_win.max(trade_pnl);
                    } else if trade_pnl < 0.0 {
                        losing_trades += 1;
                        total_loss += trade_pnl;
                        largest_loss = if largest_loss == 0.0 {
                            trade_pnl
                        } else {
                            largest_loss.min(trade_pnl)
                        };
                    }
                }
                _ => {}
            }
        }

        let total_trades = trades.len() as i64;
        let total_pnl = account.balance - account.initial_balance;
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };
        let avg_win = if winning_trades > 0 {
            total_win / winning_trades as f64
        } else {
            0.0
        };
        let avg_loss = if losing_trades > 0 {
            total_loss / losing_trades as f64
        } else {
            0.0
        };
        let return_percentage = if account.initial_balance > 0.0 {
            ((account.balance - account.initial_balance) / account.initial_balance) * 100.0
        } else {
            0.0
        };

        Ok(PaperPerformance {
            total_trades,
            winning_trades,
            losing_trades,
            total_pnl,
            total_fees,
            win_rate,
            avg_win,
            avg_loss,
            largest_win,
            largest_loss,
            current_balance: account.balance,
            initial_balance: account.initial_balance,
            return_percentage,
        })
    }
}

pub type SharedPaperTradingDatabase = Arc<RwLock<PaperTradingDatabase>>;

// ============================================================================
// Paper Trading Manager
// ============================================================================

pub struct PaperTradingManager {
    db: SharedPaperTradingDatabase,
    slippage_config: SlippageConfig,
    fee_config: FeeConfig,
    current_prices: Arc<RwLock<HashMap<String, f64>>>,
}

impl PaperTradingManager {
    pub fn new(db: SharedPaperTradingDatabase) -> Self {
        Self::with_config(db, SlippageConfig::default(), FeeConfig::default())
    }

    pub fn with_config(
        db: SharedPaperTradingDatabase,
        slippage_config: SlippageConfig,
        fee_config: FeeConfig,
    ) -> Self {
        Self {
            db,
            slippage_config,
            fee_config,
            current_prices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn validate_request(&self, request: &ExecutePaperTradeRequest) -> Result<(), String> {
        if request.quantity <= 0.0 {
            return Err("Quantity must be greater than zero".to_string());
        }
        if request.price <= 0.0 {
            return Err("Price must be greater than zero".to_string());
        }

        match request.order_type {
            OrderType::Market => Ok(()),
            OrderType::Limit | OrderType::TakeProfit => {
                let limit_price = request
                    .limit_price
                    .ok_or_else(|| "Limit price required for limit orders".to_string())?;
                match request.side {
                    OrderSide::Buy => {
                        if request.price > limit_price {
                            Err("Market price above limit price".to_string())
                        } else {
                            Ok(())
                        }
                    }
                    OrderSide::Sell => {
                        if request.price < limit_price {
                            Err("Market price below limit price".to_string())
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            OrderType::StopLoss => {
                let stop_price = request
                    .stop_price
                    .ok_or_else(|| "Stop price required for stop orders".to_string())?;
                match request.side {
                    OrderSide::Buy => {
                        if request.price < stop_price {
                            Err("Market price below stop price".to_string())
                        } else {
                            Ok(())
                        }
                    }
                    OrderSide::Sell => {
                        if request.price > stop_price {
                            Err("Market price above stop price".to_string())
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            _ => Err("Unsupported order type for paper trading".to_string()),
        }
    }

    fn calculate_slippage(&self, order_value: f64) -> f64 {
        let base_slippage = if order_value < self.slippage_config.small_order_threshold {
            self.slippage_config.small_slippage
        } else if order_value < self.slippage_config.medium_order_threshold {
            self.slippage_config.medium_slippage
        } else {
            self.slippage_config.large_slippage
        };

        let variance_range = self.slippage_config.randomness_factor;
        let variance = if variance_range > 0.0 {
            rand::random_range(-variance_range..variance_range)
        } else {
            0.0
        };

        (base_slippage * (1.0 + variance)).max(0.0)
    }

    fn calculate_trading_fee(&self, order_value: f64) -> f64 {
        order_value * self.fee_config.trading_fee_percentage
    }

    fn calculate_price_impact_fee(&self, order_value: f64, slippage: f64) -> f64 {
        if order_value > self.slippage_config.medium_order_threshold {
            order_value * slippage
        } else {
            0.0
        }
    }

    fn execution_price(&self, market_price: f64, slippage: f64, side: OrderSide) -> f64 {
        match side {
            OrderSide::Buy => market_price * (1.0 + slippage),
            OrderSide::Sell => market_price * (1.0 - slippage),
        }
    }

    pub async fn execute_trade(
        &self,
        request: ExecutePaperTradeRequest,
    ) -> Result<PaperTradeResult, String> {
        self.validate_request(&request)?;

        let db_read = self.db.read().await;
        let mut account = db_read
            .get_or_create_account(DEFAULT_INITIAL_BALANCE)
            .await
            .map_err(|e| format!("Failed to load paper account: {e}"))?;

        let order_value = request.quantity * request.price;
        let slippage = self.calculate_slippage(order_value);
        let execution_price = self.execution_price(request.price, slippage, request.side);
        let executed_value = request.quantity * execution_price;

        let trading_fee = self.calculate_trading_fee(executed_value);
        let network_fee = self.fee_config.network_fee;
        let price_impact_fee = self.calculate_price_impact_fee(executed_value, slippage);
        let total_fee = trading_fee + network_fee + price_impact_fee;

        let total_cost = match request.side {
            OrderSide::Buy => executed_value + total_fee,
            OrderSide::Sell => executed_value - total_fee,
        };

        if request.side == OrderSide::Buy && total_cost > account.balance + f64::EPSILON {
            return Err("Insufficient paper balance".to_string());
        }

        if request.side == OrderSide::Sell {
            self.ensure_position_exists(&db_read, &account.id, &request)
                .await?;
        }

        db_read
            .update_balance(
                &account.id,
                match request.side {
                    OrderSide::Buy => account.balance - total_cost,
                    OrderSide::Sell => account.balance + total_cost,
                },
            )
            .await
            .map_err(|e| format!("Failed to update paper balance: {e}"))?;

        account = db_read
            .get_or_create_account(DEFAULT_INITIAL_BALANCE)
            .await
            .map_err(|e| format!("Failed to reload paper account: {e}"))?;

        let trade = PaperTrade {
            id: Uuid::new_v4().to_string(),
            account_id: account.id.clone(),
            symbol: request.symbol.clone(),
            side: request.side.to_string(),
            order_type: request.order_type.to_string(),
            quantity: request.quantity,
            price: execution_price,
            trading_fee,
            network_fee,
            price_impact_fee,
            fee: total_fee,
            slippage,
            total_cost,
            timestamp: Utc::now(),
        };

        db_read
            .create_trade(&trade)
            .await
            .map_err(|e| format!("Failed to store paper trade: {e}"))?;

        let position = self
            .update_position(&db_read, &account.id, &request, execution_price)
            .await?;

        Ok(PaperTradeResult {
            fees: trade.fee_breakdown(),
            trade,
            account,
            position,
        })
    }

    async fn ensure_position_exists(
        &self,
        db: &PaperTradingDatabase,
        account_id: &str,
        request: &ExecutePaperTradeRequest,
    ) -> Result<(), String> {
        let position = db
            .get_position(account_id, &request.symbol)
            .await
            .map_err(|e| format!("Failed to validate position: {e}"))?;

        if let Some(position) = position {
            if request.quantity - position.quantity > MINIMUM_QUANTITY {
                return Err("Insufficient position size to sell".to_string());
            }
            Ok(())
        } else {
            Err("No open position to sell".to_string())
        }
    }

    async fn update_position(
        &self,
        db: &PaperTradingDatabase,
        account_id: &str,
        request: &ExecutePaperTradeRequest,
        execution_price: f64,
    ) -> Result<Option<PaperPosition>, String> {
        let existing = db
            .get_position(account_id, &request.symbol)
            .await
            .map_err(|e| format!("Failed to fetch paper position: {e}"))?;

        match existing {
            Some(mut position) => match request.side {
                OrderSide::Buy => {
                    let total_cost = position.quantity * position.entry_price
                        + request.quantity * execution_price;
                    let new_quantity = position.quantity + request.quantity;

                    position.entry_price = total_cost / new_quantity;
                    position.quantity = new_quantity;
                    position.current_price = execution_price;
                    position.unrealized_pnl =
                        (execution_price - position.entry_price) * position.quantity;
                    position.updated_at = Utc::now();

                    db.upsert_position(&position)
                        .await
                        .map_err(|e| format!("Failed to update paper position: {e}"))?;

                    Ok(Some(position))
                }
                OrderSide::Sell => {
                    if request.quantity - position.quantity >= MINIMUM_QUANTITY {
                        db.delete_position(&position.id)
                            .await
                            .map_err(|e| format!("Failed to clear paper position: {e}"))?;
                        Ok(None)
                    } else {
                        position.quantity -= request.quantity;
                        position.current_price = execution_price;
                        position.unrealized_pnl =
                            (execution_price - position.entry_price) * position.quantity;
                        position.updated_at = Utc::now();

                        db.upsert_position(&position)
                            .await
                            .map_err(|e| format!("Failed to update paper position: {e}"))?;

                        Ok(Some(position))
                    }
                }
            },
            None => {
                if request.side == OrderSide::Sell {
                    return Err("No open position to sell".to_string());
                }

                let position = PaperPosition {
                    id: Uuid::new_v4().to_string(),
                    account_id: account_id.to_string(),
                    symbol: request.symbol.clone(),
                    quantity: request.quantity,
                    entry_price: execution_price,
                    current_price: execution_price,
                    unrealized_pnl: 0.0,
                    opened_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                db.upsert_position(&position)
                    .await
                    .map_err(|e| format!("Failed to create paper position: {e}"))?;

                Ok(Some(position))
            }
        }
    }

    pub async fn get_account(&self) -> Result<PaperAccount, String> {
        let db_read = self.db.read().await;
        db_read
            .get_or_create_account(DEFAULT_INITIAL_BALANCE)
            .await
            .map_err(|e| format!("Failed to load paper account: {e}"))
    }

    pub async fn reset_account(
        &self,
        initial_balance: Option<f64>,
    ) -> Result<PaperAccount, String> {
        let db_read = self.db.read().await;
        db_read
            .reset_account(initial_balance.unwrap_or(DEFAULT_INITIAL_BALANCE))
            .await
            .map_err(|e| format!("Failed to reset paper account: {e}"))
    }

    pub async fn get_positions(&self) -> Result<Vec<PaperPosition>, String> {
        let db_read = self.db.read().await;
        let account = db_read
            .get_or_create_account(DEFAULT_INITIAL_BALANCE)
            .await
            .map_err(|e| format!("Failed to load paper account: {e}"))?;

        db_read
            .get_all_positions(&account.id)
            .await
            .map_err(|e| format!("Failed to load paper positions: {e}"))
    }

    pub async fn get_trade_history(&self) -> Result<Vec<PaperTrade>, String> {
        let db_read = self.db.read().await;
        let account = db_read
            .get_or_create_account(DEFAULT_INITIAL_BALANCE)
            .await
            .map_err(|e| format!("Failed to load paper account: {e}"))?;

        db_read
            .get_trade_history(&account.id)
            .await
            .map_err(|e| format!("Failed to load paper trade history: {e}"))
    }

    pub async fn get_performance(&self) -> Result<PaperPerformance, String> {
        let db_read = self.db.read().await;
        let account = db_read
            .get_or_create_account(DEFAULT_INITIAL_BALANCE)
            .await
            .map_err(|e| format!("Failed to load paper account: {e}"))?;

        db_read
            .get_performance(&account.id)
            .await
            .map_err(|e| format!("Failed to load paper performance: {e}"))
    }

    pub async fn update_position_prices(&self, symbol: &str, price: f64) -> Result<(), String> {
        self.current_prices
            .write()
            .await
            .insert(symbol.to_string(), price);

        let db_read = self.db.read().await;
        let account = db_read
            .get_or_create_account(DEFAULT_INITIAL_BALANCE)
            .await
            .map_err(|e| format!("Failed to load paper account: {e}"))?;

        if let Some(mut position) = db_read
            .get_position(&account.id, symbol)
            .await
            .map_err(|e| format!("Failed to load paper position: {e}"))?
        {
            position.current_price = price;
            position.unrealized_pnl = (price - position.entry_price) * position.quantity;

            db_read
                .update_position_price(&position.id, price, position.unrealized_pnl)
                .await
                .map_err(|e| format!("Failed to update paper position price: {e}"))?;
        }

        Ok(())
    }
}

pub type SharedPaperTradingManager = Arc<PaperTradingManager>;

// ============================================================================
// Global State
// ============================================================================

static PAPER_TRADING_STATE: OnceCell<SharedPaperTradingManager> = OnceCell::const_new();

pub async fn init_paper_trading(app_handle: &AppHandle) -> Result<(), String> {
    if PAPER_TRADING_STATE.get().is_some() {
        return Ok(());
    }

    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {e}"))?;

    let mut db_path = PathBuf::from(app_dir);
    db_path.push("paper_trading.db");

    let db = PaperTradingDatabase::new(db_path)
        .await
        .map_err(|e| format!("Failed to initialize paper trading database: {e}"))?;

    let shared_db = Arc::new(RwLock::new(db));
    let manager = Arc::new(PaperTradingManager::new(shared_db));

    PAPER_TRADING_STATE
        .set(manager)
        .map_err(|_| "Paper trading state already initialized".to_string())?;

    Ok(())
}

fn require_state() -> Result<&'static SharedPaperTradingManager, String> {
    PAPER_TRADING_STATE
        .get()
        .ok_or_else(|| "Paper trading module not initialized".to_string())
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
pub async fn paper_trading_init(handle: AppHandle) -> Result<(), String> {
    init_paper_trading(&handle).await
}

#[tauri::command]
pub async fn get_paper_account() -> Result<PaperAccount, String> {
    let manager = require_state()?;
    manager.get_account().await
}

#[tauri::command]
pub async fn reset_paper_account(initial_balance: Option<f64>) -> Result<PaperAccount, String> {
    let manager = require_state()?;
    manager.reset_account(initial_balance).await
}

#[tauri::command]
pub async fn execute_paper_trade(
    request: ExecutePaperTradeRequest,
) -> Result<PaperTradeResult, String> {
    let manager = require_state()?;
    manager.execute_trade(request).await
}

#[tauri::command]
pub async fn get_paper_positions() -> Result<Vec<PaperPosition>, String> {
    let manager = require_state()?;
    manager.get_positions().await
}

#[tauri::command]
pub async fn get_paper_trade_history() -> Result<Vec<PaperTrade>, String> {
    let manager = require_state()?;
    manager.get_trade_history().await
}

#[tauri::command]
pub async fn get_paper_performance() -> Result<PaperPerformance, String> {
    let manager = require_state()?;
    manager.get_performance().await
}

#[tauri::command]
pub async fn update_paper_position_prices(symbol: String, price: f64) -> Result<(), String> {
    let manager = require_state()?;
    manager.update_position_prices(&symbol, price).await
}

pub fn register_paper_trading_state(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = init_paper_trading(&handle).await {
            eprintln!("Failed to initialize paper trading module: {e}");
        }
    });
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn temp_db_path() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("paper_trading_test_{}.db", Uuid::new_v4()));
        path
    }

    async fn create_manager_with_configs(
        slippage_config: SlippageConfig,
        fee_config: FeeConfig,
    ) -> PaperTradingManager {
        let db_path = temp_db_path();
        let database = PaperTradingDatabase::new(db_path)
            .await
            .expect("failed to create paper trading database");
        PaperTradingManager::with_config(
            Arc::new(RwLock::new(database)),
            slippage_config,
            fee_config,
        )
    }

    fn deterministic_slippage_config() -> SlippageConfig {
        SlippageConfig {
            randomness_factor: 0.0,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_virtual_balance_updates() {
        let manager =
            create_manager_with_configs(deterministic_slippage_config(), FeeConfig::default())
                .await;

        let initial_account = manager.get_account().await.expect("account load");
        assert_eq!(initial_account.initial_balance, DEFAULT_INITIAL_BALANCE);
        assert_eq!(initial_account.balance, DEFAULT_INITIAL_BALANCE);

        let request = ExecutePaperTradeRequest {
            symbol: "SOL".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 1.0,
            price: 100.0,
            limit_price: None,
            stop_price: None,
        };

        let result = manager
            .execute_trade(request)
            .await
            .expect("trade execution");

        assert!(result.account.balance < DEFAULT_INITIAL_BALANCE);
        assert!(result.position.is_some());
    }

    #[tokio::test]
    async fn test_slippage_calculation() {
        let config = SlippageConfig {
            randomness_factor: 0.0,
            ..Default::default()
        };
        let manager = create_manager_with_configs(config.clone(), FeeConfig::default()).await;

        let small_slippage = manager.calculate_slippage(50.0);
        assert!((small_slippage - config.small_slippage).abs() < f64::EPSILON);

        let medium_slippage = manager.calculate_slippage(500.0);
        assert!((medium_slippage - config.medium_slippage).abs() < f64::EPSILON);

        let large_slippage = manager.calculate_slippage(1500.0);
        assert!((large_slippage - config.large_slippage).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_fee_calculations() {
        let manager =
            create_manager_with_configs(deterministic_slippage_config(), FeeConfig::default())
                .await;

        let request = ExecutePaperTradeRequest {
            symbol: "SOL".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 20.0,
            price: 100.0,
            limit_price: None,
            stop_price: None,
        };

        let result = manager
            .execute_trade(request)
            .await
            .expect("trade execution");

        let fees = result.fees;
        assert!(fees.trading_fee > 0.0);
        assert!(fees.network_fee > 0.0);
        assert!(fees.price_impact_fee >= 0.0);
        assert!(
            (fees.total_fee - (fees.trading_fee + fees.network_fee + fees.price_impact_fee)).abs()
                < 1e-9
        );
    }

    #[tokio::test]
    async fn test_position_open_close() {
        let manager =
            create_manager_with_configs(deterministic_slippage_config(), FeeConfig::default())
                .await;

        let buy_request = ExecutePaperTradeRequest {
            symbol: "SOL".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 2.0,
            price: 100.0,
            limit_price: None,
            stop_price: None,
        };
        manager
            .execute_trade(buy_request)
            .await
            .expect("buy execution");

        let sell_request = ExecutePaperTradeRequest {
            symbol: "SOL".to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            quantity: 1.0,
            price: 110.0,
            limit_price: None,
            stop_price: None,
        };
        let sell_result = manager
            .execute_trade(sell_request)
            .await
            .expect("sell execution");

        assert!(sell_result.position.is_some());
        assert!(sell_result
            .position
            .as_ref()
            .map(|p| (p.quantity - 1.0).abs() < 1e-9)
            .unwrap_or(false));
    }

    #[tokio::test]
    async fn test_pnl_calculation() {
        let manager =
            create_manager_with_configs(deterministic_slippage_config(), FeeConfig::default())
                .await;

        let buy_request = ExecutePaperTradeRequest {
            symbol: "SOL".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity: 1.0,
            price: 50.0,
            limit_price: None,
            stop_price: None,
        };
        manager
            .execute_trade(buy_request)
            .await
            .expect("buy execution");

        let sell_request = ExecutePaperTradeRequest {
            symbol: "SOL".to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            quantity: 1.0,
            price: 60.0,
            limit_price: None,
            stop_price: None,
        };
        manager
            .execute_trade(sell_request)
            .await
            .expect("sell execution");

        let performance = manager.get_performance().await.expect("performance");

        assert_eq!(performance.total_trades, 2);
        assert!(performance.total_pnl > 0.0);
    }
}
