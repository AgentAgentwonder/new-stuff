use serde::{Deserialize, Serialize};
use tauri::{Manager, PhysicalPosition, PhysicalSize, Window, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingWindowOptions {
    pub window_id: String,
    pub panel_id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub always_on_top: bool,
    pub transparent: bool,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub async fn get_monitors(app: tauri::AppHandle) -> Result<Vec<MonitorInfo>, String> {
    #[cfg(not(target_os = "linux"))]
    {
        use tauri::Monitor;

        let monitors = app
            .get_webview_window("main")
            .and_then(|w| w.available_monitors().ok())
            .ok_or("Failed to get monitors".to_string())?;

        let mut monitor_infos = Vec::new();

        for (idx, monitor) in monitors.iter().enumerate() {
            let position = monitor.position();
            let size = monitor.size();

            let is_primary = position.x == 0 && position.y == 0;

            monitor_infos.push(MonitorInfo {
                id: format!("monitor-{}", idx),
                name: monitor
                    .name()
                    .cloned()
                    .unwrap_or_else(|| format!("Monitor {}", idx + 1)),
                width: size.width,
                height: size.height,
                x: position.x,
                y: position.y,
                scale_factor: monitor.scale_factor(),
                is_primary,
            });
        }

        Ok(monitor_infos)
    }

    #[cfg(target_os = "linux")]
    {
        Ok(vec![MonitorInfo {
            id: "monitor-0".to_string(),
            name: "Primary Monitor".to_string(),
            width: 1920,
            height: 1080,
            x: 0,
            y: 0,
            scale_factor: 1.0,
            is_primary: true,
        }])
    }
}

#[tauri::command]
pub async fn create_floating_window(
    app: tauri::AppHandle,
    options: FloatingWindowOptions,
) -> Result<String, String> {
    let url = format!("/floating/{}", options.panel_id);

    let window = WebviewWindowBuilder::new(&app, &options.window_id, WebviewUrl::App(url.into()))
        .title(options.title.clone())
        .inner_size(options.width as f64, options.height as f64)
        .position(options.x as f64, options.y as f64)
        .resizable(true)
        .decorations(true)
        .always_on_top(options.always_on_top)
        .transparent(options.transparent)
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;

    Ok(options.window_id)
}

#[tauri::command]
pub async fn close_floating_window(app: tauri::AppHandle, window_id: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        window
            .close()
            .map_err(|e| format!("Failed to close window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_window_position(
    app: tauri::AppHandle,
    window_id: String,
    x: i32,
    y: i32,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        window
            .set_position(PhysicalPosition::new(x, y))
            .map_err(|e| format!("Failed to set position: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_window_size(
    app: tauri::AppHandle,
    window_id: String,
    width: u32,
    height: u32,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        window
            .set_size(PhysicalSize::new(width, height))
            .map_err(|e| format!("Failed to set size: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_window_always_on_top(
    app: tauri::AppHandle,
    window_id: String,
    always_on_top: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        window
            .set_always_on_top(always_on_top)
            .map_err(|e| format!("Failed to set always on top: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_window_position(
    app: tauri::AppHandle,
    window_id: String,
) -> Result<WindowPosition, String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        let position = window
            .outer_position()
            .map_err(|e| format!("Failed to get position: {}", e))?;
        Ok(WindowPosition {
            x: position.x,
            y: position.y,
        })
    } else {
        Err("Window not found".to_string())
    }
}

#[tauri::command]
pub async fn get_window_size(
    app: tauri::AppHandle,
    window_id: String,
) -> Result<WindowSize, String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        let size = window
            .outer_size()
            .map_err(|e| format!("Failed to get size: {}", e))?;
        Ok(WindowSize {
            width: size.width,
            height: size.height,
        })
    } else {
        Err("Window not found".to_string())
    }
}

#[tauri::command]
pub async fn snap_window_to_edge(
    app: tauri::AppHandle,
    window_id: String,
    edge: String,
    monitor_id: Option<String>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        let monitors = get_monitors(app.clone()).await?;

        let target_monitor = if let Some(mid) = monitor_id {
            monitors.iter().find(|m| m.id == mid)
        } else {
            monitors.first()
        };

        if let Some(monitor) = target_monitor {
            let window_size = window
                .outer_size()
                .map_err(|e| format!("Failed to get size: {}", e))?;

            let (x, y) = match edge.as_str() {
                "left" => (
                    monitor.x,
                    monitor.y + (monitor.height as i32 / 2) - (window_size.height as i32 / 2),
                ),
                "right" => (
                    monitor.x + monitor.width as i32 - window_size.width as i32,
                    monitor.y + (monitor.height as i32 / 2) - (window_size.height as i32 / 2),
                ),
                "top" => (
                    monitor.x + (monitor.width as i32 / 2) - (window_size.width as i32 / 2),
                    monitor.y,
                ),
                "bottom" => (
                    monitor.x + (monitor.width as i32 / 2) - (window_size.width as i32 / 2),
                    monitor.y + monitor.height as i32 - window_size.height as i32,
                ),
                "top-left" => (monitor.x, monitor.y),
                "top-right" => (
                    monitor.x + monitor.width as i32 - window_size.width as i32,
                    monitor.y,
                ),
                "bottom-left" => (
                    monitor.x,
                    monitor.y + monitor.height as i32 - window_size.height as i32,
                ),
                "bottom-right" => (
                    monitor.x + monitor.width as i32 - window_size.width as i32,
                    monitor.y + monitor.height as i32 - window_size.height as i32,
                ),
                _ => return Err(format!("Invalid edge: {}", edge)),
            };

            window
                .set_position(PhysicalPosition::new(x, y))
                .map_err(|e| format!("Failed to set position: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn maximize_window(app: tauri::AppHandle, window_id: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        window
            .maximize()
            .map_err(|e| format!("Failed to maximize: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn minimize_window(app: tauri::AppHandle, window_id: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_id) {
        window
            .minimize()
            .map_err(|e| format!("Failed to minimize: {}", e))?;
    }
    Ok(())
}
