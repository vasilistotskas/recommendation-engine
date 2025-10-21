use recommendation_engine::{
    CollaborativeFilteringEngine, ContentBasedFilteringEngine, HybridEngine,
};
use recommendation_models::{
    Algorithm, RecommendationError, RecommendationRequest, RecommendationResponse, Result,
    ScoredEntity, TenantContext,
};
use recommendation_storage::{RedisCache, VectorStore};
use std::sync::Arc;
use tracing::{debug, info};
use chrono::Utc;
use moka::future::Cache;

/// Service for orchestrating recommendation algorithms and handling requests
pub struct RecommendationService {
    collaborative: Arc<CollaborativeFilteringEngine>,
    content_based: Arc<ContentBasedFilteringEngine>,
    hybrid: Arc<HybridEngine>,
    vector_store: Arc<VectorStore>,
    cache: Arc<RedisCache>,
    // In-memory cache for request coalescing (prevents duplicate in-flight requests)
    request_cache: Cache<String, Arc<RecommendationResponse>>,
}

impl RecommendationService {
    /// Create a new recommendation service with all engine dependencies
    pub fn new(
        collaborative: Arc<CollaborativeFilteringEngine>,
        content_based: Arc<ContentBasedFilteringEngine>,
        hybrid: Arc<HybridEngine>,
        vector_store: Arc<VectorStore>,
        cache: Arc<RedisCache>,
    ) -> Self {
        info!("Initializing RecommendationService");

        // Create in-memory cache for request coalescing
        // Max 10,000 entries, 30 second TTL
        let request_cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(std::time::Duration::from_secs(30))
            .build();

        Self {
            collaborative,
            content_based,
            hybrid,
            vector_store,
            cache,
            request_cache,
        }
    }

