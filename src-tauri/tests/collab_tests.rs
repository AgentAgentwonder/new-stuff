use app_lib::collab::crypto::{hash_password, verify_password, RoomEncryption};
use app_lib::collab::moderation::{ensure_moderation_permission, ModerationManager};
use app_lib::collab::permissions::default_permissions_for_role;
use app_lib::collab::rtc::RtcSessionManager;
use app_lib::collab::state::CollabState;
use app_lib::collab::types::*;
use app_lib::collab::websocket::CollabWebSocketManager;
use uuid::Uuid;

#[tokio::test]
async fn test_room_creation_and_join() {
    let ws_manager = CollabWebSocketManager::without_handle();
    let state = CollabState::new(ws_manager);

    // Create a room
    let create_req = CreateRoomRequest {
        name: "Test Room".to_string(),
        description: Some("A test room".to_string()),
        max_participants: 10,
        is_public: true,
        password: None,
        settings: RoomSettings::default(),
    };

    let room = state
        .rooms
        .create_room(create_req, "owner123".to_string())
        .expect("Failed to create room");

    assert_eq!(room.name, "Test Room");
    assert_eq!(room.owner_id, "owner123");
    assert_eq!(room.max_participants, 10);

    // Join the room
    let join_req = JoinRoomRequest {
        room_id: room.id,
        password: None,
        username: "testuser".to_string(),
    };

    let participant = state
        .rooms
        .join_room(join_req, "user456".to_string())
        .expect("Failed to join room");

    assert_eq!(participant.username, "testuser");
    assert_eq!(participant.room_id, room.id);

    // List participants
    let participants = state.rooms.get_participants(&room.id);
    assert_eq!(participants.len(), 1);
}

#[tokio::test]
async fn test_password_protected_room() {
    let ws_manager = CollabWebSocketManager::without_handle();
    let state = CollabState::new(ws_manager);

    let create_req = CreateRoomRequest {
        name: "Private Room".to_string(),
        description: None,
        max_participants: 5,
        is_public: false,
        password: Some("secret123".to_string()),
        settings: RoomSettings::default(),
    };

    let room = state
        .rooms
        .create_room(create_req, "owner123".to_string())
        .unwrap();

    assert!(room.password_hash.is_some());

    // Try joining with wrong password
    let join_req_wrong = JoinRoomRequest {
        room_id: room.id,
        password: Some("wrongpassword".to_string()),
        username: "testuser".to_string(),
    };

    let result = state.rooms.join_room(join_req_wrong, "user456".to_string());
    assert!(result.is_err());

    // Join with correct password
    let join_req_correct = JoinRoomRequest {
        room_id: room.id,
        password: Some("secret123".to_string()),
        username: "testuser".to_string(),
    };

    let participant = state
        .rooms
        .join_room(join_req_correct, "user456".to_string())
        .unwrap();

    assert_eq!(participant.username, "testuser");
}

#[tokio::test]
async fn test_chat_messages() {
    let ws_manager = CollabWebSocketManager::without_handle();
    let state = CollabState::new(ws_manager);

    let create_req = CreateRoomRequest {
        name: "Chat Room".to_string(),
        description: None,
        max_participants: 10,
        is_public: true,
        password: None,
        settings: RoomSettings::default(),
    };

    let room = state
        .rooms
        .create_room(create_req, "owner123".to_string())
        .unwrap();

    let join_req = JoinRoomRequest {
        room_id: room.id,
        password: None,
        username: "testuser".to_string(),
    };

    state
        .rooms
        .join_room(join_req, "user456".to_string())
        .unwrap();

    let msg_req = SendMessageRequest {
        room_id: room.id,
        content: "Hello, world!".to_string(),
        replied_to: None,
    };

    let message = state
        .rooms
        .send_message(msg_req, "user456".to_string(), "testuser".to_string())
        .unwrap();

    assert_eq!(message.content, "Hello, world!");
    assert_eq!(message.username, "testuser");

    let messages = state.rooms.get_messages(&room.id, None);
    assert_eq!(messages.len(), 1);
}

