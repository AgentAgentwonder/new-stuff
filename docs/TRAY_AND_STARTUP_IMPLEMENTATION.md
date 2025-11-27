# Tray & Startup Controls Implementation

## Overview
Implemented system tray integration and auto-start functionality as specified in Phase 8 Tasks 8.9–8.11.

## Backend Implementation (Rust/Tauri)

### 1. System Tray Module (`src-tauri/src/tray.rs`)

**Features:**
- System tray icon with customizable styles (default, bullish, bearish, minimal)
- Tray menu with quick actions (Open, Settings, Exit)
- Portfolio stats display (portfolio value, P&L)
- Alert previews (up to 3 recent alerts)
- Badge counts for active alerts
- Minimize-to-tray and close-to-tray behavior
- Global keyboard shortcut for restore (default: CmdOrControl+Shift+M)
- Desktop notifications on minimize
- Settings persistence to JSON file

**Commands:**
- `get_tray_settings()` - Retrieve current tray settings
- `update_tray_settings(settings)` - Update tray configuration
- `update_tray_stats(stats)` - Update portfolio stats in tray menu
- `update_tray_badge(count)` - Update alert badge count
- `minimize_to_tray()` - Hide window to tray
- `restore_from_tray()` - Restore window from tray

**Tray Settings Structure:**
```rust
{
    enabled: bool,
    minimize_to_tray: bool,
    close_to_tray: bool,
    show_badge: bool,
    show_stats: bool,
    show_alerts: bool,
    show_notifications: bool,
    icon_style: "default" | "bullish" | "bearish" | "minimal",
    restore_shortcut: Option<String>
}
```

### 2. Auto-Start Module (`src-tauri/src/auto_start.rs`)

**Features:**
- Cross-platform auto-start support using `auto-launch` crate
  - Windows: Task Scheduler
  - macOS: LaunchAgent
  - Linux: systemd/autostart entry
- Start minimized option
- Configurable startup delay (0-300 seconds)
- Settings persistence to JSON file
- Auto-sync with OS startup configuration

**Commands:**
- `get_auto_start_settings()` - Retrieve current auto-start settings
- `update_auto_start_settings(settings)` - Update auto-start configuration
- `check_auto_start_enabled()` - Check if auto-start is active
- `enable_auto_start()` - Quick enable auto-start
- `disable_auto_start()` - Quick disable auto-start

**Auto-Start Settings Structure:**
```rust
{
    enabled: bool,
    start_minimized: bool,
    delay_seconds: u32
}
```

### 3. Integration in `lib.rs`

**Initialization:**
- System tray created and attached to app builder
- Tray event handler registered for menu interactions
- Window event listeners attached for close/minimize behavior
- Auto-start behavior handled on app launch
- Both managers initialized with settings loaded from disk

**Dependencies Added:**
- `tauri` features: `system-tray`, `icon-png`
- `auto-launch = "0.5.0"`
- Using existing `parking_lot` for thread-safe state management

## Frontend Implementation (React/TypeScript)

### 1. Tray Settings Component (`src/pages/Settings/TraySettings.tsx`)

**Features:**
- Master toggle to enable/disable tray functionality
- Window behavior options:
  - Minimize to tray
  - Close to tray
  - Show notifications on minimize
- Tray menu customization:
  - Toggle portfolio stats display
  - Toggle alert previews
  - Toggle badge count
- Icon style selector (4 styles)
- Custom keyboard shortcut input
- Test action button to minimize to tray
- Real-time save with success/error feedback

### 2. Startup Settings Component (`src/pages/Settings/StartupSettings.tsx`)

**Features:**
- Master toggle to enable/disable auto-start
- Startup options:
  - Start minimized toggle
  - Startup delay configuration (0-300 seconds)
- Quick action buttons for enable/disable
- Platform-specific info display
- Real-time save with success/error feedback

### 3. Settings Page Integration (`src/pages/Settings.tsx`)

**Added sections:**
- **System Tray** - Between Auto Update and Backup sections
- **Startup** - Between Tray and Auto Update sections

Both sections follow the existing design pattern with gradient icon headers and responsive layouts.

## Tests (`src/__tests__/tray.test.tsx`)

