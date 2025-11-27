#[cfg(test)]
mod activity_log_tests {
    use chrono::{Duration, Utc};
    use eclipse_market_pro::security::activity_log::{
        ActivityLogFilter, ActivityLogger, DEFAULT_RETENTION_DAYS,
    };
    use serde_json::json;
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    async fn create_test_logger() -> (ActivityLogger, TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("activity_logs.db");
        let config_path = temp_dir.path().join("activity_log_config.json");

        let logger = ActivityLogger::new_with_paths(db_path.clone(), config_path)
            .await
            .unwrap();

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        (logger, temp_dir, db_url)
    }

    #[tokio::test]
    async fn test_log_activity() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        let details = json!({
            "method": "connect",
            "wallet_type": "phantom"
        });

        logger
            .log_connect(
                "test_wallet_address",
                details,
                true,
                Some("127.0.0.1".to_string()),
            )
            .await
            .unwrap();

        let logs = logger.get_logs(ActivityLogFilter::default()).await.unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].wallet_address, "test_wallet_address");
        assert_eq!(logs[0].action, "connect");
        assert_eq!(logs[0].result, "success");
        assert_eq!(logs[0].ip_address, Some("127.0.0.1".to_string()));
    }

    #[tokio::test]
    async fn test_log_multiple_actions() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        let wallet = "test_wallet";

        logger
            .log_connect(wallet, json!({"type": "phantom"}), true, None)
            .await
            .unwrap();

        logger
            .log_sign(wallet, json!({"message": "test message"}), true, None)
            .await
            .unwrap();

        logger
            .log_send(
                wallet,
                json!({"amount": 1.5, "recipient": "recipient_address"}),
                true,
                None,
            )
            .await
            .unwrap();

        logger
            .log_disconnect(wallet, json!({}), true, None)
            .await
            .unwrap();

        let logs = logger.get_logs(ActivityLogFilter::default()).await.unwrap();

        assert_eq!(logs.len(), 4);
        assert_eq!(logs[0].action, "disconnect");
        assert_eq!(logs[1].action, "send");
        assert_eq!(logs[2].action, "sign");
        assert_eq!(logs[3].action, "connect");
    }

    #[tokio::test]
    async fn test_filter_by_wallet() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        logger
            .log_connect("wallet1", json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_connect("wallet2", json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_connect("wallet1", json!({}), true, None)
            .await
            .unwrap();

        let filter = ActivityLogFilter {
            wallet_address: Some("wallet1".to_string()),
            ..Default::default()
        };

        let logs = logger.get_logs(filter).await.unwrap();
        assert_eq!(logs.len(), 2);
        assert!(logs.iter().all(|log| log.wallet_address == "wallet1"));
    }

    #[tokio::test]
    async fn test_filter_by_action() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        let wallet = "test_wallet";

        logger
            .log_connect(wallet, json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_sign(wallet, json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_send(wallet, json!({}), true, None)
            .await
            .unwrap();

        let filter = ActivityLogFilter {
            action: Some("sign".to_string()),
            ..Default::default()
        };

        let logs = logger.get_logs(filter).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "sign");
    }

    #[tokio::test]
    async fn test_filter_by_result() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        let wallet = "test_wallet";

        logger
            .log_connect(wallet, json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_sign(wallet, json!({}), false, None)
            .await
            .unwrap();

        logger
            .log_send(wallet, json!({}), true, None)
            .await
            .unwrap();

        let filter = ActivityLogFilter {
            result: Some("failure".to_string()),
            ..Default::default()
        };

        let logs = logger.get_logs(filter).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].action, "sign");
        assert_eq!(logs[0].result, "failure");
    }

    #[tokio::test]
    async fn test_get_stats() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        let wallet = "test_wallet";

        logger
            .log_connect(wallet, json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_sign(wallet, json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_send(wallet, json!({}), true, None)
            .await
            .unwrap();

        logger
            .log_sign(wallet, json!({}), false, None)
            .await
            .unwrap();

        let stats = logger.get_stats(None).await.unwrap();

        assert_eq!(stats.total_actions, 4);
        assert_eq!(stats.actions_today, 4);
        assert_eq!(stats.success_rate, 75.0);
        assert_eq!(stats.action_type_counts.get("connect"), Some(&1));
        assert_eq!(stats.action_type_counts.get("sign"), Some(&2));
        assert_eq!(stats.action_type_counts.get("send"), Some(&1));
    }

    #[tokio::test]
    async fn test_export_to_csv() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        logger
            .log_connect("wallet1", json!({"type": "phantom"}), true, None)
            .await
            .unwrap();

        logger
            .log_sign("wallet2", json!({"message": "test"}), false, None)
            .await
            .unwrap();

        let csv = logger
            .export_to_csv(ActivityLogFilter::default())
            .await
            .unwrap();

        assert!(csv.contains("id,wallet_address,action,details,ip_address,timestamp,result"));
        assert!(csv.contains("wallet1"));
        assert!(csv.contains("wallet2"));
        assert!(csv.contains("connect"));
        assert!(csv.contains("sign"));
        assert!(csv.contains("success"));
        assert!(csv.contains("failure"));
    }

    #[tokio::test]
    async fn test_suspicious_activity() {
        let (logger, temp_dir, db_url) = create_test_logger().await;

        let wallet = "suspicious_wallet";

        for _ in 0..6 {
            logger
                .log_connect(wallet, json!({}), true, None)
                .await
                .unwrap();
            logger
                .log_disconnect(wallet, json!({}), true, None)
                .await
                .unwrap();
        }

        for _ in 0..4 {
            logger
                .log_sign(wallet, json!({}), false, None)
                .await
                .unwrap();
        }

        let suspicious = logger.check_suspicious_activity(None).await.unwrap();

        assert!(suspicious
            .iter()
            .any(|s| s.activity_type == "rapid_connections"));
        assert!(suspicious
            .iter()
            .any(|s| s.activity_type == "failed_signatures"));

        drop(temp_dir);
        drop(db_url);
    }

    #[tokio::test]
    async fn test_cleanup_old_logs() {
        let (logger, temp_dir, db_url) = create_test_logger().await;

        logger
            .log_connect("wallet", json!({}), true, None)
            .await
            .unwrap();

        // Make the entry 10 days old
        let pool = SqlitePool::connect(&db_url).await.unwrap();
        let old_timestamp = (Utc::now() - Duration::days(10)).to_rfc3339();
        sqlx::query("UPDATE activity_logs SET timestamp = ?")
            .bind(old_timestamp)
            .execute(&pool)
            .await
            .unwrap();

        let deleted_count = logger.cleanup_old_logs(Some(5)).await.unwrap();
        assert_eq!(deleted_count, 1);

        let logs = logger.get_logs(ActivityLogFilter::default()).await.unwrap();
        assert_eq!(logs.len(), 0);

        drop(temp_dir);
        drop(db_url);
    }

    #[tokio::test]
    async fn test_retention_days_config() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        let current = logger.current_retention_days().unwrap();
        assert_eq!(current, DEFAULT_RETENTION_DAYS);

        let new_retention = logger.set_retention_days(180).unwrap();
        assert_eq!(new_retention, 180);

        let updated = logger.current_retention_days().unwrap();
        assert_eq!(updated, 180);
    }

    #[tokio::test]
    async fn test_pagination() {
        let (logger, _temp_dir, _db_url) = create_test_logger().await;

        let wallet = "test_wallet";

        for i in 0..20 {
            logger
                .log_connect(wallet, json!({"index": i}), true, None)
                .await
                .unwrap();
        }

        let page1 = logger
            .get_logs(ActivityLogFilter {
                limit: Some(10),
                offset: Some(0),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(page1.len(), 10);

        let page2 = logger
            .get_logs(ActivityLogFilter {
                limit: Some(10),
                offset: Some(10),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(page2.len(), 10);

        assert_ne!(page1[0].id, page2[0].id);
    }
}
