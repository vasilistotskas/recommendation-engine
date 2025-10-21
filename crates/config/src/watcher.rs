use crate::config::{AlgorithmConfig, CacheConfig, ColdStartConfig};
use crate::error::{ConfigError, Result};
use crate::loader::ConfigLoader;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Configuration watcher for hot-reload of non-critical configuration
pub struct ConfigWatcher {
    config_path: PathBuf,
    algorithms: Arc<RwLock<AlgorithmConfig>>,
    cache: Arc<RwLock<CacheConfig>>,
    cold_start: Arc<RwLock<ColdStartConfig>>,
    _watcher: Option<RecommendedWatcher>,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new<P: AsRef<Path>>(
        config_path: P,
        algorithms: Arc<RwLock<AlgorithmConfig>>,
        cache: Arc<RwLock<CacheConfig>>,
        cold_start: Arc<RwLock<ColdStartConfig>>,
    ) -> Result<Self> {
        let config_path = config_path.as_ref().to_path_buf();

        Ok(Self {
            config_path,
            algorithms,
            cache,
            cold_start,
            _watcher: None,
        })
    }

    /// Start watching the configuration file for changes
    pub async fn start_watching(&mut self) -> Result<()> {
        info!(
            "Starting configuration file watcher for: {:?}",
            self.config_path
        );

        let (tx, mut rx) = mpsc::channel(100);
        let config_path = self.config_path.clone();

        // Create file watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res
                    && matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| ConfigError::LoadError(format!("Failed to create file watcher: {}", e)))?;

        // Watch the configuration file
        watcher
            .watch(&self.config_path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::LoadError(format!("Failed to watch config file: {}", e)))?;

        self._watcher = Some(watcher);

        // Spawn task to handle file change events
        let algorithms = Arc::clone(&self.algorithms);
        let cache = Arc::clone(&self.cache);
        let cold_start = Arc::clone(&self.cold_start);

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                debug!("Configuration file changed: {:?}", event);

                // Reload configuration
                match Self::reload_config(&config_path, &algorithms, &cache, &cold_start).await {
                    Ok(_) => {
                        info!("Configuration reloaded successfully");
                    }
                    Err(e) => {
                        error!("Failed to reload configuration: {}", e);
                    }
                }
            }
        });

        info!("Configuration file watcher started");

        Ok(())
    }

    /// Reload configuration from file
    async fn reload_config(
        config_path: &Path,
        algorithms: &Arc<RwLock<AlgorithmConfig>>,
        cache: &Arc<RwLock<CacheConfig>>,
        cold_start: &Arc<RwLock<ColdStartConfig>>,
    ) -> Result<()> {
        debug!("Reloading configuration from: {:?}", config_path);

        // Load new configuration
        let loader = ConfigLoader::new().with_file(config_path);
        let new_config = loader.load()?;

        // Update hot-reloadable configurations
        {
            let mut algo = algorithms.write().unwrap();
            *algo = new_config.algorithms;
            debug!("Updated algorithm configuration");
        }

        {
            let mut cache_config = cache.write().unwrap();
            *cache_config = new_config.cache;
            debug!("Updated cache configuration");
        }

        {
            let mut cold_start_config = cold_start.write().unwrap();
            *cold_start_config = new_config.cold_start;
            debug!("Updated cold start configuration");
        }

        info!("Configuration hot-reload completed");

        Ok(())
    }

    /// Manually trigger a configuration reload
    pub async fn reload(&self) -> Result<()> {
        Self::reload_config(
            &self.config_path,
            &self.algorithms,
            &self.cache,
            &self.cold_start,
        )
        .await
    }

    /// Get current algorithm configuration
    pub fn get_algorithms(&self) -> AlgorithmConfig {
        self.algorithms.read().unwrap().clone()
    }

    /// Get current cache configuration
    pub fn get_cache(&self) -> CacheConfig {
        self.cache.read().unwrap().clone()
    }

    /// Get current cold start configuration
    pub fn get_cold_start(&self) -> ColdStartConfig {
        self.cold_start.read().unwrap().clone()
    }
}

