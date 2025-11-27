-- Eclipse Market Pro - Feature Roadmap Database Migrations
-- Phase 1: Foundation - All schema definitions for AI, DeFi, Social, and Security features

-- ============================================================================
-- AI TRADING FEATURES
-- ============================================================================

-- Feature 1.1: Sentiment Analysis
CREATE TABLE IF NOT EXISTS sentiment_scores (
    id TEXT PRIMARY KEY,
    token_mint TEXT NOT NULL,
    token_symbol TEXT,
    sentiment_score REAL NOT NULL CHECK (sentiment_score >= -1.0 AND sentiment_score <= 1.0),
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    source TEXT NOT NULL CHECK (source IN ('twitter', 'reddit', 'discord', 'onchain')),
    sample_size INTEGER,
    positive_mentions INTEGER DEFAULT 0,
    negative_mentions INTEGER DEFAULT 0,
    neutral_mentions INTEGER DEFAULT 0,
    timestamp TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_sentiment_token ON sentiment_scores(token_mint, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_sentiment_source ON sentiment_scores(source, timestamp DESC);

-- Feature 1.2: Predictive Price Modeling
CREATE TABLE IF NOT EXISTS price_predictions (
    id TEXT PRIMARY KEY,
    token_mint TEXT NOT NULL,
    prediction_timestamp TEXT NOT NULL,
    target_timestamp TEXT NOT NULL,
    predicted_price REAL NOT NULL CHECK (predicted_price >= 0),
    confidence_lower REAL NOT NULL CHECK (confidence_lower >= 0),
    confidence_upper REAL NOT NULL CHECK (confidence_upper >= confidence_lower),
    actual_price REAL,
    model_version TEXT NOT NULL,
    features TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_predictions_token ON price_predictions(token_mint, prediction_timestamp DESC);

CREATE TABLE IF NOT EXISTS model_performance (
    id TEXT PRIMARY KEY,
    model_version TEXT NOT NULL,
    token_mint TEXT,
    timeframe TEXT NOT NULL CHECK (timeframe IN ('1h', '4h', '24h', '7d')),
    mae REAL NOT NULL,
    rmse REAL NOT NULL,
    accuracy_percent REAL NOT NULL CHECK (accuracy_percent >= 0 AND accuracy_percent <= 100),
    total_predictions INTEGER NOT NULL DEFAULT 0,
    evaluated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Feature 1.3: Strategy Backtesting
CREATE TABLE IF NOT EXISTS backtest_results (
    id TEXT PRIMARY KEY,
    strategy_name TEXT NOT NULL,
    strategy_config TEXT NOT NULL,
    token_mint TEXT,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    initial_capital REAL NOT NULL CHECK (initial_capital > 0),
    final_capital REAL NOT NULL CHECK (final_capital >= 0),
    total_return_percent REAL NOT NULL,
    sharpe_ratio REAL,
    max_drawdown_percent REAL,
    win_rate REAL CHECK (win_rate >= 0 AND win_rate <= 100),
    total_trades INTEGER DEFAULT 0,
    winning_trades INTEGER DEFAULT 0,
    losing_trades INTEGER DEFAULT 0,
    avg_win REAL,
    avg_loss REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS backtest_trades (
    id TEXT PRIMARY KEY,
    backtest_id TEXT NOT NULL,
    token_mint TEXT NOT NULL,
    entry_time TEXT NOT NULL,
    entry_price REAL NOT NULL CHECK (entry_price > 0),
    exit_time TEXT,
    exit_price REAL CHECK (exit_price > 0),
    position_size REAL NOT NULL CHECK (position_size > 0),
    pnl REAL,
    pnl_percent REAL,
    exit_reason TEXT CHECK (exit_reason IN ('stop_loss', 'take_profit', 'signal', 'end_of_test')),
    FOREIGN KEY (backtest_id) REFERENCES backtest_results(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_backtest_trades ON backtest_trades(backtest_id);

-- ============================================================================
-- DEFI INTEGRATIONS
-- ============================================================================

-- Feature 2.1: Jupiter Aggregator
CREATE TABLE IF NOT EXISTS jupiter_swaps (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    input_mint TEXT NOT NULL,
    output_mint TEXT NOT NULL,
    input_amount REAL NOT NULL CHECK (input_amount > 0),
    output_amount REAL NOT NULL CHECK (output_amount >= 0),
    quoted_price REAL NOT NULL CHECK (quoted_price > 0),
    executed_price REAL NOT NULL CHECK (executed_price > 0),
    slippage_bps INTEGER NOT NULL CHECK (slippage_bps >= 0),
    route TEXT NOT NULL,
    tx_signature TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending', 'confirmed', 'failed')) DEFAULT 'pending',
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    confirmed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_swaps_wallet ON jupiter_swaps(user_wallet, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_swaps_status ON jupiter_swaps(status, created_at DESC);

CREATE TABLE IF NOT EXISTS jupiter_limit_orders (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    input_mint TEXT NOT NULL,
    output_mint TEXT NOT NULL,
    input_amount REAL NOT NULL CHECK (input_amount > 0),
    target_price REAL NOT NULL CHECK (target_price > 0),
    expiry_time TEXT,
    status TEXT NOT NULL CHECK (status IN ('open', 'filled', 'cancelled', 'expired')) DEFAULT 'open',
    filled_amount REAL DEFAULT 0 CHECK (filled_amount >= 0),
    tx_signature TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    filled_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_orders_wallet ON jupiter_limit_orders(user_wallet, status);

-- Feature 2.2: Yield Farming Dashboard
CREATE TABLE IF NOT EXISTS yield_positions (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    protocol TEXT NOT NULL CHECK (protocol IN ('marinade', 'jito', 'kamino', 'raydium', 'orca')),
    position_type TEXT NOT NULL CHECK (position_type IN ('staking', 'lending', 'liquidity_pool')),
    token_mint TEXT NOT NULL,
    amount REAL NOT NULL CHECK (amount >= 0),
    value_usd REAL NOT NULL CHECK (value_usd >= 0),
    entry_price REAL NOT NULL CHECK (entry_price > 0),
    current_apy REAL,
    rewards_earned REAL DEFAULT 0 CHECK (rewards_earned >= 0),
    rewards_claimed REAL DEFAULT 0 CHECK (rewards_claimed >= 0),
    last_update TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_positions_wallet ON yield_positions(user_wallet, protocol);

CREATE TABLE IF NOT EXISTS apy_history (
    id TEXT PRIMARY KEY,
    protocol TEXT NOT NULL,
    position_type TEXT NOT NULL,
    token_mint TEXT NOT NULL,
    apy REAL NOT NULL,
    tvl REAL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_apy_protocol ON apy_history(protocol, timestamp DESC);

CREATE TABLE IF NOT EXISTS impermanent_loss (
    id TEXT PRIMARY KEY,
    lp_position_id TEXT NOT NULL,
    token_a_mint TEXT NOT NULL,
    token_b_mint TEXT NOT NULL,
    entry_ratio REAL NOT NULL CHECK (entry_ratio > 0),
    current_ratio REAL NOT NULL CHECK (current_ratio > 0),
    il_percent REAL NOT NULL,
    calculated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (lp_position_id) REFERENCES yield_positions(id) ON DELETE CASCADE
);

-- Feature 2.3: Liquidity Pool Analytics
CREATE TABLE IF NOT EXISTS lp_analytics (
    id TEXT PRIMARY KEY,
    position_id TEXT NOT NULL,
    token_a_mint TEXT NOT NULL,
    token_b_mint TEXT NOT NULL,
    pool_address TEXT NOT NULL,
    il_24h REAL NOT NULL DEFAULT 0,
    il_7d REAL NOT NULL DEFAULT 0,
    il_30d REAL NOT NULL DEFAULT 0,
    fees_earned_24h REAL NOT NULL DEFAULT 0 CHECK (fees_earned_24h >= 0),
    fees_earned_7d REAL NOT NULL DEFAULT 0 CHECK (fees_earned_7d >= 0),
    fees_earned_30d REAL NOT NULL DEFAULT 0 CHECK (fees_earned_30d >= 0),
    net_pnl REAL NOT NULL DEFAULT 0,
    pool_tvl REAL,
    pool_volume_24h REAL,
    fee_apr REAL,
    analyzed_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (position_id) REFERENCES yield_positions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS rebalance_suggestions (
    id TEXT PRIMARY KEY,
    position_id TEXT NOT NULL,
    suggestion_type TEXT NOT NULL CHECK (suggestion_type IN ('add_liquidity', 'remove_liquidity', 'rebalance')),
    reason TEXT NOT NULL,
    confidence_score REAL NOT NULL CHECK (confidence_score >= 0 AND confidence_score <= 1),
    expected_benefit_percent REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (position_id) REFERENCES yield_positions(id) ON DELETE CASCADE
);

-- ============================================================================
-- SOCIAL TRADING CAPABILITIES
-- ============================================================================

-- Feature 3.1: Strategy Marketplace
CREATE TABLE IF NOT EXISTS published_strategies (
    id TEXT PRIMARY KEY,
    author_wallet TEXT NOT NULL,
    author_username TEXT,
    strategy_name TEXT NOT NULL,
    description TEXT NOT NULL,
    strategy_config TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('momentum', 'mean_reversion', 'breakout', 'trend_following', 'scalping', 'swing', 'other')),
    price_type TEXT NOT NULL CHECK (price_type IN ('free', 'paid')) DEFAULT 'free',
    price_amount REAL CHECK (price_amount >= 0),
    currency TEXT CHECK (currency IN ('SOL', 'USDC')),
    is_public BOOLEAN NOT NULL DEFAULT 1,
    total_subscribers INTEGER DEFAULT 0 CHECK (total_subscribers >= 0),
    total_ratings INTEGER DEFAULT 0 CHECK (total_ratings >= 0),
    avg_rating REAL DEFAULT 0 CHECK (avg_rating >= 0 AND avg_rating <= 5),
    verified BOOLEAN DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_strategies_author ON published_strategies(author_wallet);
CREATE INDEX IF NOT EXISTS idx_strategies_category ON published_strategies(category, avg_rating DESC);

CREATE TABLE IF NOT EXISTS strategy_performance (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    timeframe TEXT NOT NULL CHECK (timeframe IN ('7d', '30d', '90d', 'all')),
    total_return_percent REAL NOT NULL,
    sharpe_ratio REAL,
    max_drawdown_percent REAL,
    win_rate REAL CHECK (win_rate >= 0 AND win_rate <= 100),
    total_trades INTEGER DEFAULT 0,
    calculated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (strategy_id) REFERENCES published_strategies(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS strategy_subscriptions (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    subscriber_wallet TEXT NOT NULL,
    subscription_type TEXT NOT NULL CHECK (subscription_type IN ('free', 'paid')),
    status TEXT NOT NULL CHECK (status IN ('active', 'cancelled', 'expired')) DEFAULT 'active',
    payment_tx TEXT,
    subscribed_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT,
    FOREIGN KEY (strategy_id) REFERENCES published_strategies(id) ON DELETE CASCADE,
    UNIQUE(strategy_id, subscriber_wallet)
);

CREATE INDEX IF NOT EXISTS idx_subscriptions_user ON strategy_subscriptions(subscriber_wallet);

CREATE TABLE IF NOT EXISTS strategy_ratings (
    id TEXT PRIMARY KEY,
    strategy_id TEXT NOT NULL,
    rater_wallet TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    verified_user BOOLEAN DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (strategy_id) REFERENCES published_strategies(id) ON DELETE CASCADE,
    UNIQUE(strategy_id, rater_wallet)
);

-- Feature 3.2: Trader Leaderboards & Rankings
CREATE TABLE IF NOT EXISTS trader_profiles (
    id TEXT PRIMARY KEY,
    wallet_address TEXT NOT NULL UNIQUE,
    username TEXT UNIQUE,
    display_name TEXT,
    bio TEXT,
    avatar_url TEXT,
    is_verified BOOLEAN DEFAULT 0,
    is_public BOOLEAN DEFAULT 1,
    total_trades INTEGER DEFAULT 0 CHECK (total_trades >= 0),
    total_volume REAL DEFAULT 0 CHECK (total_volume >= 0),
    win_rate REAL DEFAULT 0 CHECK (win_rate >= 0 AND win_rate <= 100),
    total_pnl REAL DEFAULT 0,
    follower_count INTEGER DEFAULT 0 CHECK (follower_count >= 0),
    following_count INTEGER DEFAULT 0 CHECK (following_count >= 0),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_trader_profiles_username ON trader_profiles(username);
CREATE INDEX IF NOT EXISTS idx_trader_profiles_wallet ON trader_profiles(wallet_address);

CREATE TABLE IF NOT EXISTS leaderboard_rankings (
    id TEXT PRIMARY KEY,
    trader_id TEXT NOT NULL,
    ranking_type TEXT NOT NULL CHECK (ranking_type IN ('pnl_7d', 'pnl_30d', 'win_rate', 'volume', 'roi', 'consistency')),
    rank INTEGER NOT NULL CHECK (rank > 0),
    score REAL NOT NULL,
    percentile REAL NOT NULL CHECK (percentile >= 0 AND percentile <= 100),
    calculated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (trader_id) REFERENCES trader_profiles(id) ON DELETE CASCADE,
    UNIQUE(ranking_type, rank, calculated_at)
);

CREATE INDEX IF NOT EXISTS idx_rankings_type ON leaderboard_rankings(ranking_type, rank);

CREATE TABLE IF NOT EXISTS achievements (
    id TEXT PRIMARY KEY,
    achievement_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT NOT NULL,
    rarity TEXT NOT NULL CHECK (rarity IN ('common', 'rare', 'epic', 'legendary')),
    criteria TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trader_achievements (
    id TEXT PRIMARY KEY,
    trader_id TEXT NOT NULL,
    achievement_id TEXT NOT NULL,
    unlocked_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (trader_id) REFERENCES trader_profiles(id) ON DELETE CASCADE,
    FOREIGN KEY (achievement_id) REFERENCES achievements(id) ON DELETE CASCADE,
    UNIQUE(trader_id, achievement_id)
);

CREATE TABLE IF NOT EXISTS social_follows (
    id TEXT PRIMARY KEY,
    follower_wallet TEXT NOT NULL,
    following_wallet TEXT NOT NULL,
    followed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(follower_wallet, following_wallet)
);

CREATE INDEX IF NOT EXISTS idx_follows_follower ON social_follows(follower_wallet);
CREATE INDEX IF NOT EXISTS idx_follows_following ON social_follows(following_wallet);

-- Feature 3.3: Enhanced Copy Trading V2 (Extensions to existing tables)
-- Note: These are extensions to existing copy_trade tables
CREATE TABLE IF NOT EXISTS copy_trade_performance (
    id TEXT PRIMARY KEY,
    config_id TEXT NOT NULL,
    total_trades INTEGER DEFAULT 0 CHECK (total_trades >= 0),
    winning_trades INTEGER DEFAULT 0 CHECK (winning_trades >= 0),
    losing_trades INTEGER DEFAULT 0 CHECK (losing_trades >= 0),
    total_pnl REAL DEFAULT 0,
    current_drawdown_percent REAL DEFAULT 0,
    status TEXT NOT NULL CHECK (status IN ('active', 'paused', 'auto_stopped')) DEFAULT 'active',
    last_trade_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- SECURITY ENHANCEMENTS
-- ============================================================================

-- Feature 4.1: Hardware Wallet Support
CREATE TABLE IF NOT EXISTS hardware_wallets (
    id TEXT PRIMARY KEY,
    wallet_type TEXT NOT NULL CHECK (wallet_type IN ('ledger', 'trezor')),
    device_model TEXT NOT NULL,
    public_key TEXT NOT NULL,
    derivation_path TEXT NOT NULL,
    is_connected BOOLEAN DEFAULT 0,
    last_connected_at TEXT,
    added_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_hw_wallets_pubkey ON hardware_wallets(public_key);

CREATE TABLE IF NOT EXISTS hardware_wallet_transactions (
    id TEXT PRIMARY KEY,
    hardware_wallet_id TEXT NOT NULL,
    tx_type TEXT NOT NULL,
    unsigned_tx TEXT NOT NULL,
    signed_tx TEXT,
    tx_signature TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending', 'signed', 'confirmed', 'failed')) DEFAULT 'pending',
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    signed_at TEXT,
    confirmed_at TEXT,
    FOREIGN KEY (hardware_wallet_id) REFERENCES hardware_wallets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_hw_txs_wallet ON hardware_wallet_transactions(hardware_wallet_id, created_at DESC);

-- Feature 4.2: Transaction Simulation & Risk Analysis
CREATE TABLE IF NOT EXISTS transaction_simulations (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    tx_type TEXT NOT NULL,
    unsigned_tx TEXT NOT NULL,
    simulation_result TEXT NOT NULL,
    risk_score REAL NOT NULL CHECK (risk_score >= 0 AND risk_score <= 100),
    risk_factors TEXT NOT NULL,
    balance_changes TEXT NOT NULL,
    account_creations TEXT,
    program_calls TEXT NOT NULL,
    simulated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_simulations_wallet ON transaction_simulations(user_wallet, simulated_at DESC);

CREATE TABLE IF NOT EXISTS known_scams (
    id TEXT PRIMARY KEY,
    scam_type TEXT NOT NULL CHECK (scam_type IN ('honeypot', 'rug_pull', 'fake_token', 'phishing', 'other')),
    token_mint TEXT,
    program_id TEXT,
    description TEXT NOT NULL,
    evidence_url TEXT,
    reported_by TEXT,
    confirmed BOOLEAN DEFAULT 0,
    added_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_scams_token ON known_scams(token_mint);
CREATE INDEX IF NOT EXISTS idx_scams_program ON known_scams(program_id);

CREATE TABLE IF NOT EXISTS risk_rules (
    id TEXT PRIMARY KEY,
    rule_name TEXT NOT NULL UNIQUE,
    rule_type TEXT NOT NULL CHECK (rule_type IN ('pattern', 'threshold', 'blacklist')),
    severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    condition TEXT NOT NULL,
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Feature 4.3: Security Audit Log
CREATE TABLE IF NOT EXISTS security_audit_log (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    user_wallet TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    description TEXT NOT NULL,
    ip_address TEXT,
    user_agent TEXT,
    device_fingerprint TEXT,
    metadata TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_wallet ON security_audit_log(user_wallet, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_type ON security_audit_log(event_type, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_severity ON security_audit_log(severity, timestamp DESC);

CREATE TABLE IF NOT EXISTS suspicious_activity (
    id TEXT PRIMARY KEY,
    user_wallet TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    risk_level TEXT NOT NULL CHECK (risk_level IN ('low', 'medium', 'high')),
    details TEXT NOT NULL,
    auto_action_taken TEXT CHECK (auto_action_taken IN ('none', 'account_locked', 'mfa_required')),
    resolved BOOLEAN DEFAULT 0,
    detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_suspicious_wallet ON suspicious_activity(user_wallet, resolved);

-- ============================================================================
-- FEATURE FLAGS SYSTEM
-- ============================================================================

CREATE TABLE IF NOT EXISTS feature_flags (
    id TEXT PRIMARY KEY,
    feature_name TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT 0,
    rollout_percentage INTEGER DEFAULT 0 CHECK (rollout_percentage >= 0 AND rollout_percentage <= 100),
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert initial feature flags (all disabled by default)
INSERT OR IGNORE INTO feature_flags (id, feature_name, enabled, description) VALUES
('ai_sentiment', 'AI Sentiment Analysis', 0, 'Real-time sentiment aggregation from social media and on-chain data'),
('ai_predictions', 'AI Price Predictions', 0, 'ML-based price predictions with confidence intervals'),
('ai_backtesting', 'Strategy Backtesting', 0, 'Test trading strategies against historical data'),
('defi_jupiter', 'Jupiter Swaps', 0, 'Best-execution swaps via Jupiter aggregator'),
('defi_yield', 'Yield Farming Dashboard', 0, 'Track yield positions across multiple protocols'),
('defi_lp_analytics', 'LP Analytics', 0, 'Advanced liquidity pool analytics with IL tracking'),
('social_marketplace', 'Strategy Marketplace', 0, 'Publish and subscribe to trading strategies'),
('social_leaderboards', 'Trader Leaderboards', 0, 'Competitive rankings and achievements'),
('social_profiles', 'Trader Profiles', 0, 'Public trader profiles and social following'),
('social_copy_v2', 'Enhanced Copy Trading', 0, 'Copy trading with advanced risk controls'),
('security_ledger', 'Hardware Wallet Support', 0, 'Ledger hardware wallet integration'),
('security_simulation', 'Transaction Simulation', 0, 'Pre-execution transaction simulation and risk analysis'),
('security_audit_log', 'Security Audit Log', 0, 'Comprehensive security event logging');

-- Seed initial achievements
INSERT OR IGNORE INTO achievements (id, achievement_key, name, description, icon, rarity, criteria) VALUES
('ach_first_trade', 'first_trade', 'First Trade', 'Complete your first trade', '🎯', 'common', '{"type":"trade_count","value":1}'),
('ach_perfect_week', 'perfect_week', 'Perfect Week', '100% win rate for 7 days with at least 5 trades', '💎', 'epic', '{"type":"win_streak","days":7,"min_trades":5}'),
('ach_diamond_hands', 'diamond_hands', 'Diamond Hands', 'Hold through 50%+ drawdown and exit profitably', '💪', 'legendary', '{"type":"recovery","drawdown":50,"profitable":true}'),
('ach_volume_king', 'volume_king', 'Volume King', '$1M+ trading volume', '👑', 'epic', '{"type":"volume","value":1000000}'),
('ach_strategist', 'strategist', 'Strategist', 'Publish 5+ strategies with 4+ star rating', '🧠', 'rare', '{"type":"strategies","count":5,"min_rating":4}'),
('ach_social_butterfly', 'social_butterfly', 'Social Butterfly', 'Reach 100+ followers', '🦋', 'rare', '{"type":"followers","value":100}'),
('ach_early_adopter', 'early_adopter', 'Early Adopter', 'Join Eclipse Market Pro in the first 1000 users', '🚀', 'legendary', '{"type":"user_id","max":1000}');

-- Seed initial risk rules
INSERT OR IGNORE INTO risk_rules (id, rule_name, rule_type, severity, condition) VALUES
('rule_high_slippage', 'High Slippage Detection', 'threshold', 'medium', '{"type":"slippage","threshold":5.0}'),
('rule_unknown_program', 'Unknown Program Call', 'pattern', 'high', '{"type":"program_unknown"}'),
('rule_authority_transfer', 'Token Authority Transfer', 'pattern', 'critical', '{"type":"authority_change"}'),
('rule_excessive_fee', 'Excessive Transaction Fee', 'threshold', 'medium', '{"type":"fee","threshold":0.1}'),
('rule_balance_drain', 'Balance Drain Risk', 'threshold', 'critical', '{"type":"balance_change","threshold":80.0}');
