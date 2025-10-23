# Recommendation Storage

This crate provides database and caching functionality for the recommendation engine.

## Features

- **Database Connection Pool**: PostgreSQL connection pool with retry logic and health checks
- **Migration Runner**: Automatic database schema migrations with pgvector validation
- **Vector Store**: PostgreSQL storage with pgvector for similarity search (to be implemented)
- **Redis Cache**: Caching layer for hot recommendations (to be implemented)

## Database Setup

### Prerequisites

1. PostgreSQL 12+ with pgvector extension installed
2. Redis 8+ for caching

### Installation

**Docker (Recommended)**:
```bash
docker run -d \
  --name postgres-pgvector \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=recommendations \
  -p 5432:5432 \
  pgvector/pgvector:pg17
```

**macOS**:
```bash
brew install pgvector
```

**Debian/Ubuntu**:
```bash
apt install postgresql-16-pgvector
```

## Usage

### Database Connection

```rust
use recommendation_storage::{Database, DatabaseConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create database configuration
    let config = DatabaseConfig {
        url: "postgresql://postgres:postgres@localhost:5432/recommendations".to_string(),
        max_connections: 20,
        min_connections: 5,
        acquire_timeout_secs: 3,
        idle_timeout_secs: 600,
        max_lifetime_secs: 1800,
    };

    // Initialize database with retry logic
    let db = Database::new(config).await?;

    // Health check
    let is_healthy = db.health_check().await?;
    println!("Database healthy: {}", is_healthy);

    // Get pool statistics
    let stats = db.pool_stats();
    println!("Pool size: {}, Idle: {}", stats.size, stats.idle);

    Ok(())
}
```

### Running Migrations

```rust
use recommendation_storage::{Database, DatabaseConfig, MigrationRunner, MigrationConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database
    let db_config = DatabaseConfig::default();
    let db = Database::new(db_config).await?;

    // Create migration runner
    let migration_config = MigrationConfig {
        auto_run: true,
        validate_pgvector: true,
    };
    let runner = MigrationRunner::new(db.pool().clone(), migration_config);

    // Run migrations
    runner.run_migrations().await?;

    // Check current version
    if let Some(version) = runner.get_current_version().await? {
        println!("Current migration version: {}", version);
    }

    // List applied migrations
    let migrations = runner.list_applied_migrations().await?;
    for migration in migrations {
        println!("Migration {}: {} ({})", 
            migration.version, 
            migration.description,
            if migration.success { "success" } else { "failed" }
        );
    }

    Ok(())
}
```

## Database Schema

The migration system creates the following tables:

### entities
Stores domain-agnostic entities with feature vectors for content-based filtering.

- `entity_id`: Unique identifier within entity_type
- `entity_type`: Type of entity (product, article, user, etc.)
- `tenant_id`: Tenant identifier for multi-tenancy
- `attributes`: Flexible JSON attributes
- `feature_vector`: 512-dimensional vector for similarity search
- `created_at`, `updated_at`: Timestamps

### interactions
Tracks user-entity interactions for collaborative filtering.

- `user_id`: User identifier
- `entity_id`: Entity identifier
- `entity_type`: Type of entity
- `tenant_id`: Tenant identifier
- `interaction_type`: Type of interaction (view, purchase, like, etc.)
- `weight`: Configurable weight for the interaction
- `metadata`: Optional JSON metadata
- `timestamp`: When the interaction occurred

### user_profiles
Stores user preference vectors computed from interaction history.

- `user_id`: User identifier
- `tenant_id`: Tenant identifier
- `preference_vector`: 512-dimensional preference vector
- `interaction_count`: Total interactions (for cold start detection)
- `last_interaction_at`: Last interaction timestamp
- `created_at`, `updated_at`: Timestamps

### trending_entities
Caches trending entity calculations.

- `entity_id`: Entity identifier
- `entity_type`: Type of entity
- `tenant_id`: Tenant identifier
- `score`: Trending score
- `window_start`, `window_end`: Time window for trending calculation

## Configuration

### Environment Variables

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/recommendations
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
DATABASE_ACQUIRE_TIMEOUT_SECS=3
DATABASE_IDLE_TIMEOUT_SECS=600
DATABASE_MAX_LIFETIME_SECS=1800

# Migration settings
AUTO_RUN_MIGRATIONS=true
VALIDATE_PGVECTOR=true
```

## Error Handling

The crate uses `anyhow::Result` for error handling with context:

```rust
use anyhow::Context;

let db = Database::new(config)
    .await
    .context("Failed to initialize database")?;
```

## Testing

Run tests with:

```bash
cargo test --package recommendation-storage
```

Note: Integration tests requiring a real database are not included in unit tests.

## Performance

- **Connection Pooling**: Reuses connections to minimize overhead
- **Retry Logic**: Exponential backoff for transient failures
- **Health Checks**: Fast health check queries for readiness probes
- **HNSW Indexing**: Sub-linear similarity search with pgvector

## Multi-Tenancy

All tables include a `tenant_id` column for data isolation:

```sql
-- Tenant A's data
SELECT * FROM entities WHERE tenant_id = 'tenant_a';

-- Tenant B's data (completely isolated)
SELECT * FROM entities WHERE tenant_id = 'tenant_b';
```

## License

MIT
