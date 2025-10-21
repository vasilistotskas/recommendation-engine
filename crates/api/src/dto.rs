use recommendation_models::{AttributeValue, Entity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to create a new entity
#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub entity_id: String,
    pub entity_type: String,
    pub attributes: HashMap<String, AttributeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// Request to update an existing entity
#[derive(Debug, Deserialize)]
pub struct UpdateEntityRequest {
    pub attributes: HashMap<String, AttributeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// Response for entity operations
#[derive(Debug, Serialize)]
pub struct EntityResponse {
    pub entity_id: String,
    pub entity_type: String,
    pub attributes: HashMap<String, AttributeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Entity> for EntityResponse {
    fn from(entity: Entity) -> Self {
        Self {
            entity_id: entity.entity_id,
            entity_type: entity.entity_type,
            attributes: entity.attributes,
            tenant_id: entity.tenant_id,
            created_at: entity.created_at.to_rfc3339(),
            updated_at: entity.updated_at.to_rfc3339(),
        }
    }
}

/// Request for bulk entity import
#[derive(Debug, Deserialize)]
pub struct BulkImportEntitiesRequest {
    pub entities: Vec<BulkEntityItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkEntityItem {
    pub entity_id: String,
    pub entity_type: String,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Response for bulk import operations
#[derive(Debug, Serialize)]
pub struct BulkImportResponse {
    pub job_id: String,
    pub status: String,
    pub total_records: usize,
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BulkImportErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct BulkImportErrorDetail {
    pub entity_id: String,
    pub entity_type: String,
    pub error: String,
}

/// Request to create a new interaction
#[derive(Debug, Deserialize)]
pub struct CreateInteractionRequest {
    pub user_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub interaction_type: recommendation_models::InteractionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response for interaction operations
#[derive(Debug, Serialize)]
pub struct InteractionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub user_id: String,
    pub entity_id: String,
    pub interaction_type: recommendation_models::InteractionType,
    pub weight: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub timestamp: String,
}

impl From<recommendation_models::Interaction> for InteractionResponse {
    fn from(interaction: recommendation_models::Interaction) -> Self {
        Self {
            id: interaction.id,
            user_id: interaction.user_id,
            entity_id: interaction.entity_id,
            interaction_type: interaction.interaction_type,
            weight: interaction.weight,
            metadata: interaction.metadata,
            tenant_id: interaction.tenant_id,
            timestamp: interaction.timestamp.to_rfc3339(),
        }
    }
}

/// Request for bulk interaction import
#[derive(Debug, Deserialize)]
pub struct BulkImportInteractionsRequest {
    pub interactions: Vec<BulkInteractionItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkInteractionItem {
    pub user_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub interaction_type: recommendation_models::InteractionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response for bulk interaction import
#[derive(Debug, Serialize)]
pub struct BulkImportInteractionsResponse {
    pub job_id: String,
    pub status: String,
    pub total_records: usize,
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BulkInteractionImportErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct BulkInteractionImportErrorDetail {
    pub user_id: String,
    pub entity_id: String,
    pub error: String,
}

/// Query parameters for user recommendations
#[derive(Debug, Deserialize)]
pub struct UserRecommendationsQuery {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}

fn default_algorithm() -> String {
    "hybrid".to_string()
}

fn default_count() -> usize {
    10
}

/// Query parameters for entity recommendations (similar entities)
#[derive(Debug, Deserialize)]
pub struct EntityRecommendationsQuery {
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
}

/// Query parameters for trending entities
#[derive(Debug, Deserialize)]
pub struct TrendingEntitiesQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// Response for recommendation requests
#[derive(Debug, Serialize)]
pub struct RecommendationResponse {
    pub recommendations: Vec<ScoredEntityResponse>,
    pub algorithm: String,
    pub cold_start: bool,
    pub generated_at: String,
}

impl From<recommendation_models::RecommendationResponse> for RecommendationResponse {
    fn from(response: recommendation_models::RecommendationResponse) -> Self {
        Self {
            recommendations: response
                .recommendations
                .into_iter()
                .map(ScoredEntityResponse::from)
                .collect(),
            algorithm: response.algorithm,
            cold_start: response.cold_start,
            generated_at: response.generated_at.to_rfc3339(),
        }
    }
}

/// Scored entity in recommendation response
#[derive(Debug, Serialize)]
pub struct ScoredEntityResponse {
    pub entity_id: String,
    pub entity_type: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl From<recommendation_models::ScoredEntity> for ScoredEntityResponse {
    fn from(entity: recommendation_models::ScoredEntity) -> Self {
        Self {
            entity_id: entity.entity_id,
            entity_type: entity.entity_type,
            score: entity.score,
            reason: entity.reason,
        }
    }
}

/// Response for trending entities
#[derive(Debug, Serialize)]
pub struct TrendingEntitiesResponse {
    pub trending: Vec<ScoredEntityResponse>,
    pub count: usize,
}
