# Pro Chart Analytics Guide

This guide covers the advanced charting features implemented in Phase 3 (Tasks 3.21-3.24).

## Features

### 1. Volume Profile (Task 3.21)

Volume Profile displays trading activity over price levels to identify key support/resistance zones.

**Components:**
- `VolumeProfile.tsx` - Visual rendering with horizontal bars
- `volumeProfile.ts` - Calculation utilities

**Key Metrics:**
- **POC (Point of Control)** - Price level with highest volume
- **Value Area** - 70% of total volume distribution
- **VWAP** - Volume-weighted average price
- **VWAP Bands** - Standard deviation bands (2σ)
- **Volume Delta** - Difference between buy and sell volume

**Usage:**
```tsx
import { VolumeProfile } from './components/charts/VolumeProfile';

<VolumeProfile
  candles={candleData}
  width={200}
  height={600}
  showPOC={true}
  showValueArea={true}
  showVWAP={true}
  numLevels={50}
/>
```

### 2. Order Book Depth (Task 3.22)

Real-time order book visualization with depth heatmap and trade recommendation.

**Components:**
- `OrderBookDepth.tsx` - Enhanced order book with depth visualization
- `orderBook.ts` - Depth calculations and imbalance metrics

**Features:**
- Area-based depth visualization
- Bid/ask imbalance ratio
- Spread indicators
- Quick trade recommendations based on imbalance

**Usage:**
```tsx
import { OrderBookDepth } from './components/charts/OrderBookDepth';

<OrderBookDepth
  bids={bidOrders}
  asks={askOrders}
  height={400}
  onQuickTrade={side => handleTrade(side)}
/>
```

### 3. Multi-Chart Layout (Task 3.23)

Display and synchronize up to 9 charts simultaneously.

**Components:**
- `MultiChartLayout.tsx` - Main layout manager
- `ChartPanel.tsx` - Individual chart instances
- `useMultiChart.ts` - State management hook

**Features:**
- Predefined layouts: Single, 2x2, 3x3
- Custom layouts (up to 9 panels)
- Synchronized crosshair
- Synchronized timeframes
- Layout persistence (localStorage)
- Per-panel configuration

**Default Layouts:**
- **Single** - 1x1 grid, no sync
- **2x2 Grid** - 4 charts with sync enabled
- **3x3 Grid** - 9 charts with sync enabled

**Creating Custom Layouts:**
```tsx
const { createLayout } = useMultiChart();
createLayout('My Layout', 2, 3); // 2 rows, 3 cols
```

**Syncing:**
- Enable `syncCrosshair` to synchronize crosshair across all panels
- Enable `syncTimeframe` to apply global timeframe to all panels

### 4. Custom Indicator Builder (Task 3.24)

Visual drag-drop interface for building custom technical indicators.

**Components:**
- `CustomIndicatorBuilder.tsx` - Builder UI
- `indicatorEngine.ts` - Formula evaluation engine
- `useCustomIndicators.ts` - Indicator management hook
- `indicator-calculator.worker.ts` - Web Worker for heavy calculations

**Node Types:**
- **Constant** - Fixed numeric value
- **Indicator** - Built-in indicator (SMA, EMA, RSI, Volume)
- **Operator** - Arithmetic (+, -, *, /)
- **Condition** - Logical comparison (>, <, ==, &&, ||)

**Built-in Indicators:**
- SMA (Simple Moving Average)
- EMA (Exponential Moving Average)
- RSI (Relative Strength Index)
- Volume

**Example: Creating a Custom Indicator**

1. Open Custom Indicator Builder
2. Add an EMA node with period 20
3. Add a constant node with value 50
4. Add an operator node (>) with inputs from EMA and constant
5. Test the indicator on historical data
6. Save and export

**Backtesting:**
The indicator engine includes simple backtesting to evaluate indicator performance:
- Total trades
- Profitable trades
- Total return
- Max drawdown
- Sharpe ratio (simplified)

**Import/Export:**
```tsx
const { exportIndicators, importIndicators } = useCustomIndicators();

// Export all indicators
const json = exportIndicators();

// Import from JSON
importIndicators(jsonString);
```

## Performance Optimizations

### Web Workers
Heavy computations (indicator evaluation, volume profile calculation) run in a Web Worker to prevent UI blocking:

```typescript
// Automatically used by useCustomIndicators
const result = await evaluateIndicators(candles);
```

### GPU Acceleration
Volume profile and order book depth use Canvas/SVG rendering for optimal performance with large datasets.

### Caching
The indicator engine caches results to avoid redundant calculations:

```typescript
indicatorEngine.clearCache(); // Clear when needed
```

## Testing

Tests are provided for critical functionality:

### Indicator Engine Tests
```bash
npm test src/__tests__/indicatorEngine.test.ts
```

Tests cover:
- SMA, EMA, RSI calculations
- Operator nodes (arithmetic)
- Condition nodes (logical)
- Backtesting
- Caching

### Multi-Chart Tests
```bash
npm test src/__tests__/multiChart.test.ts
```

Tests cover:
- Layout creation/deletion
- Panel updates
- Crosshair synchronization
- Timeframe synchronization
- Persistence (localStorage)

## Navigation

Access Pro Charts from the main navigation menu:
- **Pro Charts** menu item added to App.tsx
- Located between Trading and API Health

## Storage

### localStorage Keys:
- `multi_chart_layouts` - Saved chart layouts
- `custom_indicators_v2` - Custom indicator definitions
- `realtime_chart_settings` - Chart streaming settings

## Community Features (Planned)

Future enhancements include:
- Community indicator marketplace
- Shared layouts
- Indicator templates library
- Performance leaderboards

## Tips

1. **Volume Profile**: Use POC as dynamic support/resistance
2. **Order Book**: Watch for large imbalances before breakouts
3. **Multi-Chart**: Sync timeframes to spot divergences
4. **Custom Indicators**: Start simple, test thoroughly before live use

## Architecture

```
src/
├── components/charts/
│   ├── VolumeProfile.tsx
│   ├── OrderBookDepth.tsx
│   ├── MultiChartLayout.tsx
│   ├── ChartPanel.tsx
│   └── CustomIndicatorBuilder.tsx
├── hooks/
│   ├── useMultiChart.ts
│   └── useCustomIndicators.ts
├── utils/
│   ├── volumeProfile.ts
│   ├── orderBook.ts
│   └── indicatorEngine.ts
├── workers/
│   └── indicator-calculator.worker.ts
├── types/
│   └── indicators.ts
└── pages/
    └── ProCharts.tsx
```

## Dependencies

- `lightweight-charts` - Professional charting library
- `recharts` - Additional chart rendering
- `framer-motion` - Animations
- `zustand` - State management (existing)

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

Web Workers and Canvas API required.
