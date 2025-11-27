use crate::utils::{OptionalRfc3339DateTime, Rfc3339DateTime};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
    TrailingStop,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "market"),
            OrderType::Limit => write!(f, "limit"),
            OrderType::StopLoss => write!(f, "stop_loss"),
            OrderType::TakeProfit => write!(f, "take_profit"),
            OrderType::TrailingStop => write!(f, "trailing_stop"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "buy"),
            OrderSide::Sell => write!(f, "sell"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
    Failed,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::PartiallyFilled => write!(f, "partially_filled"),
            OrderStatus::Filled => write!(f, "filled"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Expired => write!(f, "expired"),
            OrderStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub status: OrderStatus,
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub amount: f64,
    pub filled_amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lowest_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_order_id: Option<String>,
    pub slippage_bps: i32,
    pub priority_fee_micro_lamports: i32,
    pub wallet_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Order {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Order {
            id: row.try_get("id")?,
            order_type: row.try_get("order_type")?,
            side: row.try_get("side")?,
            status: row.try_get("status")?,
            input_mint: row.try_get("input_mint")?,
            output_mint: row.try_get("output_mint")?,
            input_symbol: row.try_get("input_symbol")?,
            output_symbol: row.try_get("output_symbol")?,
            amount: row.try_get("amount")?,
            filled_amount: row.try_get("filled_amount")?,
            limit_price: row.try_get("limit_price")?,
            stop_price: row.try_get("stop_price")?,
            trailing_percent: row.try_get("trailing_percent")?,
            highest_price: row.try_get("highest_price")?,
            lowest_price: row.try_get("lowest_price")?,
            linked_order_id: row.try_get("linked_order_id")?,
            slippage_bps: row.try_get("slippage_bps")?,
            priority_fee_micro_lamports: row.try_get("priority_fee_micro_lamports")?,
            wallet_address: row.try_get("wallet_address")?,
            created_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("created_at")?)?.into(),
            updated_at: Rfc3339DateTime::try_from(row.try_get::<String, _>("updated_at")?)?.into(),
            triggered_at: OptionalRfc3339DateTime::try_from(row.try_get::<Option<String>, _>("triggered_at")?)?.into(),
            tx_signature: row.try_get("tx_signature")?,
            error_message: row.try_get("error_message")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub order_type: OrderType,
    pub side: OrderSide,
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trailing_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_order_id: Option<String>,
    pub slippage_bps: i32,
    pub priority_fee_micro_lamports: i32,
    pub wallet_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFill {
    pub order_id: String,
    pub filled_amount: f64,
    pub fill_price: f64,
    pub tx_signature: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    pub order_id: String,
    pub status: OrderStatus,
    pub filled_amount: Option<f64>,
    pub tx_signature: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickTradeRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub input_symbol: String,
    pub output_symbol: String,
    pub amount: f64,
    pub side: OrderSide,
    pub wallet_address: String,
    pub use_max: bool,
}
