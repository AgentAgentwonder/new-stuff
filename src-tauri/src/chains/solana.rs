use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use std::collections::HashMap;

use super::types::*;
use super::ChainId;

#[derive(Debug)]
pub struct SolanaAdapter {
    rpc_url: String,
}

impl SolanaAdapter {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }
}

#[async_trait]
impl ChainAdapter for SolanaAdapter {
    async fn get_balance(&self, wallet: &WalletInfo) -> Result<ChainBalance, String> {
        // Mock implementation - would integrate with Solana RPC
        let client = reqwest::Client::new();

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [&wallet.public_key]
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

        let lamports = data["result"]["value"]
            .as_u64()
            .ok_or("Invalid balance response")?;

        let sol_balance = lamports as f64 / 1_000_000_000.0;

        Ok(ChainBalance {
            native_balance: sol_balance,
            tokens: vec![],
            total_usd_value: sol_balance * 150.0, // Mock price
        })
    }

    async fn get_fee_estimate(&self, _wallet: &WalletInfo) -> Result<ChainFeeEstimate, String> {
        Ok(ChainFeeEstimate {
            max_fee: 0.00005,
            avg_fee: 0.000005,
            fee_currency: "SOL".to_string(),
            estimated_time_seconds: 1,
        })
    }

    async fn build_transfer(
        &self,
        wallet: &WalletInfo,
        to: &str,
        amount: f64,
    ) -> Result<ChainTransaction, String> {
        let lamports = (amount * 1_000_000_000.0) as u64;

        let mut metadata = HashMap::new();
        metadata.insert("from".to_string(), wallet.public_key.clone());
        metadata.insert("to".to_string(), to.to_string());
        metadata.insert("lamports".to_string(), lamports.to_string());

        Ok(ChainTransaction {
            chain_id: ChainId::Solana,
            raw_tx: vec![],
            signatures: vec![],
            metadata,
        })
    }

    async fn quote_swap(&self, request: ChainQuoteRequest) -> Result<ChainQuoteResponse, String> {
        // Mock Jupiter integration
        let amount_out = request.amount * 0.995; // 0.5% slippage mock

        Ok(ChainQuoteResponse {
            chain_id: ChainId::Solana,
            from_mint: request.from_mint,
            to_mint: request.to_mint,
            amount_in: request.amount,
            amount_out,
            route: vec!["Raydium".to_string()],
            estimated_fee: ChainFeeEstimate {
                max_fee: 0.00005,
                avg_fee: 0.000005,
                fee_currency: "SOL".to_string(),
                estimated_time_seconds: 1,
            },
        })
    }

    async fn submit_transaction(&self, tx: ChainTransaction) -> Result<String, String> {
        let client = reqwest::Client::new();

        let tx_base58 = general_purpose::STANDARD.encode(&tx.raw_tx);

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [tx_base58, {"encoding": "base64"}]
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

        let signature = data["result"]
            .as_str()
            .ok_or("Invalid transaction response")?;

        Ok(signature.to_string())
    }

    async fn get_status(&self) -> Result<ChainStatus, String> {
        let client = reqwest::Client::new();

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSlot"
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

        let slot = data["result"].as_u64().ok_or("Invalid slot response")?;

        Ok(ChainStatus {
            chain_id: ChainId::Solana,
            rpc_healthy: true,
            latest_block_height: slot,
            average_latency_ms: latency,
        })
    }
}
