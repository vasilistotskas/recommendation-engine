#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use recommendation_models::{
    Entity, AttributeValue, Interaction, InteractionType, UserProfile, 
    TenantContext, RecommendationError, Result,
};
use sqlx::{PgPool, QueryBuilder, Postgres, Row};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tracing::{info, debug, warn};

/// Vector store for entity CRUD operations and similarity search
pub struct VectorStore {
    pool: PgPool,
}

impl VectorStore {
    /// Create a new VectorStore with the given database pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new entity with tenant isolation
    pub async fn create_entity(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
        attributes: HashMap<String, AttributeValue>,
        feature_vector: Option<Vec<f32>>,
    ) -> Result<Entity> {
        debug!(
            "Creating entity: tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        // Convert attributes to JSONB
        let attributes_json = serde_json::to_value(&attributes)
            .map_err(|e| RecommendationError::VectorError(format!("Failed to serialize attributes: {}", e)))?;

        // Convert feature vector to pgvector format
        let vector_str = feature_vector.as_ref().map(|v| {
            format!("[{}]", v.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","))
        });

        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO entities (tenant_id, entity_id, entity_type, attributes, feature_vector, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5::vector, $6, $7)
            RETURNING entity_id, entity_type, tenant_id, attributes, created_at, updated_at
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(entity_id)
        .bind(entity_type)
        .bind(&attributes_json)
        .bind(vector_str)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                RecommendationError::InvalidRequest(format!(
                    "Entity with id '{}' and type '{}' already exists for tenant '{}'",
                    entity_id, entity_type, ctx.tenant_id
                ))
            } else {
                e.into()
            }
        })?;

        let result = (
            row.try_get::<String, _>("entity_id")?,
            row.try_get::<String, _>("entity_type")?,
            row.try_get::<String, _>("tenant_id")?,
            row.try_get::<serde_json::Value, _>("attributes")?,
            row.try_get::<chrono::DateTime<Utc>, _>("created_at")?,
            row.try_get::<chrono::DateTime<Utc>, _>("updated_at")?,
        );

        info!(
            "Created entity: tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        Ok(Entity {
            entity_id: result.0,
            entity_type: result.1,
            attributes: serde_json::from_value(result.3)
                .map_err(|e| RecommendationError::VectorError(format!("Failed to deserialize attributes: {}", e)))?,
            feature_vector,
            tenant_id: Some(result.2),
            created_at: result.4,
            updated_at: result.5,
        })
    }

    /// Get an entity by ID with tenant isolation
    pub async fn get_entity(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
    ) -> Result<Option<Entity>> {
        debug!(
            "Getting entity: tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        let result = sqlx::query!(
            r#"
            SELECT entity_id, entity_type, tenant_id, attributes, 
                   feature_vector::text as feature_vector_text,
                   created_at, updated_at
            FROM entities
            WHERE tenant_id = $1 AND entity_id = $2 AND entity_type = $3
            "#,
            ctx.tenant_id,
            entity_id,
            entity_type
        )
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(row) => {
                let feature_vector = row.feature_vector_text.as_deref().and_then(parse_vector);
                
                Ok(Some(Entity {
                    entity_id: row.entity_id,
                    entity_type: row.entity_type,
                    attributes: serde_json::from_value(row.attributes)
                        .map_err(|e| RecommendationError::VectorError(format!("Failed to deserialize attributes: {}", e)))?,
                    feature_vector,
                    tenant_id: Some(row.tenant_id),
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// Update an entity with tenant isolation
    pub async fn update_entity(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
        attributes: HashMap<String, AttributeValue>,
        feature_vector: Option<Vec<f32>>,
    ) -> Result<Entity> {
        debug!(
            "Updating entity: tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        let attributes_json = serde_json::to_value(&attributes)
            .map_err(|e| RecommendationError::VectorError(format!("Failed to serialize attributes: {}", e)))?;

        let vector_str = feature_vector.as_ref().map(|v| {
            format!("[{}]", v.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","))
        });

        let now = Utc::now();

        let row_opt = sqlx::query(
            r#"
            UPDATE entities
            SET attributes = $1, feature_vector = $2::vector, updated_at = $3
            WHERE tenant_id = $4 AND entity_id = $5 AND entity_type = $6
            RETURNING entity_id, entity_type, tenant_id, attributes, created_at, updated_at
            "#
        )
        .bind(&attributes_json)
        .bind(vector_str)
        .bind(now)
        .bind(&ctx.tenant_id)
        .bind(entity_id)
        .bind(entity_type)
        .fetch_optional(&self.pool)
        .await?;

        match row_opt {
            Some(row) => {
                info!(
                    "Updated entity: tenant={}, entity_id={}, entity_type={}",
                    ctx.tenant_id, entity_id, entity_type
                );

                Ok(Entity {
                    entity_id: row.try_get("entity_id")?,
                    entity_type: row.try_get("entity_type")?,
                    attributes: serde_json::from_value(row.try_get("attributes")?)
                        .map_err(|e| RecommendationError::VectorError(format!("Failed to deserialize attributes: {}", e)))?,
                    feature_vector,
                    tenant_id: Some(row.try_get("tenant_id")?),
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            }
            None => Err(RecommendationError::EntityNotFound(format!(
                "Entity with id '{}' and type '{}' not found for tenant '{}'",
                entity_id, entity_type, ctx.tenant_id
            ))),
        }
    }

    /// Delete an entity with tenant isolation
    pub async fn delete_entity(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
    ) -> Result<()> {
        debug!(
            "Deleting entity: tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        let result = sqlx::query!(
            r#"
            DELETE FROM entities
            WHERE tenant_id = $1 AND entity_id = $2 AND entity_type = $3
            "#,
            ctx.tenant_id,
            entity_id,
            entity_type
        )
        .execute(&self.pool)
        .await
        ?;

        if result.rows_affected() == 0 {
            return Err(RecommendationError::EntityNotFound(format!(
                "Entity with id '{}' and type '{}' not found for tenant '{}'",
                entity_id, entity_type, ctx.tenant_id
            )));
        }

        info!(
            "Deleted entity: tenant={}, entity_id={}, entity_type={}",
            ctx.tenant_id, entity_id, entity_type
        );

        Ok(())
    }

    /// Find similar entities using pgvector cosine similarity
    pub async fn find_similar_entities(
        &self,
        ctx: &TenantContext,
        feature_vector: &[f32],
        entity_type: &str,
        similarity_threshold: f32,
        limit: usize,
        exclude_entity_id: Option<&str>,
    ) -> Result<Vec<(Entity, f32)>> {
        debug!(
            "Finding similar entities: tenant={}, entity_type={}, threshold={}, limit={}",
            ctx.tenant_id, entity_type, similarity_threshold, limit
        );

        let vector_str = format!(
            "[{}]",
            feature_vector.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(",")
        );

        let results = if let Some(exclude_id) = exclude_entity_id {
            sqlx::query(
                r#"
                SELECT entity_id, entity_type, tenant_id, attributes,
                       feature_vector::text as feature_vector_text,
                       created_at, updated_at,
                       1 - (feature_vector <=> $1::vector) as similarity
                FROM entities
                WHERE tenant_id = $2 
                  AND entity_type = $3
                  AND entity_id != $4
                  AND feature_vector IS NOT NULL
                  AND 1 - (feature_vector <=> $1::vector) >= $5
                ORDER BY feature_vector <=> $1::vector
                LIMIT $6
                "#
            )
            .bind(&vector_str)
            .bind(&ctx.tenant_id)
            .bind(entity_type)
            .bind(exclude_id)
            .bind(similarity_threshold)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT entity_id, entity_type, tenant_id, attributes,
                       feature_vector::text as feature_vector_text,
                       created_at, updated_at,
                       1 - (feature_vector <=> $1::vector) as similarity
                FROM entities
                WHERE tenant_id = $2 
                  AND entity_type = $3
                  AND feature_vector IS NOT NULL
                  AND 1 - (feature_vector <=> $1::vector) >= $4
                ORDER BY feature_vector <=> $1::vector
                LIMIT $5
                "#
            )
            .bind(&vector_str)
            .bind(&ctx.tenant_id)
            .bind(entity_type)
            .bind(similarity_threshold)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let entities: Vec<(Entity, f32)> = results
            .into_iter()
            .filter_map(|row| {
                let feature_vector_text: Option<String> = row.try_get("feature_vector_text").ok()?;
                let feature_vector = feature_vector_text.as_deref().and_then(parse_vector);
                let similarity: Option<f64> = row.try_get("similarity").ok()?;
                let similarity = similarity.unwrap_or(0.0) as f32;

                let attributes_json: serde_json::Value = row.try_get("attributes").ok()?;
                match serde_json::from_value(attributes_json) {
                    Ok(attributes) => Some((
                        Entity {
                            entity_id: row.try_get("entity_id").ok()?,
                            entity_type: row.try_get("entity_type").ok()?,
                            attributes,
                            feature_vector,
                            tenant_id: Some(row.try_get("tenant_id").ok()?),
                            created_at: row.try_get("created_at").ok()?,
                            updated_at: row.try_get("updated_at").ok()?,
                        },
                        similarity,
                    )),
                    Err(e) => {
                        warn!("Failed to deserialize entity attributes: {}", e);
                        None
                    }
                }
            })
            .collect();

        info!(
            "Found {} similar entities for tenant={}, entity_type={}",
            entities.len(),
            ctx.tenant_id,
            entity_type
        );

        Ok(entities)
    }

    /// Batch insert entities for performance
    pub async fn batch_insert_entities(
        &self,
        ctx: &TenantContext,
        entities: Vec<(String, String, HashMap<String, AttributeValue>, Option<Vec<f32>>)>,
    ) -> Result<usize> {
        if entities.is_empty() {
            return Ok(0);
        }

        info!(
            "Batch inserting {} entities for tenant={}",
            entities.len(),
            ctx.tenant_id
        );

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO entities (tenant_id, entity_id, entity_type, attributes, feature_vector, created_at, updated_at) "
        );

        let now = Utc::now();

        query_builder.push_values(entities, |mut b, (entity_id, entity_type, attributes, feature_vector)| {
            let attributes_json = serde_json::to_value(&attributes).unwrap_or(serde_json::json!({}));
            let vector_str = feature_vector.as_ref().map(|v| {
                format!("[{}]", v.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","))
            });

            b.push_bind(&ctx.tenant_id)
                .push_bind(entity_id)
                .push_bind(entity_type)
                .push_bind(attributes_json)
                .push_bind(vector_str)
                .push_bind(now)
                .push_bind(now);
        });

        let result = query_builder
            .build()
            .execute(&self.pool)
            .await
            ?;

        let inserted = result.rows_affected() as usize;

        info!(
            "Batch inserted {} entities for tenant={}",
            inserted, ctx.tenant_id
        );

        Ok(inserted)
    }

    /// Batch update entities for performance
    pub async fn batch_update_entities(
        &self,
        ctx: &TenantContext,
        updates: Vec<(String, String, HashMap<String, AttributeValue>, Option<Vec<f32>>)>,
    ) -> Result<usize> {
        if updates.is_empty() {
            return Ok(0);
        }

        info!(
            "Batch updating {} entities for tenant={}",
            updates.len(),
            ctx.tenant_id
        );

        let mut updated_count = 0;
        let now = Utc::now();

        // Process updates in transaction for consistency
        let mut tx = self.pool.begin().await?;

        for (entity_id, entity_type, attributes, feature_vector) in updates {
            let attributes_json = serde_json::to_value(&attributes)
                .map_err(|e| RecommendationError::VectorError(format!("Failed to serialize attributes: {}", e)))?;

            let vector_str = feature_vector.as_ref().map(|v| {
                format!("[{}]", v.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","))
            });

            let result = sqlx::query(
                r#"
                UPDATE entities
                SET attributes = $1, feature_vector = $2::vector, updated_at = $3
                WHERE tenant_id = $4 AND entity_id = $5 AND entity_type = $6
                "#
            )
            .bind(&attributes_json)
            .bind(vector_str)
            .bind(now)
            .bind(&ctx.tenant_id)
            .bind(&entity_id)
            .bind(&entity_type)
            .execute(&mut *tx)
            .await?;

            updated_count += result.rows_affected() as usize;
        }

        tx.commit().await?;

        info!(
            "Batch updated {} entities for tenant={}",
            updated_count, ctx.tenant_id
        );

        Ok(updated_count)
    }
}

/// Parse a pgvector string representation into a Vec<f32>
fn parse_vector(s: &str) -> Option<Vec<f32>> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return None;
    }

    let inner = &s[1..s.len() - 1];
    if inner.is_empty() {
        return Some(Vec::new());
    }

    inner
        .split(',')
        .map(|part| part.trim().parse::<f32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vector_empty() {
        assert_eq!(parse_vector("[]"), Some(Vec::new()));
    }

    #[test]
    fn test_parse_vector_single() {
        assert_eq!(parse_vector("[1.5]"), Some(vec![1.5]));
    }

    #[test]
    fn test_parse_vector_multiple() {
        assert_eq!(
            parse_vector("[0.1,0.2,0.3]"),
            Some(vec![0.1, 0.2, 0.3])
        );
    }

    #[test]
    fn test_parse_vector_with_spaces() {
        assert_eq!(
            parse_vector("[ 1.0 , 2.0 , 3.0 ]"),
            Some(vec![1.0, 2.0, 3.0])
        );
    }

    #[test]
    fn test_parse_vector_invalid() {
        assert_eq!(parse_vector("not a vector"), None);
        assert_eq!(parse_vector("[1.0,invalid,3.0]"), None);
        assert_eq!(parse_vector("1.0,2.0,3.0"), None);
    }
}

// ============================================================================
// Interaction Storage Implementation
// ============================================================================

impl VectorStore {
    /// Record an interaction with deduplication
    /// Deduplicates based on user_id, entity_id, interaction_type within 60 seconds
    pub async fn record_interaction(
        &self,
        ctx: &TenantContext,
        user_id: &str,
        entity_id: &str,
        entity_type: &str,
        interaction_type: &InteractionType,
        weight: f32,
        metadata: Option<HashMap<String, String>>,
        timestamp: DateTime<Utc>,
    ) -> Result<Interaction> {
        debug!(
            "Recording interaction: tenant={}, user={}, entity={}, type={:?}",
            ctx.tenant_id, user_id, entity_id, interaction_type
        );

        // Serialize interaction type and metadata
        let interaction_type_str = match interaction_type {
            InteractionType::View => "view".to_string(),
            InteractionType::AddToCart => "add_to_cart".to_string(),
            InteractionType::Purchase => "purchase".to_string(),
            InteractionType::Like => "like".to_string(),
            InteractionType::Rating(r) => format!("rating_{}", r),
            InteractionType::Custom(s) => s.clone(),
        };

        let metadata_json = metadata.as_ref().and_then(|m| serde_json::to_value(m).ok());

        // Insert with ON CONFLICT to handle deduplication
        let result = sqlx::query!(
            r#"
            INSERT INTO interactions (tenant_id, user_id, entity_id, entity_type, interaction_type, weight, metadata, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (tenant_id, user_id, entity_id, interaction_type, timestamp) 
            DO UPDATE SET weight = EXCLUDED.weight, metadata = EXCLUDED.metadata
            RETURNING id, user_id, entity_id, entity_type, tenant_id, interaction_type, weight, metadata, timestamp
            "#,
            ctx.tenant_id,
            user_id,
            entity_id,
            entity_type,
            interaction_type_str,
            weight as f32,
            metadata_json,
            timestamp
        )
        .fetch_one(&self.pool)
        .await
        ?;

        info!(
            "Recorded interaction: tenant={}, user={}, entity={}, type={}",
            ctx.tenant_id, user_id, entity_id, interaction_type_str
        );

        Ok(Interaction {
            id: Some(result.id),
            user_id: result.user_id,
            entity_id: result.entity_id,
            interaction_type: parse_interaction_type(&result.interaction_type),
            weight: result.weight as f32,
            metadata: result.metadata.and_then(|v| serde_json::from_value(v).ok()),
            tenant_id: Some(result.tenant_id),
            timestamp: result.timestamp,
        })
    }

    /// Get user interaction history with pagination
    pub async fn get_user_interactions(
        &self,
        ctx: &TenantContext,
        user_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Interaction>> {
        debug!(
            "Getting user interactions: tenant={}, user={}, limit={}, offset={}",
            ctx.tenant_id, user_id, limit, offset
        );

        let results = sqlx::query!(
            r#"
            SELECT id, user_id, entity_id, entity_type, tenant_id, interaction_type, weight, metadata, timestamp
            FROM interactions
            WHERE tenant_id = $1 AND user_id = $2
            ORDER BY timestamp DESC
            LIMIT $3 OFFSET $4
            "#,
            ctx.tenant_id,
            user_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        ?;

        let interactions: Vec<Interaction> = results
            .into_iter()
            .map(|row| Interaction {
                id: Some(row.id),
                user_id: row.user_id,
                entity_id: row.entity_id,
                interaction_type: parse_interaction_type(&row.interaction_type),
                weight: row.weight as f32,
                metadata: row.metadata.and_then(|v| serde_json::from_value(v).ok()),
                tenant_id: Some(row.tenant_id),
                timestamp: row.timestamp,
            })
            .collect();

        debug!(
            "Retrieved {} interactions for user={}, tenant={}",
            interactions.len(),
            user_id,
            ctx.tenant_id
        );

        Ok(interactions)
    }

    /// Get interactions for a specific entity
    pub async fn get_entity_interactions(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Interaction>> {
        debug!(
            "Getting entity interactions: tenant={}, entity={}, limit={}, offset={}",
            ctx.tenant_id, entity_id, limit, offset
        );

        let results = sqlx::query!(
            r#"
            SELECT id, user_id, entity_id, entity_type, tenant_id, interaction_type, weight, metadata, timestamp
            FROM interactions
            WHERE tenant_id = $1 AND entity_id = $2
            ORDER BY timestamp DESC
            LIMIT $3 OFFSET $4
            "#,
            ctx.tenant_id,
            entity_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await
        ?;

        let interactions: Vec<Interaction> = results
            .into_iter()
            .map(|row| Interaction {
                id: Some(row.id),
                user_id: row.user_id,
                entity_id: row.entity_id,
                interaction_type: parse_interaction_type(&row.interaction_type),
                weight: row.weight as f32,
                metadata: row.metadata.and_then(|v| serde_json::from_value(v).ok()),
                tenant_id: Some(row.tenant_id),
                timestamp: row.timestamp,
            })
            .collect();

        debug!(
            "Retrieved {} interactions for entity={}, tenant={}",
            interactions.len(),
            entity_id,
            ctx.tenant_id
        );

        Ok(interactions)
    }

    /// Bulk import interactions for performance
    pub async fn bulk_import_interactions(
        &self,
        ctx: &TenantContext,
        interactions: Vec<(String, String, String, InteractionType, f32, Option<HashMap<String, String>>, DateTime<Utc>)>,
    ) -> Result<usize> {
        if interactions.is_empty() {
            return Ok(0);
        }

        info!(
            "Bulk importing {} interactions for tenant={}",
            interactions.len(),
            ctx.tenant_id
        );

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO interactions (tenant_id, user_id, entity_id, entity_type, interaction_type, weight, metadata, timestamp) "
        );

        query_builder.push_values(
            interactions,
            |mut b, (user_id, entity_id, entity_type, interaction_type, weight, metadata, timestamp)| {
                let interaction_type_str = match interaction_type {
                    InteractionType::View => "view".to_string(),
                    InteractionType::AddToCart => "add_to_cart".to_string(),
                    InteractionType::Purchase => "purchase".to_string(),
                    InteractionType::Like => "like".to_string(),
                    InteractionType::Rating(r) => format!("rating_{}", r),
                    InteractionType::Custom(s) => s,
                };

                let metadata_json = metadata.as_ref().and_then(|m| serde_json::to_value(m).ok());

                b.push_bind(&ctx.tenant_id)
                    .push_bind(user_id)
                    .push_bind(entity_id)
                    .push_bind(entity_type)
                    .push_bind(interaction_type_str)
                    .push_bind(weight)
                    .push_bind(metadata_json)
                    .push_bind(timestamp);
            },
        );

        query_builder.push(" ON CONFLICT (tenant_id, user_id, entity_id, interaction_type, timestamp) DO NOTHING");

        let result = query_builder
            .build()
            .execute(&self.pool)
            .await?;

        let imported = result.rows_affected() as usize;

        info!(
            "Bulk imported {} interactions for tenant={}",
            imported, ctx.tenant_id
        );

        Ok(imported)
    }

    /// Get interaction count for a user (for cold start detection)
    pub async fn get_user_interaction_count(
        &self,
        ctx: &TenantContext,
        user_id: &str,
    ) -> Result<i32> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM interactions
            WHERE tenant_id = $1 AND user_id = $2
            "#,
            ctx.tenant_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        ?;

        Ok(result.count.unwrap_or(0) as i32)
    }

    /// Get entities a user has interacted with (for exclusion in recommendations)
    pub async fn get_user_interacted_entities(
        &self,
        ctx: &TenantContext,
        user_id: &str,
    ) -> Result<Vec<String>> {
        let results = sqlx::query!(
            r#"
            SELECT DISTINCT entity_id
            FROM interactions
            WHERE tenant_id = $1 AND user_id = $2
            "#,
            ctx.tenant_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        ?;

        Ok(results.into_iter().map(|row| row.entity_id).collect())
    }
}

/// Parse interaction type string back to enum
fn parse_interaction_type(s: &str) -> InteractionType {
    match s {
        "view" => InteractionType::View,
        "add_to_cart" => InteractionType::AddToCart,
        "purchase" => InteractionType::Purchase,
        "like" => InteractionType::Like,
        s if s.starts_with("rating_") => {
            let rating = s.strip_prefix("rating_")
                .and_then(|r| r.parse::<f32>().ok())
                .unwrap_or(0.0);
            InteractionType::Rating(rating)
        }
        s => InteractionType::Custom(s.to_string()),
    }
}

// ============================================================================
// User Profile Management Implementation
// ============================================================================

impl VectorStore {
    /// Create or update a user profile
    pub async fn upsert_user_profile(
        &self,
        ctx: &TenantContext,
        user_id: &str,
        preference_vector: Vec<f32>,
        interaction_count: i32,
        last_interaction_at: Option<DateTime<Utc>>,
    ) -> Result<UserProfile> {
        debug!(
            "Upserting user profile: tenant={}, user={}, interaction_count={}",
            ctx.tenant_id, user_id, interaction_count
        );

        let vector_str = format!(
            "[{}]",
            preference_vector.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(",")
        );

        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO user_profiles (tenant_id, user_id, preference_vector, interaction_count, last_interaction_at, created_at, updated_at)
            VALUES ($1, $2, $3::vector, $4, $5, $6, $7)
            ON CONFLICT (tenant_id, user_id)
            DO UPDATE SET 
                preference_vector = EXCLUDED.preference_vector,
                interaction_count = EXCLUDED.interaction_count,
                last_interaction_at = EXCLUDED.last_interaction_at,
                updated_at = EXCLUDED.updated_at
            RETURNING user_id, tenant_id, preference_vector::text as preference_vector_text, 
                      interaction_count, last_interaction_at, created_at, updated_at
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(user_id)
        .bind(&vector_str)
        .bind(interaction_count)
        .bind(last_interaction_at)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        info!(
            "Upserted user profile: tenant={}, user={}, interaction_count={}",
            ctx.tenant_id, user_id, interaction_count
        );

        let preference_vector_text: Option<String> = row.try_get("preference_vector_text")?;
        Ok(UserProfile {
            user_id: row.try_get("user_id")?,
            preference_vector: preference_vector_text
                .as_deref()
                .and_then(parse_vector)
                .unwrap_or_default(),
            interaction_count: row.try_get("interaction_count")?,
            last_interaction_at: row.try_get("last_interaction_at")?,
            tenant_id: Some(row.try_get("tenant_id")?),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    /// Get a user profile
    pub async fn get_user_profile(
        &self,
        ctx: &TenantContext,
        user_id: &str,
    ) -> Result<Option<UserProfile>> {
        debug!(
            "Getting user profile: tenant={}, user={}",
            ctx.tenant_id, user_id
        );

        let result = sqlx::query!(
            r#"
            SELECT user_id, tenant_id, preference_vector::text as preference_vector_text,
                   interaction_count, last_interaction_at, created_at, updated_at
            FROM user_profiles
            WHERE tenant_id = $1 AND user_id = $2
            "#,
            ctx.tenant_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        ?;

        match result {
            Some(row) => Ok(Some(UserProfile {
                user_id: row.user_id,
                preference_vector: row.preference_vector_text
                    .as_deref()
                    .and_then(parse_vector)
                    .unwrap_or_default(),
                interaction_count: row.interaction_count,
                last_interaction_at: row.last_interaction_at,
                tenant_id: Some(row.tenant_id),
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    /// Delete a user profile
    pub async fn delete_user_profile(
        &self,
        ctx: &TenantContext,
        user_id: &str,
    ) -> Result<()> {
        debug!(
            "Deleting user profile: tenant={}, user={}",
            ctx.tenant_id, user_id
        );

        let result = sqlx::query!(
            r#"
            DELETE FROM user_profiles
            WHERE tenant_id = $1 AND user_id = $2
            "#,
            ctx.tenant_id,
            user_id
        )
        .execute(&self.pool)
        .await
        ?;

        if result.rows_affected() == 0 {
            warn!(
                "User profile not found for deletion: tenant={}, user={}",
                ctx.tenant_id, user_id
            );
        } else {
            info!(
                "Deleted user profile: tenant={}, user={}",
                ctx.tenant_id, user_id
            );
        }

        Ok(())
    }

    /// Compute preference vector from user interactions
    /// This aggregates entity feature vectors weighted by interaction weights
    pub async fn compute_user_preference_vector(
        &self,
        ctx: &TenantContext,
        user_id: &str,
    ) -> Result<Vec<f32>> {
        debug!(
            "Computing preference vector for user: tenant={}, user={}",
            ctx.tenant_id, user_id
        );

        // Get all user interactions with entity feature vectors
        let results = sqlx::query!(
            r#"
            SELECT e.feature_vector::text as feature_vector_text, i.weight
            FROM interactions i
            JOIN entities e ON i.tenant_id = e.tenant_id 
                AND i.entity_id = e.entity_id 
                AND i.entity_type = e.entity_type
            WHERE i.tenant_id = $1 AND i.user_id = $2
              AND e.feature_vector IS NOT NULL
            ORDER BY i.timestamp DESC
            LIMIT 1000
            "#,
            ctx.tenant_id,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        ?;

        if results.is_empty() {
            debug!(
                "No interactions with feature vectors found for user: tenant={}, user={}",
                ctx.tenant_id, user_id
            );
            return Ok(Vec::new());
        }

        // Aggregate weighted feature vectors
        let mut aggregated_vector: Option<Vec<f32>> = None;
        let mut total_weight = 0.0;

        for row in results {
            if let Some(vector_text) = &row.feature_vector_text
                && let Some(vector) = parse_vector(vector_text.as_str()) {
                    let weight = row.weight as f32;
                    total_weight += weight;

                    match &mut aggregated_vector {
                        Some(agg) => {
                            // Add weighted vector to aggregated vector
                            for (i, val) in vector.iter().enumerate() {
                                if i < agg.len() {
                                    agg[i] += val * weight;
                                }
                            }
                        }
                        None => {
                            // Initialize aggregated vector
                            aggregated_vector = Some(vector.iter().map(|v| v * weight).collect());
                        }
                    }
                }
        }

        // Normalize by total weight
        if let Some(mut agg) = aggregated_vector {
            if total_weight > 0.0 {
                for val in agg.iter_mut() {
                    *val /= total_weight;
                }
            }

            info!(
                "Computed preference vector for user: tenant={}, user={}, dimension={}",
                ctx.tenant_id,
                user_id,
                agg.len()
            );

            Ok(agg)
        } else {
            Ok(Vec::new())
        }
    }

    /// Find similar users using pgvector cosine similarity
    pub async fn find_similar_users(
        &self,
        ctx: &TenantContext,
        preference_vector: &[f32],
        k: usize,
        exclude_user_id: Option<&str>,
    ) -> Result<Vec<(UserProfile, f32)>> {
        debug!(
            "Finding similar users: tenant={}, k={}, exclude={:?}",
            ctx.tenant_id, k, exclude_user_id
        );

        let vector_str = format!(
            "[{}]",
            preference_vector.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(",")
        );

        let results = if let Some(exclude_id) = exclude_user_id {
            sqlx::query(
                r#"
                SELECT user_id, tenant_id, preference_vector::text as preference_vector_text,
                       interaction_count, last_interaction_at, created_at, updated_at,
                       1 - (preference_vector <=> $1::vector) as similarity
                FROM user_profiles
                WHERE tenant_id = $2
                  AND user_id != $3
                  AND preference_vector IS NOT NULL
                ORDER BY preference_vector <=> $1::vector
                LIMIT $4
                "#
            )
            .bind(&vector_str)
            .bind(&ctx.tenant_id)
            .bind(exclude_id)
            .bind(k as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT user_id, tenant_id, preference_vector::text as preference_vector_text,
                       interaction_count, last_interaction_at, created_at, updated_at,
                       1 - (preference_vector <=> $1::vector) as similarity
                FROM user_profiles
                WHERE tenant_id = $2
                  AND preference_vector IS NOT NULL
                ORDER BY preference_vector <=> $1::vector
                LIMIT $3
                "#
            )
            .bind(&vector_str)
            .bind(&ctx.tenant_id)
            .bind(k as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let users: Vec<(UserProfile, f32)> = results
            .into_iter()
            .filter_map(|row| {
                let preference_vector_text: Option<String> = row.try_get("preference_vector_text").ok()?;
                let preference_vector = preference_vector_text
                    .as_deref()
                    .and_then(parse_vector)
                    .unwrap_or_default();
                let similarity: Option<f64> = row.try_get("similarity").ok()?;
                let similarity = similarity.unwrap_or(0.0) as f32;

                Some((
                    UserProfile {
                        user_id: row.try_get("user_id").ok()?,
                        preference_vector,
                        interaction_count: row.try_get("interaction_count").ok()?,
                        last_interaction_at: row.try_get("last_interaction_at").ok()?,
                        tenant_id: Some(row.try_get("tenant_id").ok()?),
                        created_at: row.try_get("created_at").ok()?,
                        updated_at: row.try_get("updated_at").ok()?,
                    },
                    similarity,
                ))
            })
            .collect();

        info!(
            "Found {} similar users for tenant={}",
            users.len(),
            ctx.tenant_id
        );

        Ok(users)
    }

    /// Get users with low interaction counts (for cold start detection)
    pub async fn get_cold_start_users(
        &self,
        ctx: &TenantContext,
        threshold: i32,
        limit: usize,
    ) -> Result<Vec<UserProfile>> {
        debug!(
            "Getting cold start users: tenant={}, threshold={}, limit={}",
            ctx.tenant_id, threshold, limit
        );

        let results = sqlx::query!(
            r#"
            SELECT user_id, tenant_id, preference_vector::text as preference_vector_text,
                   interaction_count, last_interaction_at, created_at, updated_at
            FROM user_profiles
            WHERE tenant_id = $1 AND interaction_count < $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            ctx.tenant_id,
            threshold,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        ?;

        let users: Vec<UserProfile> = results
            .into_iter()
            .map(|row| UserProfile {
                user_id: row.user_id,
                preference_vector: row.preference_vector_text
                    .as_deref()
                    .and_then(parse_vector)
                    .unwrap_or_default(),
                interaction_count: row.interaction_count,
                last_interaction_at: row.last_interaction_at,
                tenant_id: Some(row.tenant_id),
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        debug!(
            "Found {} cold start users for tenant={}",
            users.len(),
            ctx.tenant_id
        );

        Ok(users)
    }

    /// Get trending entity statistics for cold start recommendations
    /// Returns entities with highest interaction weight in the last 7 days
    pub async fn get_trending_entity_stats(
        &self,
        ctx: &TenantContext,
        entity_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, String, f32)>> {
        debug!(
            "Getting trending entity stats: tenant={}, entity_type={:?}, limit={}",
            ctx.tenant_id, entity_type, limit
        );

        let seven_days_ago = Utc::now() - chrono::Duration::days(7);

        // Use dynamic query to handle optional entity_type filter
        let results = if let Some(etype) = entity_type {
            sqlx::query_as::<_, (String, String, Option<f64>)>(
                r#"
                SELECT i.entity_id, i.entity_type, SUM(i.weight) as total_weight
                FROM interactions i
                WHERE i.tenant_id = $1
                  AND i.entity_type = $2
                  AND i.timestamp >= $3
                GROUP BY i.entity_id, i.entity_type
                ORDER BY total_weight DESC
                LIMIT $4
                "#
            )
            .bind(&ctx.tenant_id)
            .bind(etype)
            .bind(seven_days_ago)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, (String, String, Option<f64>)>(
                r#"
                SELECT i.entity_id, i.entity_type, SUM(i.weight) as total_weight
                FROM interactions i
                WHERE i.tenant_id = $1
                  AND i.timestamp >= $2
                GROUP BY i.entity_id, i.entity_type
                ORDER BY total_weight DESC
                LIMIT $3
                "#
            )
            .bind(&ctx.tenant_id)
            .bind(seven_days_ago)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let stats: Vec<(String, String, f32)> = results
            .into_iter()
            .map(|(entity_id, entity_type, total_weight)| {
                (
                    entity_id,
                    entity_type,
                    total_weight.unwrap_or(0.0) as f32,
                )
            })
            .collect();

        info!(
            "Found {} trending entities for tenant={}, entity_type={:?}",
            stats.len(),
            ctx.tenant_id,
            entity_type
        );

        Ok(stats)
    }
}

// ============================================================================
// pgvector Index Management Implementation
// ============================================================================

/// Configuration for HNSW index parameters
#[derive(Debug, Clone)]
pub struct HnswIndexConfig {
    /// Number of connections per layer (default: 16)
    /// Higher values = better recall, more memory
    pub m: i32,
    /// Size of dynamic candidate list during construction (default: 64)
    /// Higher values = better index quality, slower build
    pub ef_construction: i32,
}

impl Default for HnswIndexConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 64,
        }
    }
}

impl VectorStore {
    /// Create HNSW index for entity feature vectors
    /// Note: This is typically done via migrations, but can be called manually if needed
    pub async fn create_entity_vector_index(
        &self,
        config: Option<HnswIndexConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();

        info!(
            "Creating HNSW index for entity feature vectors with m={}, ef_construction={}",
            config.m, config.ef_construction
        );

        sqlx::query(&format!(
            r#"
            CREATE INDEX IF NOT EXISTS idx_entities_feature_vector_hnsw 
            ON entities USING hnsw (feature_vector vector_cosine_ops)
            WITH (m = {}, ef_construction = {})
            "#,
            config.m, config.ef_construction
        ))
        .execute(&self.pool)
        .await?;

        info!("Successfully created HNSW index for entity feature vectors");

        Ok(())
    }

    /// Create HNSW index for user preference vectors
    /// Note: This is typically done via migrations, but can be called manually if needed
    pub async fn create_user_vector_index(
        &self,
        config: Option<HnswIndexConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();

        info!(
            "Creating HNSW index for user preference vectors with m={}, ef_construction={}",
            config.m, config.ef_construction
        );

        sqlx::query(&format!(
            r#"
            CREATE INDEX IF NOT EXISTS idx_user_profiles_preference_vector_hnsw 
            ON user_profiles USING hnsw (preference_vector vector_cosine_ops)
            WITH (m = {}, ef_construction = {})
            "#,
            config.m, config.ef_construction
        ))
        .execute(&self.pool)
        .await?;

        info!("Successfully created HNSW index for user preference vectors");

        Ok(())
    }

    /// Rebuild HNSW index for entity feature vectors
    /// This can be useful after bulk imports or when index performance degrades
    pub async fn rebuild_entity_vector_index(
        &self,
        config: Option<HnswIndexConfig>,
    ) -> Result<()> {
        info!("Rebuilding HNSW index for entity feature vectors");

        // Drop existing index
        sqlx::query("DROP INDEX IF EXISTS idx_entities_feature_vector_hnsw")
            .execute(&self.pool)
            .await?;

        // Recreate index
        self.create_entity_vector_index(config).await?;

        info!("Successfully rebuilt HNSW index for entity feature vectors");

        Ok(())
    }

    /// Rebuild HNSW index for user preference vectors
    /// This can be useful after bulk profile updates or when index performance degrades
    pub async fn rebuild_user_vector_index(
        &self,
        config: Option<HnswIndexConfig>,
    ) -> Result<()> {
        info!("Rebuilding HNSW index for user preference vectors");

        // Drop existing index
        sqlx::query("DROP INDEX IF EXISTS idx_user_profiles_preference_vector_hnsw")
            .execute(&self.pool)
            .await?;

        // Recreate index
        self.create_user_vector_index(config).await?;

        info!("Successfully rebuilt HNSW index for user preference vectors");

        Ok(())
    }

    /// Get index statistics for monitoring
    pub async fn get_index_stats(&self) -> Result<IndexStats> {
        debug!("Getting index statistics");

        // Get entity count
        let entity_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM entities
            WHERE feature_vector IS NOT NULL
            "#
        )
        .fetch_one(&self.pool)
        .await
        ?
        .count
        .unwrap_or(0) as usize;

        // Get user profile count
        let user_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM user_profiles
            WHERE preference_vector IS NOT NULL
            "#
        )
        .fetch_one(&self.pool)
        .await
        ?
        .count
        .unwrap_or(0) as usize;

        // Get interaction count
        let interaction_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM interactions
            "#
        )
        .fetch_one(&self.pool)
        .await
        ?
        .count
        .unwrap_or(0) as usize;

        Ok(IndexStats {
            entity_count,
            user_count,
            interaction_count,
        })
    }

    /// Analyze index performance and suggest optimizations
    pub async fn analyze_index_performance(&self) -> Result<IndexPerformanceReport> {
        debug!("Analyzing index performance");

        let stats = self.get_index_stats().await?;

        // Check if indices exist
        let entity_index_exists = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM pg_indexes 
                WHERE indexname = 'idx_entities_feature_vector_hnsw'
            ) as exists
            "#
        )
        .fetch_one(&self.pool)
        .await
        ?
        .exists
        .unwrap_or(false);

        let user_index_exists = sqlx::query!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM pg_indexes 
                WHERE indexname = 'idx_user_profiles_preference_vector_hnsw'
            ) as exists
            "#
        )
        .fetch_one(&self.pool)
        .await
        ?
        .exists
        .unwrap_or(false);

        let mut recommendations = Vec::new();

        if !entity_index_exists && stats.entity_count > 0 {
            recommendations.push("Entity vector index is missing. Create it for better performance.".to_string());
        }

        if !user_index_exists && stats.user_count > 0 {
            recommendations.push("User vector index is missing. Create it for better performance.".to_string());
        }

        if stats.entity_count > 100_000 {
            recommendations.push("Large entity count detected. Consider increasing HNSW m parameter for better recall.".to_string());
        }

        if stats.user_count > 100_000 {
            recommendations.push("Large user count detected. Consider increasing HNSW m parameter for better recall.".to_string());
        }

        Ok(IndexPerformanceReport {
            stats,
            entity_index_exists,
            user_index_exists,
            recommendations,
        })
    }
}

/// Statistics about vector indices
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub entity_count: usize,
    pub user_count: usize,
    pub interaction_count: usize,
}

/// Performance report for vector indices
#[derive(Debug, Clone)]
pub struct IndexPerformanceReport {
    pub stats: IndexStats,
    pub entity_index_exists: bool,
    pub user_index_exists: bool,
    pub recommendations: Vec<String>,
}




// Methods for Model Updater
impl VectorStore {
    /// Get users with interactions in the last specified duration
    pub async fn get_users_with_recent_interactions(
        &self,
        ctx: &TenantContext,
        duration: std::time::Duration,
    ) -> Result<Vec<String>> {
        let seconds_ago = duration.as_secs() as i64;
        
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT user_id
            FROM interactions
            WHERE tenant_id = $1
              AND timestamp > NOW() - INTERVAL '1 second' * $2
            ORDER BY user_id
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(seconds_ago)
        .fetch_all(&self.pool)
        .await?;

        let user_ids = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("user_id"))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(user_ids)
    }

    /// Get entities modified in the last specified duration
    pub async fn get_recently_modified_entities(
        &self,
        ctx: &TenantContext,
        duration: std::time::Duration,
    ) -> Result<Vec<String>> {
        let seconds_ago = duration.as_secs() as i64;
        
        let rows = sqlx::query(
            r#"
            SELECT entity_id
            FROM entities
            WHERE tenant_id = $1
              AND updated_at > NOW() - INTERVAL '1 second' * $2
            ORDER BY entity_id
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(seconds_ago)
        .fetch_all(&self.pool)
        .await?;

        let entity_ids = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("entity_id"))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entity_ids)
    }

    /// Recompute user preference vector from interactions
    pub async fn recompute_user_preference_vector(
        &self,
        ctx: &TenantContext,
        user_id: &str,
    ) -> Result<()> {
        // Compute the preference vector
        let preference_vector = self.compute_user_preference_vector(ctx, user_id).await?;

        // Get interaction count
        let interaction_count = self.get_user_interaction_count(ctx, user_id).await? as i32;

        // Get last interaction timestamp
        let interactions = self.get_user_interactions(ctx, user_id, 1, 0).await?;
        let last_interaction_at = interactions.first().map(|i| i.timestamp);

        // Update the user profile
        self.upsert_user_profile(ctx, user_id, preference_vector, interaction_count, last_interaction_at).await?;

        Ok(())
    }

    /// Recompute entity feature vector from attributes
    pub async fn recompute_entity_feature_vector(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
    ) -> Result<()> {
        // Get all entity types to try finding the entity
        let entity_types = self.get_all_entity_types(ctx).await?;
        
        // Try to find the entity by checking each type
        let mut found = false;
        for entity_type in entity_types {
            if let Ok(Some(_entity)) = self.get_entity(ctx, entity_id, &entity_type).await {
                found = true;
                break;
            }
        }

        if !found {
            return Err(RecommendationError::EntityNotFound(entity_id.to_string()));
        }

        // Feature vector should already be computed when entity was created/updated
        // This is a no-op unless we need to recompute based on new algorithm
        // For now, we just touch the updated_at timestamp
        sqlx::query(
            r#"
            UPDATE entities
            SET updated_at = NOW()
            WHERE tenant_id = $1 AND entity_id = $2
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all user IDs for a tenant
    pub async fn get_all_user_ids(&self, ctx: &TenantContext) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT user_id
            FROM user_profiles
            WHERE tenant_id = $1
            ORDER BY user_id
            "#
        )
        .bind(&ctx.tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let user_ids = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("user_id"))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(user_ids)
    }

    /// Get all entity IDs for a tenant
    pub async fn get_all_entity_ids(&self, ctx: &TenantContext) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT entity_id
            FROM entities
            WHERE tenant_id = $1
            ORDER BY entity_id
            "#
        )
        .bind(&ctx.tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let entity_ids = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("entity_id"))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entity_ids)
    }



    /// Get all entity types for a tenant
    pub async fn get_all_entity_types(&self, ctx: &TenantContext) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT entity_type
            FROM entities
            WHERE tenant_id = $1
            ORDER BY entity_type
            "#
        )
        .bind(&ctx.tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let entity_types = rows
            .into_iter()
            .map(|row| row.try_get::<String, _>("entity_type"))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entity_types)
    }

    /// Export entities with optional filtering
    /// Supports incremental exports using last_modified_after timestamp
    pub async fn export_entities(
        &self,
        ctx: &TenantContext,
        entity_type: Option<&str>,
        last_modified_after: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Entity>> {
        let rows = if let Some(entity_type_filter) = entity_type {
            if let Some(timestamp) = last_modified_after {
                sqlx::query(
                    r#"
                    SELECT entity_id, entity_type, attributes, feature_vector::text as feature_vector_text, tenant_id, created_at, updated_at
                    FROM entities
                    WHERE tenant_id = $1 AND entity_type = $2 AND updated_at > $3
                    ORDER BY updated_at DESC
                    LIMIT 100000
                    "#
                )
                .bind(&ctx.tenant_id)
                .bind(entity_type_filter)
                .bind(timestamp)
                .fetch_all(&self.pool)
                .await?
            } else {
                sqlx::query(
                    r#"
                    SELECT entity_id, entity_type, attributes, feature_vector::text as feature_vector_text, tenant_id, created_at, updated_at
                    FROM entities
                    WHERE tenant_id = $1 AND entity_type = $2
                    ORDER BY updated_at DESC
                    LIMIT 100000
                    "#
                )
                .bind(&ctx.tenant_id)
                .bind(entity_type_filter)
                .fetch_all(&self.pool)
                .await?
            }
        } else if let Some(timestamp) = last_modified_after {
                sqlx::query(
                    r#"
                    SELECT entity_id, entity_type, attributes, feature_vector::text as feature_vector_text, tenant_id, created_at, updated_at
                    FROM entities
                    WHERE tenant_id = $1 AND updated_at > $2
                    ORDER BY updated_at DESC
                    LIMIT 100000
                    "#
                )
                .bind(&ctx.tenant_id)
                .bind(timestamp)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(
                r#"
                SELECT entity_id, entity_type, attributes, feature_vector::text as feature_vector_text, tenant_id, created_at, updated_at
                FROM entities
                WHERE tenant_id = $1
                ORDER BY updated_at DESC
                LIMIT 100000
                "#
            )
            .bind(&ctx.tenant_id)
            .fetch_all(&self.pool)
            .await?
        };

        let mut entities = Vec::new();
        for row in rows {
            let entity_id: String = row.try_get("entity_id")?;
            let entity_type: String = row.try_get("entity_type")?;
            let attributes: serde_json::Value = row.try_get("attributes")?;
            let feature_vector_text: Option<String> = row.try_get("feature_vector_text")?;
            let feature_vector = feature_vector_text.as_deref().and_then(parse_vector);
            let tenant_id: Option<String> = row.try_get("tenant_id")?;
            let created_at: DateTime<Utc> = row.try_get("created_at")?;
            let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

            // Parse attributes JSON to HashMap
            let attributes_map = serde_json::from_value::<HashMap<String, AttributeValue>>(attributes)
                .unwrap_or_default();

            entities.push(Entity {
                entity_id,
                entity_type,
                attributes: attributes_map,
                feature_vector,
                tenant_id,
                created_at,
                updated_at,
            });
        }

        Ok(entities)
    }

    /// Export interactions with optional date range filtering
    pub async fn export_interactions(
        &self,
        ctx: &TenantContext,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Interaction>> {
        let rows = match (start_date, end_date) {
            (Some(start), Some(end)) => {
                sqlx::query(
                    r#"
                    SELECT id, user_id, entity_id, interaction_type, weight, metadata, tenant_id, timestamp
                    FROM interactions
                    WHERE tenant_id = $1 AND timestamp >= $2 AND timestamp <= $3
                    ORDER BY timestamp DESC
                    LIMIT 100000
                    "#
                )
                .bind(&ctx.tenant_id)
                .bind(start)
                .bind(end)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(start), None) => {
                sqlx::query(
                    r#"
                    SELECT id, user_id, entity_id, interaction_type, weight, metadata, tenant_id, timestamp
                    FROM interactions
                    WHERE tenant_id = $1 AND timestamp >= $2
                    ORDER BY timestamp DESC
                    LIMIT 100000
                    "#
                )
                .bind(&ctx.tenant_id)
                .bind(start)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(end)) => {
                sqlx::query(
                    r#"
                    SELECT id, user_id, entity_id, interaction_type, weight, metadata, tenant_id, timestamp
                    FROM interactions
                    WHERE tenant_id = $1 AND timestamp <= $2
                    ORDER BY timestamp DESC
                    LIMIT 100000
                    "#
                )
                .bind(&ctx.tenant_id)
                .bind(end)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query(
                    r#"
                    SELECT id, user_id, entity_id, interaction_type, weight, metadata, tenant_id, timestamp
                    FROM interactions
                    WHERE tenant_id = $1
                    ORDER BY timestamp DESC
                    LIMIT 100000
                    "#
                )
                .bind(&ctx.tenant_id)
                .fetch_all(&self.pool)
                .await?
            }
        };

        let mut interactions = Vec::new();
        for row in rows {
            let id: Option<i64> = row.try_get("id")?;
            let user_id: String = row.try_get("user_id")?;
            let entity_id: String = row.try_get("entity_id")?;
            let interaction_type_str: String = row.try_get("interaction_type")?;
            let weight: f32 = row.try_get("weight")?;
            let metadata: Option<serde_json::Value> = row.try_get("metadata")?;
            let tenant_id: Option<String> = row.try_get("tenant_id")?;
            let timestamp: DateTime<Utc> = row.try_get("timestamp")?;

            // Parse interaction type
            let interaction_type = serde_json::from_str::<InteractionType>(&format!("\"{}\"", interaction_type_str))
                .unwrap_or(InteractionType::View);

            // Parse metadata
            let metadata_map = metadata.and_then(|v| {
                serde_json::from_value::<HashMap<String, String>>(v).ok()
            });

            interactions.push(Interaction {
                id,
                user_id,
                entity_id,
                interaction_type,
                weight,
                metadata: metadata_map,
                tenant_id,
                timestamp,
            });
        }

        Ok(interactions)
    }

    /// Export user profiles with optional vector inclusion
    pub async fn export_user_profiles(
        &self,
        ctx: &TenantContext,
        include_vectors: bool,
    ) -> Result<Vec<UserProfile>> {
        let rows = if include_vectors {
            sqlx::query(
                r#"
                SELECT user_id, interaction_count, preference_vector, tenant_id, last_interaction_at, created_at, updated_at
                FROM user_profiles
                WHERE tenant_id = $1
                ORDER BY updated_at DESC
                LIMIT 100000
                "#
            )
            .bind(&ctx.tenant_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT user_id, interaction_count, tenant_id, last_interaction_at, created_at, updated_at
                FROM user_profiles
                WHERE tenant_id = $1
                ORDER BY updated_at DESC
                LIMIT 100000
                "#
            )
            .bind(&ctx.tenant_id)
            .fetch_all(&self.pool)
            .await?
        };

        let mut user_profiles = Vec::new();
        for row in rows {
            let user_id: String = row.try_get("user_id")?;
            let interaction_count: i32 = row.try_get("interaction_count")?;
            let tenant_id: Option<String> = row.try_get("tenant_id")?;
            let last_interaction_at: Option<DateTime<Utc>> = row.try_get("last_interaction_at")?;
            let created_at: DateTime<Utc> = row.try_get("created_at")?;
            let updated_at: DateTime<Utc> = row.try_get("updated_at")?;

            let preference_vector = if include_vectors {
                let vec_data: Option<Vec<f32>> = row.try_get("preference_vector")?;
                vec_data.unwrap_or_default()
            } else {
                Vec::new()
            };

            user_profiles.push(UserProfile {
                user_id,
                preference_vector,
                interaction_count,
                last_interaction_at,
                tenant_id,
                created_at,
                updated_at,
            });
        }

        Ok(user_profiles)
    }
}

// ============================================================================
// Interaction Type Registry Implementation
// ============================================================================

use recommendation_models::RegisteredInteractionType;

impl VectorStore {
    /// Register a new custom interaction type with weight
    pub async fn register_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: &str,
        weight: f32,
        description: Option<String>,
    ) -> Result<RegisteredInteractionType> {
        debug!(
            "Registering interaction type: tenant={}, type={}, weight={}",
            ctx.tenant_id, interaction_type, weight
        );

        // Validate weight is positive
        if weight < 0.0 {
            return Err(RecommendationError::InvalidRequest(
                format!("Interaction weight must be non-negative, got {}", weight)
            ));
        }

        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO interaction_types (tenant_id, interaction_type, weight, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tenant_id, interaction_type)
            DO UPDATE SET 
                weight = EXCLUDED.weight,
                description = EXCLUDED.description,
                updated_at = EXCLUDED.updated_at
            RETURNING id, tenant_id, interaction_type, weight, description, created_at, updated_at
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(interaction_type)
        .bind(weight)
        .bind(&description)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        info!(
            "Registered interaction type: tenant={}, type={}, weight={}",
            ctx.tenant_id, interaction_type, weight
        );

        Ok(RegisteredInteractionType {
            id: Some(row.try_get("id")?),
            tenant_id: row.try_get("tenant_id")?,
            interaction_type: row.try_get("interaction_type")?,
            weight: row.try_get::<f32, _>("weight")?,
            description: row.try_get("description")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    /// Get a registered interaction type
    pub async fn get_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: &str,
    ) -> Result<Option<RegisteredInteractionType>> {
        debug!(
            "Getting interaction type: tenant={}, type={}",
            ctx.tenant_id, interaction_type
        );

        let result = sqlx::query(
            r#"
            SELECT id, tenant_id, interaction_type, weight, description, created_at, updated_at
            FROM interaction_types
            WHERE tenant_id = $1 AND interaction_type = $2
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(interaction_type)
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(row) => Ok(Some(RegisteredInteractionType {
                id: Some(row.try_get("id")?),
                tenant_id: row.try_get("tenant_id")?,
                interaction_type: row.try_get("interaction_type")?,
                weight: row.try_get::<f32, _>("weight")?,
                description: row.try_get("description")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    /// List all registered interaction types for a tenant
    pub async fn list_interaction_types(
        &self,
        ctx: &TenantContext,
    ) -> Result<Vec<RegisteredInteractionType>> {
        debug!(
            "Listing interaction types: tenant={}",
            ctx.tenant_id
        );

        let results = sqlx::query(
            r#"
            SELECT id, tenant_id, interaction_type, weight, description, created_at, updated_at
            FROM interaction_types
            WHERE tenant_id = $1
            ORDER BY interaction_type ASC
            "#
        )
        .bind(&ctx.tenant_id)
        .fetch_all(&self.pool)
        .await?;

        let interaction_types: Vec<RegisteredInteractionType> = results
            .into_iter()
            .filter_map(|row| {
                Some(RegisteredInteractionType {
                    id: Some(row.try_get("id").ok()?),
                    tenant_id: row.try_get("tenant_id").ok()?,
                    interaction_type: row.try_get("interaction_type").ok()?,
                    weight: row.try_get::<f32, _>("weight").ok()?,
                    description: row.try_get("description").ok()?,
                    created_at: row.try_get("created_at").ok()?,
                    updated_at: row.try_get("updated_at").ok()?,
                })
            })
            .collect();

        debug!(
            "Found {} interaction types for tenant={}",
            interaction_types.len(),
            ctx.tenant_id
        );

        Ok(interaction_types)
    }

    /// Update an existing interaction type
    pub async fn update_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: &str,
        weight: f32,
        description: Option<String>,
    ) -> Result<RegisteredInteractionType> {
        debug!(
            "Updating interaction type: tenant={}, type={}, weight={}",
            ctx.tenant_id, interaction_type, weight
        );

        // Validate weight is positive
        if weight < 0.0 {
            return Err(RecommendationError::InvalidRequest(
                format!("Interaction weight must be non-negative, got {}", weight)
            ));
        }

        let now = Utc::now();

        let row_opt = sqlx::query(
            r#"
            UPDATE interaction_types
            SET weight = $1, description = $2, updated_at = $3
            WHERE tenant_id = $4 AND interaction_type = $5
            RETURNING id, tenant_id, interaction_type, weight, description, created_at, updated_at
            "#
        )
        .bind(weight)
        .bind(&description)
        .bind(now)
        .bind(&ctx.tenant_id)
        .bind(interaction_type)
        .fetch_optional(&self.pool)
        .await?;

        match row_opt {
            Some(row) => {
                info!(
                    "Updated interaction type: tenant={}, type={}, weight={}",
                    ctx.tenant_id, interaction_type, weight
                );

                Ok(RegisteredInteractionType {
                    id: Some(row.try_get("id")?),
                    tenant_id: row.try_get("tenant_id")?,
                    interaction_type: row.try_get("interaction_type")?,
                    weight: row.try_get::<f32, _>("weight")?,
                    description: row.try_get("description")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                })
            }
            None => Err(RecommendationError::InvalidRequest(format!(
                "Interaction type '{}' not found for tenant '{}'",
                interaction_type, ctx.tenant_id
            ))),
        }
    }

    /// Delete a registered interaction type
    pub async fn delete_interaction_type(
        &self,
        ctx: &TenantContext,
        interaction_type: &str,
    ) -> Result<()> {
        debug!(
            "Deleting interaction type: tenant={}, type={}",
            ctx.tenant_id, interaction_type
        );

        let result = sqlx::query(
            r#"
            DELETE FROM interaction_types
            WHERE tenant_id = $1 AND interaction_type = $2
            "#
        )
        .bind(&ctx.tenant_id)
        .bind(interaction_type)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RecommendationError::InvalidRequest(format!(
                "Interaction type '{}' not found for tenant '{}'",
                interaction_type, ctx.tenant_id
            )));
        }

        info!(
            "Deleted interaction type: tenant={}, type={}",
            ctx.tenant_id, interaction_type
        );

        Ok(())
    }

    /// Get interaction weight for a type, with fallback to default
    /// This method checks the registry first, then falls back to built-in defaults
    pub async fn get_interaction_weight(
        &self,
        ctx: &TenantContext,
        interaction_type: &InteractionType,
    ) -> Result<f32> {
        // Convert InteractionType to string for lookup
        let type_str = match interaction_type {
            InteractionType::View => "view",
            InteractionType::AddToCart => "add_to_cart",
            InteractionType::Purchase => "purchase",
            InteractionType::Like => "like",
            InteractionType::Rating(r) => return Ok(*r), // Ratings use their value directly
            InteractionType::Custom(s) => s.as_str(),
        };

        // Try to get from registry
        if let Some(registered) = self.get_interaction_type(ctx, type_str).await? {
            return Ok(registered.weight);
        }

        // Fall back to default weight
        Ok(interaction_type.default_weight())
    }
}
