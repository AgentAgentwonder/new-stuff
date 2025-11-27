use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::RwLock;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::collab::types::CollabMessage;

#[derive(Clone)]
pub struct CollabWebSocketManager {
    app_handle: Option<AppHandle>,
    subscriptions: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
    broadcast_tx: Arc<broadcast::Sender<(Uuid, CollabMessage)>>,
}

impl CollabWebSocketManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let (tx, _) = broadcast::channel(1024);

        Self {
            app_handle: Some(app_handle),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx: Arc::new(tx),
        }
    }

    pub fn without_handle() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            app_handle: None,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx: Arc::new(tx),
        }
    }

    pub fn subscribe_to_room(&self, room_id: Uuid, user_id: String) {
        self.subscriptions
            .write()
            .entry(room_id)
            .or_default()
            .push(user_id);
    }

    pub fn unsubscribe_from_room(&self, room_id: &Uuid, user_id: &str) {
        if let Some(subscribers) = self.subscriptions.write().get_mut(room_id) {
            subscribers.retain(|id| id != user_id);
        }
    }

    pub fn broadcast(&self, room_id: Uuid, message: CollabMessage) -> Result<()> {
        let _ = self.broadcast_tx.send((room_id, message.clone()));

        if let Some(app_handle) = &self.app_handle {
            let event_name = format!("collab:room:{}", room_id);
            app_handle
                .emit(&event_name, &message)
                .context("Failed to emit collab message")?;
        }

        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<(Uuid, CollabMessage)> {
        self.broadcast_tx.subscribe()
    }

    pub fn get_room_subscribers(&self, room_id: &Uuid) -> Vec<String> {
        self.subscriptions
            .read()
            .get(room_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn clean_room(&self, room_id: &Uuid) {
        self.subscriptions.write().remove(room_id);
    }
}
