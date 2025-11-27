# Wallet Performance Scoring Implementation

## Overview
Comprehensive wallet performance scoring system that analyzes trading activity, calculates risk-adjusted performance metrics, and provides detailed analytics.

## Features

### 1. Performance Score (0-100)
- **Win Rate (30%)**: Percentage of profitable trades
- **Profit Factor (30%)**: Ratio of total profits to total losses
- **Sharpe Ratio (20%)**: Risk-adjusted returns, annualized
- **Consistency Score (20%)**: Based on return variance (coefficient of variation)

### 2. Trade Tracking
- Records all buy and sell transactions
- Automatically calculates P&L for sell trades by matching with previous buys
- Tracks hold duration for each position
- Stores transaction fees and signatures

### 3. Performance Analytics

#### Score History
- Historical tracking of performance scores over time
- Visualized with line charts
- Shows score evolution and trends

#### Token-Level Performance
- Per-token statistics:
  - Trade count
  - Win rate
  - Net P&L
  - Total volume
  - Average hold duration
  - Best/worst trades

#### Timing Analysis
- Performance by hour of day
- Performance by day of week
- Identifies optimal trading times
- Visualized with bar charts

#### Best/Worst Trades
- Top 5 most profitable trades
- Top 5 biggest losses
- Details: token, amount, price, P&L, hold time

### 4. Benchmark Comparison
- Compare wallet score to market average
- Percentile ranking
- Wallet rank among all tracked wallets
- Total number of tracked wallets

### 5. Alerts
- Automatic alerts on significant score changes (±10%)
- Alert history tracking
- Displayed prominently in the UI

## Database Schema

### trades table
```sql
CREATE TABLE trades (
    id TEXT PRIMARY KEY,
    wallet_address TEXT NOT NULL,
    token_mint TEXT NOT NULL,
    token_symbol TEXT NOT NULL,
    side TEXT NOT NULL,              -- 'buy' or 'sell'
    amount REAL NOT NULL,
    price REAL NOT NULL,
    total_value REAL NOT NULL,
    fee REAL NOT NULL,
    tx_signature TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    pnl REAL,                         -- Calculated for sells
    hold_duration_seconds INTEGER     -- Time between buy and sell
);
```

### performance_scores table
```sql
CREATE TABLE performance_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_address TEXT NOT NULL,
    score REAL NOT NULL,
    win_rate REAL NOT NULL,
    total_trades INTEGER NOT NULL,
    winning_trades INTEGER NOT NULL,
    losing_trades INTEGER NOT NULL,
    total_profit REAL NOT NULL,
    total_loss REAL NOT NULL,
    net_pnl REAL NOT NULL,
    avg_profit_per_trade REAL NOT NULL,
    avg_loss_per_trade REAL NOT NULL,
    profit_factor REAL NOT NULL,
    sharpe_ratio REAL NOT NULL,
    consistency_score REAL NOT NULL,
    avg_hold_duration_seconds REAL NOT NULL,
    best_trade_pnl REAL NOT NULL,
    worst_trade_pnl REAL NOT NULL,
    calculated_at TEXT NOT NULL
);
```

### score_alerts table
```sql
CREATE TABLE score_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_address TEXT NOT NULL,
    old_score REAL NOT NULL,
    new_score REAL NOT NULL,
    change_percent REAL NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL
);
```

## Backend API

### Tauri Commands

#### `record_trade`
Records a new trade transaction.
```typescript
interface RecordTradeRequest {
  walletAddress: string;
  tokenMint: string;
  tokenSymbol: string;
  side: 'buy' | 'sell';
  amount: number;
  price: number;
  fee: number;
  txSignature: string;
}

await invoke<Trade>('record_trade', { request });
```

#### `calculate_wallet_performance`
Calculates and saves the current performance score for a wallet.
```typescript
await invoke<PerformanceScore>('calculate_wallet_performance', {
  walletAddress: 'xxx'
});
```

#### `get_wallet_performance_data`
Retrieves complete performance data including score, history, token breakdown, etc.
```typescript
await invoke<WalletPerformanceData>('get_wallet_performance_data', {
  walletAddress: 'xxx'
});
```

#### `get_performance_score_history`
Gets historical score data.
```typescript
await invoke<PerformanceScore[]>('get_performance_score_history', {
  walletAddress: 'xxx',
  limit: 30
});
```

#### `get_token_performance_breakdown`
Retrieves per-token performance statistics.
```typescript
await invoke<TokenPerformance[]>('get_token_performance_breakdown', {
  walletAddress: 'xxx'
});
```

#### `get_timing_analysis_data`
Gets timing analysis data.
```typescript
await invoke<TimingAnalysis[]>('get_timing_analysis_data', {
  walletAddress: 'xxx'
});
```

#### `get_best_worst_trades_data`
Retrieves best and worst trades.
```typescript
await invoke<BestWorstTrades>('get_best_worst_trades_data', {
  walletAddress: 'xxx',
  limit: 5
});
```

