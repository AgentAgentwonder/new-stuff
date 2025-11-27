use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

use super::ChainId;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChainBalance {
    pub native_balance: f64,
    pub tokens: Vec<TokenBalance>,
    pub total_usd_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: String,
    pub amount: f64,
    pub usd_value: f64,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainFeeEstimate {
    pub max_fee: f64,
    pub avg_fee: f64,
    pub fee_currency: String,
    pub estimated_time_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTransaction {
    pub chain_id: ChainId,
    pub raw_tx: Vec<u8>,
    pub signatures: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainQuoteRequest {
    pub from_mint: String,
    pub to_mint: String,
    pub amount: f64,
    pub slippage_bps: u16,
    pub user_wallet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainQuoteResponse {
    pub chain_id: ChainId,
    pub from_mint: String,
    pub to_mint: String,
    pub amount_in: f64,
    pub amount_out: f64,
    pub route: Vec<String>,
    pub estimated_fee: ChainFeeEstimate,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletInfo {
    #[serde(default)]
    pub public_key: String,
    pub label: Option<String>,
    #[serde(default)]
    pub chain_id: ChainId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatus {
    pub chain_id: ChainId,
    pub rpc_healthy: bool,
    pub latest_block_height: u64,
    pub average_latency_ms: f64,
}

#[async_trait]
pub trait ChainAdapter: Send + Sync + Debug {
    async fn get_balance(&self, wallet: &WalletInfo) -> Result<ChainBalance, String>;
    async fn get_fee_estimate(&self, wallet: &WalletInfo) -> Result<ChainFeeEstimate, String>;
    async fn build_transfer(
        &self,
        wallet: &WalletInfo,
        to: &str,
        amount: f64,
    ) -> Result<ChainTransaction, String>;
    async fn quote_swap(&self, _request: ChainQuoteRequest) -> Result<ChainQuoteResponse, String> {
        Err("Swap not supported for this chain".to_string())
    }
    async fn submit_transaction(&self, tx: ChainTransaction) -> Result<String, String>;
    async fn get_status(&self) -> Result<ChainStatus, String>;
}

pub type SharedChainAdapter = std::sync::Arc<dyn ChainAdapter>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainPortfolioSnapshot {
    pub chain_id: ChainId,
    pub balances: ChainBalance,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrossChainPortfolioSummary {
    pub total_value_usd: f64,
    pub per_chain: Vec<ChainPortfolioSnapshot>,
    pub per_wallet: Vec<WalletPortfolioBreakdown>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletPortfolioBreakdown {
    pub wallet: WalletInfo,
    pub total_value_usd: f64,
    pub tokens: Vec<TokenBalance>,
}
