use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for the recommendation engine
#[derive(Debug, Error)]
pub enum RecommendationError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Vector operation error: {0}")]
    VectorError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Tenant not found: {0}")]
    TenantNotFound(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error")]
    InternalError,
}

impl RecommendationError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Self::EntityNotFound(_) | Self::UserNotFound(_) | Self::TenantNotFound(_) => 404,
            Self::InvalidRequest(_) | Self::ValidationError(_) => 400,
            Self::AuthError(_) => 401,
            Self::RateLimitExceeded => 429,
            Self::DatabaseError(_)
            | Self::CacheError(_)
            | Self::VectorError(_)
            | Self::ConfigError(_)
            | Self::InternalError => 500,
        }
    }

    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::EntityNotFound(_) => "ENTITY_NOT_FOUND",
            Self::UserNotFound(_) => "USER_NOT_FOUND",
            Self::TenantNotFound(_) => "TENANT_NOT_FOUND",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::CacheError(_) => "CACHE_ERROR",
            Self::VectorError(_) => "VECTOR_ERROR",
            Self::AuthError(_) => "AUTH_ERROR",
            Self::ConfigError(_) => "CONFIG_ERROR",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    /// Convert to error response
    pub fn to_response(&self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                request_id,
                timestamp: Utc::now(),
            },
        }
    }
}

/// Error response structure for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

/// Detailed error information
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// Conversion implementations for common error types

impl From<sqlx::Error> for RecommendationError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                Self::EntityNotFound("Record not found in database".to_string())
            }
            _ => Self::DatabaseError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for RecommendationError {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidRequest(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for RecommendationError {
    fn from(_err: std::io::Error) -> Self {
        Self::InternalError
    }
}

impl From<validator::ValidationErrors> for RecommendationError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError(err.to_string())
    }
}

/// Result type alias for recommendation operations
pub type Result<T> = std::result::Result<T, RecommendationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            RecommendationError::EntityNotFound("test".to_string()).status_code(),
            404
        );
        assert_eq!(
            RecommendationError::InvalidRequest("test".to_string()).status_code(),
            400
        );
        assert_eq!(
            RecommendationError::AuthError("test".to_string()).status_code(),
            401
        );
        assert_eq!(RecommendationError::RateLimitExceeded.status_code(), 429);
        assert_eq!(RecommendationError::InternalError.status_code(), 500);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            RecommendationError::EntityNotFound("test".to_string()).error_code(),
            "ENTITY_NOT_FOUND"
        );
        assert_eq!(
            RecommendationError::ValidationError("test".to_string()).error_code(),
            "VALIDATION_ERROR"
        );
    }

    #[test]
    fn test_error_response() {
        let error = RecommendationError::EntityNotFound("entity_123".to_string());
        let response = error.to_response(Some("req_abc".to_string()));

        assert_eq!(response.error.code, "ENTITY_NOT_FOUND");
        assert_eq!(response.error.request_id, Some("req_abc".to_string()));
        assert!(response.error.message.contains("entity_123"));
    }

    #[test]
    fn test_error_serialization() {
        let error = RecommendationError::InvalidRequest("Bad input".to_string());
        let response = error.to_response(Some("req_123".to_string()));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("INVALID_REQUEST"));
        assert!(json.contains("Bad input"));
        assert!(json.contains("req_123"));
    }
}