/// Shared configuration holder for hot-reloadable settings
#[derive(Clone)]
#[allow(dead_code)]
pub struct HotReloadableConfig {
    pub algorithms: Arc<RwLock<AlgorithmConfig>>,
    pub cache: Arc<RwLock<CacheConfig>>,
    pub cold_start: Arc<RwLock<ColdStartConfig>>,
}

#[allow(dead_code)]
impl HotReloadableConfig {
    /// Create a new hot-reloadable configuration holder
    pub fn new(
        algorithms: AlgorithmConfig,
        cache: CacheConfig,
        cold_start: ColdStartConfig,
    ) -> Self {
        Self {
            algorithms: Arc::new(RwLock::new(algorithms)),
            cache: Arc::new(RwLock::new(cache)),
            cold_start: Arc::new(RwLock::new(cold_start)),
        }
    }

    /// Get current algorithm configuration
    pub fn get_algorithms(&self) -> AlgorithmConfig {
        self.algorithms.read().unwrap().clone()
    }

    /// Get current cache configuration
    pub fn get_cache(&self) -> CacheConfig {
        self.cache.read().unwrap().clone()
    }

    /// Get current cold start configuration
    pub fn get_cold_start(&self) -> ColdStartConfig {
        self.cold_start.read().unwrap().clone()
    }

    /// Update algorithm configuration
    pub fn update_algorithms(&self, config: AlgorithmConfig) {
        let mut algo = self.algorithms.write().unwrap();
        *algo = config;
    }

    /// Update cache configuration
    pub fn update_cache(&self, config: CacheConfig) {
        let mut cache = self.cache.write().unwrap();
        *cache = config;
    }

    /// Update cold start configuration
    pub fn update_cold_start(&self, config: ColdStartConfig) {
        let mut cold_start = self.cold_start.write().unwrap();
        *cold_start = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_hot_reloadable_config() {
        let algo = AlgorithmConfig::default();
        let cache = CacheConfig::default();
        let cold_start = ColdStartConfig::default();

        let config = HotReloadableConfig::new(algo.clone(), cache.clone(), cold_start.clone());

        // Test getters
        assert_eq!(
            config.get_algorithms().collaborative_weight,
            algo.collaborative_weight
        );
        assert_eq!(
            config.get_cache().recommendation_ttl_secs,
            cache.recommendation_ttl_secs
        );
        assert_eq!(
            config.get_cold_start().min_interactions,
            cold_start.min_interactions
        );

        // Test updates
        let mut new_algo = algo.clone();
        new_algo.collaborative_weight = 0.8;
        new_algo.content_based_weight = 0.2;
        config.update_algorithms(new_algo.clone());

        assert_eq!(config.get_algorithms().collaborative_weight, 0.8);
    }

    #[tokio::test]
    async fn test_config_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create a dummy config file
        fs::write(
            &config_path,
            "[algorithms]\ncollaborative_weight = 0.6\ncontent_based_weight = 0.4\n",
        )
        .unwrap();

        let algo = Arc::new(RwLock::new(AlgorithmConfig::default()));
        let cache = Arc::new(RwLock::new(CacheConfig::default()));
        let cold_start = Arc::new(RwLock::new(ColdStartConfig::default()));

        let watcher = ConfigWatcher::new(&config_path, algo, cache, cold_start);
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_hot_reloadable_config_clone() {
        let algo = AlgorithmConfig::default();
        let cache = CacheConfig::default();
        let cold_start = ColdStartConfig::default();

        let config1 = HotReloadableConfig::new(algo, cache, cold_start);
        let config2 = config1.clone();

        // Both should share the same underlying data
        let mut new_algo = config1.get_algorithms();
        new_algo.collaborative_weight = 0.9;
        new_algo.content_based_weight = 0.1;
        config1.update_algorithms(new_algo);

        assert_eq!(config2.get_algorithms().collaborative_weight, 0.9);
    }
}
