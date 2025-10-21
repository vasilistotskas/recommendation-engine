use chrono::{DateTime, Utc};
use recommendation_models::{
    Interaction, InteractionType, RecommendationError, Result, TenantContext,
};
use recommendation_storage::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::{debug, info, warn};

/// Service for interaction tracking and user profile management
/// Handles interaction recording, deduplication, user profile updates, and history queries
pub struct InteractionService {
    vector_store: Arc<VectorStore>,
    interaction_weights: HashMap<String, f32>,
}

impl InteractionService {
    /// Create a new InteractionService with the given vector store
    pub fn new(vector_store: Arc<VectorStore>) -> Self {
        // Default interaction weights
        let mut interaction_weights = HashMap::new();
        interaction_weights.insert("view".to_string(), 1.0);
        interaction_weights.insert("add_to_cart".to_string(), 3.0);
        interaction_weights.insert("purchase".to_string(), 5.0);
        interaction_weights.insert("like".to_string(), 2.0);

        Self {
            vector_store,
            interaction_weights,
        }
    }

    /// Create a new InteractionService with custom interaction weights
    pub fn with_weights(
        vector_store: Arc<VectorStore>,
        interaction_weights: HashMap<String, f32>,
    ) -> Self {
        Self {
            vector_store,
            interaction_weights,
        }
    }

    /// Record an interaction with deduplication
    /// Applies configurable interaction weights
    #[allow(clippy::too_many_arguments)]
    pub async fn record_interaction(
        &self,
        ctx: &TenantContext,
        user_id: String,
        entity_id: String,
        entity_type: String,
        interaction_type: InteractionType,
        metadata: Option<HashMap<String, String>>,
        timestamp: Option<DateTime<Utc>>,
    ) -> Result<Interaction> {
        debug!(
            "InteractionService: Recording interaction - tenant={}, user={}, entity={}, type={:?}",
            ctx.tenant_id, user_id, entity_id, interaction_type
        );

        // Validate inputs
        self.validate_user_id(&user_id)?;
        self.validate_entity_id(&entity_id)?;

        // Get weight for interaction type from database registry (with fallback to defaults)
        let weight = self
            .get_interaction_weight_async(ctx, &interaction_type)
            .await?;

        // Use provided timestamp or current time
        let timestamp = timestamp.unwrap_or_else(Utc::now);

        // Record interaction in storage (with deduplication)
        let interaction = self
            .vector_store
            .record_interaction(
                ctx,
                &user_id,
                &entity_id,
                &entity_type,
                &interaction_type,
                weight,
                metadata,
                timestamp,
            )
            .await?;

        info!(
            "InteractionService: Recorded interaction - tenant={}, user={}, entity={}, type={:?}, weight={}",
            ctx.tenant_id, user_id, entity_id, interaction_type, weight
        );

        // Trigger async user profile update (fire and forget)
        let vector_store = Arc::clone(&self.vector_store);
        let ctx_clone = ctx.clone();
        let user_id_clone = user_id.clone();

        tokio::spawn(async move {
            // Wait a bit to allow batching of multiple interactions
            sleep(Duration::from_secs(5)).await;

            if let Err(e) =
                update_user_profile_async(&vector_store, &ctx_clone, &user_id_clone).await
            {
                warn!(
                    "Failed to update user profile asynchronously: tenant={}, user={}, error={}",
                    ctx_clone.tenant_id, user_id_clone, e
                );
            }
        });

        Ok(interaction)
    }

