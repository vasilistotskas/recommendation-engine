use recommendation_service::{
    EntityService, InteractionService, InteractionTypeService, RecommendationService,
};
use recommendation_storage::{RedisCache, VectorStore};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
    pub metrics_handle: metrics_exporter_prometheus::PrometheusHandle,
    /// Flag to indicate if the service is shutting down
    pub is_shutting_down: Arc<AtomicBool>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_service: Arc<EntityService>,
        interaction_service: Arc<InteractionService>,
        interaction_type_service: Arc<InteractionTypeService>,
        recommendation_service: Arc<RecommendationService>,
        vector_store: Arc<VectorStore>,
        redis_cache: Arc<RedisCache>,
        default_tenant_id: String,
        metrics_handle: metrics_exporter_prometheus::PrometheusHandle,
    ) -> Self {
        Self {
            entity_service,
            interaction_service,
            interaction_type_service,
            recommendation_service,
            vector_store,
            redis_cache,
            default_tenant_id,
            metrics_handle,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get reference to vector store for metrics
    pub fn vector_store(&self) -> &Arc<VectorStore> {
        &self.vector_store
    }

    /// Check if the service is shutting down
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Relaxed)
    }

    /// Mark the service as shutting down
    pub fn set_shutting_down(&self) {
        self.is_shutting_down.store(true, Ordering::Relaxed);
        tracing::info!("Service marked as shutting down - readiness probe will return unhealthy");
    }
}
