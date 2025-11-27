use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeWordConfig {
    pub enabled: bool,
    pub wake_word: String,
    pub sensitivity: f32,
    pub timeout_ms: u64,
}

impl Default for WakeWordConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            wake_word: "Hey Eclipse".to_string(),
            sensitivity: 0.5,
            timeout_ms: 3000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeWordDetection {
    pub detected: bool,
    pub confidence: f32,
    pub timestamp: i64,
}

pub struct WakeWordDetector {
    config: Arc<Mutex<WakeWordConfig>>,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    is_listening: Arc<Mutex<bool>>,
}

impl WakeWordDetector {
    pub fn new(config: WakeWordConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            audio_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(16000 * 5))),
            is_listening: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start_listening(&self) -> Result<(), String> {
        let mut listening = self.is_listening.lock().map_err(|e| e.to_string())?;
        *listening = true;
        Ok(())
    }

    pub fn stop_listening(&self) -> Result<(), String> {
        let mut listening = self.is_listening.lock().map_err(|e| e.to_string())?;
        *listening = false;
        Ok(())
    }

    pub fn is_listening(&self) -> bool {
        self.is_listening.lock().map(|l| *l).unwrap_or(false)
    }

    pub fn process_audio(&self, samples: &[f32]) -> Result<Option<WakeWordDetection>, String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;

        if !config.enabled {
            return Ok(None);
        }

        let mut buffer = self.audio_buffer.lock().map_err(|e| e.to_string())?;

        for &sample in samples {
            if buffer.len() >= buffer.capacity() {
                buffer.pop_front();
            }
            buffer.push_back(sample);
        }

        // Simple energy-based detection (placeholder for actual wake word detection)
        // In production, this would use Porcupine, Vosk, or similar
        let energy: f32 = samples.iter().map(|s| s.abs()).sum::<f32>() / samples.len() as f32;

        if energy > config.sensitivity {
            let detection = WakeWordDetection {
                detected: true,
                confidence: energy.min(1.0),
                timestamp: chrono::Utc::now().timestamp_millis(),
            };

            buffer.clear();
            return Ok(Some(detection));
        }

        Ok(None)
    }

    pub fn update_config(&self, config: WakeWordConfig) -> Result<(), String> {
        let mut current = self.config.lock().map_err(|e| e.to_string())?;
        *current = config;
        Ok(())
    }

    pub fn get_config(&self) -> Result<WakeWordConfig, String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;
        Ok(config.clone())
    }

    pub fn clear_buffer(&self) -> Result<(), String> {
        let mut buffer = self.audio_buffer.lock().map_err(|e| e.to_string())?;
        buffer.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wake_word_detector_creation() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);
        assert!(!detector.is_listening());
    }

    #[test]
    fn test_start_stop_listening() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);

        detector.start_listening().unwrap();
        assert!(detector.is_listening());

        detector.stop_listening().unwrap();
        assert!(!detector.is_listening());
    }

    #[test]
    fn test_config_update() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);

        let new_config = WakeWordConfig {
            enabled: false,
            wake_word: "Custom Wake Word".to_string(),
            sensitivity: 0.7,
            timeout_ms: 5000,
        };

        detector.update_config(new_config.clone()).unwrap();
        let retrieved = detector.get_config().unwrap();

        assert_eq!(retrieved.enabled, false);
        assert_eq!(retrieved.wake_word, "Custom Wake Word");
        assert_eq!(retrieved.sensitivity, 0.7);
    }

    #[test]
    fn test_process_audio_disabled() {
        let mut config = WakeWordConfig::default();
        config.enabled = false;
        let detector = WakeWordDetector::new(config);

        let samples = vec![0.5; 1000];
        let result = detector.process_audio(&samples).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_process_audio_low_energy() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);

        let samples = vec![0.01; 1000];
        let result = detector.process_audio(&samples).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_buffer_clear() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);

        let samples = vec![0.5; 1000];
        detector.process_audio(&samples).unwrap();
        detector.clear_buffer().unwrap();

        // Buffer should be cleared
        assert_eq!(detector.audio_buffer.lock().unwrap().len(), 0);
    }
}
