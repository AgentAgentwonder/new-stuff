# Collaborative Rooms Guide

## Overview

The Collaborative Rooms feature enables real-time co-trading through WebRTC/WebSocket technology. Traders can share watchlists, joint orders, and compete in live trading competitions with voice/video chat, screen sharing, and comprehensive permission controls.

## Architecture

### Backend Components

- **Room Management** (`src-tauri/src/collab/room.rs`): Handles room creation, deletion, and participant management
- **WebSocket Manager** (`src-tauri/src/collab/websocket.rs`): Real-time message broadcasting across room participants
- **WebRTC Session Manager** (`src-tauri/src/collab/rtc.rs`): Manages peer-to-peer voice/video signaling
- **Encryption** (`src-tauri/src/collab/crypto.rs`): AES-256-GCM encryption for room messages and content
- **Permissions** (`src-tauri/src/collab/permissions.rs`): Role-based access control system
- **Moderation** (`src-tauri/src/collab/moderation.rs`): Tools for muting, kicking, and banning users

### Frontend Components

The frontend implementation should be added to `src/components/collab/` with the following components:

- **CollabRoom.tsx**: Main collaborative room interface
- **RoomList.tsx**: Browse and filter available rooms
- **SharedChart.tsx**: Synchronized chart viewing
- **RoomChat.tsx**: Real-time text chat
- **Leaderboard.tsx**: Competition rankings
- **StrategyShare.tsx**: Share and discuss trading strategies
- **VoiceVideoControls.tsx**: Audio/video controls
- **PermissionsPanel.tsx**: Manage participant roles and permissions

## Network Requirements

### Ports and Protocols

- **WebSocket**: Uses standard Tauri WebSocket implementation over existing HTTP/HTTPS ports
- **WebRTC**:
  - **STUN**: UDP ports 3478-3479 (for NAT traversal)
  - **TURN**: TCP/UDP ports 443, 3478 (relay fallback)
  - **Media**: UDP ports 49152-65535 (dynamic RTP/RTCP)

### Recommended STUN/TURN Servers

```json
{
  "iceServers": [
    { "urls": "stun:stun.l.google.com:19302" },
    { "urls": "stun:stun1.l.google.com:19302" },
    {
      "urls": "turn:your-turn-server.com:3478",
      "username": "user",
      "credential": "pass"
    }
  ]
}
```

### Firewall Configuration

For corporate networks, the following must be allowed:

- Outbound WebSocket connections (WSS on port 443)
- UDP traffic on ports 49152-65535 for WebRTC media
- STUN/TURN server access on designated ports

### Bandwidth Requirements

| Feature | Minimum | Recommended |
|---------|---------|-------------|
| Text Chat | 10 Kbps | 50 Kbps |
| Voice Only | 50 Kbps | 100 Kbps |
| Video (720p) | 500 Kbps | 1 Mbps |
| Screen Sharing | 500 Kbps | 2 Mbps |
| Multiple Participants | Scales linearly | Use SFU* |

*SFU (Selective Forwarding Unit) recommended for rooms with 5+ video participants

## Security & Privacy

### Encryption

All collaborative room communications are encrypted:

- **At Rest**: Room passwords hashed with Argon2
- **In Transit**: 
  - Text messages: AES-256-GCM encryption
  - WebRTC media: DTLS-SRTP (mandatory)
  - WebSocket: TLS 1.3 (WSS)

### Privacy Controls

#### Room Creation

```typescript
interface CreateRoomRequest {
  name: string;
  description?: string;
  max_participants: number;
  is_public: boolean;        // Public rooms appear in listings
  password?: string;          // Optional password protection
  settings: RoomSettings;
}

interface RoomSettings {
  allow_guest_join: boolean;     // Allow non-members to join
  require_approval: boolean;     // Owner must approve joins
  allow_voice_chat: boolean;
  allow_video_chat: boolean;
  allow_screen_share: boolean;
  allow_order_sharing: boolean;
  allow_strategy_sharing: boolean;
  moderation_enabled: boolean;
  competition_mode: boolean;
}
```

#### User Roles

1. **Owner**: Full control over room, can delete room
2. **Moderator**: Can kick/mute users, cannot ban
3. **Member**: Standard participation rights
4. **Guest**: View-only (configurable)

#### Permission System

