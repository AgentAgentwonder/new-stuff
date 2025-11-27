use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuBuilder, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, Manager, WebviewWindow, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrayIconStyle {
    Default,
    Bullish,
    Bearish,
    Minimal,
}

impl Default for TrayIconStyle {
    fn default() -> Self {
        TrayIconStyle::Default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraySettings {
    pub enabled: bool,
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub show_badge: bool,
    pub show_stats: bool,
    pub show_alerts: bool,
    pub show_notifications: bool,
    pub icon_style: TrayIconStyle,
    pub restore_shortcut: Option<String>,
}

impl Default for TraySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            minimize_to_tray: true,
            close_to_tray: true,
            show_badge: true,
            show_stats: true,
            show_alerts: true,
            show_notifications: true,
            icon_style: TrayIconStyle::Default,
            restore_shortcut: Some("CmdOrControl+Shift+M".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayAlertPreview {
    pub id: String,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayStats {
    pub portfolio_value: f64,
    pub pnl_percentage: f64,
    pub pnl_value: f64,
    pub alert_count: u32,
    pub recent_alerts: Vec<TrayAlertPreview>,
}

impl Default for TrayStats {
    fn default() -> Self {
        Self {
            portfolio_value: 0.0,
            pnl_percentage: 0.0,
            pnl_value: 0.0,
            alert_count: 0,
            recent_alerts: Vec::new(),
        }
    }
}

pub struct TrayManager {
    settings: RwLock<TraySettings>,
    stats: RwLock<TrayStats>,
    shortcut: RwLock<Option<String>>,
    settings_path: RwLock<Option<PathBuf>>,
    tray_handle: RwLock<Option<TrayIcon>>,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            settings: RwLock::new(TraySettings::default()),
            stats: RwLock::new(TrayStats::default()),
            shortcut: RwLock::new(None),
            settings_path: RwLock::new(None),
            tray_handle: RwLock::new(None),
        }
    }

    fn create_tray(&self, app_handle: &AppHandle) -> Result<(), String> {
        let menu = self.build_menu(app_handle)?;

        let mut builder = TrayIconBuilder::new()
            .menu(&menu)
            .tooltip("Eclipse Market Pro")
            .on_menu_event(|app, event| {
                Self::handle_menu_event(app, event.id.as_ref());
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.unminimize();
                        let _ = window.set_focus();
                    }
                }
            });

        if let Some(icon) = app_handle.default_window_icon() {
            builder = builder.icon(icon.clone());
        }

        let tray = builder
            .build(app_handle)
            .map_err(|e| format!("Failed to create tray icon: {e}"))?;

        self.tray_handle.write().replace(tray);
        Ok(())
    }

