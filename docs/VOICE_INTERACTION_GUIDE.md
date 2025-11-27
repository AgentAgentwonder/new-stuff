# Voice Interaction Guide

This document explains how Eclipse Market Pro implements the cross-platform voice interaction pipeline, covering wake word detection, multilingual speech-to-text, text-to-speech synthesis, microphone permissions, and privacy controls.

## Overview

The voice subsystem adds a conversational layer to Eclipse Market Pro while preserving privacy and offline availability. It is composed of:

- **Wake Word Detection (`Hey Eclipse`)** – lightweight energy-based detection with optional sensitivity controls. The backend is designed for integration with Porcupine or Vosk, but the default implementation provides a deterministic, privacy-focused detector that works offline.
- **Speech-to-Text (STT)** – multilingual recognition with Web Speech API fallback and an extensible Tauri backend for native engines. Users can choose languages, toggle continuous mode, and enable interim results.
- **Text-to-Speech (TTS)** – configurable voice synthesis with adjustable rate, pitch, volume, and language, providing confirmations for critical trading actions.
- **Audio Context Management** – the React hook `useVoiceInteraction` manages secure microphone sessions, audio contexts, and synchronization with backend commands.

## Getting Started

1. Open **Settings → Voice Interaction**.
2. Request microphone access. The permission state is stored locally and managed by the Tauri backend. Revoking permission immediately halts all voice activity.
3. Customize the wake word, sensitivity, or disable detection entirely when not needed.
4. Select your preferred speech recognition language and speech synthesis voice. The UI surfaces all supported locales reported by the backend.
5. Toggle confirmation prompts to require audible acknowledgements before executing high-impact actions (e.g., placing trades).
6. Enable privacy mode to suspend wake word listening, STT, and TTS instantly. Privacy mode persists between sessions until manually disabled.

## Microphone Permissions

- Permissions are requested through the `voice_request_permissions` command. The backend tracks consent timestamps, tokens, and active sessions via the `AudioContextManager`.
- When permission is revoked (`voice_revoke_permissions`), the backend clears open sessions, resets stream tokens, and ensures no audio buffers are retained.
- The frontend hook exposes `requestMicrophonePermission` and `revokeMicrophonePermission` helpers that update both UI state and Tauri state.
- Privacy mode and manual revocation both tear down the microphone session, suspend the Web Audio context, and disable wake word polling.

## Wake Word Training & Sensitivity

- Settings allow changing the wake phrase (default: `Hey Eclipse`) and adjusting sensitivity (0–100%).
- Higher sensitivity increases responsiveness but may produce false positives, especially in noisy environments. The default value (50%) balances accuracy with resilience.
- Timeout determines how long (in milliseconds) the detector waits for accompanying speech after a wake word is heard.
- The backend exposes `voice_update_wake_word_config` and `voice_process_audio_for_wake_word` for future integration with advanced training datasets (e.g., Porcupine custom wake models). Tests simulate wake word detection by passing synthetic audio samples to this command.

## Speech Recognition Workflow

1. `voice_start_microphone` secures microphone access and produces a session token.
2. `voice_start_wake_word` begins offline detection when enabled.
3. `voice_start_recognition` activates the multilingual STT engine.
4. `voice_simulate_transcription` is provided for deterministic integration tests and developer tooling.
5. `voice_stop_recognition`, `voice_stop_wake_word`, and `voice_stop_microphone` must be called (in that order) to release resources.

When running in the browser (without Tauri), the hook automatically falls back to the Web Speech API if available, ensuring feature parity during development.

## Text-to-Speech Workflow

- `voice_speak` queues text for synthesis. Voices and languages are reported via `voice_get_available_voices`.
- Rate, pitch, and volume can be adjusted between 0.1–10.0 (rate), 0.0–2.0 (pitch), and 0.0–1.0 (volume).
- `voice_stop_speaking`, `voice_pause_speaking`, and `voice_resume_speaking` provide full control of audio output.
- Confirmation prompts leverage TTS to read back actions before execution.

## React Hook (`useVoiceInteraction`)

The hook orchestrates all frontend voice responsibilities:

- Synchronizes Zustand-based voice settings with backend commands.
- Manages Web Audio contexts, microphone permissions, and cleanup.
- Provides helper methods (`startListening`, `stopListening`, `speak`, `simulateTranscription`, `togglePrivacyMode`) that respect privacy mode and confirmation settings.
- Automatically falls back to Web Speech API / SpeechSynthesis when running outside the Tauri environment.

## Testing

- **Frontend**: `src/__tests__/voice.test.ts` validates command wiring, configuration updates, and the end-to-end voice pipeline sequence (permission → wake word → STT → TTS → teardown).
- **Backend**: Unit tests inside `src-tauri/src/voice/*.rs` cover wake word detection, audio session management, STT, and TTS engines.

## Security & Privacy Considerations

- No audio is transmitted externally. Wake word processing and STT defaults remain offline unless a future plugin is enabled.
- Stream tokens and session IDs are rotated on each microphone start to prevent reuse.
- Privacy mode disables automatic activation, wake word detection, and TTS output simultaneously.

## Extending the Pipeline

- Integrate Porcupine, Vosk, or Whisper backends by replacing the engine implementations in `src-tauri/src/voice/`.
- Emit real-time events (`voice://wake-detected`, `voice://transcript`) from the commands when integrating live audio streams.
- Add additional voices by extending the `TextToSpeechEngine::get_available_voices` method.

For further assistance, consult `src/hooks/useVoiceInteraction.ts` and the backend module under `src-tauri/src/voice/`.
