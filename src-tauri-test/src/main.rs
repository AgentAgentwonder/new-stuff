// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Minimal test backend
use tauri::Manager;

#[tauri::command]
async fn test_command() -> Result<String, String> {
    Ok("Test command works!".to_string())
}

#[tauri::command]
async fn biometric_get_status() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "available": false,
        "enrolled": false,
        "fallbackConfigured": false,
        "platform": "PasswordOnly"
    }))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            test_command,
            biometric_get_status
        ])
        .setup(|app| {
            println!("[Test Backend] Setup completed successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}