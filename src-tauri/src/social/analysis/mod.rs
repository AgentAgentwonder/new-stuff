pub mod gauges;
pub mod influencer;
pub mod sentiment_engine;
pub mod service;
pub mod trend_engine;

pub use gauges::{GaugeEngine, GaugeReading};
pub use influencer::{InfluencerEngine, InfluencerScore};
pub use sentiment_engine::{LexiconEntry, SentimentEngine, SentimentSnapshot};
pub use service::{
    AnalysisError, AnalysisSummary, SharedSocialAnalysisService, SocialAnalysisService,
};
pub use trend_engine::{TrendEngine, TrendRecord, DEFAULT_WINDOWS};
