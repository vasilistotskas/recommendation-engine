use recommendation_models::{
    RecommendationError, Result, ScoredEntity, TenantContext, UserProfile,
};
use recommendation_storage::{RedisCache, VectorStore};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Configuration for collaborative filtering
#[derive(Debug, Clone)]
pub struct CollaborativeConfig {
    /// Number of similar users to consider (k in k-NN)
    pub k_neighbors: usize,
    /// Minimum similarity threshold for considering a neighbor
    pub min_similarity: f32,
    /// Default number of recommendations to return
    pub default_count: usize,
}

impl Default for CollaborativeConfig {
    fn default() -> Self {
        Self {
            k_neighbors: 50,
            min_similarity: 0.1,
            default_count: 10,
        }
    }
}

/// Collaborative filtering engine using user-based collaborative filtering
pub struct CollaborativeFilteringEngine {
    vector_store: Arc<VectorStore>,
    cache: Arc<RedisCache>,
    config: CollaborativeConfig,
}

impl CollaborativeFilteringEngine {
    /// Create a new collaborative filtering engine
    pub fn new(
        vector_store: Arc<VectorStore>,
        cache: Arc<RedisCache>,
        config: CollaborativeConfig,
    ) -> Self {
        info!(
            "Initializing CollaborativeFilteringEngine with k={}, min_similarity={}",
            config.k_neighbors, config.min_similarity
        );
        Self {
            vector_store,
            cache,
            config,
        }
    }

    /// Find similar users using cosine similarity on preference vectors
    /// Returns list of (UserProfile, similarity_score) tuples
    pub async fn find_similar_users(
        &self,
        ctx: &TenantContext,
        user_id: &str,
    ) -> Result<Vec<(UserProfile, f32)>> {
        debug!(
            "Finding similar users for user={}, tenant={}",
            user_id, ctx.tenant_id
        );

        // Get target user's profile
        let user_profile = self
            .vector_store
            .get_user_profile(ctx, user_id)
            .await?
            .ok_or_else(|| {
                RecommendationError::EntityNotFound(format!(
                    "User profile not found for user_id: {}",
                    user_id
                ))
            })?;

        // Check if user has a preference vector
        if user_profile.preference_vector.is_empty() {
            debug!(
                "User {} has no preference vector, cannot find similar users",
                user_id
            );
            return Ok(Vec::new());
        }

        // Find k-nearest neighbors using pgvector
        let similar_users = self
            .vector_store
            .find_similar_users(
                ctx,
                &user_profile.preference_vector,
                self.config.k_neighbors,
                Some(user_id),
            )
            .await?;

        // Filter by minimum similarity threshold
        let filtered_users: Vec<(UserProfile, f32)> = similar_users
            .into_iter()
            .filter(|(_, similarity)| *similarity >= self.config.min_similarity)
            .collect();

        info!(
            "Found {} similar users for user={} (after filtering by threshold {})",
            filtered_users.len(),
            user_id,
            self.config.min_similarity
        );

        Ok(filtered_users)
    }

    /// Calculate cosine similarity between two vectors
    /// Returns value in range [-1, 1] where 1 means identical, 0 means orthogonal, -1 means opposite
    pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() || vec1.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();

