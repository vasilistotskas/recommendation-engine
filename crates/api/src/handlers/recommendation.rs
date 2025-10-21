use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use recommendation_models::{Algorithm, RecommendationRequest, TenantContext};
use tracing::{debug, error};

use crate::{
    dto::{
        EntityRecommendationsQuery, RecommendationResponse, TrendingEntitiesQuery,
        TrendingEntitiesResponse, UserRecommendationsQuery,
    },
    error::ApiError,
    state::AppState,
};

/// Get user recommendations endpoint
/// GET /api/v1/recommendations/user/:id
///
/// Query parameters:
/// - algorithm: "collaborative", "content_based", or "hybrid" (default: "hybrid")
/// - count: number of recommendations (default: 10, max: 100)
/// - tenant_id: optional tenant identifier for multi-tenancy
/// - filters: additional filters (e.g., entity_type=product, category=electronics)
pub async fn get_user_recommendations(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<UserRecommendationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "GET /api/v1/recommendations/user/{} - algorithm={}, count={}",
        user_id, query.algorithm, query.count
    );

    // Parse algorithm from query string
    let algorithm = parse_algorithm(&query.algorithm, &query.filters)?;

    // Extract tenant_id (use "default" if not provided)
    let tenant_id = query.tenant_id.unwrap_or_else(|| "default".to_string());
    let ctx = TenantContext { tenant_id };

    // Build recommendation request
    let request = RecommendationRequest {
        user_id: Some(user_id.clone()),
        entity_id: None,
        algorithm,
        count: query.count,
        filters: if query.filters.is_empty() {
            None
        } else {
            Some(query.filters)
        },
    };

    // Get recommendations from service
    match state
        .recommendation_service
        .get_recommendations(&ctx, request)
        .await
    {
        Ok(response) => {
            debug!(
                "Successfully generated {} recommendations for user {}",
                response.recommendations.len(),
                user_id
            );
            Ok((StatusCode::OK, Json(RecommendationResponse::from(response))))
        }
        Err(e) => {
            error!("Failed to get recommendations for user {}: {}", user_id, e);
            Err(ApiError::from(e))
        }
    }
}

/// Get similar entities endpoint
/// GET /api/v1/recommendations/entity/:id
///
/// Query parameters:
/// - algorithm: "content_based" or "hybrid" (default: "content_based")
/// - count: number of recommendations (default: 10, max: 100)
/// - tenant_id: optional tenant identifier for multi-tenancy
/// - entity_type: required entity type for filtering
pub async fn get_similar_entities(
    State(state): State<AppState>,
    Path(entity_id): Path<String>,
    Query(query): Query<EntityRecommendationsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "GET /api/v1/recommendations/entity/{} - algorithm={}, count={}",
        entity_id, query.algorithm, query.count
    );

    // Parse algorithm from query string
    let mut filters = std::collections::HashMap::new();
    if let Some(entity_type) = query.entity_type {
        filters.insert("entity_type".to_string(), entity_type);
    }

    let algorithm = parse_algorithm(&query.algorithm, &filters)?;

    // Extract tenant_id (use "default" if not provided)
    let tenant_id = query.tenant_id.unwrap_or_else(|| "default".to_string());
    let ctx = TenantContext { tenant_id };

    // Build recommendation request
    let request = RecommendationRequest {
        user_id: None,
        entity_id: Some(entity_id.clone()),
        algorithm,
        count: query.count,
        filters: if filters.is_empty() {
            None
        } else {
            Some(filters)
        },
    };

    // Get recommendations from service
    match state
        .recommendation_service
        .get_recommendations(&ctx, request)
        .await
    {
        Ok(response) => {
            debug!(
                "Successfully generated {} similar entities for entity {}",
                response.recommendations.len(),
                entity_id
            );
            Ok((StatusCode::OK, Json(RecommendationResponse::from(response))))
        }
        Err(e) => {
            error!(
                "Failed to get similar entities for entity {}: {}",
                entity_id, e
            );
            Err(ApiError::from(e))
        }
    }
}

/// Get trending entities endpoint
/// GET /api/v1/recommendations/trending
///
/// Query parameters:
/// - entity_type: optional entity type filter
/// - count: number of trending entities (default: 10, max: 100)
/// - tenant_id: optional tenant identifier for multi-tenancy
pub async fn get_trending_entities(
    State(state): State<AppState>,
    Query(query): Query<TrendingEntitiesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "GET /api/v1/recommendations/trending - entity_type={:?}, count={}",
        query.entity_type, query.count
    );

    // Extract tenant_id (use "default" if not provided)
    let tenant_id = query.tenant_id.unwrap_or_else(|| "default".to_string());
    let ctx = TenantContext { tenant_id };

    // Get trending entities from service
    match state
        .recommendation_service
        .get_trending_entities(&ctx, query.entity_type.as_deref(), query.count)
        .await
    {
        Ok(trending) => {
            debug!(
                "Successfully retrieved {} trending entities",
                trending.len()
            );
            let response = TrendingEntitiesResponse {
                count: trending.len(),
                trending: trending
                    .into_iter()
                    .map(crate::dto::ScoredEntityResponse::from)
                    .collect(),
            };
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            error!("Failed to get trending entities: {}", e);
            Err(ApiError::from(e))
        }
    }
}

/// Parse algorithm string into Algorithm enum
fn parse_algorithm(
    algorithm_str: &str,
    filters: &std::collections::HashMap<String, String>,
) -> Result<Algorithm, ApiError> {
    match algorithm_str.to_lowercase().as_str() {
        "collaborative" => Ok(Algorithm::Collaborative),
        "content_based" => Ok(Algorithm::ContentBased),
        "hybrid" => {
            // Parse weights from filters if provided, otherwise use defaults
            let collaborative_weight = filters
                .get("collaborative_weight")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.6);

            let content_weight = filters
                .get("content_weight")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.4);

            Ok(Algorithm::Hybrid {
                collaborative_weight,
                content_weight,
            })
        }
        _ => Err(ApiError::BadRequest(format!(
            "Invalid algorithm: {}. Must be one of: collaborative, content_based, hybrid",
            algorithm_str
        ))),
    }
}
