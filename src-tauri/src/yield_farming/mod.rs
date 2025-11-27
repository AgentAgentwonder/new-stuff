//! Voice-friendly yield farming commands.
//! These re-export the DeFi yield farming adapters for convenient access.

pub use crate::defi::types::{DeFiPosition, Protocol, PositionType, Reward};
pub use crate::defi::yield_farming::{
    FarmingOpportunity, YieldFarm, YieldFarmingAdapter, get_farming_opportunities,
    get_farming_positions, get_yield_farms,
};
