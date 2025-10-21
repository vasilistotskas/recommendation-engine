# Configuration Integration Guide

This guide shows how to integrate the configuration management system into the recommendation engine application.

## Quick Start

### 1. Add Dependency

Add to your crate's `Cargo.toml`:

```toml
[dependencies]
recommendation-config = { path = "../config" }
```

### 2. Load Configuration at Startup

```rust
use recommendation_config::{ConfigLoader, TenantConfigManager, HotReloadableConfig, ConfigWatcher};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = ConfigLoader::new()
        .with_file("config.yaml")  // Optional
        .load()?;

    tracing::info!("Configuration loaded successfully");

    // Create hot-reloadable configuration
    let hot_config = HotReloadableConfig::new(
        config.algorithms.clone(),
        config.cache.clone(),
        config.cold_start.clone(),
    );

    // Start configuration watcher (optional)
    if let Ok(mut watcher) = ConfigWatcher::new(
        "config.yaml",
        hot_config.algorithms.clone(),
        hot_config.cache.clone(),
        hot_config.cold_start.clone(),
    ) {
        watcher.start_watching().await?;
        tracing::info!("Configuration watcher started");
    }

    // Load tenant configurations (if multi-tenancy is enabled)
    let tenant_manager = TenantConfigManager::new();
    if config.multi_tenancy.enabled {
        tenant_manager.load_from_file("tenants.yaml")?;
        tracing::info!("Loaded {} tenant configurations", tenant_manager.tenant_count());
    }

    // Initialize services with configuration
    let db = Database::new(config.database.clone()).await?;
    let cache = RedisCache::new(config.redis.clone()).await?;

    // Start the application
    run_server(config, hot_config, tenant_manager, db, cache).await?;

    Ok(())
}
```

### 3. Use Configuration in Services

```rust
use recommendation_config::{AppConfig, HotReloadableConfig, TenantConfigManager};

pub struct RecommendationService {
    config: AppConfig,
    hot_config: HotReloadableConfig,
    tenant_manager: TenantConfigManager,
    // ... other dependencies
}

impl RecommendationService {
    pub fn new(
        config: AppConfig,
        hot_config: HotReloadableConfig,
        tenant_manager: TenantConfigManager,
    ) -> Self {
        Self {
            config,
            hot_config,
            tenant_manager,
        }
    }

    pub async fn get_recommendations(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> Result<Vec<Recommendation>> {
        // Get tenant-specific or default configuration
        let algo_config = self.tenant_manager.get_algorithm_config(
            tenant_id,
            &self.hot_config.get_algorithms(),
        );

        let cache_config = self.tenant_manager.get_cache_config(
            tenant_id,
            &self.hot_config.get_cache(),
        );

        // Use configuration in business logic
        let collaborative_weight = algo_config.collaborative_weight;
        let cache_ttl = cache_config.recommendation_ttl_secs;

        // ... recommendation logic
    }
}
```

### 4. Access Configuration in API Handlers

```rust
use axum::{Extension, Json};
use recommendation_config::HotReloadableConfig;

pub async fn get_config_handler(
    Extension(hot_config): Extension<HotReloadableConfig>,
) -> Json<ConfigResponse> {
    let algo = hot_config.get_algorithms();
    let cache = hot_config.get_cache();
    let cold_start = hot_config.get_cold_start();

    Json(ConfigResponse {
        algorithms: algo,
        cache,
        cold_start,
    })
}
```

## Environment Variables

### New Format (Recommended)

Use double underscore for nesting:

```bash
# Server
APP_SERVER__HOST=0.0.0.0
APP_SERVER__PORT=8080
APP_SERVER__LOG_LEVEL=info

# Database
APP_DATABASE__URL=postgresql://localhost:5432/recommendations
APP_DATABASE__MAX_CONNECTIONS=50

# Algorithms (hot-reloadable)
APP_ALGORITHMS__COLLABORATIVE_WEIGHT=0.7
APP_ALGORITHMS__CONTENT_BASED_WEIGHT=0.3
```

### Legacy Format (Backward Compatible)

```bash
HOST=0.0.0.0
PORT=8080
DATABASE_URL=postgresql://localhost:5432/recommendations
COLLABORATIVE_WEIGHT=0.7
```

## Configuration Files

### Main Configuration (config.yaml)

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  log_level: "info"

database:
  url: "postgresql://localhost:5432/recommendations"
  max_connections: 20

algorithms:
  collaborative_weight: 0.6
  content_based_weight: 0.4
```

### Tenant Configuration (tenants.yaml)

```yaml
tenant_a:
  tenant_id: "tenant_a"
  algorithms:
    collaborative_weight: 0.7
    content_based_weight: 0.3
  interaction_weights:
    view: 1.0
    purchase: 5.0
