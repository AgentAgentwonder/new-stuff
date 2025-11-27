#[cfg(test)]
mod voice_tests {
    use app_lib::voice::audio_manager::AudioContextManager;
    use app_lib::voice::speech_to_text::{SpeechToTextConfig, SpeechToTextEngine};
    use app_lib::voice::text_to_speech::{TextToSpeechConfig, TextToSpeechEngine};
    use app_lib::voice::wake_word::{WakeWordConfig, WakeWordDetector};

    #[test]
    fn test_audio_manager_permissions_flow() {
        let manager = AudioContextManager::new();

        // Initial state should deny permissions
        let status = manager.snapshot().unwrap();
        assert!(!status.permissions.granted);
        assert!(status.session_id.is_none());

        // Request permissions
        let perm_status = manager.request_permissions().unwrap();
        assert!(perm_status.granted);

        // Should still not have active session until microphone is started
        let status = manager.snapshot().unwrap();
        assert!(!status.microphone_active);

        // Start microphone
        let session = manager.start_microphone().unwrap();
        assert!(session.microphone_active);
        assert!(session.session_id.is_some());
        assert!(session.stream_token.is_some());

        // Stop microphone
        manager.stop_microphone().unwrap();
        let status = manager.snapshot().unwrap();
        assert!(!status.microphone_active);
        assert!(status.session_id.is_none());
        assert!(status.stream_token.is_none());

        // Revoke permissions
        manager
            .revoke_permissions(Some("Test revocation".to_string()))
            .unwrap();
        let status = manager.snapshot().unwrap();
        assert!(!status.permissions.granted);
        assert_eq!(
            status.permissions.reason,
            Some("Test revocation".to_string())
        );
    }

    #[test]
    fn test_audio_manager_require_permissions() {
        let manager = AudioContextManager::new();

        // Starting microphone without permissions should fail
        let result = manager.start_microphone();
        assert!(result.is_err());
    }

