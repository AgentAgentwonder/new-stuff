use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BadgeRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RewardType {
    Xp,
    Badge,
    Certificate,
    Token,
    Nft,
    Reputation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rarity: BadgeRarity,
    pub icon_url: Option<String>,
    pub xp_reward: i64,
    pub reputation_boost: f64,
    pub requirements: String, // JSON
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub id: String,
    pub wallet_address: String,
    pub course_id: Option<String>,
    pub challenge_id: Option<String>,
    pub title: String,
    pub description: String,
    pub issued_at: DateTime<Utc>,
    pub certificate_url: Option<String>,
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub id: String,
    pub wallet_address: String,
    pub reward_type: RewardType,
    pub source_id: String,        // course_id, challenge_id, etc.
    pub source_type: String,      // course, challenge, webinar, etc.
    pub amount: i64,              // XP amount or token amount
    pub metadata: Option<String>, // JSON for additional data
    pub earned_at: DateTime<Utc>,
    pub claimed: bool,
    pub claimed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarnedBadge {
    pub id: String,
    pub wallet_address: String,
    pub badge_id: String,
    pub earned_at: DateTime<Utc>,
    pub source: String, // What action earned this badge
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardStats {
    pub total_rewards_issued: i64,
    pub total_xp_distributed: i64,
    pub total_badges_earned: i64,
    pub total_certificates_issued: i64,
    pub unique_reward_earners: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum RewardError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("already claimed: {0}")]
    AlreadyClaimed(String),
}

pub struct RewardEngine {
    pool: Pool<Sqlite>,
}

impl RewardEngine {
    pub async fn new(app_handle: &AppHandle) -> Result<Self, RewardError> {
        let app_dir = app_handle.path().app_data_dir().map_err(|err| {
            RewardError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Unable to resolve app data directory: {err}"),
            ))
        })?;

        let _ = std::fs::create_dir_all(&app_dir);
        let db_path = app_dir.join("academy.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: RewardEngine failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for RewardEngine");
                eprintln!("RewardEngine using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        Self::init_schema(&pool).await?;

        Ok(Self { pool })
    }

    async fn init_schema(pool: &Pool<Sqlite>) -> Result<(), RewardError> {
        // Badges table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS badges (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                rarity TEXT NOT NULL,
                icon_url TEXT,
                xp_reward INTEGER NOT NULL DEFAULT 0,
                reputation_boost REAL NOT NULL DEFAULT 0.0,
                requirements TEXT NOT NULL, -- JSON
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Certificates table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS certificates (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                course_id TEXT,
                challenge_id TEXT,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                issued_at TEXT NOT NULL,
                certificate_url TEXT,
                verification_code TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Rewards table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rewards (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                reward_type TEXT NOT NULL,
                source_id TEXT NOT NULL,
                source_type TEXT NOT NULL,
                amount INTEGER NOT NULL,
                metadata TEXT, -- JSON
                earned_at TEXT NOT NULL,
                claimed INTEGER NOT NULL DEFAULT 0,
                claimed_at TEXT
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Earned badges table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS earned_badges (
                id TEXT PRIMARY KEY NOT NULL,
                wallet_address TEXT NOT NULL,
                badge_id TEXT NOT NULL,
                earned_at TEXT NOT NULL,
                source TEXT NOT NULL,
                FOREIGN KEY (badge_id) REFERENCES badges(id),
                UNIQUE(wallet_address, badge_id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_rewards_wallet ON rewards(wallet_address)")
            .execute(pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_earned_badges_wallet ON earned_badges(wallet_address)",
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_certificates_wallet ON certificates(wallet_address)",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    // Badge operations
    pub async fn create_badge(&self, badge: Badge) -> Result<Badge, RewardError> {
        let rarity_str = format!("{:?}", badge.rarity).to_lowercase();

        sqlx::query(
            r#"
            INSERT INTO badges (
                id, name, description, rarity, icon_url, xp_reward,
                reputation_boost, requirements, is_active, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&badge.id)
        .bind(&badge.name)
        .bind(&badge.description)
        .bind(rarity_str)
        .bind(&badge.icon_url)
        .bind(badge.xp_reward)
        .bind(badge.reputation_boost)
        .bind(&badge.requirements)
        .bind(badge.is_active)
        .bind(badge.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(badge)
    }

    pub async fn get_badge(&self, id: &str) -> Result<Badge, RewardError> {
        let row = sqlx::query("SELECT * FROM badges WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| RewardError::NotFound(format!("Badge not found: {}", id)))?;

        Self::badge_from_row(&row)
    }

    pub async fn list_badges(&self) -> Result<Vec<Badge>, RewardError> {
        let rows = sqlx::query(
            "SELECT * FROM badges WHERE is_active = 1 ORDER BY rarity DESC, created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut badges = Vec::new();
        for row in rows {
            badges.push(Self::badge_from_row(&row)?);
        }

        Ok(badges)
    }

    // Award badge to user
    pub async fn award_badge(
        &self,
        wallet_address: &str,
        badge_id: &str,
        source: &str,
    ) -> Result<EarnedBadge, RewardError> {
        let id = format!("{}_{}", wallet_address, badge_id);
        let now = Utc::now();

        // Check if already earned
        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM earned_badges WHERE wallet_address = ? AND badge_id = ?",
        )
        .bind(wallet_address)
        .bind(badge_id)
        .fetch_one(&self.pool)
        .await?;

        if existing.unwrap_or(0) > 0 {
            return Err(RewardError::AlreadyClaimed(
                "Badge already earned".to_string(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO earned_badges (
                id, wallet_address, badge_id, earned_at, source
            ) VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(wallet_address)
        .bind(badge_id)
        .bind(now.to_rfc3339())
        .bind(source)
        .execute(&self.pool)
        .await?;

        // Award XP and reputation boost
        let badge = self.get_badge(badge_id).await?;

        // Create XP reward
        if badge.xp_reward > 0 {
            self.issue_reward(Reward {
                id: format!("badge_xp_{}_{}", wallet_address, badge_id),
                wallet_address: wallet_address.to_string(),
                reward_type: RewardType::Xp,
                source_id: badge_id.to_string(),
                source_type: "badge".to_string(),
                amount: badge.xp_reward,
                metadata: None,
                earned_at: now,
                claimed: false,
                claimed_at: None,
            })
            .await?;
        }

        // Create reputation boost reward
        if badge.reputation_boost > 0.0 {
            let metadata = serde_json::json!({
                "reputation_boost": badge.reputation_boost
            })
            .to_string();

            self.issue_reward(Reward {
                id: format!("badge_rep_{}_{}", wallet_address, badge_id),
                wallet_address: wallet_address.to_string(),
                reward_type: RewardType::Reputation,
                source_id: badge_id.to_string(),
                source_type: "badge".to_string(),
                amount: (badge.reputation_boost * 100.0) as i64,
                metadata: Some(metadata),
                earned_at: now,
                claimed: false,
                claimed_at: None,
            })
            .await?;
        }

        Ok(EarnedBadge {
            id,
            wallet_address: wallet_address.to_string(),
            badge_id: badge_id.to_string(),
            earned_at: now,
            source: source.to_string(),
        })
    }

    pub async fn get_user_badges(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<EarnedBadge>, RewardError> {
        let rows = sqlx::query(
            "SELECT * FROM earned_badges WHERE wallet_address = ? ORDER BY earned_at DESC",
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        let mut badges = Vec::new();
        for row in rows {
            badges.push(Self::earned_badge_from_row(&row)?);
        }

        Ok(badges)
    }

    // Certificate operations
    pub async fn issue_certificate(
        &self,
        certificate: Certificate,
    ) -> Result<Certificate, RewardError> {
        sqlx::query(
            r#"
            INSERT INTO certificates (
                id, wallet_address, course_id, challenge_id, title,
                description, issued_at, certificate_url, verification_code
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&certificate.id)
        .bind(&certificate.wallet_address)
        .bind(&certificate.course_id)
        .bind(&certificate.challenge_id)
        .bind(&certificate.title)
        .bind(&certificate.description)
        .bind(certificate.issued_at.to_rfc3339())
        .bind(&certificate.certificate_url)
        .bind(&certificate.verification_code)
        .execute(&self.pool)
        .await?;

        Ok(certificate)
    }

    pub async fn get_user_certificates(
        &self,
        wallet_address: &str,
    ) -> Result<Vec<Certificate>, RewardError> {
        let rows = sqlx::query(
            "SELECT * FROM certificates WHERE wallet_address = ? ORDER BY issued_at DESC",
        )
        .bind(wallet_address)
        .fetch_all(&self.pool)
        .await?;

        let mut certificates = Vec::new();
        for row in rows {
            certificates.push(Self::certificate_from_row(&row)?);
        }

        Ok(certificates)
    }

    pub async fn verify_certificate(
        &self,
        verification_code: &str,
    ) -> Result<Certificate, RewardError> {
        let row = sqlx::query("SELECT * FROM certificates WHERE verification_code = ?")
            .bind(verification_code)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| RewardError::NotFound("Certificate not found".to_string()))?;

        Self::certificate_from_row(&row)
    }

    // Reward operations
    pub async fn issue_reward(&self, reward: Reward) -> Result<Reward, RewardError> {
        let reward_type_str = format!("{:?}", reward.reward_type).to_lowercase();

        sqlx::query(
            r#"
            INSERT INTO rewards (
                id, wallet_address, reward_type, source_id, source_type,
                amount, metadata, earned_at, claimed, claimed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&reward.id)
        .bind(&reward.wallet_address)
        .bind(reward_type_str)
        .bind(&reward.source_id)
        .bind(&reward.source_type)
        .bind(reward.amount)
        .bind(&reward.metadata)
        .bind(reward.earned_at.to_rfc3339())
        .bind(reward.claimed)
        .bind(reward.claimed_at.map(|d| d.to_rfc3339()))
        .execute(&self.pool)
        .await?;

        Ok(reward)
    }

    pub async fn get_user_rewards(
        &self,
        wallet_address: &str,
        unclaimed_only: bool,
    ) -> Result<Vec<Reward>, RewardError> {
        let query = if unclaimed_only {
            "SELECT * FROM rewards WHERE wallet_address = ? AND claimed = 0 ORDER BY earned_at DESC"
        } else {
            "SELECT * FROM rewards WHERE wallet_address = ? ORDER BY earned_at DESC"
        };

        let rows = sqlx::query(query)
            .bind(wallet_address)
            .fetch_all(&self.pool)
            .await?;

        let mut rewards = Vec::new();
        for row in rows {
            rewards.push(Self::reward_from_row(&row)?);
        }

        Ok(rewards)
    }

    pub async fn claim_reward(&self, reward_id: &str) -> Result<(), RewardError> {
        let result = sqlx::query(
            r#"
            UPDATE rewards 
            SET claimed = 1, claimed_at = ?
            WHERE id = ? AND claimed = 0
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(reward_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RewardError::AlreadyClaimed(
                "Reward already claimed or not found".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn claim_all_rewards(&self, wallet_address: &str) -> Result<i64, RewardError> {
        let result = sqlx::query(
            r#"
            UPDATE rewards 
            SET claimed = 1, claimed_at = ?
            WHERE wallet_address = ? AND claimed = 0
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(wallet_address)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    pub async fn calculate_course_reward(&self, course_duration: i64, difficulty: &str) -> i64 {
        let base_xp = course_duration * 10; // 10 XP per minute

        let multiplier = match difficulty {
            "beginner" => 1.0,
            "intermediate" => 1.5,
            "advanced" => 2.0,
            "expert" => 2.5,
            _ => 1.0,
        };

        (base_xp as f64 * multiplier) as i64
    }

    pub async fn calculate_quiz_reward(
        &self,
        score: i64,
        total_points: i64,
        time_bonus: bool,
    ) -> i64 {
        let percentage = (score as f64 / total_points as f64) * 100.0;
        let base_xp = score * 5; // 5 XP per point

        let bonus = if percentage >= 90.0 {
            (base_xp as f64 * 0.5) as i64 // 50% bonus for excellence
        } else if percentage >= 80.0 {
            (base_xp as f64 * 0.25) as i64 // 25% bonus for good performance
        } else {
            0
        };

        let time_bonus_xp = if time_bonus {
            (base_xp as f64 * 0.1) as i64 // 10% bonus for quick completion
        } else {
            0
        };

        base_xp + bonus + time_bonus_xp
    }

    pub async fn calculate_challenge_reward(
        &self,
        difficulty: &str,
        completion_quality: f64,
    ) -> i64 {
        let base_xp = match difficulty {
            "beginner" => 500,
            "intermediate" => 1000,
            "advanced" => 2000,
            "expert" => 5000,
            _ => 500,
        };

        (base_xp as f64 * completion_quality) as i64
    }

    pub async fn ensure_default_badges(&self) -> Result<(), RewardError> {
        let badges = vec![
            Badge {
                id: "first_lesson".to_string(),
                name: "First Lesson".to_string(),
                description: "Complete your first lesson".to_string(),
                rarity: BadgeRarity::Common,
                icon_url: None,
                xp_reward: 100,
                reputation_boost: 1.0,
                requirements: serde_json::json!({"lessons_completed": 1}).to_string(),
                is_active: true,
                created_at: chrono::Utc::now(),
            },
            Badge {
                id: "quiz_master".to_string(),
                name: "Quiz Master".to_string(),
                description: "Pass 10 quizzes with 90% or higher".to_string(),
                rarity: BadgeRarity::Rare,
                icon_url: None,
                xp_reward: 500,
                reputation_boost: 5.0,
                requirements: serde_json::json!({"quizzes_passed": 10, "min_score": 90})
                    .to_string(),
                is_active: true,
                created_at: chrono::Utc::now(),
            },
            Badge {
                id: "trading_expert".to_string(),
                name: "Trading Expert".to_string(),
                description: "Complete all trading courses".to_string(),
                rarity: BadgeRarity::Epic,
                icon_url: None,
                xp_reward: 1000,
                reputation_boost: 10.0,
                requirements: serde_json::json!({"category": "trading", "all_courses": true})
                    .to_string(),
                is_active: true,
                created_at: chrono::Utc::now(),
            },
        ];

        for badge in badges {
            // Ignore errors if badge already exists
            let _ = self.create_badge(badge).await;
        }

        Ok(())
    }

    pub async fn get_reward_stats(&self) -> Result<RewardStats, RewardError> {
        let total_rewards_issued: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM rewards")
            .fetch_one(&self.pool)
            .await?;

        let total_xp_distributed: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(amount), 0) FROM rewards WHERE reward_type = 'xp' AND claimed = 1",
        )
        .fetch_one(&self.pool)
        .await?;

        let total_badges_earned: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM earned_badges")
            .fetch_one(&self.pool)
            .await?;

        let total_certificates_issued: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM certificates")
                .fetch_one(&self.pool)
                .await?;

        let unique_reward_earners: i64 =
            sqlx::query_scalar("SELECT COUNT(DISTINCT wallet_address) FROM rewards")
                .fetch_one(&self.pool)
                .await?;

        Ok(RewardStats {
            total_rewards_issued,
            total_xp_distributed,
            total_badges_earned,
            total_certificates_issued,
            unique_reward_earners,
        })
    }

    // Helper methods
    fn badge_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Badge, RewardError> {
        let rarity_str: String = row.try_get("rarity")?;
        let rarity = match rarity_str.as_str() {
            "common" => BadgeRarity::Common,
            "uncommon" => BadgeRarity::Uncommon,
            "rare" => BadgeRarity::Rare,
            "epic" => BadgeRarity::Epic,
            "legendary" => BadgeRarity::Legendary,
            _ => BadgeRarity::Common,
        };

        let created_str: String = row.try_get("created_at")?;

        Ok(Badge {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            rarity,
            icon_url: row.try_get("icon_url")?,
            xp_reward: row.try_get("xp_reward")?,
            reputation_boost: row.try_get("reputation_boost")?,
            requirements: row.try_get("requirements")?,
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map_err(|e| RewardError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
        })
    }

    fn certificate_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Certificate, RewardError> {
        let issued_str: String = row.try_get("issued_at")?;

        Ok(Certificate {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            course_id: row.try_get("course_id")?,
            challenge_id: row.try_get("challenge_id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            issued_at: DateTime::parse_from_rfc3339(&issued_str)
                .map_err(|e| RewardError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            certificate_url: row.try_get("certificate_url")?,
            verification_code: row.try_get("verification_code")?,
        })
    }

    fn reward_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Reward, RewardError> {
        let reward_type_str: String = row.try_get("reward_type")?;
        let reward_type = match reward_type_str.as_str() {
            "xp" => RewardType::Xp,
            "badge" => RewardType::Badge,
            "certificate" => RewardType::Certificate,
            "token" => RewardType::Token,
            "nft" => RewardType::Nft,
            "reputation" => RewardType::Reputation,
            _ => RewardType::Xp,
        };

        let earned_str: String = row.try_get("earned_at")?;
        let claimed_str: Option<String> = row.try_get("claimed_at")?;

        Ok(Reward {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            reward_type,
            source_id: row.try_get("source_id")?,
            source_type: row.try_get("source_type")?,
            amount: row.try_get("amount")?,
            metadata: row.try_get("metadata")?,
            earned_at: DateTime::parse_from_rfc3339(&earned_str)
                .map_err(|e| RewardError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            claimed: row.try_get::<i64, _>("claimed")? != 0,
            claimed_at: claimed_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
        })
    }

    fn earned_badge_from_row(row: &sqlx::sqlite::SqliteRow) -> Result<EarnedBadge, RewardError> {
        let earned_str: String = row.try_get("earned_at")?;

        Ok(EarnedBadge {
            id: row.try_get("id")?,
            wallet_address: row.try_get("wallet_address")?,
            badge_id: row.try_get("badge_id")?,
            earned_at: DateTime::parse_from_rfc3339(&earned_str)
                .map_err(|e| RewardError::InvalidData(e.to_string()))?
                .with_timezone(&Utc),
            source: row.try_get("source")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_rarity_serialization() {
        let rarity = BadgeRarity::Legendary;
        let json = serde_json::to_string(&rarity).unwrap();
        assert_eq!(json, "\"legendary\"");
    }

    #[test]
    fn test_reward_type_serialization() {
        let reward_type = RewardType::Xp;
        let json = serde_json::to_string(&reward_type).unwrap();
        assert_eq!(json, "\"xp\"");
    }
}
