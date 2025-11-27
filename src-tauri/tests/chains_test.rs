#[cfg(test)]
mod chain_tests {
    use app_lib::chains::*;

    #[test]
    fn test_chain_manager_creation() {
        let manager = ChainManager::new();
        assert_eq!(manager.get_active_chain(), ChainId::Solana);
        assert_eq!(manager.list_enabled_chains().len(), 5);
    }

    #[test]
    fn test_chain_switching() {
        let mut manager = ChainManager::new();
        assert!(manager.set_active_chain(ChainId::Ethereum).is_ok());
        assert_eq!(manager.get_active_chain(), ChainId::Ethereum);
    }

    #[test]
    fn test_chain_config() {
        let manager = ChainManager::new();
        let solana_config = manager.get_chain_config(&ChainId::Solana);
        assert!(solana_config.is_some());
        assert_eq!(solana_config.unwrap().native_token, "SOL");
    }

    #[test]
    fn test_chain_id_from_str() {
        assert_eq!(ChainId::from_str("solana"), Some(ChainId::Solana));
        assert_eq!(ChainId::from_str("ethereum"), Some(ChainId::Ethereum));
        assert_eq!(ChainId::from_str("eth"), Some(ChainId::Ethereum));
        assert_eq!(ChainId::from_str("base"), Some(ChainId::Base));
        assert_eq!(ChainId::from_str("polygon"), Some(ChainId::Polygon));
        assert_eq!(ChainId::from_str("matic"), Some(ChainId::Polygon));
        assert_eq!(ChainId::from_str("arbitrum"), Some(ChainId::Arbitrum));
        assert_eq!(ChainId::from_str("invalid"), None);
    }
}

#[cfg(test)]
mod bridge_tests {
    use app_lib::bridges::*;

    #[test]
    fn test_bridge_manager_creation() {
        let manager = BridgeManager::new();
        assert_eq!(manager.list_transactions().len(), 0);
    }

    #[test]
    fn test_bridge_provider_from_str() {
        assert_eq!(
            BridgeProvider::from_str("wormhole"),
            Some(BridgeProvider::Wormhole)
        );
        assert_eq!(
            BridgeProvider::from_str("allbridge"),
            Some(BridgeProvider::AllBridge)
        );
        assert_eq!(
            BridgeProvider::from_str("synapse"),
            Some(BridgeProvider::Synapse)
        );
        assert_eq!(BridgeProvider::from_str("invalid"), None);
    }

    #[tokio::test]
    async fn test_wormhole_quote() {
        use app_lib::bridges::types::BridgeAdapter;
        use app_lib::chains::ChainId;

        let adapter = WormholeAdapter::new();
        let request = BridgeQuoteRequest {
            from_chain: ChainId::Solana,
            to_chain: ChainId::Ethereum,
            token_address: "test_token".to_string(),
            amount: 100.0,
            recipient_address: "test_recipient".to_string(),
        };

        let quote = adapter.quote(&request).await;
        assert!(quote.is_ok());
        let quote = quote.unwrap();
        assert_eq!(quote.provider, BridgeProvider::Wormhole);
        assert!(quote.amount_out < request.amount);
    }
}
