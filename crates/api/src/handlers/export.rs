use axum::{
    extract::{Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use recommendation_models::TenantContext;
use serde::Deserialize;
use tracing::{debug, error};

use crate::{error::ApiError, state::AppState};

/// Query parameters for entity export
#[derive(Debug, Deserialize)]
pub struct ExportEntitiesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified_after: Option<DateTime<Utc>>,
}

/// Query parameters for interaction export
#[derive(Debug, Deserialize)]
pub struct ExportInteractionsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// Query parameters for user export
#[derive(Debug, Deserialize)]
pub struct ExportUsersQuery {
    #[serde(default)]
    pub include_vectors: bool,
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

fn default_format() -> String {
    "json".to_string()
}

/// Export entities endpoint
/// GET /api/v1/export/entities
///
/// Query parameters:
/// - entity_type: optional filter by entity type
/// - format: "json" or "csv" (default: "json")
/// - tenant_id: optional tenant identifier
/// - last_modified_after: optional timestamp for incremental exports
pub async fn export_entities(
    State(state): State<AppState>,
    Query(query): Query<ExportEntitiesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "GET /api/v1/export/entities - entity_type={:?}, format={}",
        query.entity_type, query.format
    );

    // Validate format
    if query.format != "json" && query.format != "csv" {
        return Err(ApiError::BadRequest(
            "format must be 'json' or 'csv'".to_string(),
        ));
    }

    // Extract tenant_id
    let tenant_id = query.tenant_id.unwrap_or_else(|| "default".to_string());
    let ctx = TenantContext { tenant_id };

    // Get entities from service
    match state
        .entity_service
        .export_entities(
            &ctx,
            query.entity_type.as_deref(),
            query.last_modified_after,
        )
        .await
    {
        Ok(entities) => {
            debug!("Exporting {} entities", entities.len());

            if query.format == "json" {
                // Return JSON format
                let json_data = serde_json::to_string_pretty(&entities)
                    .map_err(|e| ApiError::Internal(e.into()))?;

                Ok((
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "application/json"),
                        (
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"entities.json\"",
                        ),
                    ],
                    json_data,
                ))
            } else {
                // Return CSV format
                let csv_data = entities_to_csv(&entities)?;

                Ok((
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "text/csv"),
                        (
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"entities.csv\"",
                        ),
                    ],
                    csv_data,
                ))
            }
        }
        Err(e) => {
            error!("Failed to export entities: {}", e);
            Err(ApiError::from(e))
        }
    }
}

/// Export interactions endpoint
/// GET /api/v1/export/interactions
///
/// Query parameters:
/// - start_date: optional start date for filtering
/// - end_date: optional end date for filtering
/// - format: "json" or "csv" (default: "json")
/// - tenant_id: optional tenant identifier
pub async fn export_interactions(
    State(state): State<AppState>,
    Query(query): Query<ExportInteractionsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "GET /api/v1/export/interactions - start_date={:?}, end_date={:?}, format={}",
        query.start_date, query.end_date, query.format
    );

    // Validate format
    if query.format != "json" && query.format != "csv" {
        return Err(ApiError::BadRequest(
            "format must be 'json' or 'csv'".to_string(),
        ));
    }

    // Extract tenant_id
    let tenant_id = query.tenant_id.unwrap_or_else(|| "default".to_string());
    let ctx = TenantContext { tenant_id };

    // Get interactions from service
    match state
        .interaction_service
        .export_interactions(&ctx, query.start_date, query.end_date)
        .await
    {
        Ok(interactions) => {
            debug!("Exporting {} interactions", interactions.len());

            if query.format == "json" {
                // Return JSON format
                let json_data = serde_json::to_string_pretty(&interactions)
                    .map_err(|e| ApiError::Internal(e.into()))?;

                Ok((
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "application/json"),
                        (
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"interactions.json\"",
                        ),
                    ],
                    json_data,
                ))
            } else {
                // Return CSV format
                let csv_data = interactions_to_csv(&interactions)?;

                Ok((
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "text/csv"),
                        (
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"interactions.csv\"",
                        ),
                    ],
                    csv_data,
                ))
            }
        }
        Err(e) => {
            error!("Failed to export interactions: {}", e);
            Err(ApiError::from(e))
        }
    }
}

