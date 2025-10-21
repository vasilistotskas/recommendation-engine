use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use tracing::{info, warn, error};

/// Migration runner configuration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Whether to run migrations automatically on startup
    pub auto_run: bool,
    /// Whether to validate pgvector extension
    pub validate_pgvector: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            auto_run: true,
            validate_pgvector: true,
        }
    }
}

/// Migration runner for managing database schema
pub struct MigrationRunner {
    pool: PgPool,
    config: MigrationConfig,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new(pool: PgPool, config: MigrationConfig) -> Self {
        Self { pool, config }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self) -> Result<()> {
        if !self.config.auto_run {
            info!("Auto-run migrations disabled, skipping");
            return Ok(());
        }

        info!("Running database migrations");

        // Validate pgvector extension first
        if self.config.validate_pgvector {
            self.validate_pgvector_extension().await?;
        }

        // Run sqlx migrations
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await
            .context("Failed to run database migrations")?;

        info!("Database migrations completed successfully");

        Ok(())
    }

    /// Validate that pgvector extension is installed and available
    pub async fn validate_pgvector_extension(&self) -> Result<()> {
        info!("Validating pgvector extension");

        // Check if pgvector extension exists
        let result = sqlx::query(
            "SELECT EXISTS(
                SELECT 1 FROM pg_available_extensions 
                WHERE name = 'vector'
            ) as exists"
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to check for pgvector extension")?;

        let exists: bool = result.try_get("exists")
            .context("Failed to parse pgvector extension check result")?;

        if !exists {
            error!("pgvector extension is not available in this PostgreSQL installation");
            return Err(anyhow::anyhow!(
                "pgvector extension not found. Please install pgvector:\n\
                 - For PostgreSQL 12+: https://github.com/pgvector/pgvector#installation\n\
                 - Docker: Use postgres:16 with pgvector installed\n\
                 - Debian/Ubuntu: apt install postgresql-16-pgvector\n\
                 - macOS: brew install pgvector"
            ));
        }

        // Check if pgvector extension is enabled
        let result = sqlx::query(
            "SELECT EXISTS(
                SELECT 1 FROM pg_extension 
                WHERE extname = 'vector'
            ) as enabled"
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to check if pgvector extension is enabled")?;

        let enabled: bool = result.try_get("enabled")
            .context("Failed to parse pgvector extension enabled check result")?;

        if !enabled {
            warn!("pgvector extension is available but not enabled, it will be enabled during migrations");
        } else {
            info!("pgvector extension is installed and enabled");
        }

        Ok(())
    }

    /// Rollback the last migration (for development/testing)
    pub async fn rollback_last_migration(&self) -> Result<()> {
        info!("Rolling back last migration");

        // Note: sqlx doesn't have built-in rollback support
        // This is a placeholder for manual rollback implementation
        warn!("Migration rollback is not fully implemented yet");
        warn!("To rollback manually, you need to:");
        warn!("1. Identify the last applied migration from _sqlx_migrations table");
        warn!("2. Write and execute the corresponding down migration SQL");
        warn!("3. Delete the migration record from _sqlx_migrations table");

        Ok(())
    }

    /// Get the current migration version
    pub async fn get_current_version(&self) -> Result<Option<i64>> {
        let result = sqlx::query(
            "SELECT version FROM _sqlx_migrations 
             ORDER BY version DESC 
             LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get current migration version")?;

        match result {
            Some(row) => {
                let version: i64 = row.try_get("version")
                    .context("Failed to parse migration version")?;
                Ok(Some(version))
            }
            None => Ok(None),
        }
    }

    /// List all applied migrations
    pub async fn list_applied_migrations(&self) -> Result<Vec<MigrationInfo>> {
        let rows = sqlx::query(
            "SELECT version, description, installed_on, success 
             FROM _sqlx_migrations 
             ORDER BY version ASC"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list applied migrations")?;

        let migrations = rows
            .iter()
            .map(|row| {
                Ok(MigrationInfo {
                    version: row.try_get("version")?,
                    description: row.try_get("description")?,
                    installed_on: row.try_get("installed_on")?,
                    success: row.try_get("success")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(migrations)
    }
}

/// Information about an applied migration
#[derive(Debug, Clone)]
pub struct MigrationInfo {
    pub version: i64,
    pub description: String,
    pub installed_on: chrono::DateTime<chrono::Utc>,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_config_default() {
        let config = MigrationConfig::default();
        assert!(config.auto_run);
        assert!(config.validate_pgvector);
    }

    #[test]
    fn test_migration_config_custom() {
        let config = MigrationConfig {
            auto_run: false,
            validate_pgvector: false,
        };
        assert!(!config.auto_run);
        assert!(!config.validate_pgvector);
    }
}