        let magnitude1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude1 * magnitude2)
    }

    /// Generate recommendations by aggregating preferences from similar users
    /// Returns top-N recommendations sorted by score
    pub async fn generate_recommendations(
        &self,
        ctx: &TenantContext,
        user_id: &str,
        count: usize,
        entity_type: Option<&str>,
    ) -> Result<Vec<ScoredEntity>> {
        debug!(
            "Generating collaborative recommendations for user={}, count={}, entity_type={:?}",
            user_id, count, entity_type
        );

        // Find similar users
        let similar_users = self.find_similar_users(ctx, user_id).await?;

        if similar_users.is_empty() {
            debug!(
                "No similar users found for user={}, cannot generate recommendations",
                user_id
            );
            return Ok(Vec::new());
        }

        // Get entities the target user has already interacted with (for exclusion)
        let interacted_entities = self
            .vector_store
            .get_user_interacted_entities(ctx, user_id)
            .await?;
        let interacted_set: HashSet<String> = interacted_entities.into_iter().collect();

        debug!(
            "User {} has interacted with {} entities (will be excluded)",
            user_id,
            interacted_set.len()
        );

        // Aggregate recommendations from similar users
        let recommendations = self
            .aggregate_recommendations_from_neighbors(
                ctx,
                &similar_users,
                &interacted_set,
                entity_type,
            )
            .await?;

        // Sort by score descending and take top N
        let mut sorted_recommendations = recommendations;
        sorted_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        sorted_recommendations.truncate(count);

        info!(
            "Generated {} collaborative recommendations for user={}",
            sorted_recommendations.len(),
            user_id
        );

        Ok(sorted_recommendations)
    }

    /// Aggregate recommendations from similar users
    /// Applies interaction weights and similarity scores to compute final scores
    async fn aggregate_recommendations_from_neighbors(
        &self,
        ctx: &TenantContext,
        similar_users: &[(UserProfile, f32)],
        exclude_entities: &HashSet<String>,
        entity_type_filter: Option<&str>,
    ) -> Result<Vec<ScoredEntity>> {
        debug!(
            "Aggregating recommendations from {} similar users",
            similar_users.len()
        );

        // Map to accumulate scores: (entity_id, entity_type) -> total_score
        let mut entity_scores: HashMap<(String, String), f32> = HashMap::new();

        // For each similar user, get their interactions and weight them by similarity
        for (user_profile, similarity) in similar_users {
            // Get interactions for this similar user (limit to recent 100)
            let interactions = self
                .vector_store
                .get_user_interactions(ctx, &user_profile.user_id, 100, 0)
                .await?;

            for interaction in interactions {
                // Skip if entity is in exclusion set
                if exclude_entities.contains(&interaction.entity_id) {
                    continue;
                }

                // Note: Interaction model doesn't have entity_type field in the current implementation
                // We'll use a placeholder and rely on entity lookup later
                // In production, consider adding entity_type to interactions table for better performance

                // Calculate weighted score: interaction_weight * user_similarity
                let weighted_score = interaction.weight * similarity;

                // Use entity_id as key for now, we'll resolve entity_type later
                let key = (interaction.entity_id.clone(), String::new());
                entity_scores
                    .entry(key)
                    .and_modify(|score| *score += weighted_score)
                    .or_insert(weighted_score);
            }
        }

        // Convert to ScoredEntity list by looking up entities
        let mut recommendations: Vec<ScoredEntity> = Vec::new();

        for ((entity_id, _), score) in entity_scores {
            // Try to get entity from all possible entity types
            // This is not optimal but works with current schema
            // In production, consider storing entity_type in interactions table

            // If entity_type_filter is specified, only check that type
            if let Some(filter_type) = entity_type_filter {
                if let Ok(Some(entity)) = self
                    .vector_store
                    .get_entity(ctx, &entity_id, filter_type)
                    .await
                {
                    recommendations.push(ScoredEntity {
                        entity_id: entity.entity_id,
                        entity_type: entity.entity_type,
                        score,
                        reason: Some(format!(
                            "Liked by {} similar users",
                            similar_users.len().min(10)
                        )),
                    });
                }
            } else {
                // Without entity_type filter, we need to try common types
                // This is a limitation of the current design
                // For now, we'll skip entities we can't resolve
                // In production, add entity_type to interactions or maintain an entity_id -> entity_type mapping
                warn!(
                    "Cannot resolve entity_type for entity_id={} without filter. Consider adding entity_type to interactions.",
                    entity_id
                );
            }
        }

        debug!(
            "Aggregated {} recommendations from neighbors",
            recommendations.len()
        );

        Ok(recommendations)
    }

    /// Check if a user is in cold start state (fewer than threshold interactions)
    /// Returns true if user has fewer than 5 interactions or no profile exists
    pub async fn is_cold_start_user(&self, ctx: &TenantContext, user_id: &str) -> Result<bool> {
        const COLD_START_THRESHOLD: i32 = 5;

        // Check if user profile exists
        let profile = self.vector_store.get_user_profile(ctx, user_id).await?;

        match profile {
            Some(p) => {
                let is_cold_start = p.interaction_count < COLD_START_THRESHOLD;
                debug!(
                    "User {} has {} interactions, cold_start={}",
                    user_id, p.interaction_count, is_cold_start
                );
                Ok(is_cold_start)
            }
            None => {
                debug!("User {} has no profile, cold_start=true", user_id);
                Ok(true)
            }
        }
    }

    /// Get trending/popular entities as fallback for cold start users
    /// Returns entities with highest interaction count in the last 7 days
    pub async fn get_trending_entities(
        &self,
        ctx: &TenantContext,
        entity_type: Option<&str>,
        count: usize,
    ) -> Result<Vec<ScoredEntity>> {
        debug!(
            "Getting trending entities: entity_type={:?}, count={}",
            entity_type, count
        );

        // Try to get from cache first
        let cache_key = format!("trending:{}:{}", entity_type.unwrap_or("all"), count);

        if let Ok(Some(cached)) = self.cache.get::<Vec<ScoredEntity>>(&cache_key).await {
            debug!("Returning cached trending entities");
            return Ok(cached);
        }

        // Calculate trending from database using VectorStore method
        let results = self
            .vector_store
            .get_trending_entity_stats(ctx, entity_type, count)
            .await?;

        let mut trending: Vec<ScoredEntity> = results
            .into_iter()
            .map(|(entity_id, entity_type, total_weight)| ScoredEntity {
                entity_id,
                entity_type,
                score: total_weight,
                reason: Some("Trending".to_string()),
            })
            .collect();

        // Normalize scores to [0, 1] range
        if let Some(max_score) = trending
            .iter()
            .map(|e| e.score)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            && max_score > 0.0
        {
            for entity in &mut trending {
                entity.score /= max_score;
            }
        }

        // Cache for 1 hour
        let _ = self
            .cache
            .set(&cache_key, &trending, std::time::Duration::from_secs(3600))
            .await;

        info!(
            "Calculated {} trending entities for entity_type={:?}",
            trending.len(),
            entity_type
        );

        Ok(trending)
    }

    /// Get recommendations with cold start handling
    /// If user is in cold start, returns trending entities and sets cold_start flag
    pub async fn get_recommendations_with_cold_start(
        &self,
        ctx: &TenantContext,
        user_id: &str,
        count: usize,
        entity_type: Option<&str>,
    ) -> Result<(Vec<ScoredEntity>, bool)> {
        debug!(
            "Getting recommendations with cold start handling for user={}",
            user_id
        );

        // Check if user is in cold start state
        let is_cold_start = self.is_cold_start_user(ctx, user_id).await?;

        if is_cold_start {
            info!(
                "User {} is in cold start state, returning trending entities",
                user_id
            );
            let trending = self.get_trending_entities(ctx, entity_type, count).await?;
            return Ok((trending, true));
        }

        // User has sufficient interactions, generate personalized recommendations
        let recommendations = self
            .generate_recommendations(ctx, user_id, count, entity_type)
            .await?;

        // If we couldn't generate enough recommendations, supplement with trending
        if recommendations.len() < count {
            warn!(
                "Only generated {} recommendations for user {}, supplementing with trending",
                recommendations.len(),
                user_id
            );

            let mut combined = recommendations;
            let needed = count - combined.len();
            let trending = self.get_trending_entities(ctx, entity_type, needed).await?;

            // Exclude entities already in recommendations
            let existing_ids: HashSet<String> =
                combined.iter().map(|e| e.entity_id.clone()).collect();

            for entity in trending {
                if !existing_ids.contains(&entity.entity_id) && combined.len() < count {
                    combined.push(entity);
                }
            }

            return Ok((combined, false));
        }

        Ok((recommendations, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical_vectors() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal_vectors() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite_vectors() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![-1.0, -2.0, -3.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_cosine_similarity_empty_vectors() {
        let vec1: Vec<f32> = vec![];
        let vec2: Vec<f32> = vec![];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_cosine_similarity_zero_magnitude() {
        let vec1 = vec![0.0, 0.0, 0.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert_eq!(similarity, 0.0);
    }

    #[test]
    fn test_cosine_similarity_normalized_vectors() {
        let vec1 = vec![0.6, 0.8];
        let vec2 = vec![0.8, 0.6];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 0.96).abs() < 0.01);
    }

    #[test]
    fn test_collaborative_config_default() {
        let config = CollaborativeConfig::default();
        assert_eq!(config.k_neighbors, 50);
        assert_eq!(config.min_similarity, 0.1);
        assert_eq!(config.default_count, 10);
    }

    #[test]
    fn test_collaborative_config_custom() {
        let config = CollaborativeConfig {
            k_neighbors: 100,
            min_similarity: 0.3,
            default_count: 20,
        };
        assert_eq!(config.k_neighbors, 100);
        assert_eq!(config.min_similarity, 0.3);
        assert_eq!(config.default_count, 20);
    }

    #[test]
    fn test_cosine_similarity_partial_overlap() {
        let vec1 = vec![1.0, 2.0, 3.0, 4.0];
        let vec2 = vec![2.0, 3.0, 4.0, 5.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        // These vectors should have high positive similarity
        assert!(similarity > 0.9);
        assert!(similarity <= 1.0);
    }

    #[test]
    fn test_cosine_similarity_small_values() {
        let vec1 = vec![0.001, 0.002, 0.003];
        let vec2 = vec![0.001, 0.002, 0.003];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_large_values() {
        let vec1 = vec![1000.0, 2000.0, 3000.0];
        let vec2 = vec![1000.0, 2000.0, 3000.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_mixed_signs() {
        let vec1 = vec![1.0, -2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        // Should be less than 1.0 due to opposite sign in second component
        assert!(similarity < 1.0);
        assert!(similarity > 0.0);
    }

    #[test]
    fn test_cosine_similarity_high_dimensional() {
        // Test with 512-dimensional vectors (typical embedding size)
        let vec1: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
        let vec2: Vec<f32> = (0..512).map(|i| (i as f32) / 512.0).collect();
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_symmetry() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];
        let sim1 = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        let sim2 = CollaborativeFilteringEngine::cosine_similarity(&vec2, &vec1);
        assert!((sim1 - sim2).abs() < 0.0001);
    }

    #[test]
    fn test_collaborative_config_clone() {
        let config1 = CollaborativeConfig {
            k_neighbors: 75,
            min_similarity: 0.2,
            default_count: 15,
        };
        let config2 = config1.clone();
        assert_eq!(config1.k_neighbors, config2.k_neighbors);
        assert_eq!(config1.min_similarity, config2.min_similarity);
        assert_eq!(config1.default_count, config2.default_count);
    }

    #[test]
    fn test_collaborative_config_debug() {
        let config = CollaborativeConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("k_neighbors"));
        assert!(debug_str.contains("min_similarity"));
        assert!(debug_str.contains("default_count"));
    }

    #[test]
    fn test_cosine_similarity_unit_vectors() {
        // Unit vectors in different directions
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let vec3 = vec![0.0, 0.0, 1.0];

        let sim12 = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        let sim13 = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec3);
        let sim23 = CollaborativeFilteringEngine::cosine_similarity(&vec2, &vec3);

        // All should be orthogonal (similarity = 0)
        assert!((sim12 - 0.0).abs() < 0.001);
        assert!((sim13 - 0.0).abs() < 0.001);
        assert!((sim23 - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_scaled_vectors() {
        // Scaling a vector should not change cosine similarity
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![2.0, 4.0, 6.0]; // vec1 * 2
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_negative_correlation() {
        let vec1 = vec![1.0, 2.0, 3.0, 4.0];
        let vec2 = vec![4.0, 3.0, 2.0, 1.0]; // Reversed
        let similarity = CollaborativeFilteringEngine::cosine_similarity(&vec1, &vec2);
        // Should have positive but lower similarity
        assert!(similarity > 0.0);
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_collaborative_config_boundary_values() {
        let config = CollaborativeConfig {
            k_neighbors: 1,
            min_similarity: 0.0,
            default_count: 1,
        };
        assert_eq!(config.k_neighbors, 1);
        assert_eq!(config.min_similarity, 0.0);
        assert_eq!(config.default_count, 1);

        let config2 = CollaborativeConfig {
            k_neighbors: 1000,
            min_similarity: 1.0,
            default_count: 100,
        };
        assert_eq!(config2.k_neighbors, 1000);
        assert_eq!(config2.min_similarity, 1.0);
        assert_eq!(config2.default_count, 100);
    }
}
