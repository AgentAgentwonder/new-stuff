pub mod allbridge;
pub mod commands;
pub mod synapse;
pub mod types;
pub mod wormhole;

pub use allbridge::*;
pub use commands::*;
pub use synapse::*;
pub use types::*;
pub use wormhole::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::chains::ChainId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BridgeProvider {
    Wormhole,
    AllBridge,
    Synapse,
}

impl BridgeProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            BridgeProvider::Wormhole => "wormhole",
            BridgeProvider::AllBridge => "allbridge",
            BridgeProvider::Synapse => "synapse",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wormhole" => Some(BridgeProvider::Wormhole),
            "allbridge" => Some(BridgeProvider::AllBridge),
            "synapse" => Some(BridgeProvider::Synapse),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeQuoteRequest {
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub token_address: String,
    pub amount: f64,
    pub recipient_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeQuote {
    pub provider: BridgeProvider,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub amount_in: f64,
    pub amount_out: f64,
    pub estimated_time_seconds: u64,
    pub fee_amount: f64,
    pub fee_currency: String,
    pub route_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransactionRequest {
    pub provider: BridgeProvider,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub token_address: String,
    pub amount: f64,
    pub recipient_address: String,
    pub sender_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BridgeTransactionStatus {
    Pending,
    Submitted,
    Confirmed,
    Bridging,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: String,
    pub provider: BridgeProvider,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub token_address: String,
    pub amount: f64,
    pub recipient_address: String,
    pub sender_address: String,
    pub status: BridgeTransactionStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

pub struct BridgeManager {
    transactions: HashMap<String, BridgeTransaction>,
}

impl BridgeManager {
    pub fn new() -> Self {
        Self {
            transactions: HashMap::new(),
        }
    }

    pub fn get_transaction(&self, id: &str) -> Option<&BridgeTransaction> {
        self.transactions.get(id)
    }

    pub fn list_transactions(&self) -> Vec<BridgeTransaction> {
        self.transactions.values().cloned().collect()
    }

    pub fn list_transactions_by_status(
        &self,
        status: &BridgeTransactionStatus,
    ) -> Vec<BridgeTransaction> {
        self.transactions
            .values()
            .filter(|tx| std::mem::discriminant(&tx.status) == std::mem::discriminant(status))
            .cloned()
            .collect()
    }

    pub fn add_transaction(&mut self, transaction: BridgeTransaction) {
        self.transactions
            .insert(transaction.id.clone(), transaction);
    }

    pub fn update_transaction_status(
        &mut self,
        id: &str,
        status: BridgeTransactionStatus,
    ) -> Result<(), String> {
        if let Some(tx) = self.transactions.get_mut(id) {
            tx.status = status;
            tx.updated_at = chrono::Utc::now().to_rfc3339();
            if matches!(
                tx.status,
                BridgeTransactionStatus::Completed | BridgeTransactionStatus::Failed
            ) {
                tx.completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
            Ok(())
        } else {
            Err(format!("Transaction {} not found", id))
        }
    }

    pub fn update_transaction_hash(
        &mut self,
        id: &str,
        source_hash: Option<String>,
        destination_hash: Option<String>,
    ) -> Result<(), String> {
        if let Some(tx) = self.transactions.get_mut(id) {
            if let Some(hash) = source_hash {
                tx.source_tx_hash = Some(hash);
            }
            if let Some(hash) = destination_hash {
                tx.destination_tx_hash = Some(hash);
            }
            tx.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err(format!("Transaction {} not found", id))
        }
    }
}

pub type SharedBridgeManager = Arc<RwLock<BridgeManager>>;
