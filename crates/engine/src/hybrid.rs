use recommendation_models::{RecommendationError, Result, ScoredEntity, TenantContext};
use recommendation_storage::RedisCache;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use crate::collaborative::CollaborativeFilteringEngine;
use crate::content_based::ContentBasedFilteringEngine;

/// Configuration for hybrid recommendation engine
#[derive(Debug, Clone)]
pub struct HybridConfig {
    /// Weight for collaborative filtering (default: 0.6)
    pub collaborative_weight: f32,
    /// Weight for content-based filtering (default: 0.4)
    pub content_weight: f32,
    /// Whether to apply diversity filtering (default: true)
    pub enable_diversity: bool,
    /// Minimum number of categories for diversity (default: 3)
    pub min_categories: usize,
    /// Default number of recommendations to return
    pub default_count: usize,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            collaborative_weight: 0.6,
            content_weight: 0.4,
            enable_diversity: true,
            min_categories: 3,
            default_count: 10,
        }
    }
}

impl HybridConfig {
    /// Validate that weights sum to 1.0 (within tolerance)
    pub fn validate(&self) -> Result<()> {
        let sum = self.collaborative_weight + self.content_weight;
        const TOLERANCE: f32 = 0.001;

        if (sum - 1.0).abs() > TOLERANCE {
            return Err(RecommendationError::InvalidRequest(format!(
                "Hybrid weights must sum to 1.0, got {} (collaborative: {}, content: {})",
                sum, self.collaborative_weight, self.content_weight
            )));
        }

        if self.collaborative_weight < 0.0 || self.content_weight < 0.0 {
            return Err(RecommendationError::InvalidRequest(
                "Hybrid weights must be non-negative".to_string(),
            ));
        }

        Ok(())
    }
}

/// Hybrid recommendation engine combining collaborative and content-based filtering
pub struct HybridEngine {
    collaborative: Arc<CollaborativeFilteringEngine>,
    content_based: Arc<ContentBasedFilteringEngine>,
    cache: Arc<RedisCache>,
    config: HybridConfig,
    // Semaphore to limit concurrent recommendation generation (prevents CPU saturation)
    concurrency_limit: Arc<Semaphore>,
}

impl HybridEngine {
    /// Create a new hybrid recommendation engine
    pub fn new(
        collaborative: Arc<CollaborativeFilteringEngine>,
        content_based: Arc<ContentBasedFilteringEngine>,
        cache: Arc<RedisCache>,
        config: HybridConfig,
    ) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        info!(
            "Initializing HybridEngine with collaborative_weight={}, content_weight={}, diversity={}",
            config.collaborative_weight, config.content_weight, config.enable_diversity
        );

        // Limit concurrent recommendations to 100 (prevents CPU saturation at high scale)
        let concurrency_limit = Arc::new(Semaphore::new(100));