#### `get_benchmark_comparison_data`
Gets benchmark comparison data.
```typescript
await invoke<BenchmarkComparison | null>('get_benchmark_comparison_data', {
  walletAddress: 'xxx'
});
```

#### `get_performance_alerts`
Retrieves recent score alerts.
```typescript
await invoke<ScoreAlert[]>('get_performance_alerts', {
  walletAddress: 'xxx',
  limit: 10
});
```

## Frontend Components

### WalletPerformanceDashboard
Full-featured dashboard displaying all performance metrics and analytics.

**Props:**
- `walletAddress: string` - Wallet address to display performance for

**Features:**
- Score badge with color coding
- Key metrics cards (Win Rate, Net P&L, Profit Factor, Sharpe Ratio)
- Trade statistics breakdown
- Benchmark comparison (if available)
- Tabbed interface for different views:
  - Score History (line chart)
  - Token Performance (table)
  - Timing Analysis (bar charts)
  - Best/Worst Trades (list)

### PerformanceScoreBadge
Displays the performance score as a color-coded badge.

**Props:**
- `score: number` - Performance score (0-100)
- `size?: 'sm' | 'md' | 'lg'` - Badge size
- `showLabel?: boolean` - Show text label (Excellent, Good, etc.)
- `previousScore?: number` - Show change from previous score

**Color Coding:**
- 80-100: Green (Excellent)
- 60-79: Blue (Good)
- 40-59: Yellow (Fair)
- 20-39: Orange (Poor)
- 0-19: Red (Very Poor)

### WalletPerformanceCard
Compact card for displaying performance summary.

**Props:**
- `walletAddress: string` - Wallet address
- `onViewDetails?: () => void` - Callback when user clicks to view details

**Displays:**
- Current score badge
- Win rate
- Net P&L
- Total trades
- Quick stats (winning/losing trades)

## Integration Guide

### 1. Add Trade Recording
Whenever a trade is executed, record it:
```typescript
await invoke('record_trade', {
  request: {
    walletAddress: wallet.publicKey,
    tokenMint: 'token_mint_address',
    tokenSymbol: 'TOKEN',
    side: 'buy', // or 'sell'
    amount: 100.5,
    price: 1.23,
    fee: 0.001,
    txSignature: 'signature_hash'
  }
});
```

### 2. Display Performance in Wallet Switcher
Add the performance card to the wallet dropdown:
```typescript
import { WalletPerformanceCard } from './WalletPerformanceCard';

<WalletPerformanceCard
  walletAddress={wallet.publicKey}
  onViewDetails={() => setCurrentPage('wallet-performance')}
/>
```

### 3. Add Page to Navigation
In App.tsx, add to the pages array:
```typescript
{
  id: 'wallet-performance',
  label: 'Performance',
  icon: BarChart3,
  component: WalletPerformance
}
```

## Scoring Algorithm Details

### Win Rate
```
win_rate = (winning_trades / total_trades) * 100
```

### Profit Factor
```
profit_factor = total_profit / abs(total_loss)
```
- Capped at 10.0 when no losses exist

### Sharpe Ratio
```
sharpe_ratio = (mean_return / std_dev_return) * sqrt(252)
```
- Annualized assuming 252 trading days
- Clamped between -5.0 and 5.0

### Consistency Score
```
coefficient_of_variation = std_dev / abs(mean)
consistency_score = 100 / (1 + coefficient_of_variation)
```
- Higher score = more consistent returns

### Overall Score
```
score = 
  (win_rate / 100) * 30 +
  min(profit_factor / 3, 1) * 30 +
  normalize(sharpe_ratio, -5, 5) * 20 +
  (consistency_score / 100) * 20
```
- Result is clamped between 0 and 100

## Testing

Test file: `src-tauri/tests/performance_tests.rs`

### Test Coverage
- Trade recording and retrieval
- P&L calculation
- Hold duration calculation
- Win rate calculation
- Sharpe ratio calculation
- Consistency score calculation
- Overall score calculation
- Token performance aggregation
- Timing analysis
- Benchmark comparison
- Score alert creation
- Best/worst trades retrieval

### Running Tests
```bash
cd src-tauri
cargo test performance_tests
```

## Performance Considerations

### Database Indexes
- Indexed on: wallet_address, token_mint, timestamp
- Enables fast queries for per-wallet and per-token analytics

### Calculation Optimization
- Score calculation only runs when explicitly requested
- Historical scores are cached in the database
- Incremental updates possible by storing intermediate calculations

### UI Updates
- Performance data refreshed on component mount
- Manual refresh button available
- Automatic refresh on new trades (can be implemented via events)

## Future Enhancements

1. **Risk Metrics**
   - Maximum drawdown
   - Value at Risk (VaR)
   - Sortino ratio

2. **Advanced Analytics**
   - Moving averages of scores
   - Volatility metrics
   - Correlation analysis

3. **Social Features**
   - Leaderboards
   - Performance comparison with friends
   - Strategy sharing

4. **Alerts**
   - Customizable alert thresholds
   - Push notifications
   - Email/SMS alerts

5. **Export**
   - CSV/Excel export
   - PDF reports
   - Tax reporting integration
