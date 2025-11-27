# Workspace & Layout System Guide

## Overview

The Workspace & Layout system provides a flexible, customizable workspace management solution with support for multiple monitors, drag-and-drop panels, and persistent layout configurations.

## Features

### 1. Customizable Layouts
- **Drag & Drop**: Rearrange panels by dragging them to new positions
- **Resize**: Adjust panel dimensions to fit your needs
- **Minimize/Maximize**: Toggle panel visibility without closing them
- **Lock**: Prevent accidental moves or resizes for specific panels

### 2. Workspace Tabs
- **Unlimited Workspaces**: Create as many workspaces as needed
- **Tab Management**: 
  - Create new workspaces with `+` button
  - Rename workspaces by right-clicking and selecting "Rename"
  - Duplicate workspaces to clone layouts
  - Reorder tabs by dragging
  - Delete workspaces (minimum 1 required)
- **Unsaved Indicators**: Yellow dot shows unsaved layout changes
- **Context Menu**: Right-click tabs for quick actions

### 3. Keyboard Shortcuts
- `Cmd/Ctrl + K`: Open workspace switcher
- `Cmd/Ctrl + 1-9`: Quick switch to workspace 1-9
- `↑↓`: Navigate within workspace switcher
- `Enter`: Select workspace
- `Esc`: Close workspace switcher

### 4. Layout Presets
- **Trading Focus**: Optimized for active trading
- **Research**: Balanced view for analysis
- Custom presets can be created by saving current layouts

### 5. Multi-Monitor Support
- **Auto-Detection**: Detects monitor configuration (count, resolution, DPI)
- **Per-Monitor Layouts**: Layouts adapt to monitor setups
- **Persistence**: Layout configurations remember monitor-specific arrangements

### 6. Persistence
- **Local Storage**: Layouts saved automatically to browser local storage
- **Session Restore**: Workspaces restored on app reload
- **Unsaved Changes**: Track and save layout modifications

## Architecture

### Components

#### WorkspaceTabs (`src/components/workspace/WorkspaceTabs.tsx`)
- Renders workspace tabs with drag-to-reorder
- Handles tab context menu (rename, duplicate, delete)
- Shows unsaved indicators

#### WorkspaceSwitcher (`src/components/workspace/WorkspaceSwitcher.tsx`)
- Quick switcher modal with search
- Keyboard navigation support
- Shows active workspace and panel count

#### GridLayoutContainer (`src/components/workspace/GridLayoutContainer.tsx`)
- Wraps react-grid-layout
- Manages panel rendering and layout updates
- Responsive width handling

#### PanelWrapper (`src/components/workspace/PanelWrapper.tsx`)
- Individual panel container
- Provides minimize/maximize/lock/close controls
- Displays panel title and status

#### WorkspaceToolbar (`src/components/workspace/WorkspaceToolbar.tsx`)
- Save/reset controls
- Preset selector
- Monitor configuration display

### Store

**File**: `src/store/workspaceStore.ts`

**State**:
```typescript
interface WorkspaceState {
  workspaces: Workspace[];
  activeWorkspaceId: string;
  isWorkspaceSwitcherOpen: boolean;
  currentMonitorConfig: MonitorConfig | null;
  
  // Actions
  addWorkspace: (name?: string, layout?: WorkspaceLayout) => void;
  duplicateWorkspace: (workspaceId: string) => void;
  deleteWorkspace: (workspaceId: string) => void;
  renameWorkspace: (workspaceId: string, name: string) => void;
  setActiveWorkspace: (workspaceId: string) => void;
  reorderWorkspaces: (workspaceIds: string[]) => void;
  updateWorkspaceLayout: (workspaceId: string, layout: WorkspaceLayout) => void;
  // ... more actions
}
```

### Types

**File**: `src/types/workspace.ts`

Key types:
- `Workspace`: Complete workspace definition
- `Panel`: Individual panel metadata
- `PanelLayout`: Grid layout positioning for a panel
- `PanelType`: Available panel types (dashboard, coins, trading, etc.)
- `MonitorConfig`: Display configuration details
- `LayoutPreset`: Predefined layout templates

### Utilities

**File**: `src/utils/workspace.ts`

- `cloneWorkspaceLayout`: Deep clone layout data
- `createPanelDefinition`: Generate new panel with layout

### Hooks

**File**: `src/hooks/useMonitorConfig.ts`

- Detects monitor configuration via Tauri API
- Falls back to window.screen API
- Updates on window resize

## Usage

### Creating a Workspace

```typescript
import { useWorkspaceStore } from './store/workspaceStore';

const addWorkspace = useWorkspaceStore(state => state.addWorkspace);

// Create default workspace
addWorkspace('My Workspace');

// Create with custom layout
addWorkspace('Custom Layout', customLayoutConfig);
```

### Adding Panels

```typescript
import { useWorkspaceStore } from './store/workspaceStore';
import { createPanelDefinition } from './utils/workspace';

const addPanel = useWorkspaceStore(state => state.addPanel);
const activeWorkspaceId = useWorkspaceStore(state => state.activeWorkspaceId);

const { panel, layout } = createPanelDefinition('coins', 6, 8);
addPanel(activeWorkspaceId, panel, layout);
```

### Saving Layout

```typescript
const saveWorkspace = useWorkspaceStore(state => state.saveWorkspace);
const activeWorkspaceId = useWorkspaceStore(state => state.activeWorkspaceId);

saveWorkspace(activeWorkspaceId);
```

### Loading Preset

```typescript
const loadPreset = useWorkspaceStore(state => state.loadPreset);

loadPreset('trading-focus');
```

## Integration with Tauri

### Multi-Monitor Detection

The system uses Tauri's `availableMonitors()` API to detect multiple displays:

```typescript
import { availableMonitors } from '@tauri-apps/api/window';

const monitors = await availableMonitors();
const monitorConfig = {
  width: monitors[0].size.width,
  height: monitors[0].size.height,
  devicePixelRatio: monitors[0].scaleFactor,
  count: monitors.length,
};
```

Falls back to browser APIs if Tauri is unavailable.

## Testing

**File**: `src/__tests__/workspace.test.ts`

Tests cover:
- Workspace CRUD operations
- Panel management (add, remove, lock, minimize)
- Layout persistence
- Multi-monitor configuration
- Tab reordering

Run tests:
```bash
npm test -- workspace.test.ts
```

## Best Practices

1. **Save frequently**: Use the save button to persist layout changes
2. **Use presets**: Start with a preset and customize from there
3. **Lock panels**: Lock panels you don't want to accidentally move
4. **Minimize instead of close**: Minimize panels to preserve layout structure
5. **Name workspaces clearly**: Use descriptive names for easy identification

## Troubleshooting

### Layouts not persisting
- Check browser localStorage is enabled
- Verify `workspace-storage` key exists in localStorage
- Try clearing and recreating workspace

### Panels overlapping
- Use "Reset" button to restore default layout
- Manually resize panels to fix overlaps
- Load a preset to start fresh

### Keyboard shortcuts not working
- Ensure workspace switcher is in focus
- Check for conflicting browser shortcuts
- Verify shortcuts in WorkspaceSwitcher component

## Future Enhancements

- Cloud sync for workspace configurations
- Import/export workspace layouts
- Shared workspace templates
- Panel-specific settings and preferences
- Advanced grid snapping and alignment tools
