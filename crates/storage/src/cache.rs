use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info, warn};

// Re-export models for convenience
use recommendation_models as models;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis connection error: {0}")]
    ConnectionError(#[from] RedisError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Cache key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid TTL value: {0}")]
    InvalidTtl(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Configuration for Redis cache
#[derive(Debug, Clone)]
pub struct RedisCacheConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: Duration,
    pub max_retry_attempts: u32,
    pub retry_backoff_ms: u64,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            // Increased pool size for high concurrency
            pool_size: 25,
            // Shorter timeout to fail fast under load
            connection_timeout: Duration::from_secs(2),
            // Fewer retries to prevent cascading delays
            max_retry_attempts: 2,
            retry_backoff_ms: 50,
        }
    }
}

/// Cache metrics for monitoring and observability
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
    sets: Arc<AtomicU64>,
    deletes: Arc<AtomicU64>,
    errors: Arc<AtomicU64>,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            sets: Arc::new(AtomicU64::new(0)),
            deletes: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_set(&self) {
        self.sets.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_delete(&self) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }
    
    pub fn get_misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }
    
    pub fn get_sets(&self) -> u64 {
        self.sets.load(Ordering::Relaxed)
    }
    
    pub fn get_deletes(&self) -> u64 {
        self.deletes.load(Ordering::Relaxed)
    }
    
    pub fn get_errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
    
    pub fn get_total_requests(&self) -> u64 {
        self.get_hits() + self.get_misses()
    }
    
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.get_total_requests();
        if total == 0 {
            0.0
        } else {
            self.get_hits() as f64 / total as f64
        }
    }
    
    pub fn get_miss_rate(&self) -> f64 {
        let total = self.get_total_requests();
        if total == 0 {
            0.0
        } else {
            self.get_misses() as f64 / total as f64
        }
    }
    
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.sets.store(0, Ordering::Relaxed);
        self.deletes.store(0, Ordering::Relaxed);
        self.errors.store(0, Ordering::Relaxed);
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Redis cache implementation with connection pooling and retry logic
pub struct RedisCache {
    connection_manager: ConnectionManager,
    config: RedisCacheConfig,
    metrics: CacheMetrics,
}

impl RedisCache {
    /// Create a new RedisCache instance with connection pooling
    pub async fn new(config: RedisCacheConfig) -> Result<Self> {
        info!("Initializing Redis cache with URL: {}", config.url);
        
        let client = Client::open(config.url.clone())
            .map_err(|e| {
                error!("Failed to create Redis client: {}", e);
                CacheError::ConnectionError(e)
            })?;
        
        let connection_manager = Self::create_connection_with_retry(&client, &config).await?;
        
        info!("Redis cache initialized successfully");
        
        Ok(Self {
            connection_manager,
            config,
            metrics: CacheMetrics::new(),
        })
    }
    
    /// Get a reference to the cache metrics
    pub fn metrics(&self) -> &CacheMetrics {
        &self.metrics
    }
    
    /// Get a reference to the cache configuration
    pub fn config(&self) -> &RedisCacheConfig {
        &self.config
    }
    
    /// Create connection with retry logic and exponential backoff
    async fn create_connection_with_retry(
        client: &Client,
        config: &RedisCacheConfig,
    ) -> Result<ConnectionManager> {
        let mut attempts = 0;
        let mut backoff_ms = config.retry_backoff_ms;
        
        loop {
            attempts += 1;
            
            match ConnectionManager::new(client.clone()).await {
                Ok(manager) => {
                    debug!("Redis connection established on attempt {}", attempts);
                    return Ok(manager);
                }
                Err(e) => {
                    if attempts >= config.max_retry_attempts {
                        error!(
                            "Failed to connect to Redis after {} attempts: {}",
                            attempts, e
                        );
                        return Err(CacheError::ConnectionError(e));
                    }
                    
                    warn!(
                        "Redis connection attempt {} failed: {}. Retrying in {}ms...",
                        attempts, e, backoff_ms
                    );
                    
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms *= 2; // Exponential backoff
                }
            }
        }
    }
    