    fn handle_menu_event(app_handle: &AppHandle, menu_id: &str) {
        match menu_id {
            "open" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
            "settings" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    let _ = window.emit("navigate-to-settings", ());
                }
            }
            "quit" => {
                std::process::exit(0);
            }
            "alerts" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                    let _ = window.emit("show-alerts", ());
                }
            }
            _ => {}
        }
    }

    pub fn initialize(&self, app_handle: &AppHandle) {
        match app_handle.path().app_data_dir() {
            Ok(mut data_dir) => {
                if let Err(err) = fs::create_dir_all(&data_dir) {
                    eprintln!("Failed to ensure tray settings directory: {err}");
                } else {
                    data_dir.push("tray_settings.json");
                    let mut settings_guard = self.settings.write();
                    self.settings_path.write().replace(data_dir.clone());

                    if let Ok(contents) = fs::read_to_string(&data_dir) {
                        if let Ok(parsed) = serde_json::from_str::<TraySettings>(&contents) {
                            *settings_guard = parsed;
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to resolve app data directory for tray: {err}");
            }
        }

        if let Err(err) = self.create_tray(app_handle) {
            eprintln!("Failed to create tray icon: {err}");
            return;
        }

        if let Err(err) = self.apply_icon_style(app_handle) {
            eprintln!("Failed to apply tray icon style: {err}");
        }

        if let Err(err) = self.register_shortcut(app_handle) {
            eprintln!("Failed to register tray restore shortcut: {err}");
        }

        if let Err(err) = self.refresh_tray_menu(app_handle) {
            eprintln!("Failed to initialize tray menu: {err}");
        }
    }

    fn save_settings(&self) -> Result<(), String> {
        let path = {
            let path_guard = self.settings_path.read();
            path_guard.clone()
        };

        if let Some(path) = path {
            let settings = self.settings.read().clone();
            let contents = serde_json::to_string_pretty(&settings)
                .map_err(|e| format!("Failed to serialize tray settings: {e}"))?;
            fs::write(&path, contents)
                .map_err(|e| format!("Failed to persist tray settings: {e}"))?;
        }

        Ok(())
    }

    fn register_shortcut(&self, app_handle: &AppHandle) -> Result<(), String> {
        use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

        let mut registered = self.shortcut.write();
        if let Some(existing_str) = registered.clone() {
            // Parse existing shortcut string to unregister it
            let existing_shortcut = if existing_str.contains("CmdOrControl") && existing_str.contains("Shift") && existing_str.contains("M") {
                Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyM)
            } else {
                return Err(format!("Unsupported shortcut format: {}", existing_str));
            };

            if let Err(err) = app_handle.global_shortcut().unregister(existing_shortcut) {
                eprintln!("Failed to unregister previous tray shortcut: {err}");
            }
            registered.take();
        }

        let shortcut_str = self.settings.read().restore_shortcut.clone();
        if let Some(shortcut_str) = shortcut_str {
            // Parse the shortcut string and create a Shortcut
            // For now, we'll handle the common case of CmdOrControl+Shift+M
            let shortcut = if shortcut_str.contains("CmdOrControl") && shortcut_str.contains("Shift") && shortcut_str.contains("M") {
                Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyM)
            } else {
                return Err(format!("Unsupported shortcut format: {}", shortcut_str));
            };

            app_handle.global_shortcut().register(shortcut)
            .map_err(|e| format!("Failed to register tray restore shortcut: {e}"))?;
            registered.replace(shortcut_str);
        }

        Ok(())
    }

    fn icon_label(style: &TrayIconStyle) -> &'static str {
        match style {
            TrayIconStyle::Default => "Eclipse",
            TrayIconStyle::Bullish => "🐂 Eclipse",
            TrayIconStyle::Bearish => "🐻 Eclipse",
            TrayIconStyle::Minimal => "EMP",
        }
    }

    fn apply_icon_style(&self, _app_handle: &AppHandle) -> Result<(), String> {
        let settings = self.settings.read();
        let stats = self.stats.read();

        let mut title = Self::icon_label(&settings.icon_style).to_string();
        if settings.show_badge && stats.alert_count > 0 {
            title = format!("{} ({})", title, stats.alert_count);
        }

        let tray_guard = self.tray_handle.read();
        if let Some(tray) = tray_guard.as_ref() {
            tray.set_title(Some(&title))
                .map_err(|e| format!("Failed to set tray title: {e}"))?;
            tray.set_tooltip(Some(title))
                .map_err(|e| format!("Failed to set tray tooltip: {e}"))?;
        }

        Ok(())
    }

    fn build_menu(&self, app_handle: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
        let settings = self.settings.read().clone();
        let stats = self.stats.read().clone();

        let mut builder = MenuBuilder::new(app_handle);

        let open = MenuItem::with_id(app_handle, "open", "Open", true, None::<&str>)
            .map_err(|e| format!("Failed to create menu item: {e}"))?;
        builder = builder.item(&open);

        if settings.show_stats {
            builder = builder.separator();
            let portfolio = MenuItem::with_id(
                app_handle,
                "portfolio",
                format!("Portfolio: ${:.2}", stats.portfolio_value),
                false,
                None::<&str>,
            )
            .map_err(|e| format!("Failed to create menu item: {e}"))?;
            builder = builder.item(&portfolio);

            let pnl = MenuItem::with_id(
                app_handle,
                "pnl",
                format!(
                    "P&L: ${:.2} ({:.2}%)",
                    stats.pnl_value, stats.pnl_percentage
                ),
                false,
                None::<&str>,
            )
            .map_err(|e| format!("Failed to create menu item: {e}"))?;
            builder = builder.item(&pnl);
        }

        if settings.show_alerts && (!stats.recent_alerts.is_empty() || stats.alert_count > 0) {
            builder = builder.separator();
            if stats.alert_count > 0 {
                let alerts = MenuItem::with_id(
                    app_handle,
                    "alerts",
                    format!("🔔 {} Alerts", stats.alert_count),
                    true,
                    None::<&str>,
                )
                .map_err(|e| format!("Failed to create menu item: {e}"))?;
                builder = builder.item(&alerts);
            }

            let limit = min(3, stats.recent_alerts.len());
            for preview in stats.recent_alerts.iter().take(limit) {
                let preview_item = MenuItem::with_id(
                    app_handle,
                    format!("alert-preview-{}", preview.id),
                    format!("{} — {}", preview.title, preview.summary),
                    false,
                    None::<&str>,
                )
                .map_err(|e| format!("Failed to create menu item: {e}"))?;
                builder = builder.item(&preview_item);
            }
        }

        builder = builder.separator();
        let settings_item =
            MenuItem::with_id(app_handle, "settings", "Settings", true, None::<&str>)
                .map_err(|e| format!("Failed to create menu item: {e}"))?;
        builder = builder.item(&settings_item);

        builder = builder.separator();
        let quit = MenuItem::with_id(app_handle, "quit", "Exit", true, None::<&str>)
            .map_err(|e| format!("Failed to create menu item: {e}"))?;
        builder = builder.item(&quit);

        builder
            .build()
            .map_err(|e| format!("Failed to build menu: {e}"))
    }

    pub fn refresh_tray_menu(&self, app_handle: &AppHandle) -> Result<(), String> {
        let menu = self.build_menu(app_handle)?;

        let tray_guard = self.tray_handle.read();
        if let Some(tray) = tray_guard.as_ref() {
            tray.set_menu(Some(menu))
                .map_err(|e| format!("Failed to update tray menu: {e}"))?;
        }

        Ok(())
    }

    pub fn get_settings(&self) -> TraySettings {
        self.settings.read().clone()
    }

    pub fn update_settings(
        &self,
        app_handle: &AppHandle,
        new_settings: TraySettings,
    ) -> Result<(), String> {
        {
            let mut settings = self.settings.write();
            *settings = new_settings;
        }

        self.save_settings()?;
        self.apply_icon_style(app_handle)?;
        self.register_shortcut(app_handle)?;
        self.refresh_tray_menu(app_handle)?;
        Ok(())
    }

    pub fn update_stats(&self, app_handle: &AppHandle, new_stats: TrayStats) -> Result<(), String> {
        {
            let mut stats = self.stats.write();
            *stats = new_stats;
        }

        self.apply_icon_style(app_handle)?;
        self.refresh_tray_menu(app_handle)?;
        Ok(())
    }

    pub fn update_badge(&self, app_handle: &AppHandle, count: u32) -> Result<(), String> {
        {
            let mut stats = self.stats.write();
            stats.alert_count = count;
        }

        self.apply_icon_style(app_handle)?;
        self.refresh_tray_menu(app_handle)?;
        Ok(())
    }

    pub fn should_minimize_to_tray(&self) -> bool {
        let settings = self.settings.read();
        settings.enabled && settings.minimize_to_tray
    }

    pub fn should_close_to_tray(&self) -> bool {
        let settings = self.settings.read();
        settings.enabled && settings.close_to_tray
    }

    pub fn should_show_notifications(&self) -> bool {
        self.settings.read().show_notifications
    }

    pub fn notify_minimized(&self, app_handle: &AppHandle) {
        if !self.should_show_notifications() {
            return;
        }

        if let Err(err) = app_handle.notification()
            .builder()
            .title("Eclipse Market Pro")
            .body("Application minimized to system tray. Press CmdOrControl+Shift+M to restore.")
            .show()
        {
            eprintln!("Failed to show tray minimize notification: {err}");
        }
    }
}

