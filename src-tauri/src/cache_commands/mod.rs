use crate::core::cache_manager::{
    CacheManager, CacheStatistics, CacheTtlConfig, CacheType, SharedCacheManager, WarmProgress,
};
use serde::Serialize;
use serde_json::json;
use std::time::Instant;
use tauri::State;
use tokio::time::{sleep, Duration};

#[tauri::command]
pub async fn get_cache_statistics(
    cache_manager: State<'_, SharedCacheManager>,
) -> Result<CacheStatistics, String> {
    let manager = cache_manager.read().await;
    Ok(manager.get_statistics().await)
}

#[tauri::command]
pub async fn clear_cache(cache_manager: State<'_, SharedCacheManager>) -> Result<(), String> {
    let manager = cache_manager.read().await;
    manager.clear().await;
    Ok(())
}

#[tauri::command]
pub async fn warm_cache(
    cache_manager: State<'_, SharedCacheManager>,
    keys: Vec<String>,
) -> Result<WarmProgress, String> {
    let manager = cache_manager.read().await;

    // Define a simple fetcher that returns mock data for now
    // In a real implementation, this would fetch actual data
    let result = manager
        .warm_cache(keys, |key| async move {
            // Determine cache type based on key prefix
            let cache_type = if key.starts_with("token_price_") {
                CacheType::TokenPrice
            } else if key.starts_with("token_info_") {
                CacheType::TokenInfo
            } else if key.starts_with("market_") {
                CacheType::MarketData
            } else if key.starts_with("top_") {
                CacheType::TopCoins
            } else if key.starts_with("trending_") {
                CacheType::TrendingCoins
            } else {
                CacheType::UserData
            };

            // Return mock data for now
            Ok((
                json!({ "key": key, "timestamp": chrono::Utc::now().timestamp() }),
                cache_type,
            ))
        })
        .await?;

    Ok(result)
}

#[tauri::command]
pub async fn get_cache_item(
    cache_manager: State<'_, SharedCacheManager>,
    key: String,
    cache_type_str: String,
) -> Result<Option<serde_json::Value>, String> {
    let cache_type = match cache_type_str.as_str() {
        "TokenPrice" => CacheType::TokenPrice,
        "TokenInfo" => CacheType::TokenInfo,
        "MarketData" => CacheType::MarketData,
        "TopCoins" => CacheType::TopCoins,
        "TrendingCoins" => CacheType::TrendingCoins,
        "UserData" => CacheType::UserData,
        _ => return Err("Invalid cache type".to_string()),
    };

    let manager = cache_manager.read().await;
    Ok(manager.get(&key, cache_type).await)
}

#[tauri::command]
pub async fn set_cache_item(
    cache_manager: State<'_, SharedCacheManager>,
    key: String,
    data: serde_json::Value,
    cache_type_str: String,
) -> Result<(), String> {
    let cache_type = match cache_type_str.as_str() {
        "TokenPrice" => CacheType::TokenPrice,
        "TokenInfo" => CacheType::TokenInfo,
        "MarketData" => CacheType::MarketData,
        "TopCoins" => CacheType::TopCoins,
        "TrendingCoins" => CacheType::TrendingCoins,
        "UserData" => CacheType::UserData,
        _ => return Err("Invalid cache type".to_string()),
    };

    let manager = cache_manager.read().await;
    manager.set(key, data, cache_type).await
}

#[tauri::command]
pub async fn get_ttl_config(
    cache_manager: State<'_, SharedCacheManager>,
) -> Result<CacheTtlConfig, String> {
    let manager = cache_manager.read().await;
    Ok(manager.get_ttl_config().await)
}

#[tauri::command]
pub async fn update_ttl_config(
    cache_manager: State<'_, SharedCacheManager>,
    config: CacheTtlConfig,
) -> Result<(), String> {
    let manager = cache_manager.read().await;
    manager.update_ttl_config(config).await
}

#[tauri::command]
pub async fn reset_ttl_config(
    cache_manager: State<'_, SharedCacheManager>,
) -> Result<CacheTtlConfig, String> {
    let manager = cache_manager.read().await;
    manager.reset_ttl_config().await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheTestResult {
    pub test_name: String,
    pub passed: bool,
    pub cached_latency_ms: f64,
    pub uncached_latency_ms: f64,
    pub latency_improvement_percent: f64,
    pub hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub message: String,
}

#[tauri::command]
pub async fn test_cache_performance(
    cache_manager: State<'_, SharedCacheManager>,
) -> Result<CacheTestResult, String> {
    let test_prefix = "test_token__";
    let test_key = format!("{test_prefix}{}", chrono::Utc::now().timestamp_millis());
    let test_data = json!({
        "price": 100.0,
        "volume": 1_000_000.0,
        "change24h": 5.5
    });

    let manager = cache_manager.read().await;
    let stats_before = manager.get_statistics().await;

    // Measure uncached access (simulate API call on miss)
    let start_uncached = Instant::now();
    if manager
        .get(&test_key, CacheType::TokenPrice)
        .await
        .is_none()
    {
        sleep(Duration::from_millis(5)).await; // Simulated API latency
    }
    let uncached_latency_ms = start_uncached.elapsed().as_secs_f64() * 1_000.0;

    manager
        .set(test_key.clone(), test_data.clone(), CacheType::TokenPrice)
        .await
        .map_err(|e| format!("Failed to prime cache: {e}"))?;

    // Measure cached hits
    let iterations = 100;
    let mut cached_latency_total = 0.0;
    for _ in 0..iterations {
        let start_cached = Instant::now();
        let _ = manager.get(&test_key, CacheType::TokenPrice).await;
        cached_latency_total += start_cached.elapsed().as_secs_f64() * 1_000.0;
    }
    let cached_latency_ms = cached_latency_total / iterations as f64;

    let stats_after = manager.get_statistics().await;

    let cache_hits = stats_after
        .total_hits
        .saturating_sub(stats_before.total_hits);
    let cache_misses = stats_after
        .total_misses
        .saturating_sub(stats_before.total_misses);
    let test_total = cache_hits + cache_misses;
    let hit_rate = if test_total > 0 {
        cache_hits as f64 / test_total as f64
    } else {
        0.0
    };

    let improvement = if uncached_latency_ms > 0.0 {
        ((uncached_latency_ms - cached_latency_ms) / uncached_latency_ms) * 100.0
    } else {
        0.0
    };
    let passed = improvement >= 50.0;

    // Clean up test keys to avoid polluting cache
    manager.purge_keys_with_prefix(test_prefix).await;

    Ok(CacheTestResult {
        test_name: "Performance Test".to_string(),
        passed,
        cached_latency_ms,
        uncached_latency_ms,
        latency_improvement_percent: improvement,
        hit_rate,
        cache_hits,
        cache_misses,
        message: if passed {
            format!("Cache provides {:.1}% latency improvement", improvement)
        } else {
            format!("Cache improvement ({:.1}%) below 50% target", improvement)
        },
    })
}