**Coverage:**

1. **Tray Settings Tests:**
   - Loading settings on mount
   - Updating settings
   - Minimize/restore actions
   - Stats updates
   - Badge count updates
   - Icon style handling

2. **Startup Settings Tests:**
   - Loading settings on mount
   - Updating settings
   - Enable/disable actions
   - Status checking
   - Delay configuration
   - Start minimized configuration

3. **Integration Tests:**
   - Tray menu interactions
   - Portfolio stats display
   - Alert preview display
   - Close/minimize behavior
   - Notification preferences

## Key Features

### Tray Icon
- **Left Click**: Restore window from tray
- **Menu Items**:
  - Open: Restore and focus window
  - Portfolio Value: Display current portfolio worth (if enabled)
  - P&L: Display profit/loss percentage and value (if enabled)
  - Alerts: Show active alert count with previews (if enabled)
  - Settings: Navigate to settings page
  - Exit: Quit application

### Minimize Behavior
- **Minimize to Tray**: Window hides to tray instead of taskbar when minimized
- **Close to Tray**: Close button hides to tray instead of quitting app
- **Notifications**: Optional notification on minimize with restore shortcut hint
- **Global Shortcut**: Configurable hotkey to restore (default: Cmd/Ctrl+Shift+M)

### Auto-Start
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Start Minimized**: Launch hidden to tray on system boot
- **Startup Delay**: Configurable delay to avoid boot storms
- **Detection**: App detects `--auto-start` argument to trigger special behavior

## Platform-Specific Notes

### Windows
- Uses Task Scheduler for reliable startup
- Tray icon in system tray notification area
- Windows Hello integration (already implemented)

### macOS
- Uses LaunchAgent for auto-start
- Menu bar icon with dropdown menu
- Touch ID integration (already implemented)

### Linux
- Uses systemd or XDG autostart depending on distribution
- System tray varies by desktop environment
- Password-only authentication

## Configuration Files

Settings are persisted to:
- **Tray**: `{app_data_dir}/tray_settings.json`
- **Auto-Start**: `{app_data_dir}/auto_start_settings.json`

## Usage Example

### Updating Tray Stats from Frontend
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Update portfolio stats in tray
await invoke('update_tray_stats', {
  stats: {
    portfolio_value: 50000.0,
    pnl_percentage: 5.5,
    pnl_value: 2500.0,
    alert_count: 3,
    recent_alerts: [
      { id: '1', title: 'BTC Alert', summary: 'Price above $50K' },
      { id: '2', title: 'ETH Alert', summary: 'Volume spike' },
    ]
  }
});

// Update just the badge count
await invoke('update_tray_badge', { count: 5 });
```

### Auto-Start Configuration
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Enable auto-start with options
await invoke('update_auto_start_settings', {
  settings: {
    enabled: true,
    start_minimized: true,
    delay_seconds: 10
  }
});
```

## Dependencies

### Rust Crates
- `auto-launch = "0.5.0"` - Cross-platform auto-start
- `parking_lot = "0.12.1"` - Thread-safe locks (already in project)
- `serde` - Serialization (already in project)
- `tauri` with `system-tray` and `icon-png` features

### TypeScript/React
- `@tauri-apps/api` - Tauri IPC (already in project)
- `framer-motion` - Animations (already in project)
- `lucide-react` - Icons (already in project)

## Future Enhancements

Potential improvements:
1. Custom tray icon images per style
2. More tray menu customization options
3. Tray-only mode (no main window until opened)
4. Multiple tray icon states (idle, active, alert)
5. Tray notification history
6. Per-OS startup optimizations

## Testing Strategy

1. **Unit Tests**: Test component rendering and state management
2. **Integration Tests**: Test Tauri command invocations
3. **Manual Tests**: Test actual tray interactions and auto-start behavior
   - Minimize to tray and restore
   - Close to tray and restore
   - Global shortcut restoration
   - Auto-start on system boot
   - Startup delay timing
   - Start minimized behavior

## Documentation

All tray and auto-start settings are documented within the UI with:
- Tooltips explaining each option
- Platform-specific notes where relevant
- Keyboard shortcut hints
- Quick action buttons for common tasks
