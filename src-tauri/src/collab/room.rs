use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::collab::crypto::{hash_password, verify_password};
use crate::collab::permissions::default_permissions_for_role;
use crate::collab::types::{
    ChatMessage, Competition, CreateRoomRequest, JoinRoomRequest, Participant, ParticipantRole,
    ParticipantStatus, Room, RoomState, SendMessageRequest, SharedOrder, SharedWatchlist,
};

#[derive(Clone)]
pub struct RoomManager {
    rooms: Arc<RwLock<HashMap<Uuid, Room>>>,
    participants: Arc<RwLock<HashMap<Uuid, Vec<Participant>>>>,
    chat_messages: Arc<RwLock<HashMap<Uuid, Vec<ChatMessage>>>>,
    watchlists: Arc<RwLock<HashMap<Uuid, Vec<SharedWatchlist>>>>,
    orders: Arc<RwLock<HashMap<Uuid, Vec<SharedOrder>>>>,
    competitions: Arc<RwLock<HashMap<Uuid, Competition>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            participants: Arc::new(RwLock::new(HashMap::new())),
            chat_messages: Arc::new(RwLock::new(HashMap::new())),
            watchlists: Arc::new(RwLock::new(HashMap::new())),
            orders: Arc::new(RwLock::new(HashMap::new())),
            competitions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_room(&self, request: CreateRoomRequest, owner_id: String) -> Result<Room> {
        let room_id = Uuid::new_v4();
        let password_hash = if let Some(password) = request.password {
            Some(hash_password(&password)?)
        } else {
            None
        };

        let room = Room {
            id: room_id,
            name: request.name,
            description: request.description,
            owner_id: owner_id.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            max_participants: request.max_participants,
            is_public: request.is_public,
            password_hash,
            encryption_enabled: true,
            voice_enabled: request.settings.allow_voice_chat,
            video_enabled: request.settings.allow_video_chat,
            screen_share_enabled: request.settings.allow_screen_share,
            settings: request.settings,
        };

        self.rooms.write().insert(room_id, room.clone());
        self.participants.write().insert(room_id, Vec::new());
        self.chat_messages.write().insert(room_id, Vec::new());
        self.watchlists.write().insert(room_id, Vec::new());
        self.orders.write().insert(room_id, Vec::new());

        Ok(room)
    }

    pub fn get_room(&self, room_id: &Uuid) -> Result<Room> {
        self.rooms
            .read()
            .get(room_id)
            .cloned()
            .ok_or_else(|| anyhow!("Room not found"))
    }

    pub fn list_rooms(&self, include_private: bool) -> Vec<Room> {
        self.rooms
            .read()
            .values()
            .filter(|r| include_private || r.is_public)
            .cloned()
            .collect()
    }

    pub fn delete_room(&self, room_id: &Uuid, user_id: &str) -> Result<()> {
        let room = self.get_room(room_id)?;

        if room.owner_id != user_id {
            return Err(anyhow!("Only room owner can delete the room"));
        }

        self.rooms.write().remove(room_id);
        self.participants.write().remove(room_id);
        self.chat_messages.write().remove(room_id);
        self.watchlists.write().remove(room_id);
        self.orders.write().remove(room_id);
        self.competitions.write().remove(room_id);

        Ok(())
    }

    pub fn join_room(&self, request: JoinRoomRequest, user_id: String) -> Result<Participant> {
        let room = self.get_room(&request.room_id)?;

        if let Some(password_hash) = &room.password_hash {
            let password = request
                .password
                .ok_or_else(|| anyhow!("Password required"))?;
            if !verify_password(&password, password_hash)? {
                return Err(anyhow!("Invalid password"));
            }
        }

        let participants = self.participants.read();
        let current_participants = participants
            .get(&request.room_id)
            .map(|p| p.len())
            .unwrap_or(0);

        if current_participants >= room.max_participants {
            return Err(anyhow!("Room is full"));
        }

        drop(participants);

        let role = if user_id == room.owner_id {
            ParticipantRole::Owner
        } else if room.settings.allow_guest_join {
            ParticipantRole::Guest
        } else {
            ParticipantRole::Member
        };

        let participant = Participant {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            username: request.username,
            room_id: request.room_id,
            joined_at: Utc::now(),
            last_active: Utc::now(),
            role,
            permissions: default_permissions_for_role(role),
            status: ParticipantStatus::Active,
            is_muted: false,
            is_video_off: false,
            is_screen_sharing: false,
        };

        self.participants
            .write()
            .entry(request.room_id)
            .or_default()
            .push(participant.clone());

        Ok(participant)
    }

