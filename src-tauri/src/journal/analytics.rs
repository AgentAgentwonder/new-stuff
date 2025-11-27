use super::types::*;
use chrono::Utc;
use std::collections::HashMap;

pub struct JournalAnalytics;

impl JournalAnalytics {
    pub fn generate_weekly_report(entries: &[JournalEntry]) -> WeeklyReport {
        let now = Utc::now().timestamp();
        let week_start = now - (7 * 24 * 60 * 60);

        let mut total_entries = entries.len();
        let mut trades_taken = 0;
        let mut trades_won = 0;
        let mut trades_lost = 0;
        let mut total_pnl = 0.0;
        let mut total_confidence = 0.0;
        let mut emotion_counts: HashMap<String, usize> = HashMap::new();
        let mut total_stress = 0.0;
        let mut total_clarity = 0.0;
        let mut total_fomo = 0.0;
        let mut revenge_trading_instances = 0;
        let mut total_discipline = 0.0;
        let mut plan_adherence_count = 0;
        let mut plan_adherence_total = 0;
        let mut impulsive_trades = 0;
        let mut patient_trades = 0;
        let mut strategy_stats: HashMap<String, StrategyStats> = HashMap::new();

        for entry in entries {
            total_confidence += entry.confidence_level;

            let emotion_name = format!("{:?}", entry.emotions.primary_emotion);
            *emotion_counts.entry(emotion_name).or_insert(0) += 1;

            total_stress += entry.emotions.stress_level;
            total_clarity += entry.emotions.clarity_level;
            total_fomo += entry.emotions.fomo_level;
            total_discipline += entry.emotions.discipline_score;

            if entry.emotions.revenge_trading {
                revenge_trading_instances += 1;
            }

            if entry.emotions.primary_emotion == Emotion::Impatient {
                impulsive_trades += 1;
            } else if entry.emotions.primary_emotion == Emotion::Patient {
                patient_trades += 1;
            }

            for tag in &entry.strategy_tags {
                let stats = strategy_stats.entry(tag.clone()).or_insert(StrategyStats {
                    count: 0,
                    wins: 0,
                    total_pnl: 0.0,
                    total_confidence: 0.0,
                    emotions: HashMap::new(),
                });
                stats.count += 1;
                stats.total_confidence += entry.confidence_level;
                let emotion_name = format!("{:?}", entry.emotions.primary_emotion);
                *stats.emotions.entry(emotion_name).or_insert(0) += 1;
            }

            if let Some(outcome) = &entry.outcome {
                trades_taken += 1;
                total_pnl += outcome.pnl;

                if outcome.success {
                    trades_won += 1;
                } else {
                    trades_lost += 1;
                }

                if outcome.followed_plan {
                    plan_adherence_count += 1;
                }
                plan_adherence_total += 1;

                for tag in &entry.strategy_tags {
                    if let Some(stats) = strategy_stats.get_mut(tag) {
                        if outcome.success {
                            stats.wins += 1;
                        }
                        stats.total_pnl += outcome.pnl;
                    }
                }
            }
        }

        let count = entries.len() as f32;
        let win_rate = if trades_taken > 0 {
            (trades_won as f32 / trades_taken as f32) * 100.0
        } else {
            0.0
        };

        let average_confidence = if count > 0.0 {
            total_confidence / count
        } else {
            0.0
        };

        let emotion_breakdown = EmotionBreakdown {
            emotion_counts,
            average_stress: if count > 0.0 {
                total_stress / count
            } else {
                0.0
            },
            average_clarity: if count > 0.0 {
                total_clarity / count
            } else {
                0.0
            },
            average_fomo: if count > 0.0 { total_fomo / count } else { 0.0 },
            revenge_trading_instances,
        };

        let discipline_metrics = DisciplineMetrics {
            average_discipline_score: if count > 0.0 {
                total_discipline / count
            } else {
                0.0
            },
            plan_adherence_rate: if plan_adherence_total > 0 {
                (plan_adherence_count as f32 / plan_adherence_total as f32) * 100.0
            } else {
                0.0
            },
            impulsive_trades,
            patient_trades,
            stop_loss_adherence: if plan_adherence_total > 0 {
                (plan_adherence_count as f32 / plan_adherence_total as f32) * 100.0
            } else {
                0.0
            },
        };

        let pattern_insights = Self::generate_pattern_insights(entries);
        let strategy_performance = Self::generate_strategy_performance(strategy_stats);
        let psychological_insights = Self::generate_psychological_insights(entries);
        let recommendations = Self::generate_recommendations(
            &discipline_metrics,
            &psychological_insights,
            &pattern_insights,
        );

        WeeklyReport {
            id: uuid::Uuid::new_v4().to_string(),
            week_start,
            week_end: now,
            total_entries,
            trades_taken,
            trades_won,
            trades_lost,
            win_rate,
            total_pnl,
            average_confidence,
            emotion_breakdown,
            discipline_metrics,
            pattern_insights,
            strategy_performance,
            psychological_insights,
            recommendations,
            created_at: now,
        }
    }

