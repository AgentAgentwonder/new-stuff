#[cfg(test)]
mod journal_tests {
    use app_lib::journal::analytics::JournalAnalytics;
    use app_lib::journal::database::JournalDatabase;
    use app_lib::journal::types::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_journal_database_create_entry() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_journal.db");
        let db = JournalDatabase::new(db_path).await.unwrap();

        let entry = create_test_entry();
        db.create_entry(&entry).await.unwrap();

        let retrieved = db.get_entry(&entry.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, entry.id);
    }

    #[tokio::test]
    async fn test_journal_database_update_entry() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_journal.db");
        let db = JournalDatabase::new(db_path).await.unwrap();

        let mut entry = create_test_entry();
        db.create_entry(&entry).await.unwrap();

        entry.notes = "Updated notes".to_string();
        db.update_entry(&entry).await.unwrap();

        let retrieved = db.get_entry(&entry.id).await.unwrap().unwrap();
        assert_eq!(retrieved.notes, "Updated notes");
    }

    #[tokio::test]
    async fn test_journal_database_delete_entry() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_journal.db");
        let db = JournalDatabase::new(db_path).await.unwrap();

        let entry = create_test_entry();
        db.create_entry(&entry).await.unwrap();
        db.delete_entry(&entry.id).await.unwrap();

        let retrieved = db.get_entry(&entry.id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_journal_database_filter_by_date_range() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_journal.db");
        let db = JournalDatabase::new(db_path).await.unwrap();

        let now = chrono::Utc::now().timestamp();
        let entry1 = create_test_entry_with_timestamp(now - 100);
        let entry2 = create_test_entry_with_timestamp(now);

        db.create_entry(&entry1).await.unwrap();
        db.create_entry(&entry2).await.unwrap();

        let filters = JournalFilters {
            date_range: Some(DateRange {
                start: now - 50,
                end: now + 50,
            }),
            ..Default::default()
        };

        let entries = db.get_entries(&filters, 10, 0).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, entry2.id);
    }

    #[tokio::test]
    async fn test_journal_database_filter_by_entry_type() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_journal.db");
        let db = JournalDatabase::new(db_path).await.unwrap();

        let mut entry1 = create_test_entry();
        entry1.entry_type = EntryType::PreTrade;
        let mut entry2 = create_test_entry_with_timestamp(chrono::Utc::now().timestamp());
        entry2.entry_type = EntryType::PostTrade;

        db.create_entry(&entry1).await.unwrap();
        db.create_entry(&entry2).await.unwrap();

        let filters = JournalFilters {
            entry_types: Some(vec![EntryType::PostTrade]),
            ..Default::default()
        };

        let entries = db.get_entries(&filters, 10, 0).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entry_type, EntryType::PostTrade);
    }

    #[tokio::test]
    async fn test_journal_database_pagination() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_journal.db");
        let db = JournalDatabase::new(db_path).await.unwrap();

        for i in 0..15 {
            let entry = create_test_entry_with_timestamp(chrono::Utc::now().timestamp() + i);
            db.create_entry(&entry).await.unwrap();
        }

        let filters = JournalFilters::default();

        let page1 = db.get_entries(&filters, 10, 0).await.unwrap();
        assert_eq!(page1.len(), 10);

        let page2 = db.get_entries(&filters, 10, 10).await.unwrap();
        assert_eq!(page2.len(), 5);
    }

    #[tokio::test]
    async fn test_journal_analytics_weekly_report() {
        let entries = create_test_entries_for_week();
        let report = JournalAnalytics::generate_weekly_report(&entries);

        assert_eq!(report.total_entries, entries.len());
        assert!(report.win_rate >= 0.0 && report.win_rate <= 100.0);
        assert!(report.average_confidence >= 0.0 && report.average_confidence <= 1.0);
    }

    #[tokio::test]
    async fn test_journal_analytics_behavioral_analytics() {
        let entries = create_test_entries_for_week();
        let analytics = JournalAnalytics::calculate_behavioral_analytics(&entries);

        assert_eq!(analytics.total_entries, entries.len());
        assert!(analytics.consistency_score >= 0.0 && analytics.consistency_score <= 1.0);
        assert!(analytics.emotional_volatility >= 0.0);
    }

    #[tokio::test]
    async fn test_journal_analytics_pattern_detection() {
        let mut entries = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for i in 0..5 {
            let mut entry = create_test_entry_with_timestamp(now + i * 100);
            entry.emotions.stress_level = 0.9;
            entry.outcome = Some(TradeOutcome {
                pnl: -50.0,
                pnl_percent: -5.0,
                success: false,
                followed_plan: false,
                risk_reward_ratio: 0.5,
            });
            entries.push(entry);
        }

        let report = JournalAnalytics::generate_weekly_report(&entries);

        let has_stress_pattern = report
            .pattern_insights
            .iter()
            .any(|p| p.pattern_type == "High Stress Trading");

        assert!(has_stress_pattern);
    }

    #[tokio::test]
    async fn test_journal_analytics_cognitive_biases() {
        let mut entries = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for i in 0..3 {
            let mut entry = create_test_entry_with_timestamp(now + i * 100);
            entry.emotions.revenge_trading = true;
            entry.outcome = Some(TradeOutcome {
                pnl: -30.0,
                pnl_percent: -3.0,
                success: false,
                followed_plan: false,
                risk_reward_ratio: 0.5,
            });
            entries.push(entry);
        }

        let analytics = JournalAnalytics::calculate_behavioral_analytics(&entries);

        let has_loss_aversion = analytics
            .cognitive_biases
            .iter()
            .any(|b| b.bias_type == BiasType::LossAversion);

        assert!(has_loss_aversion);
    }

    #[tokio::test]
    async fn test_weekly_report_persistence() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_journal.db");
        let db = JournalDatabase::new(db_path).await.unwrap();

        let entries = create_test_entries_for_week();
        let report = JournalAnalytics::generate_weekly_report(&entries);

        db.save_weekly_report(&report).await.unwrap();

        let retrieved = db.get_weekly_report(report.week_start).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, report.id);
    }

    fn create_test_entry() -> JournalEntry {
        let now = chrono::Utc::now().timestamp();
        create_test_entry_with_timestamp(now)
    }

    fn create_test_entry_with_timestamp(timestamp: i64) -> JournalEntry {
        JournalEntry {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp,
            trade_id: None,
            entry_type: EntryType::PreTrade,
            strategy_tags: vec!["momentum".to_string()],
            emotions: EmotionTracking {
                primary_emotion: Emotion::Confident,
                intensity: 0.7,
                secondary_emotions: vec![],
                stress_level: 0.3,
                clarity_level: 0.8,
                fomo_level: 0.2,
                revenge_trading: false,
                discipline_score: 0.9,
            },
            notes: "Test entry".to_string(),
            market_conditions: MarketConditions {
                trend: MarketTrend::Bullish,
                volatility: Volatility::Medium,
                volume: VolumeLevel::High,
                news_sentiment: 0.5,
                notes: "Good market conditions".to_string(),
            },
            confidence_level: 0.8,
            position_size: Some(1000.0),
            entry_price: Some(100.0),
            exit_price: None,
            outcome: None,
            lessons_learned: None,
            attachments: vec![],
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    fn create_test_entries_for_week() -> Vec<JournalEntry> {
        let now = chrono::Utc::now().timestamp();
        let mut entries = Vec::new();

        for i in 0..10 {
            let mut entry = create_test_entry_with_timestamp(now - (i * 86400));
            entry.outcome = Some(TradeOutcome {
                pnl: if i % 2 == 0 { 100.0 } else { -50.0 },
                pnl_percent: if i % 2 == 0 { 10.0 } else { -5.0 },
                success: i % 2 == 0,
                followed_plan: true,
                risk_reward_ratio: 2.0,
            });
            entries.push(entry);
        }

        entries
    }

    impl Default for JournalFilters {
        fn default() -> Self {
            Self {
                date_range: None,
                entry_types: None,
                strategy_tags: None,
                emotions: None,
                min_confidence: None,
                max_confidence: None,
                outcome_success: None,
                search_query: None,
            }
        }
    }
}
