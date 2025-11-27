# Indicators & Drawings Implementation Guide

This guide describes the implementation of Phase 3 Tasks 3.19-3.20: Advanced chart indicators and drawing tools.

## Overview

The application now supports:
- **50+ technical indicators** with configurable parameters
- **15+ drawing tools** with templates and persistence
- **Indicator-based alerts** integrated with the notification system
- **Cross-device synchronization** for drawings
- **Preset management** for saving and loading indicator configurations

## Components

### Indicator Engine

#### Core Files
- `src/utils/indicators.ts` - Indicator calculation library
- `src/types/indicators.ts` - Type definitions for indicators
- `src/store/indicatorStore.ts` - State management for indicators
- `src/components/charts/IndicatorPanel.tsx` - UI for managing indicators
- `src/hooks/useIndicatorAlerts.ts` - Alert integration

#### Supported Indicators

**Trend Indicators:**
- Simple Moving Average (SMA)
- Exponential Moving Average (EMA)
- MACD (Moving Average Convergence Divergence)
- Parabolic SAR
- Ichimoku Cloud

**Momentum Indicators:**
- RSI (Relative Strength Index)
- Stochastic Oscillator
- CCI (Commodity Channel Index)
- Williams %R
- MFI (Money Flow Index)

**Volatility Indicators:**
- Bollinger Bands
- ATR (Average True Range)
- Keltner Channels
- Donchian Channels

**Volume Indicators:**
- OBV (On-Balance Volume)
- VWAP (Volume Weighted Average Price)

#### Usage Example

```typescript
import { useIndicatorStore } from './store/indicatorStore';
import { calculateRSI } from './utils/indicators';

function MyComponent() {
  const { addIndicator, indicators } = useIndicatorStore();

  // Add RSI indicator
  const handleAddRSI = () => {
    addIndicator('RSI', 'separate'); // Display in separate panel
  };

  // Calculate RSI values
  const closePrices = [100, 102, 98, 105, 110, ...];
  const rsiValues = calculateRSI(closePrices, 14);

  return (
    <button onClick={handleAddRSI}>Add RSI</button>
  );
}
```

### Drawing Tools

#### Core Files
- `src/types/drawings.ts` - Type definitions for drawings
- `src/store/drawingStore.ts` - State management for drawings
- `src/components/charts/DrawingToolbar.tsx` - UI for drawing tools

#### Supported Drawing Tools

**Lines:**
- Trendline
- Horizontal line
- Vertical line

**Fibonacci Tools:**
- Fibonacci Retracement
- Fibonacci Time Zones
- Gann Fan

**Shapes:**
- Rectangle
- Ellipse
- Triangle

**Advanced:**
- Channel
- Pitchfork
- Path/Brush
- Arrow
- Text annotation

#### Usage Example

```typescript
import { useDrawingStore } from './store/drawingStore';

function ChartComponent() {
  const { addDrawing, activeTool, setActiveTool } = useDrawingStore();

  const handleDrawingComplete = (points) => {
    addDrawing({
      userId: currentUser.id,
      symbol: 'SOL',
      tool: activeTool,
      points,
      style: activeStyle,
      locked: false,
      hidden: false,
    });
  };

  return (
    <div>
      <button onClick={() => setActiveTool('trendline')}>
        Draw Trendline
      </button>
      {/* Chart with drawing overlay */}
    </div>
  );
}
```

### Advanced Trading Chart

The `AdvancedTradingChart` component integrates both indicators and drawings:

```typescript
import AdvancedTradingChart from './components/charts/AdvancedTradingChart';

function TradingPage() {
  const priceData = [
    { timestamp: 1, open: 100, high: 105, low: 98, close: 103, volume: 1000 },
    // ... more data
  ];

  return (
    <AdvancedTradingChart
      symbol="SOL"
      data={priceData}
      height={600}
    />
  );
}
```

## Backend (Rust)

### Indicator Persistence

The indicator manager handles saving/loading indicator configurations and presets:

```rust
// src-tauri/src/indicators.rs
pub struct IndicatorManager {
    indicators_path: PathBuf,
    presets_path: PathBuf,
    alerts_path: PathBuf,
}
```

**Tauri Commands:**
- `indicator_save_state` - Save current indicators
- `indicator_load_state` - Load saved indicators
- `indicator_list_presets` - List saved presets
- `indicator_save_preset` - Save new preset
- `indicator_delete_preset` - Delete preset
- `indicator_list_alerts` - List indicator alerts
- `indicator_create_alert` - Create new alert
- `indicator_delete_alert` - Delete alert

### Drawing Persistence

The drawing manager handles per-symbol drawing storage:

```rust
// src-tauri/src/drawings.rs
pub struct DrawingManager {
    drawings_path: PathBuf,
    templates_path: PathBuf,
}
```

