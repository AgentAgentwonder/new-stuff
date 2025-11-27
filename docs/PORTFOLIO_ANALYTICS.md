# Portfolio Analytics Documentation

## Overview

The Portfolio Analytics module provides comprehensive portfolio management, rebalancing, tax lot tracking, and trading calculators as specified in Phase 2 Tasks 2.13-2.17.

## Features

### 1. Portfolio Overview (`/portfolio`)

The portfolio page displays:

- **Total Portfolio Value**: Current market value of all positions
- **Realized P&L**: Cumulative profits/losses from closed positions
- **Unrealized P&L**: Current paper profits/losses from open positions
- **Time-based P&L**: Daily, weekly, monthly, and all-time performance metrics
- **Allocation Charts**: Visual breakdown of portfolio distribution (pie chart)
- **P&L by Position**: Bar chart showing profit/loss for each asset
- **Position Table**: Sortable table with:
  - Symbol and contract address
  - Amount held
  - Current price and average entry price
  - Position value
  - Unrealized P&L (dollar amount and percentage)
  - Portfolio allocation percentage

**Refresh Timestamp**: Automatically updates every 30 seconds or manually via refresh button.

### 2. Trading Calculators

#### Position Size Calculator

Located in the Trading page, this calculator helps determine optimal position sizing based on risk parameters.

**Features**:
- **Risk Profiles**: Pre-configured conservative, moderate, and aggressive profiles
- **Custom Configuration**: Account size, risk percentage, entry/stop loss prices, leverage
- **Kelly Criterion**: Optional optimization based on win rate and risk/reward ratio
- **Results**:
  - Position size (units)
  - Position value (USD)
  - Risk amount (USD)
  - Kelly fraction (when enabled)
- **Integration**: One-click "Apply to Order Form" button

**Formulas**:
```
Position Size = (Account Size × Risk%) / (Entry Price - Stop Loss Price) × Leverage
Kelly Fraction = (Win Rate × Avg Win/Loss - (1 - Win Rate)) / Avg Win/Loss
```

#### Risk/Reward Calculator

Evaluates trade quality and expected profitability.

**Features**:
- Entry price, stop loss, take profit, and position size inputs
- Win rate percentage for expected value calculation
- **Results**:
  - Risk amount (potential loss)
  - Reward amount (potential profit)
  - Risk/reward ratio
  - Break-even win rate
  - Expected value
- **Integration**: One-click "Apply to Order Form" button

**Formulas**:
```
Risk/Reward Ratio = (Take Profit - Entry) / (Entry - Stop Loss)
Break-even Win Rate = 1 / (Risk/Reward Ratio + 1)
Expected Value = Win Rate × Reward - (1 - Win Rate) × Risk
```

### 3. Auto-Rebalancer

The rebalancer automatically maintains target allocations across your portfolio.

**Features**:
- **Rebalance Profiles**: Create multiple profiles with different allocation targets
- **Trigger Types**:
  - **Deviation-based**: Triggers when allocation deviates beyond threshold (e.g., 5%)
  - **Time-based**: Triggers at regular intervals (e.g., weekly)
- **Dry-run Mode**: Preview rebalancing actions before execution
- **Action History**: Log of all rebalancing events with timestamps
- **Notifications**: Alerts when rebalancing is recommended

**Tauri Commands**:
```typescript
// List all rebalance profiles
list_rebalance_profiles()

// Save a rebalance profile
save_rebalance_profile({
  id?: string,
  name: string,
  targets: Array<{ symbol: string, targetPercent: number }>,
  deviationTriggerPercent: number,
  timeIntervalHours?: number,
  enabled: boolean
})

// Preview rebalance actions
preview_rebalance(profileId: string)

// Execute rebalance (dry_run = true for preview)
execute_rebalance(profileId: string, dry_run: boolean)

// Get rebalance history
get_rebalance_history()

// Check if any triggers are met
check_rebalance_triggers()
```

### 4. Tax Lot Tracking

Comprehensive tax reporting and optimization features.

**Features**:
- **Lot Strategies**: FIFO, LIFO, HIFO, or specific lot identification
- **Cost Basis Tracking**: Accurate per-lot cost basis calculation
- **Realized Gains**: Automatic calculation on lot disposal
- **Short-term vs Long-term**: 365-day threshold for capital gains classification
- **Tax Loss Harvesting**: AI-powered suggestions for realizing losses
  - Potential tax savings calculation
  - Days held information
  - Sorted by tax benefit