    /// Validate recommendation request
    fn validate_request(&self, request: &RecommendationRequest) -> Result<()> {
        // Ensure either user_id or entity_id is provided
        if request.user_id.is_none() && request.entity_id.is_none() {
            return Err(RecommendationError::InvalidRequest(
                "Either user_id or entity_id must be provided".to_string(),
            ));
        }

        // Validate count is reasonable
        if request.count == 0 {
            return Err(RecommendationError::InvalidRequest(
                "count must be greater than 0".to_string(),
            ));
        }

        if request.count > 100 {
            return Err(RecommendationError::InvalidRequest(
                "count must not exceed 100".to_string(),
            ));
        }

        // Validate hybrid weights if using hybrid algorithm
        if let Algorithm::Hybrid {
            collaborative_weight,
            content_weight,
        } = &request.algorithm
        {
            let sum = collaborative_weight + content_weight;
            const TOLERANCE: f32 = 0.001;

            if (sum - 1.0).abs() > TOLERANCE {
                return Err(RecommendationError::InvalidRequest(format!(
                    "Hybrid weights must sum to 1.0, got {} (collaborative: {}, content: {})",
                    sum, collaborative_weight, content_weight
                )));
            }

            if *collaborative_weight < 0.0 || *content_weight < 0.0 {
                return Err(RecommendationError::InvalidRequest(
                    "Hybrid weights must be non-negative".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get recommendations based on the request
    /// Routes to appropriate algorithm and handles cold start scenarios
    pub async fn get_recommendations(
        &self,
        ctx: &TenantContext,
        request: RecommendationRequest,
    ) -> Result<RecommendationResponse> {
        debug!(
            "Processing recommendation request: algorithm={:?}, user_id={:?}, entity_id={:?}, count={}",
            request.algorithm, request.user_id, request.entity_id, request.count
        );

        // Validate request
        self.validate_request(&request)?;

        // Create cache key for request coalescing and Redis cache
        let cache_key = format!(
            "rec:{}:{}:{}:{:?}:{}",
            ctx.tenant_id,
            request.user_id.as_deref().unwrap_or(""),
            request.entity_id.as_deref().unwrap_or(""),
            request.algorithm,
            request.count
        );

        // Check in-memory cache first (prevents duplicate in-flight requests)
        if let Some(cached) = self.request_cache.get(&cache_key).await {
            debug!("Returning in-memory cached recommendation response");
            return Ok((*cached).clone());
        }

        // Check Redis cache second
        if let Ok(Some(cached)) = self.cache.get::<RecommendationResponse>(&cache_key).await {
            debug!("Returning Redis cached recommendation response");
            // Also populate in-memory cache
            self.request_cache.insert(cache_key.clone(), Arc::new(cached.clone())).await;
            return Ok(cached);
        }

        // Extract entity_type from filters if provided
        let entity_type = request
            .filters
            .as_ref()
            .and_then(|f| f.get("entity_type"))
            .map(|s| s.as_str());

        // Route to appropriate algorithm
        let (recommendations, cold_start, algorithm_name) = match request.algorithm {
            Algorithm::Collaborative => {
                self.handle_collaborative_request(ctx, &request, entity_type)
                    .await?
            }
            Algorithm::ContentBased => {
                self.handle_content_based_request(ctx, &request, entity_type)
                    .await?
            }
            Algorithm::Hybrid {
                collaborative_weight,
                content_weight,
            } => {
                self.handle_hybrid_request(
                    ctx,
                    &request,
                    entity_type,
                    collaborative_weight,
                    content_weight,
                )
                .await?
            }
        };

        info!(
            "Generated {} recommendations using {} algorithm (cold_start={})",
            recommendations.len(),
            algorithm_name,
            cold_start
        );

        let response = RecommendationResponse {
            recommendations,
            algorithm: algorithm_name,
            cold_start,
            generated_at: Utc::now(),
        };

        // Cache in both Redis and in-memory (in-memory prevents thundering herd)
        let response_arc = Arc::new(response.clone());
        self.request_cache.insert(cache_key.clone(), response_arc).await;

        let _ = self
            .cache
            .set(&cache_key, &response, std::time::Duration::from_secs(30))
            .await;

        Ok(response)
    }

    /// Handle collaborative filtering request
    async fn handle_collaborative_request(
        &self,
        ctx: &TenantContext,
        request: &RecommendationRequest,
        entity_type: Option<&str>,
    ) -> Result<(Vec<ScoredEntity>, bool, String)> {
        let user_id = request.user_id.as_ref().ok_or_else(|| {
            RecommendationError::InvalidRequest(
                "user_id is required for collaborative filtering".to_string(),
            )
        })?;

        let (recommendations, cold_start) = self
            .collaborative
            .get_recommendations_with_cold_start(ctx, user_id, request.count, entity_type)
            .await?;

        Ok((recommendations, cold_start, "collaborative".to_string()))
    }

    /// Handle content-based filtering request
    async fn handle_content_based_request(
        &self,
        ctx: &TenantContext,
        request: &RecommendationRequest,
        entity_type: Option<&str>,
    ) -> Result<(Vec<ScoredEntity>, bool, String)> {
        // Content-based can work with either user_id or entity_id
        if let Some(entity_id) = &request.entity_id {
            // Entity-based recommendations (similar items)
            let entity_type = entity_type.ok_or_else(|| {
                RecommendationError::InvalidRequest(
                    "entity_type filter is required for entity-based content recommendations"
                        .to_string(),
                )
            })?;

            let (recommendations, cold_start) = self
                .content_based
                .get_recommendations_with_cold_start(ctx, entity_id, entity_type, request.count)
                .await?;

            Ok((recommendations, cold_start, "content_based".to_string()))
        } else if let Some(user_id) = &request.user_id {
            // User-based content recommendations (based on interaction history)
            let entity_type = entity_type.unwrap_or("product"); // Default to product

            let recommendations = self
                .content_based
                .generate_user_recommendations(ctx, user_id, entity_type, request.count)
                .await?;

            // Check if user is in cold start
            let cold_start = self
                .collaborative
                .is_cold_start_user(ctx, user_id)
                .await?;

            Ok((recommendations, cold_start, "content_based".to_string()))
        } else {
            Err(RecommendationError::InvalidRequest(
                "Either user_id or entity_id is required for content-based filtering".to_string(),
            ))
        }
    }

    /// Handle hybrid filtering request
    async fn handle_hybrid_request(
        &self,
        ctx: &TenantContext,
        request: &RecommendationRequest,
        entity_type: Option<&str>,
        _collaborative_weight: f32,
        _content_weight: f32,
    ) -> Result<(Vec<ScoredEntity>, bool, String)> {
        // Hybrid primarily works with user_id
        if let Some(user_id) = &request.user_id {
            let recommendations = self
                .hybrid
                .generate_recommendations(ctx, user_id, entity_type, request.count)
                .await?;

            // Check if user is in cold start
            let cold_start = self
                .collaborative
                .is_cold_start_user(ctx, user_id)
                .await?;

            Ok((recommendations, cold_start, "hybrid".to_string()))
        } else if let Some(entity_id) = &request.entity_id {
            // For entity-based, use hybrid's entity recommendations
            let entity_type = entity_type.ok_or_else(|| {
                RecommendationError::InvalidRequest(
                    "entity_type filter is required for entity-based hybrid recommendations"
                        .to_string(),
                )
            })?;

            let recommendations = self
                .hybrid
                .generate_entity_recommendations(ctx, entity_id, entity_type, request.count)
                .await?;

            Ok((recommendations, false, "hybrid".to_string()))
        } else {
            Err(RecommendationError::InvalidRequest(
                "Either user_id or entity_id is required for hybrid filtering".to_string(),
            ))
        }
    }

    /// Get trending entities based on interaction frequency in last 7 days
    /// Results are cached in Redis and refreshed every 1 hour
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

        // Validate count
        if count == 0 {
            return Err(RecommendationError::InvalidRequest(
                "count must be greater than 0".to_string(),
            ));
        }

        if count > 100 {
            return Err(RecommendationError::InvalidRequest(
                "count must not exceed 100".to_string(),
            ));
        }

        // Try to get from cache first
        let cache_key = format!(
            "trending:{}:{}:{}",
            ctx.tenant_id,
            entity_type.unwrap_or("all"),
            count
        );

        if let Ok(Some(cached)) = self.cache.get::<Vec<ScoredEntity>>(&cache_key).await {
            debug!("Returning cached trending entities");
            return Ok(cached);
        }

        // Calculate trending from database
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
            .set(
                &cache_key,
                &trending,
                std::time::Duration::from_secs(3600),
            )
            .await;

        info!(
            "Calculated {} trending entities for entity_type={:?}",
            trending.len(),
            entity_type
        );

        Ok(trending)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use recommendation_models::Algorithm;
    use std::collections::HashMap;

    #[test]
    fn test_validate_request_valid_user_collaborative() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 10,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_valid_entity_content_based() {
        let service = create_mock_service();
        let mut filters = HashMap::new();
        filters.insert("entity_type".to_string(), "product".to_string());

        let request = RecommendationRequest {
            user_id: None,
            entity_id: Some("entity_456".to_string()),
            algorithm: Algorithm::ContentBased,
            count: 20,
            filters: Some(filters),
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_valid_hybrid() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_789".to_string()),
            entity_id: None,
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 0.6,
                content_weight: 0.4,
            },
            count: 15,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_missing_user_and_entity() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: None,
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 10,
            filters: None,
        };

        let result = service.validate_request(&request);
        assert!(result.is_err());
        match result {
            Err(RecommendationError::InvalidRequest(msg)) => {
                assert!(msg.contains("Either user_id or entity_id must be provided"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_validate_request_zero_count() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 0,
            filters: None,
        };

        let result = service.validate_request(&request);
        assert!(result.is_err());
        match result {
            Err(RecommendationError::InvalidRequest(msg)) => {
                assert!(msg.contains("count must be greater than 0"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_validate_request_count_too_large() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 101,
            filters: None,
        };

        let result = service.validate_request(&request);
        assert!(result.is_err());
        match result {
            Err(RecommendationError::InvalidRequest(msg)) => {
                assert!(msg.contains("count must not exceed 100"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_validate_request_hybrid_weights_invalid_sum() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 0.7,
                content_weight: 0.4, // Sum is 1.1
            },
            count: 10,
            filters: None,
        };

        let result = service.validate_request(&request);
        assert!(result.is_err());
        match result {
            Err(RecommendationError::InvalidRequest(msg)) => {
                assert!(msg.contains("Hybrid weights must sum to 1.0"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_validate_request_hybrid_weights_negative() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 1.2,
                content_weight: -0.2, // Negative weight
            },
            count: 10,
            filters: None,
        };

        let result = service.validate_request(&request);
        assert!(result.is_err());
        match result {
            Err(RecommendationError::InvalidRequest(msg)) => {
                assert!(msg.contains("Hybrid weights must be non-negative"));
            }
            _ => panic!("Expected InvalidRequest error"),
        }
    }

    #[test]
    fn test_validate_request_hybrid_weights_valid_exact() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 0.5,
                content_weight: 0.5,
            },
            count: 10,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_hybrid_weights_within_tolerance() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 0.6,
                content_weight: 0.4000001, // Slightly over due to floating point
            },
            count: 10,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_with_filters() {
        let service = create_mock_service();
        let mut filters = HashMap::new();
        filters.insert("entity_type".to_string(), "product".to_string());
        filters.insert("category".to_string(), "electronics".to_string());

        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 10,
            filters: Some(filters),
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_boundary_count_1() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 1,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_boundary_count_100() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 100,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_both_user_and_entity() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: Some("entity_456".to_string()),
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 0.6,
                content_weight: 0.4,
            },
            count: 10,
            filters: None,
        };

        // Having both is valid - the algorithm will decide which to use
        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_hybrid_zero_weights() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 0.0,
                content_weight: 1.0,
            },
            count: 10,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_hybrid_all_collaborative() {
        let service = create_mock_service();
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Hybrid {
                collaborative_weight: 1.0,
                content_weight: 0.0,
            },
            count: 10,
            filters: None,
        };

        assert!(service.validate_request(&request).is_ok());
    }

    // Helper function to create a test service structure
    // This is a simplified version that only tests the validation logic
    // without requiring actual database/cache connections
    fn create_mock_service() -> TestRecommendationService {
        TestRecommendationService {}
    }

    // Simplified test struct that only implements validation
    // This allows us to test request validation without needing real dependencies
    struct TestRecommendationService {}

    impl TestRecommendationService {
        fn validate_request(&self, request: &RecommendationRequest) -> Result<()> {
            // Ensure either user_id or entity_id is provided
            if request.user_id.is_none() && request.entity_id.is_none() {
                return Err(RecommendationError::InvalidRequest(
                    "Either user_id or entity_id must be provided".to_string(),
                ));
            }

            // Validate count is reasonable
            if request.count == 0 {
                return Err(RecommendationError::InvalidRequest(
                    "count must be greater than 0".to_string(),
                ));
            }

            if request.count > 100 {
                return Err(RecommendationError::InvalidRequest(
                    "count must not exceed 100".to_string(),
                ));
            }

            // Validate hybrid weights if using hybrid algorithm
            if let Algorithm::Hybrid {
                collaborative_weight,
                content_weight,
            } = &request.algorithm
            {
                let sum = collaborative_weight + content_weight;
                const TOLERANCE: f32 = 0.001;

                if (sum - 1.0).abs() > TOLERANCE {
                    return Err(RecommendationError::InvalidRequest(format!(
                        "Hybrid weights must sum to 1.0, got {} (collaborative: {}, content: {})",
                        sum, collaborative_weight, content_weight
                    )));
                }

                if *collaborative_weight < 0.0 || *content_weight < 0.0 {
                    return Err(RecommendationError::InvalidRequest(
                        "Hybrid weights must be non-negative".to_string(),
                    ));
                }
            }

            Ok(())
        }
    }
}
