//! Position manager module.
//! Re-exports DeFi position management commands for convenient access.

pub use crate::defi::position_manager::{
    PositionManager, PositionSnapshot, get_auto_compound_recommendations,
    get_defi_portfolio_summary, get_defi_risk_metrics, get_defi_snapshot,
};
pub use crate::defi::types::{PortfolioSummary, RiskLevel, RiskMetrics};
