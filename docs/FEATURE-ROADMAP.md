# Eclipse Market Pro - Feature Roadmap
## Comprehensive Implementation Plan for Tier 1 Priorities

---

## Executive Summary

This document outlines a modular, parallel-development roadmap for four strategic pillars:
1. **AI Trading Features** - Predictive intelligence and automated strategy optimization
2. **DeFi Integrations** - Deep protocol integration for yield, swaps, and liquidity
3. **Social Trading Capabilities** - Community-driven strategy sharing and leaderboards
4. **Security Enhancements** - Hardware wallet support, transaction simulation, audit logging

**Architecture Philosophy:** Build incrementally on existing infrastructure (SQLite, Tauri commands, Solana RPC, Zustand stores) without introducing breaking changes. Each feature is designed as a self-contained module with clear integration points.

**Development Approach:** Features are sequenced to minimize merge conflicts, enable early wins, and allow parallel development across multiple streams.

---

## Feature Selection Rationale

### Selection Criteria Applied
1. **Leverages Existing Strengths:** Builds on current wallet monitoring, copy trading, voice commands, and market data infrastructure
2. **Market Differentiation:** Features that competitors lack or execute poorly
3. **Implementation Feasibility:** Medium complexity, battle-tested dependencies, no major refactoring
4. **User Value:** Directly impacts trading decisions, risk management, and profitability
5. **Modularity:** Clear boundaries, testable in isolation, feature-flaggable

### Rejected Features (and Why)
- **Custom Smart Contract Deployment:** High complexity, security risk, outside core competency
- **Cross-chain Trading:** Requires significant bridge integrations, immature infrastructure
- **Automated Market Making:** Regulatory concerns, requires substantial capital management
- **On-chain Governance Voting:** Low user demand signal, complex integration
- **NFT Trading Features:** Out of scope for current product focus

---

## Tier 1: AI Trading Features

### Feature 1.1: AI-Powered Sentiment Analysis Pipeline
**Complexity:** Medium
**Dependencies:** None (extends existing market data services)
**Value Proposition:** Real-time sentiment aggregation from Twitter, Reddit, Discord, and on-chain activity to inform trading decisions

#### Technical Approach
**New Modules:**
- `src-tauri/src/ai/sentiment_analyzer.rs` - Core sentiment analysis engine
- `src-tauri/src/ai/sentiment_aggregator.rs` - Multi-source data aggregation
- `src-tauri/src/ai/types.rs` - Sentiment score models, confidence intervals

**Database Schema (SQLite):**
```sql
CREATE TABLE sentiment_scores (
    id TEXT PRIMARY KEY,
    token_mint TEXT NOT NULL,
    token_symbol TEXT,
    sentiment_score REAL NOT NULL,  -- -1.0 to 1.0
    confidence REAL NOT NULL,       -- 0.0 to 1.0
    source TEXT NOT NULL,            -- 'twitter', 'reddit', 'discord', 'onchain'
    sample_size INTEGER,
    positive_mentions INTEGER,
    negative_mentions INTEGER,
    neutral_mentions INTEGER,
    timestamp TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_sentiment_token ON sentiment_scores(token_mint, timestamp DESC);
CREATE INDEX idx_sentiment_source ON sentiment_scores(source, timestamp DESC);
```

**API Integration:**
- Reuse existing Reddit client pattern (see `insiders/smart_money.rs`)
- Add Twitter API v2 client (use `reqwest` like existing Birdeye integration)
- On-chain sentiment: Extend `insiders/wallet_monitor.rs` to track buy/sell ratios

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn get_token_sentiment(
    token_mint: String,
    timeframe_hours: Option<u64>,
    db: State<'_, SqlitePool>
) -> Result<SentimentAnalysis, String>

#[tauri::command]
pub async fn get_sentiment_trend(
    token_mint: String,
    start_time: String,
    end_time: String,
    db: State<'_, SqlitePool>
) -> Result<Vec<SentimentDataPoint>, String>

#[tauri::command]
pub async fn refresh_sentiment(
    token_mint: String,
    ai_state: State<'_, AiSentimentState>
) -> Result<(), String>
```

**Frontend Integration:**
- New Zustand store: `src/stores/sentimentStore.ts`
- React hook: `useSentiment(tokenMint)`
- UI component: SentimentGauge (shows score with confidence interval)

**Implementation Sequence:**
1. Create sentiment_analyzer module with mock data
2. Add SQLite schema and persistence layer
3. Integrate Twitter API (battle-tested library: `egg-mode` or direct REST)
4. Integrate Reddit API (extend existing pattern)
5. Add on-chain sentiment (buy/sell pressure from transaction monitoring)
6. Create aggregation logic with weighted scoring
7. Implement Tauri commands
8. Build frontend components

**Testing Strategy:**
- Unit tests: Sentiment scoring algorithm (positive/negative/neutral detection)
- Integration tests: Database persistence, API mocking
- E2E tests: Full pipeline from API fetch to frontend display

---

### Feature 1.2: Predictive Price Modeling
**Complexity:** Large
**Dependencies:** Sentiment Analysis (for input features)
**Value Proposition:** ML-based price predictions with confidence intervals to guide entry/exit decisions

#### Technical Approach
**New Modules:**
- `src-tauri/src/ai/price_predictor.rs` - Core prediction engine
- `src-tauri/src/ai/feature_extractor.rs` - Extract features from market data, sentiment, on-chain metrics
- `src-tauri/src/ai/model_manager.rs` - Model versioning, loading, inference

**Model Architecture:**
- **Algorithm:** Time-series forecasting (LSTM or Transformer-based)
- **Library:** `tch-rs` (Rust bindings for PyTorch) OR `smartcore` (pure Rust ML)
- **Approach:** Pre-trained models shipped with app, no on-device training
- **Features:** Price history, volume, sentiment score, social mentions, wallet activity, TVL changes

**Database Schema:**
```sql
CREATE TABLE price_predictions (
    id TEXT PRIMARY KEY,
    token_mint TEXT NOT NULL,
    prediction_timestamp TEXT NOT NULL,  -- When prediction was made
    target_timestamp TEXT NOT NULL,       -- When prediction is for (e.g., +1h, +24h)
    predicted_price REAL NOT NULL,
    confidence_lower REAL NOT NULL,       -- 95% confidence interval lower bound
    confidence_upper REAL NOT NULL,       -- 95% confidence interval upper bound
    actual_price REAL,                    -- Filled in later for accuracy tracking
    model_version TEXT NOT NULL,
    features TEXT NOT NULL,               -- JSON of features used
    created_at TEXT NOT NULL
);

CREATE INDEX idx_predictions_token ON price_predictions(token_mint, prediction_timestamp DESC);

