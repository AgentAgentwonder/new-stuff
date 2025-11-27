use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneStatus {
    pub granted: bool,
    pub reason: Option<String>,
    pub last_checked: Option<i64>,
}

impl Default for MicrophoneStatus {
    fn default() -> Self {
        Self {
            granted: false,
            reason: None,
            last_checked: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSessionSnapshot {
    pub session_id: Option<Uuid>,
    pub microphone_active: bool,
    pub permissions: MicrophoneStatus,
    pub stream_token: Option<String>,
}

#[derive(Debug)]
pub struct AudioContextManager {
    session_id: Arc<Mutex<Option<Uuid>>>,
    microphone_active: Arc<Mutex<bool>>,
    permissions: Arc<Mutex<MicrophoneStatus>>,
    stream_token: Arc<Mutex<Option<String>>>,
}

impl AudioContextManager {
    pub fn new() -> Self {
        Self {
            session_id: Arc::new(Mutex::new(None)),
            microphone_active: Arc::new(Mutex::new(false)),
            permissions: Arc::new(Mutex::new(MicrophoneStatus::default())),
            stream_token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn request_permissions(&self) -> Result<MicrophoneStatus, String> {
        let mut permissions = self.permissions.lock().map_err(|e| e.to_string())?;
        permissions.granted = true;
        permissions.reason = None;
        permissions.last_checked = Some(chrono::Utc::now().timestamp_millis());
        Ok(permissions.clone())
    }

    pub fn revoke_permissions(&self, reason: Option<String>) -> Result<(), String> {
        let mut permissions = self.permissions.lock().map_err(|e| e.to_string())?;
        permissions.granted = false;
        permissions.reason = reason;
        permissions.last_checked = Some(chrono::Utc::now().timestamp_millis());
        self.stop_microphone()?;
        Ok(())
    }

    pub fn start_microphone(&self) -> Result<AudioSessionSnapshot, String> {
        let mut permissions = self.permissions.lock().map_err(|e| e.to_string())?;
        if !permissions.granted {
            return Err("Microphone permission not granted".to_string());
        }

        let mut active = self.microphone_active.lock().map_err(|e| e.to_string())?;
        if *active {
            return Ok(self.snapshot()?);
        }

        let mut session_id = self.session_id.lock().map_err(|e| e.to_string())?;
        let mut stream_token = self.stream_token.lock().map_err(|e| e.to_string())?;

        *session_id = Some(Uuid::new_v4());
        *stream_token = Some(Uuid::new_v4().to_string());
        *active = true;

        Ok(self.snapshot()?)
    }

    pub fn stop_microphone(&self) -> Result<(), String> {
        let mut active = self.microphone_active.lock().map_err(|e| e.to_string())?;
        *active = false;

        let mut session_id = self.session_id.lock().map_err(|e| e.to_string())?;
        let mut stream_token = self.stream_token.lock().map_err(|e| e.to_string())?;

        *session_id = None;
        *stream_token = None;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<AudioSessionSnapshot, String> {
        let session_id = self.session_id.lock().map_err(|e| e.to_string())?;
        let microphone_active = self.microphone_active.lock().map_err(|e| e.to_string())?;
        let permissions = self.permissions.lock().map_err(|e| e.to_string())?;
        let stream_token = self.stream_token.lock().map_err(|e| e.to_string())?;

        Ok(AudioSessionSnapshot {
            session_id: *session_id,
            microphone_active: *microphone_active,
            permissions: permissions.clone(),
            stream_token: stream_token.clone(),
        })
    }
}

pub type SharedAudioContext = Arc<AudioContextManager>;

impl Default for AudioContextManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions_flow() {
        let manager = AudioContextManager::new();
        let status = manager.request_permissions().unwrap();
        assert!(status.granted);
        assert!(status.last_checked.is_some());

        manager
            .revoke_permissions(Some("user revoked".to_string()))
            .unwrap();
        let snapshot = manager.snapshot().unwrap();
        assert!(!snapshot.permissions.granted);
        assert_eq!(
            snapshot.permissions.reason,
            Some("user revoked".to_string())
        );
    }

    #[test]
    fn test_microphone_session() {
        let manager = AudioContextManager::new();
        assert!(manager.start_microphone().is_err());

        manager.request_permissions().unwrap();
        let snapshot = manager.start_microphone().unwrap();
        assert!(snapshot.microphone_active);
        assert!(snapshot.session_id.is_some());
        assert!(snapshot.stream_token.is_some());

        manager.stop_microphone().unwrap();
        let snapshot = manager.snapshot().unwrap();
        assert!(!snapshot.microphone_active);
        assert!(snapshot.session_id.is_none());
    }
}