    #[test]
    fn test_wake_word_detector_lifecycle() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);

        // Initial state
        assert!(!detector.is_listening());

        // Start listening
        detector.start_listening().unwrap();
        assert!(detector.is_listening());

        // Stop listening
        detector.stop_listening().unwrap();
        assert!(!detector.is_listening());
    }

    #[test]
    fn test_wake_word_config_updates() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);

        // Change config
        let new_config = WakeWordConfig {
            enabled: false,
            wake_word: "Custom Wake Word".to_string(),
            sensitivity: 0.8,
            timeout_ms: 5000,
        };

        detector.update_config(new_config.clone()).unwrap();
        let updated = detector.get_config().unwrap();

        assert!(!updated.enabled);
        assert_eq!(updated.wake_word, "Custom Wake Word");
        assert_eq!(updated.sensitivity, 0.8);
        assert_eq!(updated.timeout_ms, 5000);
    }

    #[test]
    fn test_wake_word_audio_processing() {
        let config = WakeWordConfig::default();
        let detector = WakeWordDetector::new(config);

        // Generate sample audio data (low energy)
        let low_energy_samples: Vec<f32> = vec![0.01; 1000];
        let result = detector.process_audio(&low_energy_samples).unwrap();
        assert!(result.is_none()); // Should not detect with low energy

        // Generate sample audio data (high energy)
        let high_energy_samples: Vec<f32> = vec![0.8; 1000];
        let result = detector.process_audio(&high_energy_samples).unwrap();
        assert!(result.is_some()); // Should detect with high energy

        if let Some(detection) = result {
            assert!(detection.detected);
            assert!(detection.confidence > 0.0);
        }
    }

    #[test]
    fn test_stt_engine_lifecycle() {
        let config = SpeechToTextConfig::default();
        let engine = SpeechToTextEngine::new(config);

        assert!(!engine.is_recognizing());

        engine.start_recognition().unwrap();
        assert!(engine.is_recognizing());

        engine.stop_recognition().unwrap();
        assert!(!engine.is_recognizing());
    }

    #[test]
    fn test_stt_disabled_cannot_start() {
        let mut config = SpeechToTextConfig::default();
        config.enabled = false;
        let engine = SpeechToTextEngine::new(config);

        let result = engine.start_recognition();
        assert!(result.is_err());
    }

    #[test]
    fn test_stt_config_updates() {
        let config = SpeechToTextConfig::default();
        let engine = SpeechToTextEngine::new(config);

        // Update language
        engine.set_language("es-ES".to_string()).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.language, "es-ES");

        // Update full config
        let new_config = SpeechToTextConfig {
            enabled: true,
            language: "fr-FR".to_string(),
            continuous: true,
            interim_results: false,
            max_alternatives: 3,
        };

        engine.update_config(new_config.clone()).unwrap();
        let updated = engine.get_config().unwrap();
        assert_eq!(updated.language, "fr-FR");
        assert!(updated.continuous);
        assert!(!updated.interim_results);
        assert_eq!(updated.max_alternatives, 3);
    }

    #[test]
    fn test_stt_supported_languages() {
        let languages = SpeechToTextEngine::get_supported_languages();
        assert!(!languages.is_empty());

        // Check for common languages
        assert!(languages.iter().any(|l| l.code == "en-US"));
        assert!(languages.iter().any(|l| l.code == "es-ES"));
        assert!(languages.iter().any(|l| l.code == "fr-FR"));
        assert!(languages.iter().any(|l| l.code == "zh-CN"));
    }

    #[test]
    fn test_tts_engine_lifecycle() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        assert!(!engine.is_speaking());

        engine.speak("Test message".to_string()).unwrap();
        assert!(engine.is_speaking());

        engine.stop().unwrap();
        assert!(!engine.is_speaking());
    }

    #[test]
    fn test_tts_empty_text_error() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        let result = engine.speak("   ".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_tts_disabled_cannot_speak() {
        let mut config = TextToSpeechConfig::default();
        config.enabled = false;
        let engine = TextToSpeechEngine::new(config);

        let result = engine.speak("Test".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_tts_voice_selection() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        engine.set_voice("en-us-male".to_string()).unwrap();
        let updated_config = engine.get_config().unwrap();
        assert_eq!(updated_config.voice, "en-us-male");
    }

    #[test]
    fn test_tts_available_voices() {
        let voices = TextToSpeechEngine::get_available_voices();
        assert!(!voices.is_empty());

        // Check default voice exists
        assert!(voices.iter().any(|v| v.id == "default"));
    }

    #[test]
    fn test_tts_parameter_clamping() {
        let config = TextToSpeechConfig::default();
        let engine = TextToSpeechEngine::new(config);

        // Test rate clamping
        engine.set_rate(15.0).unwrap();
        let config = engine.get_config().unwrap();
        assert_eq!(config.rate, 10.0);

        engine.set_rate(0.05).unwrap();
        let config = engine.get_config().unwrap();
        assert_eq!(config.rate, 0.1);

        // Test volume clamping
        engine.set_volume(2.0).unwrap();
        let config = engine.get_config().unwrap();
        assert_eq!(config.volume, 1.0);

        engine.set_volume(-0.5).unwrap();
        let config = engine.get_config().unwrap();
        assert_eq!(config.volume, 0.0);

        // Test pitch clamping
        engine.set_pitch(3.0).unwrap();
        let config = engine.get_config().unwrap();
        assert_eq!(config.pitch, 2.0);

        engine.set_pitch(-0.5).unwrap();
        let config = engine.get_config().unwrap();
        assert_eq!(config.pitch, 0.0);
    }

    #[test]
    fn test_integration_voice_pipeline() {
        // Initialize all components
        let audio_manager = AudioContextManager::new();
        let wake_word_detector = WakeWordDetector::new(WakeWordConfig::default());
        let stt_engine = SpeechToTextEngine::new(SpeechToTextConfig::default());
        let tts_engine = TextToSpeechEngine::new(TextToSpeechConfig::default());

        // Step 1: Request permissions
        let perm_status = audio_manager.request_permissions().unwrap();
        assert!(perm_status.granted);

        // Step 2: Start microphone
        let session = audio_manager.start_microphone().unwrap();
        assert!(session.microphone_active);

        // Step 3: Start wake word detection
        wake_word_detector.start_listening().unwrap();
        assert!(wake_word_detector.is_listening());

        // Step 4: Simulate audio and detect wake word
        let samples: Vec<f32> = vec![0.7; 1000];
        let wake_detection = wake_word_detector.process_audio(&samples).unwrap();
        assert!(wake_detection.is_some());

        // Step 5: Start speech recognition
        stt_engine.start_recognition().unwrap();
        assert!(stt_engine.is_recognizing());

        // Step 6: Speak confirmation
        tts_engine.speak("Command received".to_string()).unwrap();
        assert!(tts_engine.is_speaking());

        // Step 7: Clean up
        tts_engine.stop().unwrap();
        stt_engine.stop_recognition().unwrap();
        wake_word_detector.stop_listening().unwrap();
        audio_manager.stop_microphone().unwrap();

        // Verify clean state
        assert!(!tts_engine.is_speaking());
        assert!(!stt_engine.is_recognizing());
        assert!(!wake_word_detector.is_listening());

        let status = audio_manager.snapshot().unwrap();
        assert!(!status.microphone_active);
    }
}
