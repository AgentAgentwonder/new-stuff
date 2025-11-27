use std::str;

#[cfg(target_os = "windows")]
use std::future::IntoFuture;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::RngCore;
use base64::{engine::general_purpose, Engine as _};
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::Zeroize;

const KEYRING_SERVICE: &str = "eclipse-market-pro";
const BIOMETRIC_TOKEN_KEY: &str = "biometric-token";
const FALLBACK_HASH_KEY: &str = "biometric-fallback";
const PLATFORM_REASON: &str = "Unlock Eclipse Market Pro";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BiometricPlatform {
    WindowsHello,
    TouchId,
    PasswordOnly,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BiometricStatus {
    pub available: bool,
    pub enrolled: bool,
    pub fallback_configured: bool,
    pub platform: BiometricPlatform,
}

#[derive(Debug, Error)]
pub enum BiometricError {
    #[error("Biometric hardware unavailable: {0}")]
    Unavailable(String),
    #[error("Biometric request denied: {0}")]
    Denied(String),
    #[error("Biometric operation failed: {0}")]
    Failed(String),
    #[error("Biometric storage failure: {0}")]
    Storage(String),
    #[error("Fallback password mismatch")]
    InvalidFallback,
    #[error("Biometrics not enrolled")]
    NotEnrolled,
}

impl From<BiometricError> for String {
    fn from(value: BiometricError) -> Self {
        value.to_string()
    }
}

pub fn current_status() -> Result<BiometricStatus, BiometricError> {
    let platform = platform_kind();
    let available = platform_is_available()?;
    let enrolled = read_entry(BIOMETRIC_TOKEN_KEY)?.is_some();
    let fallback_configured = read_entry(FALLBACK_HASH_KEY)?.is_some();

    Ok(BiometricStatus {
        available,
        enrolled,
        fallback_configured,
        platform,
    })
}

pub async fn enroll(fallback_password: String) -> Result<BiometricStatus, BiometricError> {
    let available = platform_is_available()?;
    if !available {
        return Err(BiometricError::Unavailable(
            "Biometric hardware not available on this platform".into(),
        ));
    }

    platform_require_presence().await?;

    store_token()?;
    store_fallback_hash(fallback_password)?;

    current_status()
}

pub async fn verify() -> Result<(), BiometricError> {
    if read_entry(BIOMETRIC_TOKEN_KEY)?.is_none() {
        return Err(BiometricError::NotEnrolled);
    }
    platform_require_presence().await
}

pub fn disable() -> Result<BiometricStatus, BiometricError> {
    delete_entry(BIOMETRIC_TOKEN_KEY)?;
    delete_entry(FALLBACK_HASH_KEY)?;
    current_status()
}

pub fn verify_fallback(password: String) -> Result<(), BiometricError> {
    let mut password = password;
    let stored = read_entry(FALLBACK_HASH_KEY)?.ok_or(BiometricError::NotEnrolled)?;

    let result = verify_fallback_internal(password.as_bytes(), stored.as_str());
    password.zeroize();

    if result {
        Ok(())
    } else {
        Err(BiometricError::InvalidFallback)
    }
}

fn store_token() -> Result<(), BiometricError> {
    let mut token_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    let token = general_purpose::STANDARD.encode(token_bytes);
    write_entry(BIOMETRIC_TOKEN_KEY, &token)
}

fn store_fallback_hash(password: String) -> Result<(), BiometricError> {
    let mut password = password;
    let hash = hash_password(password.as_bytes())?;
    password.zeroize();
    write_entry(FALLBACK_HASH_KEY, &hash)
}

fn hash_password(password: &[u8]) -> Result<String, BiometricError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(password, &salt)
        .map_err(|e| BiometricError::Failed(format!("unable to hash fallback password: {e}")))?;

    Ok(hash.to_string())
}

fn verify_fallback_internal(password: &[u8], hash: &str) -> bool {
    PasswordHash::new(hash)
        .ok()
        .and_then(|parsed| Argon2::default().verify_password(password, &parsed).ok())
        .is_some()
}

fn read_entry(key: &str) -> Result<Option<String>, BiometricError> {
    let entry = keyring_entry(key)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(err) => Err(BiometricError::Storage(err.to_string())),
    }
}

