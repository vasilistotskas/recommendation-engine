use recommendation_models::{
    RegisteredInteractionType, TenantContext, Result,
};
use recommendation_storage::VectorStore;
use std::sync::Arc;
use tracing::{info, debug};

/// Service for managing custom interaction types
pub struct InteractionTypeService {
    vector_store: Arc<VectorStore>,
}

impl InteractionTypeService {
    /// Create a new InteractionTypeService
    pub fn new(vector_store: Arc<VectorStore>) -> Self {
        Self { vector_store }
    }

    /// Register a new custom interaction type
    pub async fn register_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: String,
        weight: f32,
        description: Option<String>,
    ) -> Result<RegisteredInteractionType> {
        debug!(
            "InteractionTypeService: Registering interaction type - tenant={}, type={}, weight={}",
            ctx.tenant_id, interaction_type, weight
        );

        let registered = self.vector_store
            .register_interaction_type(ctx, &interaction_type, weight, description)
            .await?;

        info!(
            "InteractionTypeService: Registered interaction type - tenant={}, type={}, weight={}",
            ctx.tenant_id, interaction_type, weight
        );

        Ok(registered)
    }

    /// Get a registered interaction type
    pub async fn get_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: String,
    ) -> Result<Option<RegisteredInteractionType>> {
        debug!(
            "InteractionTypeService: Getting interaction type - tenant={}, type={}",
            ctx.tenant_id, interaction_type
        );

        self.vector_store
            .get_interaction_type(ctx, &interaction_type)
            .await
    }

    /// List all registered interaction types
    pub async fn list_interaction_types(
        &self,
        ctx: &TenantContext,
    ) -> Result<Vec<RegisteredInteractionType>> {
        debug!(
            "InteractionTypeService: Listing interaction types - tenant={}",
            ctx.tenant_id
        );

        self.vector_store
            .list_interaction_types(ctx)
            .await
    }

    /// Update an existing interaction type
    pub async fn update_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: String,
        weight: f32,
        description: Option<String>,
    ) -> Result<RegisteredInteractionType> {
        debug!(
            "InteractionTypeService: Updating interaction type - tenant={}, type={}, weight={}",
            ctx.tenant_id, interaction_type, weight
        );

        let updated = self.vector_store
            .update_interaction_type(ctx, &interaction_type, weight, description)
            .await?;

        info!(
            "InteractionTypeService: Updated interaction type - tenant={}, type={}, weight={}",
            ctx.tenant_id, interaction_type, weight
        );

        Ok(updated)
    }

    /// Delete an interaction type
    pub async fn delete_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: String,
    ) -> Result<()> {
        debug!(
            "InteractionTypeService: Deleting interaction type - tenant={}, type={}",
            ctx.tenant_id, interaction_type
        );

        self.vector_store
            .delete_interaction_type(ctx, &interaction_type)
            .await?;

        info!(
            "InteractionTypeService: Deleted interaction type - tenant={}, type={}",
            ctx.tenant_id, interaction_type
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interaction_type_service_creation() {
        let pool = sqlx::PgPool::connect_lazy("postgresql://localhost/test_db").unwrap();
        let vector_store = Arc::new(VectorStore::new(pool));
        let _service = InteractionTypeService::new(vector_store);
        // Service created successfully
    }
}