    fn generate_pattern_insights(entries: &[JournalEntry]) -> Vec<PatternInsight> {
        let mut insights = Vec::new();

        let high_stress_losses = entries
            .iter()
            .filter(|e| {
                e.emotions.stress_level > 0.7
                    && e.outcome.as_ref().map(|o| !o.success).unwrap_or(false)
            })
            .count();

        if high_stress_losses > 2 {
            insights.push(PatternInsight {
                pattern_type: "High Stress Trading".to_string(),
                description: format!("Detected {} trades with high stress that resulted in losses", high_stress_losses),
                frequency: high_stress_losses,
                impact_on_performance: -0.7,
                recommendation: "Consider taking breaks or avoiding trades when stress levels are high. Practice mindfulness or breathing exercises before trading.".to_string(),
            });
        }

        let fomo_trades = entries
            .iter()
            .filter(|e| e.emotions.fomo_level > 0.7)
            .count();

        if fomo_trades > 1 {
            insights.push(PatternInsight {
                pattern_type: "FOMO Trading".to_string(),
                description: format!("Detected {} trades driven by FOMO", fomo_trades),
                frequency: fomo_trades,
                impact_on_performance: -0.5,
                recommendation: "Wait for your setups. Set alerts for your entry criteria and only trade when they're met.".to_string(),
            });
        }

        let revenge_trades = entries
            .iter()
            .filter(|e| e.emotions.revenge_trading)
            .count();

        if revenge_trades > 0 {
            insights.push(PatternInsight {
                pattern_type: "Revenge Trading".to_string(),
                description: format!("Detected {} revenge trading instances", revenge_trades),
                frequency: revenge_trades,
                impact_on_performance: -0.9,
                recommendation: "After a loss, take a mandatory break. Review your journal and calm down before the next trade.".to_string(),
            });
        }

        insights
    }

    fn generate_strategy_performance(
        strategy_stats: HashMap<String, StrategyStats>,
    ) -> Vec<StrategyPerformance> {
        strategy_stats
            .into_iter()
            .map(|(tag, stats)| {
                let win_rate = if stats.count > 0 {
                    (stats.wins as f32 / stats.count as f32) * 100.0
                } else {
                    0.0
                };
                let average_pnl = if stats.count > 0 {
                    stats.total_pnl / stats.count as f32
                } else {
                    0.0
                };
                let average_confidence = if stats.count > 0 {
                    stats.total_confidence / stats.count as f32
                } else {
                    0.0
                };
                let common_emotions: Vec<String> = stats
                    .emotions
                    .iter()
                    .take(3)
                    .map(|(e, _)| e.clone())
                    .collect();

                StrategyPerformance {
                    strategy_tag: tag,
                    trades_count: stats.count,
                    win_rate,
                    average_pnl,
                    average_confidence,
                    common_emotions,
                }
            })
            .collect()
    }

