use recommendation_models::{
    RecommendationError, Result, ScoredEntity, TenantContext, Entity,
};
use recommendation_storage::{RedisCache, VectorStore};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, info};

/// Configuration for content-based filtering
#[derive(Debug, Clone)]
pub struct ContentBasedConfig {
    /// Minimum similarity threshold for recommendations (default: 0.5)
    pub similarity_threshold: f32,
    /// Default number of recommendations to return
    pub default_count: usize,
}

impl Default for ContentBasedConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.5,
            default_count: 10,
        }
    }
}

/// Content-based filtering engine using entity feature vectors
pub struct ContentBasedFilteringEngine {
    vector_store: Arc<VectorStore>,
    cache: Arc<RedisCache>,
    config: ContentBasedConfig,
}

impl ContentBasedFilteringEngine {
    /// Create a new content-based filtering engine
    pub fn new(
        vector_store: Arc<VectorStore>,
        cache: Arc<RedisCache>,
        config: ContentBasedConfig,
    ) -> Self {
        info!(
            "Initializing ContentBasedFilteringEngine with similarity_threshold={}",
            config.similarity_threshold
        );
        Self {
            vector_store,
            cache,
            config,
        }
    }

    /// Find similar entities using pgvector cosine similarity
    /// Returns list of (Entity, similarity_score) tuples
    pub async fn find_similar_entities(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
        count: usize,
    ) -> Result<Vec<(Entity, f32)>> {
        debug!(
            "Finding similar entities for entity={}, entity_type={}, tenant={}",
            entity_id, entity_type, ctx.tenant_id
        );

        // Get target entity
        let entity = self
            .vector_store
            .get_entity(ctx, entity_id, entity_type)
            .await?
            .ok_or_else(|| {
                RecommendationError::EntityNotFound(format!(
                    "Entity not found: entity_id={}, entity_type={}",
                    entity_id, entity_type
                ))
            })?;

        // Check if entity has a feature vector
        let feature_vector = entity.feature_vector.as_ref().ok_or_else(|| {
            RecommendationError::VectorError(format!(
                "Entity {} has no feature vector",
                entity_id
            ))
        })?;

        // Find similar entities using pgvector
        let similar_entities = self
            .vector_store
            .find_similar_entities(
                ctx,
                feature_vector,
                entity_type,
                self.config.similarity_threshold,
                count,
                Some(entity_id),
            )
            .await?;

        info!(
            "Found {} similar entities for entity={} (threshold={})",
            similar_entities.len(),
            entity_id,
            self.config.similarity_threshold
        );

        Ok(similar_entities)
    }

    /// Generate recommendations based on entity similarity
    /// Returns top-N similar entities sorted by similarity score
    pub async fn generate_recommendations(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
        count: usize,
    ) -> Result<Vec<ScoredEntity>> {
        debug!(
            "Generating content-based recommendations for entity={}, entity_type={}, count={}",
            entity_id, entity_type, count
        );

        // Try to get from cache first
        let cache_key = format!(
            "content_rec:{}:{}:{}:{}",
            ctx.tenant_id, entity_id, entity_type, count
        );

        if let Ok(Some(cached)) = self.cache.get::<Vec<ScoredEntity>>(&cache_key).await {
            debug!("Returning cached content-based recommendations");
            return Ok(cached);
        }

        // Find similar entities
        let similar_entities = self
            .find_similar_entities(ctx, entity_id, entity_type, count)
            .await?;

        // Convert to ScoredEntity
        let recommendations: Vec<ScoredEntity> = similar_entities
            .into_iter()
            .map(|(entity, similarity)| ScoredEntity {
                entity_id: entity.entity_id,
                entity_type: entity.entity_type,
                score: similarity,
                reason: Some(format!(
                    "Similar to {} (similarity: {:.2})",
                    entity_id, similarity
                )),
            })
            .collect();

        // Cache for 5 minutes
        let _ = self
            .cache
            .set(&cache_key, &recommendations, std::time::Duration::from_secs(300))
            .await;

        info!(
            "Generated {} content-based recommendations for entity={}",
            recommendations.len(),
            entity_id
        );

        Ok(recommendations)
    }

    /// Generate recommendations for a user based on their interaction history
    /// Finds entities similar to those the user has interacted with
    pub async fn generate_user_recommendations(
        &self,
        ctx: &TenantContext,
        user_id: &str,
        entity_type: &str,
        count: usize,
    ) -> Result<Vec<ScoredEntity>> {
        debug!(
            "Generating content-based user recommendations for user={}, entity_type={}, count={}",
            user_id, entity_type, count
        );

        // Get user's recent interactions (limit to 20 most recent)
        let interactions = self
            .vector_store
            .get_user_interactions(ctx, user_id, 20, 0)
            .await?;

        if interactions.is_empty() {
            debug!(
                "No interactions found for user={}, cannot generate content-based recommendations",
                user_id
            );
            return Ok(Vec::new());
        }

        // Get entities the user has already interacted with (for exclusion)
        let interacted_entities = self
            .vector_store
            .get_user_interacted_entities(ctx, user_id)
            .await?;
        let interacted_set: HashSet<String> = interacted_entities.into_iter().collect();

        // Aggregate similar entities from all interacted entities
        let mut entity_scores: std::collections::HashMap<String, f32> = std::collections::HashMap::new();

        for interaction in interactions {
            // Skip if not the right entity type
            // Note: We need to check entity type by fetching the entity
            if let Ok(Some(_entity)) = self
                .vector_store
                .get_entity(ctx, &interaction.entity_id, entity_type)
                .await
            {
                // Find similar entities to this one
                if let Ok(similar) = self
                    .find_similar_entities(ctx, &interaction.entity_id, entity_type, count * 2)
                    .await
                {
                    for (similar_entity, similarity) in similar {
                        // Skip if user already interacted with this entity
                        if interacted_set.contains(&similar_entity.entity_id) {
                            continue;
                        }

                        // Weight similarity by interaction weight
                        let weighted_score = similarity * interaction.weight;

                        entity_scores
                            .entry(similar_entity.entity_id.clone())
                            .and_modify(|score| *score += weighted_score)
                            .or_insert(weighted_score);
                    }
                }
            }
        }

        // Convert to ScoredEntity and sort by score
        let mut recommendations: Vec<ScoredEntity> = entity_scores
            .into_iter()
            .map(|(entity_id, score)| ScoredEntity {
                entity_id,
                entity_type: entity_type.to_string(),
                score,
                reason: Some("Similar to items you liked".to_string()),
            })
            .collect();

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(count);

        info!(
            "Generated {} content-based user recommendations for user={}",
            recommendations.len(),
            user_id
        );

        Ok(recommendations)
    }