        Ok(Self {
            collaborative,
            content_based,
            cache,
            config,
            concurrency_limit,
        })
    }

    /// Generate hybrid recommendations by combining collaborative and content-based algorithms
    /// Executes both algorithms in parallel using tokio::join!
    pub async fn generate_recommendations(
        &self,
        ctx: &TenantContext,
        user_id: &str,
        entity_type: Option<&str>,
        count: usize,
    ) -> Result<Vec<ScoredEntity>> {
        // Acquire semaphore permit to limit concurrency
        let _permit = self
            .concurrency_limit
            .acquire()
            .await
            .map_err(|_e| RecommendationError::InternalError)?;

        debug!(
            "Generating hybrid recommendations for user={}, entity_type={:?}, count={}",
            user_id, entity_type, count
        );

        // Try to get from cache first
        let cache_key = format!(
            "hybrid_rec:{}:{}:{}:{}",
            ctx.tenant_id,
            user_id,
            entity_type.unwrap_or("all"),
            count
        );

        if let Ok(Some(cached)) = self.cache.get::<Vec<ScoredEntity>>(&cache_key).await {
            debug!("Returning cached hybrid recommendations");
            return Ok(cached);
        }

        // Execute both algorithms in parallel
        let (collab_result, content_result) = tokio::join!(
            self.collaborative.get_recommendations_with_cold_start(
                ctx,
                user_id,
                count * 2, // Request more to have options for combining
                entity_type
            ),
            self.content_based.generate_user_recommendations(
                ctx,
                user_id,
                entity_type.unwrap_or("product"), // Default to product if not specified
                count * 2
            )
        );

        // Handle results
        let (collab_recommendations, collab_cold_start) = collab_result?;
        let content_recommendations = content_result.unwrap_or_else(|e| {
            warn!(
                "Content-based recommendations failed: {}, using empty results",
                e
            );
            Vec::new()
        });

        debug!(
            "Got {} collaborative and {} content-based recommendations",
            collab_recommendations.len(),
            content_recommendations.len()
        );

        // If collaborative is in cold start and we have no content recommendations,
        // just return the collaborative results (trending)
        if collab_cold_start && content_recommendations.is_empty() {
            info!(
                "User {} is in cold start with no content recommendations, returning trending",
                user_id
            );
            let mut results = collab_recommendations;
            results.truncate(count);
            return Ok(results);
        }

        // Combine scores from both algorithms
        let combined = self.combine_scores(
            collab_recommendations,
            content_recommendations,
            self.config.collaborative_weight,
            self.config.content_weight,
        );

        // Apply diversity filtering if enabled
        let final_recommendations = if self.config.enable_diversity {
            self.apply_diversity_filter(combined, self.config.min_categories)
        } else {
            combined
        };

        // Sort by combined score descending and take top N
        let mut sorted_recommendations = final_recommendations;
        sorted_recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        sorted_recommendations.truncate(count);

        // Cache for 5 minutes
        let _ = self
            .cache
            .set(
                &cache_key,
                &sorted_recommendations,
                std::time::Duration::from_secs(300),
            )
            .await;

        info!(
            "Generated {} hybrid recommendations for user={}",
            sorted_recommendations.len(),
            user_id
        );

        Ok(sorted_recommendations)
    }

    /// Combine scores from collaborative and content-based algorithms
    /// Normalizes scores to [0,1] range and computes weighted average
    fn combine_scores(
        &self,
        collab_results: Vec<ScoredEntity>,
        content_results: Vec<ScoredEntity>,
        collab_weight: f32,
        content_weight: f32,
    ) -> Vec<ScoredEntity> {
        debug!(
            "Combining {} collaborative and {} content-based results with weights {}/{}",
            collab_results.len(),
            content_results.len(),
            collab_weight,
            content_weight
        );

        // Normalize collaborative scores to [0,1]
        let normalized_collab = Self::normalize_scores(collab_results);

        // Normalize content-based scores to [0,1]
        let normalized_content = Self::normalize_scores(content_results);

        // Create maps for efficient lookup
        let collab_map: HashMap<String, &ScoredEntity> = normalized_collab
            .iter()
            .map(|e| (e.entity_id.clone(), e))
            .collect();

        let content_map: HashMap<String, &ScoredEntity> = normalized_content
            .iter()
            .map(|e| (e.entity_id.clone(), e))
            .collect();

        // Collect all unique entity IDs
        let mut all_entity_ids: HashSet<String> = HashSet::new();
        all_entity_ids.extend(collab_map.keys().cloned());
        all_entity_ids.extend(content_map.keys().cloned());

        // Compute weighted average for each entity
        let mut combined: Vec<ScoredEntity> = all_entity_ids
            .into_iter()
            .map(|entity_id| {
                let collab_score = collab_map.get(&entity_id).map(|e| e.score).unwrap_or(0.0);

                let content_score = content_map.get(&entity_id).map(|e| e.score).unwrap_or(0.0);

                // Weighted average
                let combined_score =
                    (collab_score * collab_weight) + (content_score * content_weight);

                // Get entity_type from either source
                let entity_type = collab_map
                    .get(&entity_id)
                    .or_else(|| content_map.get(&entity_id))
                    .map(|e| e.entity_type.clone())
                    .unwrap_or_else(|| "unknown".to_string());

                // Create reason explaining the combination
                let reason = match (
                    collab_map.contains_key(&entity_id),
                    content_map.contains_key(&entity_id),
                ) {
                    (true, true) => Some(format!(
                        "Hybrid: {:.0}% collaborative, {:.0}% content similarity",
                        collab_weight * 100.0,
                        content_weight * 100.0
                    )),
                    (true, false) => Some("Based on similar users' preferences".to_string()),
                    (false, true) => Some("Based on content similarity".to_string()),
                    (false, false) => None,
                };

                ScoredEntity {
                    entity_id,
                    entity_type,
                    score: combined_score,
                    reason,
                }
            })
            .collect();

        // Sort by score descending
        combined.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        debug!("Combined into {} unique recommendations", combined.len());

        combined
    }

    /// Normalize scores to [0,1] range using min-max normalization
    fn normalize_scores(entities: Vec<ScoredEntity>) -> Vec<ScoredEntity> {
        if entities.is_empty() {
            return entities;
        }

        // Find min and max scores
        let min_score = entities
            .iter()
            .map(|e| e.score)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let max_score = entities
            .iter()
            .map(|e| e.score)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        // If all scores are the same, return as-is with score 1.0
        if (max_score - min_score).abs() < 0.0001 {
            return entities
                .into_iter()
                .map(|mut e| {
                    e.score = 1.0;
                    e
                })
                .collect();
        }

        // Normalize each score to [0,1]
        entities
            .into_iter()
            .map(|mut e| {
                e.score = (e.score - min_score) / (max_score - min_score);
                e
            })
            .collect()
    }

    /// Apply diversity filtering to ensure recommendations span multiple categories
    /// Uses a greedy algorithm to select diverse recommendations
    fn apply_diversity_filter(
        &self,
        mut recommendations: Vec<ScoredEntity>,
        min_categories: usize,
    ) -> Vec<ScoredEntity> {
        if recommendations.is_empty() || min_categories == 0 {
            return recommendations;
        }

        debug!(
            "Applying diversity filter to {} recommendations (min_categories={})",
            recommendations.len(),
            min_categories
        );

        // Track entity types we've seen
        let mut seen_types: HashMap<String, usize> = HashMap::new();
        let mut diverse_results: Vec<ScoredEntity> = Vec::new();

        // Sort by score descending (already done in combine_scores, but ensure it)
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // First pass: ensure we have at least one from each category (up to min_categories)
        let mut remaining: Vec<ScoredEntity> = Vec::new();

        for entity in recommendations {
            let type_count = seen_types.get(&entity.entity_type).copied().unwrap_or(0);

            // If we haven't reached min_categories yet and this is a new type, prioritize it
            if seen_types.len() < min_categories && type_count == 0 {
                seen_types.insert(entity.entity_type.clone(), 1);
                diverse_results.push(entity);
            } else {
                remaining.push(entity);
            }
        }

        // Second pass: add remaining items, balancing across categories
        for entity in remaining {
            let type_count = seen_types.get(&entity.entity_type).copied().unwrap_or(0);

            // Calculate average count across all types
            let avg_count = if seen_types.is_empty() {
                0.0
            } else {
                seen_types.values().sum::<usize>() as f32 / seen_types.len() as f32
            };

            // Prefer entities from underrepresented categories
            // Allow if this category is below or at average
            if type_count as f32 <= avg_count + 1.0 {
                seen_types
                    .entry(entity.entity_type.clone())
                    .and_modify(|c| *c += 1)
                    .or_insert(1);
                diverse_results.push(entity);
            }
        }

        debug!(
            "Diversity filter resulted in {} recommendations across {} categories",
            diverse_results.len(),
            seen_types.len()
        );

        diverse_results
    }

    /// Generate recommendations for a specific entity (similar items)
    /// Uses content-based filtering with hybrid scoring
    pub async fn generate_entity_recommendations(
        &self,
        ctx: &TenantContext,
        entity_id: &str,
        entity_type: &str,
        count: usize,
    ) -> Result<Vec<ScoredEntity>> {
        debug!(
            "Generating hybrid entity recommendations for entity={}, entity_type={}, count={}",
            entity_id, entity_type, count
        );

        // For entity-based recommendations, primarily use content-based
        // but could be extended to include collaborative signals
        let recommendations = self
            .content_based
            .generate_recommendations(ctx, entity_id, entity_type, count)
            .await?;

        info!(
            "Generated {} hybrid entity recommendations for entity={}",
            recommendations.len(),
            entity_id
        );

        Ok(recommendations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_config_default() {
        let config = HybridConfig::default();
        assert_eq!(config.collaborative_weight, 0.6);
        assert_eq!(config.content_weight, 0.4);
        assert!(config.enable_diversity);
        assert_eq!(config.min_categories, 3);
        assert_eq!(config.default_count, 10);
    }

    #[test]
    fn test_hybrid_config_validate_valid() {
        let config = HybridConfig {
            collaborative_weight: 0.7,
            content_weight: 0.3,
            enable_diversity: true,
            min_categories: 2,
            default_count: 15,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_hybrid_config_validate_invalid_sum() {
        let config = HybridConfig {
            collaborative_weight: 0.7,
            content_weight: 0.4, // Sum is 1.1, not 1.0
            enable_diversity: true,
            min_categories: 2,
            default_count: 15,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_hybrid_config_validate_negative_weight() {
        let config = HybridConfig {
            collaborative_weight: 1.2,
            content_weight: -0.2, // Negative weight
            enable_diversity: true,
            min_categories: 2,
            default_count: 15,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_hybrid_config_validate_exact_one() {
        let config = HybridConfig {
            collaborative_weight: 0.5,
            content_weight: 0.5,
            enable_diversity: true,
            min_categories: 2,
            default_count: 15,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_normalize_scores_empty() {
        let entities: Vec<ScoredEntity> = vec![];
        let normalized = HybridEngine::normalize_scores(entities);
        assert!(normalized.is_empty());
    }

    #[test]
    fn test_normalize_scores_single() {
        let entities = vec![ScoredEntity {
            entity_id: "e1".to_string(),
            entity_type: "product".to_string(),
            score: 5.0,
            reason: None,
        }];
        let normalized = HybridEngine::normalize_scores(entities);
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0].score, 1.0);
    }

    #[test]
    fn test_normalize_scores_range() {
        let entities = vec![
            ScoredEntity {
                entity_id: "e1".to_string(),
                entity_type: "product".to_string(),
                score: 10.0,
                reason: None,
            },
            ScoredEntity {
                entity_id: "e2".to_string(),
                entity_type: "product".to_string(),
                score: 5.0,
                reason: None,
            },
            ScoredEntity {
                entity_id: "e3".to_string(),
                entity_type: "product".to_string(),
                score: 0.0,
                reason: None,
            },
        ];
        let normalized = HybridEngine::normalize_scores(entities);
        assert_eq!(normalized.len(), 3);
        assert_eq!(normalized[0].score, 1.0); // max
        assert_eq!(normalized[1].score, 0.5); // middle
        assert_eq!(normalized[2].score, 0.0); // min
    }

    #[test]
    fn test_normalize_scores_same_values() {
        let entities = vec![
            ScoredEntity {
                entity_id: "e1".to_string(),
                entity_type: "product".to_string(),
                score: 7.0,
                reason: None,
            },
            ScoredEntity {
                entity_id: "e2".to_string(),
                entity_type: "product".to_string(),
                score: 7.0,
                reason: None,
            },
        ];
        let normalized = HybridEngine::normalize_scores(entities);
        assert_eq!(normalized.len(), 2);
        // All same values should normalize to 1.0
        assert_eq!(normalized[0].score, 1.0);
        assert_eq!(normalized[1].score, 1.0);
    }

    #[test]
    fn test_hybrid_config_clone() {
        let config1 = HybridConfig {
            collaborative_weight: 0.8,
            content_weight: 0.2,
            enable_diversity: false,
            min_categories: 5,
            default_count: 25,
        };
        let config2 = config1.clone();
        assert_eq!(config1.collaborative_weight, config2.collaborative_weight);
        assert_eq!(config1.content_weight, config2.content_weight);
        assert_eq!(config1.enable_diversity, config2.enable_diversity);
    }

    #[test]
    fn test_hybrid_config_debug() {
        let config = HybridConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("collaborative_weight"));
        assert!(debug_str.contains("content_weight"));
        assert!(debug_str.contains("enable_diversity"));
    }

    #[test]
    fn test_normalize_scores_preserves_order() {
        let entities = vec![
            ScoredEntity {
                entity_id: "e1".to_string(),
                entity_type: "product".to_string(),
                score: 100.0,
                reason: None,
            },
            ScoredEntity {
                entity_id: "e2".to_string(),
                entity_type: "product".to_string(),
                score: 50.0,
                reason: None,
            },
            ScoredEntity {
                entity_id: "e3".to_string(),
                entity_type: "product".to_string(),
                score: 25.0,
                reason: None,
            },
        ];
        let normalized = HybridEngine::normalize_scores(entities);

        // Check that relative order is preserved
        assert!(normalized[0].score > normalized[1].score);
        assert!(normalized[1].score > normalized[2].score);
    }

    #[test]
    fn test_hybrid_config_validate_tolerance() {
        // Test that small floating point errors are tolerated
        let config = HybridConfig {
            collaborative_weight: 0.6,
            content_weight: 0.4000001, // Slightly over 1.0 due to floating point
            enable_diversity: true,
            min_categories: 2,
            default_count: 10,
        };
        // Should still be valid within tolerance
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_normalize_scores_negative_values() {
        let entities = vec![
            ScoredEntity {
                entity_id: "e1".to_string(),
                entity_type: "product".to_string(),
                score: 5.0,
                reason: None,
            },
            ScoredEntity {
                entity_id: "e2".to_string(),
                entity_type: "product".to_string(),
                score: -5.0,
                reason: None,
            },
        ];
        let normalized = HybridEngine::normalize_scores(entities);
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].score, 1.0); // max
        assert_eq!(normalized[1].score, 0.0); // min
    }
}
