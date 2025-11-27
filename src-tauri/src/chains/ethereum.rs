use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

use super::types::*;
use super::ChainId;

#[derive(Debug)]
pub struct EthereumAdapter {
    rpc_url: String,
    chain_name: String,
    native_symbol: String,
}

impl EthereumAdapter {
    pub fn new(
        rpc_url: String,
        chain_name: impl Into<String>,
        native_symbol: impl Into<String>,
    ) -> Self {
        Self {
            rpc_url,
            chain_name: chain_name.into(),
            native_symbol: native_symbol.into(),
        }
    }
}

#[async_trait]
impl ChainAdapter for EthereumAdapter {
    async fn get_balance(&self, wallet: &WalletInfo) -> Result<ChainBalance, String> {
        let client = reqwest::Client::new();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_getBalance",
            "params": [wallet.public_key, "latest"],
        });

        let response = client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("RPC request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("RPC error: {}", response.status()));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let wei_balance = data["result"].as_str().ok_or("Invalid balance result")?;

        let wei = u128::from_str_radix(&wei_balance.trim_start_matches("0x"), 16)
            .map_err(|e| format!("Invalid wei balance: {}", e))?;

        let eth_balance = wei as f64 / 1e18;

        Ok(ChainBalance {
            native_balance: eth_balance,
            tokens: vec![],
            total_usd_value: eth_balance * 3200.0,
        })
    }

    async fn get_fee_estimate(&self, _wallet: &WalletInfo) -> Result<ChainFeeEstimate, String> {
        let client = reqwest::Client::new();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_gasPrice",
            "params": [],
        });

        let response = client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Gas price request failed: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let gas_price_hex = data["result"].as_str().ok_or("Invalid gas price result")?;

        let gas_price_wei = u128::from_str_radix(&gas_price_hex.trim_start_matches("0x"), 16)
            .map_err(|e| format!("Invalid gas price: {}", e))?;

        let gas_price_gwei = gas_price_wei as f64 / 1e9;

        Ok(ChainFeeEstimate {
            max_fee: gas_price_gwei * 21000.0 / 1e9,
            avg_fee: gas_price_gwei * 15000.0 / 1e9,
            fee_currency: self.native_symbol.clone(),
            estimated_time_seconds: 15,
        })
    }

    async fn build_transfer(
        &self,
        wallet: &WalletInfo,
        to: &str,
        amount: f64,
    ) -> Result<ChainTransaction, String> {
        // Simplified, actual implementation would encode transaction RLP
        let wei_amount = (amount * 1e18) as u128;
        let mut metadata = HashMap::new();
        metadata.insert("from".to_string(), wallet.public_key.clone());
        metadata.insert("to".to_string(), to.to_string());
        metadata.insert("amount_wei".to_string(), format!("0x{:x}", wei_amount));

        Ok(ChainTransaction {
            chain_id: ChainId::Ethereum,
            raw_tx: vec![],
            signatures: vec![],
            metadata,
        })
    }

    async fn submit_transaction(&self, tx: ChainTransaction) -> Result<String, String> {
        let client = reqwest::Client::new();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_sendRawTransaction",
            "params": [format!("0x{}", hex::encode(tx.raw_tx))],
        });

        let response = client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Transaction submission failed: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(error) = data.get("error") {
            return Err(format!("Transaction error: {}", error));
        }

        let hash = data["result"].as_str().ok_or("Invalid hash result")?;
        Ok(hash.to_string())
    }

    async fn get_status(&self) -> Result<ChainStatus, String> {
        let client = reqwest::Client::new();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_blockNumber",
            "params": [],
        });

        let start = std::time::Instant::now();
        let response = client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Status request failed: {}", e))?;

        let latency = start.elapsed().as_millis() as f64;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let block_hex = data["result"].as_str().ok_or("Invalid block number")?;
        let block_number = u64::from_str_radix(&block_hex.trim_start_matches("0x"), 16)
            .map_err(|e| format!("Invalid block number: {}", e))?;

        Ok(ChainStatus {
            chain_id: ChainId::Ethereum,
            rpc_healthy: true,
            latest_block_height: block_number,
            average_latency_ms: latency,
        })
    }
}
