#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn test_activity_action_conversion() {
        assert_eq!(ActivityAction::Buy.as_str(), "buy");
        assert_eq!(ActivityAction::Sell.as_str(), "sell");
        assert_eq!(ActivityAction::Transfer.as_str(), "transfer");
        assert_eq!(ActivityAction::Swap.as_str(), "swap");
        assert_eq!(ActivityAction::Unknown.as_str(), "unknown");

        assert_eq!(ActivityAction::from_str("buy"), ActivityAction::Buy);
        assert_eq!(ActivityAction::from_str("BUY"), ActivityAction::Buy);
        assert_eq!(ActivityAction::from_str("Sell"), ActivityAction::Sell);
        assert_eq!(ActivityAction::from_str("transfer"), ActivityAction::Transfer);
        assert_eq!(ActivityAction::from_str("random"), ActivityAction::Unknown);
    }

    #[test]
    fn test_activity_filter_empty() {
        let filter = ActivityFilter {
            wallets: None,
            tokens: None,
            actions: None,
            min_amount_usd: None,
            max_amount_usd: None,
            start_date: None,
            end_date: None,
        };

        assert!(filter.wallets.is_none());
        assert!(filter.tokens.is_none());
        assert!(filter.actions.is_none());
    }

    #[test]
    fn test_activity_filter_with_values() {
        use chrono::Utc;

        let filter = ActivityFilter {
            wallets: Some(vec!["wallet1".to_string(), "wallet2".to_string()]),
            tokens: Some(vec!["SOL".to_string()]),
            actions: Some(vec!["buy".to_string()]),
            min_amount_usd: Some(1000.0),
            max_amount_usd: Some(10000.0),
            start_date: Some(Utc::now()),
            end_date: Some(Utc::now()),
        };

        assert_eq!(filter.wallets.as_ref().unwrap().len(), 2);
        assert_eq!(filter.tokens.as_ref().unwrap().len(), 1);
        assert_eq!(filter.actions.as_ref().unwrap()[0], "buy");
        assert_eq!(filter.min_amount_usd.unwrap(), 1000.0);
    }

    #[test]
    fn test_add_monitored_wallet_request_validation() {
        let request = AddMonitoredWalletRequest {
            wallet_address: "SomeWalletAddress123".to_string(),
            label: Some("Test Whale".to_string()),
            min_transaction_size: Some(5000.0),
            is_whale: true,
        };

        assert_eq!(request.wallet_address, "SomeWalletAddress123");
        assert_eq!(request.label.unwrap(), "Test Whale");
        assert!(request.is_whale);
        assert_eq!(request.min_transaction_size.unwrap(), 5000.0);
    }

    #[test]
    fn test_update_monitored_wallet_request() {
        let request = UpdateMonitoredWalletRequest {
            id: "wallet-id-123".to_string(),
            label: Some("Updated Label".to_string()),
            min_transaction_size: Some(10000.0),
            is_whale: Some(false),
            is_active: Some(true),
        };

        assert_eq!(request.id, "wallet-id-123");
        assert_eq!(request.label.unwrap(), "Updated Label");
        assert_eq!(request.min_transaction_size.unwrap(), 10000.0);
        assert!(!request.is_whale.unwrap());
        assert!(request.is_active.unwrap());
    }

    #[test]
    fn test_wallet_statistics_computation() {
        use chrono::Utc;

        let stats = WalletStatistics {
            wallet_address: "test_wallet".to_string(),
            total_transactions: 100,
            buy_count: 60,
            sell_count: 30,
            transfer_count: 10,
            total_volume_usd: 500000.0,
            avg_transaction_size: 5000.0,
            last_activity: Some(Utc::now()),
        };

        assert_eq!(stats.total_transactions, 100);
        assert_eq!(stats.buy_count + stats.sell_count + stats.transfer_count, 100);
        assert_eq!(stats.avg_transaction_size, 5000.0);
    }

    #[test]
    fn test_copy_trade_request() {
        let request = CopyTradeRequest {
            wallet_activity_id: "activity-123".to_string(),
            wallet_address: "my-wallet".to_string(),
            multiplier: 1.5,
            delay_seconds: 10,
        };

        assert_eq!(request.multiplier, 1.5);
        assert_eq!(request.delay_seconds, 10);
        assert_eq!(request.wallet_activity_id, "activity-123");
    }
}
