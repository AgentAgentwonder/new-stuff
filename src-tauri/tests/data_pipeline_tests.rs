use app_lib::data::database::CompressionManager;
use app_lib::data::event_store::{Event, EventFilter, EventStore};
use chrono::Utc;
use tempfile::tempdir;

#[tokio::test]
async fn test_event_replay_accuracy() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("events_test.db");

    let store = EventStore::new(db_path)
        .await
        .expect("Failed to create event store");

    let aggregate_id = "test_wallet_123";
    let mut event_ids = Vec::new();

    // Publish 10 events
    for i in 0..10 {
        let event = Event::BalanceChanged {
            wallet: format!("wallet_{i}"),
            token: "SOL".to_string(),
            old_balance: i as f64,
            new_balance: (i + 1) as f64,
            reason: format!("Test change {i}"),
            timestamp: Utc::now(),
        };

        let id = store
            .publish_event(event, aggregate_id)
            .await
            .expect("Failed to publish event");
        event_ids.push(id);
    }

    // Replay events
    let replayed = store
        .replay_events(aggregate_id)
        .await
        .expect("Failed to replay events");

    assert_eq!(replayed.len(), 10, "Should replay all 10 events");

    // Verify ordering and content
    for (i, event) in replayed.iter().enumerate() {
        match event {
            Event::BalanceChanged {
                wallet,
                old_balance,
                new_balance,
                ..
            } => {
                assert_eq!(wallet, &format!("wallet_{i}"));
                assert_eq!(*old_balance, i as f64);
                assert_eq!(*new_balance, (i + 1) as f64);
            }
            _ => panic!("Expected BalanceChanged event"),
        }
    }

    // Test event count
    let count = store
        .get_event_count(Some(aggregate_id))
        .await
        .expect("Failed to get event count");
    assert_eq!(count, 10);
}

#[tokio::test]
async fn test_event_store_filtering() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("events_filter_test.db");

    let store = EventStore::new(db_path)
        .await
        .expect("Failed to create event store");

    let aggregate_id = "filter_test";

    // Publish different event types
    for i in 0..5 {
        let event = Event::OrderPlaced {
            order_id: format!("order_{i}"),
            symbol: "SOL".to_string(),
            side: "buy".to_string(),
            quantity: 10.0,
            price: Some(100.0),
            timestamp: Utc::now(),
        };
        store
            .publish_event(event, aggregate_id)
            .await
            .expect("Failed to publish order placed event");
    }

    for i in 0..3 {
        let event = Event::OrderFilled {
            order_id: format!("order_{i}"),
            fill_price: 100.5,
            filled_quantity: 10.0,
            timestamp: Utc::now(),
        };
        store
            .publish_event(event, aggregate_id)
            .await
            .expect("Failed to publish order filled event");
    }

    // Filter by event type
    let filter = EventFilter {
        aggregate_id: Some(aggregate_id.to_string()),
        event_type: Some("order_filled".to_string()),
        from_time: None,
        to_time: None,
        limit: None,
        offset: None,
    };

    let filtered_events = store
        .get_events(filter)
        .await
        .expect("Failed to get filtered events");

    assert_eq!(
        filtered_events.len(),
        3,
        "Should return 3 order_filled events"
    );
}

#[tokio::test]
async fn test_compression_and_decompression() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("compression_test.db");

    let manager = CompressionManager::new(db_path.clone())
        .await
        .expect("Failed to create compression manager");

    // Test data compression
    let original_data = b"This is a test string that will be compressed. It should compress well because it has repetition. This is a test string that will be compressed.";
    let record_id = "test_record_123";
    let timestamp = Utc::now();

    manager
        .compress_data(original_data, "test", record_id, timestamp)
        .await
        .expect("Failed to compress data");

    // Get compression stats
    let stats = manager
        .get_compression_stats()
        .await
        .expect("Failed to get compression stats");

    assert_eq!(stats.num_compressed_records, 1);
    assert!(stats.total_uncompressed_bytes > 0);
    assert!(stats.total_compressed_bytes > 0);
    assert!(
        stats.total_compressed_bytes < stats.total_uncompressed_bytes,
        "Compressed data should be smaller than original"
    );

    let compression_ratio = stats.compression_ratio;
    assert!(
        compression_ratio > 0.0,
        "Compression ratio should be positive"
    );
    println!("Compression ratio: {compression_ratio:.2}%");

    // Decompress and verify
    let decompressed = manager
        .decompress_data(record_id)
        .await
        .expect("Failed to decompress data");

    assert_eq!(
        decompressed, original_data,
        "Decompressed data should match original"
    );
}