    fn generate_psychological_insights(entries: &[JournalEntry]) -> PsychologicalInsights {
        let mut emotion_counts: HashMap<String, usize> = HashMap::new();
        let mut stress_losses = 0;
        let mut stress_total_trades = 0;
        let mut confidence_wins = 0;
        let mut confidence_total_trades = 0;
        let mut fomo_impact_sum = 0.0;
        let mut fomo_count = 0;

        let mut best_emotion_wins: HashMap<String, usize> = HashMap::new();
        let mut best_emotion_total: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            let emotion_name = format!("{:?}", entry.emotions.primary_emotion);
            *emotion_counts.entry(emotion_name.clone()).or_insert(0) += 1;

            if let Some(outcome) = &entry.outcome {
                if entry.emotions.stress_level > 0.6 {
                    stress_total_trades += 1;
                    if !outcome.success {
                        stress_losses += 1;
                    }
                }

                if entry.confidence_level > 0.7 {
                    confidence_total_trades += 1;
                    if outcome.success {
                        confidence_wins += 1;
                    }
                }

                if entry.emotions.fomo_level > 0.5 {
                    fomo_count += 1;
                    fomo_impact_sum += if outcome.success { 0.0 } else { -outcome.pnl };
                }

                *best_emotion_total.entry(emotion_name.clone()).or_insert(0) += 1;
                if outcome.success {
                    *best_emotion_wins.entry(emotion_name).or_insert(0) += 1;
                }
            }
        }

        let dominant_emotions: Vec<String> = {
            let mut emotions: Vec<_> = emotion_counts.iter().collect();
            emotions.sort_by(|a, b| b.1.cmp(a.1));
            emotions
                .into_iter()
                .take(3)
                .map(|(e, _)| e.clone())
                .collect()
        };

        let stress_correlation_with_loss = if stress_total_trades > 0 {
            stress_losses as f32 / stress_total_trades as f32
        } else {
            0.0
        };

        let confidence_correlation_with_win = if confidence_total_trades > 0 {
            confidence_wins as f32 / confidence_total_trades as f32
        } else {
            0.0
        };

        let fomo_impact = if fomo_count > 0 {
            fomo_impact_sum / fomo_count as f32
        } else {
            0.0
        };

        let (best_mental_state, worst_mental_state) =
            Self::find_best_worst_mental_states(&best_emotion_wins, &best_emotion_total);

        let cognitive_biases_detected = Self::detect_cognitive_biases(entries);

