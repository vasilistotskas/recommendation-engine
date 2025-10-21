use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use recommendation_models::RecommendationError;
use serde::Serialize;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    Internal(anyhow::Error),
    Recommendation(RecommendationError),
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Internal(err) => {
                tracing::error!("Internal error: {:?}", err);
                let body = Json(ErrorResponse {
                    error: ErrorDetail {
                        code: "INTERNAL_ERROR".to_string(),
                        message: "Internal server error".to_string(),
                    },
                });
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            ApiError::Recommendation(err) => {
                let status_code = StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                
                let body = Json(ErrorResponse {
                    error: ErrorDetail {
                        code: err.error_code().to_string(),
                        message: err.to_string(),
                    },
                });

                (status_code, body).into_response()
            }
            ApiError::BadRequest(message) => {
                let body = Json(ErrorResponse {
                    error: ErrorDetail {
                        code: "BAD_REQUEST".to_string(),
                        message,
                    },
                });
                (StatusCode::BAD_REQUEST, body).into_response()
            }
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err)
    }
}

impl From<RecommendationError> for ApiError {
    fn from(err: RecommendationError) -> Self {
        ApiError::Recommendation(err)
    }
}
