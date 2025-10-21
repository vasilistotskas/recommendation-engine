use recommendation_service::{EntityService, InteractionService, InteractionTypeService, RecommendationService};
use recommendation_storage::{VectorStore, RedisCache};
use std::sync::Arc;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub entity_service: Arc<EntityService>,
    pub interaction_service: Arc<InteractionService>,
    pub interaction_type_service: Arc<InteractionTypeService>,
    pub recommendation_service: Arc<RecommendationService>,
    pub vector_store: Arc<VectorStore>,
    pub redis_cache: Arc<RedisCache>,
    pub default_tenant_id: String,
}

impl AppState {
    pub fn new(
        entity_service: Arc<EntityService>,
        interaction_service: Arc<InteractionService>,
        interaction_type_service: Arc<InteractionTypeService>,
        recommendation_service: Arc<RecommendationService>,
        vector_store: Arc<VectorStore>,
        redis_cache: Arc<RedisCache>,
        default_tenant_id: String,
    ) -> Self {
        Self {
            entity_service,
            interaction_service,
            interaction_type_service,
            recommendation_service,
            vector_store,
            redis_cache,
            default_tenant_id,
        }
    }
}