#[tokio::test]
async fn test_watchlist_sharing() {
    let ws_manager = CollabWebSocketManager::without_handle();
    let state = CollabState::new(ws_manager);

    let create_req = CreateRoomRequest {
        name: "Trading Room".to_string(),
        description: None,
        max_participants: 10,
        is_public: true,
        password: None,
        settings: RoomSettings::default(),
    };

    let room = state
        .rooms
        .create_room(create_req, "owner123".to_string())
        .unwrap();

    let watchlist = SharedWatchlist {
        id: Uuid::new_v4(),
        room_id: room.id,
        name: "My Watchlist".to_string(),
        owner_id: "user456".to_string(),
        symbols: vec!["SOL".to_string(), "BTC".to_string(), "ETH".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state.rooms.add_watchlist(watchlist.clone()).unwrap();

    let watchlists = state.rooms.get_watchlists(&room.id);
    assert_eq!(watchlists.len(), 1);
    assert_eq!(watchlists[0].symbols.len(), 3);
}

#[tokio::test]
async fn test_order_sharing() {
    let ws_manager = CollabWebSocketManager::without_handle();
    let state = CollabState::new(ws_manager);

    let create_req = CreateRoomRequest {
        name: "Trading Room".to_string(),
        description: None,
        max_participants: 10,
        is_public: true,
        password: None,
        settings: RoomSettings::default(),
    };

    let room = state
        .rooms
        .create_room(create_req, "owner123".to_string())
        .unwrap();

    let order = SharedOrder {
        id: Uuid::new_v4(),
        room_id: room.id,
        user_id: "user456".to_string(),
        username: "testuser".to_string(),
        symbol: "SOL".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: 10.0,
        price: Some(100.0),
        status: OrderStatus::Pending,
        timestamp: chrono::Utc::now(),
        notes: Some("Test order".to_string()),
    };

    state.rooms.add_order(order.clone()).unwrap();

    let orders = state.rooms.get_orders(&room.id);
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].symbol, "SOL");
}

#[tokio::test]
async fn test_permissions() {
    let owner_perms = default_permissions_for_role(ParticipantRole::Owner);
    assert!(owner_perms.can_moderate);
    assert!(owner_perms.can_kick);
    assert!(owner_perms.can_ban);

    let moderator_perms = default_permissions_for_role(ParticipantRole::Moderator);
    assert!(moderator_perms.can_moderate);
    assert!(moderator_perms.can_kick);
    assert!(!moderator_perms.can_ban);

    let member_perms = default_permissions_for_role(ParticipantRole::Member);
    assert!(!member_perms.can_moderate);
    assert!(!member_perms.can_kick);
    assert!(!member_perms.can_ban);

    let guest_perms = default_permissions_for_role(ParticipantRole::Guest);
    assert!(!guest_perms.can_speak);
    assert!(!guest_perms.can_chat);
}

#[tokio::test]
async fn test_moderation_actions() {
    let mod_manager = ModerationManager::new();

    let room_id = Uuid::new_v4();
    let action = mod_manager
        .apply_moderation(
            room_id,
            "moderator123".to_string(),
            "baduser456".to_string(),
            ModerationActionType::Mute,
            "Spamming".to_string(),
            Some(std::time::Duration::from_secs(300)),
        )
        .unwrap();

    assert_eq!(action.action_type, ModerationActionType::Mute);
    assert_eq!(action.reason, "Spamming");
    assert!(action.expires_at.is_some());
}

#[tokio::test]
async fn test_encryption() {
    let key = RoomEncryption::generate_key();
    let encryption = RoomEncryption::new(&key).unwrap();

    let plaintext = "Secret trading message";
    let encrypted = encryption.encrypt(plaintext).unwrap();
    let decrypted = encryption.decrypt(&encrypted).unwrap();

    assert_eq!(plaintext, decrypted);
}

#[tokio::test]
async fn test_password_hashing() {
    let password = "supersecret123";
    let hash = hash_password(password).unwrap();

    assert!(verify_password(password, &hash).unwrap());
    assert!(!verify_password("wrongpassword", &hash).unwrap());
}

