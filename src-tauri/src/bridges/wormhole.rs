use async_trait::async_trait;
use uuid::Uuid;

use super::types::*;
use super::{
    BridgeProvider, BridgeQuote, BridgeQuoteRequest, BridgeTransaction, BridgeTransactionRequest,
    BridgeTransactionStatus,
};

#[derive(Debug)]
pub struct WormholeAdapter;

impl WormholeAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BridgeAdapter for WormholeAdapter {
    async fn quote(&self, request: &BridgeQuoteRequest) -> Result<BridgeQuote, String> {
        let fee = request.amount * 0.002;
        let amount_out = request.amount - fee;

        let estimated_time = match (&request.from_chain, &request.to_chain) {
            (crate::chains::ChainId::Solana, _) => 300,
            (_, crate::chains::ChainId::Solana) => 300,
            _ => 900,
        };

        Ok(BridgeQuote {
            provider: BridgeProvider::Wormhole,
            from_chain: request.from_chain.clone(),
            to_chain: request.to_chain.clone(),
            amount_in: request.amount,
            amount_out,
            estimated_time_seconds: estimated_time,
            fee_amount: fee,
            fee_currency: "USD".to_string(),
            route_info: format!(
                "Wormhole Bridge: {} -> {}",
                request.from_chain.as_str(),
                request.to_chain.as_str()
            ),
        })
    }

    async fn prepare_transaction(
        &self,
        request: &BridgeTransactionRequest,
    ) -> Result<BridgeTransaction, String> {
        let now = chrono::Utc::now().to_rfc3339();

        Ok(BridgeTransaction {
            id: Uuid::new_v4().to_string(),
            provider: BridgeProvider::Wormhole,
            from_chain: request.from_chain.clone(),
            to_chain: request.to_chain.clone(),
            token_address: request.token_address.clone(),
            amount: request.amount,
            recipient_address: request.recipient_address.clone(),
            sender_address: request.sender_address.clone(),
            status: BridgeTransactionStatus::Pending,
            source_tx_hash: None,
            destination_tx_hash: None,
            created_at: now.clone(),
            updated_at: now,
            completed_at: None,
        })
    }

    async fn poll_status(&self, _transaction_id: &str) -> Result<BridgeTransactionStatus, String> {
        Ok(BridgeTransactionStatus::Bridging)
    }
}
