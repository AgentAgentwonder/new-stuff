use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PriceData {
    pub timestamp: i64,
    pub price: f64,
    pub volume: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionData {
    pub timestamp: i64,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub price: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Anomaly {
    pub id: String,
    pub token_address: String,
    pub anomaly_type: String,
    pub severity: String,
    pub timestamp: i64,
    pub value: f64,
    pub threshold: f64,
    pub explanation: String,
    pub details: HashMap<String, String>,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnomalyDetectionConfig {
    pub enabled: bool,
    pub zscore_threshold: f64,
    pub iqr_multiplier: f64,
    pub wash_trading_threshold: f64,
    pub min_data_points: usize,
    pub notification_channels: Vec<String>,
}

pub type SharedAnomalyDetector = Arc<RwLock<AnomalyDetector>>;

pub struct AnomalyDetector {
    price_history: HashMap<String, Vec<PriceData>>,
    transaction_history: HashMap<String, Vec<TransactionData>>,
    anomalies: Vec<Anomaly>,
    config: AnomalyDetectionConfig,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            price_history: HashMap::new(),
            transaction_history: HashMap::new(),
            anomalies: Vec::new(),
            config: AnomalyDetectionConfig {
                enabled: true,
                zscore_threshold: 3.0,
                iqr_multiplier: 1.5,
                wash_trading_threshold: 0.8,
                min_data_points: 20,
                notification_channels: vec!["in-app".to_string()],
            },
        }
    }

    pub fn add_price_data(&mut self, token_address: String, data: PriceData) {
        let history = self
            .price_history
            .entry(token_address.clone())
            .or_insert_with(Vec::new);
        history.push(data);

        if history.len() > 1000 {
            history.drain(0..history.len() - 1000);
        }

        if self.config.enabled && history.len() >= self.config.min_data_points {
            self.detect_price_anomalies(&token_address);
        }
    }

    pub fn add_transaction_data(&mut self, token_address: String, data: TransactionData) {
        let history = self
            .transaction_history
            .entry(token_address.clone())
            .or_insert_with(Vec::new);
        history.push(data);

        if history.len() > 1000 {
            history.drain(0..history.len() - 1000);
        }

        if self.config.enabled && history.len() >= self.config.min_data_points {
            self.detect_wash_trading(&token_address);
        }
    }

    fn detect_price_anomalies(&mut self, token_address: &str) {
        if self.price_history.contains_key(token_address) {
            let history = self.price_history[token_address].clone();
            if history.len() < self.config.min_data_points {
                return;
            }

            let latest = &history[history.len() - 1];

            self.detect_zscore_anomaly(token_address, &history, latest);
            self.detect_iqr_anomaly(token_address, &history, latest);
            self.detect_volume_anomaly(token_address, &history, latest);
        }
    }

    fn detect_zscore_anomaly(
        &mut self,
        token_address: &str,
        history: &[PriceData],
        latest: &PriceData,
    ) {
        let prices: Vec<f64> = history.iter().map(|d| d.price).collect();
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance =
            prices.iter().map(|&p| (p - mean).powi(2)).sum::<f64>() / prices.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            let zscore = (latest.price - mean) / std_dev;

            if zscore.abs() >= self.config.zscore_threshold {
                let severity = if zscore.abs() >= 5.0 {
                    "critical"
                } else if zscore.abs() >= 4.0 {
                    "high"
                } else {
                    "medium"
                };

                let direction = if zscore > 0.0 { "above" } else { "below" };
                let explanation = format!(
                    "Price anomaly detected using Z-score method. Current price (${:.6}) is {:.2} standard deviations {} the mean (${:.6}). \
                    This represents a significant deviation from typical price behavior and may indicate unusual market activity.",
                    latest.price, zscore.abs(), direction, mean
                );

                let mut details = HashMap::new();
                details.insert("method".to_string(), "zscore".to_string());
                details.insert("zscore".to_string(), format!("{:.2}", zscore));
                details.insert("mean".to_string(), format!("{:.6}", mean));
                details.insert("std_dev".to_string(), format!("{:.6}", std_dev));
                details.insert("current_price".to_string(), format!("{:.6}", latest.price));

                let anomaly = Anomaly {
                    id: uuid::Uuid::new_v4().to_string(),
                    token_address: token_address.to_string(),
                    anomaly_type: "price_zscore".to_string(),
                    severity: severity.to_string(),
                    timestamp: latest.timestamp,
                    value: latest.price,
                    threshold: self.config.zscore_threshold,
                    explanation,
                    details,
                    is_active: true,
                };

                self.anomalies.push(anomaly);
            }
        }
    }

    fn detect_iqr_anomaly(
        &mut self,
        token_address: &str,
        history: &[PriceData],
        latest: &PriceData,
    ) {
        let mut prices: Vec<f64> = history.iter().map(|d| d.price).collect();
        prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = prices.len();
        let q1_idx = n / 4;
        let q3_idx = (3 * n) / 4;

        if q3_idx < prices.len() {
            let q1 = prices[q1_idx];
            let q3 = prices[q3_idx];
            let iqr = q3 - q1;

            let lower_bound = q1 - self.config.iqr_multiplier * iqr;
            let upper_bound = q3 + self.config.iqr_multiplier * iqr;

            if latest.price < lower_bound || latest.price > upper_bound {
                let severity = if latest.price < q1 - 3.0 * iqr || latest.price > q3 + 3.0 * iqr {
                    "high"
                } else {
                    "medium"
                };

                let direction = if latest.price < lower_bound {
                    "below"
                } else {
                    "above"
                };
                let bound = if latest.price < lower_bound {
                    lower_bound
                } else {
                    upper_bound
                };

                let explanation = format!(
                    "Price outlier detected using IQR (Interquartile Range) method. Current price (${:.6}) is {} the expected range \
                    [${:.6} - ${:.6}]. Q1=${:.6}, Q3=${:.6}, IQR=${:.6}. This suggests the price is outside normal variation and \
                    may indicate market manipulation or significant news events.",
                    latest.price, direction, lower_bound, upper_bound, q1, q3, iqr
                );

                let mut details = HashMap::new();
                details.insert("method".to_string(), "iqr".to_string());
                details.insert("q1".to_string(), format!("{:.6}", q1));
                details.insert("q3".to_string(), format!("{:.6}", q3));
                details.insert("iqr".to_string(), format!("{:.6}", iqr));
                details.insert("lower_bound".to_string(), format!("{:.6}", lower_bound));
                details.insert("upper_bound".to_string(), format!("{:.6}", upper_bound));
                details.insert("current_price".to_string(), format!("{:.6}", latest.price));

                let anomaly = Anomaly {
                    id: uuid::Uuid::new_v4().to_string(),
                    token_address: token_address.to_string(),
                    anomaly_type: "price_iqr".to_string(),
                    severity: severity.to_string(),
                    timestamp: latest.timestamp,
                    value: latest.price,
                    threshold: self.config.iqr_multiplier,
                    explanation,
                    details,
                    is_active: true,
                };

                self.anomalies.push(anomaly);
            }
        }
    }

    fn detect_volume_anomaly(
        &mut self,
        token_address: &str,
        history: &[PriceData],
        latest: &PriceData,
    ) {
        let volumes: Vec<f64> = history.iter().map(|d| d.volume).collect();
        let mean_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;

        if mean_volume > 0.0 {
            let volume_ratio = latest.volume / mean_volume;

            if volume_ratio >= 5.0 {
                let severity = if volume_ratio >= 10.0 {
                    "high"
                } else {
                    "medium"
                };

                let explanation = format!(
                    "Unusual volume spike detected. Current volume (${:.2}) is {:.2}x the average volume (${:.2}). \
                    This significant increase in trading volume may indicate: 1) Major news or announcements, \
                    2) Whale activity, 3) Coordinated trading, or 4) Market manipulation attempts.",
                    latest.volume, volume_ratio, mean_volume
                );

                let mut details = HashMap::new();
                details.insert("method".to_string(), "volume_spike".to_string());
                details.insert(
                    "current_volume".to_string(),
                    format!("{:.2}", latest.volume),
                );
                details.insert("mean_volume".to_string(), format!("{:.2}", mean_volume));
                details.insert("volume_ratio".to_string(), format!("{:.2}", volume_ratio));

                let anomaly = Anomaly {
                    id: uuid::Uuid::new_v4().to_string(),
                    token_address: token_address.to_string(),
                    anomaly_type: "volume_spike".to_string(),
                    severity: severity.to_string(),
                    timestamp: latest.timestamp,
                    value: latest.volume,
                    threshold: 5.0,
                    explanation,
                    details,
                    is_active: true,
                };

                self.anomalies.push(anomaly);
            }
        }

        if self.anomalies.len() > 200 {
            self.anomalies.drain(0..self.anomalies.len() - 200);
        }
    }

    fn detect_wash_trading(&mut self, token_address: &str) {
        if let Some(history) = self.transaction_history.get(token_address) {
            if history.len() < self.config.min_data_points {
                return;
            }

            let recent_window = 50.min(history.len());
            let recent = &history[history.len() - recent_window..];

            let mut address_pairs: HashMap<(String, String), Vec<&TransactionData>> =
                HashMap::new();

            for tx in recent {
                let pair = if tx.from < tx.to {
                    (tx.from.clone(), tx.to.clone())
                } else {
                    (tx.to.clone(), tx.from.clone())
                };
                address_pairs.entry(pair).or_insert_with(Vec::new).push(tx);
            }

            for ((addr1, addr2), txs) in address_pairs.iter() {
                if txs.len() >= 3 {
                    let back_and_forth = self.analyze_back_and_forth_pattern(txs);

                    if back_and_forth >= self.config.wash_trading_threshold {
                        let total_volume: f64 = txs.iter().map(|tx| tx.amount * tx.price).sum();
                        let avg_price: f64 =
                            txs.iter().map(|tx| tx.price).sum::<f64>() / txs.len() as f64;

                        let explanation = format!(
                            "Potential wash trading detected between addresses {} and {}. \
                            {} transactions observed with {:.0}% back-and-forth pattern. \
                            Total volume: ${:.2}, Average price: ${:.6}. \
                            Wash trading involves buying and selling the same asset to create misleading market activity. \
                            This pattern suggests artificial volume inflation.",
                            &addr1[..8], &addr2[..8], txs.len(), back_and_forth * 100.0, total_volume, avg_price
                        );

                        let mut details = HashMap::new();
                        details.insert("method".to_string(), "wash_trading".to_string());
                        details.insert("address_1".to_string(), addr1.clone());
                        details.insert("address_2".to_string(), addr2.clone());
                        details.insert("transaction_count".to_string(), txs.len().to_string());
                        details.insert(
                            "pattern_score".to_string(),
                            format!("{:.2}", back_and_forth),
                        );
                        details.insert("total_volume".to_string(), format!("{:.2}", total_volume));
                        details.insert("avg_price".to_string(), format!("{:.6}", avg_price));

                        let anomaly = Anomaly {
                            id: uuid::Uuid::new_v4().to_string(),
                            token_address: token_address.to_string(),
                            anomaly_type: "wash_trading".to_string(),
                            severity: "high".to_string(),
                            timestamp: Utc::now().timestamp(),
                            value: back_and_forth,
                            threshold: self.config.wash_trading_threshold,
                            explanation,
                            details,
                            is_active: true,
                        };

                        self.anomalies.push(anomaly);
                    }
                }
            }
        }
    }

    fn analyze_back_and_forth_pattern(&self, txs: &[&TransactionData]) -> f64 {
        if txs.len() < 2 {
            return 0.0;
        }

        let mut alternations = 0;
        for i in 1..txs.len() {
            let prev = txs[i - 1];
            let curr = txs[i];

            if (prev.from == curr.to && prev.to == curr.from)
                || (prev.from == curr.from && prev.to == curr.to)
            {
                alternations += 1;
            }
        }

        alternations as f64 / (txs.len() - 1) as f64
    }

    pub fn get_anomalies(
        &self,
        token_address: Option<&str>,
        anomaly_type: Option<&str>,
    ) -> Vec<Anomaly> {
        self.anomalies
            .iter()
            .filter(|a| {
                let token_match = token_address.map_or(true, |addr| a.token_address == addr);
                let type_match = anomaly_type.map_or(true, |typ| a.anomaly_type == typ);
                token_match && type_match
            })
            .cloned()
            .collect()
    }

    pub fn get_active_anomalies(&self) -> Vec<Anomaly> {
        self.anomalies
            .iter()
            .filter(|a| a.is_active)
            .cloned()
            .collect()
    }

    pub fn dismiss_anomaly(&mut self, anomaly_id: &str) {
        if let Some(anomaly) = self.anomalies.iter_mut().find(|a| a.id == anomaly_id) {
            anomaly.is_active = false;
        }
    }

    pub fn update_config(&mut self, config: AnomalyDetectionConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> AnomalyDetectionConfig {
        self.config.clone()
    }

    pub fn get_statistics(&self, token_address: &str) -> Option<AnomalyStatistics> {
        let token_anomalies: Vec<&Anomaly> = self
            .anomalies
            .iter()
            .filter(|a| a.token_address == token_address)
            .collect();

        if token_anomalies.is_empty() {
            return None;
        }

        let total = token_anomalies.len();
        let by_type: HashMap<String, usize> =
            token_anomalies.iter().fold(HashMap::new(), |mut acc, a| {
                *acc.entry(a.anomaly_type.clone()).or_insert(0) += 1;
                acc
            });

        let by_severity: HashMap<String, usize> =
            token_anomalies.iter().fold(HashMap::new(), |mut acc, a| {
                *acc.entry(a.severity.clone()).or_insert(0) += 1;
                acc
            });

        Some(AnomalyStatistics {
            token_address: token_address.to_string(),
            total_anomalies: total,
            active_anomalies: token_anomalies.iter().filter(|a| a.is_active).count(),
            by_type,
            by_severity,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnomalyStatistics {
    pub token_address: String,
    pub total_anomalies: usize,
    pub active_anomalies: usize,
    pub by_type: HashMap<String, usize>,
    pub by_severity: HashMap<String, usize>,
}

#[tauri::command]
pub async fn add_price_data(
    token_address: String,
    data: PriceData,
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<(), String> {
    let mut det = detector.write().await;
    det.add_price_data(token_address, data);
    Ok(())
}

#[tauri::command]
pub async fn add_transaction_data(
    token_address: String,
    data: TransactionData,
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<(), String> {
    let mut det = detector.write().await;
    det.add_transaction_data(token_address, data);
    Ok(())
}

#[tauri::command]
pub async fn get_anomalies(
    token_address: Option<String>,
    anomaly_type: Option<String>,
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<Vec<Anomaly>, String> {
    let det = detector.read().await;
    Ok(det.get_anomalies(token_address.as_deref(), anomaly_type.as_deref()))
}

#[tauri::command]
pub async fn get_active_anomalies(
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<Vec<Anomaly>, String> {
    let det = detector.read().await;
    Ok(det.get_active_anomalies())
}

#[tauri::command]
pub async fn dismiss_anomaly(
    anomaly_id: String,
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<(), String> {
    let mut det = detector.write().await;
    det.dismiss_anomaly(&anomaly_id);
    Ok(())
}

#[tauri::command]
pub async fn update_anomaly_detection_config(
    config: AnomalyDetectionConfig,
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<(), String> {
    let mut det = detector.write().await;
    det.update_config(config);
    Ok(())
}

#[tauri::command]
pub async fn get_anomaly_detection_config(
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<AnomalyDetectionConfig, String> {
    let det = detector.read().await;
    Ok(det.get_config())
}

#[tauri::command]
pub async fn get_anomaly_statistics(
    token_address: String,
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<Option<AnomalyStatistics>, String> {
    let det = detector.read().await;
    Ok(det.get_statistics(&token_address))
}

#[tauri::command]
pub async fn generate_mock_anomaly_data(
    token_address: String,
    detector: tauri::State<'_, SharedAnomalyDetector>,
) -> Result<(), String> {
    let mut det = detector.write().await;
    let now = Utc::now().timestamp();

    for i in 0..50 {
        let base_price = 100.0;
        let normal_variation = ((i as f64 % 7.0) - 3.0) * 1.2;
        let anomaly_variation = if i == 45 { 50.0 } else { 0.0 };

        let price = base_price + normal_variation + anomaly_variation;
        let base_volume = 10000.0 + ((i as f64 % 5.0) - 2.0) * 600.0;
        let anomaly_volume = if i == 48 { 50000.0 } else { 0.0 };

        let data = PriceData {
            timestamp: now - (50 - i) * 3600,
            price,
            volume: base_volume + anomaly_volume,
        };

        det.add_price_data(token_address.clone(), data);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zscore_anomaly_detection() {
        let mut detector = AnomalyDetector::new();
        detector.config.min_data_points = 10;

        let token_address = "test_token".to_string();
        let now = Utc::now().timestamp();

        for i in 0..10 {
            let data = PriceData {
                timestamp: now + i * 60,
                price: 100.0,
                volume: 1000.0,
            };
            detector.add_price_data(token_address.clone(), data);
        }

        let anomaly_data = PriceData {
            timestamp: now + 600,
            price: 200.0,
            volume: 1000.0,
        };
        detector.add_price_data(token_address.clone(), anomaly_data);

        let anomalies = detector.get_anomalies(Some(&token_address), None);
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| a.anomaly_type == "price_zscore"));
    }

    #[test]
    fn test_volume_spike_detection() {
        let mut detector = AnomalyDetector::new();
        detector.config.min_data_points = 10;

        let token_address = "test_token".to_string();
        let now = Utc::now().timestamp();

        for i in 0..10 {
            let data = PriceData {
                timestamp: now + i * 60,
                price: 100.0,
                volume: 1000.0,
            };
            detector.add_price_data(token_address.clone(), data);
        }

        let spike_data = PriceData {
            timestamp: now + 600,
            price: 100.0,
            volume: 10000.0,
        };
        detector.add_price_data(token_address.clone(), spike_data);

        let anomalies = detector.get_anomalies(Some(&token_address), Some("volume_spike"));
        assert!(!anomalies.is_empty());
    }

    #[test]
    fn test_wash_trading_detection() {
        let mut detector = AnomalyDetector::new();
        detector.config.min_data_points = 10;
        detector.config.wash_trading_threshold = 0.5;

        let token_address = "test_token".to_string();
        let now = Utc::now().timestamp();
        let addr1 = "address1".to_string();
        let addr2 = "address2".to_string();

        for i in 0..10 {
            let from = if i % 2 == 0 {
                addr1.clone()
            } else {
                addr2.clone()
            };
            let to = if i % 2 == 0 {
                addr2.clone()
            } else {
                addr1.clone()
            };

            let data = TransactionData {
                timestamp: now + i * 60,
                from,
                to,
                amount: 100.0,
                price: 1.0,
            };
            detector.add_transaction_data(token_address.clone(), data);
        }

        let anomalies = detector.get_anomalies(Some(&token_address), Some("wash_trading"));
        assert!(!anomalies.is_empty());
    }

    #[test]
    fn test_anomaly_statistics() {
        let mut detector = AnomalyDetector::new();
        detector.config.min_data_points = 5;

        let token_address = "test_token".to_string();
        let now = Utc::now().timestamp();

        for i in 0..10 {
            let data = PriceData {
                timestamp: now + i * 60,
                price: 100.0,
                volume: 1000.0,
            };
            detector.add_price_data(token_address.clone(), data);
        }

        let anomaly_data = PriceData {
            timestamp: now + 600,
            price: 300.0,
            volume: 50000.0,
        };
        detector.add_price_data(token_address.clone(), anomaly_data);

        let stats = detector.get_statistics(&token_address);
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert!(stats.total_anomalies > 0);
    }
}
