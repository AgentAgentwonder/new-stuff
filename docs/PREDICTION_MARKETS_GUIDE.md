# Prediction Markets Module Guide

## Overview

The Prediction Markets module integrates Polymarket and Drift prediction APIs into Eclipse Market Pro, providing a unified interface to track, analyze, and compare prediction markets across multiple platforms.

## Features

- **Multi-Platform Integration**: Aggregates prediction markets from Polymarket and Drift
- **Data Normalization**: Unified data structure across different prediction market sources
- **Caching**: Built-in caching with 60-second TTL to reduce API calls
- **Custom Predictions**: Create and track your own predictions
- **Performance Tracking**: Monitor your accuracy vs. market consensus
- **Portfolio Comparison**: Compare your prediction performance against community averages
- **Consensus Heatmaps**: Visualize agreement/disagreement between markets
- **Category Filtering**: Filter markets by category (Crypto, DeFi, Infrastructure, etc.)

## Architecture

### Backend (Rust/Tauri)

#### Adapters

**Polymarket Adapter** (`src-tauri/src/market/polymarket_adapter.rs`)
- Connects to Polymarket CLOB API
- Fetches active prediction markets
- Retrieves order book and trade data
- Built-in caching with 60-second TTL

**Drift Adapter** (`src-tauri/src/market/drift_adapter.rs`)
- Connects to Drift API
- Fetches prediction markets and derivatives data
- Retrieves perpetual markets and funding rates
- Built-in caching with 60-second TTL

#### Data Normalization

All prediction markets are normalized to a common format:

```rust
pub struct PredictionMarket {
    pub id: String,
    pub source: String, // "polymarket" | "drift"
    pub title: String,
    pub description: String,
    pub category: String,
    pub outcomes: Vec<String>,
    pub outcome_prices: Vec<f64>, // Probabilities (0.0 to 1.0)
    pub volume_24h: f64,
    pub total_volume: f64,
    pub liquidity: f64,
    pub created_at: Option<i64>,
    pub end_date: Option<i64>,
    pub resolved: bool,
    pub winning_outcome: Option<usize>,
    pub tags: Vec<String>,
    pub image_url: Option<String>,
}
```

### Frontend (React/TypeScript)

**PredictionMarkets Component** (`src/pages/PredictionMarkets.tsx`)
- Grid view of active markets
- Search and filter capabilities
- Market detail modal with consensus data
- Portfolio stats dashboard
- Create custom prediction form

## Configuration

### API Keys (Optional)

Both Polymarket and Drift APIs currently work without authentication for public data. If rate limits are encountered, you may need to configure API keys.

#### Polymarket API

No API key required for public market data. Rate limits:
- **Public endpoints**: ~100 requests per minute
- **Authenticated endpoints**: Contact Polymarket for API access

#### Drift Protocol API

No API key required for public endpoints. Rate limits:
- **Public endpoints**: ~60 requests per minute
- **WebSocket**: Real-time updates available

### Environment Variables

Create a `.env` file in the project root (optional):

```bash
# Polymarket (if using authenticated endpoints)
VITE_POLYMARKET_API_KEY=your_polymarket_api_key_here

# Drift Protocol (if using authenticated endpoints)
VITE_DRIFT_API_KEY=your_drift_api_key_here
```

### Cache Configuration

Cache TTL is set to 60 seconds by default. To modify:

Edit `src-tauri/src/market/polymarket_adapter.rs` and `drift_adapter.rs`:

```rust
const CACHE_TTL: Duration = Duration::from_secs(60); // Modify this value
```

## API Endpoints

### Tauri Commands

All commands are accessible via `invoke()`:

#### Get Prediction Markets
```typescript
const markets = await invoke<PredictionMarket[]>('get_prediction_markets', {
  useMock: false // Set to true for testing
});
```

#### Search Markets
```typescript
const results = await invoke<PredictionMarket[]>('search_prediction_markets', {
  query: 'bitcoin',
  useMock: false
});
```

#### Create Custom Prediction
```typescript
const prediction = await invoke<CustomPrediction>('create_custom_prediction', {
  prediction: {
    id: uuid(),
    userId: 'user_123',
    title: 'My Prediction',
    description: 'Description here',
    outcomes: ['Yes', 'No'],
    userPrediction: [0.7, 0.3],
    confidence: 0.8,
    createdAt: Date.now(),
    updatedAt: Date.now(),
    notes: 'Optional notes'
  }
});
```