pub type SharedTrayManager = Arc<TrayManager>;

pub fn attach_window_listeners(window: &tauri::WebviewWindow, tray_manager: SharedTrayManager) {
    let app_handle = window.app_handle();
    let handle_clone = app_handle.clone();
    let tray_manager_clone = tray_manager.clone();

    window.on_window_event(move |event| match event {
        WindowEvent::CloseRequested { api, .. } => {
            if tray_manager_clone.should_close_to_tray() {
                api.prevent_close();
                if let Some(window) = handle_clone.get_webview_window("main") {
                    let _ = window.hide();
                }
                tray_manager_clone.notify_minimized(&handle_clone);
            }
        }
        WindowEvent::Destroyed => {
            if let Some(shortcut_str) = tray_manager_clone.shortcut.read().clone() {
                use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

                // Parse shortcut string to unregister it
                let shortcut = if shortcut_str.contains("CmdOrControl") && shortcut_str.contains("Shift") && shortcut_str.contains("M") {
                    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyM)
                } else {
                    eprintln!("Failed to unregister tray shortcut on destroy: unsupported format");
                    return;
                };

                if let Err(err) = handle_clone.global_shortcut().unregister(shortcut) {
                    eprintln!("Failed to unregister tray shortcut on destroy: {err}");
                }
            }
        }
        _ => {}
    });
}

// Tauri commands

#[tauri::command]
pub fn get_tray_settings(
    tray_manager: tauri::State<'_, SharedTrayManager>,
) -> Result<TraySettings, String> {
    Ok(tray_manager.get_settings())
}

#[tauri::command]
pub fn update_tray_settings(
    settings: TraySettings,
    tray_manager: tauri::State<'_, SharedTrayManager>,
    app: AppHandle,
) -> Result<(), String> {
    tray_manager.update_settings(&app, settings)
}

#[tauri::command]
pub fn update_tray_stats(
    stats: TrayStats,
    tray_manager: tauri::State<'_, SharedTrayManager>,
    app: AppHandle,
) -> Result<(), String> {
    tray_manager.update_stats(&app, stats)
}

#[tauri::command]
pub fn update_tray_badge(
    count: u32,
    tray_manager: tauri::State<'_, SharedTrayManager>,
    app: AppHandle,
) -> Result<(), String> {
    tray_manager.update_badge(&app, count)
}

#[tauri::command]
pub fn minimize_to_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {e}"))?;
    }
    if let Some(tray_manager) = app.try_state::<SharedTrayManager>() {
        tray_manager.notify_minimized(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn restore_from_tray(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .show()
            .map_err(|e| format!("Failed to show window: {e}"))?;
        window
            .unminimize()
            .map_err(|e| format!("Failed to unminimize window: {e}"))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus window: {e}"))?;
    }
    Ok(())
}
