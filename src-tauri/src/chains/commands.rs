use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

use super::types::*;
use super::{ArbitrumAdapter, BaseAdapter, EthereumAdapter, PolygonAdapter, SolanaAdapter};
use super::{ChainConfig, ChainId, ChainManager, SharedChainManager};

#[tauri::command]
pub async fn chain_get_active(
    chain_manager: State<'_, SharedChainManager>,
) -> Result<ChainId, String> {
    let manager = chain_manager.read().await;
    Ok(manager.get_active_chain())
}

#[tauri::command]
pub async fn chain_set_active(
    chain_id: String,
    chain_manager: State<'_, SharedChainManager>,
) -> Result<ChainId, String> {
    let chain =
        ChainId::from_str(&chain_id).ok_or_else(|| format!("Invalid chain ID: {}", chain_id))?;

    let mut manager = chain_manager.write().await;
    manager.set_active_chain(chain.clone())?;
    Ok(chain)
}

#[tauri::command]
pub async fn chain_list_chains(
    chain_manager: State<'_, SharedChainManager>,
) -> Result<Vec<ChainConfig>, String> {
    let manager = chain_manager.read().await;
    Ok(manager.list_chains())
}

#[tauri::command]
pub async fn chain_list_enabled(
    chain_manager: State<'_, SharedChainManager>,
) -> Result<Vec<ChainConfig>, String> {
    let manager = chain_manager.read().await;
    Ok(manager.list_enabled_chains())
}

#[tauri::command]
pub async fn chain_update_config(
    config: ChainConfig,
    chain_manager: State<'_, SharedChainManager>,
) -> Result<(), String> {
    let mut manager = chain_manager.write().await;
    manager.update_chain_config(config);
    Ok(())
}

#[tauri::command]
pub async fn chain_get_balance(
    wallet_address: String,
    chain_id: String,
    chain_manager: State<'_, SharedChainManager>,
) -> Result<ChainBalance, String> {
    let chain =
        ChainId::from_str(&chain_id).ok_or_else(|| format!("Invalid chain ID: {}", chain_id))?;

    let manager = chain_manager.read().await;
    let config = manager
        .get_chain_config(&chain)
        .ok_or_else(|| format!("Chain config not found for {:?}", chain))?;

    let wallet_info = WalletInfo {
        public_key: wallet_address,
        label: None,
        chain_id: chain.clone(),
    };

    let adapter = get_chain_adapter(&chain, &config.rpc_url);
    adapter.get_balance(&wallet_info).await
}

#[tauri::command]
pub async fn chain_get_fee_estimate(
    wallet_address: String,
    chain_id: String,
    chain_manager: State<'_, SharedChainManager>,
) -> Result<ChainFeeEstimate, String> {
    let chain =
        ChainId::from_str(&chain_id).ok_or_else(|| format!("Invalid chain ID: {}", chain_id))?;

    let manager = chain_manager.read().await;
    let config = manager
        .get_chain_config(&chain)
        .ok_or_else(|| format!("Chain config not found for {:?}", chain))?;

    let wallet_info = WalletInfo {
        public_key: wallet_address,
        label: None,
        chain_id: chain.clone(),
    };

    let adapter = get_chain_adapter(&chain, &config.rpc_url);
    adapter.get_fee_estimate(&wallet_info).await
}

#[tauri::command]
pub async fn chain_get_status(
    chain_id: String,
    chain_manager: State<'_, SharedChainManager>,
) -> Result<ChainStatus, String> {
    let chain =
        ChainId::from_str(&chain_id).ok_or_else(|| format!("Invalid chain ID: {}", chain_id))?;

    let manager = chain_manager.read().await;
    let config = manager
        .get_chain_config(&chain)
        .ok_or_else(|| format!("Chain config not found for {:?}", chain))?;

    let adapter = get_chain_adapter(&chain, &config.rpc_url);
    adapter.get_status().await
}

#[tauri::command]
pub async fn chain_get_cross_chain_portfolio(
    wallet_addresses: HashMap<String, String>,
    chain_manager: State<'_, SharedChainManager>,
) -> Result<CrossChainPortfolioSummary, String> {
    let manager = chain_manager.read().await;
    let mut summary = CrossChainPortfolioSummary::default();

    for (chain_str, address) in wallet_addresses.iter() {
        let chain = match ChainId::from_str(chain_str) {
            Some(c) => c,
            None => continue,
        };

        let config = match manager.get_chain_config(&chain) {
            Some(c) => c,
            None => continue,
        };

        let wallet_info = WalletInfo {
            public_key: address.clone(),
            label: None,
            chain_id: chain.clone(),
        };

        let adapter = get_chain_adapter(&chain, &config.rpc_url);

        if let Ok(balance) = adapter.get_balance(&wallet_info).await {
            summary.total_value_usd += balance.total_usd_value;
            summary.per_chain.push(ChainPortfolioSnapshot {
                chain_id: chain.clone(),
                balances: balance.clone(),
            });
            summary.per_wallet.push(WalletPortfolioBreakdown {
                wallet: wallet_info,
                total_value_usd: balance.total_usd_value,
                tokens: balance.tokens,
            });
        }
    }

    Ok(summary)
}

fn get_chain_adapter(chain: &ChainId, rpc_url: &str) -> SharedChainAdapter {
    match chain {
        ChainId::Solana => std::sync::Arc::new(SolanaAdapter::new(rpc_url.to_string())),
        ChainId::Ethereum => {
            std::sync::Arc::new(EthereumAdapter::new(rpc_url.to_string(), "Ethereum", "ETH"))
        }
        ChainId::Base => std::sync::Arc::new(BaseAdapter::new(rpc_url.to_string())),
        ChainId::Polygon => std::sync::Arc::new(PolygonAdapter::new(rpc_url.to_string())),
        ChainId::Arbitrum => std::sync::Arc::new(ArbitrumAdapter::new(rpc_url.to_string())),
    }
}