    /// Handle cold start for new entities by recommending based on content similarity to popular ones
    /// Returns entities similar to trending/popular entities
    pub async fn get_cold_start_recommendations(
        &self,
        ctx: &TenantContext,
        entity_type: &str,
        count: usize,
    ) -> Result<Vec<ScoredEntity>> {
        debug!(
            "Getting cold start recommendations for entity_type={}, count={}",
            entity_type, count
        );

        // Get trending entities (popular ones)
        let trending = self
            .vector_store
            .get_trending_entity_stats(ctx, Some(entity_type), 5)
            .await?;

        if trending.is_empty() {
            debug!(
                "No trending entities found for entity_type={}, cannot generate cold start recommendations",
                entity_type
            );
            return Ok(Vec::new());
        }

        // Find entities similar to trending ones
        let mut entity_scores: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        let mut seen_entities: HashSet<String> = HashSet::new();

        for (trending_entity_id, _, popularity_score) in trending {
            // Skip if we've already seen this entity
            if seen_entities.contains(&trending_entity_id) {
                continue;
            }
            seen_entities.insert(trending_entity_id.clone());

            // Find similar entities
            if let Ok(similar) = self
                .find_similar_entities(ctx, &trending_entity_id, entity_type, count * 2)
                .await
            {
                for (similar_entity, similarity) in similar {
                    // Skip if already seen
                    if seen_entities.contains(&similar_entity.entity_id) {
                        continue;
                    }

                    // Weight similarity by popularity of the trending entity
                    let weighted_score = similarity * popularity_score;

                    entity_scores
                        .entry(similar_entity.entity_id.clone())
                        .and_modify(|score| *score += weighted_score)
                        .or_insert(weighted_score);
                }
            }
        }

        // Convert to ScoredEntity and sort by score
        let mut recommendations: Vec<ScoredEntity> = entity_scores
            .into_iter()
            .map(|(entity_id, score)| ScoredEntity {
                entity_id,
                entity_type: entity_type.to_string(),
                score,
                reason: Some("Similar to popular items".to_string()),
            })
            .collect();

        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(count);

        info!(
            "Generated {} cold start recommendations for entity_type={}",
            recommendations.len(),
            entity_type
        );

        Ok(recommendations)
    }

    /// Get recommendations with automatic cold start handling
    /// If entity is new (no similar entities found), returns cold start recommendations
    pub async fn get_recommendations_with_cold_start(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
        count: usize,
    ) -> Result<(Vec<ScoredEntity>, bool)> {
        debug!(
            "Getting recommendations with cold start handling for entity={}",
            entity_id
        );

        // Try to generate normal recommendations
        match self
            .generate_recommendations(ctx, entity_id, entity_type, count)
            .await
        {
            Ok(recommendations) if !recommendations.is_empty() => {
                Ok((recommendations, false))
            }
            Ok(_) | Err(_) => {
                // Entity is new or has no similar entities, use cold start
                info!(
                    "Entity {} is in cold start state, returning similar to popular items",
                    entity_id
                );
                let cold_start_recs = self
                    .get_cold_start_recommendations(ctx, entity_type, count)
                    .await?;
                Ok((cold_start_recs, true))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_based_config_default() {
        let config = ContentBasedConfig::default();
        assert_eq!(config.similarity_threshold, 0.5);
        assert_eq!(config.default_count, 10);
    }

    #[test]
    fn test_content_based_config_custom() {
        let config = ContentBasedConfig {
            similarity_threshold: 0.7,
            default_count: 20,
        };
        assert_eq!(config.similarity_threshold, 0.7);
        assert_eq!(config.default_count, 20);
    }

    #[test]
    fn test_content_based_config_clone() {
        let config1 = ContentBasedConfig {
            similarity_threshold: 0.6,
            default_count: 15,
        };
        let config2 = config1.clone();
        assert_eq!(config1.similarity_threshold, config2.similarity_threshold);
        assert_eq!(config1.default_count, config2.default_count);
    }

    #[test]
    fn test_content_based_config_debug() {
        let config = ContentBasedConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("similarity_threshold"));
        assert!(debug_str.contains("default_count"));
    }

    #[test]
    fn test_content_based_config_boundary_values() {
        let config = ContentBasedConfig {
            similarity_threshold: 0.0,
            default_count: 1,
        };
        assert_eq!(config.similarity_threshold, 0.0);
        assert_eq!(config.default_count, 1);

        let config2 = ContentBasedConfig {
            similarity_threshold: 1.0,
            default_count: 100,
        };
        assert_eq!(config2.similarity_threshold, 1.0);
        assert_eq!(config2.default_count, 100);
    }
}
