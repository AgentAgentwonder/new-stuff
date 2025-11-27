pub mod auth;
pub mod push;
pub mod sync;
pub mod trades;
pub mod widgets;

pub use auth::*;
pub use push::*;
pub use sync::*;
pub use trades::*;
pub use widgets::*;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileDevice {
    pub device_id: String,
    pub device_name: String,
    pub platform: String, // "ios" or "android"
    pub push_token: Option<String>,
    pub last_sync: Option<i64>,
    pub biometric_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileSession {
    pub session_id: String,
    pub device_id: String,
    pub user_id: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub is_active: bool,
}

pub type SharedMobileAuthManager = Arc<RwLock<auth::MobileAuthManager>>;
pub type SharedPushNotificationManager = Arc<RwLock<push::PushNotificationManager>>;
pub type SharedMobileSyncManager = Arc<RwLock<sync::MobileSyncManager>>;
