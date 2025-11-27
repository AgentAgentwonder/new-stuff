use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use super::types::*;
use super::{AllBridgeAdapter, SynapseAdapter, WormholeAdapter};
use super::{
    BridgeProvider, BridgeQuote, BridgeQuoteRequest, BridgeTransaction, BridgeTransactionRequest,
    BridgeTransactionStatus, SharedBridgeManager,
};

#[tauri::command]
pub async fn bridge_get_quote(
    request: BridgeQuoteRequest,
    provider: Option<String>,
) -> Result<Vec<BridgeQuote>, String> {
    let mut quotes = Vec::new();

    if let Some(prov_str) = provider {
        let prov = BridgeProvider::from_str(&prov_str)
            .ok_or_else(|| format!("Invalid bridge provider: {}", prov_str))?;
        let adapter = get_bridge_adapter(&prov);
        let quote = adapter.quote(&request).await?;
        quotes.push(quote);
    } else {
        let wormhole = WormholeAdapter::new();
        let allbridge = AllBridgeAdapter::new();
        let synapse = SynapseAdapter::new();

        if let Ok(quote) = wormhole.quote(&request).await {
            quotes.push(quote);
        }
        if let Ok(quote) = allbridge.quote(&request).await {
            quotes.push(quote);
        }
        if let Ok(quote) = synapse.quote(&request).await {
            quotes.push(quote);
        }
    }

    quotes.sort_by(|a, b| {
        b.amount_out
            .partial_cmp(&a.amount_out)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(quotes)
}

#[tauri::command]
pub async fn bridge_create_transaction(
    request: BridgeTransactionRequest,
    bridge_manager: State<'_, SharedBridgeManager>,
) -> Result<BridgeTransaction, String> {
    let adapter = get_bridge_adapter(&request.provider);
    let mut transaction = adapter.prepare_transaction(&request).await?;

    let mut manager = bridge_manager.write().await;
    manager.add_transaction(transaction.clone());

    Ok(transaction)
}

#[tauri::command]
pub async fn bridge_get_transaction(
    transaction_id: String,
    bridge_manager: State<'_, SharedBridgeManager>,
) -> Result<BridgeTransaction, String> {
    let manager = bridge_manager.read().await;
    manager
        .get_transaction(&transaction_id)
        .cloned()
        .ok_or_else(|| format!("Transaction {} not found", transaction_id))
}

#[tauri::command]
pub async fn bridge_list_transactions(
    bridge_manager: State<'_, SharedBridgeManager>,
) -> Result<Vec<BridgeTransaction>, String> {
    let manager = bridge_manager.read().await;
    Ok(manager.list_transactions())
}

#[tauri::command]
pub async fn bridge_list_transactions_by_status(
    status: String,
    bridge_manager: State<'_, SharedBridgeManager>,
) -> Result<Vec<BridgeTransaction>, String> {
    let status_enum = match status.to_lowercase().as_str() {
        "pending" => BridgeTransactionStatus::Pending,
        "submitted" => BridgeTransactionStatus::Submitted,
        "confirmed" => BridgeTransactionStatus::Confirmed,
        "bridging" => BridgeTransactionStatus::Bridging,
        "completed" => BridgeTransactionStatus::Completed,
        "failed" => BridgeTransactionStatus::Failed,
        _ => return Err(format!("Invalid status: {}", status)),
    };

    let manager = bridge_manager.read().await;
    Ok(manager.list_transactions_by_status(&status_enum))
}

#[tauri::command]
pub async fn bridge_update_transaction_status(
    transaction_id: String,
    status: String,
    bridge_manager: State<'_, SharedBridgeManager>,
) -> Result<(), String> {
    let status_enum = match status.to_lowercase().as_str() {
        "pending" => BridgeTransactionStatus::Pending,
        "submitted" => BridgeTransactionStatus::Submitted,
        "confirmed" => BridgeTransactionStatus::Confirmed,
        "bridging" => BridgeTransactionStatus::Bridging,
        "completed" => BridgeTransactionStatus::Completed,
        "failed" => BridgeTransactionStatus::Failed,
        _ => return Err(format!("Invalid status: {}", status)),
    };

    let mut manager = bridge_manager.write().await;
    manager.update_transaction_status(&transaction_id, status_enum)
}

#[tauri::command]
pub async fn bridge_update_transaction_hash(
    transaction_id: String,
    source_hash: Option<String>,
    destination_hash: Option<String>,
    bridge_manager: State<'_, SharedBridgeManager>,
) -> Result<(), String> {
    let mut manager = bridge_manager.write().await;
    manager.update_transaction_hash(&transaction_id, source_hash, destination_hash)
}

#[tauri::command]
pub async fn bridge_poll_status(
    transaction_id: String,
    provider: String,
    bridge_manager: State<'_, SharedBridgeManager>,
) -> Result<BridgeTransactionStatus, String> {
    let prov = BridgeProvider::from_str(&provider)
        .ok_or_else(|| format!("Invalid bridge provider: {}", provider))?;

    let adapter = get_bridge_adapter(&prov);
    let status = adapter.poll_status(&transaction_id).await?;

    let mut manager = bridge_manager.write().await;
    manager.update_transaction_status(&transaction_id, status.clone())?;

    Ok(status)
}

fn get_bridge_adapter(provider: &BridgeProvider) -> SharedBridgeAdapter {
    match provider {
        BridgeProvider::Wormhole => Arc::new(WormholeAdapter::new()),
        BridgeProvider::AllBridge => Arc::new(AllBridgeAdapter::new()),
        BridgeProvider::Synapse => Arc::new(SynapseAdapter::new()),
    }
}
