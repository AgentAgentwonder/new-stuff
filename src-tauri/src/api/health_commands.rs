use super::health_monitor::{ApiHealthDashboard, ApiHealthMetrics, SharedApiHealthMonitor};
use tauri::State;

#[tauri::command]
pub async fn get_api_health_dashboard(
    monitor: State<'_, SharedApiHealthMonitor>,
) -> Result<ApiHealthDashboard, String> {
    let mon = monitor.read().await;
    mon.get_dashboard().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_service_health_metrics(
    monitor: State<'_, SharedApiHealthMonitor>,
    service_name: String,
) -> Result<ApiHealthMetrics, String> {
    let mon = monitor.read().await;
    mon.get_metrics(&service_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_health_records(
    monitor: State<'_, SharedApiHealthMonitor>,
    days: Option<i64>,
) -> Result<usize, String> {
    let mon = monitor.read().await;
    mon.cleanup_old_records(days.unwrap_or(30))
        .await
        .map_err(|e| e.to_string())
}
