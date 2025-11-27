use std::sync::{Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use data_encoding::BASE32;
use hmac::{Hmac, Mac};
use qrcodegen::{QrCode, QrCodeEcc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use tauri::State;

use crate::security::keystore::{Keystore, KeystoreError};

type HmacSha1 = Hmac<Sha1>;

const TOTP_SECRET_KEY: &str = "totp-secret";
const TOTP_CONFIG_KEY: &str = "totp-config";
const TOTP_ISSUER: &str = "EclipseMarketPro";
const TOTP_DIGITS: u32 = 6;
const TOTP_STEP: u64 = 30;
const BACKUP_CODE_COUNT: usize = 10;
const BACKUP_CODE_LENGTH: usize = 10;

#[derive(Debug, thiserror::Error)]
pub enum TwoFactorError {
    #[error("2FA not enrolled")]
    NotEnrolled,
    #[error("2FA already enrolled")]
    AlreadyEnrolled,
    #[error("invalid TOTP code")]
    InvalidCode,
    #[error("invalid backup code")]
    InvalidBackupCode,
    #[error("keystore error: {0}")]
    Keystore(#[from] KeystoreError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("qr generation error")]
    QrGeneration,
    #[error("internal error")]
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorConfig {
    pub enrolled: bool,
    pub backup_code_hashes: Vec<String>,
    pub used_backup_code_hashes: Vec<String>,
    pub enrolled_at: Option<DateTime<Utc>>,
}

impl Default for TwoFactorConfig {
    fn default() -> Self {
        Self {
            enrolled: false,
            backup_code_hashes: Vec::new(),
            used_backup_code_hashes: Vec::new(),
            enrolled_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorEnrollment {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
    pub manual_entry_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorStatus {
    pub enrolled: bool,
    pub enrolled_at: Option<DateTime<Utc>>,
    pub backup_codes_remaining: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateBackupCodesResponse {
    pub backup_codes: Vec<String>,
}

pub struct TwoFactorManager {
    config: Mutex<TwoFactorConfig>,
}

impl TwoFactorManager {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(TwoFactorConfig::default()),
        }
    }

    pub fn hydrate(&self, keystore: &Keystore) -> Result<(), TwoFactorError> {
        match keystore.retrieve_secret(TOTP_CONFIG_KEY) {
            Ok(bytes) => {
                let config: TwoFactorConfig = serde_json::from_slice(bytes.as_ref())?;
                let mut guard = self.lock_config()?;
                *guard = config;
            }
            Err(KeystoreError::NotFound) => {}
            Err(err) => return Err(TwoFactorError::Keystore(err)),
        }
        Ok(())
    }

    pub fn enroll(
        &self,
        user_id: &str,
        keystore: &Keystore,
    ) -> Result<TwoFactorEnrollment, TwoFactorError> {
        {
            let config = self.lock_config()?;
            if config.enrolled {
                return Err(TwoFactorError::AlreadyEnrolled);
            }
        }

        let secret = Self::generate_secret();
        let secret_bytes = secret.clone().into_bytes();
        keystore.store_secret(TOTP_SECRET_KEY, &secret_bytes)?;

        let backup_codes = Self::generate_backup_codes();
        let backup_hashes: Vec<String> = backup_codes.iter().map(|code| hash_code(code)).collect();

        let qr_code = Self::generate_qr_code(user_id, &secret)?;

        {
            let mut config = self.lock_config()?;
            config.enrolled = true;
            config.backup_code_hashes = backup_hashes;
            config.used_backup_code_hashes.clear();
            config.enrolled_at = Some(Utc::now());
            self.persist_config(keystore, &config)?;
        }

        Ok(TwoFactorEnrollment {
            secret: secret.clone(),
            qr_code,
            backup_codes,
            manual_entry_key: secret,
        })
    }

    pub fn status(&self) -> Result<TwoFactorStatus, TwoFactorError> {
        let config = self.lock_config()?;
        Ok(TwoFactorStatus {
            enrolled: config.enrolled,
            enrolled_at: config.enrolled_at,
            backup_codes_remaining: config.backup_code_hashes.len(),
        })
    }

    pub fn verify(&self, code: &str, keystore: &Keystore) -> Result<bool, TwoFactorError> {
        let trimmed = code.trim().to_uppercase();
        if trimmed.is_empty() {
            return Err(TwoFactorError::InvalidCode);
        }

        if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() == TOTP_DIGITS as usize {
            return self.verify_totp(&trimmed, keystore);
        }

        self.verify_backup_code(&trimmed, keystore)
    }

    pub fn disable(&self, keystore: &Keystore) -> Result<(), TwoFactorError> {
        {
            let mut config = self.lock_config()?;
            if !config.enrolled {
                return Err(TwoFactorError::NotEnrolled);
            }
            *config = TwoFactorConfig::default();
            self.persist_config(keystore, &config)?;
        }

        let _ = keystore.remove_secret(TOTP_SECRET_KEY);
        let _ = keystore.remove_secret(TOTP_CONFIG_KEY);
        Ok(())
    }

    pub fn regenerate_backup_codes(
        &self,
        keystore: &Keystore,
    ) -> Result<Vec<String>, TwoFactorError> {
        let mut config = self.lock_config()?;
        if !config.enrolled {
            return Err(TwoFactorError::NotEnrolled);
        }

        let new_codes = Self::generate_backup_codes();
        config.backup_code_hashes = new_codes.iter().map(|code| hash_code(code)).collect();
        config.used_backup_code_hashes.clear();
        self.persist_config(keystore, &config)?;

        Ok(new_codes)
    }

    fn verify_totp(&self, code: &str, keystore: &Keystore) -> Result<bool, TwoFactorError> {
        let config = self.lock_config()?;
        if !config.enrolled {
            return Err(TwoFactorError::NotEnrolled);
        }
        drop(config);

        let secret_bytes = keystore.retrieve_secret(TOTP_SECRET_KEY)?;
        let secret = BASE32
            .decode(secret_bytes.as_ref())
            .map_err(|_| TwoFactorError::Internal)?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| TwoFactorError::Internal)?
            .as_secs() as i64;

        let mut valid = false;
        for offset in -1..=1 {
            let timestamp = now + offset * TOTP_STEP as i64;
            if timestamp < 0 {
                continue;
            }
            let counter = (timestamp as u64) / TOTP_STEP;
            let value = generate_totp(&secret, counter)?;
            let formatted = format!("{:0width$}", value, width = TOTP_DIGITS as usize);
            if formatted == code {
                valid = true;
                break;
            }
        }

        Ok(valid)
    }

    fn verify_backup_code(&self, code: &str, keystore: &Keystore) -> Result<bool, TwoFactorError> {
        let mut config = self.lock_config()?;
        if !config.enrolled {
            return Err(TwoFactorError::NotEnrolled);
        }

        let hash = hash_code(code);
        if config.backup_code_hashes.contains(&hash) {
            config
                .backup_code_hashes
                .retain(|existing| existing != &hash);
            config.used_backup_code_hashes.push(hash);
            self.persist_config(keystore, &config)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn persist_config(
        &self,
        keystore: &Keystore,
        config: &TwoFactorConfig,
    ) -> Result<(), TwoFactorError> {
        let payload = serde_json::to_vec(config)?;
        keystore.store_secret(TOTP_CONFIG_KEY, &payload)?;
        Ok(())
    }

    fn lock_config(&self) -> Result<MutexGuard<'_, TwoFactorConfig>, TwoFactorError> {
        self.config.lock().map_err(|_| TwoFactorError::Internal)
    }

    fn generate_secret() -> String {
        let secret: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
        BASE32.encode(&secret)
    }

    fn generate_backup_codes() -> Vec<String> {
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();

        (0..BACKUP_CODE_COUNT)
            .map(|_| {
                (0..BACKUP_CODE_LENGTH)
                    .map(|_| chars[rand::random_range(0..chars.len())])
                    .collect()
            })
            .collect()
    }

    fn generate_qr_code(user_id: &str, secret: &str) -> Result<String, TwoFactorError> {
        let uri = format!(
            "otpauth://totp/{issuer}:{user}?secret={secret}&issuer={issuer}&digits={digits}&period={period}",
            issuer = TOTP_ISSUER,
            user = user_id,
            secret = secret,
            digits = TOTP_DIGITS,
            period = TOTP_STEP
        );

        let qr = QrCode::encode_text(&uri, QrCodeEcc::Medium)
            .map_err(|_| TwoFactorError::QrGeneration)?;

        let size = qr.size() as usize;
        let border = 4;
        let total_size = size + border * 2;

        let mut svg = String::new();
        svg.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"0 0 {} {}\" stroke=\"none\">",
            total_size, total_size
        ));
        svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>");
        svg.push_str("<path d=\"");

        for y in 0..size {
            for x in 0..size {
                if qr.get_module(x as i32, y as i32) {
                    let qx = x + border;
                    let qy = y + border;
                    svg.push_str(&format!("M{},{}h1v1h-1z", qx, qy));
                }
            }
        }

        svg.push_str("\" fill=\"#000000\"/></svg>");
        Ok(svg)
    }
}

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn generate_totp(secret: &[u8], counter: u64) -> Result<u32, TwoFactorError> {
    let counter_bytes = counter.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(secret).map_err(|_| TwoFactorError::Internal)?;
    mac.update(&counter_bytes);
    let result = mac.finalize().into_bytes();

    let offset = (result[19] & 0xf) as usize;
    let truncated_hash: u32 = ((result[offset] & 0x7f) as u32) << 24
        | (result[offset + 1] as u32) << 16
        | (result[offset + 2] as u32) << 8
        | result[offset + 3] as u32;

    let code = truncated_hash % 10u32.pow(TOTP_DIGITS);
    Ok(code)
}

#[tauri::command]
pub async fn two_factor_enroll(
    user_id: String,
    state: State<'_, TwoFactorManager>,
    keystore: State<'_, Keystore>,
) -> Result<TwoFactorEnrollment, String> {
    state
        .enroll(&user_id, keystore.inner())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn two_factor_verify(
    request: VerifyRequest,
    state: State<'_, TwoFactorManager>,
    keystore: State<'_, Keystore>,
) -> Result<bool, String> {
    state
        .verify(&request.code, keystore.inner())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn two_factor_disable(
    state: State<'_, TwoFactorManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    state.disable(keystore.inner()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn two_factor_status(
    state: State<'_, TwoFactorManager>,
) -> Result<TwoFactorStatus, String> {
    state.status().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn two_factor_regenerate_backup_codes(
    state: State<'_, TwoFactorManager>,
    keystore: State<'_, Keystore>,
) -> Result<RegenerateBackupCodesResponse, String> {
    let codes = state
        .regenerate_backup_codes(keystore.inner())
        .map_err(|e| e.to_string())?;
    Ok(RegenerateBackupCodesResponse {
        backup_codes: codes,
    })
}
