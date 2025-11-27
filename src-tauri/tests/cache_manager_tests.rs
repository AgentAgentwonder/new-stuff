use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Import from lib (src/core/cache_manager.rs)
use app_lib::core::cache_manager::{CacheManager, CacheTtlConfig, CacheType, TimeProvider};

#[derive(Debug)]
struct FakeTimeProvider {
    current_time: std::sync::Mutex<SystemTime>,
}

impl FakeTimeProvider {
    fn new(initial_time: SystemTime) -> Self {
        Self {
            current_time: std::sync::Mutex::new(initial_time),
        }
    }

    fn advance(&self, duration: Duration) {
        let mut time = self.current_time.lock().unwrap();
        *time = time
            .checked_add(duration)
            .expect("SystemTime overflow while advancing time");
    }

    fn set(&self, new_time: SystemTime) {
        let mut time = self.current_time.lock().unwrap();
        *time = new_time;
    }
}

impl TimeProvider for FakeTimeProvider {
    fn now(&self) -> SystemTime {
        *self.current_time.lock().unwrap()
    }
}

#[tokio::test]
async fn test_cache_hit_and_miss() {
    let epoch = UNIX_EPOCH + Duration::from_secs(1_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    let manager =
        CacheManager::with_time_provider_and_path(100, 1000, config_path, time_provider.clone());

    let key = "test_key".to_string();
    let data = json!({"price": 123.45});

    // Cache miss on first request
    let result = manager.get(&key, CacheType::TokenPrice).await;
    assert!(result.is_none());

    // Set value
    manager
        .set(key.clone(), data.clone(), CacheType::TokenPrice)
        .await
        .unwrap();

    // Cache hit on second request
    let result = manager.get(&key, CacheType::TokenPrice).await;
    assert!(result.is_some());
    assert_eq!(result.unwrap(), data);

    let stats = manager.get_statistics().await;
    assert_eq!(stats.total_hits, 1);
    assert_eq!(stats.total_misses, 1);
    assert_eq!(stats.hit_rate, 0.5);
}

#[tokio::test]
async fn test_ttl_expiration() {
    let epoch = UNIX_EPOCH + Duration::from_secs(2_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    // Create manager with short TTL for prices (1000ms)
    let manager = CacheManager::with_time_provider_and_path(
        100,
        1000,
        config_path.clone(),
        time_provider.clone(),
    );

    let key = "price_key".to_string();
    let data = json!({"price": 100.0});

    // Set value
    manager
        .set(key.clone(), data.clone(), CacheType::TokenPrice)
        .await
        .unwrap();

    // Should hit immediately
    let result = manager.get(&key, CacheType::TokenPrice).await;
    assert!(result.is_some());

    // Advance time by 500ms (still within TTL)
    time_provider.advance(Duration::from_millis(500));
    let result = manager.get(&key, CacheType::TokenPrice).await;
    assert!(result.is_some());

    // Advance time by another 600ms (total 1100ms > 1000ms TTL)
    time_provider.advance(Duration::from_millis(600));
    let result = manager.get(&key, CacheType::TokenPrice).await;
    assert!(
        result.is_none(),
        "Cache entry should have expired after TTL"
    );

    let stats = manager.get_statistics().await;
    assert_eq!(stats.total_hits, 2, "Expected 2 hits before expiration");
    assert_eq!(stats.total_misses, 1, "Expected 1 miss after expiration");
}

#[tokio::test]
async fn test_lru_eviction() {
    let epoch = UNIX_EPOCH + Duration::from_secs(3_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    // Small cache (max 3 entries)
    let manager =
        CacheManager::with_time_provider_and_path(100, 3, config_path, time_provider.clone());

    // Add 3 entries
    for i in 1..=3 {
        let key = format!("key_{i}");
        let data = json!({"value": i});
        manager.set(key, data, CacheType::TokenPrice).await.unwrap();
        time_provider.advance(Duration::from_millis(10)); // Ensure different timestamps
    }

    let stats_before = manager.get_statistics().await;
    assert_eq!(stats_before.total_entries, 3);

    // Access key_2 to make it recently used
    let _ = manager.get("key_2", CacheType::TokenPrice).await;
    time_provider.advance(Duration::from_millis(10));

    // Add 4th entry - should evict key_1 (least recently used)
    manager
        .set(
            "key_4".to_string(),
            json!({"value": 4}),
            CacheType::TokenPrice,
        )
        .await
        .unwrap();

    let stats_after = manager.get_statistics().await;
    assert_eq!(stats_after.total_entries, 3);
    assert_eq!(stats_after.total_evictions, 1);

    // key_1 should be evicted
    let result = manager.get("key_1", CacheType::TokenPrice).await;
    assert!(result.is_none(), "key_1 should have been evicted");

    // key_2, key_3, key_4 should still be present
    let result = manager.get("key_2", CacheType::TokenPrice).await;
    assert!(result.is_some(), "key_2 should still be cached");

    let result = manager.get("key_3", CacheType::TokenPrice).await;
    assert!(result.is_some(), "key_3 should still be cached");

    let result = manager.get("key_4", CacheType::TokenPrice).await;
    assert!(result.is_some(), "key_4 should still be cached");
}

#[tokio::test]
async fn test_concurrent_access() {
    use tokio::task;

    let epoch = UNIX_EPOCH + Duration::from_secs(4_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    let manager = Arc::new(tokio::sync::RwLock::new(
        CacheManager::with_time_provider_and_path(100, 1000, config_path, time_provider.clone()),
    ));

    let mut handles = vec![];

    // Spawn 10 concurrent tasks writing different keys
    for i in 0..10 {
        let mgr = manager.clone();
        let handle = task::spawn(async move {
            let key = format!("concurrent_key_{i}");
            let data = json!({"id": i});
            let mgr_read = mgr.read().await;
            mgr_read
                .set(key.clone(), data, CacheType::TokenPrice)
                .await
                .unwrap();
            drop(mgr_read);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mgr_read = manager.read().await;
    let stats = mgr_read.get_statistics().await;
    assert_eq!(stats.total_entries, 10);

    // Spawn 10 concurrent tasks reading the same key
    let mgr_read = manager.read().await;
    mgr_read
        .set(
            "shared_key".to_string(),
            json!({"shared": true}),
            CacheType::TokenPrice,
        )
        .await
        .unwrap();
    drop(mgr_read);

    let mut handles = vec![];
    for _ in 0..10 {
        let mgr = manager.clone();
        let handle = task::spawn(async move {
            let mgr_read = mgr.read().await;
            let result = mgr_read.get("shared_key", CacheType::TokenPrice).await;
            assert!(result.is_some());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let mgr_read = manager.read().await;
    let stats = mgr_read.get_statistics().await;
    assert_eq!(stats.total_hits, 10);
}

#[tokio::test]
async fn test_ttl_config_update() {
    let epoch = UNIX_EPOCH + Duration::from_secs(5_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    let manager = CacheManager::with_time_provider_and_path(
        100,
        1000,
        config_path.clone(),
        time_provider.clone(),
    );

    // Get initial config
    let initial_config = manager.get_ttl_config().await;
    assert_eq!(initial_config.prices, 1_000);
    assert_eq!(initial_config.metadata, 3_600_000);
    assert_eq!(initial_config.history, 86_400_000);

    // Update config
    let new_config = CacheTtlConfig {
        prices: 5_000,
        metadata: 10_000,
        history: 20_000,
    };
    manager.update_ttl_config(new_config.clone()).await.unwrap();

    // Verify config updated
    let updated_config = manager.get_ttl_config().await;
    assert_eq!(updated_config.prices, 5_000);
    assert_eq!(updated_config.metadata, 10_000);
    assert_eq!(updated_config.history, 20_000);

    // Verify config file was written
    let file_content = std::fs::read_to_string(&config_path).unwrap();
    let file_config: CacheTtlConfig = serde_json::from_str(&file_content).unwrap();
    assert_eq!(file_config.prices, 5_000);
}

#[tokio::test]
async fn test_ttl_config_validation() {
    let epoch = UNIX_EPOCH + Duration::from_secs(6_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    let manager =
        CacheManager::with_time_provider_and_path(100, 1000, config_path, time_provider.clone());

    // Try to set TTL below minimum (100ms)
    let invalid_config = CacheTtlConfig {
        prices: 50,
        metadata: 3_600_000,
        history: 86_400_000,
    };
    let result = manager.update_ttl_config(invalid_config).await;
    assert!(result.is_err(), "Should reject TTL below minimum");

    // Try to set TTL above maximum (7 days)
    let invalid_config = CacheTtlConfig {
        prices: 1_000,
        metadata: 3_600_000,
        history: 8 * 24 * 60 * 60 * 1000, // 8 days
    };
    let result = manager.update_ttl_config(invalid_config).await;
    assert!(result.is_err(), "Should reject TTL above maximum");
}

#[tokio::test]
async fn test_reset_ttl_config() {
    let epoch = UNIX_EPOCH + Duration::from_secs(7_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    let manager =
        CacheManager::with_time_provider_and_path(100, 1000, config_path, time_provider.clone());

    // Update to custom values
    let custom_config = CacheTtlConfig {
        prices: 5_000,
        metadata: 10_000,
        history: 20_000,
    };
    manager.update_ttl_config(custom_config).await.unwrap();

    // Reset to defaults
    let defaults = manager.reset_ttl_config().await.unwrap();
    assert_eq!(defaults.prices, 1_000);
    assert_eq!(defaults.metadata, 3_600_000);
    assert_eq!(defaults.history, 86_400_000);

    let current_config = manager.get_ttl_config().await;
    assert_eq!(current_config.prices, 1_000);
}

#[tokio::test]
async fn test_cache_clear() {
    let epoch = UNIX_EPOCH + Duration::from_secs(8_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    let manager =
        CacheManager::with_time_provider_and_path(100, 1000, config_path, time_provider.clone());

    // Add multiple entries
    for i in 1..=5 {
        let key = format!("key_{i}");
        let data = json!({"value": i});
        manager.set(key, data, CacheType::TokenPrice).await.unwrap();
    }

    let stats_before = manager.get_statistics().await;
    assert_eq!(stats_before.total_entries, 5);

    // Clear cache
    manager.clear().await;

    let stats_after = manager.get_statistics().await;
    assert_eq!(stats_after.total_entries, 0);
    assert_eq!(stats_after.total_size_bytes, 0);

    // Verify all keys are gone
    for i in 1..=5 {
        let key = format!("key_{i}");
        let result = manager.get(&key, CacheType::TokenPrice).await;
        assert!(result.is_none());
    }
}

#[tokio::test]
async fn test_cache_api_call_reduction() {
    let epoch = UNIX_EPOCH + Duration::from_secs(9_000_000);
    let time_provider = Arc::new(FakeTimeProvider::new(epoch));
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("cache_ttl.json");

    let manager = CacheManager::with_time_provider_and_path(
        100,
        1000,
        config_path.clone(),
        time_provider.clone(),
    );

    let key = "api_key".to_string();
    let data = json!({"result": "expensive_api_call"});
    let mut api_call_count = 0;

    // Simulate 10 requests
    for _ in 0..10 {
        let result = manager.get(&key, CacheType::TokenPrice).await;
        if result.is_none() {
            // Simulate API call
            api_call_count += 1;
            manager
                .set(key.clone(), data.clone(), CacheType::TokenPrice)
                .await
                .unwrap();
        }
    }

    // Should only make 1 API call (first miss)
    assert_eq!(api_call_count, 1);

    let stats = manager.get_statistics().await;
    assert_eq!(stats.total_hits, 9);
    assert_eq!(stats.total_misses, 1);

    // Verify >50% call reduction (actually 90% in this case)
    let reduction_percent = (1.0 - (api_call_count as f64 / 10.0)) * 100.0;
    assert!(
        reduction_percent > 50.0,
        "Cache should reduce API calls by more than 50%"
    );

    drop(manager);

    // Re-create manager to ensure disk persistence works
    let manager_rehydrated = CacheManager::with_time_provider_and_path(
        100,
        1000,
        config_path.clone(),
        time_provider.clone(),
    );

    // Entry should be resurrected from disk without API call
    let result = manager_rehydrated.get(&key, CacheType::TokenPrice).await;
    assert!(
        result.is_some(),
        "Expected disk-backed cache to return persisted value"
    );

    let stats_after = manager_rehydrated.get_statistics().await;
    assert_eq!(stats_after.disk_hits, 1, "Disk cache should register a hit");
}
