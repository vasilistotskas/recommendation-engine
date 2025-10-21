use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn};

/// Database configuration for connection pool
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost:5432/recommendations".to_string(),
            // Increased for high load (50K entities, high concurrency)
            max_connections: 50,
            min_connections: 10,
            // Shorter timeout to fail fast under load
            acquire_timeout_secs: 2,
            // Keep connections alive longer to reduce overhead
            idle_timeout_secs: 300,
            max_lifetime_secs: 1800,
        }
    }
}

/// Database connection pool manager
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool with the given configuration
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!(
            "Initializing database connection pool with max_connections={}",
            config.max_connections
        );

        let pool = Self::create_pool_with_retry(&config).await?;

        info!("Database connection pool initialized successfully");

        Ok(Self { pool })
    }

    /// Create connection pool with retry logic and exponential backoff
    async fn create_pool_with_retry(config: &DatabaseConfig) -> Result<PgPool> {
        let max_retries = 3;
        let mut retry_count = 0;
        let mut backoff_ms = 1000;

        loop {
            match Self::create_pool(config).await {
                Ok(pool) => return Ok(pool),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(e).context(format!(
                            "Failed to create database pool after {} retries",
                            max_retries
                        ));
                    }

                    warn!(
                        "Failed to create database pool (attempt {}/{}): {}. Retrying in {}ms...",
                        retry_count, max_retries, e, backoff_ms
                    );

                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    backoff_ms *= 2; // Exponential backoff
                }
            }
        }
    }

    /// Create a connection pool with the given configuration
    async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.acquire_timeout_secs))
            .idle_timeout(Some(Duration::from_secs(config.idle_timeout_secs)))
            .max_lifetime(Some(Duration::from_secs(config.max_lifetime_secs)))
            .connect(&config.url)
            .await
            .context("Failed to connect to database")?;

        Ok(pool)
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Health check query for readiness probe
    /// Returns true if the database is accessible and responsive
    pub async fn health_check(&self) -> Result<bool> {
        match sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get connection pool statistics for monitoring
    pub fn pool_stats(&self) -> PoolStats {
        PoolStats {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }

    /// Close the connection pool gracefully
    pub async fn close(&self) {
        info!("Closing database connection pool");
        self.pool.close().await;
    }
}

/// Connection pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.acquire_timeout_secs, 3);
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats {
            size: 10,
            idle: 5,
        };
        assert_eq!(stats.size, 10);
        assert_eq!(stats.idle, 5);
    }
}
