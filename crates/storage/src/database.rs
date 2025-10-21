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
            // Increased for very high concurrency (1000+ concurrent requests)
            // With 1000 concurrent requests, we need more connections to avoid exhaustion
            max_connections: 100,
            min_connections: 20,
            // Longer timeout to handle high load without failing requests
            acquire_timeout_secs: 5,
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
            // Optimize connection settings for high-performance vector queries
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Set statement timeout to 10 seconds for all queries
                    // This prevents slow queries from exhausting the connection pool
                    sqlx::query("SET statement_timeout = '10s'")
                        .execute(&mut *conn)
                        .await?;

                    // Increase work_mem for better vector query performance
                    // Higher work_mem allows PostgreSQL to use more memory for sorting/hashing
                    // which is beneficial for vector similarity searches
                    sqlx::query("SET work_mem = '16MB'")
                        .execute(&mut *conn)
                        .await?;

                    // Enable parallel query execution for large datasets
                    sqlx::query("SET max_parallel_workers_per_gather = 4")
                        .execute(&mut *conn)
                        .await?;

                    Ok(())
                })
            })
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
        match sqlx::query("SELECT 1").fetch_one(&self.pool).await {
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
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.min_connections, 20);
        assert_eq!(config.acquire_timeout_secs, 5);
    }

    #[test]
    fn test_pool_stats() {
        let stats = PoolStats { size: 10, idle: 5 };
        assert_eq!(stats.size, 10);
        assert_eq!(stats.idle, 5);
    }
}
