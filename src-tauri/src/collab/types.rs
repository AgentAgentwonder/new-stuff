use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub max_participants: usize,
    pub is_public: bool,
    pub password_hash: Option<String>,
    pub encryption_enabled: bool,
    pub voice_enabled: bool,
    pub video_enabled: bool,
    pub screen_share_enabled: bool,
    pub settings: RoomSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSettings {
    pub allow_guest_join: bool,
    pub require_approval: bool,
    pub allow_voice_chat: bool,
    pub allow_video_chat: bool,
    pub allow_screen_share: bool,
    pub allow_order_sharing: bool,
    pub allow_strategy_sharing: bool,
    pub moderation_enabled: bool,
    pub competition_mode: bool,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            allow_guest_join: false,
            require_approval: true,
            allow_voice_chat: true,
            allow_video_chat: true,
            allow_screen_share: true,
            allow_order_sharing: true,
            allow_strategy_sharing: true,
            moderation_enabled: true,
            competition_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub user_id: String,
    pub username: String,
    pub room_id: Uuid,
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub role: ParticipantRole,
    pub permissions: ParticipantPermissions,
    pub status: ParticipantStatus,
    pub is_muted: bool,
    pub is_video_off: bool,
    pub is_screen_sharing: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParticipantRole {
    Owner,
    Moderator,
    Member,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPermissions {
    pub can_speak: bool,
    pub can_share_video: bool,
    pub can_share_screen: bool,
    pub can_chat: bool,
    pub can_share_orders: bool,
    pub can_share_strategies: bool,
    pub can_moderate: bool,
    pub can_kick: bool,
    pub can_ban: bool,
}

impl Default for ParticipantPermissions {
    fn default() -> Self {
        Self {
            can_speak: true,
            can_share_video: true,
            can_share_screen: true,
            can_chat: true,
            can_share_orders: true,
            can_share_strategies: true,
            can_moderate: false,
            can_kick: false,
            can_ban: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParticipantStatus {
    Active,
    Idle,
    Away,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub encrypted: bool,
    pub mentions: Vec<String>,
    pub replied_to: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWatchlist {
    pub id: Uuid,
    pub room_id: Uuid,
    pub name: String,
    pub owner_id: String,
    pub symbols: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedOrder {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: String,
    pub username: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub status: OrderStatus,
    pub timestamp: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: String,
    pub username: String,
    pub name: String,
    pub description: String,
    pub code: String,
    pub language: StrategyLanguage,
    pub shared_at: DateTime<Utc>,
    pub likes: usize,
    pub comments: Vec<StrategyComment>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StrategyLanguage {
    Python,
    JavaScript,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyComment {
    pub id: Uuid,
    pub user_id: String,
    pub username: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    pub id: Uuid,
    pub room_id: Uuid,
    pub name: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub rules: CompetitionRules,
    pub leaderboard: Vec<LeaderboardEntry>,
    pub status: CompetitionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionRules {
    pub starting_capital: f64,
    pub allowed_assets: Option<Vec<String>>,
    pub max_position_size: Option<f64>,
    pub scoring_method: ScoringMethod,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScoringMethod {
    TotalReturn,
    RiskAdjustedReturn,
    WinRate,
    ProfitFactor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub user_id: String,
    pub username: String,
    pub score: f64,
    pub trades: usize,
    pub win_rate: f64,
    pub total_return: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompetitionStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationAction {
    pub id: Uuid,
    pub room_id: Uuid,
    pub moderator_id: String,
    pub target_user_id: String,
    pub action_type: ModerationActionType,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ModerationActionType {
    Mute,
    Kick,
    Ban,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTCSignal {
    pub from_user_id: String,
    pub to_user_id: String,
    pub signal_type: SignalType,
    pub data: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SignalType {
    Offer,
    Answer,
    IceCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollabMessage {
    RoomCreated {
        room: Room,
    },
    RoomUpdated {
        room: Room,
    },
    RoomDeleted {
        room_id: Uuid,
    },
    ParticipantJoined {
        participant: Participant,
    },
    ParticipantLeft {
        participant_id: Uuid,
        user_id: String,
    },
    ParticipantUpdated {
        participant: Participant,
    },
    ChatMessage {
        message: ChatMessage,
    },
    WatchlistUpdated {
        watchlist: SharedWatchlist,
    },
    OrderShared {
        order: SharedOrder,
    },
    OrderUpdated {
        order: SharedOrder,
    },
    StrategyShared {
        strategy: Strategy,
    },
    CompetitionUpdated {
        competition: Competition,
    },
    LeaderboardUpdated {
        room_id: Uuid,
        leaderboard: Vec<LeaderboardEntry>,
    },
    ModerationAction {
        action: ModerationAction,
    },
    WebRTCSignal {
        signal: WebRTCSignal,
    },
    StateSync {
        state: RoomState,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomState {
    pub room: Room,
    pub participants: Vec<Participant>,
    pub watchlists: Vec<SharedWatchlist>,
    pub active_orders: Vec<SharedOrder>,
    pub competition: Option<Competition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub name: String,
    pub description: Option<String>,
    pub max_participants: usize,
    pub is_public: bool,
    pub password: Option<String>,
    pub settings: RoomSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRoomRequest {
    pub room_id: Uuid,
    pub password: Option<String>,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePermissionsRequest {
    pub room_id: Uuid,
    pub user_id: String,
    pub permissions: ParticipantPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub room_id: Uuid,
    pub content: String,
    pub replied_to: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareOrderRequest {
    pub room_id: Uuid,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareStrategyRequest {
    pub room_id: Uuid,
    pub name: String,
    pub description: String,
    pub code: String,
    pub language: StrategyLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerateUserRequest {
    pub room_id: Uuid,
    pub target_user_id: String,
    pub action_type: ModerationActionType,
    pub reason: String,
    pub duration_minutes: Option<i64>,
}