CREATE TABLE model_performance (
    id TEXT PRIMARY KEY,
    model_version TEXT NOT NULL,
    token_mint TEXT,
    timeframe TEXT NOT NULL,              -- '1h', '24h', '7d'
    mae REAL NOT NULL,                    -- Mean Absolute Error
    rmse REAL NOT NULL,                   -- Root Mean Squared Error
    accuracy_percent REAL NOT NULL,       -- % of predictions within confidence interval
    total_predictions INTEGER NOT NULL,
    evaluated_at TEXT NOT NULL
);
```

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn get_price_prediction(
    token_mint: String,
    timeframe: String,  // '1h', '4h', '24h', '7d'
    predictor: State<'_, PricePredictor>,
    db: State<'_, SqlitePool>
) -> Result<PricePrediction, String>

#[tauri::command]
pub async fn get_prediction_accuracy(
    model_version: Option<String>,
    db: State<'_, SqlitePool>
) -> Result<ModelPerformanceMetrics, String>

#[tauri::command]
pub async fn backtest_predictions(
    token_mint: String,
    start_date: String,
    end_date: String,
    predictor: State<'_, PricePredictor>
) -> Result<BacktestResults, String>
```

**Implementation Sequence:**
1. Design feature extraction pipeline (price, volume, sentiment, on-chain)
2. Train initial models offline (Python/PyTorch), export to TorchScript
3. Create model_manager for loading pre-trained models
4. Implement inference engine in Rust
5. Add prediction persistence and accuracy tracking
6. Build backtesting framework
7. Create Tauri commands
8. Frontend: Prediction chart with confidence bands

**Risk Mitigation:**
- Start with simple models (linear regression, ARIMA) before deep learning
- If `tch-rs` proves problematic, use `smartcore` (pure Rust, simpler)
- Include prominent disclaimers: "Predictions are not financial advice"
- Track and display model accuracy prominently

---

### Feature 1.3: Strategy Backtesting Engine
**Complexity:** Medium
**Dependencies:** Price data (existing), prediction models (optional)
**Value Proposition:** Test trading strategies against historical data before risking real capital

#### Technical Approach
**New Modules:**
- `src-tauri/src/ai/backtest_engine.rs` - Core backtesting logic
- `src-tauri/src/ai/strategy_executor.rs` - Execute strategy rules against historical data
- `src-tauri/src/ai/performance_metrics.rs` - Calculate Sharpe ratio, max drawdown, win rate

**Strategy Definition (JSON/TOML):**
```json
{
  "name": "Momentum with Stop Loss",
  "rules": {
    "entry": {
      "conditions": [
        {"indicator": "rsi", "operator": ">", "value": 70},
        {"indicator": "volume_24h", "operator": ">", "value": 1000000}
      ],
      "logic": "AND"
    },
    "exit": {
      "conditions": [
        {"indicator": "price_change", "operator": "<", "value": -5, "unit": "percent"},
        {"indicator": "rsi", "operator": "<", "value": 30}
      ],
      "logic": "OR"
    }
  },
  "position_sizing": {
    "type": "fixed_percent",
    "value": 10
  },
  "risk_management": {
    "stop_loss_percent": 5,
    "take_profit_percent": 15,
    "max_open_positions": 3
  }
}
```

**Database Schema:**
```sql
CREATE TABLE backtest_results (
    id TEXT PRIMARY KEY,
    strategy_name TEXT NOT NULL,
    strategy_config TEXT NOT NULL,   -- JSON
    token_mint TEXT,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    initial_capital REAL NOT NULL,
    final_capital REAL NOT NULL,
    total_return_percent REAL NOT NULL,
    sharpe_ratio REAL,
    max_drawdown_percent REAL,
    win_rate REAL,
    total_trades INTEGER,
    winning_trades INTEGER,
    losing_trades INTEGER,
    avg_win REAL,
    avg_loss REAL,
    created_at TEXT NOT NULL
);

CREATE TABLE backtest_trades (
    id TEXT PRIMARY KEY,
    backtest_id TEXT NOT NULL,
    token_mint TEXT NOT NULL,
    entry_time TEXT NOT NULL,
    entry_price REAL NOT NULL,
    exit_time TEXT,
    exit_price REAL,
    position_size REAL NOT NULL,
    pnl REAL,
    pnl_percent REAL,
    exit_reason TEXT,  -- 'stop_loss', 'take_profit', 'signal', 'end_of_test'
    FOREIGN KEY (backtest_id) REFERENCES backtest_results(id)
);
```

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn run_backtest(
    strategy_config: StrategyConfig,
    token_mint: String,
    start_date: String,
    end_date: String,
    initial_capital: f64,
    engine: State<'_, BacktestEngine>,
    db: State<'_, SqlitePool>
) -> Result<BacktestResult, String>

#[tauri::command]
pub async fn get_backtest_history(
    strategy_name: Option<String>,
    db: State<'_, SqlitePool>
) -> Result<Vec<BacktestSummary>, String>

#[tauri::command]
pub async fn compare_strategies(
    backtest_ids: Vec<String>,
    db: State<'_, SqlitePool>
) -> Result<StrategyComparison, String>
```

**Implementation Sequence:**
1. Design strategy DSL (JSON schema)
2. Build strategy parser and validator
3. Implement indicator library (RSI, MACD, Bollinger Bands, Volume)
4. Create backtest engine with event-driven simulation
5. Add performance metrics calculator
6. Persist results to database
7. Build comparison and analysis tools
8. Frontend: Backtest results dashboard with equity curve

---

## Tier 1: DeFi Integrations

### Feature 2.1: Jupiter Aggregator Integration
**Complexity:** Medium
**Dependencies:** Wallet connections (existing Phantom integration)
**Value Proposition:** Best execution for swaps, limit orders, DCA across all Solana DEXs

#### Technical Approach
**New Modules:**
- `src-tauri/src/defi/jupiter.rs` - Jupiter API client
- `src-tauri/src/defi/swap_executor.rs` - Execute swaps with slippage protection
- `src-tauri/src/defi/limit_orders.rs` - Manage Jupiter limit orders

**Jupiter API Integration:**
- API v6: `https://quote-api.jup.ag/v6/`
- Rate limiting: Use `tower` middleware for request throttling
- Websocket: Subscribe to limit order fills