    /// Get a value from cache and deserialize it
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        debug!("Cache GET: {}", key);
        
        let mut conn = self.connection_manager.clone();
        
        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| {
                error!("Failed to get key '{}' from cache: {}", key, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        match value {
            Some(json) => {
                let deserialized = serde_json::from_str(&json)
                    .map_err(|e| {
                        error!("Failed to deserialize cached value for key '{}': {}", key, e);
                        self.metrics.record_error();
                        CacheError::SerializationError(e)
                    })?;
                debug!("Cache HIT: {}", key);
                self.metrics.record_hit();
                Ok(Some(deserialized))
            }
            None => {
                debug!("Cache MISS: {}", key);
                self.metrics.record_miss();
                Ok(None)
            }
        }
    }
    
    /// Set a value in cache with TTL
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<()> {
        debug!("Cache SET: {} (TTL: {:?})", key, ttl);
        
        let json = serde_json::to_string(value)
            .map_err(|e| {
                error!("Failed to serialize value for key '{}': {}", key, e);
                self.metrics.record_error();
                CacheError::SerializationError(e)
            })?;
        
        let mut conn = self.connection_manager.clone();
        
        let ttl_secs = ttl.as_secs();
        if ttl_secs == 0 {
            self.metrics.record_error();
            return Err(CacheError::InvalidTtl(
                "TTL must be greater than 0 seconds".to_string(),
            ));
        }
        
        let _: () = conn.set_ex(key, json, ttl_secs)
            .await
            .map_err(|e| {
                error!("Failed to set key '{}' in cache: {}", key, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        debug!("Cache SET successful: {}", key);
        self.metrics.record_set();
        Ok(())
    }
    
    /// Delete a value from cache
    pub async fn delete(&self, key: &str) -> Result<bool> {
        debug!("Cache DELETE: {}", key);
        
        let mut conn = self.connection_manager.clone();
        
        let deleted: i32 = conn
            .del(key)
            .await
            .map_err(|e| {
                error!("Failed to delete key '{}' from cache: {}", key, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        let was_deleted = deleted > 0;
        if was_deleted {
            self.metrics.record_delete();
        }
        
        debug!("Cache DELETE {}: {} (deleted: {})", 
               if was_deleted { "successful" } else { "not found" }, 
               key, 
               deleted);
        
        Ok(was_deleted)
    }
    
    /// Delete multiple keys matching a pattern
    pub async fn delete_pattern(&self, pattern: &str) -> Result<usize> {
        debug!("Cache DELETE pattern: {}", pattern);
        
        let mut conn = self.connection_manager.clone();
        
        // Get all keys matching the pattern
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| {
                error!("Failed to get keys matching pattern '{}': {}", pattern, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        if keys.is_empty() {
            debug!("No keys found matching pattern: {}", pattern);
            return Ok(0);
        }
        
        // Delete all matching keys
        let deleted: i32 = conn
            .del(&keys)
            .await
            .map_err(|e| {
                error!("Failed to delete keys matching pattern '{}': {}", pattern, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        // Record each deletion
        for _ in 0..deleted {
            self.metrics.record_delete();
        }
        
        debug!("Deleted {} keys matching pattern: {}", deleted, pattern);
        Ok(deleted as usize)
    }
    
    /// Check if a key exists in cache
    pub async fn exists(&self, key: &str) -> Result<bool> {
        debug!("Cache EXISTS: {}", key);
        
        let mut conn = self.connection_manager.clone();
        
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| {
                error!("Failed to check existence of key '{}': {}", key, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        debug!("Cache EXISTS {}: {}", key, exists);
        Ok(exists)
    }
    
    /// Get the TTL of a key in seconds
    pub async fn ttl(&self, key: &str) -> Result<Option<i64>> {
        debug!("Cache TTL: {}", key);
        
        let mut conn = self.connection_manager.clone();
        
        let ttl: i64 = conn
            .ttl(key)
            .await
            .map_err(|e| {
                error!("Failed to get TTL for key '{}': {}", key, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        // Redis returns -2 if key doesn't exist, -1 if key has no expiry
        let result = match ttl {
            -2 => None,
            -1 => Some(-1),
            n => Some(n),
        };
        
        debug!("Cache TTL {}: {:?}", key, result);
        Ok(result)
    }
    
    /// Ping the Redis server to check connection health
    pub async fn ping(&self) -> Result<()> {
        debug!("Pinging Redis server");
        
        let mut conn = self.connection_manager.clone();
        
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Redis ping failed: {}", e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;
        
        debug!("Redis ping successful");
        Ok(())
    }
}

/// Cache key generation and caching strategies
pub mod cache_keys {
    use std::collections::HashMap;
    use std::time::Duration;
    
    /// TTL constants for different cache types
    pub const RECOMMENDATION_TTL: Duration = Duration::from_secs(300); // 5 minutes
    pub const TRENDING_TTL: Duration = Duration::from_secs(3600); // 1 hour
    pub const USER_PREFERENCE_TTL: Duration = Duration::from_secs(600); // 10 minutes
    pub const ENTITY_FEATURE_TTL: Duration = Duration::from_secs(3600); // 1 hour
    
    /// Generate cache key for recommendation results
    /// Format: rec:{user_id}:{algorithm}:{count}:{filters_hash}
    pub fn recommendation_key(
        user_id: &str,
        algorithm: &str,
        count: usize,
        filters: &Option<HashMap<String, String>>,
    ) -> String {
        let filters_hash = match filters {
            Some(f) if !f.is_empty() => {
                let mut keys: Vec<_> = f.keys().collect();
                keys.sort();
                let filter_str: String = keys
                    .iter()
                    .map(|k| format!("{}:{}", k, f.get(*k).unwrap()))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(":{}", hash_string(&filter_str))
            }
            _ => String::new(),
        };
        
        format!("rec:{}:{}:{}{}", user_id, algorithm, count, filters_hash)
    }
    
    /// Generate cache key for trending entities
    /// Format: trending:{entity_type}:{count}
    pub fn trending_key(entity_type: &str, count: usize) -> String {
        format!("trending:{}:{}", entity_type, count)
    }
    
    /// Generate cache key for user preference vector
    /// Format: user_pref:{user_id}
    pub fn user_preference_key(user_id: &str) -> String {
        format!("user_pref:{}", user_id)
    }
    
    /// Generate cache key for entity feature vector
    /// Format: entity_feat:{entity_id}
    pub fn entity_feature_key(entity_id: &str) -> String {
        format!("entity_feat:{}", entity_id)
    }
    
    /// Generate pattern for invalidating user-related caches
    /// Format: rec:{user_id}:*
    pub fn user_recommendation_pattern(user_id: &str) -> String {
        format!("rec:{}:*", user_id)
    }
    
    /// Generate pattern for invalidating entity-related caches
    /// Format: entity_feat:{entity_id}
    pub fn entity_pattern(entity_id: &str) -> String {
        format!("entity_feat:{}", entity_id)
    }
    
    /// Generate pattern for invalidating all trending caches
    /// Format: trending:*
    pub fn trending_pattern() -> String {
        "trending:*".to_string()
    }
    
    /// Simple hash function for filter strings
    fn hash_string(s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

/// High-level caching strategies for the recommendation engine
impl RedisCache {
    /// Cache recommendation results with 5-minute TTL
    pub async fn cache_recommendations(
        &self,
        user_id: &str,
        algorithm: &str,
        count: usize,
        filters: &Option<HashMap<String, String>>,
        recommendations: &models::RecommendationResponse,
    ) -> Result<()> {
        let key = cache_keys::recommendation_key(user_id, algorithm, count, filters);
        self.set(&key, recommendations, cache_keys::RECOMMENDATION_TTL).await
    }
    
    /// Get cached recommendation results
    pub async fn get_cached_recommendations(
        &self,
        user_id: &str,
        algorithm: &str,
        count: usize,
        filters: &Option<HashMap<String, String>>,
    ) -> Result<Option<models::RecommendationResponse>> {
        let key = cache_keys::recommendation_key(user_id, algorithm, count, filters);
        self.get(&key).await
    }
    
    /// Cache trending entities with 1-hour TTL
    pub async fn cache_trending_entities(
        &self,
        entity_type: &str,
        count: usize,
        entities: &Vec<models::ScoredEntity>,
    ) -> Result<()> {
        let key = cache_keys::trending_key(entity_type, count);
        self.set(&key, entities, cache_keys::TRENDING_TTL).await
    }
    
    /// Get cached trending entities
    pub async fn get_cached_trending_entities(
        &self,
        entity_type: &str,
        count: usize,
    ) -> Result<Option<Vec<models::ScoredEntity>>> {
        let key = cache_keys::trending_key(entity_type, count);
        self.get(&key).await
    }
    
    /// Cache user preference vector with 10-minute TTL
    pub async fn cache_user_preference_vector(
        &self,
        user_id: &str,
        vector: &Vec<f32>,
    ) -> Result<()> {
        let key = cache_keys::user_preference_key(user_id);
        self.set(&key, vector, cache_keys::USER_PREFERENCE_TTL).await
    }
    
    /// Get cached user preference vector
    pub async fn get_cached_user_preference_vector(
        &self,
        user_id: &str,
    ) -> Result<Option<Vec<f32>>> {
        let key = cache_keys::user_preference_key(user_id);
        self.get(&key).await
    }
    
    /// Cache entity feature vector with 1-hour TTL
    pub async fn cache_entity_feature_vector(
        &self,
        entity_id: &str,
        vector: &Vec<f32>,
    ) -> Result<()> {
        let key = cache_keys::entity_feature_key(entity_id);
        self.set(&key, vector, cache_keys::ENTITY_FEATURE_TTL).await
    }
    
    /// Get cached entity feature vector
    pub async fn get_cached_entity_feature_vector(
        &self,
        entity_id: &str,
    ) -> Result<Option<Vec<f32>>> {
        let key = cache_keys::entity_feature_key(entity_id);
        self.get(&key).await
    }
    
    /// Invalidate all caches related to a user (called when new interaction is recorded)
    pub async fn invalidate_user_caches(&self, user_id: &str) -> Result<usize> {
        let mut total_deleted = 0;
        
        // Invalidate recommendation caches
        let rec_pattern = cache_keys::user_recommendation_pattern(user_id);
        total_deleted += self.delete_pattern(&rec_pattern).await?;
        
        // Invalidate user preference vector cache
        let pref_key = cache_keys::user_preference_key(user_id);
        if self.delete(&pref_key).await? {
            total_deleted += 1;
        }
        
        debug!("Invalidated {} cache entries for user: {}", total_deleted, user_id);
        Ok(total_deleted)
    }
    
    /// Invalidate all caches related to an entity (called when entity is updated)
    pub async fn invalidate_entity_caches(&self, entity_id: &str) -> Result<usize> {
        let mut total_deleted = 0;
        
        // Invalidate entity feature vector cache
        let feat_key = cache_keys::entity_feature_key(entity_id);
        if self.delete(&feat_key).await? {
            total_deleted += 1;
        }
        
        // Note: We don't invalidate recommendation caches here as they will expire naturally
        // and entity updates are less frequent than interactions
        
        debug!("Invalidated {} cache entries for entity: {}", total_deleted, entity_id);
        Ok(total_deleted)
    }
    
    /// Invalidate all trending caches (called when trending calculation is updated)
    pub async fn invalidate_trending_caches(&self) -> Result<usize> {
        let pattern = cache_keys::trending_pattern();
        let deleted = self.delete_pattern(&pattern).await?;
        debug!("Invalidated {} trending cache entries", deleted);
        Ok(deleted)
    }
}

impl RedisCache {
    /// Set a string value in cache with TTL (for raw string storage)
    pub async fn set_string(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        debug!("Cache SET (string): {} (TTL: {:?})", key, ttl);

        let mut conn = self.connection_manager.clone();

        let ttl_secs = ttl.as_secs();
        if ttl_secs == 0 {
            self.metrics.record_error();
            return Err(CacheError::InvalidTtl(
                "TTL must be greater than 0 seconds".to_string(),
            ));
        }

        let _: () = conn.set_ex(key, value, ttl_secs)
            .await
            .map_err(|e| {
                error!("Failed to set key '{}' in cache: {}", key, e);
                self.metrics.record_error();
                CacheError::ConnectionError(e)
            })?;

        debug!("Cache SET (string) successful: {}", key);
        self.metrics.record_set();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = RedisCacheConfig::default();
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.pool_size, 10);
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.retry_backoff_ms, 100);
    }

    #[test]
    fn test_cache_config_custom() {
        let config = RedisCacheConfig {
            url: "redis://custom:6380".to_string(),
            pool_size: 20,
            connection_timeout: Duration::from_secs(10),
            max_retry_attempts: 5,
            retry_backoff_ms: 200,
        };

        assert_eq!(config.url, "redis://custom:6380");
        assert_eq!(config.pool_size, 20);
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert_eq!(config.max_retry_attempts, 5);
        assert_eq!(config.retry_backoff_ms, 200);
    }

    #[test]
    fn test_recommendation_key_generation() {
        let key = cache_keys::recommendation_key("user_123", "hybrid", 10, &None);
        assert_eq!(key, "rec:user_123:hybrid:10");
    }

    #[test]
    fn test_recommendation_key_with_filters() {
        let mut filters = HashMap::new();
        filters.insert("category".to_string(), "electronics".to_string());
        filters.insert("price_max".to_string(), "1000".to_string());

        let key = cache_keys::recommendation_key("user_456", "collaborative", 20, &Some(filters));
        assert!(key.starts_with("rec:user_456:collaborative:20:"));
        assert!(key.len() > "rec:user_456:collaborative:20:".len());
    }

    #[test]
    fn test_recommendation_key_filter_consistency() {
        let mut filters1 = HashMap::new();
        filters1.insert("a".to_string(), "1".to_string());
        filters1.insert("b".to_string(), "2".to_string());

        let mut filters2 = HashMap::new();
        filters2.insert("b".to_string(), "2".to_string());
        filters2.insert("a".to_string(), "1".to_string());

        let key1 = cache_keys::recommendation_key("user_789", "hybrid", 10, &Some(filters1));
        let key2 = cache_keys::recommendation_key("user_789", "hybrid", 10, &Some(filters2));

        // Keys should be identical regardless of insertion order
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_trending_key_generation() {
        let key = cache_keys::trending_key("product", 50);
        assert_eq!(key, "trending:product:50");
    }

    #[test]
    fn test_user_preference_key_generation() {
        let key = cache_keys::user_preference_key("user_999");
        assert_eq!(key, "user_pref:user_999");
    }

    #[test]
    fn test_entity_feature_key_generation() {
        let key = cache_keys::entity_feature_key("entity_abc");
        assert_eq!(key, "entity_feat:entity_abc");
    }

    #[test]
    fn test_user_recommendation_pattern() {
        let pattern = cache_keys::user_recommendation_pattern("user_123");
        assert_eq!(pattern, "rec:user_123:*");
    }

    #[test]
    fn test_entity_pattern() {
        let pattern = cache_keys::entity_pattern("entity_456");
        assert_eq!(pattern, "entity_feat:entity_456");
    }

    #[test]
    fn test_trending_pattern() {
        let pattern = cache_keys::trending_pattern();
        assert_eq!(pattern, "trending:*");
    }

    #[test]
    fn test_ttl_constants() {
        assert_eq!(cache_keys::RECOMMENDATION_TTL, Duration::from_secs(300));
        assert_eq!(cache_keys::TRENDING_TTL, Duration::from_secs(3600));
        assert_eq!(cache_keys::USER_PREFERENCE_TTL, Duration::from_secs(600));
        assert_eq!(cache_keys::ENTITY_FEATURE_TTL, Duration::from_secs(3600));
    }

    #[test]
    fn test_cache_metrics_initialization() {
        let metrics = CacheMetrics::new();
        assert_eq!(metrics.get_hits(), 0);
        assert_eq!(metrics.get_misses(), 0);
        assert_eq!(metrics.get_sets(), 0);
        assert_eq!(metrics.get_deletes(), 0);
        assert_eq!(metrics.get_errors(), 0);
        assert_eq!(metrics.get_total_requests(), 0);
        assert_eq!(metrics.get_hit_rate(), 0.0);
        assert_eq!(metrics.get_miss_rate(), 0.0);
    }

    #[test]
    fn test_cache_metrics_record_hit() {
        let metrics = CacheMetrics::new();
        metrics.record_hit();
        metrics.record_hit();
        metrics.record_hit();

        assert_eq!(metrics.get_hits(), 3);
        assert_eq!(metrics.get_total_requests(), 3);
        assert_eq!(metrics.get_hit_rate(), 1.0);
    }

    #[test]
    fn test_cache_metrics_record_miss() {
        let metrics = CacheMetrics::new();
        metrics.record_miss();
        metrics.record_miss();

        assert_eq!(metrics.get_misses(), 2);
        assert_eq!(metrics.get_total_requests(), 2);
        assert_eq!(metrics.get_miss_rate(), 1.0);
    }

    #[test]
    fn test_cache_metrics_hit_rate_calculation() {
        let metrics = CacheMetrics::new();

        // 7 hits, 3 misses = 70% hit rate
        for _ in 0..7 {
            metrics.record_hit();
        }
        for _ in 0..3 {
            metrics.record_miss();
        }

        assert_eq!(metrics.get_hits(), 7);
        assert_eq!(metrics.get_misses(), 3);
        assert_eq!(metrics.get_total_requests(), 10);
        assert_eq!(metrics.get_hit_rate(), 0.7);
        assert_eq!(metrics.get_miss_rate(), 0.3);
    }

    #[test]
    fn test_cache_metrics_record_set() {
        let metrics = CacheMetrics::new();
        metrics.record_set();
        metrics.record_set();
        metrics.record_set();
        metrics.record_set();

        assert_eq!(metrics.get_sets(), 4);
    }

    #[test]
    fn test_cache_metrics_record_delete() {
        let metrics = CacheMetrics::new();
        metrics.record_delete();
        metrics.record_delete();

        assert_eq!(metrics.get_deletes(), 2);
    }

    #[test]
    fn test_cache_metrics_record_error() {
        let metrics = CacheMetrics::new();
        metrics.record_error();
        metrics.record_error();
        metrics.record_error();

        assert_eq!(metrics.get_errors(), 3);
    }

    #[test]
    fn test_cache_metrics_reset() {
        let metrics = CacheMetrics::new();

        metrics.record_hit();
        metrics.record_miss();
        metrics.record_set();
        metrics.record_delete();
        metrics.record_error();

        assert_eq!(metrics.get_hits(), 1);
        assert_eq!(metrics.get_misses(), 1);
        assert_eq!(metrics.get_sets(), 1);
        assert_eq!(metrics.get_deletes(), 1);
        assert_eq!(metrics.get_errors(), 1);

        metrics.reset();

        assert_eq!(metrics.get_hits(), 0);
        assert_eq!(metrics.get_misses(), 0);
        assert_eq!(metrics.get_sets(), 0);
        assert_eq!(metrics.get_deletes(), 0);
        assert_eq!(metrics.get_errors(), 0);
    }

    #[test]
    fn test_cache_metrics_default() {
        let metrics = CacheMetrics::default();
        assert_eq!(metrics.get_hits(), 0);
        assert_eq!(metrics.get_misses(), 0);
    }

    #[test]
    fn test_cache_metrics_clone() {
        let metrics1 = CacheMetrics::new();
        metrics1.record_hit();
        metrics1.record_miss();

        let metrics2 = metrics1.clone();

        // Both should share the same underlying atomic counters
        metrics2.record_hit();

        assert_eq!(metrics1.get_hits(), 2);
        assert_eq!(metrics2.get_hits(), 2);
        assert_eq!(metrics1.get_misses(), 1);
        assert_eq!(metrics2.get_misses(), 1);
    }
}