    pub fn leave_room(&self, room_id: &Uuid, user_id: &str) -> Result<()> {
        let mut participants = self.participants.write();
        if let Some(room_participants) = participants.get_mut(room_id) {
            room_participants.retain(|p| p.user_id != user_id);
            Ok(())
        } else {
            Err(anyhow!("Room not found"))
        }
    }

    pub fn get_participants(&self, room_id: &Uuid) -> Vec<Participant> {
        self.participants
            .read()
            .get(room_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_participant(&self, room_id: &Uuid, user_id: &str) -> Result<Participant> {
        self.participants
            .read()
            .get(room_id)
            .and_then(|participants| participants.iter().find(|p| p.user_id == user_id).cloned())
            .ok_or_else(|| anyhow!("Participant not found"))
    }

    pub fn update_participant(&self, participant: Participant) -> Result<()> {
        let mut participants = self.participants.write();
        if let Some(room_participants) = participants.get_mut(&participant.room_id) {
            if let Some(p) = room_participants
                .iter_mut()
                .find(|p| p.id == participant.id)
            {
                *p = participant;
                Ok(())
            } else {
                Err(anyhow!("Participant not found"))
            }
        } else {
            Err(anyhow!("Room not found"))
        }
    }

    pub fn send_message(
        &self,
        request: SendMessageRequest,
        user_id: String,
        username: String,
    ) -> Result<ChatMessage> {
        let room = self.get_room(&request.room_id)?;
        let participant = self.get_participant(&request.room_id, &user_id)?;

        if !participant.permissions.can_chat {
            return Err(anyhow!("User does not have chat permissions"));
        }

        let message = ChatMessage {
            id: Uuid::new_v4(),
            room_id: request.room_id,
            user_id,
            username,
            content: request.content,
            timestamp: Utc::now(),
            encrypted: room.encryption_enabled,
            mentions: Vec::new(),
            replied_to: request.replied_to,
        };

        self.chat_messages
            .write()
            .entry(request.room_id)
            .or_default()
            .push(message.clone());

        Ok(message)
    }

    pub fn get_messages(&self, room_id: &Uuid, limit: Option<usize>) -> Vec<ChatMessage> {
        let messages = self.chat_messages.read();
        let room_messages = messages.get(room_id).cloned().unwrap_or_default();

        if let Some(limit) = limit {
            room_messages.into_iter().rev().take(limit).rev().collect()
        } else {
            room_messages
        }
    }

    pub fn add_watchlist(&self, watchlist: SharedWatchlist) -> Result<()> {
        self.watchlists
            .write()
            .entry(watchlist.room_id)
            .or_default()
            .push(watchlist);
        Ok(())
    }

    pub fn get_watchlists(&self, room_id: &Uuid) -> Vec<SharedWatchlist> {
        self.watchlists
            .read()
            .get(room_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn add_order(&self, order: SharedOrder) -> Result<()> {
        self.orders
            .write()
            .entry(order.room_id)
            .or_default()
            .push(order);
        Ok(())
    }

    pub fn get_orders(&self, room_id: &Uuid) -> Vec<SharedOrder> {
        self.orders.read().get(room_id).cloned().unwrap_or_default()
    }

    pub fn update_order(&self, order: SharedOrder) -> Result<()> {
        let mut orders = self.orders.write();
        if let Some(room_orders) = orders.get_mut(&order.room_id) {
            if let Some(o) = room_orders.iter_mut().find(|o| o.id == order.id) {
                *o = order;
                Ok(())
            } else {
                Err(anyhow!("Order not found"))
            }
        } else {
            Err(anyhow!("Room not found"))
        }
    }

    pub fn set_competition(&self, competition: Competition) -> Result<()> {
        self.competitions
            .write()
            .insert(competition.room_id, competition);
        Ok(())
    }

    pub fn get_competition(&self, room_id: &Uuid) -> Option<Competition> {
        self.competitions.read().get(room_id).cloned()
    }

    pub fn get_room_state(&self, room_id: &Uuid) -> Result<RoomState> {
        let room = self.get_room(room_id)?;
        let participants = self.get_participants(room_id);
        let watchlists = self.get_watchlists(room_id);
        let active_orders = self.get_orders(room_id);
        let competition = self.get_competition(room_id);

        Ok(RoomState {
            room,
            participants,
            watchlists,
            active_orders,
            competition,
        })
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}
