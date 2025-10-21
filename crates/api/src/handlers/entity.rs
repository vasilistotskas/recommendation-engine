use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use recommendation_models::TenantContext;
use serde::Deserialize;
use tracing::{debug, info};

use crate::{
    dto::{
        BulkImportEntitiesRequest, BulkImportErrorDetail, BulkImportResponse, CreateEntityRequest,
        EntityResponse, UpdateEntityRequest,
    },
    error::ApiResult,
    state::AppState,
};

/// Query parameters for entity operations
#[derive(Debug, Deserialize)]
pub struct EntityQuery {
    #[serde(default)]
    pub entity_type: Option<String>,
}

/// Create entity endpoint
/// POST /api/v1/entities
/// Requirements: 1.1, 21.1, 21.5
pub async fn create_entity(
    State(state): State<AppState>,
    Json(request): Json<CreateEntityRequest>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "POST /api/v1/entities - entity_id={}, entity_type={}, tenant_id={:?}",
        request.entity_id, request.entity_type, request.tenant_id
    );

    // Extract tenant_id from request or use default
    let tenant_ctx = TenantContext::from(request.tenant_id.clone());

    // Call EntityService.create_entity
    let entity = state
        .entity_service
        .create_entity(
            &tenant_ctx,
            request.entity_id.clone(),
            request.entity_type.clone(),
            request.attributes,
        )
        .await?;

    info!(
        "Created entity - entity_id={}, entity_type={}, tenant_id={}",
        entity.entity_id, entity.entity_type, tenant_ctx.tenant_id
    );

    // Return 201 with created entity
    Ok((StatusCode::CREATED, Json(EntityResponse::from(entity))))
}

/// Get entity endpoint
/// GET /api/v1/entities/:id
/// Requirements: 1.1
pub async fn get_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EntityQuery>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "GET /api/v1/entities/{} - entity_type={:?}",
        id, query.entity_type
    );

    // For now, use default tenant (will be enhanced with tenant extraction from headers)
    let tenant_ctx = TenantContext::default();

    // entity_type is required for get_entity
    let entity_type = query.entity_type.ok_or_else(|| {
        recommendation_models::RecommendationError::InvalidRequest(
            "entity_type query parameter is required".to_string(),
        )
    })?;

    // Call EntityService.get_entity
    let entity = state
        .entity_service
        .get_entity(&tenant_ctx, id.clone(), entity_type)
        .await?;

    match entity {
        Some(entity) => {
            info!("Found entity - entity_id={}", entity.entity_id);
            Ok((StatusCode::OK, Json(EntityResponse::from(entity))))
        }
        None => {
            info!("Entity not found - entity_id={}", id);
            Err(recommendation_models::RecommendationError::EntityNotFound(format!(
                "Entity with id '{}' not found",
                id
            ))
            .into())
        }
    }
}

/// Update entity endpoint
/// PUT /api/v1/entities/:id
/// Requirements: 1.2
pub async fn update_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EntityQuery>,
    Json(request): Json<UpdateEntityRequest>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "PUT /api/v1/entities/{} - entity_type={:?}, tenant_id={:?}",
        id, query.entity_type, request.tenant_id
    );

    // Extract tenant_id from request or use default
    let tenant_ctx = TenantContext::from(request.tenant_id.clone());

    // entity_type is required for update_entity
    let entity_type = query.entity_type.ok_or_else(|| {
        recommendation_models::RecommendationError::InvalidRequest(
            "entity_type query parameter is required".to_string(),
        )
    })?;

    // Call EntityService.update_entity
    let entity = state
        .entity_service
        .update_entity(&tenant_ctx, id.clone(), entity_type, request.attributes)
        .await?;

    info!(
        "Updated entity - entity_id={}, tenant_id={}",
        entity.entity_id, tenant_ctx.tenant_id
    );

    // Return 200 with updated entity
    Ok((StatusCode::OK, Json(EntityResponse::from(entity))))
}

/// Delete entity endpoint
/// DELETE /api/v1/entities/:id
/// Requirements: 1.3
pub async fn delete_entity(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<EntityQuery>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "DELETE /api/v1/entities/{} - entity_type={:?}",
        id, query.entity_type
    );

    // For now, use default tenant (will be enhanced with tenant extraction from headers)
    let tenant_ctx = TenantContext::default();

    // entity_type is required for delete_entity
    let entity_type = query.entity_type.ok_or_else(|| {
        recommendation_models::RecommendationError::InvalidRequest(
            "entity_type query parameter is required".to_string(),
        )
    })?;

    // Call EntityService.delete_entity
    state
        .entity_service
        .delete_entity(&tenant_ctx, id.clone(), entity_type)
        .await?;

    info!("Deleted entity - entity_id={}", id);

    // Return 204 No Content
    Ok(StatusCode::NO_CONTENT)
}

/// Bulk import entities endpoint
/// POST /api/v1/entities/bulk
/// Requirements: 24.1, 24.3
pub async fn bulk_import_entities(
    State(state): State<AppState>,
    Json(request): Json<BulkImportEntitiesRequest>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "POST /api/v1/entities/bulk - count={}, tenant_id={:?}",
        request.entities.len(),
        request.tenant_id
    );

    // Extract tenant_id from request or use default
    let tenant_ctx = TenantContext::from(request.tenant_id.clone());

    // Convert request entities to service format
    let entities: Vec<_> = request
        .entities
        .into_iter()
        .map(|item| (item.entity_id, item.entity_type, item.attributes))
        .collect();

    // Call EntityService.bulk_import
    let result = state
        .entity_service
        .bulk_import_entities(&tenant_ctx, entities)
        .await?;

    info!(
        "Bulk import completed - job_id={}, total={}, successful={}, failed={}",
        result.job_id, result.total_records, result.successful, result.failed
    );

    // Convert service result to API response
    let response = BulkImportResponse {
        job_id: result.job_id,
        status: match result.status {
            recommendation_service::entity::ImportStatus::Completed => "completed".to_string(),
            recommendation_service::entity::ImportStatus::PartiallyCompleted => {
                "partially_completed".to_string()
            }
            recommendation_service::entity::ImportStatus::Failed => "failed".to_string(),
        },
        total_records: result.total_records,
        processed: result.processed,
        successful: result.successful,
        failed: result.failed,
        errors: result
            .errors
            .into_iter()
            .map(|e| BulkImportErrorDetail {
                entity_id: e.entity_id,
                entity_type: e.entity_type,
                error: e.error,
            })
            .collect(),
    };

    // Return 202 with job ID
    Ok((StatusCode::ACCEPTED, Json(response)))
}
