use super::types::*;
use serde_json;
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct JournalDatabase {
    pool: Pool<Sqlite>,
}

impl JournalDatabase {
    pub async fn new(db_path: PathBuf) -> Result<Self, sqlx::Error> {
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        
        let pool = match SqlitePool::connect(&db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!("Warning: JournalDatabase failed to connect to {:?}: {}", db_path, e);
                eprintln!("Falling back to in-memory database for JournalDatabase");
                eprintln!("JournalDatabase using in-memory database for this session");
                SqlitePool::connect("sqlite::memory:").await?
            }
        };

        let db = Self { pool };
        db.initialize().await?;

        Ok(db)
    }

    async fn initialize(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS journal_entries (
                id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                trade_id TEXT,
                entry_type TEXT NOT NULL,
                strategy_tags TEXT NOT NULL,
                emotions TEXT NOT NULL,
                notes TEXT NOT NULL,
                market_conditions TEXT NOT NULL,
                confidence_level REAL NOT NULL,
                position_size REAL,
                entry_price REAL,
                exit_price REAL,
                outcome TEXT,
                lessons_learned TEXT,
                attachments TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_journal_timestamp ON journal_entries(timestamp);
            CREATE INDEX IF NOT EXISTS idx_journal_entry_type ON journal_entries(entry_type);
            CREATE INDEX IF NOT EXISTS idx_journal_trade_id ON journal_entries(trade_id);
            CREATE INDEX IF NOT EXISTS idx_journal_created_at ON journal_entries(created_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS weekly_reports (
                id TEXT PRIMARY KEY,
                week_start INTEGER NOT NULL,
                week_end INTEGER NOT NULL,
                total_entries INTEGER NOT NULL,
                trades_taken INTEGER NOT NULL,
                trades_won INTEGER NOT NULL,
                trades_lost INTEGER NOT NULL,
                win_rate REAL NOT NULL,
                total_pnl REAL NOT NULL,
                average_confidence REAL NOT NULL,
                emotion_breakdown TEXT NOT NULL,
                discipline_metrics TEXT NOT NULL,
                pattern_insights TEXT NOT NULL,
                strategy_performance TEXT NOT NULL,
                psychological_insights TEXT NOT NULL,
                recommendations TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_weekly_reports_week_start ON weekly_reports(week_start);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_entry(&self, entry: &JournalEntry) -> Result<(), sqlx::Error> {
        let strategy_tags_json = serde_json::to_string(&entry.strategy_tags).unwrap_or_default();
        let emotions_json = serde_json::to_string(&entry.emotions).unwrap_or_default();
        let market_conditions_json =
            serde_json::to_string(&entry.market_conditions).unwrap_or_default();
        let outcome_json = entry
            .outcome
            .as_ref()
            .map(|o| serde_json::to_string(o).unwrap_or_default());
        let attachments_json = serde_json::to_string(&entry.attachments).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO journal_entries (
                id, timestamp, trade_id, entry_type, strategy_tags,
                emotions, notes, market_conditions, confidence_level,
                position_size, entry_price, exit_price, outcome,
                lessons_learned, attachments, created_at, updated_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17
            )
            "#,
        )
        .bind(&entry.id)
        .bind(entry.timestamp)
        .bind(&entry.trade_id)
        .bind(serde_json::to_string(&entry.entry_type).unwrap_or_default())
        .bind(strategy_tags_json)
        .bind(emotions_json)
        .bind(&entry.notes)
        .bind(market_conditions_json)
        .bind(entry.confidence_level)
        .bind(entry.position_size)
        .bind(entry.entry_price)
        .bind(entry.exit_price)
        .bind(outcome_json)
        .bind(&entry.lessons_learned)
        .bind(attachments_json)
        .bind(entry.created_at)
        .bind(entry.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_entry(&self, id: &str) -> Result<Option<JournalEntry>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM journal_entries WHERE id = ?1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| self.row_to_entry(&r)))
    }

    pub async fn update_entry(&self, entry: &JournalEntry) -> Result<(), sqlx::Error> {
        let strategy_tags_json = serde_json::to_string(&entry.strategy_tags).unwrap_or_default();
        let emotions_json = serde_json::to_string(&entry.emotions).unwrap_or_default();
        let market_conditions_json =
            serde_json::to_string(&entry.market_conditions).unwrap_or_default();
        let outcome_json = entry
            .outcome
            .as_ref()
            .map(|o| serde_json::to_string(o).unwrap_or_default());
        let attachments_json = serde_json::to_string(&entry.attachments).unwrap_or_default();

        sqlx::query(
            r#"
            UPDATE journal_entries SET
                timestamp = ?2, trade_id = ?3, entry_type = ?4, strategy_tags = ?5,
                emotions = ?6, notes = ?7, market_conditions = ?8, confidence_level = ?9,
                position_size = ?10, entry_price = ?11, exit_price = ?12, outcome = ?13,
                lessons_learned = ?14, attachments = ?15, updated_at = ?16
            WHERE id = ?1
            "#,
        )
        .bind(&entry.id)
        .bind(entry.timestamp)
        .bind(&entry.trade_id)
        .bind(serde_json::to_string(&entry.entry_type).unwrap_or_default())
        .bind(strategy_tags_json)
        .bind(emotions_json)
        .bind(&entry.notes)
        .bind(market_conditions_json)
        .bind(entry.confidence_level)
        .bind(entry.position_size)
        .bind(entry.entry_price)
        .bind(entry.exit_price)
        .bind(outcome_json)
        .bind(&entry.lessons_learned)
        .bind(attachments_json)
        .bind(entry.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_entry(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM journal_entries WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_entries(
        &self,
        filters: &JournalFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<JournalEntry>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM journal_entries WHERE 1=1");
        let mut bindings: Vec<String> = Vec::new();

        if let Some(date_range) = &filters.date_range {
            query.push_str(" AND timestamp >= ? AND timestamp <= ?");
        }

        if let Some(entry_types) = &filters.entry_types {
            if !entry_types.is_empty() {
                let placeholders: Vec<String> =
                    entry_types.iter().map(|_| "?".to_string()).collect();
                query.push_str(&format!(" AND entry_type IN ({})", placeholders.join(",")));
            }
        }

        if let Some(min_confidence) = filters.min_confidence {
            query.push_str(" AND confidence_level >= ?");
        }

        if let Some(max_confidence) = filters.max_confidence {
            query.push_str(" AND confidence_level <= ?");
        }

        if let Some(search_query) = &filters.search_query {
            query.push_str(" AND (notes LIKE ? OR lessons_learned LIKE ?)");
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");

        let mut prepared_query = sqlx::query(&query);

        if let Some(date_range) = &filters.date_range {
            prepared_query = prepared_query.bind(date_range.start).bind(date_range.end);
        }

        if let Some(entry_types) = &filters.entry_types {
            for et in entry_types {
                prepared_query = prepared_query.bind(serde_json::to_string(et).unwrap_or_default());
            }
        }

        if let Some(min_confidence) = filters.min_confidence {
            prepared_query = prepared_query.bind(min_confidence);
        }

        if let Some(max_confidence) = filters.max_confidence {
            prepared_query = prepared_query.bind(max_confidence);
        }

        if let Some(search_query) = &filters.search_query {
            let search_pattern = format!("%{}%", search_query);
            prepared_query = prepared_query.bind(search_pattern.clone()).bind(search_pattern);
        }

        prepared_query = prepared_query.bind(limit).bind(offset);

        let rows = prepared_query.fetch_all(&self.pool).await?;

        let entries = rows.iter().map(|r| self.row_to_entry(r)).collect();

        Ok(entries)
    }

    pub async fn get_entries_count(&self, filters: &JournalFilters) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) as count FROM journal_entries WHERE 1=1");

        if let Some(date_range) = &filters.date_range {
            query.push_str(" AND timestamp >= ? AND timestamp <= ?");
        }

        if let Some(entry_types) = &filters.entry_types {
            if !entry_types.is_empty() {
                let placeholders: Vec<String> =
                    entry_types.iter().map(|_| "?".to_string()).collect();
                query.push_str(&format!(" AND entry_type IN ({})", placeholders.join(",")));
            }
        }

        if let Some(min_confidence) = filters.min_confidence {
            query.push_str(" AND confidence_level >= ?");
        }

        if let Some(max_confidence) = filters.max_confidence {
            query.push_str(" AND confidence_level <= ?");
        }

        if let Some(search_query) = &filters.search_query {
            query.push_str(" AND (notes LIKE ? OR lessons_learned LIKE ?)");
        }

        let mut prepared_query = sqlx::query(&query);

        if let Some(date_range) = &filters.date_range {
            prepared_query = prepared_query.bind(date_range.start).bind(date_range.end);
        }

        if let Some(entry_types) = &filters.entry_types {
            for et in entry_types {
                prepared_query = prepared_query.bind(serde_json::to_string(et).unwrap_or_default());
            }
        }

        if let Some(min_confidence) = filters.min_confidence {
            prepared_query = prepared_query.bind(min_confidence);
        }

        if let Some(max_confidence) = filters.max_confidence {
            prepared_query = prepared_query.bind(max_confidence);
        }

        if let Some(search_query) = &filters.search_query {
            let search_pattern = format!("%{}%", search_query);
            prepared_query = prepared_query.bind(search_pattern.clone()).bind(search_pattern);
        }

        let row = prepared_query.fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");

        Ok(count)
    }

    pub async fn save_weekly_report(&self, report: &WeeklyReport) -> Result<(), sqlx::Error> {
        let emotion_breakdown_json =
            serde_json::to_string(&report.emotion_breakdown).unwrap_or_default();
        let discipline_metrics_json =
            serde_json::to_string(&report.discipline_metrics).unwrap_or_default();
        let pattern_insights_json =
            serde_json::to_string(&report.pattern_insights).unwrap_or_default();
        let strategy_performance_json =
            serde_json::to_string(&report.strategy_performance).unwrap_or_default();
        let psychological_insights_json =
            serde_json::to_string(&report.psychological_insights).unwrap_or_default();
        let recommendations_json =
            serde_json::to_string(&report.recommendations).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO weekly_reports (
                id, week_start, week_end, total_entries, trades_taken,
                trades_won, trades_lost, win_rate, total_pnl,
                average_confidence, emotion_breakdown, discipline_metrics,
                pattern_insights, strategy_performance, psychological_insights,
                recommendations, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17
            )
            "#,
        )
        .bind(&report.id)
        .bind(report.week_start)
        .bind(report.week_end)
        .bind(report.total_entries as i64)
        .bind(report.trades_taken as i64)
        .bind(report.trades_won as i64)
        .bind(report.trades_lost as i64)
        .bind(report.win_rate)
        .bind(report.total_pnl)
        .bind(report.average_confidence)
        .bind(emotion_breakdown_json)
        .bind(discipline_metrics_json)
        .bind(pattern_insights_json)
        .bind(strategy_performance_json)
        .bind(psychological_insights_json)
        .bind(recommendations_json)
        .bind(report.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_weekly_report(
        &self,
        week_start: i64,
    ) -> Result<Option<WeeklyReport>, sqlx::Error> {
        let row = sqlx::query("SELECT * FROM weekly_reports WHERE week_start = ?1")
            .bind(week_start)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| self.row_to_weekly_report(&r)))
    }

    pub async fn get_weekly_reports(&self, limit: i64) -> Result<Vec<WeeklyReport>, sqlx::Error> {
        let rows = sqlx::query("SELECT * FROM weekly_reports ORDER BY week_start DESC LIMIT ?1")
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        let reports = rows.iter().map(|r| self.row_to_weekly_report(r)).collect();

        Ok(reports)
    }

    fn row_to_entry(&self, row: &sqlx::sqlite::SqliteRow) -> JournalEntry {
        JournalEntry {
            id: row.get("id"),
            timestamp: row.get("timestamp"),
            trade_id: row.get("trade_id"),
            entry_type: serde_json::from_str(row.get("entry_type"))
                .unwrap_or(EntryType::Reflection),
            strategy_tags: serde_json::from_str(row.get("strategy_tags")).unwrap_or_default(),
            emotions: serde_json::from_str(row.get("emotions")).unwrap_or_else(|_| {
                EmotionTracking {
                    primary_emotion: Emotion::Neutral,
                    intensity: 0.5,
                    secondary_emotions: vec![],
                    stress_level: 0.5,
                    clarity_level: 0.5,
                    fomo_level: 0.0,
                    revenge_trading: false,
                    discipline_score: 0.5,
                }
            }),
            notes: row.get("notes"),
            market_conditions: serde_json::from_str(row.get("market_conditions")).unwrap_or_else(
                |_| MarketConditions {
                    trend: MarketTrend::Neutral,
                    volatility: Volatility::Medium,
                    volume: VolumeLevel::Medium,
                    news_sentiment: 0.0,
                    notes: String::new(),
                },
            ),
            confidence_level: row.get("confidence_level"),
            position_size: row.get("position_size"),
            entry_price: row.get("entry_price"),
            exit_price: row.get("exit_price"),
            outcome: row
                .get::<Option<String>, _>("outcome")
                .and_then(|s| serde_json::from_str(&s).ok()),
            lessons_learned: row.get("lessons_learned"),
            attachments: serde_json::from_str(row.get("attachments")).unwrap_or_default(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_weekly_report(&self, row: &sqlx::sqlite::SqliteRow) -> WeeklyReport {
        WeeklyReport {
            id: row.get("id"),
            week_start: row.get("week_start"),
            week_end: row.get("week_end"),
            total_entries: row.get::<i64, _>("total_entries") as usize,
            trades_taken: row.get::<i64, _>("trades_taken") as usize,
            trades_won: row.get::<i64, _>("trades_won") as usize,
            trades_lost: row.get::<i64, _>("trades_lost") as usize,
            win_rate: row.get("win_rate"),
            total_pnl: row.get("total_pnl"),
            average_confidence: row.get("average_confidence"),
            emotion_breakdown: serde_json::from_str(row.get("emotion_breakdown")).unwrap_or_else(
                |_| EmotionBreakdown {
                    emotion_counts: std::collections::HashMap::new(),
                    average_stress: 0.0,
                    average_clarity: 0.0,
                    average_fomo: 0.0,
                    revenge_trading_instances: 0,
                },
            ),
            discipline_metrics: serde_json::from_str(row.get("discipline_metrics")).unwrap_or_else(
                |_| DisciplineMetrics {
                    average_discipline_score: 0.0,
                    plan_adherence_rate: 0.0,
                    impulsive_trades: 0,
                    patient_trades: 0,
                    stop_loss_adherence: 0.0,
                },
            ),
            pattern_insights: serde_json::from_str(row.get("pattern_insights")).unwrap_or_default(),
            strategy_performance: serde_json::from_str(row.get("strategy_performance"))
                .unwrap_or_default(),
            psychological_insights: serde_json::from_str(row.get("psychological_insights"))
                .unwrap_or_else(|_| PsychologicalInsights {
                    dominant_emotions: vec![],
                    stress_correlation_with_loss: 0.0,
                    confidence_correlation_with_win: 0.0,
                    fomo_impact: 0.0,
                    best_mental_state: String::new(),
                    worst_mental_state: String::new(),
                    cognitive_biases_detected: vec![],
                }),
            recommendations: serde_json::from_str(row.get("recommendations")).unwrap_or_default(),
            created_at: row.get("created_at"),
        }
    }
}

pub type SharedJournalDatabase = Arc<RwLock<JournalDatabase>>;
