use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use parking_lot::Mutex as ParkingMutex;

const TTL_CONFIG_PATH: &str = "config/cache_ttl.json";
const DISK_CACHE_DIR: &str = "cache/disk";
const MIN_TTL_MS: u64 = 100;
const MAX_TTL_MS: u64 = 7 * 24 * 60 * 60 * 1000;

pub trait TimeProvider: Send + Sync {
    fn now(&self) -> SystemTime;
}

#[derive(Debug, Default)]
pub struct SystemTimeProvider;

impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    pub key: String,
    pub data: serde_json::Value,
    pub created_at_ms: u64,
    pub access_count: u64,
    pub last_accessed_ms: u64,
    pub size_bytes: usize,
    pub cache_type: CacheType,
    pub ttl_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum CacheType {
    TokenPrice,
    TokenInfo,
    MarketData,
    TopCoins,
    TrendingCoins,
    UserData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheTtlConfig {
    /// TTL in milliseconds for price data (fast moving)
    pub prices: u64,
    /// TTL in milliseconds for metadata (token info, lists, etc.)
    pub metadata: u64,
    /// TTL in milliseconds for history/backfill data (slow changing)
    pub history: u64,
}

impl Default for CacheTtlConfig {
    fn default() -> Self {
        Self {
            prices: 1_000,       // 1 second
            metadata: 3_600_000, // 1 hour
            history: 86_400_000, // 1 day
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStatistics {
    pub total_hits: u64,
    pub total_misses: u64,
    pub hit_rate: f64,
    pub total_evictions: u64,
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub disk_hits: u64,
    pub disk_misses: u64,
    pub warm_loads: u64,
    pub per_type_stats: HashMap<String, TypeStatistics>,
    pub last_warmed: Option<u64>,
}

impl Default for CacheStatistics {
    fn default() -> Self {
        Self {
            total_hits: 0,
            total_misses: 0,
            hit_rate: 0.0,
            total_evictions: 0,
            total_entries: 0,
            total_size_bytes: 0,
            disk_hits: 0,
            disk_misses: 0,
            warm_loads: 0,
            per_type_stats: HashMap::new(),
            last_warmed: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeStatistics {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub entries: usize,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarmProgress {
    pub total: usize,
    pub completed: usize,
    pub percentage: f64,
    pub current_key: Option<String>,
}

struct DiskCacheBackend {
    base_path: PathBuf,
    io_lock: ParkingMutex<()>,
}

impl DiskCacheBackend {
    fn new(base_path: PathBuf) -> Self {
        if let Err(err) = fs::create_dir_all(&base_path) {
            eprintln!(
                "Failed to create disk cache directory {}: {err}",
                base_path.display()
            );
        }

        Self {
            base_path,
            io_lock: ParkingMutex::new(()),
        }
    }

    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        format!("{:x}.json", hash)
    }

    fn entry_path(&self, key: &str) -> PathBuf {
        self.base_path.join(Self::hash_key(key))
    }

    fn persist(&self, entry: &CacheEntry) -> Result<(), String> {
        if !self.base_path.exists() {
            if let Err(err) = fs::create_dir_all(&self.base_path) {
                return Err(format!(
                    "Failed to create disk cache directory {}: {err}",
                    self.base_path.display()
                ));
            }
        }

        let path = self.entry_path(&entry.key);
        let data = serde_json::to_vec(entry)
            .map_err(|e| format!("Failed to serialize cache entry for disk: {e}"))?;

        let _guard = self.io_lock.lock();
        fs::write(&path, data)
            .map_err(|e| format!("Failed to write disk cache entry {}: {e}", path.display()))
    }

    fn load(&self, key: &str, now_ms: u64) -> Result<Option<CacheEntry>, String> {
        let path = self.entry_path(key);
        if !path.exists() {
            return Ok(None);
        }

        let _guard = self.io_lock.lock();
        let raw = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!("Failed to read disk cache entry {}: {err}", path.display());
                return Ok(None);
            }
        };

        let mut entry: CacheEntry = match serde_json::from_slice(&raw) {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!(
                    "Failed to deserialize disk cache entry {}: {err}",
                    path.display()
                );
                let _ = fs::remove_file(&path);
                return Ok(None);
            }
        };

        let age = now_ms.saturating_sub(entry.created_at_ms);
        if age > entry.ttl_ms {
            let _ = fs::remove_file(&path);
            return Ok(None);
        }

        entry.last_accessed_ms = now_ms;
        entry.access_count = entry.access_count.saturating_add(1);

        // Persist updated metadata (access count / last accessed)
        if let Ok(serialized) = serde_json::to_vec(&entry) {
            if let Err(err) = fs::write(&path, serialized) {
                eprintln!(
                    "Failed to update disk cache entry {}: {err}",
                    path.display()
                );
            }
        }

        Ok(Some(entry))
    }

    fn clear(&self) -> Result<(), String> {
        if !self.base_path.exists() {
            return Ok(());
        }

        let _guard = self.io_lock.lock();
        for entry in fs::read_dir(&self.base_path)
            .map_err(|e| format!("Failed to read disk cache directory: {e}"))?
        {
            let entry = entry.map_err(|e| format!("Failed to iterate disk cache entry: {e}"))?;
            if entry
                .file_type()
                .map_err(|e| format!("Failed to read file type: {e}"))?
                .is_file()
            {
                if let Err(err) = fs::remove_file(entry.path()) {
                    eprintln!(
                        "Failed to remove disk cache entry {}: {err}",
                        entry.path().display()
                    );
                }
            }
        }

        Ok(())
    }

    fn remove(&self, key: &str) {
        let path = self.entry_path(key);
        if path.exists() {
            if let Err(err) = fs::remove_file(&path) {
                eprintln!(
                    "Failed to remove disk cache entry {}: {err}",
                    path.display()
                );
            }
        }
    }

    fn purge_prefix(&self, prefix: &str) -> Result<usize, String> {
        if !self.base_path.exists() {
            return Ok(0);
        }

        let _guard = self.io_lock.lock();
        let mut removed = 0usize;

        for entry in fs::read_dir(&self.base_path)
            .map_err(|e| format!("Failed to read disk cache directory: {e}"))?
        {
            let entry = entry.map_err(|e| format!("Failed to iterate disk cache entry: {e}"))?;
            if !entry
                .file_type()
                .map_err(|e| format!("Failed to read file type: {e}"))?
                .is_file()
            {
                continue;
            }

            let path = entry.path();
            let raw = match fs::read(&path) {
                Ok(raw) => raw,
                Err(err) => {
                    eprintln!("Failed to read disk cache entry {}: {err}", path.display());
                    continue;
                }
            };

            if let Ok(cache_entry) = serde_json::from_slice::<CacheEntry>(&raw) {
                if cache_entry.key.starts_with(prefix) {
                    if let Err(err) = fs::remove_file(&path) {
                        eprintln!(
                            "Failed to remove disk cache entry {}: {err}",
                            path.display()
                        );
                    } else {
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }

    fn load_recent(&self, limit: usize, now_ms: u64) -> Result<Vec<CacheEntry>, String> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let _guard = self.io_lock.lock();
        let mut entries = Vec::new();

        for entry in fs::read_dir(&self.base_path)
            .map_err(|e| format!("Failed to read disk cache directory: {e}"))?
        {
            let entry = entry.map_err(|e| format!("Failed to iterate disk cache entry: {e}"))?;
            if !entry
                .file_type()
                .map_err(|e| format!("Failed to read file type: {e}"))?
                .is_file()
            {
                continue;
            }

            let path = entry.path();
            let raw = match fs::read(&path) {
                Ok(raw) => raw,
                Err(err) => {
                    eprintln!("Failed to read disk cache entry {}: {err}", path.display());
                    continue;
                }
            };

            match serde_json::from_slice::<CacheEntry>(&raw) {
                Ok(cache_entry) => {
                    let age = now_ms.saturating_sub(cache_entry.created_at_ms);
                    if age <= cache_entry.ttl_ms {
                        entries.push(cache_entry);
                    } else {
                        let _ = fs::remove_file(&path);
                    }
                }
                Err(err) => {
                    eprintln!(
                        "Failed to deserialize disk cache entry {}: {err}",
                        path.display()
                    );
                    let _ = fs::remove_file(&path);
                }
            }
        }

        entries.sort_by(|a, b| b.last_accessed_ms.cmp(&a.last_accessed_ms));
        entries.truncate(limit);
        Ok(entries)
    }

    fn prune_expired(&self, now_ms: u64) {
        if !self.base_path.exists() {
            return;
        }

        let _guard = self.io_lock.lock();
        for entry in match fs::read_dir(&self.base_path) {
            Ok(iter) => iter,
            Err(err) => {
                eprintln!("Failed to read disk cache directory: {err}");
                return;
            }
        } {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    eprintln!("Failed to iterate disk cache: {err}");
                    continue;
                }
            };

            let path = entry.path();
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            let raw = match fs::read(&path) {
                Ok(raw) => raw,
                Err(err) => {
                    eprintln!("Failed to read disk cache entry {}: {err}", path.display());
                    continue;
                }
            };

            if let Ok(cache_entry) = serde_json::from_slice::<CacheEntry>(&raw) {
                let age = now_ms.saturating_sub(cache_entry.created_at_ms);
                if age > cache_entry.ttl_ms {
                    let _ = fs::remove_file(&path);
                }
            } else {
                let _ = fs::remove_file(&path);
            }
        }
    }
}

pub struct CacheManager {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    stats: Arc<RwLock<CacheStatistics>>,
    ttl_config: Arc<RwLock<CacheTtlConfig>>,
    ttl_config_path: PathBuf,
    disk_cache: Arc<DiskCacheBackend>,
    max_size_bytes: usize,
    max_entries: usize,
    time_provider: Arc<dyn TimeProvider>,
}

impl CacheManager {
    pub fn new(max_size_mb: usize, max_entries: usize) -> Self {
        Self::with_time_provider_and_path(
            max_size_mb,
            max_entries,
            PathBuf::from(TTL_CONFIG_PATH),
            Arc::new(SystemTimeProvider::default()),
        )
    }

    pub fn with_time_provider_and_path(
        max_size_mb: usize,
        max_entries: usize,
        ttl_config_path: PathBuf,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
        let ttl_config = Self::load_ttl_config(ttl_config_path.as_path()).unwrap_or_else(|_| {
            let default_config = CacheTtlConfig::default();
            let _ = Self::write_ttl_config(ttl_config_path.as_path(), &default_config);
            default_config
        });

        let disk_path = ttl_config_path
            .parent()
            .map(|parent| parent.join("disk_cache"))
            .unwrap_or_else(|| PathBuf::from(DISK_CACHE_DIR));
        let disk_cache = Arc::new(DiskCacheBackend::new(disk_path));

        let manager = Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStatistics::default())),
            ttl_config: Arc::new(RwLock::new(ttl_config)),
            ttl_config_path,
            disk_cache,
            max_size_bytes: max_size_mb * 1024 * 1024,
            max_entries,
            time_provider,
        };

        manager.disk_cache.prune_expired(manager.now_ms());
        manager
    }

    fn load_ttl_config(path: &Path) -> Result<CacheTtlConfig, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        if !path.exists() {
            let defaults = CacheTtlConfig::default();
            Self::write_ttl_config(path, &defaults)?;
            return Ok(defaults);
        }

        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read TTL config: {e}"))?;
        let config: CacheTtlConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse TTL config: {e}"))?;
        Self::validate_ttl_config(&config)?;
        Ok(config)
    }

    fn write_ttl_config(path: &Path, config: &CacheTtlConfig) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize TTL config: {e}"))?;
        fs::write(path, content).map_err(|e| format!("Failed to write TTL config: {e}"))
    }

    fn validate_ttl_config(config: &CacheTtlConfig) -> Result<(), String> {
        for (name, value) in [
            ("prices", config.prices),
            ("metadata", config.metadata),
            ("history", config.history),
        ] {
            if value < MIN_TTL_MS {
                return Err(format!("TTL for {name} must be at least {MIN_TTL_MS}ms"));
            }
            if value > MAX_TTL_MS {
                return Err(format!("TTL for {name} exceeds maximum of {MAX_TTL_MS}ms"));
            }
        }
        Ok(())
    }

    fn ttl_for_type(cache_type: &CacheType, ttl_config: &CacheTtlConfig) -> u64 {
        match cache_type {
            CacheType::TokenPrice => ttl_config.prices,
            CacheType::TokenInfo
            | CacheType::MarketData
            | CacheType::TopCoins
            | CacheType::TrendingCoins => ttl_config.metadata,
            CacheType::UserData => ttl_config.history,
        }
    }

    fn now_ms(&self) -> u64 {
        self.time_provider
            .now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_millis() as u64
    }

    fn now_secs(&self) -> u64 {
        self.now_ms() / 1_000
    }

    async fn update_hit_rate(stats: &mut CacheStatistics) {
        let total = stats.total_hits + stats.total_misses;
        stats.hit_rate = if total > 0 {
            stats.total_hits as f64 / total as f64
        } else {
            0.0
        };
    }

    async fn update_type_hit_rate(type_stats: &mut TypeStatistics) {
        let total = type_stats.hits + type_stats.misses;
        type_stats.hit_rate = if total > 0 {
            type_stats.hits as f64 / total as f64
        } else {
            0.0
        };
    }

    pub async fn get_ttl_config(&self) -> CacheTtlConfig {
        let config = self.ttl_config.read().await;
        config.clone()
    }

    pub async fn update_ttl_config(&self, new_config: CacheTtlConfig) -> Result<(), String> {
        Self::validate_ttl_config(&new_config)?;
        Self::write_ttl_config(self.ttl_config_path.as_path(), &new_config)?;

        {
            let mut config = self.ttl_config.write().await;
            *config = new_config.clone();
        }

        let mut cache = self.cache.write().await;
        for entry in cache.values_mut() {
            entry.ttl_ms = Self::ttl_for_type(&entry.cache_type, &new_config);
        }

        Ok(())
    }

    pub async fn reset_ttl_config(&self) -> Result<CacheTtlConfig, String> {
        let defaults = CacheTtlConfig::default();
        self.update_ttl_config(defaults.clone()).await?;
        Ok(defaults)
    }

    pub async fn get(&self, key: &str, cache_type: CacheType) -> Option<serde_json::Value> {
        let current_time = self.now_ms();
        {
            let mut cache = self.cache.write().await;
            let mut stats = self.stats.write().await;

            if let Some(entry) = cache.get_mut(key) {
                let age = current_time.saturating_sub(entry.created_at_ms);
                if age > entry.ttl_ms {
                    let expired_entry = cache.remove(key).unwrap();
                    stats.total_size_bytes = stats
                        .total_size_bytes
                        .saturating_sub(expired_entry.size_bytes);
                    stats.total_entries = cache.len();

                    let type_key = format!("{:?}", expired_entry.cache_type);
                    if let Some(type_stats) = stats.per_type_stats.get_mut(&type_key) {
                        type_stats.entries = type_stats.entries.saturating_sub(1);
                        type_stats.size_bytes = type_stats
                            .size_bytes
                            .saturating_sub(expired_entry.size_bytes);
                    }

                    stats.total_misses = stats.total_misses.saturating_add(1);
                    let type_key = format!("{:?}", cache_type);
                    let type_stats =
                        stats
                            .per_type_stats
                            .entry(type_key)
                            .or_insert(TypeStatistics {
                                hits: 0,
                                misses: 0,
                                hit_rate: 0.0,
                                entries: 0,
                                size_bytes: 0,
                            });
                    type_stats.misses = type_stats.misses.saturating_add(1);
                    Self::update_type_hit_rate(type_stats).await;
                    Self::update_hit_rate(&mut stats).await;
                    drop(stats);
                    drop(cache);
                } else {
                    entry.access_count += 1;
                    entry.last_accessed_ms = current_time;

                    stats.total_hits = stats.total_hits.saturating_add(1);

                    let type_key = format!("{:?}", cache_type);
                    let type_stats =
                        stats
                            .per_type_stats
                            .entry(type_key.clone())
                            .or_insert(TypeStatistics {
                                hits: 0,
                                misses: 0,
                                hit_rate: 0.0,
                                entries: 0,
                                size_bytes: 0,
                            });
                    type_stats.hits = type_stats.hits.saturating_add(1);
                    Self::update_type_hit_rate(type_stats).await;
                    Self::update_hit_rate(&mut stats).await;

                    return Some(entry.data.clone());
                }
            } else {
                stats.total_misses = stats.total_misses.saturating_add(1);

                let type_key = format!("{:?}", cache_type);
                let type_stats =
                    stats
                        .per_type_stats
                        .entry(type_key.clone())
                        .or_insert(TypeStatistics {
                            hits: 0,
                            misses: 0,
                            hit_rate: 0.0,
                            entries: 0,
                            size_bytes: 0,
                        });
                type_stats.misses = type_stats.misses.saturating_add(1);
                Self::update_type_hit_rate(type_stats).await;
                Self::update_hit_rate(&mut stats).await;
            }
        }

        match self.disk_cache.load(key, current_time) {
            Ok(Some(entry)) => {
                {
                    let mut stats = self.stats.write().await;
                    stats.disk_hits = stats.disk_hits.saturating_add(1);
                }

                let value = entry.data.clone();
                if let Err(err) = self
                    .set(
                        entry.key.clone(),
                        entry.data.clone(),
                        entry.cache_type.clone(),
                    )
                    .await
                {
                    eprintln!("Failed to hydrate memory cache from disk: {err}");
                }
                Some(value)
            }
            Ok(None) => {
                let mut stats = self.stats.write().await;
                stats.disk_misses = stats.disk_misses.saturating_add(1);
                None
            }
            Err(err) => {
                eprintln!("Failed to load disk cache entry for {key}: {err}");
                let mut stats = self.stats.write().await;
                stats.disk_misses = stats.disk_misses.saturating_add(1);
                None
            }
        }
    }

    pub async fn set(
        &self,
        key: String,
        data: serde_json::Value,
        cache_type: CacheType,
    ) -> Result<(), String> {
        let ttl_ms = {
            let config = self.ttl_config.read().await;
            Self::ttl_for_type(&cache_type, &config)
        };

        let size_bytes = serde_json::to_vec(&data)
            .map_err(|e| format!("Failed to calculate cache entry size: {e}"))?
            .len();
        let current_time = self.now_ms();

        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        if stats.total_size_bytes + size_bytes > self.max_size_bytes
            || cache.len() >= self.max_entries
        {
            self.evict_lru(&mut cache, &mut stats).await;
        }

        if let Some(old_entry) = cache.get(&key) {
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(old_entry.size_bytes);

            let type_key = format!("{:?}", old_entry.cache_type);
            if let Some(type_stats) = stats.per_type_stats.get_mut(&type_key) {
                type_stats.size_bytes = type_stats.size_bytes.saturating_sub(old_entry.size_bytes);
                type_stats.entries = type_stats.entries.saturating_sub(1);
            }
        }

        let entry = CacheEntry {
            key: key.clone(),
            data,
            created_at_ms: current_time,
            access_count: 0,
            last_accessed_ms: current_time,
            size_bytes,
            cache_type: cache_type.clone(),
            ttl_ms,
        };

        cache.insert(key.clone(), entry.clone());

        stats.total_entries = cache.len();
        stats.total_size_bytes = stats.total_size_bytes.saturating_add(size_bytes);

        let type_key = format!("{:?}", cache_type);
        let type_stats = stats
            .per_type_stats
            .entry(type_key.clone())
            .or_insert(TypeStatistics {
                hits: 0,
                misses: 0,
                hit_rate: 0.0,
                entries: 0,
                size_bytes: 0,
            });
        type_stats.entries += 1;
        type_stats.size_bytes = type_stats.size_bytes.saturating_add(size_bytes);

        drop(stats);
        drop(cache);

        if let Err(err) = self.disk_cache.persist(&entry) {
            return Err(err);
        }

        Ok(())
    }

    async fn evict_lru(
        &self,
        cache: &mut HashMap<String, CacheEntry>,
        stats: &mut CacheStatistics,
    ) {
        if let Some((lru_key, lru_entry)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed_ms)
            .map(|(k, e)| (k.clone(), e.clone()))
        {
            cache.remove(&lru_key);
            stats.total_evictions = stats.total_evictions.saturating_add(1);
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(lru_entry.size_bytes);
            stats.total_entries = cache.len();

            let type_key = format!("{:?}", lru_entry.cache_type);
            if let Some(type_stats) = stats.per_type_stats.get_mut(&type_key) {
                type_stats.entries = type_stats.entries.saturating_sub(1);
                type_stats.size_bytes = type_stats.size_bytes.saturating_sub(lru_entry.size_bytes);
            }

            self.disk_cache.remove(&lru_key);
        }
    }

    pub async fn purge_keys_with_prefix(&self, prefix: &str) -> usize {
        let mut removed = 0usize;
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            if let Some(entry) = cache.remove(&key) {
                removed += 1;
                stats.total_size_bytes = stats.total_size_bytes.saturating_sub(entry.size_bytes);
                stats.total_entries = cache.len();

                let type_key = format!("{:?}", entry.cache_type);
                if let Some(type_stats) = stats.per_type_stats.get_mut(&type_key) {
                    type_stats.entries = type_stats.entries.saturating_sub(1);
                    type_stats.size_bytes = type_stats.size_bytes.saturating_sub(entry.size_bytes);
                }
            }
        }

        drop(cache);
        drop(stats);

        if let Err(err) = self.disk_cache.purge_prefix(prefix) {
            eprintln!("Failed to purge disk cache keys with prefix {prefix}: {err}");
        }

        removed
    }

    pub async fn get_statistics(&self) -> CacheStatistics {
        let stats = self.stats.read().await;
        stats.clone()
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        cache.clear();
        *stats = CacheStatistics::default();

        drop(cache);
        drop(stats);

        if let Err(err) = self.disk_cache.clear() {
            eprintln!("Failed to clear disk cache: {err}");
        }
    }

    pub async fn get_top_accessed_keys(&self, limit: usize) -> Vec<String> {
        let cache = self.cache.read().await;
        let mut entries: Vec<_> = cache.values().collect();
        entries.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        entries.iter().take(limit).map(|e| e.key.clone()).collect()
    }

    pub async fn warm_cache<F, Fut>(
        &self,
        keys: Vec<String>,
        fetcher: F,
    ) -> Result<WarmProgress, String>
    where
        F: Fn(String) -> Fut,
        Fut: std::future::Future<Output = Result<(serde_json::Value, CacheType), String>>,
    {
        let total = keys.len();
        let mut completed = 0;

        for key in keys {
            match fetcher(key.clone()).await {
                Ok((data, cache_type)) => {
                    self.set(key.clone(), data, cache_type).await?;
                    completed += 1;
                }
                Err(err) => {
                    eprintln!("Failed to warm cache for {key}: {err}");
                }
            }
        }

        let mut stats = self.stats.write().await;
        stats.last_warmed = Some(self.now_secs());
        stats.warm_loads = stats.warm_loads.saturating_add(completed as u64);

        Ok(WarmProgress {
            total,
            completed,
            percentage: if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                100.0
            },
            current_key: None,
        })
    }

    pub async fn populate_from_disk(&self, limit: usize) -> usize {
        let now = self.now_ms();
        let entries = match self.disk_cache.load_recent(limit, now) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!("Failed to load disk cache entries for warming: {err}");
                return 0;
            }
        };

        let mut warmed = 0usize;
        for entry in entries {
            if self
                .set(
                    entry.key.clone(),
                    entry.data.clone(),
                    entry.cache_type.clone(),
                )
                .await
                .is_ok()
            {
                warmed += 1;
            }
        }

        let mut stats = self.stats.write().await;
        stats.warm_loads = stats.warm_loads.saturating_add(warmed as u64);
        warmed
    }
}

pub type SharedCacheManager = Arc<RwLock<CacheManager>>;