fn write_entry(key: &str, value: &str) -> Result<(), BiometricError> {
    let entry = keyring_entry(key)?;
    entry
        .set_password(value)
        .map_err(|err| BiometricError::Storage(err.to_string()))
}

fn delete_entry(key: &str) -> Result<(), BiometricError> {
    let entry = keyring_entry(key)?;
    match entry.delete_password() {
        Ok(_) => Ok(()),
        Err(KeyringError::NoEntry) => Ok(()),
        Err(err) => Err(BiometricError::Storage(err.to_string())),
    }
}

fn keyring_entry(key: &str) -> Result<Entry, BiometricError> {
    Entry::new(KEYRING_SERVICE, key).map_err(|err| BiometricError::Storage(err.to_string()))
}

fn platform_kind() -> BiometricPlatform {
    #[cfg(target_os = "windows")]
    {
        return BiometricPlatform::WindowsHello;
    }

    #[cfg(target_os = "macos")]
    {
        return BiometricPlatform::TouchId;
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        BiometricPlatform::PasswordOnly
    }
}

fn platform_is_available() -> Result<bool, BiometricError> {
    #[cfg(target_os = "windows")]
    {
        return windows_available();
    }

    #[cfg(target_os = "macos")]
    {
        return macos_available();
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok(false)
    }
}

async fn platform_require_presence() -> Result<(), BiometricError> {
    #[cfg(target_os = "windows")]
    {
        return windows_verify().await;
    }

    #[cfg(target_os = "macos")]
    {
        return macos_verify().await;
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(BiometricError::Unavailable(
            "Biometric authentication is not supported on this platform".into(),
        ))
    }
}

#[cfg(target_os = "windows")]
fn windows_available() -> Result<bool, BiometricError> {
    use windows::Security::Credentials::UI::{
        UserConsentVerifier, UserConsentVerifierAvailability,
    };

    let async_op = UserConsentVerifier::CheckAvailabilityAsync()
        .map_err(|err| BiometricError::Failed(err.to_string()))?;

    // Use blocking get() method instead of async await
    let availability = async_op
        .get()
        .map_err(|err| BiometricError::Failed(err.to_string()))?;
    
    Ok(matches!(
        availability,
        UserConsentVerifierAvailability::Available
    ))
}

#[cfg(target_os = "windows")]
async fn windows_verify() -> Result<(), BiometricError> {
    use windows::{
        core::HSTRING,
        Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier},
    };

    let async_op = UserConsentVerifier::RequestVerificationAsync(&HSTRING::from(PLATFORM_REASON))
        .map_err(|err| BiometricError::Failed(err.to_string()))?;

    // Use blocking get() method instead of async await
    let result = async_op
        .get()
        .map_err(|err| BiometricError::Failed(err.to_string()))?;

    match result {
        UserConsentVerificationResult::Verified => Ok(()),
        UserConsentVerificationResult::DeviceBusy => Err(BiometricError::Failed(
            "Windows Hello device is busy".into(),
        )),
        UserConsentVerificationResult::DeviceNotPresent => Err(BiometricError::Unavailable(
            "No Windows Hello compatible device found".into(),
        )),
        UserConsentVerificationResult::DisabledByPolicy => Err(BiometricError::Unavailable(
            "Windows Hello is disabled by policy".into(),
        )),
        // Updated enum variants for windows crate 0.58
        UserConsentVerificationResult::NotConfiguredForUser => Err(BiometricError::Unavailable(
            "Windows Hello is not configured for this user".into(),
        )),
        UserConsentVerificationResult::Canceled => Err(BiometricError::Denied(
            "Biometric verification was canceled".into(),
        )),
        UserConsentVerificationResult::RetriesExhausted => Err(BiometricError::Failed(
            "Too many failed biometric attempts".into(),
        )),
        _ => Err(BiometricError::Failed(
            "Unhandled Windows Hello state".into(),
        )),
    }
}

#[cfg(target_os = "macos")]
fn macos_available() -> Result<bool, BiometricError> {
    use security_framework::os::macos::keychain::SecKeychain;

    match SecKeychain::default() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(target_os = "macos")]
async fn macos_verify() -> Result<(), BiometricError> {
    Ok(())
}
