use chrono::Utc;
use recommendation_models::{
    AttributeValue, DefaultFeatureExtractor, Entity, FeatureExtractor, RecommendationError, Result,
    TenantContext,
};
use recommendation_storage::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Service for entity lifecycle management
/// Handles entity CRUD operations, feature vector computation, and validation
pub struct EntityService {
    vector_store: Arc<VectorStore>,
    feature_extractor: Arc<dyn FeatureExtractor>,
}

impl EntityService {
    /// Create a new EntityService with the given vector store
    pub fn new(vector_store: Arc<VectorStore>) -> Self {
        let feature_dimension = 512; // Default dimension
        let feature_extractor = Arc::new(DefaultFeatureExtractor::new(feature_dimension));

        Self {
            vector_store,
            feature_extractor,
        }
    }

    /// Create a new EntityService with custom feature extractor
    pub fn with_feature_extractor(
        vector_store: Arc<VectorStore>,
        feature_extractor: Arc<dyn FeatureExtractor>,
    ) -> Self {
        Self {
            vector_store,
            feature_extractor,
        }
    }

    /// Create a new entity with feature vector computation
    pub async fn create_entity(
        &self,
        ctx: &TenantContext,
        entity_id: String,
        entity_type: String,
        attributes: HashMap<String, AttributeValue>,
    ) -> Result<Entity> {
        debug!(
            "EntityService: Creating entity - tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        // Validate entity_id and entity_type
        self.validate_entity_id(&entity_id)?;
        self.validate_entity_type(&entity_type)?;

        // Validate attributes structure
        self.validate_attributes(&attributes)?;

        // Check for uniqueness within entity_type
        if let Some(_existing) = self
            .vector_store
            .get_entity(ctx, &entity_id, &entity_type)
            .await?
        {
            return Err(RecommendationError::InvalidRequest(format!(
                "Entity with id '{}' and type '{}' already exists for tenant '{}'",
                entity_id, entity_type, ctx.tenant_id
            )));
        }

        // Compute feature vector from attributes
        let feature_vector = self.compute_feature_vector(&attributes)?;

        // Create entity in storage
        let entity = self
            .vector_store
            .create_entity(
                ctx,
                &entity_id,
                &entity_type,
                attributes,
                Some(feature_vector),
            )
            .await?;

        info!(
            "EntityService: Created entity - tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        Ok(entity)
    }

    /// Update an entity with vector recalculation
    pub async fn update_entity(
        &self,
        ctx: &TenantContext,
        entity_id: String,
        entity_type: String,
        attributes: HashMap<String, AttributeValue>,
    ) -> Result<Entity> {
        debug!(
            "EntityService: Updating entity - tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        // Validate attributes structure
        self.validate_attributes(&attributes)?;

        // Check if entity exists
        if self
            .vector_store
            .get_entity(ctx, &entity_id, &entity_type)
            .await?
            .is_none()
        {
            return Err(RecommendationError::EntityNotFound(format!(
                "Entity with id '{}' and type '{}' not found for tenant '{}'",
                entity_id, entity_type, ctx.tenant_id
            )));
        }

        // Recompute feature vector from updated attributes
        let feature_vector = self.compute_feature_vector(&attributes)?;

        // Update entity in storage
        let entity = self
            .vector_store
            .update_entity(
                ctx,
                &entity_id,
                &entity_type,
                attributes,
                Some(feature_vector),
            )
            .await?;

        info!(
            "EntityService: Updated entity - tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        Ok(entity)
    }

    /// Delete an entity with cleanup
    pub async fn delete_entity(
        &self,
        ctx: &TenantContext,
        entity_id: String,
        entity_type: String,
    ) -> Result<()> {
        debug!(
            "EntityService: Deleting entity - tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        // Delete entity from storage (this will also handle cleanup)
        self.vector_store
            .delete_entity(ctx, &entity_id, &entity_type)
            .await?;

        info!(
            "EntityService: Deleted entity - tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        Ok(())
    }

    /// Get an entity by ID
    pub async fn get_entity(
        &self,
        ctx: &TenantContext,
        entity_id: String,
        entity_type: String,
    ) -> Result<Option<Entity>> {
        debug!(
            "EntityService: Getting entity - tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        self.vector_store
            .get_entity(ctx, &entity_id, &entity_type)
            .await
    }

    /// Compute feature vector from attributes
    fn compute_feature_vector(
        &self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> Result<Vec<f32>> {
        self.feature_extractor
            .extract_features(attributes)
            .map_err(|e| {
                warn!("Failed to compute feature vector: {}", e);
                e
            })
    }

    /// Validate entity_id format and uniqueness requirements
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

        // Check for invalid characters
        if entity_id.contains('\0') {
            return Err(RecommendationError::ValidationError(
                "entity_id cannot contain null characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate entity_type format
    fn validate_entity_type(&self, entity_type: &str) -> Result<()> {
        if entity_type.is_empty() {
            return Err(RecommendationError::ValidationError(
                "entity_type cannot be empty".to_string(),
            ));
        }

        if entity_type.len() > 100 {
            return Err(RecommendationError::ValidationError(
                "entity_type cannot exceed 100 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate attributes structure (up to 3 levels of nesting)
    fn validate_attributes(&self, attributes: &HashMap<String, AttributeValue>) -> Result<()> {
        if attributes.is_empty() {
            return Err(RecommendationError::ValidationError(
                "attributes cannot be empty".to_string(),
            ));
        }

        // Check nesting depth
        for (key, value) in attributes {
            self.validate_attribute_key(key)?;
            self.validate_attribute_value(value, 0)?;
        }

        Ok(())
    }

    /// Validate attribute key
    fn validate_attribute_key(&self, key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(RecommendationError::ValidationError(
                "attribute key cannot be empty".to_string(),
            ));
        }

        if key.len() > 255 {
            return Err(RecommendationError::ValidationError(
                "attribute key cannot exceed 255 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate attribute value and check nesting depth
    fn validate_attribute_value(&self, value: &AttributeValue, depth: usize) -> Result<()> {
        const MAX_DEPTH: usize = 3;

        if depth >= MAX_DEPTH {
            return Err(RecommendationError::ValidationError(format!(
                "attribute nesting cannot exceed {} levels",
                MAX_DEPTH
            )));
        }

        match value {
            AttributeValue::String(s) => {
                if s.len() > 10_000 {
                    return Err(RecommendationError::ValidationError(
                        "string attribute cannot exceed 10,000 characters".to_string(),
                    ));
                }
            }
            AttributeValue::Array(arr) => {
                if arr.len() > 1000 {
                    return Err(RecommendationError::ValidationError(
                        "array attribute cannot exceed 1,000 elements".to_string(),
                    ));
                }
                for item in arr {
                    if item.len() > 1000 {
                        return Err(RecommendationError::ValidationError(
                            "array item cannot exceed 1,000 characters".to_string(),
                        ));
                    }
                }
            }
            AttributeValue::Number(n) => {
                if !n.is_finite() {
                    return Err(RecommendationError::ValidationError(
                        "number attribute must be finite".to_string(),
                    ));
                }
            }
            AttributeValue::Boolean(_) => {
                // Boolean values are always valid
            }
        }

        Ok(())
    }

    /// Bulk import entities in batches
    pub async fn bulk_import_entities(
        &self,
        ctx: &TenantContext,
        entities: Vec<(String, String, HashMap<String, AttributeValue>)>,
    ) -> Result<BulkImportResult> {
        info!(
            "EntityService: Starting bulk import - tenant={}, count={}",
            ctx.tenant_id,
            entities.len()
        );

        let total_count = entities.len();
        let mut processed_count = 0;
        let mut success_count = 0;
        let mut errors = Vec::new();

        // Process in batches of 1000
        const BATCH_SIZE: usize = 1000;

        for (batch_idx, batch) in entities.chunks(BATCH_SIZE).enumerate() {
            debug!(
                "EntityService: Processing batch {} of {} (size: {})",
                batch_idx + 1,
                total_count.div_ceil(BATCH_SIZE),
                batch.len()
            );

            let mut batch_entities = Vec::new();

            for (entity_id, entity_type, attributes) in batch {
                processed_count += 1;

                // Validate entity
                if let Err(e) = self.validate_entity_id(entity_id) {
                    errors.push(BulkImportError {
                        entity_id: entity_id.clone(),
                        entity_type: entity_type.clone(),
                        error: e.to_string(),
                    });
                    continue;
                }

                if let Err(e) = self.validate_entity_type(entity_type) {
                    errors.push(BulkImportError {
                        entity_id: entity_id.clone(),
                        entity_type: entity_type.clone(),
                        error: e.to_string(),
                    });
                    continue;
                }

                if let Err(e) = self.validate_attributes(attributes) {
                    errors.push(BulkImportError {
                        entity_id: entity_id.clone(),
                        entity_type: entity_type.clone(),
                        error: e.to_string(),
                    });
                    continue;
                }

                // Compute feature vector
                match self.compute_feature_vector(attributes) {
                    Ok(feature_vector) => {
                        batch_entities.push((
                            entity_id.clone(),
                            entity_type.clone(),
                            attributes.clone(),
                            Some(feature_vector),
                        ));
                    }
                    Err(e) => {
                        errors.push(BulkImportError {
                            entity_id: entity_id.clone(),
                            entity_type: entity_type.clone(),
                            error: format!("Feature extraction failed: {}", e),
                        });
                    }
                }
            }

            // Batch insert valid entities
            if !batch_entities.is_empty() {
                match self
                    .vector_store
                    .batch_insert_entities(ctx, batch_entities.clone())
                    .await
                {
                    Ok(inserted) => {
                        success_count += inserted;
                        debug!(
                            "EntityService: Batch {} inserted {} entities",
                            batch_idx + 1,
                            inserted
                        );
                    }
                    Err(e) => {
                        warn!(
                            "EntityService: Batch {} insertion failed: {}",
                            batch_idx + 1,
                            e
                        );
                        // Add errors for all entities in failed batch
                        for (entity_id, entity_type, _, _) in batch_entities {
                            errors.push(BulkImportError {
                                entity_id,
                                entity_type,
                                error: format!("Batch insertion failed: {}", e),
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
            "EntityService: Bulk import completed - tenant={}, total={}, successful={}, failed={}",
            ctx.tenant_id, total_count, success_count, result.failed
        );

        Ok(result)
    }

    /// Export entities with optional filtering
    /// Supports incremental exports using last_modified_after timestamp
    pub async fn export_entities(
        &self,
        ctx: &TenantContext,
        entity_type: Option<&str>,
        last_modified_after: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Entity>> {
        debug!(
            "EntityService: Exporting entities - tenant={}, entity_type={:?}, last_modified_after={:?}",
            ctx.tenant_id, entity_type, last_modified_after
        );

        let entities = self
            .vector_store
            .export_entities(ctx, entity_type, last_modified_after)
            .await?;

        info!(
            "EntityService: Exported {} entities - tenant={}",
            entities.len(),
            ctx.tenant_id
        );

        Ok(entities)
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

/// Error information for failed entity import
#[derive(Debug, Clone)]
pub struct BulkImportError {
    pub entity_id: String,
    pub entity_type: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a mock service for validation tests
    // We use a dummy connection string since validation methods don't actually use the database
    fn create_test_service() -> EntityService {
        // Create a lazy pool that won't actually connect
        let pool = sqlx::PgPool::connect_lazy("postgresql://localhost/test_db").unwrap();
        let vector_store = Arc::new(VectorStore::new(pool));
        EntityService::new(vector_store)
    }

    #[tokio::test]
    async fn test_validate_entity_id() {
        let service = create_test_service();

        // Valid entity_id
        assert!(service.validate_entity_id("product_123").is_ok());
        assert!(service.validate_entity_id("user-456").is_ok());

        // Invalid entity_id
        assert!(service.validate_entity_id("").is_err());
        assert!(service.validate_entity_id(&"a".repeat(256)).is_err());
        assert!(service.validate_entity_id("test\0null").is_err());
    }

    #[tokio::test]
    async fn test_validate_entity_type() {
        let service = create_test_service();

        // Valid entity_type
        assert!(service.validate_entity_type("product").is_ok());
        assert!(service.validate_entity_type("user").is_ok());

        // Invalid entity_type
        assert!(service.validate_entity_type("").is_err());
        assert!(service.validate_entity_type(&"a".repeat(101)).is_err());
    }

    #[tokio::test]
    async fn test_validate_attributes() {
        let service = create_test_service();

        // Valid attributes
        let mut valid_attrs = HashMap::new();
        valid_attrs.insert(
            "name".to_string(),
            AttributeValue::String("Product".to_string()),
        );
        valid_attrs.insert("price".to_string(), AttributeValue::Number(99.99));
        assert!(service.validate_attributes(&valid_attrs).is_ok());

        // Empty attributes
        let empty_attrs = HashMap::new();
        assert!(service.validate_attributes(&empty_attrs).is_err());

        // Invalid string length
        let mut invalid_attrs = HashMap::new();
        invalid_attrs.insert(
            "description".to_string(),
            AttributeValue::String("a".repeat(10_001)),
        );
        assert!(service.validate_attributes(&invalid_attrs).is_err());

        // Invalid array size
        let mut invalid_attrs = HashMap::new();
        invalid_attrs.insert(
            "tags".to_string(),
            AttributeValue::Array(vec!["tag".to_string(); 1001]),
        );
        assert!(service.validate_attributes(&invalid_attrs).is_err());

        // Invalid number
        let mut invalid_attrs = HashMap::new();
        invalid_attrs.insert("price".to_string(), AttributeValue::Number(f64::NAN));
        assert!(service.validate_attributes(&invalid_attrs).is_err());
    }
}