        PsychologicalInsights {
            dominant_emotions,
            stress_correlation_with_loss,
            confidence_correlation_with_win,
            fomo_impact,
            best_mental_state,
            worst_mental_state,
            cognitive_biases_detected,
        }
    }

    fn find_best_worst_mental_states(
        wins: &HashMap<String, usize>,
        totals: &HashMap<String, usize>,
    ) -> (String, String) {
        let mut rates: Vec<_> = totals
            .iter()
            .filter(|(_, &total)| total >= 3)
            .map(|(emotion, &total)| {
                let win_count = wins.get(emotion).copied().unwrap_or(0);
                let win_rate = win_count as f32 / total as f32;
                (emotion.clone(), win_rate)
            })
            .collect();

        rates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best = rates
            .first()
            .map(|(e, _)| e.clone())
            .unwrap_or_else(|| "Calm".to_string());
        let worst = rates
            .last()
            .map(|(e, _)| e.clone())
            .unwrap_or_else(|| "Stressed".to_string());

        (best, worst)
    }

    fn detect_cognitive_biases(entries: &[JournalEntry]) -> Vec<String> {
        let mut biases = Vec::new();

        let recent_losses: Vec<_> = entries
            .iter()
            .filter(|e| e.outcome.as_ref().map(|o| !o.success).unwrap_or(false))
            .take(3)
            .collect();

        if recent_losses.len() >= 2 {
            let revenge_count = recent_losses
                .iter()
                .filter(|e| e.emotions.revenge_trading)
                .count();

            if revenge_count > 0 {
                biases.push(
                    "Loss Aversion: Attempting to recover losses quickly through revenge trading"
                        .to_string(),
                );
            }
        }

        let overconfident_losses = entries
            .iter()
            .filter(|e| {
                e.confidence_level > 0.8 && e.outcome.as_ref().map(|o| !o.success).unwrap_or(false)
            })
            .count();

        if overconfident_losses > 2 {
            biases.push(
                "Overconfidence Bias: High confidence trades resulting in losses".to_string(),
            );
        }

        let fomo_trades = entries
            .iter()
            .filter(|e| e.emotions.fomo_level > 0.7)
            .count();

        if fomo_trades > 1 {
            biases.push("Herd Mentality: Trading based on FOMO rather than analysis".to_string());
        }

        biases
    }

    fn generate_recommendations(
        discipline: &DisciplineMetrics,
        psychology: &PsychologicalInsights,
        patterns: &[PatternInsight],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if discipline.average_discipline_score < 0.6 {
            recommendations.push("Focus on improving trading discipline. Create a detailed trading plan and stick to it.".to_string());
        }

        if discipline.plan_adherence_rate < 70.0 {
            recommendations.push("Your plan adherence is low. Consider using a pre-trade checklist to ensure you follow your rules.".to_string());
        }

        if psychology.stress_correlation_with_loss > 0.6 {
            recommendations.push("High stress is strongly correlated with losses. Implement stress management techniques before trading.".to_string());
        }

        if psychology.fomo_impact > 100.0 {
            recommendations.push("FOMO is significantly impacting your performance. Wait for your setups and avoid chasing the market.".to_string());
        }

        if !psychology.cognitive_biases_detected.is_empty() {
            recommendations.push(format!(
                "Cognitive biases detected: {}. Work on awareness and mitigation strategies.",
                psychology.cognitive_biases_detected.join(", ")
            ));
        }

        if discipline.impulsive_trades > discipline.patient_trades {
            recommendations.push("You're taking more impulsive trades than patient ones. Slow down and wait for quality setups.".to_string());
        }

        if patterns.iter().any(|p| p.pattern_type == "Revenge Trading") {
            recommendations.push(
                "Implement a mandatory cool-down period after losses to avoid revenge trading."
                    .to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations.push(
                "Great job! Keep maintaining your discipline and emotional control.".to_string(),
            );
        }

        recommendations
    }

    pub fn calculate_behavioral_analytics(entries: &[JournalEntry]) -> BehavioralAnalytics {
        let total_entries = entries.len();

        let consistency_score = Self::calculate_consistency_score(entries);
        let emotional_volatility = Self::calculate_emotional_volatility(entries);
        let discipline_trend = Self::calculate_discipline_trend(entries);
        let win_rate_by_emotion = Self::calculate_win_rate_by_emotion(entries);
        let best_trading_hours = Self::calculate_best_trading_hours(entries);
        let cognitive_biases = Self::analyze_cognitive_biases(entries);
        let growth_indicators = Self::calculate_growth_indicators(entries);

        BehavioralAnalytics {
            total_entries,
            consistency_score,
            emotional_volatility,
            discipline_trend,
            win_rate_by_emotion,
            best_trading_hours,
            cognitive_biases,
            growth_indicators,
        }
    }

    fn calculate_consistency_score(entries: &[JournalEntry]) -> f32 {
        if entries.len() < 7 {
            return 0.5;
        }

        let days_with_entries = entries
            .iter()
            .map(|e| e.timestamp / (24 * 60 * 60))
            .collect::<std::collections::HashSet<_>>()
            .len();

        let weeks = entries.len() / 7;
        let expected_days = weeks * 5;

        if expected_days > 0 {
            (days_with_entries as f32 / expected_days as f32).min(1.0)
        } else {
            0.5
        }
    }

    fn calculate_emotional_volatility(entries: &[JournalEntry]) -> f32 {
        if entries.is_empty() {
            return 0.0;
        }

        let stress_values: Vec<f32> = entries.iter().map(|e| e.emotions.stress_level).collect();

        let mean = stress_values.iter().sum::<f32>() / stress_values.len() as f32;
        let variance = stress_values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>()
            / stress_values.len() as f32;

        variance.sqrt()
    }

    fn calculate_discipline_trend(entries: &[JournalEntry]) -> Vec<DisciplineTrendPoint> {
        entries
            .iter()
            .map(|e| DisciplineTrendPoint {
                timestamp: e.timestamp,
                score: e.emotions.discipline_score,
            })
            .collect()
    }

    fn calculate_win_rate_by_emotion(entries: &[JournalEntry]) -> HashMap<String, f32> {
        let mut emotion_stats: HashMap<String, (usize, usize)> = HashMap::new();

        for entry in entries {
            if let Some(outcome) = &entry.outcome {
                let emotion = format!("{:?}", entry.emotions.primary_emotion);
                let (wins, total) = emotion_stats.entry(emotion).or_insert((0, 0));
                *total += 1;
                if outcome.success {
                    *wins += 1;
                }
            }
        }

        emotion_stats
            .into_iter()
            .map(|(emotion, (wins, total))| {
                let win_rate = if total > 0 {
                    (wins as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                (emotion, win_rate)
            })
            .collect()
    }

    fn calculate_best_trading_hours(entries: &[JournalEntry]) -> Vec<usize> {
        let mut hour_stats: HashMap<usize, (usize, usize)> = HashMap::new();

        for entry in entries {
            let hour = ((entry.timestamp / 3600) % 24) as usize;
            if let Some(outcome) = &entry.outcome {
                let (wins, total) = hour_stats.entry(hour).or_insert((0, 0));
                *total += 1;
                if outcome.success {
                    *wins += 1;
                }
            }
        }

        let mut hour_rates: Vec<_> = hour_stats
            .into_iter()
            .filter(|(_, (_, total))| *total >= 3)
            .map(|(hour, (wins, total))| {
                let win_rate = wins as f32 / total as f32;
                (hour, win_rate)
            })
            .collect();

        hour_rates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        hour_rates
            .into_iter()
            .take(3)
            .map(|(hour, _)| hour)
            .collect()
    }

    fn analyze_cognitive_biases(entries: &[JournalEntry]) -> Vec<CognitiveBias> {
        let mut biases = Vec::new();

        let overconfident_losses = entries
            .iter()
            .filter(|e| {
                e.confidence_level > 0.8 && e.outcome.as_ref().map(|o| !o.success).unwrap_or(false)
            })
            .count();

        if overconfident_losses > 2 {
            biases.push(CognitiveBias {
                bias_type: BiasType::OverconfidenceBias,
                severity: 0.7,
                instances: overconfident_losses,
                description: "Overconfidence leading to losses despite high confidence scores".to_string(),
                mitigation_strategy: "Double-check your analysis and consider contrary evidence before high-conviction trades".to_string(),
            });
        }

        let fomo_trades = entries
            .iter()
            .filter(|e| e.emotions.fomo_level > 0.7)
            .count();

        if fomo_trades > 1 {
            biases.push(CognitiveBias {
                bias_type: BiasType::HerdMentality,
                severity: 0.6,
                instances: fomo_trades,
                description: "Trading based on FOMO rather than analytical setups".to_string(),
                mitigation_strategy:
                    "Create strict entry criteria and wait for your setups rather than chasing"
                        .to_string(),
            });
        }

        let revenge_trades = entries
            .iter()
            .filter(|e| e.emotions.revenge_trading)
            .count();

        if revenge_trades > 0 {
            biases.push(CognitiveBias {
                bias_type: BiasType::LossAversion,
                severity: 0.9,
                instances: revenge_trades,
                description: "Attempting to recover losses through impulsive revenge trading"
                    .to_string(),
                mitigation_strategy:
                    "Implement mandatory breaks after losses and review journal before next trade"
                        .to_string(),
            });
        }

        biases
    }

    fn calculate_growth_indicators(entries: &[JournalEntry]) -> GrowthIndicators {
        if entries.len() < 10 {
            return GrowthIndicators {
                improvement_rate: 0.0,
                consistency_improvement: 0.0,
                emotional_control_improvement: 0.0,
                strategy_refinement_score: 0.0,
            };
        }

        let mid_point = entries.len() / 2;
        let first_half = &entries[..mid_point];
        let second_half = &entries[mid_point..];

        let first_discipline = first_half
            .iter()
            .map(|e| e.emotions.discipline_score)
            .sum::<f32>()
            / first_half.len() as f32;

        let second_discipline = second_half
            .iter()
            .map(|e| e.emotions.discipline_score)
            .sum::<f32>()
            / second_half.len() as f32;

        let first_stress = first_half
            .iter()
            .map(|e| e.emotions.stress_level)
            .sum::<f32>()
            / first_half.len() as f32;

        let second_stress = second_half
            .iter()
            .map(|e| e.emotions.stress_level)
            .sum::<f32>()
            / second_half.len() as f32;

        let improvement_rate = (second_discipline - first_discipline).max(0.0);
        let emotional_control_improvement = (first_stress - second_stress).max(0.0);

        GrowthIndicators {
            improvement_rate,
            consistency_improvement: 0.5,
            emotional_control_improvement,
            strategy_refinement_score: 0.7,
        }
    }
}

struct StrategyStats {
    count: usize,
    wins: usize,
    total_pnl: f32,
    total_confidence: f32,
    emotions: HashMap<String, usize>,
}