```typescript
interface ParticipantPermissions {
  can_speak: boolean;
  can_share_video: boolean;
  can_share_screen: boolean;
  can_chat: boolean;
  can_share_orders: boolean;
  can_share_strategies: boolean;
  can_moderate: boolean;
  can_kick: boolean;
  can_ban: boolean;
}
```

### Moderation Tools

Moderators and owners can:

- **Mute**: Temporarily disable voice for a user
- **Kick**: Remove user from room (can rejoin)
- **Ban**: Permanently block user from room
- **Warning**: Issue warnings without restrictions

All moderation actions are logged with:
- Action type and reason
- Moderator ID
- Timestamp
- Optional expiration time

### Data Retention

- **Chat messages**: Stored in memory, cleared on room close
- **Shared orders**: Persist while room is active
- **Watchlists**: Saved to room state
- **Competitions**: Leaderboard data persists after room closes
- **Moderation logs**: Retained for audit purposes

## API Reference

### Room Management

```rust
// Create a new room
collab_create_room(request: CreateRoomRequest, user_id: String) -> Result<Room>

// List available rooms
collab_list_rooms(include_private: bool) -> Result<Vec<Room>>

// Get room details
collab_get_room(room_id: String) -> Result<Room>

// Delete room (owner only)
collab_delete_room(room_id: String, user_id: String) -> Result<()>

// Join a room
collab_join_room(request: JoinRoomRequest, user_id: String) -> Result<Participant>

// Leave a room
collab_leave_room(room_id: String, user_id: String) -> Result<()>
```

### Participant Management

```rust
// Get all participants
collab_get_participants(room_id: String) -> Result<Vec<Participant>>

// Update user permissions
collab_update_permissions(
  request: UpdatePermissionsRequest, 
  moderator_id: String
) -> Result<()>
```

### Communication

```rust
// Send chat message
collab_send_message(
  request: SendMessageRequest, 
  user_id: String, 
  username: String
) -> Result<ChatMessage>

// Get message history
collab_get_messages(room_id: String, limit: Option<usize>) -> Result<Vec<ChatMessage>>

// Send WebRTC signaling data
collab_send_webrtc_signal(room_id: String, signal: WebRTCSignal) -> Result<()>

// Get pending WebRTC signals
collab_get_webrtc_signals(room_id: String, user_id: String) -> Result<Vec<WebRTCSignal>>
```

### Trading Features

```rust
// Share watchlist
collab_share_watchlist(
  room_id: String, 
  name: String, 
  symbols: Vec<String>, 
  user_id: String
) -> Result<SharedWatchlist>

// Get room watchlists
collab_get_watchlists(room_id: String) -> Result<Vec<SharedWatchlist>>

// Share an order
collab_share_order(
  request: ShareOrderRequest, 
  user_id: String, 
  username: String
) -> Result<SharedOrder>

// Get shared orders
collab_get_orders(room_id: String) -> Result<Vec<SharedOrder>>

// Update order status
collab_update_order(
  order_id: String, 
  room_id: String, 
  status: OrderStatus
) -> Result<()>

// Share a strategy
collab_share_strategy(
  request: ShareStrategyRequest, 
  user_id: String, 
  username: String
) -> Result<Strategy>
```

### Competitions

```rust
// Create/update competition
collab_set_competition(competition: Competition) -> Result<()>

// Get competition details
collab_get_competition(room_id: String) -> Result<Option<Competition>>

// Update leaderboard
collab_update_leaderboard(
  room_id: String, 
  leaderboard: Vec<LeaderboardEntry>
) -> Result<()>
```

### Moderation

```rust
// Moderate a user
collab_moderate_user(
  request: ModerateUserRequest, 
  moderator_id: String
) -> Result<ModerationAction>
```

### State Synchronization

```rust
// Get complete room state
collab_get_room_state(room_id: String) -> Result<RoomState>
```

## WebSocket Events

The frontend should listen for these events:

```typescript
// Room events
listen('collab:room:${roomId}', (event: CollabMessage) => {
  switch (event.type) {
    case 'RoomCreated':
    case 'RoomUpdated':
    case 'RoomDeleted':
    case 'ParticipantJoined':
    case 'ParticipantLeft':
    case 'ParticipantUpdated':
    case 'ChatMessage':
    case 'WatchlistUpdated':
    case 'OrderShared':
    case 'OrderUpdated':
    case 'StrategyShared':
    case 'CompetitionUpdated':
    case 'LeaderboardUpdated':
    case 'ModerationAction':
    case 'WebRTCSignal':
    case 'StateSync':
  }
});
```

