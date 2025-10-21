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
        BulkImportInteractionsRequest, BulkImportInteractionsResponse,
        BulkInteractionImportErrorDetail, CreateInteractionRequest, InteractionResponse,
    },
    error::ApiResult,
    state::AppState,
};

/// Query parameters for get user interactions
#[derive(Debug, Deserialize)]
pub struct GetUserInteractionsQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    50
}

/// Create interaction endpoint
/// POST /api/v1/interactions
/// Requirements: 2.1, 21.1
pub async fn create_interaction(
    State(state): State<AppState>,
    Json(request): Json<CreateInteractionRequest>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "POST /api/v1/interactions - user_id={}, entity_id={}, entity_type={}, interaction_type={:?}, tenant_id={:?}",
        request.user_id, request.entity_id, request.entity_type, request.interaction_type, request.tenant_id
    );

    // Extract tenant_id from request or use default
    let tenant_ctx = TenantContext::from(request.tenant_id.clone());

    // Call InteractionService.record_interaction
    let interaction = state
        .interaction_service
        .record_interaction(
            &tenant_ctx,
            request.user_id.clone(),
            request.entity_id.clone(),
            request.entity_type.clone(),
            request.interaction_type.clone(),
            request.metadata,
            request.timestamp,
        )
        .await?;

    info!(
        "Created interaction - user_id={}, entity_id={}, interaction_type={:?}, tenant_id={}",
        interaction.user_id, interaction.entity_id, interaction.interaction_type, tenant_ctx.tenant_id
    );

    // Return 201 with created interaction
    Ok((StatusCode::CREATED, Json(InteractionResponse::from(interaction))))
}

/// Get user interactions endpoint
/// GET /api/v1/interactions/user/:id
/// Requirements: 2.1
pub async fn get_user_interactions(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<GetUserInteractionsQuery>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "GET /api/v1/interactions/user/{} - limit={}, offset={}",
        id, query.limit, query.offset
    );

    // For now, use default tenant (will be enhanced with tenant extraction from headers)
    let tenant_ctx = TenantContext::default();

    // Call InteractionService.get_user_interactions
    let interactions = state
        .interaction_service
        .get_user_interactions(&tenant_ctx, id.clone(), query.limit, query.offset)
        .await?;

    info!(
        "Retrieved {} interactions for user_id={}, tenant_id={}",
        interactions.len(),
        id,
        tenant_ctx.tenant_id
    );

    // Convert to response format
    let response: Vec<InteractionResponse> = interactions
        .into_iter()
        .map(InteractionResponse::from)
        .collect();

    // Return 200 with paginated interactions
    Ok((StatusCode::OK, Json(response)))
}

/// Bulk import interactions endpoint
/// POST /api/v1/interactions/bulk
/// Requirements: 24.2
pub async fn bulk_import_interactions(
    State(state): State<AppState>,
    Json(request): Json<BulkImportInteractionsRequest>,
) -> ApiResult<impl IntoResponse> {
    debug!(
        "POST /api/v1/interactions/bulk - count={}, tenant_id={:?}",
        request.interactions.len(),
        request.tenant_id
    );

    // Extract tenant_id from request or use default
    let tenant_ctx = TenantContext::from(request.tenant_id.clone());

    // Convert request interactions to service format
    let interactions: Vec<_> = request
        .interactions
        .into_iter()
        .map(|item| {
            (
                item.user_id,
                item.entity_id,
                item.entity_type,
                item.interaction_type,
                item.metadata,
                item.timestamp,
            )
        })
        .collect();

    // Call InteractionService.bulk_import
    let result = state
        .interaction_service
        .bulk_import_interactions(&tenant_ctx, interactions)
        .await?;

    info!(
        "Bulk import completed - job_id={}, total={}, successful={}, failed={}",
        result.job_id, result.total_records, result.successful, result.failed
    );

    // Convert service result to API response
    let response = BulkImportInteractionsResponse {
        job_id: result.job_id,
        status: match result.status {
            recommendation_service::interaction::ImportStatus::Completed => "completed".to_string(),
            recommendation_service::interaction::ImportStatus::PartiallyCompleted => {
                "partially_completed".to_string()
            }
            recommendation_service::interaction::ImportStatus::Failed => "failed".to_string(),
        },
        total_records: result.total_records,
        processed: result.processed,
        successful: result.successful,
        failed: result.failed,
        errors: result
            .errors
            .into_iter()
            .map(|e| BulkInteractionImportErrorDetail {
                user_id: e.user_id,
                entity_id: e.entity_id,
                error: e.error,
            })
            .collect(),
    };

    // Return 202 with job ID
    Ok((StatusCode::ACCEPTED, Json(response)))
}
