use super::analytics::JournalAnalytics;
use super::database::SharedJournalDatabase;
use super::types::*;
use chrono::Utc;

#[tauri::command]
pub async fn create_journal_entry(
    entry: JournalEntry,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<JournalEntry, String> {
    let mut entry = entry;
    let now = Utc::now().timestamp();

    if entry.id.is_empty() {
        entry.id = uuid::Uuid::new_v4().to_string();
    }

    if entry.created_at == 0 {
        entry.created_at = now;
    }

    entry.updated_at = now;

    let db_lock = db.write().await;
    db_lock
        .create_entry(&entry)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entry)
}

#[tauri::command]
pub async fn get_journal_entry(
    id: String,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<Option<JournalEntry>, String> {
    let db_lock = db.read().await;
    db_lock.get_entry(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_journal_entry(
    entry: JournalEntry,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<JournalEntry, String> {
    let mut entry = entry;
    entry.updated_at = Utc::now().timestamp();

    let db_lock = db.write().await;
    db_lock
        .update_entry(&entry)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entry)
}

#[tauri::command]
pub async fn delete_journal_entry(
    id: String,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<(), String> {
    let db_lock = db.write().await;
    db_lock.delete_entry(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_journal_entries(
    filters: JournalFilters,
    limit: i64,
    offset: i64,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<Vec<JournalEntry>, String> {
    let db_lock = db.read().await;
    db_lock
        .get_entries(&filters, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_journal_entries_count(
    filters: JournalFilters,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<i64, String> {
    let db_lock = db.read().await;
    db_lock
        .get_entries_count(&filters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_weekly_report(
    week_start: Option<i64>,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<WeeklyReport, String> {
    let now = Utc::now().timestamp();
    let week_start = week_start.unwrap_or(now - (7 * 24 * 60 * 60));
    let week_end = week_start + (7 * 24 * 60 * 60);

    let filters = JournalFilters {
        date_range: Some(DateRange {
            start: week_start,
            end: week_end,
        }),
        entry_types: None,
        strategy_tags: None,
        emotions: None,
        min_confidence: None,
        max_confidence: None,
        outcome_success: None,
        search_query: None,
    };

    let db_lock = db.read().await;
    let entries = db_lock
        .get_entries(&filters, 1000, 0)
        .await
        .map_err(|e| e.to_string())?;
    drop(db_lock);

    let report = JournalAnalytics::generate_weekly_report(&entries);

    let db_lock = db.write().await;
    db_lock
        .save_weekly_report(&report)
        .await
        .map_err(|e| e.to_string())?;

    Ok(report)
}

#[tauri::command]
pub async fn get_weekly_report(
    week_start: i64,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<Option<WeeklyReport>, String> {
    let db_lock = db.read().await;
    db_lock
        .get_weekly_report(week_start)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_weekly_reports(
    limit: i64,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<Vec<WeeklyReport>, String> {
    let db_lock = db.read().await;
    db_lock
        .get_weekly_reports(limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_behavioral_analytics(
    filters: Option<JournalFilters>,
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<BehavioralAnalytics, String> {
    let filters = filters.unwrap_or(JournalFilters {
        date_range: None,
        entry_types: None,
        strategy_tags: None,
        emotions: None,
        min_confidence: None,
        max_confidence: None,
        outcome_success: None,
        search_query: None,
    });

    let db_lock = db.read().await;
    let entries = db_lock
        .get_entries(&filters, 10000, 0)
        .await
        .map_err(|e| e.to_string())?;
    drop(db_lock);

    Ok(JournalAnalytics::calculate_behavioral_analytics(&entries))
}

#[tauri::command]
pub async fn get_journal_stats(
    db: tauri::State<'_, SharedJournalDatabase>,
) -> Result<JournalStats, String> {
    let now = Utc::now().timestamp();
    let week_ago = now - (7 * 24 * 60 * 60);
    let month_ago = now - (30 * 24 * 60 * 60);

    let all_filters = JournalFilters {
        date_range: None,
        entry_types: None,
        strategy_tags: None,
        emotions: None,
        min_confidence: None,
        max_confidence: None,
        outcome_success: None,
        search_query: None,
    };

    let week_filters = JournalFilters {
        date_range: Some(DateRange {
            start: week_ago,
            end: now,
        }),
        ..all_filters.clone()
    };

    let month_filters = JournalFilters {
        date_range: Some(DateRange {
            start: month_ago,
            end: now,
        }),
        ..all_filters.clone()
    };

    let db_lock = db.read().await;
    let total_entries = db_lock
        .get_entries_count(&all_filters)
        .await
        .map_err(|e| e.to_string())? as usize;
    let entries_this_week = db_lock
        .get_entries_count(&week_filters)
        .await
        .map_err(|e| e.to_string())? as usize;
    let entries_this_month = db_lock
        .get_entries_count(&month_filters)
        .await
        .map_err(|e| e.to_string())? as usize;

    let all_entries = db_lock
        .get_entries(&all_filters, 10000, 0)
        .await
        .map_err(|e| e.to_string())?;
    drop(db_lock);

    let total_trades_logged = all_entries.iter().filter(|e| e.outcome.is_some()).count();

    let average_entries_per_week = if total_entries > 0 {
        total_entries as f32 / 4.0
    } else {
        0.0
    };

    let mut strategy_counts: std::collections::HashMap<String, (usize, usize)> =
        std::collections::HashMap::new();
    let mut emotion_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut total_discipline = 0.0;

    for entry in &all_entries {
        for tag in &entry.strategy_tags {
            let (count, wins) = strategy_counts.entry(tag.clone()).or_insert((0, 0));
            *count += 1;
            if entry.outcome.as_ref().map(|o| o.success).unwrap_or(false) {
                *wins += 1;
            }
        }

        let emotion = format!("{:?}", entry.emotions.primary_emotion);
        *emotion_counts.entry(emotion).or_insert(0) += 1;

        total_discipline += entry.emotions.discipline_score;
    }

    let most_used_strategies: Vec<StrategyUsage> = {
        let mut strategies: Vec<_> = strategy_counts
            .into_iter()
            .map(|(tag, (count, wins))| {
                let win_rate = if count > 0 {
                    (wins as f32 / count as f32) * 100.0
                } else {
                    0.0
                };
                StrategyUsage {
                    tag,
                    count,
                    win_rate,
                }
            })
            .collect();
        strategies.sort_by(|a, b| b.count.cmp(&a.count));
        strategies.into_iter().take(5).collect()
    };

    let most_common_emotions: Vec<EmotionUsage> = {
        let total = all_entries.len() as f32;
        let mut emotions: Vec<_> = emotion_counts
            .into_iter()
            .map(|(emotion, count)| {
                let percentage = if total > 0.0 {
                    (count as f32 / total) * 100.0
                } else {
                    0.0
                };
                EmotionUsage {
                    emotion,
                    count,
                    percentage,
                }
            })
            .collect();
        emotions.sort_by(|a, b| b.count.cmp(&a.count));
        emotions.into_iter().take(5).collect()
    };

    let overall_discipline_score = if total_entries > 0 {
        total_discipline / total_entries as f32
    } else {
        0.0
    };

    Ok(JournalStats {
        total_entries,
        entries_this_week,
        entries_this_month,
        total_trades_logged,
        average_entries_per_week,
        most_used_strategies,
        most_common_emotions,
        overall_discipline_score,
    })
}
