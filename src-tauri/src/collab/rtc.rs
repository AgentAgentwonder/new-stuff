use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::RwLock;
use uuid::Uuid;

use crate::collab::types::WebRTCSignal;

#[derive(Clone, Default)]
pub struct RtcSessionManager {
    signals: Arc<RwLock<HashMap<Uuid, Vec<WebRTCSignal>>>>,
}

impl RtcSessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_signal(&self, room_id: Uuid, signal: WebRTCSignal) {
        self.signals
            .write()
            .entry(room_id)
            .or_default()
            .push(signal);
    }

    pub fn take_signals(&self, room_id: &Uuid, target_user_id: &str) -> Vec<WebRTCSignal> {
        let mut signals = self.signals.write();
        if let Some(queue) = signals.get_mut(room_id) {
            let (target_messages, remaining): (Vec<_>, Vec<_>) = queue
                .drain(..)
                .partition(|msg| msg.to_user_id == target_user_id);
            *queue = remaining;
            target_messages
        } else {
            Vec::new()
        }
    }

    pub fn clear_room(&self, room_id: &Uuid) {
        self.signals.write().remove(room_id);
    }
}

pub fn validate_signal(signal: &WebRTCSignal) -> Result<()> {
    if signal.data.trim().is_empty() {
        anyhow::bail!("Signal payload cannot be empty");
    }
    Ok(())
}
