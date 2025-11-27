use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{FeatureScore, LaunchPrediction, TokenFeatures};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchModel {
    pub weights: HashMap<String, f64>,
    pub intercept: f64,
    pub feature_importance: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub auc_roc: f64,
    pub training_samples: usize,
}

impl LaunchModel {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        let mut importance = HashMap::new();

        weights.insert("developer_reputation".to_string(), 18.0);
        importance.insert("developer_reputation".to_string(), 0.15);

        weights.insert("developer_launch_experience".to_string(), 8.0);
        importance.insert("developer_launch_experience".to_string(), 0.08);

        weights.insert("developer_success_rate".to_string(), 22.0);
        importance.insert("developer_success_rate".to_string(), 0.20);

        weights.insert("contract_complexity".to_string(), -5.0);
        importance.insert("contract_complexity".to_string(), 0.04);

        weights.insert("proxy_pattern_detected".to_string(), -8.0);
        importance.insert("proxy_pattern_detected".to_string(), 0.05);

        weights.insert("upgradeable_contract".to_string(), -6.0);
        importance.insert("upgradeable_contract".to_string(), 0.03);

        weights.insert("liquidity_depth".to_string(), 15.0);
        importance.insert("liquidity_depth".to_string(), 0.12);

        weights.insert("liquidity_ratio".to_string(), 10.0);
        importance.insert("liquidity_ratio".to_string(), 0.07);

        weights.insert("liquidity_change".to_string(), -4.0);
        importance.insert("liquidity_change".to_string(), 0.02);

        weights.insert("market_cap".to_string(), 7.0);
        importance.insert("market_cap".to_string(), 0.06);

        weights.insert("marketing_hype".to_string(), -3.0);
        importance.insert("marketing_hype".to_string(), 0.02);

        weights.insert("marketing_spend".to_string(), 5.0);
        importance.insert("marketing_spend".to_string(), 0.04);

        weights.insert("social_followers_growth".to_string(), 6.0);
        importance.insert("social_followers_growth".to_string(), 0.05);

        weights.insert("community_engagement".to_string(), 12.0);
        importance.insert("community_engagement".to_string(), 0.10);

        weights.insert("influencer_sentiment".to_string(), 9.0);
        importance.insert("influencer_sentiment".to_string(), 0.07);

        weights.insert("security_audit".to_string(), 14.0);
        importance.insert("security_audit".to_string(), 0.11);

        weights.insert("dex_depth".to_string(), 11.0);
        importance.insert("dex_depth".to_string(), 0.09);

        weights.insert("watchlist_interest".to_string(), 13.0);
        importance.insert("watchlist_interest".to_string(), 0.10);

        weights.insert("retention".to_string(), 16.0);
        importance.insert("retention".to_string(), 0.13);

        weights.insert("launch_age_days".to_string(), 4.0);
        importance.insert("launch_age_days".to_string(), 0.03);

        weights.insert("developer_reputation_x_engagement".to_string(), 10.0);
        importance.insert("developer_reputation_x_engagement".to_string(), 0.08);

        weights.insert("marketing_hype_x_social".to_string(), -2.0);
        importance.insert("marketing_hype_x_social".to_string(), 0.02);

