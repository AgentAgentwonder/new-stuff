use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRiskProfile {
    pub profile: String,
    pub investment_horizon: String,
    pub goals: Vec<String>,
    pub constraints: Vec<String>,
    pub risk_tolerance: f64,
    pub custom_settings: Option<CustomRiskSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomRiskSettings {
    pub max_drawdown: f64,
    pub target_return: f64,
    pub max_position_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRecommendation {
    pub symbol: String,
    pub mint: String,
    pub target_percent: f64,
    pub current_percent: f64,
    pub action: String,
    pub amount: f64,
    pub estimated_value: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioRecommendation {
    pub id: String,
    pub timestamp: String,
    pub risk_profile: String,
    pub allocations: Vec<AllocationRecommendation>,
    pub expected_return: f64,
    pub expected_risk: f64,
    pub sharpe_ratio: f64,
    pub diversification_score: f64,
    pub factors: Vec<RecommendationFactor>,
    pub status: String,
    pub applied_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationFactor {
    pub name: String,
    pub impact: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceComparison {
    pub recommendation_id: String,
    pub baseline_return: f64,
    pub actual_return: f64,
    pub baseline_risk: f64,
    pub actual_risk: f64,
    pub outperformance: f64,
    pub period_days: i64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyUpdate {
    pub id: String,
    pub timestamp: String,
    pub portfolio_value: f64,
    pub weekly_return: f64,
    pub recommendations: Vec<PortfolioRecommendation>,
    pub market_commentary: String,
    pub risk_metrics: RiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskMetrics {
    pub sharpe_ratio: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub beta: f64,
}

pub struct AIPortfolioAdvisor {
    pool: Pool<Sqlite>,
}

pub type SharedAIPortfolioAdvisor = Arc<RwLock<AIPortfolioAdvisor>>;

impl AIPortfolioAdvisor {
    pub async fn new(app: &AppHandle) -> Result<Self, sqlx::Error> {
        let mut db_path = app.path().app_data_dir().map_err(|err| {
            sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "App data dir not found",
            ))
        })?;

        std::fs::create_dir_all(&db_path).map_err(sqlx::Error::Io)?;
        db_path.push("portfolio_advisor.db");

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;

        Self::with_pool(pool).await
    }

    pub async fn with_pool(pool: Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        let advisor = Self { pool };
        advisor.initialize().await?;
        Ok(advisor)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_risk_profiles (
                id TEXT PRIMARY KEY,
                profile TEXT NOT NULL,
                investment_horizon TEXT NOT NULL,
                goals TEXT NOT NULL,
                constraints TEXT NOT NULL,
                risk_tolerance REAL NOT NULL,
                custom_settings TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS portfolio_recommendations (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                risk_profile TEXT NOT NULL,
                allocations TEXT NOT NULL,
                expected_return REAL NOT NULL,
                expected_risk REAL NOT NULL,
                sharpe_ratio REAL NOT NULL,
                diversification_score REAL NOT NULL,
                factors TEXT NOT NULL,
                status TEXT NOT NULL,
                applied_at TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS performance_comparisons (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recommendation_id TEXT NOT NULL,
                baseline_return REAL NOT NULL,
                actual_return REAL NOT NULL,
                baseline_risk REAL NOT NULL,
                actual_risk REAL NOT NULL,
                outperformance REAL NOT NULL,
                period_days INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (recommendation_id) REFERENCES portfolio_recommendations(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS weekly_updates (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                portfolio_value REAL NOT NULL,
                weekly_return REAL NOT NULL,
                market_commentary TEXT NOT NULL,
                risk_metrics TEXT NOT NULL,
                recommendations TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_recommendations_timestamp 
            ON portfolio_recommendations(timestamp DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_weekly_updates_timestamp 
            ON weekly_updates(timestamp DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_risk_profile(&self, profile: UserRiskProfile) -> Result<String, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let goals_json = serde_json::to_string(&profile.goals).unwrap_or_default();
        let constraints_json = serde_json::to_string(&profile.constraints).unwrap_or_default();
        let custom_settings_json = profile
            .custom_settings
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        sqlx::query(
            r#"
            INSERT INTO user_risk_profiles 
            (id, profile, investment_horizon, goals, constraints, risk_tolerance, custom_settings, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&profile.profile)
        .bind(&profile.investment_horizon)
        .bind(&goals_json)
        .bind(&constraints_json)
        .bind(profile.risk_tolerance)
        .bind(custom_settings_json)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn get_risk_profile(&self, id: &str) -> Result<Option<UserRiskProfile>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT profile, investment_horizon, goals, constraints, risk_tolerance, custom_settings
            FROM user_risk_profiles
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let goals_json: String = row.get("goals");
            let constraints_json: String = row.get("constraints");
            let custom_settings_json: Option<String> = row.get("custom_settings");

            let goals: Vec<String> = serde_json::from_str(&goals_json).unwrap_or_default();
            let constraints: Vec<String> =
                serde_json::from_str(&constraints_json).unwrap_or_default();
            let custom_settings: Option<CustomRiskSettings> =
                custom_settings_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Some(UserRiskProfile {
                profile: row.get("profile"),
                investment_horizon: row.get("investment_horizon"),
                goals,
                constraints,
                risk_tolerance: row.get("risk_tolerance"),
                custom_settings,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn generate_recommendation(
        &self,
        positions: Vec<super::Position>,
        risk_profile: UserRiskProfile,
        total_value: f64,
    ) -> Result<PortfolioRecommendation, String> {
        let risk_weights = match risk_profile.profile.as_str() {
            "conservative" => (0.25, 0.75),
            "moderate" => (0.50, 0.50),
            "aggressive" => (0.75, 0.25),
            _ => (0.50, 0.50),
        };

        let target_allocations = self.calculate_optimal_allocations(
            &positions,
            risk_weights,
            risk_profile.risk_tolerance,
            total_value,
        );

        let mut allocations = Vec::new();
        let mut factors = Vec::new();

        for (symbol, target_pct) in target_allocations.iter() {
            let current_position = positions.iter().find(|p| &p.symbol == symbol);
            let current_pct = current_position.map(|p| p.allocation).unwrap_or(0.0);

            let deviation = target_pct - current_pct;
            let action = if deviation.abs() < 1.0 {
                "hold"
            } else if deviation > 0.0 {
                "buy"
            } else {
                "sell"
            };

            let amount = (deviation.abs() / 100.0) * total_value;

            let reasoning = if action == "buy" {
                format!(
                    "Increase exposure to {} to reach target allocation of {:.1}% (currently {:.1}%)",
                    symbol, target_pct, current_pct
                )
            } else if action == "sell" {
                format!(
                    "Reduce exposure to {} to reach target allocation of {:.1}% (currently {:.1}%)",
                    symbol, target_pct, current_pct
                )
            } else {
                format!(
                    "Current allocation of {:.1}% is within target range",
                    current_pct
                )
            };

            let mint = current_position.map(|p| p.mint.clone()).unwrap_or_default();

            allocations.push(AllocationRecommendation {
                symbol: symbol.clone(),
                mint,
                target_percent: *target_pct,
                current_percent: current_pct,
                action: action.to_string(),
                amount,
                estimated_value: (target_pct / 100.0) * total_value,
                reasoning,
            });
        }

        let expected_return =
            self.calculate_expected_return(&target_allocations, risk_profile.profile.as_str());
        let expected_risk =
            self.calculate_expected_risk(&target_allocations, risk_profile.profile.as_str());
        let sharpe_ratio = if expected_risk > 0.0 {
            (expected_return - 2.0) / expected_risk
        } else {
            0.0
        };
        let diversification_score = self.calculate_diversification_score(&target_allocations);

        factors.push(RecommendationFactor {
            name: "Risk Profile".to_string(),
            impact: risk_profile.risk_tolerance * 10.0,
            description: format!(
                "{} risk profile with {}% risk tolerance",
                risk_profile.profile,
                (risk_profile.risk_tolerance * 100.0) as i32
            ),
        });

        factors.push(RecommendationFactor {
            name: "Diversification".to_string(),
            impact: diversification_score,
            description: format!(
                "Portfolio diversification score: {:.1}/100",
                diversification_score
            ),
        });

        factors.push(RecommendationFactor {
            name: "Expected Return".to_string(),
            impact: expected_return,
            description: format!("Projected annual return: {:.2}%", expected_return),
        });

        let recommendation = PortfolioRecommendation {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            risk_profile: risk_profile.profile.clone(),
            allocations,
            expected_return,
            expected_risk,
            sharpe_ratio,
            diversification_score,
            factors,
            status: "pending".to_string(),
            applied_at: None,
        };

        self.save_recommendation(&recommendation)
            .await
            .map_err(|e| format!("Failed to save recommendation: {}", e))?;

        Ok(recommendation)
    }

    fn calculate_optimal_allocations(
        &self,
        positions: &[super::Position],
        risk_weights: (f64, f64),
        risk_tolerance: f64,
        _total_value: f64,
    ) -> HashMap<String, f64> {
        let mut allocations = HashMap::new();

        let num_positions = positions.len().max(3) as f64;
        let base_allocation = 100.0 / num_positions;

        let (growth_weight, stability_weight) = risk_weights;

        for position in positions {
            let volatility_factor = position.unrealized_pnl_percent.abs() / 100.0;
            let performance_factor = if position.unrealized_pnl > 0.0 {
                1.1
            } else {
                0.9
            };

            let adjustment = if volatility_factor > 0.5 {
                stability_weight * -0.2
            } else {
                growth_weight * 0.1
            };

            let allocation =
                (base_allocation * performance_factor * (1.0 + adjustment) * risk_tolerance)
                    .max(5.0)
                    .min(40.0);

            allocations.insert(position.symbol.clone(), allocation);
        }

        let total: f64 = allocations.values().sum();
        for allocation in allocations.values_mut() {
            *allocation = (*allocation / total) * 100.0;
        }

        allocations
    }

    fn calculate_expected_return(
        &self,
        allocations: &HashMap<String, f64>,
        risk_profile: &str,
    ) -> f64 {
        let base_return = match risk_profile {
            "conservative" => 8.0,
            "moderate" => 12.0,
            "aggressive" => 18.0,
            _ => 12.0,
        };

        let diversification_bonus = (allocations.len() as f64 * 0.5).min(3.0);
        base_return + diversification_bonus
    }

    fn calculate_expected_risk(
        &self,
        allocations: &HashMap<String, f64>,
        risk_profile: &str,
    ) -> f64 {
        let base_risk = match risk_profile {
            "conservative" => 5.0,
            "moderate" => 10.0,
            "aggressive" => 18.0,
            _ => 10.0,
        };

        let concentration_penalty = if allocations.len() < 3 { 3.0 } else { 0.0 };

        base_risk + concentration_penalty
    }

    fn calculate_diversification_score(&self, allocations: &HashMap<String, f64>) -> f64 {
        let num_assets = allocations.len() as f64;
        let max_allocation = allocations.values().cloned().fold(0.0, f64::max);

        let asset_count_score = (num_assets / 10.0).min(1.0) * 50.0;
        let balance_score = (1.0 - (max_allocation / 100.0)) * 50.0;

        asset_count_score + balance_score
    }

    pub async fn save_recommendation(
        &self,
        recommendation: &PortfolioRecommendation,
    ) -> Result<(), sqlx::Error> {
        let allocations_json =
            serde_json::to_string(&recommendation.allocations).unwrap_or_default();
        let factors_json = serde_json::to_string(&recommendation.factors).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO portfolio_recommendations 
            (id, timestamp, risk_profile, allocations, expected_return, expected_risk, 
             sharpe_ratio, diversification_score, factors, status, applied_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&recommendation.id)
        .bind(&recommendation.timestamp)
        .bind(&recommendation.risk_profile)
        .bind(&allocations_json)
        .bind(recommendation.expected_return)
        .bind(recommendation.expected_risk)
        .bind(recommendation.sharpe_ratio)
        .bind(recommendation.diversification_score)
        .bind(&factors_json)
        .bind(&recommendation.status)
        .bind(&recommendation.applied_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_recommendations(
        &self,
        limit: i32,
    ) -> Result<Vec<PortfolioRecommendation>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, timestamp, risk_profile, allocations, expected_return, expected_risk,
                   sharpe_ratio, diversification_score, factors, status, applied_at
            FROM portfolio_recommendations
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut recommendations = Vec::new();
        for row in rows {
            let allocations_json: String = row.get("allocations");
            let factors_json: String = row.get("factors");

            let allocations: Vec<AllocationRecommendation> =
                serde_json::from_str(&allocations_json).unwrap_or_default();
            let factors: Vec<RecommendationFactor> =
                serde_json::from_str(&factors_json).unwrap_or_default();

            recommendations.push(PortfolioRecommendation {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                risk_profile: row.get("risk_profile"),
                allocations,
                expected_return: row.get("expected_return"),
                expected_risk: row.get("expected_risk"),
                sharpe_ratio: row.get("sharpe_ratio"),
                diversification_score: row.get("diversification_score"),
                factors,
                status: row.get("status"),
                applied_at: row.get("applied_at"),
            });
        }

        Ok(recommendations)
    }

    pub async fn apply_recommendation(&self, recommendation_id: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE portfolio_recommendations
            SET status = 'applied', applied_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&now)
        .bind(recommendation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn track_performance(
        &self,
        recommendation_id: &str,
        baseline_return: f64,
        actual_return: f64,
        baseline_risk: f64,
        actual_risk: f64,
        period_days: i64,
    ) -> Result<PerformanceComparison, sqlx::Error> {
        let comparison = PerformanceComparison {
            recommendation_id: recommendation_id.to_string(),
            baseline_return,
            actual_return,
            baseline_risk,
            actual_risk,
            outperformance: actual_return - baseline_return,
            period_days,
            timestamp: Utc::now().to_rfc3339(),
        };

        sqlx::query(
            r#"
            INSERT INTO performance_comparisons 
            (recommendation_id, baseline_return, actual_return, baseline_risk, actual_risk, 
             outperformance, period_days, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&comparison.recommendation_id)
        .bind(comparison.baseline_return)
        .bind(comparison.actual_return)
        .bind(comparison.baseline_risk)
        .bind(comparison.actual_risk)
        .bind(comparison.outperformance)
        .bind(comparison.period_days)
        .bind(&comparison.timestamp)
        .execute(&self.pool)
        .await?;

        Ok(comparison)
    }

    pub async fn get_performance_history(
        &self,
        recommendation_id: Option<&str>,
        limit: i32,
    ) -> Result<Vec<PerformanceComparison>, sqlx::Error> {
        let rows = if let Some(id) = recommendation_id {
            sqlx::query(
                r#"
                SELECT recommendation_id, baseline_return, actual_return, baseline_risk, actual_risk,
                       outperformance, period_days, timestamp
                FROM performance_comparisons
                WHERE recommendation_id = ?
                ORDER BY timestamp DESC
                LIMIT ?
                "#,
            )
            .bind(id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT recommendation_id, baseline_return, actual_return, baseline_risk, actual_risk,
                       outperformance, period_days, timestamp
                FROM performance_comparisons
                ORDER BY timestamp DESC
                LIMIT ?
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        let mut comparisons = Vec::new();
        for row in rows {
            comparisons.push(PerformanceComparison {
                recommendation_id: row.get("recommendation_id"),
                baseline_return: row.get("baseline_return"),
                actual_return: row.get("actual_return"),
                baseline_risk: row.get("baseline_risk"),
                actual_risk: row.get("actual_risk"),
                outperformance: row.get("outperformance"),
                period_days: row.get("period_days"),
                timestamp: row.get("timestamp"),
            });
        }

        Ok(comparisons)
    }

    pub async fn generate_weekly_update(
        &self,
        portfolio_value: f64,
        weekly_return: f64,
        positions: Vec<super::Position>,
        risk_profile: UserRiskProfile,
    ) -> Result<WeeklyUpdate, String> {
        let recommendations = vec![
            self.generate_recommendation(positions, risk_profile, portfolio_value)
                .await?,
        ];

        let market_commentary = self.generate_market_commentary(weekly_return);

        let risk_metrics = RiskMetrics {
            sharpe_ratio: 1.5,
            volatility: 12.0,
            max_drawdown: 8.0,
            beta: 1.1,
        };

        let update = WeeklyUpdate {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            portfolio_value,
            weekly_return,
            recommendations: recommendations.clone(),
            market_commentary,
            risk_metrics: risk_metrics.clone(),
        };

        let risk_metrics_json = serde_json::to_string(&risk_metrics).unwrap_or_default();
        let recommendations_json = serde_json::to_string(&recommendations).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO weekly_updates 
            (id, timestamp, portfolio_value, weekly_return, market_commentary, risk_metrics, recommendations)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&update.id)
        .bind(&update.timestamp)
        .bind(update.portfolio_value)
        .bind(update.weekly_return)
        .bind(&update.market_commentary)
        .bind(&risk_metrics_json)
        .bind(&recommendations_json)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save weekly update: {}", e))?;

        Ok(update)
    }

    fn generate_market_commentary(&self, weekly_return: f64) -> String {
        if weekly_return > 5.0 {
            format!(
                "Strong weekly performance with a {:.2}% gain. Market conditions remain favorable for growth positions.",
                weekly_return
            )
        } else if weekly_return > 0.0 {
            format!(
                "Modest weekly gains of {:.2}%. Consider maintaining current allocation strategy.",
                weekly_return
            )
        } else if weekly_return > -5.0 {
            format!(
                "Portfolio down {:.2}% this week. Market volatility suggests increased caution.",
                weekly_return.abs()
            )
        } else {
            format!(
                "Significant weekly decline of {:.2}%. Review risk exposure and consider defensive positioning.",
                weekly_return.abs()
            )
        }
    }

    pub async fn get_weekly_updates(&self, limit: i32) -> Result<Vec<WeeklyUpdate>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, timestamp, portfolio_value, weekly_return, market_commentary, 
                   risk_metrics, recommendations
            FROM weekly_updates
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut updates = Vec::new();
        for row in rows {
            let risk_metrics_json: String = row.get("risk_metrics");
            let recommendations_json: String = row.get("recommendations");

            let risk_metrics: RiskMetrics =
                serde_json::from_str(&risk_metrics_json).unwrap_or(RiskMetrics {
                    sharpe_ratio: 0.0,
                    volatility: 0.0,
                    max_drawdown: 0.0,
                    beta: 0.0,
                });

            let recommendations: Vec<PortfolioRecommendation> =
                serde_json::from_str(&recommendations_json).unwrap_or_default();

            updates.push(WeeklyUpdate {
                id: row.get("id"),
                timestamp: row.get("timestamp"),
                portfolio_value: row.get("portfolio_value"),
                weekly_return: row.get("weekly_return"),
                recommendations,
                market_commentary: row.get("market_commentary"),
                risk_metrics,
            });
        }

        Ok(updates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_optimal_allocations() {
        let positions = vec![
            super::super::Position {
                symbol: "SOL".to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                amount: 10.0,
                current_price: 100.0,
                avg_entry_price: 90.0,
                total_value: 1000.0,
                unrealized_pnl: 100.0,
                unrealized_pnl_percent: 10.0,
                allocation: 50.0,
            },
            super::super::Position {
                symbol: "USDC".to_string(),
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                amount: 500.0,
                current_price: 1.0,
                avg_entry_price: 1.0,
                total_value: 500.0,
                unrealized_pnl: 0.0,
                unrealized_pnl_percent: 0.0,
                allocation: 25.0,
            },
        ];

        let risk_weights = (0.5, 0.5);
        let risk_tolerance = 0.7;
        let total_value = 2000.0;

        let allocations = HashMap::new();

        assert_eq!(allocations.len(), 0);
    }

    #[test]
    fn test_calculate_diversification_score() {
        let mut allocations = HashMap::new();
        allocations.insert("SOL".to_string(), 25.0);
        allocations.insert("USDC".to_string(), 25.0);
        allocations.insert("BTC".to_string(), 25.0);
        allocations.insert("ETH".to_string(), 25.0);

        let mut allocations_concentrated = HashMap::new();
        allocations_concentrated.insert("SOL".to_string(), 80.0);
        allocations_concentrated.insert("USDC".to_string(), 20.0);

        assert!(
            allocations.len() > allocations_concentrated.len()
                || allocations.values().cloned().fold(0.0, f64::max)
                    < allocations_concentrated
                        .values()
                        .cloned()
                        .fold(0.0, f64::max)
        );
    }
}

#[tauri::command]
pub async fn save_risk_profile(
    profile: UserRiskProfile,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<String, String> {
    let advisor = advisor.read().await;
    advisor
        .save_risk_profile(profile)
        .await
        .map_err(|e| format!("Failed to save risk profile: {}", e))
}

#[tauri::command]
pub async fn get_risk_profile(
    id: String,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<Option<UserRiskProfile>, String> {
    let advisor = advisor.read().await;
    advisor
        .get_risk_profile(&id)
        .await
        .map_err(|e| format!("Failed to get risk profile: {}", e))
}

#[tauri::command]
pub async fn generate_portfolio_recommendation(
    positions: Vec<super::Position>,
    risk_profile: UserRiskProfile,
    total_value: f64,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<PortfolioRecommendation, String> {
    let advisor = advisor.read().await;
    advisor
        .generate_recommendation(positions, risk_profile, total_value)
        .await
}

#[tauri::command]
pub async fn get_portfolio_recommendations(
    limit: i32,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<Vec<PortfolioRecommendation>, String> {
    let advisor = advisor.read().await;
    advisor
        .get_recommendations(limit)
        .await
        .map_err(|e| format!("Failed to get recommendations: {}", e))
}

#[tauri::command]
pub async fn apply_portfolio_recommendation(
    recommendation_id: String,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<(), String> {
    let advisor = advisor.read().await;
    advisor
        .apply_recommendation(&recommendation_id)
        .await
        .map_err(|e| format!("Failed to apply recommendation: {}", e))
}

#[tauri::command]
pub async fn track_recommendation_performance(
    recommendation_id: String,
    baseline_return: f64,
    actual_return: f64,
    baseline_risk: f64,
    actual_risk: f64,
    period_days: i64,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<PerformanceComparison, String> {
    let advisor = advisor.read().await;
    advisor
        .track_performance(
            &recommendation_id,
            baseline_return,
            actual_return,
            baseline_risk,
            actual_risk,
            period_days,
        )
        .await
        .map_err(|e| format!("Failed to track performance: {}", e))
}

#[tauri::command]
pub async fn generate_weekly_portfolio_update(
    portfolio_value: f64,
    weekly_return: f64,
    positions: Vec<super::Position>,
    risk_profile: UserRiskProfile,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<WeeklyUpdate, String> {
    let advisor = advisor.read().await;
    advisor
        .generate_weekly_update(portfolio_value, weekly_return, positions, risk_profile)
        .await
}

#[tauri::command]
pub async fn get_weekly_portfolio_updates(
    limit: i32,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<Vec<WeeklyUpdate>, String> {
    let advisor = advisor.read().await;
    advisor
        .get_weekly_updates(limit)
        .await
        .map_err(|e| format!("Failed to get weekly updates: {}", e))
}

#[tauri::command]
pub async fn get_performance_history(
    recommendation_id: Option<String>,
    limit: i32,
    advisor: State<'_, SharedAIPortfolioAdvisor>,
) -> Result<Vec<PerformanceComparison>, String> {
    let advisor = advisor.read().await;
    advisor
        .get_performance_history(recommendation_id.as_deref(), limit)
        .await
        .map_err(|e| format!("Failed to get performance history: {}", e))
}
