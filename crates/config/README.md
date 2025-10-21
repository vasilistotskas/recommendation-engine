# Recommendation Engine Configuration

This crate provides comprehensive configuration management for the recommendation engine, including:

- **Multi-source configuration loading** (environment variables, config files, defaults)
- **Configuration validation** with fail-fast error messages
- **Per-tenant configuration overrides** for multi-tenancy support
- **Hot-reload** for non-critical configuration changes without restart

## Features

### 1. Configuration Loading

The configuration loader supports multiple sources with the following precedence (highest to lowest):

1. **Environment variables** (highest precedence)
2. **Configuration file** (YAML, TOML, or JSON)
3. **Default values** (lowest precedence)

#### Example Usage

```rust
use recommendation_config::{ConfigLoader, AppConfig};

// Load with defaults and environment variables
let config = ConfigLoader::new().load()?;

// Load with configuration file
let config = ConfigLoader::new()
    .with_file("config.yaml")
    .load()?;
```

#### Environment Variables

Two formats are supported:

**New format** (recommended):
```bash
APP_SERVER__PORT=8080
APP_DATABASE__MAX_CONNECTIONS=50
APP_ALGORITHMS__COLLABORATIVE_WEIGHT=0.7
```

**Legacy format** (backward compatible):
```bash
PORT=8080
DATABASE_MAX_CONNECTIONS=50
COLLABORATIVE_WEIGHT=0.7
```

### 2. Configuration Validation

All configuration is validated at startup with clear error messages:

```rust
let config = ConfigLoader::new().load()?;
// Validation happens automatically during load()

// Manual validation
config.validate()?;
```

**Validation Rules:**
- Algorithm weights must sum to 1.0
- Similarity threshold must be between 0.0 and 1.0
- Connection pool sizes must be positive
- Min connections cannot exceed max connections
- TTL values must be greater than 0
- And more...

### 3. Per-Tenant Configuration

Support for tenant-specific configuration overrides:

```rust
use recommendation_config::{TenantConfigManager, TenantConfig};

let manager = TenantConfigManager::new();

// Load tenant configurations from file
manager.load_from_file("tenants.yaml")?;

// Get tenant-specific configuration
let tenant_algo = manager.get_algorithm_config("tenant_a", &default_algo);
let tenant_cache = manager.get_cache_config("tenant_a", &default_cache);

// Get custom interaction weights
let weight = manager.get_interaction_weight("tenant_a", "purchase");
```

**Tenant Configuration File Example** (`tenants.yaml`):

```yaml
tenant_a:
  tenant_id: "tenant_a"
  algorithms:
    collaborative_weight: 0.7
    content_based_weight: 0.3
  cache:
    recommendation_ttl_secs: 600
  interaction_weights:
    view: 1.0
    purchase: 5.0
    like: 2.0

tenant_b:
  tenant_id: "tenant_b"
  algorithms:
    collaborative_weight: 0.5
    content_based_weight: 0.5
```

### 4. Hot-Reload

Non-critical configuration can be reloaded without restarting the service:

```rust
use recommendation_config::{ConfigWatcher, HotReloadableConfig};
use std::sync::{Arc, RwLock};

// Create hot-reloadable configuration
let hot_config = HotReloadableConfig::new(
    config.algorithms.clone(),
    config.cache.clone(),
    config.cold_start.clone(),
);

// Start watching for file changes
let mut watcher = ConfigWatcher::new(
    "config.yaml",
    hot_config.algorithms.clone(),
    hot_config.cache.clone(),
    hot_config.cold_start.clone(),
)?;

watcher.start_watching().await?;

// Access current configuration (always up-to-date)
let current_algo = hot_config.get_algorithms();
let current_cache = hot_config.get_cache();
```

**Hot-Reloadable Settings:**
- Algorithm weights and thresholds
- Cache TTL values
- Cold start parameters

**Non-Hot-Reloadable Settings** (require restart):
- Server host and port
- Database connection settings
- Redis connection settings
- Authentication settings

## Configuration Structure

### Main Configuration (`AppConfig`)

```rust
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub algorithms: AlgorithmConfig,          // Hot-reloadable
    pub model_updates: ModelUpdateConfig,
    pub cache: CacheConfig,                   // Hot-reloadable
    pub authentication: AuthConfig,
    pub rate_limiting: RateLimitConfig,
    pub observability: ObservabilityConfig,
    pub cold_start: ColdStartConfig,          // Hot-reloadable
    pub multi_tenancy: MultiTenancyConfig,
    pub webhooks: WebhookConfig,
    pub bulk_operations: BulkOperationsConfig,
    pub performance: PerformanceConfig,
    pub security: SecurityConfig,
    pub shutdown: ShutdownConfig,
}
```

### Tenant Configuration (`TenantConfig`)

```rust
pub struct TenantConfig {
    pub tenant_id: String,
    pub algorithms: Option<AlgorithmConfig>,
    pub cache: Option<CacheConfig>,
    pub cold_start: Option<ColdStartConfig>,
    pub interaction_weights: Option<HashMap<String, f32>>,
}
```

## Configuration Files

### Main Configuration File

Supports YAML, TOML, and JSON formats:

**YAML** (`config.yaml`):
```yaml
server:
  host: "0.0.0.0"
  port: 8080
  log_level: "info"

algorithms:
  collaborative_weight: 0.6
  content_based_weight: 0.4
  similarity_threshold: 0.5
```

**TOML** (`config.toml`):
```toml
[server]
host = "0.0.0.0"
port = 8080
log_level = "info"

[algorithms]
collaborative_weight = 0.6
content_based_weight = 0.4
similarity_threshold = 0.5
```

### Tenant Configuration File

**YAML** (`tenants.yaml`):
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

## Error Handling

All configuration operations return `Result<T, ConfigError>`:

```rust
pub enum ConfigError {
    LoadError(String),
    ValidationError(String),
    EnvError(std::env::VarError),
    FileError(config::ConfigError),
    SerializationError(serde_json::Error),
    IoError(std::io::Error),
    TenantNotFound(String),
    InvalidValue(String),
}
```

## Best Practices

1. **Use configuration files for complex setups**: Environment variables work well for simple deployments, but configuration files are easier to manage for complex setups.

2. **Validate early**: Configuration is validated during loading, so errors are caught at startup before the service begins processing requests.

3. **Use tenant overrides sparingly**: Only override what's necessary for each tenant to keep configuration manageable.

4. **Monitor hot-reload**: Watch logs for configuration reload events and errors.

5. **Test configuration changes**: Validate configuration files before deploying to production.

## Examples

See the example configuration files in the repository:
- `config.example.yaml` - Main configuration example
- `tenants.example.yaml` - Tenant configuration example

## Testing

Run tests with:

```bash
cargo test -p recommendation-config
```

## Dependencies

- `config` - Configuration management
- `serde` - Serialization/deserialization
- `notify` - File system watching for hot-reload
- `tokio` - Async runtime
- `tracing` - Logging