**Database Schema:**
```sql
CREATE TABLE jupiter_swaps (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    input_mint TEXT NOT NULL,
    output_mint TEXT NOT NULL,
    input_amount REAL NOT NULL,
    output_amount REAL NOT NULL,
    quoted_price REAL NOT NULL,
    executed_price REAL NOT NULL,
    slippage_bps INTEGER NOT NULL,
    route TEXT NOT NULL,             -- JSON of DEXs used
    tx_signature TEXT,
    status TEXT NOT NULL,            -- 'pending', 'confirmed', 'failed'
    error_message TEXT,
    created_at TEXT NOT NULL,
    confirmed_at TEXT
);

CREATE TABLE jupiter_limit_orders (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    input_mint TEXT NOT NULL,
    output_mint TEXT NOT NULL,
    input_amount REAL NOT NULL,
    target_price REAL NOT NULL,
    expiry_time TEXT,
    status TEXT NOT NULL,            -- 'open', 'filled', 'cancelled', 'expired'
    filled_amount REAL DEFAULT 0,
    tx_signature TEXT,
    created_at TEXT NOT NULL,
    filled_at TEXT
);

CREATE INDEX idx_swaps_wallet ON jupiter_swaps(user_wallet, created_at DESC);
CREATE INDEX idx_orders_wallet ON jupiter_limit_orders(user_wallet, status);
```

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn get_jupiter_quote(
    input_mint: String,
    output_mint: String,
    amount: u64,
    slippage_bps: u16,
    jupiter: State<'_, JupiterClient>
) -> Result<SwapQuote, String>

