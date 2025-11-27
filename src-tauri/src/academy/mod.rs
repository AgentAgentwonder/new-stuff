pub mod commands;
pub mod content;
pub mod progress;
pub mod rewards;

pub use commands::*;
pub use content::*;
pub use progress::*;
pub use rewards::*;

use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedAcademyEngine = Arc<RwLock<AcademyEngine>>;

pub struct AcademyEngine {
    content_service: Arc<RwLock<content::ContentService>>,
    progress_tracker: Arc<RwLock<progress::ProgressTracker>>,
    reward_engine: Arc<RwLock<rewards::RewardEngine>>,
}

impl AcademyEngine {
    pub async fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let content_service = content::ContentService::new(app_handle).await?;
        let progress_tracker = progress::ProgressTracker::new(app_handle).await?;
        let reward_engine = rewards::RewardEngine::new(app_handle).await?;

        // Ensure default rewards/badges are seeded
        reward_engine.ensure_default_badges().await?;

        Ok(Self {
            content_service: Arc::new(RwLock::new(content_service)),
            progress_tracker: Arc::new(RwLock::new(progress_tracker)),
            reward_engine: Arc::new(RwLock::new(reward_engine)),
        })
    }

    pub fn content_service(&self) -> Arc<RwLock<content::ContentService>> {
        self.content_service.clone()
    }

    pub fn progress_tracker(&self) -> Arc<RwLock<progress::ProgressTracker>> {
        self.progress_tracker.clone()
    }

    pub fn reward_engine(&self) -> Arc<RwLock<rewards::RewardEngine>> {
        self.reward_engine.clone()
    }
}
