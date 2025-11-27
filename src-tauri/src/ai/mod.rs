// AI Trading Features Module
// Provides sentiment analysis, price predictions, and strategy backtesting

pub mod types;
pub mod sentiment_analyzer;
pub mod price_predictor;
pub mod backtest_engine;

pub use types::*;

// Re-export main functionality
pub use sentiment_analyzer::SentimentAnalyzer;
pub use price_predictor::PricePredictor;
pub use backtest_engine::BacktestEngine;