- **Export Formats**:
  - TurboTax (CSV)
  - CoinTracker (CSV)
  - Generic CSV

**Tauri Commands**:
```typescript
// Get all tax lots (including disposed)
get_tax_lots()

// Get open (undisposed) tax lots
get_open_tax_lots()

// Set lot selection strategy
set_tax_lot_strategy(strategy: 'FIFO' | 'LIFO' | 'HIFO' | 'SPECIFIC')

// Get current strategy
get_tax_lot_strategy()

// Dispose a lot
dispose_tax_lot({
  lotId: string,
  amount: number,
  salePrice: number
})

// Generate tax report for a year
generate_tax_report({ taxYear: number })

// Export tax report
export_tax_report({ taxYear: number }, format: 'turbotax' | 'cointracker' | 'csv')

// Get tax loss harvesting suggestions
get_tax_loss_harvesting_suggestions()
```

### 5. Order Form Integration

Calculators integrate seamlessly with the order form:

1. Configure calculator parameters
2. Click "Apply to Order Form"
3. Order form auto-populates with calculated values
4. Visual notification shows calculator suggestion applied
5. Review and submit order

**Store Integration**:
Uses `useOrderFormSuggestionStore` to communicate between calculators and order form.

## Testing

### Unit Tests (TypeScript)

Located in `src/__tests__/calculators.test.ts` and `src/__tests__/portfolioAnalytics.test.tsx`:

```bash
npm run test
```

**Calculator Tests** (`calculators.test.ts`):
- Position size calculation
- Kelly Criterion formula
- Leverage handling
- Risk/reward ratio calculation
- Break-even win rate
- Expected value (positive and negative)
- Rebalancing deviation detection
- Rebalance amount calculation
- Tax lot cost basis (FIFO)
- Realized gain calculation
- Short-term vs long-term classification
- Tax loss harvesting savings

**Portfolio Analytics Tests** (`portfolioAnalytics.test.tsx`):
- Analytics page rendering and loading states
- Correlation matrix display and calculations
- Diversification metrics visualization
- Sharpe ratio calculations and status indicators
- Factor analysis display
- Concentration alerts rendering and dismissal
- Sector allocation breakdown
- Export functionality
- Cache clearing and refresh behavior
- Error handling for API failures
- Empty portfolio state handling
- Concentration risk level categorization

### Integration Tests (Rust)

Located in `src-tauri/src/portfolio/analytics.rs` and `src-tauri/src/portfolio/rebalancer.rs`:

```bash
cd src-tauri && cargo test --lib
```

**Analytics Module Tests**:
- `test_calculate_returns`: Verifies return calculation from price series
- `test_mean_and_variance`: Statistical functions accuracy
- `test_correlation`: Pearson correlation coefficient calculation
- `test_correlation_matrix`: Pairwise correlation matrix generation
- `test_diversification_score`: Diversification metrics computation
- `test_risk_concentration`: Position concentration classification
- `test_sharpe_ratio`: Risk-adjusted performance metrics
- `test_concentration_alerts`: Alert generation for over-concentrated positions
- `test_sector_classification`: Automatic sector categorization
- `test_sector_allocation`: Sector-wise position aggregation
- `test_cache_operations`: Cache storage and retrieval with TTL

**Rebalancer Module Tests**:
- Rebalancer action detection
- Portfolio allocation adjustment
- Deviation trigger detection
- Tax lot disposal
- Tax report generation (short/long-term separation)
- Tax loss harvesting detection

### UI Snapshot Tests

UI components can be snapshot tested using:

```bash
npm run test -- --update-snapshot
```

Components with snapshot tests:
- `CorrelationHeatmap`: Heatmap rendering and color scheme
- `SectorAllocationChart`: Pie chart and sector breakdown
- `RiskDiversificationSummary`: Metric cards and indicators
- `ConcentrationAlerts`: Alert cards and severity badges

## Data Models

### Position
```typescript
{
  symbol: string
  mint: string
  amount: number
  currentPrice: number
  avgEntryPrice: number
  totalValue: number
  unrealizedPnl: number
  unrealizedPnlPercent: number
  allocation: number
}
```

