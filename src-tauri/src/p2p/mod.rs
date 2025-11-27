pub mod commands;
pub mod compliance;
pub mod database;
pub mod escrow;
pub mod matching;
pub mod types;

pub use commands::*;
pub use compliance::ComplianceChecker;
pub use database::P2PDatabase;
pub use escrow::{EscrowSmartContract, EscrowStateMachine};
pub use matching::LocalMatcher;
pub use types::*;

use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

pub type SharedP2PDatabase = Arc<RwLock<P2PDatabase>>;

pub async fn init_p2p_system(
    app_handle: &AppHandle,
) -> Result<SharedP2PDatabase, Box<dyn std::error::Error>> {
    let app_dir = app_handle
        .path()
        .app_data_dir()?;

    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("p2p.db");

    let db = P2PDatabase::new(db_path).await?;
    Ok(Arc::new(RwLock::new(db)))
}
