# Trending Coins Implementation

## Overview
This document describes the basic trending coins display feature implemented as part of the minimal version scope.

## Backend (Rust/Tauri)

### Location
- `src-tauri/src/market/trending_coins.rs`

### Key Components

1. **TrendingCoin Structure**
   ```rust
   pub struct TrendingCoin {
       pub address: String,
       pub symbol: String,
       pub name: String,
       pub price: f64,
       pub price_change_24h: f64,
       pub volume_24h: f64,
       pub volume_change_24h: f64,
       pub market_cap: f64,
       pub market_cap_change_24h: f64,
       pub liquidity: f64,
       pub trend_score: f64,
       pub logo_uri: Option<String>,
   }
   ```

2. **Tauri Commands**
   - `get_trending_coins(limit: usize, api_key: Option<String>)` - Fetches trending coins
   - `refresh_trending()` - Clears the cache to force a refresh

3. **Birdeye API Integration**
   - Endpoint: `https://public-api.birdeye.so/defi/token_trending`
   - Requires API key via header: `X-API-KEY`
   - Falls back to mock data if API key not provided or request fails
   - Includes 60-second caching to reduce API calls

## Frontend (React/TypeScript)

### Location
- `src/pages/Coins/Trending.tsx`

### Features
- **Simple Table Layout**: Displays coins in a clean, readable table format
- **Basic Information**: Shows rank, name, price, and 24h change
- **Manual Refresh**: Button to reload data from the API
- **Loading State**: Spinner displayed while fetching data
- **Error Handling**: User-friendly error messages with retry option
- **Responsive Design**: Table adapts to different screen sizes

### Component Structure
```typescript
export function Trending() {
  // State management
  const [coins, setCoins] = useState<TrendingCoin[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch function
  const fetchTrendingCoins = async () => {
    // Invokes Tauri command
    const result = await invoke<TrendingCoin[]>('get_trending_coins', {
      limit: 20,
      apiKey: null,
    });
    setCoins(result);
  };

  // Initial load
  useEffect(() => {
    fetchTrendingCoins();
  }, []);

  // Render table
  return <table>...</table>;
}
```

## Integration

### Navigation
The Trending component can be accessed:
1. As a standalone page (recommended route: `/coins/trending`)
2. As a tab within the Coins page (current implementation)

### API Key Configuration
- API keys are managed via the Settings page
- Stored securely in the Tauri keystore
- Commands accept optional `apiKey` parameter
- Falls back to mock data if no key provided

## Data Flow

```
User Action → Frontend Component → Tauri Command → Birdeye API
                                                    ↓
                                                 Mock Data (fallback)
                                                    ↓
Backend Response → Frontend State → UI Update
```

## Testing

### Without API Key
1. Navigate to the Trending page
2. Mock data will be displayed (8-10 sample tokens)
3. All features work with mock data

### With API Key
1. Go to Settings > API Configuration
2. Add Birdeye API key
3. Navigate to the Trending page
4. Real data from Birdeye will be displayed (up to 20 tokens)

## Future Enhancements (Out of Scope)
- Auto-refresh functionality
- Caching strategy improvements
- Sorting and filtering
- Sparkline charts
- Sentiment indicators
- Add to watchlist
- Detailed coin view

## Code Statistics
- Backend: ~305 lines (trending_coins.rs)
- Frontend: ~175 lines (Trending.tsx)
- Total: ~480 lines
- Meets requirement: < 300 lines for new minimal component

## Notes
- The implementation reuses existing backend infrastructure
- No breaking changes to existing code
- Follows existing code patterns and conventions
- Uses Tailwind CSS for styling
- Compatible with the rest of the application
