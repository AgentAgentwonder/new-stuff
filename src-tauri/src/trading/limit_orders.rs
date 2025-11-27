use crate::trading::database::{OrderDatabase, SharedOrderDatabase};
use crate::trading::order_manager::{OrderManager, SharedOrderManager};
use crate::trading::types::{CreateOrderRequest, Order, OrderStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::OnceCell;

pub struct TradingState {
    pub db: SharedOrderDatabase,
    pub manager: SharedOrderManager,
}

static TRADING_STATE: OnceCell<TradingState> = OnceCell::const_new();

pub async fn init_trading(app_handle: &AppHandle) -> Result<(), String> {
    if TRADING_STATE.get().is_some() {
        return Ok(());
    }

    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    let mut db_path = PathBuf::from(app_dir);
    db_path.push("orders.db");

    let db = OrderDatabase::new(db_path)
        .await
        .map_err(|e| format!("Failed to initialize order database: {}", e))?;

    let shared_db = Arc::new(tokio::sync::RwLock::new(db));
    let manager = Arc::new(OrderManager::new(shared_db.clone(), app_handle.clone()));

    TRADING_STATE
        .set(TradingState {
            db: shared_db.clone(),
            manager: manager.clone(),
        })
        .map_err(|_| "Trading state already initialized".to_string())?;

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        OrderManager::start_monitoring(manager).await;
        let _ = handle.emit("order_monitoring_stopped", "Order monitoring exited");
    });

    crate::trading::price_listener::start_price_listener(app_handle.clone()).await;

    Ok(())
}

pub fn require_state<'a>() -> Result<&'a TradingState, String> {
    TRADING_STATE
        .get()
        .ok_or_else(|| "Trading module not initialized".to_string())
}

#[tauri::command]
pub async fn trading_init(handle: AppHandle) -> Result<(), String> {
    init_trading(&handle).await
}

#[tauri::command]
pub async fn create_order(request: CreateOrderRequest) -> Result<Order, String> {
    let state = require_state()?;
    state.manager.create_order(request).await
}

#[tauri::command]
pub async fn cancel_order(order_id: String) -> Result<(), String> {
    let state = require_state()?;
    state.manager.cancel_order(&order_id).await
}

#[tauri::command]
pub async fn get_active_orders(wallet_address: String) -> Result<Vec<Order>, String> {
    let state = require_state()?;
    state.manager.get_active_orders(&wallet_address).await
}

#[tauri::command]
pub async fn get_order_history(
    wallet_address: String,
    limit: Option<i64>,
) -> Result<Vec<Order>, String> {
    let state = require_state()?;
    state
        .manager
        .get_order_history(&wallet_address, limit.unwrap_or(100))
        .await
}

#[tauri::command]
pub async fn get_order(order_id: String) -> Result<Order, String> {
    let state = require_state()?;
    state.manager.get_order(&order_id).await
}

#[tauri::command]
pub async fn acknowledge_order(order_id: String) -> Result<(), String> {
    let state = require_state()?;
    state
        .db
        .write()
        .await
        .update_order_status(&order_id, OrderStatus::Pending, None)
        .await
        .map_err(|e| format!("Failed to acknowledge order: {}", e))
}

pub fn register_trading_state(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = init_trading(&handle).await {
            eprintln!("Failed to initialize trading module: {}", e);
        }
    });
}
