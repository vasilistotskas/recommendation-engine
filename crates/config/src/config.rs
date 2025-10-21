use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub algorithms: AlgorithmConfig,
    pub model_updates: ModelUpdateConfig,
    pub cache: CacheConfig,
    pub authentication: AuthConfig,
    pub rate_limiting: RateLimitConfig,
    pub observability: ObservabilityConfig,
    pub cold_start: ColdStartConfig,
    pub multi_tenancy: MultiTenancyConfig,
    pub webhooks: WebhookConfig,
    pub bulk_operations: BulkOperationsConfig,
    pub performance: PerformanceConfig,
    pub security: SecurityConfig,
    pub shutdown: ShutdownConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout_secs: u64,
}

/// Algorithm configuration (hot-reloadable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    pub collaborative_weight: f32,
    pub content_based_weight: f32,
    pub similarity_threshold: f32,
    pub default_recommendation_count: usize,
    pub max_recommendation_count: usize,
}

/// Feature vector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVectorConfig {
    pub dimension: usize,
    pub use_tfidf_for_text: bool,
    pub normalize_numerical_features: bool,
}

/// Model update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdateConfig {
    pub incremental_update_interval_secs: u64,
    pub full_rebuild_interval_hours: u64,
    pub trending_update_interval_hours: u64,
    pub batch_size: usize,
}

/// Cache configuration (hot-reloadable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub recommendation_ttl_secs: u64,
    pub trending_ttl_secs: u64,
    pub user_preference_ttl_secs: u64,
    pub entity_feature_ttl_secs: u64,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub api_key: String,
    pub require_api_key: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
    pub tracing_endpoint: Option<String>,
}

/// Cold start configuration (hot-reloadable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColdStartConfig {
    pub min_interactions: usize,
    pub trending_window_days: u64,
    pub popular_entities_count: usize,
}

/// Multi-tenancy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTenancyConfig {
    pub default_tenant_id: String,
    pub enabled: bool,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: Option<String>,
    pub secret: Option<String>,
    pub retry_max_attempts: u32,
    pub retry_backoff_secs: u64,
}

/// Bulk operations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationsConfig {
    pub import_batch_size: usize,
    pub import_max_records: usize,
    pub export_batch_size: usize,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_threads: usize,
    pub max_blocking_threads: usize,
    pub enable_connection_pooling: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_cors: bool,
    pub cors_allowed_origins: String,
    pub enable_compression: bool,
    pub max_request_size_mb: usize,
}

/// Graceful shutdown configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownConfig {
    pub timeout_secs: u64,
    pub drain_requests: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            log_level: "info".to_string(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://postgres:postgres@localhost:5432/recommendations".to_string(),
            max_connections: 20,
            min_connections: 5,
            acquire_timeout_secs: 3,
            idle_timeout_secs: 600,
            max_lifetime_secs: 1800,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            pool_size: 10,
            connection_timeout_secs: 5,
        }
    }
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            collaborative_weight: 0.6,
            content_based_weight: 0.4,
            similarity_threshold: 0.5,
            default_recommendation_count: 10,
            max_recommendation_count: 100,
        }
    }
}

impl Default for FeatureVectorConfig {
    fn default() -> Self {
        Self {
            dimension: 512,
            use_tfidf_for_text: true,
            normalize_numerical_features: true,
        }
    }
}

impl Default for ModelUpdateConfig {
    fn default() -> Self {
        Self {
            incremental_update_interval_secs: 10,
            full_rebuild_interval_hours: 24,
            trending_update_interval_hours: 1,
            batch_size: 1000,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            recommendation_ttl_secs: 300,
            trending_ttl_secs: 3600,
            user_preference_ttl_secs: 600,
            entity_feature_ttl_secs: 3600,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key: "your-secret-api-key-change-in-production".to_string(),
            require_api_key: true,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 1000,
            burst_size: 100,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_port: 9090,
            tracing_enabled: true,
            tracing_endpoint: Some("http://localhost:4317".to_string()),
        }
    }
}

impl Default for ColdStartConfig {
    fn default() -> Self {
        Self {
            min_interactions: 5,
            trending_window_days: 7,
            popular_entities_count: 100,
        }
    }
}

impl Default for MultiTenancyConfig {
    fn default() -> Self {
        Self {
            default_tenant_id: "default".to_string(),
            enabled: false,
        }
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: None,
            secret: None,
            retry_max_attempts: 3,
            retry_backoff_secs: 5,
        }
    }
}

impl Default for BulkOperationsConfig {
    fn default() -> Self {
        Self {
            import_batch_size: 1000,
            import_max_records: 100000,
            export_batch_size: 5000,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: 4,
            max_blocking_threads: 512,
            enable_connection_pooling: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            cors_allowed_origins: "*".to_string(),
            enable_compression: true,
            max_request_size_mb: 10,
        }
    }
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            drain_requests: true,
        }
    }
}

