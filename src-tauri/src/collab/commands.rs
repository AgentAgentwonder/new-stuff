use anyhow::{Context, Result};
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::collab::crypto::RoomEncryption;
use crate::collab::moderation::ModerationManager;
use crate::collab::permissions::{can_modify_permissions, default_permissions_for_role};
use crate::collab::state::CollabState;
use crate::collab::types::*;

#[tauri::command]
pub async fn collab_create_room(
    request: CreateRoomRequest,
    user_id: String,
    state: State<'_, CollabState>,
) -> Result<Room, String> {
    let room = state
        .rooms
        .create_room(request, user_id.clone())
        .map_err(|e| e.to_string())?;

    let encryption_key = RoomEncryption::generate_key();
    state.set_encryption_key(room.id, encryption_key);

    state
        .websocket
        .broadcast(room.id, CollabMessage::RoomCreated { room: room.clone() })
        .map_err(|e| e.to_string())?;

    Ok(room)
}

#[tauri::command]
pub async fn collab_list_rooms(
    include_private: bool,
    state: State<'_, CollabState>,
) -> Result<Vec<Room>, String> {
    Ok(state.rooms.list_rooms(include_private))
}

#[tauri::command]
pub async fn collab_get_room(
    room_id: String,
    state: State<'_, CollabState>,
) -> Result<Room, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    state.rooms.get_room(&uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn collab_delete_room(
    room_id: String,
    user_id: String,
    state: State<'_, CollabState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;

    state
        .rooms
        .delete_room(&uuid, &user_id)
        .map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(uuid, CollabMessage::RoomDeleted { room_id: uuid })
        .map_err(|e| e.to_string())?;

    state.websocket.clean_room(&uuid);
    state.rtc.clear_room(&uuid);

    Ok(())
}

#[tauri::command]
pub async fn collab_join_room(
    request: JoinRoomRequest,
    user_id: String,
    state: State<'_, CollabState>,
) -> Result<Participant, String> {
    let participant = state
        .rooms
        .join_room(request.clone(), user_id.clone())
        .map_err(|e| e.to_string())?;

    state.websocket.subscribe_to_room(request.room_id, user_id);

    state
        .websocket
        .broadcast(
            request.room_id,
            CollabMessage::ParticipantJoined {
                participant: participant.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(participant)
}

#[tauri::command]
pub async fn collab_leave_room(
    room_id: String,
    user_id: String,
    state: State<'_, CollabState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;

    let participant = state
        .rooms
        .get_participant(&uuid, &user_id)
        .map_err(|e| e.to_string())?;

    state
        .rooms
        .leave_room(&uuid, &user_id)
        .map_err(|e| e.to_string())?;

    state.websocket.unsubscribe_from_room(&uuid, &user_id);

    state
        .websocket
        .broadcast(
            uuid,
            CollabMessage::ParticipantLeft {
                participant_id: participant.id,
                user_id,
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn collab_get_participants(
    room_id: String,
    state: State<'_, CollabState>,
) -> Result<Vec<Participant>, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    Ok(state.rooms.get_participants(&uuid))
}

#[tauri::command]
pub async fn collab_update_permissions(
    request: UpdatePermissionsRequest,
    moderator_id: String,
    state: State<'_, CollabState>,
) -> Result<(), String> {
    let moderator = state
        .rooms
        .get_participant(&request.room_id, &moderator_id)
        .map_err(|e| e.to_string())?;

    if !can_modify_permissions(moderator.role) {
        return Err("Insufficient permissions".to_string());
    }

    let mut participant = state
        .rooms
        .get_participant(&request.room_id, &request.user_id)
        .map_err(|e| e.to_string())?;

    participant.permissions = request.permissions;

    state
        .rooms
        .update_participant(participant.clone())
        .map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(
            request.room_id,
            CollabMessage::ParticipantUpdated { participant },
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn collab_send_message(
    request: SendMessageRequest,
    user_id: String,
    username: String,
    state: State<'_, CollabState>,
) -> Result<ChatMessage, String> {
    let message = state
        .rooms
        .send_message(request.clone(), user_id, username)
        .map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(
            request.room_id,
            CollabMessage::ChatMessage {
                message: message.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(message)
}

#[tauri::command]
pub async fn collab_get_messages(
    room_id: String,
    limit: Option<usize>,
    state: State<'_, CollabState>,
) -> Result<Vec<ChatMessage>, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    Ok(state.rooms.get_messages(&uuid, limit))
}

#[tauri::command]
pub async fn collab_share_watchlist(
    room_id: String,
    name: String,
    symbols: Vec<String>,
    user_id: String,
    state: State<'_, CollabState>,
) -> Result<SharedWatchlist, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;

    let watchlist = SharedWatchlist {
        id: Uuid::new_v4(),
        room_id: uuid,
        name,
        owner_id: user_id,
        symbols,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state
        .rooms
        .add_watchlist(watchlist.clone())
        .map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(
            uuid,
            CollabMessage::WatchlistUpdated {
                watchlist: watchlist.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(watchlist)
}

#[tauri::command]
pub async fn collab_get_watchlists(
    room_id: String,
    state: State<'_, CollabState>,
) -> Result<Vec<SharedWatchlist>, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    Ok(state.rooms.get_watchlists(&uuid))
}

#[tauri::command]
pub async fn collab_share_order(
    request: ShareOrderRequest,
    user_id: String,
    username: String,
    state: State<'_, CollabState>,
) -> Result<SharedOrder, String> {
    let order = SharedOrder {
        id: Uuid::new_v4(),
        room_id: request.room_id,
        user_id,
        username,
        symbol: request.symbol,
        side: request.side,
        order_type: request.order_type,
        quantity: request.quantity,
        price: request.price,
        status: OrderStatus::Pending,
        timestamp: Utc::now(),
        notes: request.notes,
    };

    state
        .rooms
        .add_order(order.clone())
        .map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(
            request.room_id,
            CollabMessage::OrderShared {
                order: order.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(order)
}

#[tauri::command]
pub async fn collab_get_orders(
    room_id: String,
    state: State<'_, CollabState>,
) -> Result<Vec<SharedOrder>, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    Ok(state.rooms.get_orders(&uuid))
}

#[tauri::command]
pub async fn collab_update_order(
    order_id: String,
    room_id: String,
    status: OrderStatus,
    state: State<'_, CollabState>,
) -> Result<(), String> {
    let order_uuid = Uuid::parse_str(&order_id).map_err(|e| e.to_string())?;
    let room_uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;

    let mut orders = state.rooms.get_orders(&room_uuid);
    if let Some(order) = orders.iter_mut().find(|o| o.id == order_uuid) {
        order.status = status;
        state
            .rooms
            .update_order(order.clone())
            .map_err(|e| e.to_string())?;

        state
            .websocket
            .broadcast(
                room_uuid,
                CollabMessage::OrderUpdated {
                    order: order.clone(),
                },
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err("Order not found".to_string())
    }
}

#[tauri::command]
pub async fn collab_share_strategy(
    request: ShareStrategyRequest,
    user_id: String,
    username: String,
    state: State<'_, CollabState>,
) -> Result<Strategy, String> {
    let strategy = Strategy {
        id: Uuid::new_v4(),
        room_id: request.room_id,
        user_id,
        username,
        name: request.name,
        description: request.description,
        code: request.code,
        language: request.language,
        shared_at: Utc::now(),
        likes: 0,
        comments: Vec::new(),
    };

    state
        .websocket
        .broadcast(
            request.room_id,
            CollabMessage::StrategyShared {
                strategy: strategy.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(strategy)
}

#[tauri::command]
pub async fn collab_send_webrtc_signal(
    room_id: String,
    signal: WebRTCSignal,
    state: State<'_, CollabState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;

    state.rtc.enqueue_signal(uuid, signal.clone());

    state
        .websocket
        .broadcast(uuid, CollabMessage::WebRTCSignal { signal })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn collab_get_webrtc_signals(
    room_id: String,
    user_id: String,
    state: State<'_, CollabState>,
) -> Result<Vec<WebRTCSignal>, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    Ok(state.rtc.take_signals(&uuid, &user_id))
}

#[tauri::command]
pub async fn collab_moderate_user(
    request: ModerateUserRequest,
    moderator_id: String,
    state: State<'_, CollabState>,
) -> Result<ModerationAction, String> {
    let moderator = state
        .rooms
        .get_participant(&request.room_id, &moderator_id)
        .map_err(|e| e.to_string())?;

    let target = state
        .rooms
        .get_participant(&request.room_id, &request.target_user_id)
        .map_err(|e| e.to_string())?;

    use crate::collab::moderation::ensure_moderation_permission;
    ensure_moderation_permission(
        moderator.role,
        target.role,
        &moderator.permissions,
        request.action_type,
    )
    .map_err(|e| e.to_string())?;

    let duration = request
        .duration_minutes
        .map(|mins| std::time::Duration::from_secs((mins * 60) as u64));

    let action = state
        .moderation
        .apply_moderation(
            request.room_id,
            moderator_id,
            request.target_user_id,
            request.action_type,
            request.reason,
            duration,
        )
        .map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(
            request.room_id,
            CollabMessage::ModerationAction {
                action: action.clone(),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(action)
}

#[tauri::command]
pub async fn collab_get_room_state(
    room_id: String,
    state: State<'_, CollabState>,
) -> Result<RoomState, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    state.get_room_state(&uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn collab_set_competition(
    competition: Competition,
    state: State<'_, CollabState>,
) -> Result<(), String> {
    state
        .rooms
        .set_competition(competition.clone())
        .map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(
            competition.room_id,
            CollabMessage::CompetitionUpdated { competition },
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn collab_get_competition(
    room_id: String,
    state: State<'_, CollabState>,
) -> Result<Option<Competition>, String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;
    Ok(state.rooms.get_competition(&uuid))
}

#[tauri::command]
pub async fn collab_update_leaderboard(
    room_id: String,
    leaderboard: Vec<LeaderboardEntry>,
    state: State<'_, CollabState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&room_id).map_err(|e| e.to_string())?;

    state
        .websocket
        .broadcast(
            uuid,
            CollabMessage::LeaderboardUpdated {
                room_id: uuid,
                leaderboard,
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}