## Usage Examples

### Creating a Private Trading Room

```typescript
const room = await invoke('collab_create_room', {
  request: {
    name: 'Elite Traders',
    description: 'For experienced traders only',
    max_participants: 10,
    is_public: false,
    password: 'secret123',
    settings: {
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
  },
  userId: 'user123'
});
```

### Joining a Room

```typescript
const participant = await invoke('collab_join_room', {
  request: {
    room_id: roomId,
    password: 'secret123', // if password protected
    username: 'TraderJoe'
  },
  userId: 'user456'
});

// Listen for room events
await listen(`collab:room:${roomId}`, (event) => {
  console.log('Room event:', event.payload);
});
```

### Sharing a Watchlist

```typescript
const watchlist = await invoke('collab_share_watchlist', {
  roomId: roomId,
  name: 'Top Gainers',
  symbols: ['SOL', 'BTC', 'ETH', 'BONK'],
  userId: 'user123'
});
```

### Starting a WebRTC Video Call

```typescript
// Create offer
const pc = new RTCPeerConnection(iceConfig);
const offer = await pc.createOffer();
await pc.setLocalDescription(offer);

// Send offer via signaling
await invoke('collab_send_webrtc_signal', {
  roomId: roomId,
  signal: {
    from_user_id: myUserId,
    to_user_id: targetUserId,
    signal_type: 'Offer',
    data: JSON.stringify(offer)
  }
});

// Poll for answer
const signals = await invoke('collab_get_webrtc_signals', {
  roomId: roomId,
  userId: myUserId
});
```

### Setting Up a Competition

```typescript
const competition = await invoke('collab_set_competition', {
  competition: {
    id: uuid(),
    room_id: roomId,
    name: '24h Trading Challenge',
    description: 'Who can make the most profit?',
    start_time: new Date().toISOString(),
    end_time: new Date(Date.now() + 24*60*60*1000).toISOString(),
    rules: {
      starting_capital: 10000,
      allowed_assets: ['SOL', 'BTC', 'ETH'],
      max_position_size: 1000,
      scoring_method: 'TotalReturn'
    },
    leaderboard: [],
    status: 'Active'
  }
});
```

## Testing

Integration tests are located in `src-tauri/tests/collab_tests.rs` and cover:

- Room creation and joining
- Password-protected rooms
- Chat messaging
- Watchlist sharing
- Order sharing
- Permissions and roles
- Moderation actions
- Encryption
- WebRTC signaling
- Competition leaderboards
- State synchronization

Run tests with:
```bash
cd src-tauri
cargo test collab_tests
```

## Performance Considerations

### Scalability

- **Small rooms (2-5 participants)**: Direct P2P WebRTC connections
- **Medium rooms (6-15 participants)**: Consider mesh topology
- **Large rooms (16+ participants)**: Implement SFU for media relay

### Optimization Tips

1. **Message Batching**: Batch non-critical updates to reduce event frequency
2. **State Compression**: Use delta updates instead of full state syncs
3. **Video Quality**: Adaptive bitrate based on network conditions
4. **Screen Sharing**: Limit framerate to 5-10 fps for better performance

### Resource Management

- Limit message history in memory (e.g., last 100 messages)
- Clean up expired moderation actions periodically
- Implement reconnection logic with exponential backoff
- Monitor bandwidth usage and adjust quality settings

## Troubleshooting

### Common Issues

1. **Cannot join room**: Check password, room capacity, and ban list
2. **WebRTC not connecting**: Verify STUN/TURN servers and firewall rules
3. **Poor audio/video quality**: Check bandwidth, reduce quality settings
4. **Messages not syncing**: Verify WebSocket connection status

### Debug Mode

Enable debug logging:
```rust
RUST_LOG=collab=debug cargo run
```

## Future Enhancements

- [ ] End-to-end encryption for P2P media
- [ ] Recording and replay of trading sessions
- [ ] AI-powered trade analysis in rooms
- [ ] Integration with external trading signals
- [ ] Mobile app support
- [ ] Room templates and presets
- [ ] Advanced analytics dashboard
- [ ] Reputation system integration
- [ ] Scheduled rooms and events
- [ ] Room bookmarking and favorites

## Support

For issues or questions:
- Check logs in `~/.local/share/eclipse-market-pro/logs/`
- Review test cases for usage examples
- Consult the main README.md for general setup

## License

This feature is part of Eclipse Market Pro and follows the same license terms.