/// Export users endpoint
/// GET /api/v1/export/users
///
/// Query parameters:
/// - include_vectors: whether to include preference vectors (default: false)
/// - format: "json" or "csv" (default: "json")
/// - tenant_id: optional tenant identifier
pub async fn export_users(
    State(state): State<AppState>,
    Query(query): Query<ExportUsersQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(
        "GET /api/v1/export/users - include_vectors={}, format={}",
        query.include_vectors, query.format
    );

    // Validate format
    if query.format != "json" && query.format != "csv" {
        return Err(ApiError::BadRequest(
            "format must be 'json' or 'csv'".to_string(),
        ));
    }

    // Extract tenant_id
    let tenant_id = query.tenant_id.unwrap_or_else(|| "default".to_string());
    let ctx = TenantContext { tenant_id };

    // Get user profiles from service
    match state
        .interaction_service
        .export_user_profiles(&ctx, query.include_vectors)
        .await
    {
        Ok(users) => {
            debug!("Exporting {} user profiles", users.len());

            if query.format == "json" {
                // Return JSON format
                let json_data = serde_json::to_string_pretty(&users)
                    .map_err(|e| ApiError::Internal(e.into()))?;

                Ok((
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "application/json"),
                        (
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"users.json\"",
                        ),
                    ],
                    json_data,
                ))
            } else {
                // Return CSV format
                let csv_data = users_to_csv(&users, query.include_vectors)?;

                Ok((
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "text/csv"),
                        (
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"users.csv\"",
                        ),
                    ],
                    csv_data,
                ))
            }
        }
        Err(e) => {
            error!("Failed to export user profiles: {}", e);
            Err(ApiError::from(e))
        }
    }
}

/// Convert entities to CSV format
fn entities_to_csv(entities: &[recommendation_models::Entity]) -> Result<String, ApiError> {
    let mut csv =
        String::from("entity_id,entity_type,attributes,tenant_id,created_at,updated_at\n");

    for entity in entities {
        let attributes_json =
            serde_json::to_string(&entity.attributes).map_err(|e| ApiError::Internal(e.into()))?;

        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            entity.entity_id,
            entity.entity_type,
            attributes_json.replace('"', "\"\""), // Escape quotes
            entity.tenant_id.as_deref().unwrap_or(""),
            entity.created_at.to_rfc3339(),
            entity.updated_at.to_rfc3339()
        ));
    }

    Ok(csv)
}

/// Convert interactions to CSV format
fn interactions_to_csv(
    interactions: &[recommendation_models::Interaction],
) -> Result<String, ApiError> {
    let mut csv =
        String::from("user_id,entity_id,interaction_type,weight,metadata,tenant_id,timestamp\n");

    for interaction in interactions {
        let metadata_json = if let Some(ref metadata) = interaction.metadata {
            serde_json::to_string(metadata).map_err(|e| ApiError::Internal(e.into()))?
        } else {
            String::new()
        };

        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{},\"{}\",\"{}\",\"{}\"\n",
            interaction.user_id,
            interaction.entity_id,
            serde_json::to_string(&interaction.interaction_type)
                .map_err(|e| ApiError::Internal(e.into()))?
                .trim_matches('"'),
            interaction.weight,
            metadata_json.replace('"', "\"\""), // Escape quotes
            interaction.tenant_id.as_deref().unwrap_or(""),
            interaction.timestamp.to_rfc3339()
        ));
    }

    Ok(csv)
}

/// Convert user profiles to CSV format
fn users_to_csv(
    users: &[recommendation_models::UserProfile],
    include_vectors: bool,
) -> Result<String, ApiError> {
    let mut csv = if include_vectors {
        String::from(
            "user_id,interaction_count,preference_vector,tenant_id,last_interaction_at,created_at,updated_at\n",
        )
    } else {
        String::from(
            "user_id,interaction_count,tenant_id,last_interaction_at,created_at,updated_at\n",
        )
    };

    for user in users {
        if include_vectors {
            let vector_json = serde_json::to_string(&user.preference_vector)
                .map_err(|e| ApiError::Internal(e.into()))?;

            csv.push_str(&format!(
                "\"{}\",{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                user.user_id,
                user.interaction_count,
                vector_json.replace('"', "\"\""), // Escape quotes
                user.tenant_id.as_deref().unwrap_or(""),
                user.last_interaction_at
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default(),
                user.created_at.to_rfc3339(),
                user.updated_at.to_rfc3339()
            ));
        } else {
            csv.push_str(&format!(
                "\"{}\",{},\"{}\",\"{}\",\"{}\",\"{}\"\n",
                user.user_id,
                user.interaction_count,
                user.tenant_id.as_deref().unwrap_or(""),
                user.last_interaction_at
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default(),
                user.created_at.to_rfc3339(),
                user.updated_at.to_rfc3339()
            ));
        }
    }

    Ok(csv)
}
