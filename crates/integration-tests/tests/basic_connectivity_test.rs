// Basic connectivity test to validate infrastructure without vector operations
// This test verifies that the test infrastructure (PostgreSQL, Redis) is working
// without hitting the pgvector type compatibility issue

use anyhow::Result;
use recommendation_storage::{Database, DatabaseConfig, RedisCache, RedisCacheConfig};
use sqlx::Row;

#[tokio::test]
async fn test_database_connectivity() -> Result<()> {
    // Use test database URL from environment or default
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost:5432/recommendations_test".to_string());

    let db_config = DatabaseConfig {
        url: database_url,
        max_connections: 5,
        min_connections: 1,
        acquire_timeout_secs: 3,
        idle_timeout_secs: 600,
        max_lifetime_secs: 1800,
    };

    // Test database connection
    let database = Database::new(db_config).await?;
    
    // Verify we can execute a simple query
    let result = sqlx::query("SELECT 1 as test")
        .fetch_one(database.pool())
        .await?;
    
    let test_value: i32 = result.try_get("test")?;
    assert_eq!(test_value, 1, "Database query should return 1");
    
    println!("✓ Database connectivity test passed");
    
    Ok(())
}

#[tokio::test]
async fn test_redis_connectivity() -> Result<()> {
    // Use test Redis URL from environment or default
    let redis_url = std::env::var("TEST_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let redis_config = RedisCacheConfig {
        url: redis_url,
        pool_size: 5,
        connection_timeout: std::time::Duration::from_secs(5),
        max_retry_attempts: 3,
        retry_backoff_ms: 100,
    };

    // Test Redis connection
    let redis_cache = RedisCache::new(redis_config).await?;
    
    // Test set and get operations
    let test_key = "test_connectivity_key";
    let test_value = "test_value".to_string();
    
    redis_cache.set(test_key, &test_value, std::time::Duration::from_secs(60)).await?;
    let retrieved: Option<String> = redis_cache.get(test_key).await?;
    
    assert_eq!(retrieved, Some(test_value), "Redis should return the stored value");
    
    // Cleanup
    redis_cache.delete(test_key).await?;
    
    println!("✓ Redis connectivity test passed");
    
    Ok(())
}

#[tokio::test]
async fn test_database_migrations_applied() -> Result<()> {
    // Use test database URL from environment or default
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost:5432/recommendations_test".to_string());

    let db_config = DatabaseConfig {
        url: database_url,
        max_connections: 5,
        min_connections: 1,
        acquire_timeout_secs: 3,
        idle_timeout_secs: 600,
        max_lifetime_secs: 1800,
    };

    let database = Database::new(db_config).await?;
    
    // Check if required tables exist
    let tables = vec![
        "entities",
        "interactions",
        "user_profiles",
        "trending_entities",
        "interaction_types",
    ];
    
    for table in tables {
        let query = format!(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = '{}'
            )",
            table
        );
        
        let result = sqlx::query(&query)
            .fetch_one(database.pool())
            .await?;
        
        let exists: bool = result.try_get("exists")?;
        assert!(exists, "Table '{}' should exist after migrations", table);
    }
    
    println!("✓ All required tables exist");
    
    // Check if pgvector extension is installed
    let result = sqlx::query(
        "SELECT EXISTS (
            SELECT FROM pg_extension 
            WHERE extname = 'vector'
        )"
    )
    .fetch_one(database.pool())
    .await?;
    
    let pgvector_exists: bool = result.try_get("exists")?;
    
    if pgvector_exists {
        println!("✓ pgvector extension is installed");
    } else {
        println!("⚠ pgvector extension is NOT installed (required for full tests)");
    }
    
    Ok(())
}
