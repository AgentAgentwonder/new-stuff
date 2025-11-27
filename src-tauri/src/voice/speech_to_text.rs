use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechToTextConfig {
    pub enabled: bool,
    pub language: String,
    pub continuous: bool,
    pub interim_results: bool,
    pub max_alternatives: u32,
}

impl Default for SpeechToTextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            language: "en-US".to_string(),
            continuous: false,
            interim_results: true,
            max_alternatives: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechRecognitionResult {
    pub transcript: String,
    pub confidence: f32,
    pub is_final: bool,
    pub alternatives: Vec<SpeechAlternative>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAlternative {
    pub transcript: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageOption {
    pub code: String,
    pub name: String,
    pub supported: bool,
}

pub struct SpeechToTextEngine {
    config: Arc<Mutex<SpeechToTextConfig>>,
    is_recognizing: Arc<Mutex<bool>>,
}

impl SpeechToTextEngine {
    pub fn new(config: SpeechToTextConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            is_recognizing: Arc::new(Mutex::new(false)),
        }
    }

    pub fn simulate_transcription(
        &self,
        transcript: &str,
        confidence: f32,
    ) -> SpeechRecognitionResult {
        let config = self.config.lock().expect("stt config poisoned").clone();
        let normalized_confidence = confidence.clamp(0.0, 1.0);

        SpeechRecognitionResult {
            transcript: transcript.trim().to_string(),
            confidence: normalized_confidence,
            is_final: true,
            alternatives: vec![SpeechAlternative {
                transcript: transcript.trim().to_string(),
                confidence: normalized_confidence,
            }],
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn start_recognition(&self) -> Result<(), String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;

        if !config.enabled {
            return Err("Speech recognition is disabled".to_string());
        }

        let mut recognizing = self.is_recognizing.lock().map_err(|e| e.to_string())?;
        *recognizing = true;

        Ok(())
    }

    pub fn stop_recognition(&self) -> Result<(), String> {
        let mut recognizing = self.is_recognizing.lock().map_err(|e| e.to_string())?;
        *recognizing = false;
        Ok(())
    }

    pub fn is_recognizing(&self) -> bool {
        self.is_recognizing.lock().map(|r| *r).unwrap_or(false)
    }

    pub fn process_audio_chunk(
        &self,
        _audio_data: &[f32],
    ) -> Result<Option<SpeechRecognitionResult>, String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;

        if !config.enabled || !self.is_recognizing() {
            return Ok(None);
        }

        // Placeholder: In production, this would use Web Speech API or native engine
        // For now, return mock data for testing
        Ok(None)
    }

    pub fn update_config(&self, config: SpeechToTextConfig) -> Result<(), String> {
        let mut current = self.config.lock().map_err(|e| e.to_string())?;
        *current = config;
        Ok(())
    }

    pub fn get_config(&self) -> Result<SpeechToTextConfig, String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;
        Ok(config.clone())
    }

    pub fn get_supported_languages() -> Vec<LanguageOption> {
        vec![
            LanguageOption {
                code: "en-US".to_string(),
                name: "English (US)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "en-GB".to_string(),
                name: "English (UK)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "es-ES".to_string(),
                name: "Spanish (Spain)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "es-MX".to_string(),
                name: "Spanish (Mexico)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "fr-FR".to_string(),
                name: "French".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "de-DE".to_string(),
                name: "German".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "it-IT".to_string(),
                name: "Italian".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "pt-BR".to_string(),
                name: "Portuguese (Brazil)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "pt-PT".to_string(),
                name: "Portuguese (Portugal)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "zh-CN".to_string(),
                name: "Chinese (Simplified)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "zh-TW".to_string(),
                name: "Chinese (Traditional)".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "ja-JP".to_string(),
                name: "Japanese".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "ko-KR".to_string(),
                name: "Korean".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "ru-RU".to_string(),
                name: "Russian".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "ar-SA".to_string(),
                name: "Arabic".to_string(),
                supported: true,
            },
            LanguageOption {
                code: "hi-IN".to_string(),
                name: "Hindi".to_string(),
                supported: true,
            },
        ]
    }

    pub fn set_language(&self, language_code: String) -> Result<(), String> {
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        config.language = language_code;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stt_engine_creation() {
        let config = SpeechToTextConfig::default();
        let engine = SpeechToTextEngine::new(config);
        assert!(!engine.is_recognizing());
    }

    #[test]
    fn test_start_stop_recognition() {
        let config = SpeechToTextConfig::default();
        let engine = SpeechToTextEngine::new(config);

        engine.start_recognition().unwrap();
        assert!(engine.is_recognizing());

        engine.stop_recognition().unwrap();
        assert!(!engine.is_recognizing());
    }

    #[test]
    fn test_disabled_recognition() {
        let mut config = SpeechToTextConfig::default();
        config.enabled = false;
        let engine = SpeechToTextEngine::new(config);

        let result = engine.start_recognition();
        assert!(result.is_err());
    }

    #[test]
    fn test_language_update() {
        let config = SpeechToTextConfig::default();
        let engine = SpeechToTextEngine::new(config);

        engine.set_language("es-ES".to_string()).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.language, "es-ES");
    }

    #[test]
    fn test_supported_languages() {
        let languages = SpeechToTextEngine::get_supported_languages();
        assert!(languages.len() > 0);
        assert!(languages.iter().any(|l| l.code == "en-US"));
    }

    #[test]
    fn test_config_update() {
        let config = SpeechToTextConfig::default();
        let engine = SpeechToTextEngine::new(config);

        let new_config = SpeechToTextConfig {
            enabled: false,
            language: "fr-FR".to_string(),
            continuous: true,
            interim_results: false,
            max_alternatives: 3,
        };

        engine.update_config(new_config.clone()).unwrap();
        let retrieved = engine.get_config().unwrap();

        assert_eq!(retrieved.enabled, false);
        assert_eq!(retrieved.language, "fr-FR");
        assert_eq!(retrieved.continuous, true);
        assert_eq!(retrieved.max_alternatives, 3);
    }
}