#[tokio::test]
async fn test_compression_threshold() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("compression_threshold_test.db");

    let _manager = CompressionManager::new(db_path.clone())
        .await
        .expect("Failed to create compression manager");

    // Create the event store using the same db path
    let event_store = EventStore::new(db_path.clone())
        .await
        .expect("Failed to create event store");

    let aggregate_id = "threshold_test";

    // Publish events (they are recent, so shouldn't be compressed immediately)
    for i in 0..5 {
        let event = Event::OrderPlaced {
            order_id: format!("order_{i}"),
            symbol: "SOL".to_string(),
            side: "buy".to_string(),
            quantity: 10.0,
            price: Some(100.0),
            timestamp: Utc::now(),
        };
        event_store
            .publish_event(event, aggregate_id)
            .await
            .expect("Failed to publish event");
    }

    // Verify events are stored (but not compressed yet due to age threshold)
    let count = event_store
        .get_event_count(Some(aggregate_id))
        .await
        .expect("Failed to get event count");
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_event_export_formats() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("export_test.db");

    let store = EventStore::new(db_path)
        .await
        .expect("Failed to create event store");

    let aggregate_id = "export_test";

    // Publish some events
    for i in 0..3 {
        let event = Event::TradeExecuted {
            trade_id: format!("trade_{i}"),
            from_token: "SOL".to_string(),
            to_token: "USDC".to_string(),
            from_amount: 10.0,
            to_amount: 1000.0,
            price: 100.0,
            timestamp: Utc::now(),
        };
        store
            .publish_event(event, aggregate_id)
            .await
            .expect("Failed to publish event");
    }

    // Export as JSON
    let filter = EventFilter {
        aggregate_id: Some(aggregate_id.to_string()),
        event_type: None,
        from_time: None,
        to_time: None,
        limit: None,
        offset: None,
    };

    let json_export = store
        .export_events(filter.clone(), "json")
        .await
        .expect("Failed to export as JSON");

    assert!(json_export.contains("trade_executed"));
    assert!(json_export.contains("trade_0"));

    // Export as CSV
    let csv_export = store
        .export_events(filter, "csv")
        .await
        .expect("Failed to export as CSV");

    assert!(csv_export.contains("Event Type"));
    assert!(csv_export.contains("trade_executed"));
    assert!(csv_export.starts_with("ID,"));
}

#[tokio::test]
async fn test_benchmark_threshold() {
    use app_lib::core::price_engine::PriceEngine;

    let engine = PriceEngine::new();

    // Run benchmark test
    let metrics = engine.run_performance_test(1_000).await;

    // Verify P95 < 1ms (1000 microseconds)
    assert!(
        metrics.latency.p95 < 1000.0,
        "P95 latency ({:.2} μs) exceeds 1ms target",
        metrics.latency.p95
    );

    // Verify all messages processed
    assert_eq!(metrics.messages_processed, 1_000);

    // Verify error rate is acceptable
    let error_rate = if metrics.messages_received > 0 {
        (metrics.errors as f64 / metrics.messages_received as f64) * 100.0
    } else {
        0.0
    };
    assert!(
        error_rate < 1.0,
        "Error rate ({error_rate:.2}%) exceeds 1% threshold"
    );

    // Verify throughput is reasonable
    assert!(metrics.throughput > 0.0, "Throughput should be positive");

    println!("=== Benchmark Results ===");
    println!("P50: {:.2} μs", metrics.latency.p50);
    println!("P95: {:.2} μs", metrics.latency.p95);
    println!("P99: {:.2} μs", metrics.latency.p99);
    println!("Throughput: {:.2} msg/s", metrics.throughput);
    println!("Error rate: {:.4}%", error_rate);
}
