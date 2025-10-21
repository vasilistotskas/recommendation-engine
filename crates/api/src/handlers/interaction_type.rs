use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use recommendation_models::{
    ListInteractionTypesResponse, RegisterInteractionTypeRequest, TenantContext,
    UpdateInteractionTypeRequest,
};
use tracing::{debug, error};

use crate::{error::ApiError, state::AppState};

/// Register a new custom interaction type
/// POST /api/v1/interaction-types
pub async fn register_interaction_type(
    State(state): State<AppState>,
    Json(request): Json<RegisterInteractionTypeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "Registering interaction type: type={}, weight={}",
        request.interaction_type, request.weight
    );

    // Validate request
    if request.interaction_type.is_empty() {
        return Err(ApiError::BadRequest(
            "Interaction type cannot be empty".to_string(),
        ));
    }

    if request.weight < 0.0 {
        return Err(ApiError::BadRequest(format!(
            "Weight must be non-negative, got {}",
            request.weight
        )));
    }

    // Create tenant context
    let tenant_id = request
        .tenant_id
        .clone()
        .unwrap_or_else(|| state.default_tenant_id.clone());
    let ctx = TenantContext { tenant_id };

    // Register interaction type
    let registered = state
        .interaction_type_service
        .register_interaction_type(
            &ctx,
            request.interaction_type,
            request.weight,
            request.description,
        )
        .await
        .map_err(|e| {
            error!("Failed to register interaction type: {}", e);
            ApiError::from(e)
        })?;

    Ok((StatusCode::CREATED, Json(registered)))
}

/// Get a specific interaction type
/// GET /api/v1/interaction-types/:type
pub async fn get_interaction_type(
    State(state): State<AppState>,
    Path(interaction_type): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Getting interaction type: type={}", interaction_type);

    // Use default tenant for now (can be extended to support tenant_id query param)
    let ctx = TenantContext {
        tenant_id: state.default_tenant_id.clone(),
    };

    // Get interaction type
    let registered = state
        .interaction_type_service
        .get_interaction_type(&ctx, interaction_type)
        .await
        .map_err(|e| {
            error!("Failed to get interaction type: {}", e);
            ApiError::from(e)
        })?;

    match registered {
        Some(it) => Ok((StatusCode::OK, Json(it))),
        None => Err(ApiError::BadRequest(
            "Interaction type not found".to_string(),
        )),
    }
}

/// List all registered interaction types
/// GET /api/v1/interaction-types
pub async fn list_interaction_types(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Listing interaction types");

    // Use default tenant for now (can be extended to support tenant_id query param)
    let ctx = TenantContext {
        tenant_id: state.default_tenant_id.clone(),
    };

    // List interaction types
    let interaction_types = state
        .interaction_type_service
        .list_interaction_types(&ctx)
        .await
        .map_err(|e| {
            error!("Failed to list interaction types: {}", e);
            ApiError::from(e)
        })?;

    let response = ListInteractionTypesResponse {
        total: interaction_types.len(),
        interaction_types,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Update an existing interaction type
/// PUT /api/v1/interaction-types/:type
pub async fn update_interaction_type(
    State(state): State<AppState>,
    Path(interaction_type): Path<String>,
    Json(request): Json<UpdateInteractionTypeRequest>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "Updating interaction type: type={}, weight={}",
        interaction_type, request.weight
    );

    // Validate request
    if request.weight < 0.0 {
        return Err(ApiError::BadRequest(format!(
            "Weight must be non-negative, got {}",
            request.weight
        )));
    }

    // Use default tenant for now
    let ctx = TenantContext {
        tenant_id: state.default_tenant_id.clone(),
    };

    // Update interaction type
    let updated = state
        .interaction_type_service
        .update_interaction_type(&ctx, interaction_type, request.weight, request.description)
        .await
        .map_err(|e| {
            error!("Failed to update interaction type: {}", e);
            ApiError::from(e)
        })?;

    Ok((StatusCode::OK, Json(updated)))
}

/// Delete an interaction type
/// DELETE /api/v1/interaction-types/:type
pub async fn delete_interaction_type(
    State(state): State<AppState>,
    Path(interaction_type): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Deleting interaction type: type={}", interaction_type);

    // Use default tenant for now
    let ctx = TenantContext {
        tenant_id: state.default_tenant_id.clone(),
    };

    // Delete interaction type
    state
        .interaction_type_service
        .delete_interaction_type(&ctx, interaction_type)
        .await
        .map_err(|e| {
            error!("Failed to delete interaction type: {}", e);
            ApiError::from(e)
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_interaction_type_request() {
        let request = RegisterInteractionTypeRequest {
            interaction_type: "share".to_string(),
            weight: 3.0,
            description: Some("User shared entity".to_string()),
            tenant_id: None,
        };

        assert_eq!(request.interaction_type, "share");
        assert_eq!(request.weight, 3.0);
    }

    #[test]
    fn test_validate_negative_weight() {
        let request = RegisterInteractionTypeRequest {
            interaction_type: "test".to_string(),
            weight: -1.0,
            description: None,
            tenant_id: None,
        };

        assert!(request.weight < 0.0);
    }
}
