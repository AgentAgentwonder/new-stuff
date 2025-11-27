use super::audio_manager::{AudioContextManager, AudioSessionSnapshot, MicrophoneStatus};
use super::speech_to_text::{
    LanguageOption, SpeechRecognitionResult, SpeechToTextConfig, SpeechToTextEngine,
};
use super::text_to_speech::{SpeechSynthesisStatus, TextToSpeechConfig, TextToSpeechEngine, Voice};
use super::wake_word::{WakeWordConfig, WakeWordDetection, WakeWordDetector};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub type SharedVoiceState = Arc<RwLock<VoiceState>>;

pub struct VoiceState {
    pub audio_manager: Arc<AudioContextManager>,
    pub wake_word_detector: Arc<WakeWordDetector>,
    pub stt_engine: Arc<SpeechToTextEngine>,
    pub tts_engine: Arc<TextToSpeechEngine>,
}

impl VoiceState {
    pub fn new() -> Self {
        Self {
            audio_manager: Arc::new(AudioContextManager::new()),
            wake_word_detector: Arc::new(WakeWordDetector::new(WakeWordConfig::default())),
            stt_engine: Arc::new(SpeechToTextEngine::new(SpeechToTextConfig::default())),
            tts_engine: Arc::new(TextToSpeechEngine::new(TextToSpeechConfig::default())),
        }
    }
}

impl Default for VoiceState {
    fn default() -> Self {
        Self::new()
    }
}

// Audio Manager Commands

#[tauri::command]
pub async fn voice_request_permissions(
    state: State<'_, SharedVoiceState>,
) -> Result<MicrophoneStatus, String> {
    let voice_state = state.read().await;
    voice_state.audio_manager.request_permissions()
}

#[tauri::command]
pub async fn voice_revoke_permissions(
    state: State<'_, SharedVoiceState>,
    reason: Option<String>,
) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.audio_manager.revoke_permissions(reason)
}

#[tauri::command]
pub async fn voice_start_microphone(
    state: State<'_, SharedVoiceState>,
) -> Result<AudioSessionSnapshot, String> {
    let voice_state = state.read().await;
    voice_state.audio_manager.start_microphone()
}

#[tauri::command]
pub async fn voice_stop_microphone(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.audio_manager.stop_microphone()
}

#[tauri::command]
pub async fn voice_get_audio_status(
    state: State<'_, SharedVoiceState>,
) -> Result<AudioSessionSnapshot, String> {
    let voice_state = state.read().await;
    voice_state.audio_manager.snapshot()
}

// Wake Word Commands

#[tauri::command]
pub async fn voice_start_wake_word(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.wake_word_detector.start_listening()
}

#[tauri::command]
pub async fn voice_stop_wake_word(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.wake_word_detector.stop_listening()
}

#[tauri::command]
pub async fn voice_get_wake_word_config(
    state: State<'_, SharedVoiceState>,
) -> Result<WakeWordConfig, String> {
    let voice_state = state.read().await;
    voice_state.wake_word_detector.get_config()
}

#[tauri::command]
pub async fn voice_update_wake_word_config(
    state: State<'_, SharedVoiceState>,
    config: WakeWordConfig,
) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.wake_word_detector.update_config(config)
}

#[tauri::command]
pub async fn voice_process_audio_for_wake_word(
    state: State<'_, SharedVoiceState>,
    samples: Vec<f32>,
) -> Result<Option<WakeWordDetection>, String> {
    let voice_state = state.read().await;
    voice_state.wake_word_detector.process_audio(&samples)
}

// Speech-to-Text Commands

#[tauri::command]
pub async fn voice_start_recognition(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.stt_engine.start_recognition()
}

#[tauri::command]
pub async fn voice_stop_recognition(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.stt_engine.stop_recognition()
}

#[tauri::command]
pub async fn voice_get_stt_config(
    state: State<'_, SharedVoiceState>,
) -> Result<SpeechToTextConfig, String> {
    let voice_state = state.read().await;
    voice_state.stt_engine.get_config()
}

#[tauri::command]
pub async fn voice_update_stt_config(
    state: State<'_, SharedVoiceState>,
    config: SpeechToTextConfig,
) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.stt_engine.update_config(config)
}

#[tauri::command]
pub async fn voice_get_supported_languages() -> Result<Vec<LanguageOption>, String> {
    Ok(SpeechToTextEngine::get_supported_languages())
}

#[tauri::command]
pub async fn voice_set_stt_language(
    state: State<'_, SharedVoiceState>,
    language_code: String,
) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.stt_engine.set_language(language_code)
}

#[tauri::command]
pub async fn voice_simulate_transcription(
    state: State<'_, SharedVoiceState>,
    transcript: String,
    confidence: f32,
) -> Result<SpeechRecognitionResult, String> {
    let voice_state = state.read().await;
    Ok(voice_state
        .stt_engine
        .simulate_transcription(&transcript, confidence))
}

// Text-to-Speech Commands

#[tauri::command]
pub async fn voice_speak(state: State<'_, SharedVoiceState>, text: String) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.speak(text)
}

#[tauri::command]
pub async fn voice_stop_speaking(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.stop()
}

#[tauri::command]
pub async fn voice_pause_speaking(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.pause()
}

#[tauri::command]
pub async fn voice_resume_speaking(state: State<'_, SharedVoiceState>) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.resume()
}

#[tauri::command]
pub async fn voice_get_tts_status(
    state: State<'_, SharedVoiceState>,
) -> Result<SpeechSynthesisStatus, String> {
    let voice_state = state.read().await;
    Ok(voice_state.tts_engine.get_status())
}

#[tauri::command]
pub async fn voice_get_tts_config(
    state: State<'_, SharedVoiceState>,
) -> Result<TextToSpeechConfig, String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.get_config()
}

#[tauri::command]
pub async fn voice_update_tts_config(
    state: State<'_, SharedVoiceState>,
    config: TextToSpeechConfig,
) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.update_config(config)
}

#[tauri::command]
pub async fn voice_get_available_voices() -> Result<Vec<Voice>, String> {
    Ok(TextToSpeechEngine::get_available_voices())
}

#[tauri::command]
pub async fn voice_set_voice(
    state: State<'_, SharedVoiceState>,
    voice_id: String,
) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.set_voice(voice_id)
}

#[tauri::command]
pub async fn voice_set_rate(state: State<'_, SharedVoiceState>, rate: f32) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.set_rate(rate)
}

#[tauri::command]
pub async fn voice_set_pitch(state: State<'_, SharedVoiceState>, pitch: f32) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.set_pitch(pitch)
}

#[tauri::command]
pub async fn voice_set_volume(
    state: State<'_, SharedVoiceState>,
    volume: f32,
) -> Result<(), String> {
    let voice_state = state.read().await;
    voice_state.tts_engine.set_volume(volume)
}
