use crate::mobile::{MobileDevice, SharedMobileAuthManager};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotification {
    pub notification_id: String,
    pub device_id: String,
    pub category: NotificationCategory,
    pub title: String,
    pub body: String,
    pub payload: serde_json::Value,
    pub created_at: i64,
    pub delivered_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    Alert,
    QuickTrade,
    Portfolio,
    Watch,
    System,
}

pub struct PushNotificationManager {
    pub notifications: HashMap<String, PushNotification>,
    pub queue: VecDeque<String>,
    max_queue_size: usize,
}

impl PushNotificationManager {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            notifications: HashMap::new(),
            queue: VecDeque::new(),
            max_queue_size,
        }
    }

    pub fn create_notification(
        &mut self,
        device_id: String,
        category: NotificationCategory,
        title: String,
        body: String,
        payload: serde_json::Value,
    ) -> PushNotification {
        let notification = PushNotification {
            notification_id: Uuid::new_v4().to_string(),
            device_id,
            category,
            title,
            body,
            payload,
            created_at: Utc::now().timestamp(),
            delivered_at: None,
        };

        self.queue_notification(notification.clone());

        notification
    }

    fn queue_notification(&mut self, notification: PushNotification) {
        if self.queue.len() >= self.max_queue_size {
            if let Some(oldest) = self.queue.pop_front() {
                self.notifications.remove(&oldest);
            }
        }

        self.queue.push_back(notification.notification_id.clone());
        self.notifications
            .insert(notification.notification_id.clone(), notification);
    }

    pub fn mark_delivered(&mut self, notification_id: &str) {
        if let Some(notification) = self.notifications.get_mut(notification_id) {
            notification.delivered_at = Some(Utc::now().timestamp());
        }
    }

    pub fn get_pending_notifications(&self, device_id: &str) -> Vec<PushNotification> {
        self.queue
            .iter()
            .filter_map(|id| self.notifications.get(id))
            .filter(|notification| {
                notification.device_id == device_id && notification.delivered_at.is_none()
            })
            .cloned()
            .collect()
    }

    pub fn dequeue_next(&mut self, device_id: &str) -> Option<PushNotification> {
        let position = self.queue.iter().position(|id| {
            self.notifications
                .get(id)
                .map(|n| n.device_id.as_str() == device_id)
                .unwrap_or(false)
        });

        if let Some(pos) = position {
            if let Some(id) = self.queue.remove(pos) {
                return self.notifications.remove(&id);
            }
        }

        None
    }
}

#[tauri::command]
pub async fn mobile_queue_notification(
    device_id: String,
    category: NotificationCategory,
    title: String,
    body: String,
    payload: serde_json::Value,
    push_manager: tauri::State<'_, Arc<RwLock<PushNotificationManager>>>,
    mobile_auth: tauri::State<'_, Arc<RwLock<crate::mobile::auth::MobileAuthManager>>>,
) -> Result<PushNotification, String> {
    let devices = {
        let auth = mobile_auth.read().await;
        auth.get_devices()
    };

    let device_registered = devices.iter().any(|device| device.device_id == device_id);
    if !device_registered {
        return Err("Device not registered".into());
    }

    let mut manager = push_manager.write().await;
    Ok(manager.create_notification(device_id, category, title, body, payload))
}

#[tauri::command]
pub async fn mobile_get_pending_notifications(
    device_id: String,
    push_manager: tauri::State<'_, Arc<RwLock<PushNotificationManager>>>,
    mobile_auth: tauri::State<'_, Arc<RwLock<crate::mobile::auth::MobileAuthManager>>>,
) -> Result<Vec<PushNotification>, String> {
    let devices = {
        let auth = mobile_auth.read().await;
        auth.get_devices()
    };

    let device_registered = devices.iter().any(|device| device.device_id == device_id);
    if !device_registered {
        return Err("Device not registered".into());
    }

    let manager = push_manager.read().await;
    Ok(manager.get_pending_notifications(&device_id))
}

#[tauri::command]
pub async fn mobile_dequeue_notification(
    device_id: String,
    push_manager: tauri::State<'_, Arc<RwLock<PushNotificationManager>>>,
    mobile_auth: tauri::State<'_, Arc<RwLock<crate::mobile::auth::MobileAuthManager>>>,
) -> Result<Option<PushNotification>, String> {
    let devices = {
        let auth = mobile_auth.read().await;
        auth.get_devices()
    };

    let device_registered = devices.iter().any(|device| device.device_id == device_id);
    if !device_registered {
        return Err("Device not registered".into());
    }

    let mut manager = push_manager.write().await;
    Ok(manager.dequeue_next(&device_id))
}
