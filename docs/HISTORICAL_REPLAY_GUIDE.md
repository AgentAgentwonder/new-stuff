# Historical Replay Simulator Guide

## Overview

The Historical Replay Simulator allows you to test trading strategies and analyze portfolio performance using historical market data. This feature provides a "time machine" capability to understand how your holdings would have performed over specific time periods.

## Key Features

### 1. Historical Data Fetching
- **Data Sources**: Fetch data from Birdeye API or use mock data for testing
- **Intervals**: Support for 1m, 5m, 15m, 1h, 4h, 1d intervals
- **Caching**: Automatic caching of fetched data to reduce API calls
- **Streaming**: Chunked data fetching for performance with large datasets

### 2. Portfolio Simulation
- **Initial Capital**: Configure starting capital for simulations
- **Commission & Slippage**: Realistic trading costs including:
  - Commission rates (default: 0.1%)
  - Slippage rates (default: 0.05%)
- **Actions**: Support for buy, sell, and rebalance operations
- **Holdings Import**: Import existing portfolio holdings via JSON

### 3. Counterfactual Analysis
- **"What If" Scenarios**: Analyze what would have happened if you:
  - Held a position since a specific date
  - Sold at different times
  - Rebalanced your portfolio
- **Performance Metrics**: 
  - Total return (absolute and percentage)
  - Annualized return
  - Maximum drawdown
  - Volatility
  - Sharpe ratio

### 4. Interactive Playback
- **Timeline Scrubber**: Navigate through historical data points
- **Playback Controls**: Play, pause, skip forward/backward
- **Speed Control**: Adjust playback speed (0.25x to 8x)
- **Real-time Updates**: See portfolio value changes as time progresses

## Usage

### Basic Workflow

1. **Select Data Range**
   - Choose symbol (e.g., SOL, BTC, ETH)
   - Set start and end dates
   - Select time interval

2. **Fetch Data**
   - Click "Fetch Historical Data"
   - Data is cached for future use
   - Progress indicator shows fetch status

3. **Configure Simulation**
   - Set initial capital
   - Configure commission and slippage rates
   - Add trading actions (optional)

4. **Run Simulation**
   - Click "Run Simulation"
   - View real-time portfolio performance
   - Analyze metrics and charts

5. **Counterfactual Analysis**
   - Click "What If Held Since Start?"
   - Compare different holding strategies
   - View side-by-side performance

### Importing Portfolio Holdings

Create a JSON file with your holdings:

```json
[
  {
    "symbol": "SOL",
    "quantity": 100,
    "average_entry_price": 50.0,
    "first_purchase_time": 1640000000
  },
  {
    "symbol": "BTC",
    "quantity": 0.5,
    "average_entry_price": 42000.0,
    "first_purchase_time": 1640000000
  }
]
```

Upload this file using the Portfolio Importer component.

### Adding Strategy Actions

1. Enter symbol, action type (buy/sell), quantity, and price
2. Set timestamp for when action should execute
3. Click "Add Action"
4. Run simulation to see strategy performance

## Data Storage

### Database Structure

Historical data is stored in SQLite database: `historical_replay.db`

**Tables:**
- `historical_prices`: OHLCV data points
- `historical_orderbooks`: Order book snapshots
- `data_cache_metadata`: Cache coverage tracking

### Storage Management

**Cache Statistics:**
```rust
// Get cache stats for a symbol
historical_get_cache_stats(symbol: "SOL")
```

**Clear Old Data:**
```rust
// Clear data older than 90 days
historical_clear_old_data(days: 90)
```

### Storage Impact

Typical storage requirements:
- **1 month of 1-hour data**: ~720 records (~36 KB)
- **1 month of 1-minute data**: ~43,200 records (~2.1 MB)
- **Order book snapshots**: ~500 KB per 1,000 snapshots

Recommendations:
- Use longer intervals (1h, 4h, 1d) for long-term backtests
- Periodically clear old data (90+ days)
- Monitor cache with `get_cache_stats`