#### Get Portfolio Comparison
```typescript
const stats = await invoke<PortfolioComparison>('get_portfolio_comparison', {
  userId: 'user_123'
});
```

#### Get Consensus Data
```typescript
const consensus = await invoke<ConsensusData>('get_consensus_data', {
  marketId: 'polymarket_0x123...',
  useMock: false
});
```

#### Record Performance
```typescript
await invoke('record_prediction_performance', {
  performance: {
    predictionId: 'pred_123',
    userId: 'user_123',
    initialPrediction: [0.6, 0.4],
    actualOutcome: 0, // Index of winning outcome
    accuracyScore: 0.85,
    brierScore: 0.12,
    logScore: -0.5,
    marketComparison: 0.15,
    timestamp: Date.now()
  }
});
```

## Mock Data

For development and testing, the module includes comprehensive mock data generators:

- **Polymarket**: 3 sample markets (Bitcoin, Ethereum, Solana)
- **Drift**: 3 sample predictions + 3 perpetual markets
- All mock data includes realistic random variations

To use mock data:
```typescript
const markets = await invoke('get_prediction_markets', { useMock: true });
```

## Error Handling

All API calls include error handling with fallback to mock data:

```rust
let markets = if use_mock {
    generate_mock_polymarket_markets()
} else {
    self.adapter.fetch_markets()
        .await
        .unwrap_or_else(|_| generate_mock_polymarket_markets())
};
```

## Performance Metrics

### Brier Score
Measures prediction accuracy. Lower is better (0 = perfect, 1 = worst).

```
Brier Score = Σ(probability - actual)² / N
```

### Log Score
Measures prediction confidence. Higher is better.

```
Log Score = log(probability of correct outcome)
```

### Accuracy Rate
Percentage of correct predictions (threshold: 70% confidence).

## Rate Limiting

### Best Practices

1. **Use Caching**: The built-in cache reduces API calls automatically
2. **Batch Requests**: Request multiple markets in a single call when possible
3. **Polling Interval**: Recommended minimum 30 seconds between refreshes
4. **Error Handling**: Always implement fallback to cached/mock data

### Handling Rate Limits

If you encounter rate limits:

1. Increase cache TTL
2. Reduce polling frequency  
3. Implement exponential backoff
4. Contact API provider for higher limits

## Testing

### Unit Tests

Run backend tests:
```bash
cd src-tauri
cargo test prediction_markets
```

### Integration Tests

Test with mock data:
```typescript
describe('Prediction Markets', () => {
  it('should load markets', async () => {
    const markets = await invoke('get_prediction_markets', { useMock: true });
    expect(markets.length).toBeGreaterThan(0);
  });

  it('should search markets', async () => {
    const results = await invoke('search_prediction_markets', {
      query: 'bitcoin',
      useMock: true
    });
    expect(results.length).toBeGreaterThan(0);
  });
});
```

### UI Tests

UI interaction tests are located in `src/__tests__/PredictionMarkets.test.tsx`.

Run UI tests:
```bash
npm test
```

## Troubleshooting

### Markets Not Loading

1. Check network connectivity
2. Verify API endpoints are accessible
3. Check browser console for errors
4. Try mock mode: `{ useMock: true }`

### Performance Issues

1. Increase cache TTL
2. Reduce number of markets fetched
3. Disable real-time updates
4. Clear cache: Close and reopen app

### Data Inconsistencies

1. Compare with source (Polymarket/Drift website)
2. Check cache expiration
3. Verify data normalization logic
4. Report issues with specific market IDs

## Future Enhancements

- [ ] WebSocket support for real-time updates
- [ ] More prediction market sources (Augur, Gnosis)
- [ ] Advanced analytics (Kelly criterion, EV calculation)
- [ ] Social features (follow top predictors)
- [ ] Automated bet placement
- [ ] Historical market resolution tracking
- [ ] Leaderboards and rankings

## Support

For issues or questions:
- GitHub Issues: [Repository URL]
- Documentation: This file
- Community: [Discord/Forum]

## License

Same as Eclipse Market Pro main license.
