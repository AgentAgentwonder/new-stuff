use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    pub widget_id: String,
    pub widget_type: WidgetType,
    pub data: serde_json::Value,
    pub last_update: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    PriceWatch,
    PortfolioSummary,
    Alerts,
    TopMovers,
    QuickActions,
}

pub struct WidgetManager {
    widgets: Vec<WidgetData>,
}

impl WidgetManager {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub async fn get_widget_data(&self, widget_type: WidgetType) -> Option<WidgetData> {
        match widget_type {
            WidgetType::PriceWatch => Some(self.generate_price_watch_widget()),
            WidgetType::PortfolioSummary => Some(self.generate_portfolio_summary_widget()),
            WidgetType::Alerts => Some(self.generate_alerts_widget()),
            WidgetType::TopMovers => Some(self.generate_top_movers_widget()),
            WidgetType::QuickActions => Some(self.generate_quick_actions_widget()),
        }
    }

    fn generate_price_watch_widget(&self) -> WidgetData {
        let now = Utc::now().timestamp();
        WidgetData {
            widget_id: "price_watch".to_string(),
            widget_type: WidgetType::PriceWatch,
            data: serde_json::json!({
                "assets": [
                    {"symbol": "SOL", "price": 145.67, "change_pct": 5.23},
                    {"symbol": "USDC", "price": 1.00, "change_pct": 0.01},
                    {"symbol": "BONK", "price": 0.000023, "change_pct": -2.45},
                ]
            }),
            last_update: now,
        }
    }

    fn generate_portfolio_summary_widget(&self) -> WidgetData {
        let now = Utc::now().timestamp();
        WidgetData {
            widget_id: "portfolio_summary".to_string(),
            widget_type: WidgetType::PortfolioSummary,
            data: serde_json::json!({
                "total_value": 12345.67,
                "total_change_24h": 234.56,
                "total_change_pct": 1.93,
                "top_asset": "SOL"
            }),
            last_update: now,
        }
    }

    fn generate_alerts_widget(&self) -> WidgetData {
        let now = Utc::now().timestamp();
        WidgetData {
            widget_id: "alerts".to_string(),
            widget_type: WidgetType::Alerts,
            data: serde_json::json!({
                "active_alerts": 3,
                "triggered_today": 1,
                "latest": {
                    "symbol": "SOL",
                    "condition": "above",
                    "value": 150.0
                }
            }),
            last_update: now,
        }
    }

    fn generate_top_movers_widget(&self) -> WidgetData {
        let now = Utc::now().timestamp();
        WidgetData {
            widget_id: "top_movers".to_string(),
            widget_type: WidgetType::TopMovers,
            data: serde_json::json!({
                "gainers": [
                    {"symbol": "SOL", "change_pct": 5.23},
                    {"symbol": "JUP", "change_pct": 3.45},
                ],
                "losers": [
                    {"symbol": "BONK", "change_pct": -2.45},
                ]
            }),
            last_update: now,
        }
    }

    fn generate_quick_actions_widget(&self) -> WidgetData {
        let now = Utc::now().timestamp();
        WidgetData {
            widget_id: "quick_actions".to_string(),
            widget_type: WidgetType::QuickActions,
            data: serde_json::json!({
                "actions": [
                    {"id": "buy_sol", "label": "Buy SOL", "enabled": true},
                    {"id": "sell_sol", "label": "Sell SOL", "enabled": true},
                    {"id": "view_portfolio", "label": "View Portfolio", "enabled": true},
                ]
            }),
            last_update: now,
        }
    }

    pub async fn get_all_widget_data(&self) -> Vec<WidgetData> {
        vec![
            self.generate_price_watch_widget(),
            self.generate_portfolio_summary_widget(),
            self.generate_alerts_widget(),
            self.generate_top_movers_widget(),
            self.generate_quick_actions_widget(),
        ]
    }
}

#[tauri::command]
pub async fn mobile_get_widget_data(
    widget_type: WidgetType,
    widget_manager: tauri::State<'_, Arc<RwLock<WidgetManager>>>,
) -> Result<Option<WidgetData>, String> {
    let manager = widget_manager.read().await;
    Ok(manager.get_widget_data(widget_type).await)
}

#[tauri::command]
pub async fn mobile_get_all_widgets(
    widget_manager: tauri::State<'_, Arc<RwLock<WidgetManager>>>,
) -> Result<Vec<WidgetData>, String> {
    let manager = widget_manager.read().await;
    Ok(manager.get_all_widget_data().await)
}
