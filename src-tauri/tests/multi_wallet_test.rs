#[cfg(test)]
mod multi_wallet_tests {
    use eclipse_market_pro::security::keystore::Keystore;
    use eclipse_market_pro::wallet::multi_wallet::{
        AddWalletRequest, CreateGroupRequest, GroupSettings, MultiWalletManager, WalletType,
    };
    use tauri::{App, Manager};
    use tempfile::tempdir;

    fn create_test_keystore() -> Keystore {
        let temp = tempdir().expect("Failed to create temp directory");
        let handle = create_mock_app_handle(temp.path().to_str().unwrap());
        Keystore::initialize(&handle).expect("Failed to initialize keystore")
    }

    fn create_mock_app_handle(temp_path: &str) -> tauri::AppHandle {
        unimplemented!("Mock app handle for testing")
    }

    #[test]
    fn test_add_wallet() {
        let keystore = create_test_keystore();
        let manager = MultiWalletManager::initialize(&keystore).expect("Failed to init manager");

        let request = AddWalletRequest {
            public_key: "TestPublicKey123".to_string(),
            label: "Test Wallet".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        let result = manager.add_wallet(request, &keystore);
        assert!(result.is_ok());

        let wallet = result.unwrap();
        assert_eq!(wallet.label, "Test Wallet");
        assert_eq!(wallet.public_key, "TestPublicKey123");
    }

    #[test]
    fn test_add_duplicate_wallet() {
        let keystore = create_test_keystore();
        let manager = MultiWalletManager::initialize(&keystore).expect("Failed to init manager");

        let request = AddWalletRequest {
            public_key: "TestPublicKey123".to_string(),
            label: "Test Wallet".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        manager
            .add_wallet(request.clone(), &keystore)
            .expect("First add should succeed");

        let result = manager.add_wallet(request, &keystore);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_active_wallet() {
        let keystore = create_test_keystore();
        let manager = MultiWalletManager::initialize(&keystore).expect("Failed to init manager");

        let request1 = AddWalletRequest {
            public_key: "Key1".to_string(),
            label: "Wallet 1".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        let request2 = AddWalletRequest {
            public_key: "Key2".to_string(),
            label: "Wallet 2".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        let wallet1 = manager
            .add_wallet(request1, &keystore)
            .expect("Add wallet 1");
        let wallet2 = manager
            .add_wallet(request2, &keystore)
            .expect("Add wallet 2");

        let result = manager.set_active_wallet(&wallet2.id, &keystore);
        assert!(result.is_ok());

        let active = manager.get_active_wallet().expect("Get active");
        assert_eq!(active.unwrap().id, wallet2.id);
    }

    #[test]
    fn test_create_group() {
        let keystore = create_test_keystore();
        let manager = MultiWalletManager::initialize(&keystore).expect("Failed to init manager");

        let wallet_request = AddWalletRequest {
            public_key: "Key1".to_string(),
            label: "Wallet 1".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        let wallet = manager
            .add_wallet(wallet_request, &keystore)
            .expect("Add wallet");

        let group_request = CreateGroupRequest {
            name: "Test Group".to_string(),
            description: Some("A test group".to_string()),
            wallet_ids: vec![wallet.id.clone()],
            shared_settings: Some(GroupSettings {
                max_slippage: Some(1.5),
                default_priority_fee: Some(5000),
                risk_level: Some("balanced".to_string()),
                auto_rebalance: true,
            }),
        };

        let result = manager.create_group(group_request, &keystore);
        assert!(result.is_ok());

        let group = result.unwrap();
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.wallet_ids.len(), 1);
    }

    #[test]
    fn test_remove_wallet() {
        let keystore = create_test_keystore();
        let manager = MultiWalletManager::initialize(&keystore).expect("Failed to init manager");

        let request = AddWalletRequest {
            public_key: "TestKey".to_string(),
            label: "Test Wallet".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        let wallet = manager.add_wallet(request, &keystore).expect("Add wallet");
        let result = manager.remove_wallet(&wallet.id, &keystore);
        assert!(result.is_ok());

        let wallets = manager.list_wallets().expect("List wallets");
        assert_eq!(wallets.len(), 0);
    }

    #[test]
    fn test_wallet_isolation_mode() {
        let keystore = create_test_keystore();
        let manager = MultiWalletManager::initialize(&keystore).expect("Failed to init manager");

        let request = AddWalletRequest {
            public_key: "IsolationKey".to_string(),
            label: "Isolation Wallet".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        let wallet = manager.add_wallet(request, &keystore).expect("Add wallet");
        assert_eq!(wallet.preferences.isolation_mode, false);
        assert_eq!(wallet.preferences.trading_enabled, true);
    }

    #[test]
    fn test_aggregated_portfolio() {
        let keystore = create_test_keystore();
        let manager = MultiWalletManager::initialize(&keystore).expect("Failed to init manager");

        let request1 = AddWalletRequest {
            public_key: "Key1".to_string(),
            label: "Wallet 1".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        let request2 = AddWalletRequest {
            public_key: "Key2".to_string(),
            label: "Wallet 2".to_string(),
            network: "devnet".to_string(),
            wallet_type: WalletType::Phantom,
            group_id: None,
        };

        manager
            .add_wallet(request1, &keystore)
            .expect("Add wallet 1");
        manager
            .add_wallet(request2, &keystore)
            .expect("Add wallet 2");

        let portfolio = manager
            .get_aggregated_portfolio()
            .expect("Get aggregated portfolio");

        assert_eq!(portfolio.total_wallets, 2);
        assert_eq!(portfolio.total_balance, 0.0);
    }
}