**Tauri Commands:**
- `drawing_list` - List drawings for a symbol
- `drawing_save` - Save drawings for a symbol
- `drawing_sync` - Sync drawings (cross-device)
- `drawing_list_templates` - List saved templates
- `drawing_save_templates` - Save templates

## Indicator Alerts

Indicator alerts integrate with the existing notification system:

```typescript
import { useIndicatorAlerts } from './hooks/useIndicatorAlerts';

function ChartWithAlerts() {
  const indicatorValues = calculateAllIndicators(priceData);
  
  // Automatically monitors and triggers alerts
  useIndicatorAlerts(indicatorValues);
  
  return <Chart />;
}
```

### Alert Configuration

```typescript
const { addAlert } = useIndicatorStore();

// Create RSI alert
await addAlert(
  rsiIndicatorId,
  'below',       // condition: 'above' | 'below' | 'crosses_above' | 'crosses_below'
  30,            // threshold
);
```

## Presets

Save and load complete indicator setups:

```typescript
const { savePreset, loadPreset, presets } = useIndicatorStore();

// Save current configuration
await savePreset('My Day Trading Setup', 'RSI + MACD + Bollinger Bands');

// Load preset
loadPreset(presets[0].id);
```

## Templates

Save and reuse drawing templates:

```typescript
const { addTemplate, templates } = useDrawingStore();

// Save template
addTemplate({
  name: 'Support Level',
  tool: 'horizontal',
  style: { strokeColor: '#10b981', strokeWidth: 2 },
  defaultPoints: [{ x: 0, y: 100 }],
});
```

## Testing

### Indicator Calculations

Tests ensure accuracy of technical indicators:

```bash
npm test -- indicators.test.ts
```

### Drawing Persistence

Tests verify drawing state management:

```bash
npm test -- drawings.test.ts
```

## File Structure

```
src/
├── components/
│   └── charts/
│       ├── AdvancedTradingChart.tsx
│       ├── IndicatorPanel.tsx
│       └── DrawingToolbar.tsx
├── store/
│   ├── indicatorStore.ts
│   └── drawingStore.ts
├── types/
│   ├── indicators.ts
│   └── drawings.ts
├── utils/
│   ├── indicators.ts
│   └── __tests__/
│       └── indicators.test.ts
└── hooks/
    └── useIndicatorAlerts.ts

src-tauri/src/
├── indicators.rs
└── drawings.rs

tests/
└── drawings.test.ts
```

## Key Features

### 1. Configurable Parameters
Each indicator supports customizable periods, colors, and display options.

### 2. Overlay vs Separate Panels
Indicators can be displayed either:
- **Overlay**: Drawn on top of the price chart (e.g., SMA, Bollinger Bands)
- **Separate**: In their own panel below the chart (e.g., RSI, MACD)

### 3. Signal Generation
Indicators automatically generate buy/sell signals based on:
- RSI oversold/overbought levels
- MACD crossovers
- Stochastic levels
- Williams %R extremes

### 4. Drawing Persistence
- Drawings are saved per symbol
- Locked drawings cannot be edited
- Hidden drawings remain in storage but aren't displayed
- Templates can be created from existing drawings

### 5. Cross-Device Sync
The `drawing_sync` command can be extended to synchronize drawings across devices using cloud storage or WebSocket.

### 6. Alert Integration
Indicator alerts trigger notifications through:
- In-app notifications
- Email (if configured)
- Webhooks
- Telegram/Discord (if configured)

## Performance Considerations

1. **Indicator Calculations**: Run on-demand when data changes
2. **Drawing Rendering**: Uses SVG overlay for efficient rendering
3. **State Management**: Zustand provides minimal re-renders
4. **Persistence**: Debounced writes to prevent excessive disk I/O

## Future Enhancements

- [ ] Custom indicator formulas
- [ ] Drawing collaboration (real-time sync)
- [ ] Indicator backtesting
- [ ] Advanced pattern recognition
- [ ] AI-powered indicator suggestions
- [ ] Mobile touch gestures for drawings
- [ ] Chart sharing with drawings embedded
- [ ] Drawing annotations with rich text
- [ ] Indicator strategy builder

## Troubleshooting

### Indicators not displaying
- Check if indicator is enabled in the store
- Verify sufficient data points for calculation
- Check console for calculation errors

### Drawings not persisting
- Ensure app has write permissions
- Check if drawing save was called after modifications
- Verify app data directory exists

### Alerts not triggering
- Confirm alert is enabled
- Check if indicator values are being calculated
- Verify notification system is configured

## API Reference

See type definitions in:
- `src/types/indicators.ts`
- `src/types/drawings.ts`

For calculation details, refer to:
- `src/utils/indicators.ts`