        Self {
            weights,
            intercept: 35.0,
            feature_importance: importance,
        }
    }

    pub fn predict(&self, features: &TokenFeatures) -> LaunchPrediction {
        let feature_map = features.feature_vector();
        let mut score = self.intercept;
        let mut contributing_features = Vec::new();

        for (name, value) in &feature_map {
            if let Some(&weight) = self.weights.get(name) {
                let contribution = weight * value;
                score += contribution;

                let importance = self.feature_importance.get(name).unwrap_or(&0.0);
                let impact = if contribution > 0.0 {
                    "Positive"
                } else if contribution < 0.0 {
                    "Negative"
                } else {
                    "Neutral"
                };

                contributing_features.push((
                    name.clone(),
                    *value,
                    *importance,
                    contribution,
                    impact,
                ));
            }
        }

        let probability = sigmoid(score / 100.0);
        score = (probability * 100.0).clamp(0.0, 100.0);

        contributing_features
            .sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        let feature_scores: Vec<FeatureScore> = contributing_features
            .iter()
            .map(
                |(name, value, importance, _contribution, impact)| FeatureScore {
                    feature_name: format_feature_name(name),
                    value: *value,
                    importance: *importance,
                    impact: impact.to_string(),
                    description: describe_feature(name, *value),
                },
            )
            .collect();

        let risk_level = if score >= 70.0 {
            "Low"
        } else if score >= 50.0 {
            "Medium"
        } else if score >= 30.0 {
            "High"
        } else {
            "Critical"
        };

        let predicted_peak_timeframe = estimate_peak_timeframe(&feature_map, score);
        let confidence = calculate_confidence(&feature_map);
        let early_warnings = features.early_warnings();

        LaunchPrediction {
            token_address: features.token_address.clone(),
            success_probability: score,
            risk_level: risk_level.to_string(),
            confidence,
            predicted_peak_timeframe,
            feature_scores,
            early_warnings,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn train(&mut self, training_data: Vec<(TokenFeatures, f64)>) -> ModelMetrics {
        if training_data.is_empty() {
            return ModelMetrics {
                accuracy: 0.0,
                precision: 0.0,
                recall: 0.0,
                f1_score: 0.0,
                auc_roc: 0.0,
                training_samples: 0,
            };
        }

        let learning_rate = 0.01;
        let epochs = 100;

        for _epoch in 0..epochs {
            for (features, actual_outcome) in &training_data {
                let feature_map = features.feature_vector();
                let mut prediction = self.intercept;

                for (name, value) in &feature_map {
                    if let Some(&weight) = self.weights.get(name) {
                        prediction += weight * value;
                    }
                }

                let predicted_prob = sigmoid(prediction / 100.0);
                let error = actual_outcome - predicted_prob;

                self.intercept += learning_rate * error;

                for (name, value) in &feature_map {
                    if let Some(weight) = self.weights.get_mut(name) {
                        *weight += learning_rate * error * value;
                    }
                }
            }
        }

        self.calculate_metrics(&training_data)
    }

    fn calculate_metrics(&self, test_data: &[(TokenFeatures, f64)]) -> ModelMetrics {
        let mut correct = 0;
        let mut true_positives = 0;
        let mut false_positives = 0;
        let mut false_negatives = 0;

        for (features, actual) in test_data {
            let prediction = self.predict(features);
            let predicted_success = prediction.success_probability > 50.0;
            let actual_success = *actual > 0.5;

            if predicted_success == actual_success {
                correct += 1;
            }

            if predicted_success && actual_success {
                true_positives += 1;
            } else if predicted_success && !actual_success {
                false_positives += 1;
            } else if !predicted_success && actual_success {
                false_negatives += 1;
            }
        }

        let accuracy = correct as f64 / test_data.len() as f64;

        let precision = if (true_positives + false_positives) > 0 {
            true_positives as f64 / (true_positives + false_positives) as f64
        } else {
            0.0
        };

        let recall = if (true_positives + false_negatives) > 0 {
            true_positives as f64 / (true_positives + false_negatives) as f64
        } else {
            0.0
        };

        let f1_score = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        ModelMetrics {
            accuracy,
            precision,
            recall,
            f1_score,
            auc_roc: accuracy,
            training_samples: test_data.len(),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn format_feature_name(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn describe_feature(name: &str, value: f64) -> String {
    match name {
        "developer_reputation" => format!("Developer reputation score: {:.2}", value),
        "developer_launch_experience" => {
            format!("Developer launch experience: {:.1} launches", value.exp())
        }
        "developer_success_rate" => format!("Historical success rate: {:.0}%", value * 100.0),
        "liquidity_depth" => format!("Liquidity depth: ${:.0}", value.exp()),
        "community_engagement" => format!("Community engagement level: {:.0}%", value * 100.0),
        "security_audit" => format!("Security audit score: {:.0}%", value * 100.0),
        "watchlist_interest" => format!("Watchlist interest: {:.0}%", value * 100.0),
        "retention" => format!("Holder retention: {:.0}%", value * 100.0),
        _ => format!("{}: {:.2}", format_feature_name(name), value),
    }
}

fn estimate_peak_timeframe(features: &HashMap<String, f64>, score: f64) -> Option<String> {
    let hype = features.get("marketing_hype").unwrap_or(&0.5);
    let engagement = features.get("community_engagement").unwrap_or(&0.5);
    let liquidity = features.get("liquidity_depth").unwrap_or(&0.5);

    if score < 40.0 {
        return None;
    }

    if *hype > 0.7 && *engagement > 0.6 {
        Some("24-72 hours".to_string())
    } else if *liquidity > 0.6 && *engagement > 0.5 {
        Some("3-7 days".to_string())
    } else if score > 60.0 {
        Some("1-2 weeks".to_string())
    } else {
        Some("2-4 weeks".to_string())
    }
}

fn calculate_confidence(features: &HashMap<String, f64>) -> f64 {
    let mut confidence = 0.5;

    let has_audit = features.get("security_audit").unwrap_or(&0.0) > &0.5;
    let has_liquidity = features.get("liquidity_depth").unwrap_or(&0.0) > &0.6;
    let has_engagement = features.get("community_engagement").unwrap_or(&0.0) > &0.5;
    let has_reputation = features.get("developer_reputation").unwrap_or(&0.0) > &0.6;

    if has_audit {
        confidence += 0.1;
    }
    if has_liquidity {
        confidence += 0.15;
    }
    if has_engagement {
        confidence += 0.1;
    }
    if has_reputation {
        confidence += 0.15;
    }

    confidence.clamp(0.0, 1.0)
}
