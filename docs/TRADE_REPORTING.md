# Trade Reporting & Export System

## Overview

The Trade Reporting & Export System provides comprehensive tools for managing, analyzing, and exporting trade history with advanced filtering, pagination, and automated scheduling capabilities.

## Features

### 1. Advanced Trade Filtering

Filter trades by multiple criteria simultaneously:

- **Date Range**: Filter trades between specific dates
- **Tokens**: Filter by source or destination tokens
- **Trade Side**: Filter by buy/sell operations
- **Status**: Filter by trade status (filled, pending, failed, cancelled)
- **P&L Range**: Filter by profit/loss amounts
- **Search**: Full-text search by transaction signature or token names
- **Wallet Address**: Filter by specific wallet
- **Trade Type**: Filter between real trades and paper trades

### 2. Trade Detail Modal

View comprehensive information about each trade:

- **Transaction Information**
  - Transaction signature with explorer links
  - Execution timestamp
  - Trade status with visual indicators

- **Execution Quality Metrics**
  - Overall execution quality rating (Excellent/Good/Fair/Poor)
  - Slippage percentage
  - Price impact
  - Expected vs. actual execution price

- **Financial Details**
  - Profit & Loss (absolute and percentage)
  - Gas costs
  - Priority fees
  - MEV protection savings

- **Explorer Integration**
  - Direct links to Solscan
  - Direct links to Solana Explorer

### 3. Export System

#### Supported Formats

- **CSV**: Comma-separated values for spreadsheet applications
- **XLSX**: Microsoft Excel format with formatted columns

#### Export Presets

1. **Tax Report**
   - Transaction signature
   - Date/time
   - Token pairs
   - Amounts
   - Prices
   - Gas costs
   - P&L
   - Wallet address

2. **Performance Analysis**
   - Execution quality metrics
   - P&L tracking
   - Slippage and price impact
   - Trade outcomes

3. **Trade Journal**
   - Comprehensive trade details
   - Execution metadata
   - MEV protection status
   - All fee information

4. **Custom**
   - Select any combination of columns
   - Full flexibility

#### Column Selection

Choose from 20+ data columns:
- Date/Time
- Transaction Signature
- Token Information
- Trade Side & Amount
- Execution Prices
- Slippage & Price Impact
- Fees (Gas, Priority)
- P&L Metrics
- Status & Quality
- MEV Protection
- Wallet Information

#### Timezone Support

Export data in your preferred timezone:
- UTC
- Local timezone
- Major world timezones (ET, PT, London, Tokyo, etc.)

### 4. Automated Export Scheduling

#### Schedule Frequencies

- **Daily**: Run exports once per day
- **Weekly**: Run exports once per week
- **Monthly**: Run exports once per month
- **Custom**: Define custom intervals in minutes

#### Delivery Methods

1. **Email**
   - Receive exports directly to your inbox
   - Configurable email address per schedule

2. **Webhook**
   - POST export data to custom endpoints
   - Integration with automation tools

#### Schedule Management

- Enable/disable schedules without deletion
- View last run timestamp
- See next scheduled run time
- Track execution history

### 5. Pagination & Sorting

- Configurable page sizes (5, 10, 25, 50 items per page)
- Sort by any column (timestamp, P&L, status, etc.)
- Sort order (ascending/descending)
- Visual pagination controls

## Usage

### Accessing Trade History

The Enhanced Trade History is available on the Trading page and includes all filtering, export, and scheduling capabilities.

### Creating a Manual Export

1. Click the **Export** button in the trade history header
2. Select export format (CSV or XLSX)
3. Choose a preset or configure custom columns
4. Optionally set date range filters
5. Select timezone
6. Click **Export** to download

### Setting Up Scheduled Exports

1. Click the **Schedule** button in the trade history header
2. Click **Create Schedule** or **Add New Schedule**
3. Configure schedule details:
   - Name your schedule
   - Choose frequency (daily/weekly/monthly/custom)
   - Select export preset and format
   - Choose delivery method (email/webhook)
   - Enter delivery details
4. Click **Create Schedule**
5. Schedule is automatically enabled and initialized

### Filtering Trades

1. Click the **Filters** button to show filter panel
2. Set any combination of filters
3. Active filter count is displayed
4. Click **Reset Filters** to clear all filters

### Viewing Trade Details

- Click any trade row to open the detailed modal
- View all trade information
- Click explorer links to view on blockchain explorers
- Close modal to return to list

## Data Model

### Enhanced Trade Metrics

```typescript
interface EnhancedTradeMetrics {
  id: string;
  timestamp: number;
  txSignature?: string;
  fromToken: string;
  toToken: string;
  side: 'buy' | 'sell';
  status: 'pending' | 'filled' | 'failed' | 'cancelled';
  amount: string;
  slippage: number;
  priceImpact: number;
  gasCost: number;
  priorityFeeMicroLamports: number;
  mevProtected: boolean;
  mevSavings?: number;
  executionPrice?: number;
  expectedPrice?: number;
  pnl?: number;
  pnlPercent?: number;
  walletAddress?: string;
  isPaperTrade?: boolean;
}
```

### Execution Quality Calculation

Execution quality is calculated based on three factors:
- **Slippage Score**: Lower slippage = higher score
- **Price Impact Score**: Lower impact = higher score
- **MEV Protection Score**: Protected trades get bonus points

Ratings:
- **Excellent**: Average score ≥ 90
- **Good**: Average score ≥ 75
- **Fair**: Average score ≥ 55
- **Poor**: Average score < 55

## Testing

Comprehensive test coverage includes:

### Filter Tests (`tradeFilters.test.ts`)
- Date range filtering
- Token filtering
- Side and status filtering
- P&L range filtering
- Search functionality
- Wallet address filtering
- Paper trade filtering
- Sorting operations
- Pagination logic
- Execution quality calculation

### Export Tests (`tradeExport.test.ts`)
- Column preset selection
- Value formatting
- CSV generation
- XLSX generation
- Header inclusion/exclusion
- Special character escaping
- Column filtering
- Filename generation

### Scheduling Tests (`tradeScheduling.test.ts`)
- Next run date computation
- Schedule due detection
- Schedule advancement
- Schedule initialization
- Multiple cadence types
- Custom interval handling

## Technical Details

### Store Management

Trade reporting uses two Zustand stores:

1. **tradingSettingsStore**: Manages trade history and filters
2. **tradeReportingStore**: Manages export schedules

Both stores use persistence middleware to maintain state across sessions.

### Export Libraries

- **xlsx**: Used for XLSX file generation
- **date-fns**: Used for date formatting and manipulation

### Type Safety

All components and utilities are fully typed with TypeScript for compile-time safety and better developer experience.

## Best Practices

1. **Regular Exports**: Set up scheduled exports for important data (tax reports, etc.)
2. **Filter Before Export**: Apply filters to export only relevant data
3. **Timezone Awareness**: Always select the correct timezone for your needs
4. **Column Selection**: Only export columns you need for better file sizes
5. **Schedule Monitoring**: Regularly check schedule execution status

## Future Enhancements

Potential future additions:
- Export to additional formats (JSON, XML)
- Advanced analytics dashboards
- Custom alert rules
- Integration with tax software
- Batch trade operations
- Trade comparison tools