## API Integration

### Birdeye API

Configure Birdeye API key for real data:

```typescript
// Set API key
await invoke('historical_set_api_key', {
  apiKey: 'your-birdeye-api-key'
});
```

**Rate Limits:**
- Birdeye: 100 requests/minute (free tier)
- Consider chunked fetching for large date ranges

**Fallback:**
- Mock data generation when API unavailable
- Automatic fallback on API errors

## Performance Metrics

### Simulation Metrics

- **Final Value**: Portfolio value at end of simulation
- **Total Return**: Absolute and percentage gains/losses
- **Sharpe Ratio**: Risk-adjusted returns
- **Maximum Drawdown**: Largest peak-to-trough decline
- **Number of Trades**: Total executed trades
- **Total Fees**: All commissions and slippage costs

### Interpretation

- **Sharpe Ratio > 1**: Good risk-adjusted performance
- **Sharpe Ratio > 2**: Excellent performance
- **Max Drawdown < 20%**: Reasonable risk management
- **High trade count**: Consider reducing over-trading

## Testing

### Backend Tests

```bash
cd src-tauri
cargo test historical
```

### Frontend Tests

```bash
npm run test -- HistoricalReplay
```

### Integration Tests

Located in `tests/historical-replay.test.ts`

## Troubleshooting

### Common Issues

1. **"No data available"**
   - Check date range and symbol
   - Verify API key if using Birdeye
   - Try mock data mode first

2. **"Simulation failed"**
   - Ensure dataset is loaded
   - Check action timestamps are within data range
   - Verify sufficient capital for trades

3. **"Cache full"**
   - Run `historical_clear_old_data(90)`
   - Reduce data range or interval
   - Check disk space

4. **Slow playback**
   - Reduce data density (use longer intervals)
   - Clear browser cache
   - Check system resources

## Best Practices

1. **Start Small**: Test with short date ranges (7-30 days)
2. **Use Realistic Costs**: Set commission/slippage to match real trading
3. **Test Strategies**: Use multiple counterfactuals to validate ideas
4. **Monitor Storage**: Regularly clear old cached data
5. **Document Findings**: Export simulation results for analysis

## Advanced Features

### Custom Data Sources

Extend `HistoricalDataFetcher` to add custom data sources:

```rust
impl HistoricalDataFetcher {
    async fn fetch_from_custom_source(&self, request: &FetchRequest) -> Result<Vec<HistoricalDataPoint>, Box<dyn std::error::Error>> {
        // Your custom data source implementation
    }
}
```

### Strategy Backtesting

Implement automated strategy testing:

```typescript
const strategy = {
  name: "Moving Average Crossover",
  actions: generateMAActions(dataset),
};

const result = await runSimulation(strategy);
```

## API Reference

### Backend Commands

```rust
// Fetch historical dataset
historical_fetch_dataset(request: FetchRequest) -> HistoricalDataSet

// Fetch order book history
historical_fetch_orderbooks(symbol: String, start_time: i64, end_time: i64) -> Vec<OrderBookSnapshot>

// Run portfolio simulation
historical_run_simulation(payload: SimulationPayload) -> SimulationResult

// Compute counterfactual
historical_compute_counterfactual(request: CounterfactualRequest) -> Option<CounterfactualResult>

// Get cache statistics
historical_get_cache_stats(symbol: String) -> HashMap<String, u64>

// Clear old data
historical_clear_old_data(days: i64) -> u64

// Set API key
historical_set_api_key(api_key: Option<String>) -> ()
```

### Frontend Store

```typescript
// Store actions
const { 
  setDataset,
  setCurrentDataset,
  setSimulationResult,
  addCounterfactual,
  setPlaybackState,
  reset,
} = useHistoricalReplayStore();
```

## Support

For issues or feature requests:
- GitHub Issues: [repository-url]/issues
- Documentation: [repository-url]/docs
- Community: [discord-url]

## License

This feature is part of Eclipse Market Pro.
See LICENSE file for details.
