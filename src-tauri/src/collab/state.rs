use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use tauri::State;
use uuid::Uuid;

use crate::collab::crypto::RoomEncryption;
use crate::collab::moderation::ModerationManager;
use crate::collab::room::RoomManager;
use crate::collab::rtc::RtcSessionManager;
use crate::collab::types::{CollabMessage, RoomState};
use crate::collab::websocket::CollabWebSocketManager;

#[derive(Clone)]
pub struct CollabState {
    pub rooms: Arc<RoomManager>,
    pub rtc: Arc<RtcSessionManager>,
    pub websocket: Arc<CollabWebSocketManager>,
    pub moderation: Arc<ModerationManager>,
    encryption_keys: Arc<RwLock<HashMap<Uuid, [u8; 32]>>>,
}

impl CollabState {
    pub fn new(websocket: CollabWebSocketManager) -> Self {
        Self {
            rooms: Arc::new(RoomManager::new()),
            rtc: Arc::new(RtcSessionManager::new()),
            websocket: Arc::new(websocket),
            moderation: Arc::new(ModerationManager::new()),
            encryption_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_encryption_key(&self, room_id: Uuid, key: [u8; 32]) {
        self.encryption_keys.write().insert(room_id, key);
    }

    pub fn get_encryption(&self, room_id: &Uuid) -> Option<RoomEncryption> {
        self.encryption_keys
            .read()
            .get(room_id)
            .and_then(|key| RoomEncryption::new(key).ok())
    }

    pub fn broadcast_state(&self, room_id: Uuid) -> Result<()> {
        let state = self.rooms.get_room_state(&room_id)?;
        self.websocket
            .broadcast(room_id, CollabMessage::StateSync { state })
    }

    pub fn get_room_state(&self, room_id: &Uuid) -> Result<RoomState> {
        self.rooms.get_room_state(room_id)
    }
}

pub type CollabStateHandle<'a> = State<'a, CollabState>;
