use crate::data::event_store::{Event as AuditEvent, SharedEventStore};
use crate::security::activity_log::ActivityLogger;
use base64::engine::general_purpose::STANDARD as BASE64_ENGINE;
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::VersionedTransaction};
use std::{fs, path::PathBuf, str::FromStr};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{Mutex, MutexGuard};

const SESSION_FILE: &str = "phantom_session.json";
const DEFAULT_NETWORK: &str = "devnet";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomSession {
    pub public_key: String,
    pub network: String,
    pub connected: bool,
    pub last_connected: Option<String>,
    pub label: Option<String>,
}

impl PhantomSession {
    fn new(public_key: String, network: String, label: Option<String>) -> Self {
        Self {
            public_key,
            network,
            connected: true,
            last_connected: Some(Utc::now().to_rfc3339()),
            label,
        }
    }
}

fn get_event_store(handle: &AppHandle) -> Option<SharedEventStore> {
    handle
        .try_state::<SharedEventStore>()
        .map(|state| state.inner().clone())
}

async fn publish_wallet_event(handle: &AppHandle, event: AuditEvent, wallet_address: &str) {
    if let Some(store) = get_event_store(handle) {
        let aggregate_id = format!("wallet_{}", wallet_address);
        let result = {
            let guard = store.read().await;
            guard.publish_event(event, &aggregate_id).await
        };

        if let Err(err) = result {
            eprintln!(
                "Failed to publish wallet event for {}: {}",
                wallet_address, err
            );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomConnectPayload {
    pub public_key: String,
    pub network: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomSignMessageRequest {
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomSignMessageResponse {
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomSignTransactionRequest {
    pub transaction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomSignTransactionResponse {
    pub valid: bool,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhantomError {
    pub code: PhantomErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhantomErrorCode {
    NotConnected,
    InvalidInput,
    Storage,
    Serialization,
    Internal,
}

impl PhantomError {
    fn new(code: PhantomErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn storage(message: impl Into<String>) -> Self {
        Self::new(PhantomErrorCode::Storage, message)
    }

    fn serialization(message: impl Into<String>) -> Self {
        Self::new(PhantomErrorCode::Serialization, message)
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::new(PhantomErrorCode::Internal, message)
    }
}

impl std::fmt::Display for PhantomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            serde_json::to_string(&self.code).unwrap_or_else(|_| "unknown".to_string()),
            self.message
        )
    }
}

impl std::error::Error for PhantomError {}

#[derive(Default)]
pub struct WalletState {
    session: Mutex<Option<PhantomSession>>,
}

impl WalletState {
    pub fn new() -> Self {
        Self::default()
    }
}

async fn lock_session<'a>(
    state: &'a State<'_, WalletState>,
) -> MutexGuard<'a, Option<PhantomSession>> {
    state.session.lock().await
}

pub fn hydrate_wallet_state(app: &AppHandle) -> Result<(), PhantomError> {
    let state: State<WalletState> = app.state();
    let mut guard = tauri::async_runtime::block_on(state.session.lock());
    if guard.is_none() {
        if let Some(session) = read_persisted_session(app)? {
            *guard = Some(session);
        }
    }
    Ok(())
}

fn session_path(app: &AppHandle) -> Result<PathBuf, PhantomError> {
    let mut path = app.path().app_data_dir().map_err(|err| {
        PhantomError::storage(format!("Unable to resolve app data directory: {err}"))
    })?;
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|err| {
            PhantomError::storage(format!("Failed to create app data directory: {err}"))
        })?;
    }
    path.push(SESSION_FILE);
    Ok(path)
}

fn persist_session(app: &AppHandle, session: &PhantomSession) -> Result<(), PhantomError> {
    let path = session_path(app)?;
    let data = serde_json::to_string(session).map_err(|err| {
        PhantomError::serialization(format!("Failed to serialize session: {err}"))
    })?;
    fs::write(&path, data)
        .map_err(|err| PhantomError::storage(format!("Failed to persist session: {err}")))
}

fn remove_persisted_session(app: &AppHandle) -> Result<(), PhantomError> {
    let path = session_path(app)?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|err| PhantomError::storage(format!("Failed to remove session: {err}")))?;
    }
    Ok(())
}

fn read_persisted_session(app: &AppHandle) -> Result<Option<PhantomSession>, PhantomError> {
    let path = session_path(app)?;
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(path)
        .map_err(|err| PhantomError::storage(format!("Failed to read session file: {err}")))?;
    let session = serde_json::from_str(&data)
        .map_err(|err| PhantomError::serialization(format!("Failed to parse session: {err}")))?;
    Ok(Some(session))
}

#[tauri::command]
pub async fn phantom_connect(
    payload: PhantomConnectPayload,
    state: State<'_, WalletState>,
    app: AppHandle,
) -> Result<PhantomSession, PhantomError> {
    let logger = app.state::<ActivityLogger>();
    let public_key = payload.public_key.trim().to_string();

    if public_key.is_empty() {
        let _ = logger
            .log_connect(
                "unknown",
                json!({ "error": "Public key is required", "source": "phantom" }),
                false,
                None,
            )
            .await;

        return Err(PhantomError::new(
            PhantomErrorCode::InvalidInput,
            "Public key is required",
        ));
    }

    let network = payload
        .network
        .clone()
        .unwrap_or_else(|| DEFAULT_NETWORK.to_string());
    let label = payload.label.clone();

    let session = PhantomSession::new(public_key.clone(), network.clone(), label.clone());

    if let Err(err) = persist_session(&app, &session) {
        let _ = logger
            .log_connect(
                &public_key,
                json!({
                    "network": network.clone(),
                    "label": label.clone(),
                    "error": err.to_string(),
                    "source": "phantom"
                }),
                false,
                None,
            )
            .await;
        return Err(err);
    }

    {
        let mut guard = lock_session(&state).await;
        *guard = Some(session.clone());
    }

    let _ = logger
        .log_connect(
            &public_key,
            json!({
                "network": network.clone(),
                "label": label.clone(),
                "source": "phantom"
            }),
            true,
            None,
        )
        .await;

    // Publish wallet connected event to event store
    publish_wallet_event(
        &app,
        AuditEvent::WalletConnected {
            wallet_address: public_key.clone(),
            wallet_type: "phantom".to_string(),
            timestamp: Utc::now(),
        },
        &public_key,
    )
    .await;

    Ok(session)
}

#[tauri::command]
pub async fn phantom_disconnect(
    state: State<'_, WalletState>,
    app: AppHandle,
) -> Result<(), PhantomError> {
    let logger = app.state::<ActivityLogger>();
    let wallet_address = {
        let guard = lock_session(&state).await;
        guard.as_ref().map(|s| s.public_key.clone())
    };

    let had_session = wallet_address.is_some();
    let wallet_addr = wallet_address.unwrap_or_else(|| "unknown".to_string());

    {
        let mut guard = lock_session(&state).await;
        *guard = None;
    }

    match remove_persisted_session(&app) {
        Ok(_) => {
            let _ = logger
                .log_disconnect(
                    &wallet_addr,
                    json!({
                        "source": "phantom",
                        "hadSession": had_session
                    }),
                    had_session,
                    None,
                )
                .await;

            if had_session {
                publish_wallet_event(
                    &app,
                    AuditEvent::WalletDisconnected {
                        wallet_address: wallet_addr.clone(),
                        timestamp: Utc::now(),
                    },
                    &wallet_addr,
                )
                .await;
            }

            Ok(())
        }
        Err(err) => {
            let _ = logger
                .log_disconnect(
                    &wallet_addr,
                    json!({
                        "error": err.to_string(),
                        "source": "phantom",
                        "hadSession": had_session
                    }),
                    false,
                    None,
                )
                .await;
            Err(err)
        }
    }
}

#[tauri::command]
pub async fn phantom_session(
    state: State<'_, WalletState>,
) -> Result<Option<PhantomSession>, PhantomError> {
    let guard = lock_session(&state).await;
    Ok(guard.clone())
}

#[tauri::command]
pub async fn phantom_sign_message(
    request: PhantomSignMessageRequest,
    state: State<'_, WalletState>,
    app: AppHandle,
) -> Result<PhantomSignMessageResponse, PhantomError> {
    let logger = app.state::<ActivityLogger>();
    let guard = lock_session(&state).await;
    let session = guard.as_ref().ok_or_else(|| {
        PhantomError::new(PhantomErrorCode::NotConnected, "Wallet is not connected")
    })?;
    let session_data = session.clone();
    drop(guard);

    // Now we can use session_data safely

    let wallet_address = session_data.public_key.clone();
    let pubkey = match Pubkey::from_str(&session_data.public_key) {
        Ok(value) => value,
        Err(err) => {
            let err_msg = err.to_string();
            let _ = logger
                .log_sign(
                    &wallet_address,
                    json!({
                        "error": err_msg,
                        "source": "phantom"
                    }),
                    false,
                    None,
                )
                .await;
            return Err(PhantomError::new(
                PhantomErrorCode::InvalidInput,
                format!("Invalid session public key: {err}"),
            ));
        }
    };

    let signature = match Signature::from_str(&request.signature) {
        Ok(value) => value,
        Err(err) => {
            let err_msg = err.to_string();
            let _ = logger
                .log_sign(
                    &wallet_address,
                    json!({
                        "error": err_msg,
                        "source": "phantom"
                    }),
                    false,
                    None,
                )
                .await;
            return Err(PhantomError::new(
                PhantomErrorCode::InvalidInput,
                format!("Invalid signature: {err}"),
            ));
        }
    };

    let valid = signature.verify(pubkey.as_ref(), request.message.as_bytes());

    let _ = logger
        .log_sign(
            &wallet_address,
            json!({
                "messageLength": request.message.len(),
                "signatureValid": valid,
                "source": "phantom"
            }),
            valid,
            None,
        )
        .await;

    Ok(PhantomSignMessageResponse { valid })
}

#[tauri::command]
pub async fn phantom_sign_transaction(
    request: PhantomSignTransactionRequest,
    state: State<'_, WalletState>,
) -> Result<PhantomSignTransactionResponse, PhantomError> {
    let session_data = {
        let guard = lock_session(&state).await;
        let session = guard.as_ref().ok_or_else(|| {
            PhantomError::new(PhantomErrorCode::NotConnected, "Wallet is not connected")
        })?;
        session.clone()
    };

    let bytes = BASE64_ENGINE
        .decode(request.transaction.as_bytes())
        .map_err(|err| {
            PhantomError::new(
                PhantomErrorCode::InvalidInput,
                format!("Invalid transaction encoding: {err}"),
            )
        })?;

    let transaction: VersionedTransaction = bincode::deserialize(&bytes).map_err(|err| {
        PhantomError::serialization(format!("Failed to decode transaction: {err}"))
    })?;

    let signature = transaction.signatures.first().cloned().ok_or_else(|| {
        PhantomError::new(
            PhantomErrorCode::InvalidInput,
            "Transaction does not contain any signatures",
        )
    })?;

    let message_bytes = transaction.message.serialize();
    let pubkey = Pubkey::from_str(&session_data.public_key).map_err(|err| {
        PhantomError::new(
            PhantomErrorCode::InvalidInput,
            format!("Invalid session public key: {err}"),
        )
    })?;

    let valid = signature.verify(pubkey.as_ref(), &message_bytes);

    Ok(PhantomSignTransactionResponse {
        valid,
        signature: Some(signature.to_string()),
    })
}

fn resolve_endpoint(network: &str) -> String {
    if let Ok(custom) = std::env::var("SOLANA_RPC_ENDPOINT") {
        if !custom.trim().is_empty() {
            return custom;
        }
    }

    match network {
        "mainnet" | "mainnet-beta" => "https://api.mainnet-beta.solana.com".to_string(),
        "testnet" => "https://api.testnet.solana.com".to_string(),
        _ => "https://api.devnet.solana.com".to_string(),
    }
}

#[tauri::command]
pub async fn phantom_balance(
    address: String,
    state: State<'_, WalletState>,
) -> Result<f64, PhantomError> {
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    let pubkey = Pubkey::from_str(&address).map_err(|err| {
        PhantomError::new(
            PhantomErrorCode::InvalidInput,
            format!("Invalid address: {err}"),
        )
    })?;

    let network = {
        let guard = lock_session(&state).await;
        guard
            .as_ref()
            .map(|session| session.network.clone())
            .unwrap_or_else(|| DEFAULT_NETWORK.to_string())
    };

    let rpc_url = resolve_endpoint(&network);
    let client = RpcClient::new(rpc_url);

    match client.get_balance(&pubkey) {
        Ok(lamports) => Ok(lamports as f64 / 1_000_000_000.0),
        Err(err) => Err(PhantomError::internal(format!(
            "Failed to fetch balance: {err}"
        ))),
    }
}
