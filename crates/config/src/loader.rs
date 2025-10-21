use config::{Config, Environment, File};
use std::path::Path;
use tracing::{info, warn};
use crate::config::AppConfig;
use crate::error::{ConfigError, Result};

/// Configuration loader that supports multiple sources with precedence
pub struct ConfigLoader {
    config_file_path: Option<String>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Self {
        Self {
            config_file_path: None,
        }
    }

    /// Set the configuration file path (YAML or TOML)
    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config_file_path = Some(path.as_ref().to_string_lossy().to_string());
        self
    }

    /// Load configuration with the following precedence (highest to lowest):
    /// 1. Environment variables
    /// 2. Configuration file (if provided)
    /// 3. Default values
    pub fn load(&self) -> Result<AppConfig> {
        info!("Loading configuration...");

        let mut builder = Config::builder();

        // Start with defaults
        let defaults = AppConfig::default();
        let defaults_json = serde_json::to_string(&defaults)
            .map_err(ConfigError::SerializationError)?;
        builder = builder.add_source(
            config::File::from_str(&defaults_json, config::FileFormat::Json)
        );

        // Load from configuration file if provided
        if let Some(ref path) = self.config_file_path {
            if Path::new(path).exists() {
                info!("Loading configuration from file: {}", path);
                
                // Determine file format from extension
                let format = if path.ends_with(".toml") {
                    config::FileFormat::Toml
                } else if path.ends_with(".yaml") || path.ends_with(".yml") {
                    config::FileFormat::Yaml
                } else if path.ends_with(".json") {
                    config::FileFormat::Json
                } else {
                    return Err(ConfigError::LoadError(
                        format!("Unsupported configuration file format: {}", path)
                    ));
                };

                builder = builder.add_source(
                    File::with_name(path.trim_end_matches(".toml")
                        .trim_end_matches(".yaml")
                        .trim_end_matches(".yml")
                        .trim_end_matches(".json"))
                        .format(format)
                        .required(false)
                );
            } else {
                warn!("Configuration file not found: {}", path);
            }
        }

        // Load from environment variables (highest precedence)
        // Environment variables should be prefixed with APP_ and use double underscore for nesting
        // Example: APP_SERVER__PORT=8080, APP_DATABASE__MAX_CONNECTIONS=50
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true)
        );

        // Also support legacy environment variables without prefix for backward compatibility
        builder = self.add_legacy_env_vars(builder);

        // Build the configuration
        let config = builder.build()
            .map_err(ConfigError::FileError)?;

        // Deserialize into AppConfig
        let app_config: AppConfig = config.try_deserialize()
            .map_err(ConfigError::FileError)?;

        info!("Configuration loaded successfully");

        // Validate the configuration
        app_config.validate()?;

        info!("Configuration validated successfully");

        Ok(app_config)
    }

    /// Add legacy environment variables for backward compatibility
    fn add_legacy_env_vars(&self, mut builder: config::ConfigBuilder<config::builder::DefaultState>) -> config::ConfigBuilder<config::builder::DefaultState> {
        // Server
        if let Ok(val) = std::env::var("HOST") {
            builder = builder.set_override("server.host", val).unwrap();
        }
        if let Ok(val) = std::env::var("PORT") {
            builder = builder.set_override("server.port", val).unwrap();
        }
        if let Ok(val) = std::env::var("LOG_LEVEL") {
            builder = builder.set_override("server.log_level", val).unwrap();
        }

        // Database
        if let Ok(val) = std::env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", val).unwrap();
        }
        if let Ok(val) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            builder = builder.set_override("database.max_connections", val).unwrap();
        }
        if let Ok(val) = std::env::var("DATABASE_MIN_CONNECTIONS") {
            builder = builder.set_override("database.min_connections", val).unwrap();
        }
        if let Ok(val) = std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS") {
            builder = builder.set_override("database.acquire_timeout_secs", val).unwrap();
        }
        if let Ok(val) = std::env::var("DATABASE_IDLE_TIMEOUT_SECS") {
            builder = builder.set_override("database.idle_timeout_secs", val).unwrap();
        }
        if let Ok(val) = std::env::var("DATABASE_MAX_LIFETIME_SECS") {
            builder = builder.set_override("database.max_lifetime_secs", val).unwrap();
        }

        // Redis
        if let Ok(val) = std::env::var("REDIS_URL") {
            builder = builder.set_override("redis.url", val).unwrap();
        }
        if let Ok(val) = std::env::var("REDIS_POOL_SIZE") {
            builder = builder.set_override("redis.pool_size", val).unwrap();
        }
        if let Ok(val) = std::env::var("REDIS_CONNECTION_TIMEOUT_SECS") {
            builder = builder.set_override("redis.connection_timeout_secs", val).unwrap();
        }

        // Algorithms
        if let Ok(val) = std::env::var("COLLABORATIVE_WEIGHT") {
            builder = builder.set_override("algorithms.collaborative_weight", val).unwrap();
        }
        if let Ok(val) = std::env::var("CONTENT_BASED_WEIGHT") {
            builder = builder.set_override("algorithms.content_based_weight", val).unwrap();
        }
        if let Ok(val) = std::env::var("SIMILARITY_THRESHOLD") {
            builder = builder.set_override("algorithms.similarity_threshold", val).unwrap();
        }
        if let Ok(val) = std::env::var("DEFAULT_RECOMMENDATION_COUNT") {
            builder = builder.set_override("algorithms.default_recommendation_count", val).unwrap();
        }
        if let Ok(val) = std::env::var("MAX_RECOMMENDATION_COUNT") {
            builder = builder.set_override("algorithms.max_recommendation_count", val).unwrap();
        }

        // Model Updates
        if let Ok(val) = std::env::var("INCREMENTAL_UPDATE_INTERVAL_SECS") {
            builder = builder.set_override("model_updates.incremental_update_interval_secs", val).unwrap();
        }
        if let Ok(val) = std::env::var("FULL_REBUILD_INTERVAL_HOURS") {
            builder = builder.set_override("model_updates.full_rebuild_interval_hours", val).unwrap();
        }
        if let Ok(val) = std::env::var("TRENDING_UPDATE_INTERVAL_HOURS") {
            builder = builder.set_override("model_updates.trending_update_interval_hours", val).unwrap();
        }
        if let Ok(val) = std::env::var("MODEL_UPDATE_BATCH_SIZE") {
            builder = builder.set_override("model_updates.batch_size", val).unwrap();
        }

        // Cache
        if let Ok(val) = std::env::var("RECOMMENDATION_CACHE_TTL_SECS") {
            builder = builder.set_override("cache.recommendation_ttl_secs", val).unwrap();
        }
        if let Ok(val) = std::env::var("TRENDING_CACHE_TTL_SECS") {
            builder = builder.set_override("cache.trending_ttl_secs", val).unwrap();
        }
        if let Ok(val) = std::env::var("USER_PREFERENCE_CACHE_TTL_SECS") {
            builder = builder.set_override("cache.user_preference_ttl_secs", val).unwrap();
        }
        if let Ok(val) = std::env::var("ENTITY_FEATURE_CACHE_TTL_SECS") {
            builder = builder.set_override("cache.entity_feature_ttl_secs", val).unwrap();
        }

        // Authentication
        if let Ok(val) = std::env::var("API_KEY") {
            builder = builder.set_override("authentication.api_key", val).unwrap();
        }
        if let Ok(val) = std::env::var("REQUIRE_API_KEY") {
            builder = builder.set_override("authentication.require_api_key", val).unwrap();
        }

        // Rate Limiting
        if let Ok(val) = std::env::var("RATE_LIMIT_ENABLED") {
            builder = builder.set_override("rate_limiting.enabled", val).unwrap();
        }
        if let Ok(val) = std::env::var("RATE_LIMIT_REQUESTS_PER_MINUTE") {
            builder = builder.set_override("rate_limiting.requests_per_minute", val).unwrap();
        }
        if let Ok(val) = std::env::var("RATE_LIMIT_BURST_SIZE") {
            builder = builder.set_override("rate_limiting.burst_size", val).unwrap();
        }

        // Observability
        if let Ok(val) = std::env::var("METRICS_ENABLED") {
            builder = builder.set_override("observability.metrics_enabled", val).unwrap();
        }
        if let Ok(val) = std::env::var("METRICS_PORT") {
            builder = builder.set_override("observability.metrics_port", val).unwrap();
        }
        if let Ok(val) = std::env::var("TRACING_ENABLED") {
            builder = builder.set_override("observability.tracing_enabled", val).unwrap();
        }
        if let Ok(val) = std::env::var("TRACING_ENDPOINT") {
            builder = builder.set_override("observability.tracing_endpoint", val).unwrap();
        }

        // Cold Start
        if let Ok(val) = std::env::var("COLD_START_MIN_INTERACTIONS") {
            builder = builder.set_override("cold_start.min_interactions", val).unwrap();
        }
        if let Ok(val) = std::env::var("TRENDING_WINDOW_DAYS") {
            builder = builder.set_override("cold_start.trending_window_days", val).unwrap();
        }
        if let Ok(val) = std::env::var("POPULAR_ENTITIES_COUNT") {
            builder = builder.set_override("cold_start.popular_entities_count", val).unwrap();
        }

        // Multi-Tenancy
        if let Ok(val) = std::env::var("DEFAULT_TENANT_ID") {
            builder = builder.set_override("multi_tenancy.default_tenant_id", val).unwrap();
        }
        if let Ok(val) = std::env::var("ENABLE_MULTI_TENANCY") {
            builder = builder.set_override("multi_tenancy.enabled", val).unwrap();
        }

        // Webhooks
        if let Ok(val) = std::env::var("WEBHOOK_ENABLED") {
            builder = builder.set_override("webhooks.enabled", val).unwrap();
        }
        if let Ok(val) = std::env::var("WEBHOOK_URL") {
            builder = builder.set_override("webhooks.url", val).unwrap();
        }
        if let Ok(val) = std::env::var("WEBHOOK_SECRET") {
            builder = builder.set_override("webhooks.secret", val).unwrap();
        }
        if let Ok(val) = std::env::var("WEBHOOK_RETRY_MAX_ATTEMPTS") {
            builder = builder.set_override("webhooks.retry_max_attempts", val).unwrap();
        }
        if let Ok(val) = std::env::var("WEBHOOK_RETRY_BACKOFF_SECS") {
            builder = builder.set_override("webhooks.retry_backoff_secs", val).unwrap();
        }

        // Bulk Operations
        if let Ok(val) = std::env::var("BULK_IMPORT_BATCH_SIZE") {
            builder = builder.set_override("bulk_operations.import_batch_size", val).unwrap();
        }
        if let Ok(val) = std::env::var("BULK_IMPORT_MAX_RECORDS") {
            builder = builder.set_override("bulk_operations.import_max_records", val).unwrap();
        }
        if let Ok(val) = std::env::var("BULK_EXPORT_BATCH_SIZE") {
            builder = builder.set_override("bulk_operations.export_batch_size", val).unwrap();
        }

        // Performance
        if let Ok(val) = std::env::var("WORKER_THREADS") {
            builder = builder.set_override("performance.worker_threads", val).unwrap();
        }
        if let Ok(val) = std::env::var("MAX_BLOCKING_THREADS") {
            builder = builder.set_override("performance.max_blocking_threads", val).unwrap();
        }
        if let Ok(val) = std::env::var("ENABLE_CONNECTION_POOLING") {
            builder = builder.set_override("performance.enable_connection_pooling", val).unwrap();
        }

        // Security
        if let Ok(val) = std::env::var("ENABLE_CORS") {
            builder = builder.set_override("security.enable_cors", val).unwrap();
        }
        if let Ok(val) = std::env::var("CORS_ALLOWED_ORIGINS") {
            builder = builder.set_override("security.cors_allowed_origins", val).unwrap();
        }
        if let Ok(val) = std::env::var("ENABLE_COMPRESSION") {
            builder = builder.set_override("security.enable_compression", val).unwrap();
        }
        if let Ok(val) = std::env::var("MAX_REQUEST_SIZE_MB") {
            builder = builder.set_override("security.max_request_size_mb", val).unwrap();
        }

        // Shutdown
        if let Ok(val) = std::env::var("SHUTDOWN_TIMEOUT_SECS") {
            builder = builder.set_override("shutdown.timeout_secs", val).unwrap();
        }
        if let Ok(val) = std::env::var("DRAIN_REQUESTS_ON_SHUTDOWN") {
            builder = builder.set_override("shutdown.drain_requests", val).unwrap();
        }

        builder
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Use a mutex to ensure tests run sequentially to avoid env var conflicts
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_load_default_config() {
        let _guard = TEST_MUTEX.lock().unwrap();
        
        // Clear any existing env vars that might interfere
        unsafe {
            env::remove_var("PORT");
            env::remove_var("DATABASE_MAX_CONNECTIONS");
            env::remove_var("COLLABORATIVE_WEIGHT");
            env::remove_var("CONTENT_BASED_WEIGHT");
        }
        
        let loader = ConfigLoader::new();
        let config = loader.load().unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 20);
    }

    #[test]
    fn test_load_with_env_vars() {
        let _guard = TEST_MUTEX.lock().unwrap();
        
        // Clear any existing env vars
        unsafe {
            env::remove_var("COLLABORATIVE_WEIGHT");
            env::remove_var("CONTENT_BASED_WEIGHT");
        }
        
        unsafe {
            env::set_var("PORT", "9090");
            env::set_var("DATABASE_MAX_CONNECTIONS", "50");
        }
        
        let loader = ConfigLoader::new();
        let config = loader.load().unwrap();
        
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.database.max_connections, 50);
        
        // Cleanup
        unsafe {
            env::remove_var("PORT");
            env::remove_var("DATABASE_MAX_CONNECTIONS");
        }
    }

    #[test]
    fn test_validation_fails_on_invalid_config() {
        let _guard = TEST_MUTEX.lock().unwrap();
        
        // Clear any existing env vars
        unsafe {
            env::remove_var("PORT");
            env::remove_var("DATABASE_MAX_CONNECTIONS");
        }
        
        unsafe {
            env::set_var("COLLABORATIVE_WEIGHT", "0.7");
            env::set_var("CONTENT_BASED_WEIGHT", "0.5");
        }
        
        let loader = ConfigLoader::new();
        let result = loader.load();
        
        assert!(result.is_err());
        
        // Cleanup
        unsafe {
            env::remove_var("COLLABORATIVE_WEIGHT");
            env::remove_var("CONTENT_BASED_WEIGHT");
        }
    }
}
