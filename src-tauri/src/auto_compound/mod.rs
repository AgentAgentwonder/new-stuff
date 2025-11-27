//! Auto-compound module.
//! Provides convenient re-exports for auto-compound related commands.

pub use crate::defi::auto_compound::{
    AutoCompoundEngine, CompoundTransaction, configure_auto_compound,
    estimate_compound_apy_boost, get_auto_compound_config, get_compound_history,
};
pub use crate::defi::types::AutoCompoundSettings;