impl AppConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port must be greater than 0".to_string(),
            ));
        }

        // Validate database configuration
        if self.database.max_connections == 0 {
            return Err(ConfigError::ValidationError(
                "Database max_connections must be greater than 0".to_string(),
            ));
        }
        if self.database.min_connections > self.database.max_connections {
            return Err(ConfigError::ValidationError(
                "Database min_connections cannot exceed max_connections".to_string(),
            ));
        }

        // Validate Redis configuration
        if self.redis.pool_size == 0 {
            return Err(ConfigError::ValidationError(
                "Redis pool_size must be greater than 0".to_string(),
            ));
        }

        // Validate algorithm weights
        let weight_sum =
            self.algorithms.collaborative_weight + self.algorithms.content_based_weight;
        if (weight_sum - 1.0).abs() > 0.001 {
            return Err(ConfigError::ValidationError(format!(
                "Algorithm weights must sum to 1.0, got {}",
                weight_sum
            )));
        }
        if self.algorithms.collaborative_weight < 0.0 || self.algorithms.collaborative_weight > 1.0
        {
            return Err(ConfigError::ValidationError(
                "Collaborative weight must be between 0.0 and 1.0".to_string(),
            ));
        }
        if self.algorithms.content_based_weight < 0.0 || self.algorithms.content_based_weight > 1.0
        {
            return Err(ConfigError::ValidationError(
                "Content-based weight must be between 0.0 and 1.0".to_string(),
            ));
        }
        if self.algorithms.similarity_threshold < 0.0 || self.algorithms.similarity_threshold > 1.0
        {
            return Err(ConfigError::ValidationError(
                "Similarity threshold must be between 0.0 and 1.0".to_string(),
            ));
        }
        if self.algorithms.default_recommendation_count == 0 {
            return Err(ConfigError::ValidationError(
                "Default recommendation count must be greater than 0".to_string(),
            ));
        }
        if self.algorithms.default_recommendation_count > self.algorithms.max_recommendation_count {
            return Err(ConfigError::ValidationError(
                "Default recommendation count cannot exceed max recommendation count".to_string(),
            ));
        }

        // Validate cache TTLs
        if self.cache.recommendation_ttl_secs == 0 {
            return Err(ConfigError::ValidationError(
                "Recommendation cache TTL must be greater than 0".to_string(),
            ));
        }

        // Validate cold start configuration
        if self.cold_start.min_interactions == 0 {
            return Err(ConfigError::ValidationError(
                "Cold start min_interactions must be greater than 0".to_string(),
            ));
        }

        // Validate rate limiting
        if self.rate_limiting.enabled && self.rate_limiting.requests_per_minute == 0 {
            return Err(ConfigError::ValidationError(
                "Rate limit requests_per_minute must be greater than 0 when enabled".to_string(),
            ));
        }

        // Validate bulk operations
        if self.bulk_operations.import_batch_size == 0 {
            return Err(ConfigError::ValidationError(
                "Bulk import batch_size must be greater than 0".to_string(),
            ));
        }

        // Validate performance configuration
        if self.performance.worker_threads == 0 {
            return Err(ConfigError::ValidationError(
                "Worker threads must be greater than 0".to_string(),
            ));
        }

        // Validate security configuration
        if self.security.max_request_size_mb == 0 {
            return Err(ConfigError::ValidationError(
                "Max request size must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get cache TTL as Duration
    pub fn recommendation_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.recommendation_ttl_secs)
    }

    pub fn trending_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.trending_ttl_secs)
    }

    pub fn user_preference_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.user_preference_ttl_secs)
    }

    pub fn entity_feature_cache_ttl(&self) -> Duration {
        Duration::from_secs(self.cache.entity_feature_ttl_secs)
    }

    /// Get model update intervals as Duration
    pub fn incremental_update_interval(&self) -> Duration {
        Duration::from_secs(self.model_updates.incremental_update_interval_secs)
    }

    pub fn full_rebuild_interval(&self) -> Duration {
        Duration::from_secs(self.model_updates.full_rebuild_interval_hours * 3600)
    }

    pub fn trending_update_interval(&self) -> Duration {
        Duration::from_secs(self.model_updates.trending_update_interval_hours * 3600)
    }

    /// Get shutdown timeout as Duration
    pub fn shutdown_timeout(&self) -> Duration {
        Duration::from_secs(self.shutdown.timeout_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_port() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_database_connections() {
        let mut config = AppConfig::default();
        config.database.min_connections = 30;
        config.database.max_connections = 20;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_algorithm_weights() {
        let mut config = AppConfig::default();
        config.algorithms.collaborative_weight = 0.7;
        config.algorithms.content_based_weight = 0.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_similarity_threshold() {
        let mut config = AppConfig::default();
        config.algorithms.similarity_threshold = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cache_ttl_conversions() {
        let config = AppConfig::default();
        assert_eq!(config.recommendation_cache_ttl(), Duration::from_secs(300));
        assert_eq!(config.trending_cache_ttl(), Duration::from_secs(3600));
    }

    #[test]
    fn test_model_update_interval_conversions() {
        let config = AppConfig::default();
        assert_eq!(
            config.incremental_update_interval(),
            Duration::from_secs(10)
        );
        assert_eq!(
            config.full_rebuild_interval(),
            Duration::from_secs(24 * 3600)
        );
    }
}
