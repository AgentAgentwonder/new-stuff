use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm,
};
use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as BASE64_ENGINE, Engine};
use chrono::{DateTime, Utc};
use keyring::Entry;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "EclipseMarketPro";
const MASTER_KEY_ID: &str = "keystore-master";
const KEYSTORE_FILE: &str = "keystore.json";
const KEYSTORE_VERSION: u8 = 1;
const ARGON2_M_COST: u32 = 19_456;
const ARGON2_T_COST: u32 = 2;
const ARGON2_P_COST: u32 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("encryption error")]
    Encryption,
    #[error("decryption error")]
    Decryption,
    #[error("secret not found")]
    NotFound,
    #[error("internal keystore error")]
    Internal,
    #[error("lock error")]
    LockError,
    #[error("serialization error")]
    SerializationError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSecret {
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeystoreDocument {
    pub version: u8,
    pub secrets: HashMap<String, StoredSecret>,
    pub exported_at: Option<DateTime<Utc>>,
}

impl Default for KeystoreDocument {
    fn default() -> Self {
        Self {
            version: KEYSTORE_VERSION,
            secrets: HashMap::new(),
            exported_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeystoreBackup {
    pub version: u8,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
    pub created_at: DateTime<Utc>,
}

pub struct Keystore {
    path: PathBuf,
    document: Mutex<KeystoreDocument>,
}

impl Keystore {
    pub fn initialize(app: &AppHandle) -> Result<Self, KeystoreError> {
        let path = keystore_path(app)?;
        let document = if path.exists() {
            let data = fs::read_to_string(&path)?;
            serde_json::from_str::<KeystoreDocument>(&data)?
        } else {
            KeystoreDocument::default()
        };

        Ok(Self {
            path,
            document: Mutex::new(document),
        })
    }

    pub fn store_secret(&self, key: &str, secret: &[u8]) -> Result<(), KeystoreError> {
        let mut guard = self.lock_document()?;
        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);

        let master_key = Self::master_key()?;
        let derived_key = derive_key(master_key.as_ref(), &salt)?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(derived_key.as_ref()));

        let ciphertext = cipher
            .encrypt(GenericArray::from_slice(&nonce), secret)
            .map_err(|_| KeystoreError::Encryption)?;

        let now = Utc::now();
        let created_at = {
            guard
                .secrets
                .get(key)
                .map(|existing| existing.created_at)
                .unwrap_or(now)
        };

        guard.secrets.insert(
            key.to_string(),
            StoredSecret {
                salt: BASE64_ENGINE.encode(salt),
                nonce: BASE64_ENGINE.encode(nonce),
                ciphertext: BASE64_ENGINE.encode(ciphertext),
                created_at,
                updated_at: now,
            },
        );

        persist_document(&self.path, &guard)
    }

    pub fn retrieve_secret(&self, key: &str) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        let guard = self.lock_document()?;
        let entry = guard.secrets.get(key).ok_or(KeystoreError::NotFound)?;

        let salt = BASE64_ENGINE
            .decode(entry.salt.as_bytes())
            .map_err(|_| KeystoreError::Decryption)?;
        let nonce = BASE64_ENGINE
            .decode(entry.nonce.as_bytes())
            .map_err(|_| KeystoreError::Decryption)?;
        let ciphertext = BASE64_ENGINE
            .decode(entry.ciphertext.as_bytes())
            .map_err(|_| KeystoreError::Decryption)?;

        let master_key = Self::master_key()?;
        let derived_key = derive_key(master_key.as_ref(), &salt)?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(derived_key.as_ref()));

        let plaintext = cipher
            .decrypt(GenericArray::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|_| KeystoreError::Decryption)?;

        Ok(Zeroizing::new(plaintext))
    }

    pub fn remove_secret(&self, key: &str) -> Result<(), KeystoreError> {
        let mut guard = self.lock_document()?;
        if guard.secrets.remove(key).is_some() {
            persist_document(&self.path, &guard)?;
        }
        Ok(())
    }

    pub fn export_backup(&self, password: &str) -> Result<KeystoreBackup, KeystoreError> {
        let guard = self.lock_document()?;
        let mut document = guard.clone();
        document.exported_at = Some(Utc::now());
        let serialized = serde_json::to_vec(&document)?;

        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce);

        let derived_key = derive_key(password.as_bytes(), &salt)?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(derived_key.as_ref()));

        let ciphertext = cipher
            .encrypt(GenericArray::from_slice(&nonce), serialized.as_ref())
            .map_err(|_| KeystoreError::Encryption)?;

        Ok(KeystoreBackup {
            version: document.version,
            salt: BASE64_ENGINE.encode(salt),
            nonce: BASE64_ENGINE.encode(nonce),
            ciphertext: BASE64_ENGINE.encode(ciphertext),
            created_at: document.exported_at.unwrap_or_else(Utc::now),
        })
    }

    pub fn import_backup(
        &self,
        password: &str,
        backup: KeystoreBackup,
    ) -> Result<(), KeystoreError> {
        if backup.version != KEYSTORE_VERSION {
            return Err(KeystoreError::Decryption);
        }

        let salt = BASE64_ENGINE
            .decode(backup.salt.as_bytes())
            .map_err(|_| KeystoreError::Decryption)?;
        let nonce = BASE64_ENGINE
            .decode(backup.nonce.as_bytes())
            .map_err(|_| KeystoreError::Decryption)?;
        let ciphertext = BASE64_ENGINE
            .decode(backup.ciphertext.as_bytes())
            .map_err(|_| KeystoreError::Decryption)?;

        let derived_key = derive_key(password.as_bytes(), &salt)?;
        let cipher = Aes256Gcm::new(GenericArray::from_slice(derived_key.as_ref()));

        let plaintext = cipher
            .decrypt(GenericArray::from_slice(&nonce), ciphertext.as_ref())
            .map_err(|_| KeystoreError::Decryption)?;

        let mut document: KeystoreDocument = serde_json::from_slice(&plaintext)?;
        document.exported_at = None;

        {
            let mut guard = self.lock_document()?;
            guard.secrets = document.secrets;
            persist_document(&self.path, &guard)?;
        }

        Ok(())
    }

    pub fn rotate_master_key(&self) -> Result<(), KeystoreError> {
        let old_master_key = Self::master_key()?;

        let mut new_key = Zeroizing::new(vec![0u8; 32]);
        OsRng.fill_bytes(&mut new_key);
        let new_key_slice: &[u8] = new_key.as_ref();
        let new_key_b64 = BASE64_ENGINE.encode(new_key_slice);

        let mut guard = self.lock_document()?;
        let mut updated_entries = HashMap::new();

        for (name, entry) in guard.secrets.iter() {
            let salt = BASE64_ENGINE
                .decode(entry.salt.as_bytes())
                .map_err(|_| KeystoreError::Decryption)?;
            let nonce = BASE64_ENGINE
                .decode(entry.nonce.as_bytes())
                .map_err(|_| KeystoreError::Decryption)?;
            let ciphertext = BASE64_ENGINE
                .decode(entry.ciphertext.as_bytes())
                .map_err(|_| KeystoreError::Decryption)?;

            let old_key = derive_key(old_master_key.as_ref(), &salt)?;
            let cipher = Aes256Gcm::new(GenericArray::from_slice(old_key.as_ref()));
            let plaintext = cipher
                .decrypt(GenericArray::from_slice(&nonce), ciphertext.as_ref())
                .map_err(|_| KeystoreError::Decryption)?;

            let mut new_salt = [0u8; SALT_LEN];
            let mut new_nonce = [0u8; NONCE_LEN];
            OsRng.fill_bytes(&mut new_salt);
            OsRng.fill_bytes(&mut new_nonce);

            let new_key_material = derive_key(new_key.as_ref(), &new_salt)?;
            let new_cipher = Aes256Gcm::new(GenericArray::from_slice(new_key_material.as_ref()));
            let new_ciphertext = new_cipher
                .encrypt(GenericArray::from_slice(&new_nonce), plaintext.as_ref())
                .map_err(|_| KeystoreError::Encryption)?;

            updated_entries.insert(
                name.clone(),
                StoredSecret {
                    salt: BASE64_ENGINE.encode(new_salt),
                    nonce: BASE64_ENGINE.encode(new_nonce),
                    ciphertext: BASE64_ENGINE.encode(new_ciphertext),
                    created_at: entry.created_at,
                    updated_at: Utc::now(),
                },
            );
        }

        guard.secrets = updated_entries;
        persist_document(&self.path, &guard)?;

        let entry = Entry::new(KEYRING_SERVICE, MASTER_KEY_ID)?;
        entry.set_password(&new_key_b64)?;

        Ok(())
    }

    pub fn list_keys(&self) -> Result<Vec<String>, KeystoreError> {
        let guard = self.lock_document()?;
        Ok(guard.secrets.keys().cloned().collect())
    }

    fn lock_document(&self) -> Result<MutexGuard<'_, KeystoreDocument>, KeystoreError> {
        self.document.lock().map_err(|_| KeystoreError::Internal)
    }

    fn master_key() -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        let entry = Entry::new(KEYRING_SERVICE, MASTER_KEY_ID)?;
        match entry.get_password() {
            Ok(value) => {
                let decoded = BASE64_ENGINE
                    .decode(value.as_bytes())
                    .map_err(|_| KeystoreError::Decryption)?;
                Ok(Zeroizing::new(decoded))
            }
            Err(keyring::Error::NoEntry) => {
                let mut key = Zeroizing::new(vec![0u8; 32]);
                OsRng.fill_bytes(&mut key);
                let key_slice: &[u8] = key.as_ref();
                let encoded = BASE64_ENGINE.encode(key_slice);
                entry.set_password(&encoded)?;
                Ok(key)
            }
            Err(err) => Err(KeystoreError::Keyring(err)),
        }
    }
}

fn derive_key(secret: &[u8], salt: &[u8]) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|_| KeystoreError::Encryption)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = Zeroizing::new(vec![0u8; 32]);
    argon2
        .hash_password_into(secret, salt, output.as_mut())
        .map_err(|_| KeystoreError::Encryption)?;

    Ok(output)
}

fn persist_document(path: &PathBuf, document: &KeystoreDocument) -> Result<(), KeystoreError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let serialized = serde_json::to_string_pretty(document)?;
    fs::write(path, serialized)?;
    Ok(())
}

fn keystore_path(app: &AppHandle) -> Result<PathBuf, KeystoreError> {
    let app_handle = app.clone();
    let mut path = app_handle
        .path()
        .app_data_dir()
        .map_err(|_| KeystoreError::Internal)?;
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    path.push(KEYSTORE_FILE);
    Ok(path)
}
