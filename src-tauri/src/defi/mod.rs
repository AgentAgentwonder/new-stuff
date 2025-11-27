// DeFi Integrations Module
// Jupiter swaps, yield farming, and LP analytics

pub mod types;
pub mod jupiter;
pub mod yield_tracker;
pub mod lp_analyzer;

// Export existing DeFi modules
pub mod solend;
pub mod marginfi;
pub mod kamino;
pub mod staking;
pub mod yield_farming;
pub mod position_manager;
pub mod governance;
pub mod auto_compound;

pub use types::*;
pub use yield_tracker::YieldTracker;
pub use lp_analyzer::LpAnalyzer;

// Tauri command exports - wildcards ensure new commands are automatically available
pub use yield_farming::*;
pub use position_manager::*;
pub use auto_compound::*;
// Explicit exports for governance to avoid naming conflict with standalone governance module
pub use governance::{get_governance_proposals, vote_on_proposal, get_governance_participation};
// Protocol-specific command exports
pub use solend::{get_solend_reserves, get_solend_pools, get_solend_positions};
pub use marginfi::{get_marginfi_banks, get_marginfi_positions};
pub use kamino::{get_kamino_vaults, get_kamino_positions, get_kamino_farms};
pub use staking::{get_staking_pools, get_staking_positions, get_staking_schedule};
