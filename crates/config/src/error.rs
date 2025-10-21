use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration loading error: {0}")]
    LoadError(String),

    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),

    #[error("Configuration file error: {0}")]
    FileError(#[from] config::ConfigError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Tenant not found: {0}")]
    TenantNotFound(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;
