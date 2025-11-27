use crate::data::event_store::{Event as AuditEvent, SharedEventStore};
use crate::trading::database::{OrderDatabase, SharedOrderDatabase};
use crate::trading::types::{
    CreateOrderRequest, Order, OrderFill, OrderSide, OrderStatus, OrderType, OrderUpdate,
    QuickTradeRequest,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub symbol: String,
    pub price: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderTriggeredEvent {
    pub order_id: String,
    pub order_type: OrderType,
    pub symbol: String,
    pub side: OrderSide,
    pub trigger_price: f64,
    pub amount: f64,
}

pub struct OrderManager {
    db: SharedOrderDatabase,
    app_handle: AppHandle,
    current_prices: Arc<RwLock<HashMap<String, f64>>>,
    event_store: Option<SharedEventStore>,
}

impl OrderManager {
    pub fn new(db: SharedOrderDatabase, app_handle: AppHandle) -> Self {
        let event_store = app_handle
            .try_state::<SharedEventStore>()
            .map(|state| state.inner().clone());

        Self {
            db,
            app_handle,
            current_prices: Arc::new(RwLock::new(HashMap::new())),
            event_store,
        }
    }

    pub async fn create_order(&self, request: CreateOrderRequest) -> Result<Order, String> {
        let order = Order {
            id: Uuid::new_v4().to_string(),
            order_type: request.order_type,
            side: request.side,
            status: OrderStatus::Pending,
            input_mint: request.input_mint,
            output_mint: request.output_mint,
            input_symbol: request.input_symbol,
            output_symbol: request.output_symbol,
            amount: request.amount,
            filled_amount: 0.0,
            limit_price: request.limit_price,
            stop_price: request.stop_price,
            trailing_percent: request.trailing_percent,
            highest_price: None,
            lowest_price: None,
            linked_order_id: request.linked_order_id,
            slippage_bps: request.slippage_bps,
            priority_fee_micro_lamports: request.priority_fee_micro_lamports,
            wallet_address: request.wallet_address,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            triggered_at: None,
            tx_signature: None,
            error_message: None,
        };

        self.db
            .write()
            .await
            .create_order(&order)
            .await
            .map_err(|e| format!("Failed to create order: {}", e))?;

        // Publish event to event store
        if let Some(ref event_store) = self.event_store {
            let symbol = if order.side == OrderSide::Buy {
                &order.output_symbol
            } else {
                &order.input_symbol
            };

            let event = AuditEvent::OrderPlaced {
                order_id: order.id.clone(),
                symbol: symbol.clone(),
                side: order.side.to_string(),
                quantity: order.amount,
                price: order.limit_price.or(order.stop_price),
                timestamp: Utc::now(),
            };

            let aggregate_id = format!("order_{}", order.id);
            let _ = event_store
                .read()
                .await
                .publish_event(event, &aggregate_id)
                .await;
        }

        self.emit_order_update(&order);

        Ok(order)
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<(), String> {
        let order = self.get_order(order_id).await?;

        if order.status != OrderStatus::Pending && order.status != OrderStatus::PartiallyFilled {
            return Err("Order cannot be cancelled".to_string());
        }

        self.db
            .write()
            .await
            .cancel_order(order_id)
            .await
            .map_err(|e| format!("Failed to cancel order: {}", e))?;

        if let Some(linked_id) = &order.linked_order_id {
            let _ = self.db.write().await.cancel_linked_orders(linked_id).await;
        }

        // Publish event to event store
        if let Some(ref event_store) = self.event_store {
            let event = AuditEvent::OrderCancelled {
                order_id: order_id.to_string(),
                reason: "Manual cancellation".to_string(),
                timestamp: Utc::now(),
            };

            let aggregate_id = format!("order_{}", order_id);
            let _ = event_store
                .read()
                .await
                .publish_event(event, &aggregate_id)
                .await;
        }

        let mut cancelled_order = order;
        cancelled_order.status = OrderStatus::Cancelled;
        cancelled_order.updated_at = Utc::now();

        self.emit_order_update(&cancelled_order);

        Ok(())
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order, String> {
        self.db
            .read()
            .await
            .get_order(order_id)
            .await
            .map_err(|e| format!("Failed to get order: {}", e))?
            .ok_or_else(|| "Order not found".to_string())
    }

    pub async fn get_active_orders(&self, wallet_address: &str) -> Result<Vec<Order>, String> {
        self.db
            .read()
            .await
            .get_active_orders(wallet_address)
            .await
            .map_err(|e| format!("Failed to get active orders: {}", e))
    }

    pub async fn get_order_history(
        &self,
        wallet_address: &str,
        limit: i64,
    ) -> Result<Vec<Order>, String> {
        self.db
            .read()
            .await
            .get_order_history(wallet_address, limit)
            .await
            .map_err(|e| format!("Failed to get order history: {}", e))
    }

    pub async fn update_price(&self, symbol: &str, price: f64) {
        let mut prices = self.current_prices.write().await;
        prices.insert(symbol.to_string(), price);
    }

    pub async fn check_and_trigger_orders(&self) -> Result<(), String> {
        let orders = self
            .db
            .read()
            .await
            .get_all_active_orders()
            .await
            .map_err(|e| format!("Failed to get active orders: {}", e))?;

        let prices = self.current_prices.read().await.clone();

        for order in orders {
            let symbol = if order.side == OrderSide::Buy {
                &order.output_symbol
            } else {
                &order.input_symbol
            };

            if let Some(&current_price) = prices.get(symbol) {
                if self.should_trigger_order(&order, current_price).await? {
                    if let Err(e) = self.execute_order(&order, current_price).await {
                        eprintln!("Failed to execute order {}: {}", order.id, e);
                        let _ = self.db.write().await.update_order_status(
                            &order.id,
                            OrderStatus::Failed,
                            Some(e),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    async fn should_trigger_order(
        &self,
        order: &Order,
        current_price: f64,
    ) -> Result<bool, String> {
        match order.order_type {
            OrderType::Limit => {
                if let Some(limit_price) = order.limit_price {
                    return Ok(match order.side {
                        OrderSide::Buy => current_price <= limit_price,
                        OrderSide::Sell => current_price >= limit_price,
                    });
                }
            }
            OrderType::StopLoss => {
                if let Some(stop_price) = order.stop_price {
                    return Ok(match order.side {
                        OrderSide::Buy => current_price >= stop_price,
                        OrderSide::Sell => current_price <= stop_price,
                    });
                }
            }
            OrderType::TakeProfit => {
                if let Some(take_profit_price) = order.limit_price {
                    return Ok(match order.side {
                        OrderSide::Buy => current_price <= take_profit_price,
                        OrderSide::Sell => current_price >= take_profit_price,
                    });
                }
            }
            OrderType::TrailingStop => {
                return self.check_trailing_stop(order, current_price).await;
            }
            OrderType::Market => return Ok(true),
        }

        Ok(false)
    }

    async fn check_trailing_stop(&self, order: &Order, current_price: f64) -> Result<bool, String> {
        let trailing_percent = order.trailing_percent.ok_or("Trailing percent not set")?;

        let mut should_trigger = false;
        let mut new_highest = order.highest_price;
        let mut new_lowest = order.lowest_price;
        let mut new_stop_price = order.stop_price;

        match order.side {
            OrderSide::Sell => {
                let highest = new_highest.unwrap_or(current_price);
                if current_price > highest {
                    new_highest = Some(current_price);
                    new_stop_price = Some(current_price * (1.0 - trailing_percent / 100.0));
                } else if let Some(stop_price) = new_stop_price {
                    should_trigger = current_price <= stop_price;
                }
            }
            OrderSide::Buy => {
                let lowest = new_lowest.unwrap_or(current_price);
                if current_price < lowest {
                    new_lowest = Some(current_price);
                    new_stop_price = Some(current_price * (1.0 + trailing_percent / 100.0));
                } else if let Some(stop_price) = new_stop_price {
                    should_trigger = current_price >= stop_price;
                }
            }
        }

        if new_highest != order.highest_price
            || new_lowest != order.lowest_price
            || new_stop_price != order.stop_price
        {
            self.db
                .write()
                .await
                .update_trailing_stop(&order.id, new_highest, new_lowest, new_stop_price)
                .await
                .map_err(|e| format!("Failed to update trailing stop: {}", e))?;
        }

        Ok(should_trigger)
    }

    async fn execute_order(&self, order: &Order, trigger_price: f64) -> Result<(), String> {
        self.emit_order_triggered(order, trigger_price);

        let tx_signature = format!("simulated_{}", Uuid::new_v4());

        self.db
            .write()
            .await
            .update_order_fill(
                &order.id,
                order.amount,
                OrderStatus::Filled,
                Some(tx_signature.clone()),
            )
            .await
            .map_err(|e| format!("Failed to update order: {}", e))?;

        if let Some(linked_id) = &order.linked_order_id {
            let _ = self.db.write().await.cancel_linked_orders(linked_id).await;
        }

        let mut filled_order = order.clone();
        filled_order.status = OrderStatus::Filled;
        filled_order.filled_amount = order.amount;
        filled_order.tx_signature = Some(tx_signature);
        filled_order.triggered_at = Some(Utc::now());
        filled_order.updated_at = Utc::now();

        // Publish order filled event
        if let Some(ref event_store) = self.event_store {
            let event = AuditEvent::OrderFilled {
                order_id: order.id.clone(),
                fill_price: trigger_price,
                filled_quantity: order.amount,
                timestamp: Utc::now(),
            };
            let aggregate_id = format!("order_{}", order.id);
            let _ = event_store
                .read()
                .await
                .publish_event(event, &aggregate_id)
                .await;
        }

        self.emit_order_update(&filled_order);

        Ok(())
    }

    async fn publish_audit_event(&self, aggregate_id: String, event: AuditEvent) {
        if let Some(store) = &self.event_store {
            let store = store.clone();
            let result = {
                let guard = store.read().await;
                guard.publish_event(event, &aggregate_id).await
            };

            if let Err(err) = result {
                eprintln!("Failed to publish audit event {}: {}", aggregate_id, err);
            }
        }
    }

    fn emit_order_update(&self, order: &Order) {
        let _ = self.app_handle.emit("order_update", order);
    }

    fn emit_order_triggered(&self, order: &Order, trigger_price: f64) {
        let event = OrderTriggeredEvent {
            order_id: order.id.clone(),
            order_type: order.order_type,
            symbol: if order.side == OrderSide::Buy {
                order.output_symbol.clone()
            } else {
                order.input_symbol.clone()
            },
            side: order.side,
            trigger_price,
            amount: order.amount,
        };

        let _ = self.app_handle.emit("order_triggered", event);
    }

    pub async fn start_monitoring(manager: Arc<Self>) {
        let mut ticker = interval(Duration::from_millis(500));

        loop {
            ticker.tick().await;
            if let Err(e) = manager.check_and_trigger_orders().await {
                eprintln!("Error checking orders: {}", e);
            }
        }
    }
}

pub type SharedOrderManager = Arc<OrderManager>;
