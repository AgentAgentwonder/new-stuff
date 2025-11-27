use serde::{Deserialize, Serialize};

use super::ModelMetrics;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchTrainingSummary {
    pub model_version: i64,
    pub metrics: ModelMetrics,
    pub trained_at: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchBiasMetric {
    pub segment: String,
    pub sample_size: usize,
    pub success_rate: f64,
    pub delta_from_global: f64,
    pub adverse_impact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchBiasReport {
    pub generated_at: String,
    pub global_success_rate: f64,
    pub metrics: Vec<LaunchBiasMetric>,
    pub flagged_segments: Vec<String>,
}
