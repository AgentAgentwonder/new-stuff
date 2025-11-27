// Trader Leaderboards
// Rankings and achievements

use super::types::*;
use sqlx::SqlitePool;

pub struct Leaderboard {
    db: SqlitePool,
}

impl Leaderboard {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Get leaderboard rankings
    pub async fn get_rankings(
        &self,
        ranking_type: String,
        limit: usize,
        offset: usize,
    ) -> SocialResult<Vec<TraderProfile>> {
        // TODO: Implement leaderboard rankings
        log::info!("Fetching {} leaderboard", ranking_type);
        Ok(vec![])
    }
}