    /// Get interaction history for a user with pagination
    pub async fn get_user_interactions(
        &self,
        ctx: &TenantContext,
        user_id: String,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Interaction>> {
        debug!(
            "InteractionService: Getting user interactions - tenant={}, user={}, limit={}, offset={}",
            ctx.tenant_id, user_id, limit, offset
        );

        self.validate_user_id(&user_id)?;

        let interactions = self
            .vector_store
            .get_user_interactions(ctx, &user_id, limit, offset)
            .await?;

        debug!(
            "InteractionService: Retrieved {} interactions for user={}, tenant={}",
            interactions.len(),
            user_id,
            ctx.tenant_id
        );

        Ok(interactions)
    }

    /// Get interaction history for a user filtered by interaction type
    pub async fn get_user_interactions_by_type(
        &self,
        ctx: &TenantContext,
        user_id: String,
        interaction_type: InteractionType,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Interaction>> {
        debug!(
            "InteractionService: Getting user interactions by type - tenant={}, user={}, type={:?}",
            ctx.tenant_id, user_id, interaction_type
        );

        self.validate_user_id(&user_id)?;

        // Get all interactions and filter by type
        let all_interactions = self
            .vector_store
            .get_user_interactions(ctx, &user_id, limit * 2, offset)
            .await?;

        let filtered: Vec<Interaction> = all_interactions
            .into_iter()
            .filter(|i| matches_interaction_type(&i.interaction_type, &interaction_type))
            .take(limit)
            .collect();

        debug!(
            "InteractionService: Retrieved {} filtered interactions for user={}, tenant={}",
            filtered.len(),
            user_id,
            ctx.tenant_id
        );

        Ok(filtered)
    }

    /// Get interaction history for a user within a date range
    pub async fn get_user_interactions_by_date_range(
        &self,
        ctx: &TenantContext,
        user_id: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Interaction>> {
        debug!(
            "InteractionService: Getting user interactions by date range - tenant={}, user={}, start={}, end={}",
            ctx.tenant_id, user_id, start_date, end_date
        );

        self.validate_user_id(&user_id)?;

        if start_date > end_date {
            return Err(RecommendationError::ValidationError(
                "start_date must be before end_date".to_string(),
            ));
        }

        // Get all interactions and filter by date range
        let all_interactions = self
            .vector_store
            .get_user_interactions(ctx, &user_id, limit * 2, offset)
            .await?;

        let filtered: Vec<Interaction> = all_interactions
            .into_iter()
            .filter(|i| i.timestamp >= start_date && i.timestamp <= end_date)
            .take(limit)
            .collect();

        debug!(
            "InteractionService: Retrieved {} interactions in date range for user={}, tenant={}",
            filtered.len(),
            user_id,
            ctx.tenant_id
        );

        Ok(filtered)
    }

    /// Bulk import interactions in batches
    #[allow(clippy::type_complexity)]
    pub async fn bulk_import_interactions(
        &self,
        ctx: &TenantContext,
        interactions: Vec<(
            String,
            String,
            String,
            InteractionType,
            Option<HashMap<String, String>>,
            Option<DateTime<Utc>>,
        )>,
    ) -> Result<BulkImportResult> {
        info!(
            "InteractionService: Starting bulk import - tenant={}, count={}",
            ctx.tenant_id,
            interactions.len()
        );

        let total_count = interactions.len();
        let mut processed_count = 0;
        let mut success_count = 0;
        let mut errors = Vec::new();

        // Process in batches of 1000
        const BATCH_SIZE: usize = 1000;

        for (batch_idx, batch) in interactions.chunks(BATCH_SIZE).enumerate() {
            debug!(
                "InteractionService: Processing batch {} of {} (size: {})",
                batch_idx + 1,
                total_count.div_ceil(BATCH_SIZE),
                batch.len()
            );

            let mut batch_interactions = Vec::new();

            for (user_id, entity_id, entity_type, interaction_type, metadata, timestamp) in batch {
                processed_count += 1;

                // Validate interaction
                if let Err(e) = self.validate_user_id(user_id) {
                    errors.push(BulkImportError {
                        user_id: user_id.clone(),
                        entity_id: entity_id.clone(),
                        error: e.to_string(),
                    });
                    continue;
                }

                if let Err(e) = self.validate_entity_id(entity_id) {
                    errors.push(BulkImportError {
                        user_id: user_id.clone(),
                        entity_id: entity_id.clone(),
                        error: e.to_string(),
                    });
                    continue;
                }

                // Get weight for interaction type
                let weight = self.get_interaction_weight(interaction_type);

                // Use provided timestamp or current time
                let ts = timestamp.unwrap_or_else(Utc::now);

                batch_interactions.push((
                    user_id.clone(),
                    entity_id.clone(),
                    entity_type.clone(),
                    interaction_type.clone(),
                    weight,
                    metadata.clone(),
                    ts,
                ));
            }

            // Batch insert valid interactions
            if !batch_interactions.is_empty() {
                match self
                    .vector_store
                    .bulk_import_interactions(ctx, batch_interactions.clone())
                    .await
                {
                    Ok(imported) => {
                        success_count += imported;
                        debug!(
                            "InteractionService: Batch {} imported {} interactions",
                            batch_idx + 1,
                            imported
                        );
                    }
                    Err(e) => {
                        warn!(
                            "InteractionService: Batch {} import failed: {}",
                            batch_idx + 1,
                            e
                        );
                        // Add errors for all interactions in failed batch
                        for (user_id, entity_id, _, _, _, _, _) in batch_interactions {
                            errors.push(BulkImportError {
                                user_id,
                                entity_id,
                                error: format!("Batch import failed: {}", e),
                            });
                        }
                    }
                }
            }
        }

        let result = BulkImportResult {
            job_id: format!("import_{}", Utc::now().timestamp()),
            status: if errors.is_empty() {
                ImportStatus::Completed
            } else if success_count > 0 {
                ImportStatus::PartiallyCompleted
            } else {
                ImportStatus::Failed
            },
            total_records: total_count,
            processed: processed_count,
            successful: success_count,
            failed: errors.len(),
            errors,
        };

        info!(
            "InteractionService: Bulk import completed - tenant={}, total={}, successful={}, failed={}",
            ctx.tenant_id, total_count, success_count, result.failed
        );

        Ok(result)
    }

    /// Get interaction weight for a given interaction type
    /// This method uses the in-memory weights map which can be loaded from tenant config
    /// For database-backed weights, use get_interaction_weight_async
    fn get_interaction_weight(&self, interaction_type: &InteractionType) -> f32 {
        match interaction_type {
            InteractionType::View => *self.interaction_weights.get("view").unwrap_or(&1.0),
            InteractionType::AddToCart => {
                *self.interaction_weights.get("add_to_cart").unwrap_or(&3.0)
            }
            InteractionType::Purchase => *self.interaction_weights.get("purchase").unwrap_or(&5.0),
            InteractionType::Like => *self.interaction_weights.get("like").unwrap_or(&2.0),
            InteractionType::Rating(rating) => *rating,
            InteractionType::Custom(custom_type) => {
                *self.interaction_weights.get(custom_type).unwrap_or(&1.0)
            }
        }
    }

    /// Get interaction weight for a given interaction type from database registry
    /// Falls back to in-memory weights if not found in database
    pub async fn get_interaction_weight_async(
        &self,
        ctx: &TenantContext,
        interaction_type: &InteractionType,
    ) -> Result<f32> {
        // Try to get weight from database registry first
        let weight = self
            .vector_store
            .get_interaction_weight(ctx, interaction_type)
            .await?;

        Ok(weight)
    }

    /// Validate user_id format
    fn validate_user_id(&self, user_id: &str) -> Result<()> {
        if user_id.is_empty() {
            return Err(RecommendationError::ValidationError(
                "user_id cannot be empty".to_string(),
            ));
        }

        if user_id.len() > 255 {
            return Err(RecommendationError::ValidationError(
                "user_id cannot exceed 255 characters".to_string(),
            ));
        }

        if user_id.contains('\0') {
            return Err(RecommendationError::ValidationError(
                "user_id cannot contain null characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate entity_id format
    fn validate_entity_id(&self, entity_id: &str) -> Result<()> {
        if entity_id.is_empty() {
            return Err(RecommendationError::ValidationError(
                "entity_id cannot be empty".to_string(),
            ));
        }

        if entity_id.len() > 255 {
            return Err(RecommendationError::ValidationError(
                "entity_id cannot exceed 255 characters".to_string(),
            ));
        }

        if entity_id.contains('\0') {
            return Err(RecommendationError::ValidationError(
                "entity_id cannot contain null characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Export interactions with optional date range filtering
    pub async fn export_interactions(
        &self,
        ctx: &TenantContext,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<Interaction>> {
        debug!(
            "InteractionService: Exporting interactions - tenant={}, start_date={:?}, end_date={:?}",
            ctx.tenant_id, start_date, end_date
        );

        // Validate date range if both are provided
        if let (Some(start), Some(end)) = (start_date, end_date)
            && start > end
        {
            return Err(RecommendationError::ValidationError(
                "start_date must be before end_date".to_string(),
            ));
        }

        let interactions = self
            .vector_store
            .export_interactions(ctx, start_date, end_date)
            .await?;

        info!(
            "InteractionService: Exported {} interactions - tenant={}",
            interactions.len(),
            ctx.tenant_id
        );

        Ok(interactions)
    }

    /// Export user profiles with optional vector inclusion
    pub async fn export_user_profiles(
        &self,
        ctx: &TenantContext,
        include_vectors: bool,
    ) -> Result<Vec<recommendation_models::UserProfile>> {
        debug!(
            "InteractionService: Exporting user profiles - tenant={}, include_vectors={}",
            ctx.tenant_id, include_vectors
        );

        let user_profiles = self
            .vector_store
            .export_user_profiles(ctx, include_vectors)
            .await?;

        info!(
            "InteractionService: Exported {} user profiles - tenant={}",
            user_profiles.len(),
            ctx.tenant_id
        );

        Ok(user_profiles)
    }
}

/// Helper function to update user profile asynchronously
async fn update_user_profile_async(
    vector_store: &Arc<VectorStore>,
    ctx: &TenantContext,
    user_id: &str,
) -> Result<()> {
    debug!(
        "Updating user profile asynchronously: tenant={}, user={}",
        ctx.tenant_id, user_id
    );

    // Compute preference vector from interactions
    let preference_vector = vector_store
        .compute_user_preference_vector(ctx, user_id)
        .await?;

    if preference_vector.is_empty() {
        debug!(
            "No preference vector computed for user: tenant={}, user={}",
            ctx.tenant_id, user_id
        );
        return Ok(());
    }

    // Get interaction count
    let interaction_count = vector_store
        .get_user_interaction_count(ctx, user_id)
        .await?;

    // Get last interaction timestamp
    let interactions = vector_store
        .get_user_interactions(ctx, user_id, 1, 0)
        .await?;
    let last_interaction_at = interactions.first().map(|i| i.timestamp);

    // Upsert user profile
    vector_store
        .upsert_user_profile(
            ctx,
            user_id,
            preference_vector,
            interaction_count,
            last_interaction_at,
        )
        .await?;

    info!(
        "Updated user profile: tenant={}, user={}, interaction_count={}",
        ctx.tenant_id, user_id, interaction_count
    );

    Ok(())
}

/// Helper function to check if interaction types match
fn matches_interaction_type(actual: &InteractionType, filter: &InteractionType) -> bool {
    match (actual, filter) {
        (InteractionType::View, InteractionType::View) => true,
        (InteractionType::AddToCart, InteractionType::AddToCart) => true,
        (InteractionType::Purchase, InteractionType::Purchase) => true,
        (InteractionType::Like, InteractionType::Like) => true,
        (InteractionType::Rating(_), InteractionType::Rating(_)) => true,
        (InteractionType::Custom(a), InteractionType::Custom(b)) => a == b,
        _ => false,
    }
}

/// Result of bulk import operation
#[derive(Debug, Clone)]
pub struct BulkImportResult {
    pub job_id: String,
    pub status: ImportStatus,
    pub total_records: usize,
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<BulkImportError>,
}

/// Status of bulk import job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportStatus {
    Completed,
    PartiallyCompleted,
    Failed,
}

/// Error information for failed interaction import
#[derive(Debug, Clone)]
pub struct BulkImportError {
    pub user_id: String,
    pub entity_id: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test service
    // We use a dummy connection string since validation methods don't actually use the database
    fn create_test_service() -> InteractionService {
        let pool = sqlx::PgPool::connect_lazy("postgresql://localhost/test_db").unwrap();
        let vector_store = Arc::new(VectorStore::new(pool));
        InteractionService::new(vector_store)
    }

    // Helper to create a test service with custom weights
    fn create_test_service_with_weights(weights: HashMap<String, f32>) -> InteractionService {
        let pool = sqlx::PgPool::connect_lazy("postgresql://localhost/test_db").unwrap();
        let vector_store = Arc::new(VectorStore::new(pool));
        InteractionService::with_weights(vector_store, weights)
    }

    #[tokio::test]
    async fn test_interaction_service_creation() {
        let service = create_test_service();

        // Check default weights
        assert_eq!(service.get_interaction_weight(&InteractionType::View), 1.0);
        assert_eq!(
            service.get_interaction_weight(&InteractionType::AddToCart),
            3.0
        );
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Purchase),
            5.0
        );
        assert_eq!(service.get_interaction_weight(&InteractionType::Like), 2.0);
    }

    #[tokio::test]
    async fn test_interaction_service_with_custom_weights() {
        let mut custom_weights = HashMap::new();
        custom_weights.insert("view".to_string(), 2.0);
        custom_weights.insert("like".to_string(), 5.0);
        custom_weights.insert("share".to_string(), 10.0);

        let service = create_test_service_with_weights(custom_weights);

        // Check custom weights
        assert_eq!(service.get_interaction_weight(&InteractionType::View), 2.0);
        assert_eq!(service.get_interaction_weight(&InteractionType::Like), 5.0);
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Custom("share".to_string())),
            10.0
        );

        // Default weight for unknown custom type
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Custom("unknown".to_string())),
            1.0
        );
    }

    #[tokio::test]
    async fn test_get_interaction_weight_rating() {
        let service = create_test_service();

        // Rating interactions use the rating value as weight
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Rating(4.5)),
            4.5
        );
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Rating(3.0)),
            3.0
        );
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Rating(5.0)),
            5.0
        );
    }

    #[tokio::test]
    async fn test_validate_user_id() {
        let service = create_test_service();

        // Valid user_id
        assert!(service.validate_user_id("user_123").is_ok());
        assert!(service.validate_user_id("user-456").is_ok());
        assert!(service.validate_user_id("u").is_ok());

        // Invalid user_id - empty
        assert!(service.validate_user_id("").is_err());

        // Invalid user_id - too long
        assert!(service.validate_user_id(&"a".repeat(256)).is_err());

        // Invalid user_id - null character
        assert!(service.validate_user_id("user\0null").is_err());
    }

    #[tokio::test]
    async fn test_validate_entity_id() {
        let service = create_test_service();

        // Valid entity_id
        assert!(service.validate_entity_id("product_123").is_ok());
        assert!(service.validate_entity_id("entity-456").is_ok());
        assert!(service.validate_entity_id("e").is_ok());

        // Invalid entity_id - empty
        assert!(service.validate_entity_id("").is_err());

        // Invalid entity_id - too long
        assert!(service.validate_entity_id(&"a".repeat(256)).is_err());

        // Invalid entity_id - null character
        assert!(service.validate_entity_id("entity\0null").is_err());
    }

    #[test]
    fn test_matches_interaction_type() {
        // Exact matches
        assert!(matches_interaction_type(
            &InteractionType::View,
            &InteractionType::View
        ));
        assert!(matches_interaction_type(
            &InteractionType::AddToCart,
            &InteractionType::AddToCart
        ));
        assert!(matches_interaction_type(
            &InteractionType::Purchase,
            &InteractionType::Purchase
        ));
        assert!(matches_interaction_type(
            &InteractionType::Like,
            &InteractionType::Like
        ));

        // Rating matches any rating
        assert!(matches_interaction_type(
            &InteractionType::Rating(4.5),
            &InteractionType::Rating(0.0)
        ));
        assert!(matches_interaction_type(
            &InteractionType::Rating(3.0),
            &InteractionType::Rating(5.0)
        ));

        // Custom type matches
        assert!(matches_interaction_type(
            &InteractionType::Custom("share".to_string()),
            &InteractionType::Custom("share".to_string())
        ));

        // Non-matches
        assert!(!matches_interaction_type(
            &InteractionType::View,
            &InteractionType::Like
        ));
        assert!(!matches_interaction_type(
            &InteractionType::Purchase,
            &InteractionType::AddToCart
        ));
        assert!(!matches_interaction_type(
            &InteractionType::Custom("share".to_string()),
            &InteractionType::Custom("comment".to_string())
        ));
    }

    #[test]
    fn test_bulk_import_result_status() {
        // Test completed status
        let completed = BulkImportResult {
            job_id: "job_1".to_string(),
            status: ImportStatus::Completed,
            total_records: 100,
            processed: 100,
            successful: 100,
            failed: 0,
            errors: vec![],
        };
        assert_eq!(completed.status, ImportStatus::Completed);
        assert_eq!(completed.errors.len(), 0);

        // Test partially completed status
        let partial = BulkImportResult {
            job_id: "job_2".to_string(),
            status: ImportStatus::PartiallyCompleted,
            total_records: 100,
            processed: 100,
            successful: 80,
            failed: 20,
            errors: vec![BulkImportError {
                user_id: "user_1".to_string(),
                entity_id: "entity_1".to_string(),
                error: "Validation failed".to_string(),
            }],
        };
        assert_eq!(partial.status, ImportStatus::PartiallyCompleted);
        assert_eq!(partial.errors.len(), 1);

        // Test failed status
        let failed = BulkImportResult {
            job_id: "job_3".to_string(),
            status: ImportStatus::Failed,
            total_records: 100,
            processed: 100,
            successful: 0,
            failed: 100,
            errors: vec![],
        };
        assert_eq!(failed.status, ImportStatus::Failed);
    }

    #[test]
    fn test_bulk_import_error_structure() {
        let error = BulkImportError {
            user_id: "user_123".to_string(),
            entity_id: "entity_456".to_string(),
            error: "Invalid user_id format".to_string(),
        };

        assert_eq!(error.user_id, "user_123");
        assert_eq!(error.entity_id, "entity_456");
        assert!(error.error.contains("Invalid"));
    }

    #[tokio::test]
    async fn test_interaction_type_weight_defaults() {
        let service = create_test_service();

        // Test all default interaction type weights
        assert_eq!(service.get_interaction_weight(&InteractionType::View), 1.0);
        assert_eq!(
            service.get_interaction_weight(&InteractionType::AddToCart),
            3.0
        );
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Purchase),
            5.0
        );
        assert_eq!(service.get_interaction_weight(&InteractionType::Like), 2.0);

        // Custom type without configured weight should default to 1.0
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Custom("unknown".to_string())),
            1.0
        );
    }

    #[tokio::test]
    async fn test_validation_error_messages() {
        let service = create_test_service();

        // Test empty user_id error message
        let result = service.validate_user_id("");
        assert!(result.is_err());
        if let Err(RecommendationError::ValidationError(msg)) = result {
            assert!(msg.contains("empty"));
        }

        // Test too long user_id error message
        let result = service.validate_user_id(&"a".repeat(256));
        assert!(result.is_err());
        if let Err(RecommendationError::ValidationError(msg)) = result {
            assert!(msg.contains("255"));
        }

        // Test null character error message
        let result = service.validate_user_id("user\0null");
        assert!(result.is_err());
        if let Err(RecommendationError::ValidationError(msg)) = result {
            assert!(msg.contains("null"));
        }
    }

    #[tokio::test]
    async fn test_custom_interaction_weights_override() {
        let mut custom_weights = HashMap::new();
        custom_weights.insert("view".to_string(), 10.0);
        custom_weights.insert("purchase".to_string(), 100.0);

        let service = create_test_service_with_weights(custom_weights);

        // Custom weights should override defaults
        assert_eq!(service.get_interaction_weight(&InteractionType::View), 10.0);
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Purchase),
            100.0
        );

        // Non-overridden types should use defaults
        assert_eq!(
            service.get_interaction_weight(&InteractionType::AddToCart),
            3.0
        );
        assert_eq!(service.get_interaction_weight(&InteractionType::Like), 2.0);
    }

    #[test]
    fn test_import_status_equality() {
        assert_eq!(ImportStatus::Completed, ImportStatus::Completed);
        assert_eq!(
            ImportStatus::PartiallyCompleted,
            ImportStatus::PartiallyCompleted
        );
        assert_eq!(ImportStatus::Failed, ImportStatus::Failed);

        assert_ne!(ImportStatus::Completed, ImportStatus::Failed);
        assert_ne!(ImportStatus::PartiallyCompleted, ImportStatus::Completed);
    }

    #[tokio::test]
    async fn test_unknown_interaction_type_defaults_to_one() {
        let service = create_test_service();

        // Unknown custom interaction types should default to weight 1.0
        let unknown_type = InteractionType::Custom("completely_unknown_type".to_string());
        assert_eq!(service.get_interaction_weight(&unknown_type), 1.0);

        // Another unknown type
        let another_unknown = InteractionType::Custom("never_seen_before".to_string());
        assert_eq!(service.get_interaction_weight(&another_unknown), 1.0);
    }

    #[tokio::test]
    async fn test_custom_interaction_type_with_configured_weight() {
        let mut custom_weights = HashMap::new();
        custom_weights.insert("share".to_string(), 4.0);
        custom_weights.insert("bookmark".to_string(), 2.5);

        let service = create_test_service_with_weights(custom_weights);

        // Configured custom types should use their configured weight
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Custom("share".to_string())),
            4.0
        );
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Custom("bookmark".to_string())),
            2.5
        );

        // Unknown custom type should still default to 1.0
        assert_eq!(
            service.get_interaction_weight(&InteractionType::Custom("unknown".to_string())),
            1.0
        );
    }
}
