use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

use crate::security::keystore::{Keystore, KeystoreError};

const JWT_SECRET_KEY: &str = "jwt-signing-key";
const SESSION_STATE_KEY: &str = "session-state";
const DEFAULT_SESSION_TIMEOUT_MINUTES: u64 = 15;
const SESSION_WARNING_SECONDS: u64 = 60;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session expired")]
    Expired,
    #[error("invalid token")]
    InvalidToken,
    #[error("no active session")]
    NoSession,
    #[error("keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionClaims {
    pub sub: String,
    pub session_id: String,
    pub exp: u64,
    pub iat: u64,
    pub nbf: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub token: String,
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub timeout_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatus {
    pub active: bool,
    pub session_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub timeout_minutes: u64,
    pub warning_threshold_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub timeout_minutes: Option<u64>,
}

pub struct SessionManager {
    current_session: Mutex<Option<SessionState>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            current_session: Mutex::new(None),
        }
    }

    pub fn hydrate(&self, keystore: &Keystore) -> Result<(), SessionError> {
        match keystore.retrieve_secret(SESSION_STATE_KEY) {
            Ok(payload) => {
                let session: SessionState = serde_json::from_slice(payload.as_ref())?;
                if Self::is_session_valid(&session) {
                    let mut guard = self.lock_session()?;
                    *guard = Some(session);
                } else {
                    let _ = keystore.remove_secret(SESSION_STATE_KEY);
                }
            }
            Err(KeystoreError::NotFound) => {}
            Err(err) => return Err(SessionError::Keystore(err)),
        }
        Ok(())
    }

    pub fn create_session(
        &self,
        user_id: String,
        timeout_minutes: Option<u64>,
        keystore: &Keystore,
    ) -> Result<SessionState, SessionError> {
        let timeout = timeout_minutes.unwrap_or(DEFAULT_SESSION_TIMEOUT_MINUTES);
        let now = Utc::now();
        let session_id = Uuid::new_v4().to_string();

        let iat = now.timestamp() as u64;
        let exp = (now + chrono::Duration::minutes(timeout as i64)).timestamp() as u64;

        let claims = SessionClaims {
            sub: user_id,
            session_id: session_id.clone(),
            exp,
            iat,
            nbf: iat,
        };

        let secret = self.get_jwt_secret(keystore)?;
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )?;

        let session = SessionState {
            token,
            session_id,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(timeout as i64),
            last_activity: now,
            timeout_minutes: timeout,
        };

        self.persist_session(keystore, &session)?;
        let mut guard = self.lock_session()?;
        *guard = Some(session.clone());

        Ok(session)
    }

    pub fn renew_session(&self, keystore: &Keystore) -> Result<SessionState, SessionError> {
        let mut guard = self.lock_session()?;
        let current = guard.as_mut().ok_or(SessionError::NoSession)?;

        if !Self::is_session_valid(current) {
            return Err(SessionError::Expired);
        }

        let now = Utc::now();
        let new_expiry = now + chrono::Duration::minutes(current.timeout_minutes as i64);

        let secret = self.get_jwt_secret(keystore)?;
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        let token_data = decode::<SessionClaims>(&current.token, &decoding_key, &validation)?;

        let new_claims = SessionClaims {
            sub: token_data.claims.sub,
            session_id: token_data.claims.session_id,
            exp: new_expiry.timestamp() as u64,
            iat: token_data.claims.iat,
            nbf: token_data.claims.nbf,
        };

        let new_token = encode(
            &Header::new(Algorithm::HS256),
            &new_claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )?;

        current.token = new_token;
        current.expires_at = new_expiry;
        current.last_activity = now;

        self.persist_session(keystore, current)?;
        Ok(current.clone())
    }

    pub fn end_session(&self, keystore: &Keystore) -> Result<(), SessionError> {
        let mut guard = self.lock_session()?;
        *guard = None;
        let _ = keystore.remove_secret(SESSION_STATE_KEY);
        Ok(())
    }

    pub fn get_status(&self) -> Result<SessionStatus, SessionError> {
        let guard = self.lock_session()?;
        if let Some(session) = guard.as_ref() {
            let active = Self::is_session_valid(session);
            Ok(SessionStatus {
                active,
                session_id: Some(session.session_id.clone()),
                expires_at: Some(session.expires_at),
                last_activity: Some(session.last_activity),
                timeout_minutes: session.timeout_minutes,
                warning_threshold_seconds: SESSION_WARNING_SECONDS,
            })
        } else {
            Ok(SessionStatus {
                active: false,
                session_id: None,
                expires_at: None,
                last_activity: None,
                timeout_minutes: DEFAULT_SESSION_TIMEOUT_MINUTES,
                warning_threshold_seconds: SESSION_WARNING_SECONDS,
            })
        }
    }

    pub fn verify_session(&self) -> Result<bool, SessionError> {
        let guard = self.lock_session()?;
        if let Some(session) = guard.as_ref() {
            Ok(Self::is_session_valid(session))
        } else {
            Ok(false)
        }
    }

    pub fn update_activity(&self, keystore: &Keystore) -> Result<(), SessionError> {
        let mut guard = self.lock_session()?;
        if let Some(session) = guard.as_mut() {
            session.last_activity = Utc::now();
            self.persist_session(keystore, session)?;
        }
        Ok(())
    }

    pub fn configure_timeout(
        &self,
        timeout_minutes: u64,
        keystore: &Keystore,
    ) -> Result<(), SessionError> {
        let mut guard = self.lock_session()?;
        if let Some(session) = guard.as_mut() {
            session.timeout_minutes = timeout_minutes;
            let now = Utc::now();
            session.expires_at = now + chrono::Duration::minutes(timeout_minutes as i64);
            self.persist_session(keystore, session)?;
        }
        Ok(())
    }

    fn get_jwt_secret(&self, keystore: &Keystore) -> Result<Zeroizing<Vec<u8>>, SessionError> {
        match keystore.retrieve_secret(JWT_SECRET_KEY) {
            Ok(secret) => Ok(secret),
            Err(KeystoreError::NotFound) => {
                let mut secret = Zeroizing::new(vec![0u8; 64]);
                rand_core::OsRng.fill_bytes(secret.as_mut());
                keystore.store_secret(JWT_SECRET_KEY, secret.as_ref())?;
                Ok(secret)
            }
            Err(err) => Err(SessionError::Keystore(err)),
        }
    }

    fn persist_session(
        &self,
        keystore: &Keystore,
        session: &SessionState,
    ) -> Result<(), SessionError> {
        let payload = serde_json::to_vec(session)?;
        keystore.store_secret(SESSION_STATE_KEY, &payload)?;
        Ok(())
    }

    fn is_session_valid(session: &SessionState) -> bool {
        let now = Utc::now();
        now < session.expires_at
    }

    fn lock_session(&self) -> Result<MutexGuard<'_, Option<SessionState>>, SessionError> {
        self.current_session
            .lock()
            .map_err(|_| SessionError::Internal)
    }
}

#[tauri::command]
pub async fn session_create(
    request: CreateSessionRequest,
    state: State<'_, SessionManager>,
    keystore: State<'_, Keystore>,
) -> Result<SessionState, String> {
    state
        .create_session(request.user_id, request.timeout_minutes, keystore.inner())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn session_renew(
    state: State<'_, SessionManager>,
    keystore: State<'_, Keystore>,
) -> Result<SessionState, String> {
    state
        .renew_session(keystore.inner())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn session_end(
    state: State<'_, SessionManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    state
        .end_session(keystore.inner())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn session_status(state: State<'_, SessionManager>) -> Result<SessionStatus, String> {
    state.get_status().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn session_verify(state: State<'_, SessionManager>) -> Result<bool, String> {
    state.verify_session().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn session_update_activity(
    state: State<'_, SessionManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    state
        .update_activity(keystore.inner())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn session_configure_timeout(
    timeout_minutes: u64,
    state: State<'_, SessionManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    state
        .configure_timeout(timeout_minutes, keystore.inner())
        .map_err(|e| e.to_string())
}