```

## Docker Integration

### Dockerfile

```dockerfile
FROM rust:1.82-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/recommendation-engine /usr/local/bin/
COPY config.yaml /etc/recommendation-engine/config.yaml
COPY tenants.yaml /etc/recommendation-engine/tenants.yaml
ENV APP_CONFIG_FILE=/etc/recommendation-engine/config.yaml
CMD ["recommendation-engine"]
```

### docker-compose.yml

```yaml
version: '3.8'
services:
  recommendation-engine:
    build: .
    ports:
      - "8080:8080"
    environment:
      - APP_DATABASE__URL=postgresql://postgres:postgres@db:5432/recommendations
      - APP_REDIS__URL=redis://redis:6379
      - APP_SERVER__LOG_LEVEL=info
    volumes:
      - ./config.yaml:/etc/recommendation-engine/config.yaml
      - ./tenants.yaml:/etc/recommendation-engine/tenants.yaml
```

## Kubernetes Integration

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: recommendation-engine-config
data:
  config.yaml: |
    server:
      host: "0.0.0.0"
      port: 8080
    algorithms:
      collaborative_weight: 0.6
      content_based_weight: 0.4
  tenants.yaml: |
    tenant_a:
      tenant_id: "tenant_a"
      algorithms:
        collaborative_weight: 0.7
        content_based_weight: 0.3
```

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: recommendation-engine
spec:
  template:
    spec:
      containers:
      - name: recommendation-engine
        image: recommendation-engine:latest
        env:
        - name: APP_DATABASE__URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: database-url
        - name: APP_REDIS__URL
          valueFrom:
            secretKeyRef:
              name: recommendation-secrets
              key: redis-url
        volumeMounts:
        - name: config
          mountPath: /etc/recommendation-engine
      volumes:
      - name: config
        configMap:
          name: recommendation-engine-config
```

## Hot-Reload in Production

To enable hot-reload in production:

1. Mount configuration file as a volume
2. Update the file externally
3. The watcher will automatically reload changes

**Example with Kubernetes:**

```bash
# Update ConfigMap
kubectl edit configmap recommendation-engine-config

# The file watcher will detect changes and reload
# Check logs for reload confirmation
kubectl logs -f deployment/recommendation-engine | grep "Configuration reloaded"
```

## Testing Configuration

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = ConfigLoader::new().load().unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_tenant_config() {
        let manager = TenantConfigManager::new();
        manager.load_from_file("tenants.yaml").unwrap();
        assert!(manager.has_tenant("tenant_a"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_config_hot_reload() {
    let config = ConfigLoader::new()
        .with_file("test_config.yaml")
        .load()
        .unwrap();

    let hot_config = HotReloadableConfig::new(
        config.algorithms.clone(),
        config.cache.clone(),
        config.cold_start.clone(),
    );

    // Modify config file
    // ...

    // Verify hot-reload
    tokio::time::sleep(Duration::from_secs(3)).await;
    let updated = hot_config.get_algorithms();
    assert_eq!(updated.collaborative_weight, 0.8);
}
```

## Troubleshooting

### Configuration Not Loading

1. Check file path is correct
2. Verify file format (YAML/TOML/JSON)
3. Check file permissions
4. Review logs for error messages

### Validation Errors

1. Check algorithm weights sum to 1.0
2. Verify all required fields are present
3. Ensure numeric values are in valid ranges
4. Review error message for specific issue

### Hot-Reload Not Working

1. Verify file watcher is started
2. Check file system supports inotify (Linux)
3. Ensure file is being modified, not replaced
4. Review logs for reload events

### Tenant Configuration Issues

1. Verify tenant file exists and is readable
2. Check tenant_id matches in requests
3. Ensure tenant config validates
4. Review fallback to default behavior

## Best Practices

1. **Use configuration files for complex setups** - Easier to manage than many environment variables
2. **Validate early** - Configuration is validated at startup
3. **Use tenant overrides sparingly** - Only override what's necessary
4. **Monitor hot-reload** - Watch logs for reload events
5. **Test configuration changes** - Validate before deploying
6. **Document custom settings** - Keep track of non-default values
7. **Use secrets management** - Don't commit sensitive values
8. **Version configuration files** - Track changes over time

## Support

For issues or questions:
- Check the configuration module README: `crates/config/README.md`
- Review example files: `config.example.yaml`, `tenants.example.yaml`
- Run tests: `cargo test -p recommendation-config`
- Check implementation summary: `TASK_18_IMPLEMENTATION.md`
