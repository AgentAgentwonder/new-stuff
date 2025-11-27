pub mod arbitrum;
pub mod base;
pub mod commands;
pub mod ethereum;
pub mod polygon;
pub mod solana;
pub mod types;

pub use arbitrum::*;
pub use base::*;
pub use commands::*;
pub use ethereum::*;
pub use polygon::*;
pub use solana::*;
pub use types::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChainId {
    Solana,
    Ethereum,
    Base,
    Polygon,
    Arbitrum,
}

impl Default for ChainId {
    fn default() -> Self {
        ChainId::Solana
    }
}

impl ChainId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainId::Solana => "solana",
            ChainId::Ethereum => "ethereum",
            ChainId::Base => "base",
            ChainId::Polygon => "polygon",
            ChainId::Arbitrum => "arbitrum",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "solana" => Some(ChainId::Solana),
            "ethereum" | "eth" => Some(ChainId::Ethereum),
            "base" => Some(ChainId::Base),
            "polygon" | "matic" => Some(ChainId::Polygon),
            "arbitrum" | "arb" => Some(ChainId::Arbitrum),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub explorer_url: String,
    pub native_token: String,
    pub enabled: bool,
}

pub struct ChainManager {
    configs: HashMap<ChainId, ChainConfig>,
    active_chain: ChainId,
}

impl ChainManager {
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        // Initialize with default configs
        configs.insert(
            ChainId::Solana,
            ChainConfig {
                chain_id: ChainId::Solana,
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                explorer_url: "https://solscan.io".to_string(),
                native_token: "SOL".to_string(),
                enabled: true,
            },
        );

        configs.insert(
            ChainId::Ethereum,
            ChainConfig {
                chain_id: ChainId::Ethereum,
                rpc_url: "https://eth.llamarpc.com".to_string(),
                explorer_url: "https://etherscan.io".to_string(),
                native_token: "ETH".to_string(),
                enabled: true,
            },
        );

        configs.insert(
            ChainId::Base,
            ChainConfig {
                chain_id: ChainId::Base,
                rpc_url: "https://mainnet.base.org".to_string(),
                explorer_url: "https://basescan.org".to_string(),
                native_token: "ETH".to_string(),
                enabled: true,
            },
        );

        configs.insert(
            ChainId::Polygon,
            ChainConfig {
                chain_id: ChainId::Polygon,
                rpc_url: "https://polygon-rpc.com".to_string(),
                explorer_url: "https://polygonscan.com".to_string(),
                native_token: "MATIC".to_string(),
                enabled: true,
            },
        );

        configs.insert(
            ChainId::Arbitrum,
            ChainConfig {
                chain_id: ChainId::Arbitrum,
                rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
                explorer_url: "https://arbiscan.io".to_string(),
                native_token: "ETH".to_string(),
                enabled: true,
            },
        );

        ChainManager {
            configs,
            active_chain: ChainId::Solana,
        }
    }

    pub fn get_active_chain(&self) -> ChainId {
        self.active_chain.clone()
    }

    pub fn set_active_chain(&mut self, chain_id: ChainId) -> Result<(), String> {
        if let Some(config) = self.configs.get(&chain_id) {
            if config.enabled {
                self.active_chain = chain_id;
                Ok(())
            } else {
                Err(format!("Chain {:?} is not enabled", chain_id))
            }
        } else {
            Err(format!("Chain {:?} not found", chain_id))
        }
    }

    pub fn get_chain_config(&self, chain_id: &ChainId) -> Option<&ChainConfig> {
        self.configs.get(chain_id)
    }

    pub fn update_chain_config(&mut self, config: ChainConfig) {
        self.configs.insert(config.chain_id.clone(), config);
    }

    pub fn list_chains(&self) -> Vec<ChainConfig> {
        self.configs.values().cloned().collect()
    }

    pub fn list_enabled_chains(&self) -> Vec<ChainConfig> {
        self.configs
            .values()
            .filter(|c| c.enabled)
            .cloned()
            .collect()
    }
}

pub type SharedChainManager = Arc<RwLock<ChainManager>>;