### PortfolioMetrics
```typescript
{
  totalValue: number
  dailyPnl: number
  dailyPnlPercent: number
  weeklyPnl: number
  weeklyPnlPercent: number
  monthlyPnl: number
  monthlyPnlPercent: number
  allTimePnl: number
  allTimePnlPercent: number
  realizedPnl: number
  unrealizedPnl: number
  lastUpdated: string
}
```

### TaxLot
```typescript
{
  id: string
  symbol: string
  mint: string
  amount: number
  costBasis: number
  pricePerUnit: number
  acquiredAt: string
  disposedAmount?: number
  disposedAt?: string
  realizedGain?: number
}
```

### TaxReport
```typescript
{
  taxYear: number
  lots: TaxLot[]
  totalRealizedGains: number
  totalRealizedLosses: number
  netGainLoss: number
  shortTermGains: number
  longTermGains: number
  strategy: 'FIFO' | 'LIFO' | 'HIFO' | 'SPECIFIC'
  generatedAt: string
}
```

## Architecture

### Frontend Structure
```
src/
├── pages/
│   └── Portfolio.tsx           # Main portfolio dashboard
├── components/
│   └── trading/
│       ├── PositionSizeCalculator.tsx
│       ├── RiskRewardCalculator.tsx
│       └── OrderForm.tsx       # Enhanced with calculator integration
├── store/
│   └── orderFormSuggestionStore.ts  # Calculator-to-form communication
└── types/
    └── portfolio.ts            # TypeScript types
```

### Backend Structure
```
src-tauri/src/
└── portfolio/
    ├── mod.rs                  # Module exports
    ├── types.rs                # Rust types
    ├── rebalancer.rs           # Auto-rebalancing logic
    └── tax_lots.rs             # Tax tracking and reporting
```

## Usage Examples

### 1. Checking Rebalance Triggers
```typescript
const triggers = await invoke('check_rebalance_triggers');
if (triggers.length > 0) {
  // Show notification: "Rebalancing recommended"
}
```

### 2. Creating a Rebalance Profile
```typescript
await invoke('save_rebalance_profile', {
  input: {
    name: "60/30/10 Strategy",
    targets: [
      { symbol: "SOL", targetPercent: 60 },
      { symbol: "BTC", targetPercent: 30 },
      { symbol: "USDC", targetPercent: 10 }
    ],
    deviationTriggerPercent: 5,
    timeIntervalHours: 168, // Weekly
    enabled: true
  }
});
```

### 3. Generating Tax Report
```typescript
const report = await invoke('generate_tax_report', {
  params: { taxYear: 2024 }
});

console.log(`Net Gain/Loss: $${report.netGainLoss}`);
console.log(`Short-term: $${report.shortTermGains}`);
console.log(`Long-term: $${report.longTermGains}`);
```

### 4. Exporting for TurboTax
```typescript
const csv = await invoke('export_tax_report', {
  params: { taxYear: 2024 },
  format: 'turbotax'
});

// Save to file or copy to clipboard
downloadFile('tax_report_2024.csv', csv);
```

### 5. Tax Loss Harvesting
```typescript
const suggestions = await invoke('get_tax_loss_harvesting_suggestions');

suggestions.forEach(s => {
  console.log(`${s.symbol}: Realize $${s.unrealizedLoss} loss`);
  console.log(`  Potential savings: $${s.potentialTaxSavings}`);
  console.log(`  Days held: ${s.daysHeld}`);
});
```

## Performance Considerations

- Portfolio metrics recalculate automatically when positions change
- Allocation percentages are computed in real-time
- Rebalance trigger checks are optimized to run every 10 minutes (configurable)
- Tax lot queries use efficient filtering to avoid full table scans
- History logs are limited to the most recent 100 entries

### 6. Advanced Portfolio Analytics (`/portfolio-analytics`)

The advanced analytics page provides comprehensive portfolio analysis with real-time risk metrics, diversification scoring, and correlation analysis.

**Features**:
- **Correlation Matrix Heatmap**: Visual representation of pairwise asset correlations
  - Color-coded heat map (red = strong positive, blue = strong negative)
  - Interactive hover for detailed correlation values
  - Automatic calculation using historical price data
  
