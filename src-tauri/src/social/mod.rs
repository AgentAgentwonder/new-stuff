// Social Trading Module
// Strategy marketplace, profiles, and leaderboards

pub mod types;
pub mod strategy_marketplace;
pub mod trader_profiles;
pub mod leaderboard;

// Export existing social modules
pub mod models;
pub mod cache;
pub mod service;
pub mod commands;
pub mod reddit;
pub mod twitter;
pub mod analysis;

pub use types::*;
pub use strategy_marketplace::StrategyMarketplace;
pub use trader_profiles::TraderProfileManager;
pub use leaderboard::Leaderboard;

// Re-export commonly used types for convenience
pub use cache::SocialCache;
pub use service::{SocialDataService, SharedSocialDataService};
pub use analysis::{SocialAnalysisService, SharedSocialAnalysisService};
