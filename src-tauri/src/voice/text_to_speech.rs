use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextToSpeechConfig {
    pub enabled: bool,
    pub voice: String,
    pub rate: f32,
    pub pitch: f32,
    pub volume: f32,
    pub language: String,
}

impl Default for TextToSpeechConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            voice: "default".to_string(),
            rate: 1.0,
            pitch: 1.0,
            volume: 1.0,
            language: "en-US".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    pub id: String,
    pub name: String,
    pub language: String,
    pub gender: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSynthesisStatus {
    pub speaking: bool,
    pub paused: bool,
    pub pending: bool,
}

pub struct TextToSpeechEngine {
    config: Arc<Mutex<TextToSpeechConfig>>,
    is_speaking: Arc<Mutex<bool>>,
    queue: Arc<Mutex<Vec<String>>>,
}

impl TextToSpeechEngine {
    pub fn new(config: TextToSpeechConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            is_speaking: Arc::new(Mutex::new(false)),
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn speak(&self, text: String) -> Result<(), String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;

        if !config.enabled {
            return Err("Text-to-speech is disabled".to_string());
        }

        if text.trim().is_empty() {
            return Err("Cannot speak empty text".to_string());
        }

        let mut queue = self.queue.lock().map_err(|e| e.to_string())?;
        queue.push(text);

        let mut speaking = self.is_speaking.lock().map_err(|e| e.to_string())?;
        *speaking = true;

        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut speaking = self.is_speaking.lock().map_err(|e| e.to_string())?;
        *speaking = false;

        let mut queue = self.queue.lock().map_err(|e| e.to_string())?;
        queue.clear();

        Ok(())
    }

    pub fn pause(&self) -> Result<(), String> {
        // Placeholder for pause functionality
        Ok(())
    }

    pub fn resume(&self) -> Result<(), String> {
        // Placeholder for resume functionality
        Ok(())
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking.lock().map(|s| *s).unwrap_or(false)
    }

    pub fn get_status(&self) -> SpeechSynthesisStatus {
        let speaking = self.is_speaking();
        let pending = self.queue.lock().map(|q| !q.is_empty()).unwrap_or(false);

        SpeechSynthesisStatus {
            speaking,
            paused: false,
            pending,
        }
    }

    pub fn update_config(&self, config: TextToSpeechConfig) -> Result<(), String> {
        let mut current = self.config.lock().map_err(|e| e.to_string())?;
        *current = config;
        Ok(())
    }

    pub fn get_config(&self) -> Result<TextToSpeechConfig, String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;
        Ok(config.clone())
    }

    pub fn get_available_voices() -> Vec<Voice> {
        vec![
            Voice {
                id: "default".to_string(),
                name: "Default".to_string(),
                language: "en-US".to_string(),
                gender: "neutral".to_string(),
            },
            Voice {
                id: "en-us-male".to_string(),
                name: "English (US) - Male".to_string(),
                language: "en-US".to_string(),
                gender: "male".to_string(),
            },
            Voice {
                id: "en-us-female".to_string(),
                name: "English (US) - Female".to_string(),
                language: "en-US".to_string(),
                gender: "female".to_string(),
            },
            Voice {
                id: "en-gb-male".to_string(),
                name: "English (UK) - Male".to_string(),
                language: "en-GB".to_string(),
                gender: "male".to_string(),
            },
            Voice {
                id: "en-gb-female".to_string(),
                name: "English (UK) - Female".to_string(),
                language: "en-GB".to_string(),
                gender: "female".to_string(),
            },
        ]
    }

    pub fn set_voice(&self, voice_id: String) -> Result<(), String> {
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        config.voice = voice_id;
        Ok(())
    }

    pub fn set_rate(&self, rate: f32) -> Result<(), String> {
        let rate = rate.clamp(0.1, 10.0);
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        config.rate = rate;
        Ok(())
    }

    pub fn set_pitch(&self, pitch: f32) -> Result<(), String> {
        let pitch = pitch.clamp(0.0, 2.0);
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        config.pitch = pitch;
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        let volume = volume.clamp(0.0, 1.0);
        let mut config = self.config.lock().map_err(|e| e.to_string())?;
        config.volume = volume;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_engine_creation() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);
        assert!(!engine.is_speaking());
    }

    #[test]
    fn test_speak() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        engine.speak("Hello world".to_string()).unwrap();
        assert!(engine.is_speaking());
    }

    #[test]
    fn test_speak_empty_text() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        let result = engine.speak("   ".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_speak_disabled() {
        let mut config = TextToSpeechConfig::default();
        config.enabled = false;
        let engine = TextToSpeechEngine::new(config);

        let result = engine.speak("Hello".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_stop() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        engine.speak("Hello world".to_string()).unwrap();
        assert!(engine.is_speaking());

        engine.stop().unwrap();
        assert!(!engine.is_speaking());
    }

    #[test]
    fn test_available_voices() {
        let voices = TextToSpeechEngine::get_available_voices();
        assert!(voices.len() > 0);
        assert!(voices.iter().any(|v| v.id == "default"));
    }

    #[test]
    fn test_voice_selection() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        engine.set_voice("en-us-male".to_string()).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.voice, "en-us-male");
    }

    #[test]
    fn test_rate_clamping() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        engine.set_rate(15.0).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.rate, 10.0);

        engine.set_rate(0.05).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.rate, 0.1);
    }

    #[test]
    fn test_volume_clamping() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        engine.set_volume(1.5).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.volume, 1.0);

        engine.set_volume(-0.5).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.volume, 0.0);
    }
}