#[tokio::test]
async fn test_webrtc_signals() {
    let rtc_manager = RtcSessionManager::new();
    let room_id = Uuid::new_v4();

    let signal = WebRTCSignal {
        from_user_id: "user1".to_string(),
        to_user_id: "user2".to_string(),
        signal_type: SignalType::Offer,
        data: "mock-sdp-offer".to_string(),
    };

    rtc_manager.enqueue_signal(room_id, signal.clone());

    let signals = rtc_manager.take_signals(&room_id, "user2");
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].from_user_id, "user1");
    assert_eq!(signals[0].signal_type, SignalType::Offer);
}

#[tokio::test]
async fn test_competition_leaderboard() {
    let ws_manager = CollabWebSocketManager::without_handle();
    let state = CollabState::new(ws_manager);

    let create_req = CreateRoomRequest {
        name: "Competition Room".to_string(),
        description: None,
        max_participants: 20,
        is_public: true,
        password: None,
        settings: RoomSettings {
            competition_mode: true,
            ..Default::default()
        },
    };

    let room = state
        .rooms
        .create_room(create_req, "owner123".to_string())
        .unwrap();

    let competition = Competition {
        id: Uuid::new_v4(),
        room_id: room.id,
        name: "Trading Competition".to_string(),
        description: "Test competition".to_string(),
        start_time: chrono::Utc::now(),
        end_time: chrono::Utc::now() + chrono::Duration::hours(24),
        rules: CompetitionRules {
            starting_capital: 10000.0,
            allowed_assets: Some(vec!["SOL".to_string(), "BTC".to_string()]),
            max_position_size: Some(1000.0),
            scoring_method: ScoringMethod::TotalReturn,
        },
        leaderboard: vec![
            LeaderboardEntry {
                rank: 1,
                user_id: "user1".to_string(),
                username: "Trader1".to_string(),
                score: 1250.0,
                trades: 15,
                win_rate: 0.8,
                total_return: 12.5,
            },
            LeaderboardEntry {
                rank: 2,
                user_id: "user2".to_string(),
                username: "Trader2".to_string(),
                score: 850.0,
                trades: 10,
                win_rate: 0.7,
                total_return: 8.5,
            },
        ],
        status: CompetitionStatus::Active,
    };

    state.rooms.set_competition(competition.clone()).unwrap();

    let retrieved_comp = state.rooms.get_competition(&room.id).unwrap();
    assert_eq!(retrieved_comp.leaderboard.len(), 2);
    assert_eq!(retrieved_comp.status, CompetitionStatus::Active);
}

#[tokio::test]
async fn test_moderation_permission_check() {
    let moderator_perms = default_permissions_for_role(ParticipantRole::Moderator);

    // Moderator can mute a member
    let result = ensure_moderation_permission(
        ParticipantRole::Moderator,
        ParticipantRole::Member,
        &moderator_perms,
        ModerationActionType::Mute,
    );
    assert!(result.is_ok());

    // Moderator cannot ban (only kick)
    let member_perms = default_permissions_for_role(ParticipantRole::Member);
    let result = ensure_moderation_permission(
        ParticipantRole::Member,
        ParticipantRole::Member,
        &member_perms,
        ModerationActionType::Kick,
    );
    assert!(result.is_err());
}

#[tokio::test]
async fn test_room_state_sync() {
    let ws_manager = CollabWebSocketManager::without_handle();
    let state = CollabState::new(ws_manager);

    let create_req = CreateRoomRequest {
        name: "Sync Room".to_string(),
        description: None,
        max_participants: 10,
        is_public: true,
        password: None,
        settings: RoomSettings::default(),
    };

    let room = state
        .rooms
        .create_room(create_req, "owner123".to_string())
        .unwrap();

    let join_req = JoinRoomRequest {
        room_id: room.id,
        password: None,
        username: "testuser".to_string(),
    };

    state
        .rooms
        .join_room(join_req, "user456".to_string())
        .unwrap();

    let room_state = state.get_room_state(&room.id).unwrap();

    assert_eq!(room_state.room.name, "Sync Room");
    assert_eq!(room_state.participants.len(), 1);
    assert_eq!(room_state.watchlists.len(), 0);
    assert_eq!(room_state.active_orders.len(), 0);
    assert!(room_state.competition.is_none());
}