- **Diversification Metrics**:
  - Diversification Score (0-100): Overall portfolio diversity measure
  - Effective N: Number of truly independent positions
  - Average Correlation: Mean pairwise correlation across portfolio
  - Concentration Risk (Herfindahl Index): Position concentration measure

- **Risk-Adjusted Performance**:
  - **Sharpe Ratio**: Risk-adjusted returns calculation
  - Annualized return and volatility metrics
  - Risk-free rate configurable (default: 3%)
  - Performance status indicators (Excellent, Good, Fair, Caution, Negative)

- **Factor Analysis**:
  - Market Beta: Portfolio sensitivity to market movements
  - Factor Exposures: Market, Size, Momentum factors
  - Systematic vs. Specific Risk decomposition
  - Individual factor beta coefficients

- **Sector Allocation**:
  - Automatic sector classification (Layer 1, DeFi, Meme, Stablecoin, etc.)
  - Sector-wise allocation breakdown with pie chart
  - Symbol groupings by sector
  - Dollar value allocation per sector

- **Concentration Risk Alerts**:
  - Real-time alerts for over-concentrated positions
  - Four severity levels: Low, Medium, High, Critical
  - Threshold-based warnings (30% and 40% allocation)
  - Actionable recommendations for rebalancing
  - Dismissible alert notifications

**Tauri Commands**:
```typescript
// Calculate comprehensive portfolio analytics
calculate_portfolio_analytics({
  positions: Position[],
  timeSeries: Record<string, PricePoint[]>,
  riskFreeRate?: number
}): Promise<PortfolioAnalytics>

// Get concentration risk alerts
get_concentration_alerts({
  positions: Position[]
}): Promise<ConcentrationAlert[]>

// Get sector allocation breakdown
get_sector_allocation({
  positions: Position[]
}): Promise<SectorAllocation[]>

// Clear analytics cache (force recalculation)
clear_portfolio_cache(): Promise<void>
```

**Analytics Data Models**:
```typescript
interface PortfolioAnalytics {
  correlation: CorrelationMatrix
  diversification: DiversificationMetrics
  concentration: RiskConcentration[]
  sharpe: SharpeMetrics
  factors: FactorAnalysis
  calculatedAt: string
}

interface CorrelationMatrix {
  symbols: string[]
  matrix: number[][]  // NxN correlation matrix
  calculatedAt: string
}

interface DiversificationMetrics {
  score: number              // 0-100
  effectiveN: number         // Effective number of positions
  avgCorrelation: number     // Mean pairwise correlation
  concentrationRisk: number  // Herfindahl index
}

interface SharpeMetrics {
  sharpeRatio: number
  annualizedReturn: number
  annualizedVolatility: number
  riskFreeRate: number
}

interface ConcentrationAlert {
  id: string
  symbol: string
  allocation: number
  severity: 'warning' | 'critical'
  message: string
  threshold: number
  createdAt: string
}
```

**Caching**:
- Analytics calculations are cached for 5 minutes (300 seconds)
- Automatic cache invalidation on time expiry
- Manual cache clearing available via refresh button
- Cache key based on position symbols for efficient lookup

**Statistical Calculations**:
- **Correlation**: Pearson correlation coefficient using return series
- **Diversification Score**: `(Effective N / N) × (1 - |Avg Correlation|) × 100`
- **Effective N**: `1 / Herfindahl Index`
- **Sharpe Ratio**: `(Portfolio Return - Risk-free Rate) / Portfolio Volatility`
- **Factor Analysis**: Multi-factor regression model (Market, Size, Momentum)

**Export Functionality**:
- Export full analytics report as JSON
- Includes all metrics, alerts, and sector data
- Timestamped export files
- Compatible with portfolio tracking tools

## Future Enhancements

- Real-time price updates via WebSocket integration
- Value at Risk (VaR) and Conditional VaR calculations
- Maximum drawdown analysis with recovery period tracking
- Monte Carlo portfolio simulation
- Backtesting rebalance strategies with historical data
- Multi-account portfolio consolidation
- Integration with external portfolio trackers (Zapper, DeBank)
- Mobile-responsive portfolio dashboard
- CSV import for historical transactions
- Benchmark comparison (SPY, BTC, custom indices)
- Portfolio optimization using Modern Portfolio Theory
- Tax-aware rebalancing recommendations
- ESG (Environmental, Social, Governance) scoring

## Support

For issues or feature requests, please refer to the project's issue tracker.
