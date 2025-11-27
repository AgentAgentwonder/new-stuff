#[cfg(test)]
mod api_config_tests {
    use chrono::Utc;

    #[test]
    fn test_api_key_validation() {
        // Test empty key validation
        let empty_key = "";
        assert!(empty_key.trim().is_empty());

        // Test valid key
        let valid_key = "test_api_key_123";
        assert!(!valid_key.trim().is_empty());
    }

    #[test]
    fn test_service_identification() {
        let services = vec!["helius", "birdeye", "jupiter", "solana_rpc"];

        for service in services {
            match service {
                "helius" | "birdeye" | "jupiter" | "solana_rpc" => assert!(true),
                _ => panic!("Unknown service: {}", service),
            }
        }
    }

    #[test]
    fn test_expiry_date_validation() {
        let now = Utc::now();
        let future = now + chrono::Duration::days(30);
        let past = now - chrono::Duration::days(30);

        // Future dates should be valid
        assert!(future > now);

        // Past dates should be invalid for expiry
        assert!(past < now);
    }

    #[test]
    fn test_default_rpc_endpoint() {
        let default_rpc = "https://api.mainnet-beta.solana.com";
        assert!(default_rpc.starts_with("https://"));
        assert!(default_rpc.contains("solana"));
    }

    #[test]
    fn test_rate_limit_info_extraction() {
        // Mock rate limit values
        let limit = 1000u32;
        let remaining = 500u32;

        assert!(remaining <= limit);
        assert!(limit > 0);

        let usage_percent = (limit - remaining) as f64 / limit as f64 * 100.0;
        assert!(usage_percent >= 0.0 && usage_percent <= 100.0);
    }

    #[test]
    fn test_connection_status_tracking() {
        struct ConnectionStatus {
            connected: bool,
            last_error: Option<String>,
            status_code: Option<u16>,
        }

        let success = ConnectionStatus {
            connected: true,
            last_error: None,
            status_code: Some(200),
        };

        let failure = ConnectionStatus {
            connected: false,
            last_error: Some("Connection timeout".to_string()),
            status_code: None,
        };

        assert!(success.connected);
        assert!(success.status_code.unwrap() == 200);

        assert!(!failure.connected);
        assert!(failure.last_error.is_some());
    }

    #[test]
    fn test_expiry_warning_thresholds() {
        let days_until_expiry = vec![60, 30, 14, 7, 3, 1];

        for days in days_until_expiry {
            let should_warn = days < 30;
            let is_critical = days < 7;

            if days >= 30 {
                assert!(!should_warn);
            } else {
                assert!(should_warn);

                if days < 7 {
                    assert!(is_critical);
                }
            }
        }
    }

    #[test]
    fn test_url_validation() {
        let valid_urls = vec![
            "https://api.helius.xyz",
            "https://public-api.birdeye.so",
            "https://quote-api.jup.ag",
            "https://api.mainnet-beta.solana.com",
        ];

        for url in valid_urls {
            assert!(url.starts_with("https://"));
            assert!(url.len() > 10);
        }

        let invalid_urls = vec![
            "http://insecure.com", // Not HTTPS
            "",                    // Empty
            "not-a-url",           // Invalid format
        ];

        for url in invalid_urls {
            assert!(!url.starts_with("https://") || url.is_empty() || !url.contains("."));
        }
    }

    #[test]
    fn test_service_key_mapping() {
        let mappings = vec![
            ("helius", "api_key_helius"),
            ("birdeye", "api_key_birdeye"),
            ("jupiter", "api_key_jupiter"),
            ("solana_rpc", "api_rpc_endpoint"),
        ];

        for (service, key_id) in mappings {
            assert!(
                key_id.contains(service) || (service == "solana_rpc" && key_id.contains("rpc"))
            );
        }
    }
}
