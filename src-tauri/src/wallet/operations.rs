use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::security::keystore::{Keystore, KeystoreError};

const KEYSTORE_TOKEN_CACHE_KEY: &str = "wallet.token_cache";
const KEYSTORE_ADDRESS_BOOK_KEY: &str = "wallet.address_book";
const KEYSTORE_SWAP_HISTORY_KEY: &str = "wallet.swap_history";

// Token Balance Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub balance: f64,
    pub decimals: u8,
    pub usd_value: f64,
    pub change_24h: f64,
    pub logo_uri: Option<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalancesCache {
    pub balances: HashMap<String, Vec<TokenBalance>>,
    pub last_updated: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl Default for TokenBalancesCache {
    fn default() -> Self {
        Self {
            balances: HashMap::new(),
            last_updated: Utc::now(),
            ttl_seconds: 60,
        }
    }
}

// Transaction Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTransactionInput {
    pub recipient: String,
    pub amount: f64,
    pub token_mint: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionFeeEstimate {
    pub base_fee: f64,
    pub priority_fee: f64,
    pub total_fee: f64,
    pub estimated_units: u64,
}

// Address Book Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBookContact {
    pub id: String,
    pub address: String,
    pub label: String,
    pub nickname: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub transaction_count: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBook {
    pub contacts: HashMap<String, AddressBookContact>,
    pub last_updated: DateTime<Utc>,
}

impl Default for AddressBook {
    fn default() -> Self {
        Self {
            contacts: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddContactRequest {
    pub address: String,
    pub label: String,
    pub nickname: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContactRequest {
    pub contact_id: String,
    pub label: Option<String>,
    pub nickname: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

// Swap History Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapHistoryEntry {
    pub id: String,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: f64,
    pub to_amount: f64,
    pub rate: f64,
    pub fee: f64,
    pub price_impact: f64,
    pub tx_signature: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub status: SwapStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwapStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapHistory {
    pub swaps: Vec<SwapHistoryEntry>,
    pub last_updated: DateTime<Utc>,
}

impl Default for SwapHistory {
    fn default() -> Self {
        Self {
            swaps: Vec::new(),
            last_updated: Utc::now(),
        }
    }
}

// QR Code Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QRCodeData {
    pub address: String,
    pub amount: Option<f64>,
    pub label: Option<String>,
    pub message: Option<String>,
    pub spl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaPayQR {
    pub url: String,
    pub qr_data: String,
    pub recipient: String,
    pub amount: Option<f64>,
    pub spl_token: Option<String>,
    pub reference: Option<String>,
    pub label: Option<String>,
    pub message: Option<String>,
    pub memo: Option<String>,
}

// Bridge and CEX Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeProvider {
    pub id: String,
    pub name: String,
    pub logo: String,
    pub supported_chains: Vec<String>,
    pub fees: ProviderFees,
    pub estimated_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderFees {
    pub percentage: f64,
    pub fixed: f64,
}

// Managers
pub struct WalletOperationsManager {
    token_cache: Mutex<TokenBalancesCache>,
    address_book: Mutex<AddressBook>,
    swap_history: Mutex<SwapHistory>,
}

impl WalletOperationsManager {
    pub fn initialize(keystore: &Keystore) -> Result<Self, KeystoreError> {
        let token_cache = match keystore.retrieve_secret(KEYSTORE_TOKEN_CACHE_KEY) {
            Ok(raw) => serde_json::from_slice(&raw).unwrap_or_default(),
            Err(KeystoreError::NotFound) => TokenBalancesCache::default(),
            Err(err) => return Err(err),
        };

        let address_book = match keystore.retrieve_secret(KEYSTORE_ADDRESS_BOOK_KEY) {
            Ok(raw) => serde_json::from_slice(&raw).unwrap_or_default(),
            Err(KeystoreError::NotFound) => AddressBook::default(),
            Err(err) => return Err(err),
        };

        let swap_history = match keystore.retrieve_secret(KEYSTORE_SWAP_HISTORY_KEY) {
            Ok(raw) => serde_json::from_slice(&raw).unwrap_or_default(),
            Err(KeystoreError::NotFound) => SwapHistory::default(),
            Err(err) => return Err(err),
        };

        Ok(Self {
            token_cache: Mutex::new(token_cache),
            address_book: Mutex::new(address_book),
            swap_history: Mutex::new(swap_history),
        })
    }

    pub fn persist_token_cache(&self, keystore: &Keystore) -> Result<(), KeystoreError> {
        let guard = self
            .token_cache
            .lock()
            .map_err(|_| KeystoreError::LockError)?;
        let data = serde_json::to_vec(&*guard).map_err(|_| KeystoreError::SerializationError)?;
        keystore.store_secret(KEYSTORE_TOKEN_CACHE_KEY, &data)
    }

    pub fn persist_address_book(&self, keystore: &Keystore) -> Result<(), KeystoreError> {
        let guard = self
            .address_book
            .lock()
            .map_err(|_| KeystoreError::LockError)?;
        let data = serde_json::to_vec(&*guard).map_err(|_| KeystoreError::SerializationError)?;
        keystore.store_secret(KEYSTORE_ADDRESS_BOOK_KEY, &data)
    }

    pub fn persist_swap_history(&self, keystore: &Keystore) -> Result<(), KeystoreError> {
        let guard = self
            .swap_history
            .lock()
            .map_err(|_| KeystoreError::LockError)?;
        let data = serde_json::to_vec(&*guard).map_err(|_| KeystoreError::SerializationError)?;
        keystore.store_secret(KEYSTORE_SWAP_HISTORY_KEY, &data)
    }
}

// Tauri Commands
#[tauri::command]
pub async fn wallet_get_token_balances(
    address: String,
    force_refresh: bool,
    operations: State<'_, WalletOperationsManager>,
    keystore: State<'_, Keystore>,
) -> Result<Vec<TokenBalance>, String> {
    let mut cache = operations.token_cache.lock().map_err(|e| e.to_string())?;

    let now = Utc::now();
    let should_refresh = force_refresh
        || !cache.balances.contains_key(&address)
        || (now.timestamp() - cache.last_updated.timestamp()) > cache.ttl_seconds as i64;

    if should_refresh {
        // In a real implementation, this would fetch from blockchain
        // For now, we'll return mock data
        let mock_balances = vec![
            TokenBalance {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                symbol: "SOL".to_string(),
                name: "Solana".to_string(),
                balance: 1.5,
                decimals: 9,
                usd_value: 150.0,
                change_24h: 2.5,
                logo_uri: Some("https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png".to_string()),
                last_updated: now,
            },
            TokenBalance {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                symbol: "USDC".to_string(),
                name: "USD Coin".to_string(),
                balance: 100.0,
                decimals: 6,
                usd_value: 100.0,
                change_24h: 0.0,
                logo_uri: Some("https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v/logo.png".to_string()),
                last_updated: now,
            },
        ];

        cache
            .balances
            .insert(address.clone(), mock_balances.clone());
        cache.last_updated = now;

        operations
            .persist_token_cache(&keystore)
            .map_err(|e| e.to_string())?;

        Ok(mock_balances)
    } else {
        Ok(cache.balances.get(&address).cloned().unwrap_or_default())
    }
}

#[tauri::command]
pub async fn wallet_estimate_fee(
    recipient: String,
    amount: f64,
    token_mint: Option<String>,
) -> Result<TransactionFeeEstimate, String> {
    // Mock implementation - in production, this would calculate actual fees
    let base_fee = if token_mint.is_some() {
        0.00001
    } else {
        0.000005
    };
    let priority_fee = 0.000001;

    Ok(TransactionFeeEstimate {
        base_fee,
        priority_fee,
        total_fee: base_fee + priority_fee,
        estimated_units: 200000,
    })
}

#[tauri::command]
pub async fn wallet_send_transaction(
    input: SendTransactionInput,
    wallet_address: String,
) -> Result<String, String> {
    // Mock implementation - in production, this would sign and send transaction
    // Returns transaction signature
    Ok(format!("mock_tx_signature_{}", Uuid::new_v4()))
}

#[tauri::command]
pub async fn wallet_generate_qr(data: QRCodeData) -> Result<String, String> {
    // Generate basic QR code data URI
    // In production, use qrcode crate
    let json_data = serde_json::to_string(&data).map_err(|e| e.to_string())?;
    Ok(format!(
        "data:image/png;base64,mock_qr_code_for_{}",
        json_data
    ))
}

#[tauri::command]
pub async fn wallet_generate_solana_pay_qr(
    recipient: String,
    amount: Option<f64>,
    spl_token: Option<String>,
    reference: Option<String>,
    label: Option<String>,
    message: Option<String>,
    memo: Option<String>,
) -> Result<SolanaPayQR, String> {
    let mut url = format!("solana:{}", recipient);
    let mut params = vec![];

    if let Some(amt) = amount {
        params.push(format!("amount={}", amt));
    }
    if let Some(spl) = &spl_token {
        params.push(format!("spl-token={}", spl));
    }
    if let Some(ref_val) = &reference {
        params.push(format!("reference={}", ref_val));
    }
    if let Some(lbl) = &label {
        params.push(format!("label={}", lbl));
    }
    if let Some(msg) = &message {
        params.push(format!("message={}", msg));
    }
    if let Some(mem) = &memo {
        params.push(format!("memo={}", mem));
    }

    if !params.is_empty() {
        url.push_str("?");
        url.push_str(&params.join("&"));
    }

    Ok(SolanaPayQR {
        url: url.clone(),
        qr_data: format!("data:image/png;base64,mock_solana_pay_qr"),
        recipient: recipient.clone(),
        amount,
        spl_token,
        reference,
        label,
        message,
        memo,
    })
}

// Address Book Commands
#[tauri::command]
pub async fn address_book_add_contact(
    request: AddContactRequest,
    operations: State<'_, WalletOperationsManager>,
    keystore: State<'_, Keystore>,
) -> Result<AddressBookContact, String> {
    let mut book = operations.address_book.lock().map_err(|e| e.to_string())?;

    // Check if address already exists
    if book.contacts.values().any(|c| c.address == request.address) {
        return Err("Contact with this address already exists".to_string());
    }

    let contact_id = format!("contact_{}", Uuid::new_v4());
    let now = Utc::now();

    let contact = AddressBookContact {
        id: contact_id.clone(),
        address: request.address,
        label: request.label,
        nickname: request.nickname,
        notes: request.notes,
        created_at: now,
        updated_at: now,
        last_used: None,
        transaction_count: 0,
        tags: request.tags,
    };

    book.contacts.insert(contact_id, contact.clone());
    book.last_updated = now;

    operations
        .persist_address_book(&keystore)
        .map_err(|e| e.to_string())?;

    Ok(contact)
}

#[tauri::command]
pub async fn address_book_update_contact(
    request: UpdateContactRequest,
    operations: State<'_, WalletOperationsManager>,
    keystore: State<'_, Keystore>,
) -> Result<AddressBookContact, String> {
    let updated_contact = {
        let mut book = operations.address_book.lock().map_err(|e| e.to_string())?;

        let now = Utc::now();
        let updated = {
            let contact = book
                .contacts
                .get_mut(&request.contact_id)
                .ok_or_else(|| "Contact not found".to_string())?;

            if let Some(label) = request.label {
                contact.label = label;
            }
            if let Some(nickname) = request.nickname {
                contact.nickname = nickname;
            }
            if let Some(notes) = request.notes {
                contact.notes = notes;
            }
            if let Some(tags) = request.tags {
                contact.tags = tags;
            }

            contact.updated_at = now;
            contact.clone()
        };

        book.last_updated = now;
        updated
    };

    operations
        .persist_address_book(&keystore)
        .map_err(|e| e.to_string())?;

    Ok(updated_contact)
}

#[tauri::command]
pub async fn address_book_delete_contact(
    contact_id: String,
    operations: State<'_, WalletOperationsManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    let mut book = operations.address_book.lock().map_err(|e| e.to_string())?;

    book.contacts
        .remove(&contact_id)
        .ok_or_else(|| "Contact not found".to_string())?;

    book.last_updated = Utc::now();
    operations
        .persist_address_book(&keystore)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn address_book_list_contacts(
    operations: State<'_, WalletOperationsManager>,
) -> Result<Vec<AddressBookContact>, String> {
    let book = operations.address_book.lock().map_err(|e| e.to_string())?;

    let mut contacts: Vec<AddressBookContact> = book.contacts.values().cloned().collect();
    contacts.sort_by(|a, b| {
        b.last_used
            .cmp(&a.last_used)
            .then(b.created_at.cmp(&a.created_at))
    });

    Ok(contacts)
}

#[tauri::command]
pub async fn address_book_search_contacts(
    query: String,
    operations: State<'_, WalletOperationsManager>,
) -> Result<Vec<AddressBookContact>, String> {
    let book = operations.address_book.lock().map_err(|e| e.to_string())?;

    let query_lower = query.to_lowercase();
    let contacts: Vec<AddressBookContact> = book
        .contacts
        .values()
        .filter(|c| {
            c.label.to_lowercase().contains(&query_lower)
                || c.address.to_lowercase().contains(&query_lower)
                || c.nickname
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || c.tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect();

    Ok(contacts)
}

#[tauri::command]
pub async fn address_book_export(
    operations: State<'_, WalletOperationsManager>,
) -> Result<String, String> {
    let book = operations.address_book.lock().map_err(|e| e.to_string())?;
    serde_json::to_string_pretty(&*book).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn address_book_import(
    data: String,
    operations: State<'_, WalletOperationsManager>,
    keystore: State<'_, Keystore>,
) -> Result<usize, String> {
    let imported_book: AddressBook = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    let mut book = operations.address_book.lock().map_err(|e| e.to_string())?;
    let imported_count = imported_book.contacts.len();

    for (id, contact) in imported_book.contacts {
        if !book.contacts.contains_key(&id) {
            book.contacts.insert(id, contact);
        }
    }

    book.last_updated = Utc::now();
    operations
        .persist_address_book(&keystore)
        .map_err(|e| e.to_string())?;

    Ok(imported_count)
}

// Swap History Commands
#[tauri::command]
pub async fn swap_history_add_entry(
    entry: SwapHistoryEntry,
    operations: State<'_, WalletOperationsManager>,
    keystore: State<'_, Keystore>,
) -> Result<(), String> {
    let mut history = operations.swap_history.lock().map_err(|e| e.to_string())?;

    history.swaps.push(entry);
    history.swaps.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Keep only last 100 swaps
    if history.swaps.len() > 100 {
        history.swaps.truncate(100);
    }

    history.last_updated = Utc::now();
    operations
        .persist_swap_history(&keystore)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn swap_history_get_recent(
    limit: usize,
    operations: State<'_, WalletOperationsManager>,
) -> Result<Vec<SwapHistoryEntry>, String> {
    let history = operations.swap_history.lock().map_err(|e| e.to_string())?;

    let swaps: Vec<SwapHistoryEntry> = history.swaps.iter().take(limit).cloned().collect();

    Ok(swaps)
}

#[tauri::command]
pub async fn wallet_get_bridge_providers() -> Result<Vec<BridgeProvider>, String> {
    // Mock bridge providers
    Ok(vec![
        BridgeProvider {
            id: "wormhole".to_string(),
            name: "Wormhole".to_string(),
            logo: "https://wormhole.com/logo.png".to_string(),
            supported_chains: vec![
                "ethereum".to_string(),
                "bsc".to_string(),
                "polygon".to_string(),
            ],
            fees: ProviderFees {
                percentage: 0.1,
                fixed: 0.0,
            },
            estimated_time: "15-30 minutes".to_string(),
        },
        BridgeProvider {
            id: "allbridge".to_string(),
            name: "Allbridge".to_string(),
            logo: "https://allbridge.io/logo.png".to_string(),
            supported_chains: vec![
                "ethereum".to_string(),
                "avalanche".to_string(),
                "fantom".to_string(),
            ],
            fees: ProviderFees {
                percentage: 0.15,
                fixed: 0.0,
            },
            estimated_time: "10-20 minutes".to_string(),
        },
    ])
}
