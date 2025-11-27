use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    BridgeProvider, BridgeQuote, BridgeQuoteRequest, BridgeTransaction, BridgeTransactionRequest,
    BridgeTransactionStatus,
};

#[async_trait]
pub trait BridgeAdapter: Send + Sync {
    async fn quote(&self, request: &BridgeQuoteRequest) -> Result<BridgeQuote, String>;
    async fn prepare_transaction(
        &self,
        request: &BridgeTransactionRequest,
    ) -> Result<BridgeTransaction, String>;
    async fn poll_status(&self, transaction_id: &str) -> Result<BridgeTransactionStatus, String>;
}

pub type SharedBridgeAdapter = std::sync::Arc<dyn BridgeAdapter>;