#[tauri::command]
pub async fn execute_jupiter_swap(
    quote: SwapQuote,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<SwapTransaction, String>

#[tauri::command]
pub async fn create_limit_order(
    input_mint: String,
    output_mint: String,
    input_amount: u64,
    target_price: f64,
    expiry_hours: Option<u64>,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<LimitOrder, String>

#[tauri::command]
pub async fn get_active_limit_orders(
    wallet_address: String,
    db: State<'_, SqlitePool>
) -> Result<Vec<LimitOrder>, String>

#[tauri::command]
pub async fn cancel_limit_order(
    order_id: String,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<(), String>
```

**Implementation Sequence:**
1. Implement Jupiter API client (quote, swap, limit order endpoints)
2. Add Solana transaction building for swaps
3. Integrate with Phantom wallet for signing
4. Add database persistence for swap history
5. Implement limit order management
6. Add websocket listener for order fills
7. Create Tauri commands
8. Frontend: Swap interface with route comparison

**Testing:**
- Use Solana devnet for testing
- Mock Jupiter API responses for unit tests
- Test slippage protection and route selection

---

### Feature 2.2: Yield Farming Dashboard
**Complexity:** Medium
**Dependencies:** Wallet connections
**Value Proposition:** Track positions across Marinade, Jito, Kamino, Raydium with APY calculations and auto-compounding

#### Technical Approach
**New Modules:**
- `src-tauri/src/defi/yield_tracker.rs` - Track yield positions across protocols
- `src-tauri/src/defi/apy_calculator.rs` - Calculate real-time APY with historical data
- `src-tauri/src/defi/protocols/marinade.rs` - Marinade Finance integration
- `src-tauri/src/defi/protocols/jito.rs` - Jito staking integration
- `src-tauri/src/defi/protocols/kamino.rs` - Kamino lending/yield integration

**Protocol Integrations:**
- **Marinade:** Liquid staking (mSOL), track staking rewards
- **Jito:** MEV rewards tracking, JitoSOL positions
- **Kamino:** Lending positions, borrow utilization, yield strategies
- **Raydium:** LP positions, impermanent loss tracking

**Database Schema:**
```sql
CREATE TABLE yield_positions (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    protocol TEXT NOT NULL,           -- 'marinade', 'jito', 'kamino', 'raydium'
    position_type TEXT NOT NULL,      -- 'staking', 'lending', 'liquidity_pool'
    token_mint TEXT NOT NULL,
    amount REAL NOT NULL,
    value_usd REAL NOT NULL,
    entry_price REAL NOT NULL,
    current_apy REAL,
    rewards_earned REAL,
    rewards_claimed REAL,
    last_update TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE apy_history (
    id TEXT PRIMARY KEY,
    protocol TEXT NOT NULL,
    position_type TEXT NOT NULL,
    token_mint TEXT NOT NULL,
    apy REAL NOT NULL,
    tvl REAL,
    timestamp TEXT NOT NULL
);

CREATE TABLE impermanent_loss (
    id TEXT PRIMARY KEY,
    lp_position_id TEXT NOT NULL,
    token_a_mint TEXT NOT NULL,
    token_b_mint TEXT NOT NULL,
    entry_ratio REAL NOT NULL,
    current_ratio REAL NOT NULL,
    il_percent REAL NOT NULL,          -- Negative = loss
    calculated_at TEXT NOT NULL,
    FOREIGN KEY (lp_position_id) REFERENCES yield_positions(id)
);

CREATE INDEX idx_positions_wallet ON yield_positions(user_wallet, protocol);
CREATE INDEX idx_apy_protocol ON apy_history(protocol, timestamp DESC);
```

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn get_yield_positions(
    wallet_address: String,
    yield_tracker: State<'_, YieldTracker>
) -> Result<Vec<YieldPosition>, String>

#[tauri::command]
pub async fn get_protocol_apy(
    protocol: String,
    timeframe_hours: u64
) -> Result<ApyData, String>

#[tauri::command]
pub async fn calculate_impermanent_loss(
    lp_position_id: String,
    db: State<'_, SqlitePool>
) -> Result<ImpermanentLossData, String>

#[tauri::command]
pub async fn refresh_yield_data(
    wallet_address: String,
    yield_tracker: State<'_, YieldTracker>,
    db: State<'_, SqlitePool>
) -> Result<(), String>

#[tauri::command]
pub async fn get_yield_suggestions(
    wallet_address: String,
    risk_tolerance: String,  // 'low', 'medium', 'high'
    yield_tracker: State<'_, YieldTracker>
) -> Result<Vec<YieldOpportunity>, String>
```

**Implementation Sequence:**
1. Research protocol APIs and on-chain program interfaces
2. Implement position detection from wallet addresses
3. Build APY calculator with historical tracking
4. Add impermanent loss calculator for LP positions
5. Implement protocol-specific integrations (Marinade first, then others)
6. Create aggregated dashboard data structures
7. Add yield opportunity recommendations
8. Frontend: Yield dashboard with position cards and APY charts

---

### Feature 2.3: Liquidity Pool Analytics
**Complexity:** Medium
**Dependencies:** DeFi protocol integrations
**Value Proposition:** Deep analytics on LP positions including impermanent loss, fee earnings, and optimal rebalancing

#### Technical Approach
**New Modules:**
- `src-tauri/src/defi/lp_analyzer.rs` - Core LP analytics engine
- `src-tauri/src/defi/il_calculator.rs` - Impermanent loss calculations
- `src-tauri/src/defi/fee_tracker.rs` - Track trading fees earned

**Analytics Features:**
- Real-time IL calculation
- Fee earnings vs. IL comparison
- Optimal entry/exit points based on volatility
- Pool health metrics (TVL, volume, fee APR)

**Database Schema:**
```sql
CREATE TABLE lp_analytics (
    id TEXT PRIMARY KEY,
    position_id TEXT NOT NULL,
    token_a_mint TEXT NOT NULL,
    token_b_mint TEXT NOT NULL,
    pool_address TEXT NOT NULL,
    il_24h REAL NOT NULL,
    il_7d REAL NOT NULL,
    il_30d REAL NOT NULL,
    fees_earned_24h REAL NOT NULL,
    fees_earned_7d REAL NOT NULL,
    fees_earned_30d REAL NOT NULL,
    net_pnl REAL NOT NULL,              -- Fees - IL
    pool_tvl REAL,
    pool_volume_24h REAL,
    fee_apr REAL,
    analyzed_at TEXT NOT NULL,
    FOREIGN KEY (position_id) REFERENCES yield_positions(id)
);

CREATE TABLE rebalance_suggestions (
    id TEXT PRIMARY KEY,
    position_id TEXT NOT NULL,
    suggestion_type TEXT NOT NULL,      -- 'add_liquidity', 'remove_liquidity', 'rebalance'
    reason TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    expected_benefit_percent REAL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (position_id) REFERENCES yield_positions(id)
);
```

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn get_lp_analytics(
    position_id: String,
    analyzer: State<'_, LpAnalyzer>,
    db: State<'_, SqlitePool>
) -> Result<LpAnalytics, String>

#[tauri::command]
pub async fn calculate_optimal_range(
    token_a_mint: String,
    token_b_mint: String,
    pool_address: String,
    analyzer: State<'_, LpAnalyzer>
) -> Result<PriceRange, String>

#[tauri::command]
pub async fn get_rebalance_suggestions(
    position_id: String,
    db: State<'_, SqlitePool>
) -> Result<Vec<RebalanceSuggestion>, String>

#[tauri::command]
pub async fn simulate_lp_position(
    token_a_mint: String,
    token_b_mint: String,
    amount_a: f64,
    amount_b: f64,
    days: u32,
    analyzer: State<'_, LpAnalyzer>
) -> Result<LpSimulation, String>
```

**Implementation Sequence:**
1. Implement IL calculator (standard formula + Uniswap v3 concentrated liquidity)
2. Build fee earnings tracker (parse pool transaction history)
3. Create net P&L calculator (fees - IL)
4. Add pool health metrics (TVL, volume from Raydium/Orca APIs)
5. Implement rebalancing suggestions algorithm
6. Build position simulator
7. Frontend: LP analytics dashboard with IL/fee comparison charts

---

## Tier 1: Social Trading Capabilities

### Feature 3.1: Strategy Marketplace
**Complexity:** Large
**Dependencies:** Backtest engine, user authentication
**Value Proposition:** Share, rate, and subscribe to proven trading strategies with performance transparency

#### Technical Approach
**New Modules:**
- `src-tauri/src/social/strategy_marketplace.rs` - Strategy publishing and discovery
- `src-tauri/src/social/strategy_ratings.rs` - Rating and review system
- `src-tauri/src/social/subscription_manager.rs` - Manage strategy subscriptions

**Database Schema:**
```sql
CREATE TABLE published_strategies (
    id TEXT PRIMARY KEY,
    author_wallet TEXT NOT NULL,
    author_username TEXT,
    strategy_name TEXT NOT NULL,
    description TEXT NOT NULL,
    strategy_config TEXT NOT NULL,    -- JSON strategy definition
    category TEXT NOT NULL,           -- 'momentum', 'mean_reversion', 'breakout', etc.
    price_type TEXT NOT NULL,         -- 'free', 'paid'
    price_amount REAL,
    currency TEXT,                    -- 'SOL', 'USDC'
    is_public BOOLEAN NOT NULL DEFAULT 1,
    total_subscribers INTEGER DEFAULT 0,
    total_ratings INTEGER DEFAULT 0,
    avg_rating REAL DEFAULT 0,
    verified BOOLEAN DEFAULT 0,       -- Verified by platform
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE strategy_performance (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    timeframe TEXT NOT NULL,          -- '7d', '30d', '90d', 'all'
    total_return_percent REAL NOT NULL,
    sharpe_ratio REAL,
    max_drawdown_percent REAL,
    win_rate REAL,
    total_trades INTEGER,
    calculated_at TEXT NOT NULL,
    FOREIGN KEY (strategy_id) REFERENCES published_strategies(id)
);

CREATE TABLE strategy_subscriptions (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    subscriber_wallet TEXT NOT NULL,
    subscription_type TEXT NOT NULL,  -- 'free', 'paid'
    status TEXT NOT NULL,             -- 'active', 'cancelled', 'expired'
    payment_tx TEXT,
    subscribed_at TEXT NOT NULL,
    expires_at TEXT,
    FOREIGN KEY (strategy_id) REFERENCES published_strategies(id)
);

CREATE TABLE strategy_ratings (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    rater_wallet TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    verified_user BOOLEAN DEFAULT 0,  -- Has actually traded with strategy
    created_at TEXT NOT NULL,
    FOREIGN KEY (strategy_id) REFERENCES published_strategies(id),
    UNIQUE(strategy_id, rater_wallet)
);

CREATE INDEX idx_strategies_author ON published_strategies(author_wallet);
CREATE INDEX idx_strategies_category ON published_strategies(category, avg_rating DESC);
CREATE INDEX idx_subscriptions_user ON strategy_subscriptions(subscriber_wallet);
```

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn publish_strategy(
    strategy: StrategyConfig,
    description: String,
    category: String,
    price_info: Option<PriceInfo>,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<PublishedStrategy, String>

#[tauri::command]
pub async fn browse_strategies(
    category: Option<String>,
    sort_by: String,  // 'rating', 'subscribers', 'performance', 'recent'
    limit: usize,
    offset: usize,
    db: State<'_, SqlitePool>
) -> Result<Vec<StrategyListing>, String>

#[tauri::command]
pub async fn get_strategy_details(
    strategy_id: String,
    db: State<'_, SqlitePool>
) -> Result<StrategyDetails, String>

#[tauri::command]
pub async fn subscribe_to_strategy(
    strategy_id: String,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<Subscription, String>

#[tauri::command]
pub async fn rate_strategy(
    strategy_id: String,
    rating: u8,
    review: Option<String>,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<(), String>

#[tauri::command]
pub async fn get_my_published_strategies(
    wallet_address: String,
    db: State<'_, SqlitePool>
) -> Result<Vec<PublishedStrategy>, String>

#[tauri::command]
pub async fn get_my_subscriptions(
    wallet_address: String,
    db: State<'_, SqlitePool>
) -> Result<Vec<SubscribedStrategy>, String>
```

**Implementation Sequence:**
1. Design strategy publication workflow
2. Implement database schema and persistence
3. Create strategy browsing and filtering
4. Add rating and review system
5. Implement subscription management (free first, paid later)
6. Add performance tracking for published strategies
7. Build verification system (platform review for quality)
8. Frontend: Marketplace UI with strategy cards, details pages, ratings

**Monetization Approach:**
- Free strategies: No payment required
- Paid strategies: SOL/USDC payment to strategy author
- Platform fee: 10% on paid strategy subscriptions
- Payment handling: Solana transactions, not fiat

---

### Feature 3.2: Trader Leaderboards & Rankings
**Complexity:** Medium
**Dependencies:** User profiles, trading history (from paper/copy trading)
**Value Proposition:** Gamification and social proof via competitive rankings based on verifiable performance

#### Technical Approach
**New Modules:**
- `src-tauri/src/social/leaderboard.rs` - Leaderboard calculation and ranking
- `src-tauri/src/social/achievements.rs` - Trading achievements and badges
- `src-tauri/src/social/trader_profiles.rs` - Public trader profiles

**Database Schema:**
```sql
CREATE TABLE trader_profiles (
    id TEXT PRIMARY KEY,
    wallet_address TEXT NOT NULL UNIQUE,
    username TEXT UNIQUE,
    display_name TEXT,
    bio TEXT,
    avatar_url TEXT,
    is_verified BOOLEAN DEFAULT 0,
    is_public BOOLEAN DEFAULT 1,
    total_trades INTEGER DEFAULT 0,
    total_volume REAL DEFAULT 0,
    win_rate REAL DEFAULT 0,
    total_pnl REAL DEFAULT 0,
    follower_count INTEGER DEFAULT 0,
    following_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE leaderboard_rankings (
    id TEXT PRIMARY KEY,
    trader_id TEXT NOT NULL,
    ranking_type TEXT NOT NULL,       -- 'pnl_7d', 'pnl_30d', 'win_rate', 'volume', 'roi'
    rank INTEGER NOT NULL,
    score REAL NOT NULL,
    percentile REAL NOT NULL,
    calculated_at TEXT NOT NULL,
    FOREIGN KEY (trader_id) REFERENCES trader_profiles(id),
    UNIQUE(ranking_type, rank, calculated_at)
);

CREATE TABLE achievements (
    id TEXT PRIMARY KEY,
    achievement_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT NOT NULL,
    rarity TEXT NOT NULL,             -- 'common', 'rare', 'epic', 'legendary'
    criteria TEXT NOT NULL            -- JSON definition
);

CREATE TABLE trader_achievements (
    id TEXT PRIMARY KEY,
    trader_id TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    unlocked_at TEXT NOT NULL,
    FOREIGN KEY (trader_id) REFERENCES trader_profiles(id),
    FOREIGN KEY (achievement_id) REFERENCES achievements(id),
    UNIQUE(trader_id, achievement_id)
);

CREATE TABLE social_follows (
    id TEXT PRIMARY KEY,
    follower_wallet TEXT NOT NULL,
    following_wallet TEXT NOT NULL,
    followed_at TEXT NOT NULL,
    UNIQUE(follower_wallet, following_wallet)
);

CREATE INDEX idx_rankings_type ON leaderboard_rankings(ranking_type, rank);
CREATE INDEX idx_trader_profiles_username ON trader_profiles(username);
CREATE INDEX idx_follows_follower ON social_follows(follower_wallet);
CREATE INDEX idx_follows_following ON social_follows(following_wallet);
```

**Leaderboard Categories:**
1. **7-Day P&L** - Highest profits in last 7 days
2. **30-Day P&L** - Highest profits in last 30 days
3. **Win Rate** - Highest % of winning trades (min 10 trades)
4. **Trading Volume** - Highest total volume traded
5. **ROI** - Best return on investment percentage
6. **Consistency** - Lowest volatility in returns

**Achievement Examples:**
- "First Trade" - Complete your first trade
- "Perfect Week" - 100% win rate for 7 days (min 5 trades)
- "Diamond Hands" - Hold a position through 50%+ drawdown and exit profitably
- "Volume King" - $1M+ trading volume
- "Strategist" - Publish 5+ strategies with 4+ star rating
- "Social Butterfly" - 100+ followers

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn get_leaderboard(
    ranking_type: String,
    timeframe: String,
    limit: usize,
    offset: usize,
    db: State<'_, SqlitePool>
) -> Result<Vec<LeaderboardEntry>, String>

#[tauri::command]
pub async fn get_trader_profile(
    wallet_or_username: String,
    db: State<'_, SqlitePool>
) -> Result<TraderProfile, String>

#[tauri::command]
pub async fn update_trader_profile(
    username: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    is_public: Option<bool>,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<TraderProfile, String>

#[tauri::command]
pub async fn follow_trader(
    following_wallet: String,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<(), String>

#[tauri::command]
pub async fn get_trader_achievements(
    trader_id: String,
    db: State<'_, SqlitePool>
) -> Result<Vec<Achievement>, String>

#[tauri::command]
pub async fn get_my_rank(
    wallet_address: String,
    ranking_type: String,
    db: State<'_, SqlitePool>
) -> Result<MyRankInfo, String>
```

**Implementation Sequence:**
1. Create trader profile system
2. Implement leaderboard calculation (batch job every hour)
3. Design achievement criteria and unlock logic
4. Add follow/unfollow functionality
5. Build achievement notification system
6. Create profile verification process (for public figures)
7. Frontend: Leaderboard page, profile pages, achievement showcase

**Privacy Considerations:**
- Traders can opt out of leaderboards (private profiles)
- Wallet addresses hidden for privacy (show username only)
- Performance data aggregated from consented sources only

---

### Feature 3.3: Enhanced Copy Trading (V2)
**Complexity:** Medium
**Dependencies:** Existing copy trading infrastructure, trader profiles
**Value Proposition:** Improved copy trading with partial position sizing, risk limits, and auto-stop conditions

#### Technical Approach
**Extends:** Existing `src-tauri/src/trading/copy_trading.rs`

**New Features:**
1. **Partial Position Sizing** - Copy 50% of leader's position size
2. **Risk Limits** - Max $ per trade, max total exposure
3. **Auto-Stop Conditions** - Stop copying after X losing trades or Y% drawdown
4. **Selective Copying** - Copy only specific tokens or strategies
5. **Copy Delay** - Introduce delay to avoid front-running concerns

**Database Schema Extensions:**
```sql
-- Add to existing copy_trading tables
ALTER TABLE copy_trade_configs ADD COLUMN position_multiplier REAL DEFAULT 1.0;
ALTER TABLE copy_trade_configs ADD COLUMN max_trade_amount REAL;
ALTER TABLE copy_trade_configs ADD COLUMN max_total_exposure REAL;
ALTER TABLE copy_trade_configs ADD COLUMN auto_stop_enabled BOOLEAN DEFAULT 0;
ALTER TABLE copy_trade_configs ADD COLUMN auto_stop_loss_count INTEGER;
ALTER TABLE copy_trade_configs ADD COLUMN auto_stop_drawdown_percent REAL;
ALTER TABLE copy_trade_configs ADD COLUMN token_whitelist TEXT;  -- JSON array
ALTER TABLE copy_trade_configs ADD COLUMN token_blacklist TEXT;  -- JSON array
ALTER TABLE copy_trade_configs ADD COLUMN copy_delay_seconds INTEGER DEFAULT 0;

CREATE TABLE copy_trade_performance (
    id TEXT PRIMARY KEY,
    config_id TEXT NOT NULL,
    total_trades INTEGER DEFAULT 0,
    winning_trades INTEGER DEFAULT 0,
    losing_trades INTEGER DEFAULT 0,
    total_pnl REAL DEFAULT 0,
    current_drawdown_percent REAL DEFAULT 0,
    status TEXT NOT NULL,             -- 'active', 'paused', 'auto_stopped'
    last_trade_at TEXT,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (config_id) REFERENCES copy_trade_configs(id)
);
```

**Tauri Commands (Extensions):**
```rust
#[tauri::command]
pub async fn create_copy_config_v2(
    leader_wallet: String,
    position_multiplier: f64,
    risk_limits: RiskLimits,
    auto_stop: Option<AutoStopConfig>,
    token_filters: Option<TokenFilters>,
    copy_delay_seconds: Option<u64>,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<CopyTradeConfig, String>

#[tauri::command]
pub async fn get_copy_performance(
    config_id: String,
    db: State<'_, SqlitePool>
) -> Result<CopyPerformance, String>

#[tauri::command]
pub async fn update_copy_config_risk_limits(
    config_id: String,
    risk_limits: RiskLimits,
    db: State<'_, SqlitePool>
) -> Result<(), String>

#[tauri::command]
pub async fn pause_copy_config(
    config_id: String,
    db: State<'_, SqlitePool>
) -> Result<(), String>

#[tauri::command]
pub async fn resume_copy_config(
    config_id: String,
    db: State<'_, SqlitePool>
) -> Result<(), String>
```

**Implementation Sequence:**
1. Extend existing CopyTradeConfig with new fields
2. Implement position size multiplier logic
3. Add risk limit checks before copying trades
4. Build auto-stop monitoring (check after each trade)
5. Add token whitelist/blacklist filtering
6. Implement copy delay mechanism
7. Create performance tracking for copy configs
8. Frontend: Enhanced copy config UI with advanced settings

---

## Tier 1: Security Enhancements

### Feature 4.1: Hardware Wallet Support (Ledger)
**Complexity:** Large
**Dependencies:** Wallet infrastructure
**Value Proposition:** Industry-standard security for high-value accounts via hardware wallet integration

#### Technical Approach
**New Modules:**
- `src-tauri/src/security/ledger.rs` - Ledger device communication
- `src-tauri/src/security/hardware_wallet_manager.rs` - Abstract interface for hardware wallets

**Ledger Integration:**
- **Library:** `ledger-transport-hid` (USB HID communication)
- **Solana App:** Use Ledger Solana app for transaction signing
- **Derivation Paths:** Standard Solana paths (m/44'/501'/0'/0')

**Database Schema:**
```sql
CREATE TABLE hardware_wallets (
    id TEXT PRIMARY KEY,
    wallet_type TEXT NOT NULL,        -- 'ledger', 'trezor' (future)
    device_model TEXT NOT NULL,       -- 'nano_s', 'nano_x', 'nano_s_plus'
    public_key TEXT NOT NULL,
    derivation_path TEXT NOT NULL,
    is_connected BOOLEAN DEFAULT 0,
    last_connected_at TEXT,
    added_at TEXT NOT NULL
);

CREATE TABLE hardware_wallet_transactions (
    id TEXT PRIMARY KEY,
    hardware_wallet_id TEXT NOT NULL,
    tx_type TEXT NOT NULL,            -- 'transfer', 'swap', 'stake', etc.
    unsigned_tx TEXT NOT NULL,        -- Base64 encoded transaction
    signed_tx TEXT,                   -- Base64 encoded signed transaction
    tx_signature TEXT,
    status TEXT NOT NULL,             -- 'pending', 'signed', 'confirmed', 'failed'
    error_message TEXT,
    created_at TEXT NOT NULL,
    signed_at TEXT,
    confirmed_at TEXT,
    FOREIGN KEY (hardware_wallet_id) REFERENCES hardware_wallets(id)
);

CREATE INDEX idx_hw_wallets_pubkey ON hardware_wallets(public_key);
CREATE INDEX idx_hw_txs_wallet ON hardware_wallet_transactions(hardware_wallet_id, created_at DESC);
```

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn detect_hardware_wallets(
    hw_manager: State<'_, HardwareWalletManager>
) -> Result<Vec<DetectedDevice>, String>

#[tauri::command]
pub async fn connect_ledger(
    derivation_path: Option<String>,
    hw_manager: State<'_, HardwareWalletManager>,
    db: State<'_, SqlitePool>
) -> Result<HardwareWallet, String>

#[tauri::command]
pub async fn get_ledger_public_key(
    derivation_path: String,
    hw_manager: State<'_, HardwareWalletManager>
) -> Result<String, String>

#[tauri::command]
pub async fn sign_with_ledger(
    transaction: String,  // Base64 encoded
    derivation_path: String,
    hw_manager: State<'_, HardwareWalletManager>,
    db: State<'_, SqlitePool>
) -> Result<SignedTransaction, String>

#[tauri::command]
pub async fn verify_ledger_address(
    derivation_path: String,
    hw_manager: State<'_, HardwareWalletManager>
) -> Result<bool, String>  // Shows address on device for verification
```

**Implementation Sequence:**
1. Research Ledger Solana app APDU commands
2. Implement USB HID communication layer
3. Add public key derivation
4. Implement transaction signing flow
5. Add device detection and connection management
6. Build UI for device connection and verification
7. Test with Ledger Nano S, Nano X, Nano S Plus
8. Add error handling for disconnections

**Security Considerations:**
- Never store private keys (handled by device)
- Verify addresses on device screen
- Rate limit signing requests to prevent abuse
- Implement timeout for pending signatures

---

### Feature 4.2: Transaction Simulation & Risk Analysis
**Complexity:** Medium
**Dependencies:** Solana RPC, DeFi protocol knowledge
**Value Proposition:** Simulate transactions before execution to detect scams, excessive slippage, and unexpected outcomes

#### Technical Approach
**New Modules:**
- `src-tauri/src/security/tx_simulator.rs` - Transaction simulation engine
- `src-tauri/src/security/risk_analyzer.rs` - Risk scoring and detection
- `src-tauri/src/security/scam_detector.rs` - Known scam pattern detection

**Simulation Strategy:**
- Use Solana RPC `simulateTransaction` endpoint
- Parse simulation results for balance changes, account creations
- Detect suspicious patterns (unexpected token mints, authority changes)

**Database Schema:**
```sql
CREATE TABLE transaction_simulations (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    tx_type TEXT NOT NULL,
    unsigned_tx TEXT NOT NULL,        -- Base64 encoded
    simulation_result TEXT NOT NULL,  -- JSON of simulation response
    risk_score REAL NOT NULL,         -- 0-100 (higher = riskier)
    risk_factors TEXT NOT NULL,       -- JSON array of detected risks
    balance_changes TEXT NOT NULL,    -- JSON of expected balance changes
    account_creations TEXT,           -- JSON of new accounts created
    program_calls TEXT NOT NULL,      -- JSON of programs invoked
    simulated_at TEXT NOT NULL
);

CREATE TABLE known_scams (
    id TEXT PRIMARY KEY,
    scam_type TEXT NOT NULL,          -- 'honeypot', 'rug_pull', 'fake_token', etc.
    token_mint TEXT,
    program_id TEXT,
    description TEXT NOT NULL,
    evidence_url TEXT,
    reported_by TEXT,
    confirmed BOOLEAN DEFAULT 0,
    added_at TEXT NOT NULL
);

CREATE TABLE risk_rules (
    id TEXT PRIMARY KEY,
    rule_name TEXT NOT NULL UNIQUE,
    rule_type TEXT NOT NULL,          -- 'pattern', 'threshold', 'blacklist'
    severity TEXT NOT NULL,           -- 'low', 'medium', 'high', 'critical'
    condition TEXT NOT NULL,          -- JSON rule definition
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_simulations_wallet ON transaction_simulations(user_wallet, simulated_at DESC);
CREATE INDEX idx_scams_token ON known_scams(token_mint);
```

**Risk Factors Detected:**
1. **High Slippage** - Execution price differs significantly from expected
2. **Unknown Program** - Transaction calls unfamiliar/unverified program
3. **Authority Transfer** - Transaction transfers token mint/freeze authority
4. **Excessive Fees** - Transaction fee unusually high
5. **Token Blacklisted** - Interacting with known scam token
6. **Suspicious Account Creation** - Creates accounts with unusual parameters
7. **Balance Drain** - Transaction would drain significant % of balance

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn simulate_transaction(
    transaction: String,  // Base64 encoded
    wallet_address: String,
    simulator: State<'_, TxSimulator>,
    db: State<'_, SqlitePool>
) -> Result<SimulationResult, String>

#[tauri::command]
pub async fn analyze_transaction_risk(
    simulation_result: SimulationResult,
    analyzer: State<'_, RiskAnalyzer>,
    db: State<'_, SqlitePool>
) -> Result<RiskAnalysis, String>

#[tauri::command]
pub async fn check_token_safety(
    token_mint: String,
    db: State<'_, SqlitePool>
) -> Result<TokenSafetyInfo, String>

#[tauri::command]
pub async fn report_scam(
    token_mint: Option<String>,
    program_id: Option<String>,
    scam_type: String,
    description: String,
    evidence_url: Option<String>,
    wallet: State<'_, PhantomWallet>,
    db: State<'_, SqlitePool>
) -> Result<(), String>

#[tauri::command]
pub async fn get_simulation_history(
    wallet_address: String,
    limit: usize,
    db: State<'_, SqlitePool>
) -> Result<Vec<SimulationSummary>, String>
```

**Implementation Sequence:**
1. Implement Solana RPC simulateTransaction wrapper
2. Parse simulation results (logs, account changes, errors)
3. Build risk scoring algorithm
4. Create risk factor detection rules
5. Integrate known scam database (seed with community reports)
6. Add scam reporting mechanism
7. Build UI for simulation results display (red/yellow/green indicators)
8. Add user preferences for auto-simulation (on/off)

**User Experience:**
- Auto-simulate every transaction before signing
- Show clear risk warnings with explanations
- Allow users to proceed with acknowledgment for medium-risk txs
- Block critical-risk transactions with override option (advanced users)

---

### Feature 4.3: Security Audit Log
**Complexity:** Small
**Dependencies:** Existing auth system
**Value Proposition:** Comprehensive audit trail of all security events for forensics and compliance

#### Technical Approach
**New Modules:**
- `src-tauri/src/security/audit_logger.rs` - Centralized audit logging
- `src-tauri/src/security/audit_viewer.rs` - Query and filter audit logs

**Database Schema:**
```sql
CREATE TABLE security_audit_log (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,         -- 'login', 'logout', 'tx_signed', 'settings_changed', etc.
    user_wallet TEXT NOT NULL,
    severity TEXT NOT NULL,           -- 'info', 'warning', 'critical'
    description TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    device_fingerprint TEXT,
    metadata TEXT,                    -- JSON with event-specific data
    timestamp TEXT NOT NULL
);

CREATE TABLE suspicious_activity (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    activity_type TEXT NOT NULL,      -- 'multiple_failed_logins', 'unusual_location', etc.
    risk_level TEXT NOT NULL,         -- 'low', 'medium', 'high'
    details TEXT NOT NULL,
    auto_action_taken TEXT,           -- 'none', 'account_locked', 'mfa_required'
    resolved BOOLEAN DEFAULT 0,
    detected_at TEXT NOT NULL,
    resolved_at TEXT
);

CREATE INDEX idx_audit_wallet ON security_audit_log(user_wallet, timestamp DESC);
CREATE INDEX idx_audit_type ON security_audit_log(event_type, timestamp DESC);
CREATE INDEX idx_suspicious_wallet ON suspicious_activity(user_wallet, resolved);
```

**Logged Events:**
- Authentication (login, logout, MFA attempts, biometric checks)
- Transactions (signed, rejected, simulated)
- Settings changes (2FA enabled/disabled, session timeout changed)
- Wallet connections (new wallet added, removed)
- Permission changes (voice access granted/revoked)
- Data exports (CSV downloads, API access)
- Security alerts (suspicious login, unusual activity)

**Tauri Commands:**
```rust
#[tauri::command]
pub async fn get_audit_log(
    wallet_address: String,
    event_type: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: usize,
    offset: usize,
    db: State<'_, SqlitePool>
) -> Result<Vec<AuditLogEntry>, String>

#[tauri::command]
pub async fn get_suspicious_activity(
    wallet_address: String,
    db: State<'_, SqlitePool>
) -> Result<Vec<SuspiciousActivity>, String>

#[tauri::command]
pub async fn export_audit_log(
    wallet_address: String,
    start_date: String,
    end_date: String,
    format: String,  // 'csv', 'json'
    db: State<'_, SqlitePool>
) -> Result<String, String>  // Returns file path

#[tauri::command]
pub async fn mark_suspicious_activity_resolved(
    activity_id: String,
    db: State<'_, SqlitePool>
) -> Result<(), String>
```

**Implementation Sequence:**
1. Design audit event schema and taxonomy
2. Implement centralized AuditLogger trait
3. Integrate logging into all security-sensitive operations
4. Add suspicious activity detection rules
5. Build audit log viewer with filtering
6. Add export functionality (CSV, JSON)
7. Frontend: Security settings page with audit log viewer
8. Add retention policy (auto-delete logs older than 1 year)

**Integration Points:**
- Call `audit_logger.log()` in:
  - `src-tauri/src/auth/session_manager.rs` (login/logout)
  - `src-tauri/src/auth/two_factor.rs` (2FA events)
  - `src-tauri/src/wallet/phantom.rs` (transaction signing)
  - `src-tauri/src/security/ledger.rs` (hardware wallet events)
  - `src-tauri/src/trading/*` (trade executions)

---

## Implementation Roadmap & Sequencing

### Phase 1: Foundation (Weeks 1-2)
**Goal:** Core infrastructure for AI, DeFi, Social, Security

**Tasks:**
1. Create database migrations for all new schemas
2. Set up module structure (`src-tauri/src/ai/`, `src-tauri/src/defi/`, `src-tauri/src/social/`, `src-tauri/src/security/`)
3. Define shared types and traits
4. Implement audit logging infrastructure
5. Add feature flags for progressive rollout

**Deliverables:**
- Database schema created
- Module scaffolding complete
- Audit logging operational

---

### Phase 2: Quick Wins (Weeks 3-4)
**Goal:** Deliver visible features early to demonstrate progress

**Priority Features:**
1. **Security Audit Log (4.3)** - Small, high-value, tests audit infrastructure
2. **Jupiter Swap Integration (2.1)** - Core DeFi feature, builds on existing wallet code
3. **Trader Profiles (3.2)** - Social foundation, enables other social features
4. **Sentiment Analysis (1.1)** - AI foundation, high user interest

**Deliverables:**
- Users can view security audit logs
- Users can swap tokens via Jupiter with best rates
- Users can create public profiles
- Users can see sentiment scores for tokens

---

### Phase 3: Core Value (Weeks 5-8)
**Goal:** Implement high-complexity, high-value features

**Priority Features:**
1. **Transaction Simulation (4.2)** - Critical security, prevents losses
2. **Strategy Backtester (1.3)** - Core AI feature, enables strategy development
3. **Yield Farming Dashboard (2.2)** - Core DeFi feature, high retention
4. **Strategy Marketplace (3.1)** - Core social feature, network effects

**Deliverables:**
- All transactions simulated before signing
- Users can backtest custom strategies
- Users can track yield positions across protocols
- Users can publish and subscribe to strategies

---

### Phase 4: Advanced Features (Weeks 9-12)
**Goal:** Complete feature set with advanced capabilities

**Priority Features:**
1. **Hardware Wallet Support (4.1)** - Security for power users
2. **Predictive Modeling (1.2)** - Advanced AI, differentiating feature
3. **LP Analytics (2.3)** - Advanced DeFi, impermanent loss calculations
4. **Enhanced Copy Trading (3.3)** - Advanced social, builds on existing copy trading
5. **Leaderboards (3.2)** - Gamification, drives engagement

**Deliverables:**
- Users can sign transactions with Ledger
- Users get ML-based price predictions
- Users analyze LP positions with IL tracking
- Users copy trades with advanced risk controls
- Users compete on leaderboards

---

## Testing & Quality Assurance

### Testing Strategy
Each feature must include:

1. **Unit Tests** (Rust)
   - Test core logic in isolation
   - Mock external dependencies (APIs, database)
   - Target: 80%+ code coverage

2. **Integration Tests** (Rust)
   - Test Tauri commands end-to-end
   - Use test database (SQLite in-memory)
   - Test error handling and edge cases

3. **Frontend Tests** (TypeScript)
   - React Testing Library for components
   - MSW for API mocking
   - Test user interactions and state management

4. **E2E Tests** (Playwright/Tauri)
   - Critical user flows (swap, backtest, publish strategy)
   - Test across platforms (Windows, macOS, Linux)

### Quality Gates
Before merging to main:
- ✅ All tests passing
- ✅ Zero new compilation warnings
- ✅ Feature flag implemented
- ✅ Documentation updated (inline comments + user docs)
- ✅ Security review (for security-sensitive features)

---

## Risk Mitigation

### Technical Risks

**Risk:** ML models too large for desktop app
**Mitigation:** Use lightweight models (smartcore), model quantization, lazy loading

**Risk:** Ledger integration fails on some OS
**Mitigation:** Thorough cross-platform testing, fallback to software wallet

**Risk:** Jupiter API rate limits
**Mitigation:** Implement request caching, rate limiting, backoff strategy

**Risk:** Database grows too large with audit logs
**Mitigation:** Retention policy, log rotation, compression

### Product Risks

**Risk:** Users don't adopt strategy marketplace
**Mitigation:** Seed with high-quality strategies, influencer partnerships

**Risk:** Predictions are inaccurate, damage trust
**Mitigation:** Prominent disclaimers, display accuracy metrics, conservative confidence intervals

**Risk:** Transaction simulation false positives
**Mitigation:** Tunable risk thresholds, user education, "override" option

---

## Dependencies & Integration Points

### External Dependencies (New)
- `tch-rs` or `smartcore` - Machine learning
- `ledger-transport-hid` - Ledger hardware wallet
- `egg-mode` or `twitter-v2` - Twitter API (sentiment)
- `roux` - Reddit API (extend existing)
- `tower` - Rate limiting

### Internal Integration Points
- **Extend:** `src-tauri/src/insiders/wallet_monitor.rs` - Add sentiment from on-chain data
- **Extend:** `src-tauri/src/trading/copy_trading.rs` - Add risk limits and advanced features
- **Extend:** `src-tauri/src/wallet/phantom.rs` - Add hardware wallet option
- **Extend:** `src-tauri/src/auth/session_manager.rs` - Add audit logging hooks
- **Reuse:** Existing SQLite patterns, Tauri command structure, Zustand stores

---

## Success Metrics

### Quantitative KPIs
- **AI Features:** 30%+ of users enable sentiment analysis, 20%+ backtest strategies
- **DeFi Features:** 50%+ of swaps use Jupiter, 10%+ users track yield positions
- **Social Features:** 100+ published strategies, 500+ strategy subscriptions
- **Security Features:** 80%+ transactions simulated, 10%+ users connect Ledger

### Qualitative Goals
- Zero critical security vulnerabilities introduced
- Positive user feedback on new features
- Maintainable, well-documented code
- Successful parallel development without merge conflicts

---

## Conclusion

This roadmap delivers a balanced feature set across AI, DeFi, Social, and Security pillars. Features are designed to:
- Build incrementally on existing infrastructure
- Enable parallel development with minimal conflicts
- Deliver early wins while working toward complex features
- Maintain security and code quality standards

The modular architecture ensures features can be developed, tested, and deployed independently, reducing risk and enabling rapid iteration based on user feedback.

**Next Steps:**
1. Review and approve roadmap
2. Create GitHub issues for Phase 1 tasks
3. Assign initial features to development streams
4. Begin implementation with Foundation phase
