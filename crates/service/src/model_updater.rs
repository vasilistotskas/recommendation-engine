use crate::webhook::{WebhookDelivery, WebhookEvent};
use recommendation_models::{Result, TenantContext};
use recommendation_storage::{RedisCache, VectorStore};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};

/// Background task scheduler for model updates and maintenance
pub struct ModelUpdater {
    vector_store: Arc<VectorStore>,
    cache: Arc<RedisCache>,
    webhook_delivery: Option<Arc<WebhookDelivery>>,
    incremental_interval: Duration,
    full_rebuild_interval: Duration,
    trending_interval: Duration,
}

impl ModelUpdater {
    /// Create a new ModelUpdater with specified update intervals
    pub fn new(
        vector_store: Arc<VectorStore>,
        cache: Arc<RedisCache>,
        webhook_delivery: Option<Arc<WebhookDelivery>>,
        incremental_interval_secs: u64,
        full_rebuild_interval_hours: u64,
        trending_interval_hours: u64,
    ) -> Self {
        info!(
            "Initializing ModelUpdater with intervals: incremental={}s, full_rebuild={}h, trending={}h, webhooks={}",
            incremental_interval_secs,
            full_rebuild_interval_hours,
            trending_interval_hours,
            webhook_delivery.is_some()
        );

        Self {
            vector_store,
            cache,
            webhook_delivery,
            incremental_interval: Duration::from_secs(incremental_interval_secs),
            full_rebuild_interval: Duration::from_secs(full_rebuild_interval_hours * 3600),
            trending_interval: Duration::from_secs(trending_interval_hours * 3600),
        }
    }

    /// Create a ModelUpdater with default intervals
    /// - Incremental updates: every 10 seconds
    /// - Full rebuild: every 24 hours
    /// - Trending calculation: every 1 hour
    pub fn with_defaults(
        vector_store: Arc<VectorStore>,
        cache: Arc<RedisCache>,
        webhook_delivery: Option<Arc<WebhookDelivery>>,
    ) -> Self {
        Self::new(vector_store, cache, webhook_delivery, 10, 24, 1)
    }
}

/// Task scheduler for managing background update tasks
pub struct TaskScheduler {
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    /// Spawn a new background task
    pub fn spawn<F>(&mut self, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(task);
        self.handles.push(handle);
    }

    /// Wait for all tasks to complete (typically used for graceful shutdown)
    pub async fn join_all(self) {
        for handle in self.handles {
            if let Err(e) = handle.await {
                error!("Background task panicked: {:?}", e);
            }
        }
    }

    /// Abort all running tasks
    pub fn abort_all(&self) {
        for handle in &self.handles {
            handle.abort();
        }
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelUpdater {
    /// Run incremental updates for user similarity matrices and entity indices
    /// Updates user similarity matrices every 10 seconds
    /// Updates entity similarity indices every 5 seconds
    /// Processes updates without blocking requests
    pub async fn incremental_update(&self, ctx: &TenantContext) -> Result<()> {
        debug!("Starting incremental update for tenant: {}", ctx.tenant_id);
        let start_time = std::time::Instant::now();

        let mut users_updated = 0;
        let mut entities_updated = 0;

        // Update user preference vectors for users with recent interactions
        match self.update_user_preference_vectors(ctx).await {
            Ok(count) => {
                debug!("Updated {} user preference vectors", count);
                users_updated = count;
            }
            Err(e) => {
                error!("Failed to update user preference vectors: {:?}", e);
            }
        }

        // Update entity feature vectors for recently modified entities
        match self.update_entity_feature_vectors(ctx).await {
            Ok(count) => {
                debug!("Updated {} entity feature vectors", count);
                entities_updated = count;
            }
            Err(e) => {
                error!("Failed to update entity feature vectors: {:?}", e);
            }
        }

        // Invalidate affected cache entries
        match self.invalidate_affected_cache(ctx).await {
            Ok(count) => {
                debug!("Invalidated {} cache entries", count);
            }
            Err(e) => {
                warn!("Failed to invalidate cache: {:?}", e);
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        info!("Completed incremental update for tenant: {}", ctx.tenant_id);

        // Trigger model_updated webhook if updates occurred
        if (users_updated > 0 || entities_updated > 0) && self.webhook_delivery.is_some() {
            let event = WebhookEvent::model_updated(
                ctx.tenant_id.clone(),
                users_updated,
                entities_updated,
                duration_ms,
            );

            if let Some(webhook) = &self.webhook_delivery {
                webhook.clone().dispatch_async(event);
            }
        }

        Ok(())
    }

    /// Update user preference vectors for users with recent interactions
    async fn update_user_preference_vectors(&self, ctx: &TenantContext) -> Result<usize> {
        // Get users with interactions in the last update interval
        let users_to_update = self
            .vector_store
            .get_users_with_recent_interactions(ctx, self.incremental_interval)
            .await?;

        let mut updated_count = 0;

        for user_id in users_to_update {
            match self
                .vector_store
                .recompute_user_preference_vector(ctx, &user_id)
                .await
            {
                Ok(_) => {
                    updated_count += 1;
                    // Invalidate user's recommendation cache
                    let cache_key = format!("rec:{}:*", user_id);
                    let _ = self.cache.delete_pattern(&cache_key).await;
                }
                Err(e) => {
                    warn!(
                        "Failed to update preference vector for user {}: {:?}",
                        user_id, e
                    );
                }
            }
        }

        Ok(updated_count)
    }

    /// Update entity feature vectors for recently modified entities
    async fn update_entity_feature_vectors(&self, ctx: &TenantContext) -> Result<usize> {
        // Get entities modified in the last update interval
        let entities_to_update = self
            .vector_store
            .get_recently_modified_entities(ctx, self.incremental_interval)
            .await?;

        let mut updated_count = 0;

        for entity_id in entities_to_update {
            match self
                .vector_store
                .recompute_entity_feature_vector(ctx, &entity_id)
                .await
            {
                Ok(_) => {
                    updated_count += 1;
                }
                Err(e) => {
                    warn!(
                        "Failed to update feature vector for entity {}: {:?}",
                        entity_id, e
                    );
                }
            }
        }

        Ok(updated_count)
    }

    /// Invalidate cache entries affected by recent updates
    async fn invalidate_affected_cache(&self, ctx: &TenantContext) -> Result<usize> {
        // Invalidate trending cache for this tenant
        let trending_pattern = format!("trending:{}:*", ctx.tenant_id);
        let count = self
            .cache
            .delete_pattern(&trending_pattern)
            .await
            .map_err(|e| recommendation_models::RecommendationError::CacheError(e.to_string()))?;
        Ok(count)
    }
}

impl ModelUpdater {
    /// Start incremental update background task
    /// Runs every configured interval (default: 10 seconds)
    pub fn start_incremental_updates(
        self: Arc<Self>,
        ctx: TenantContext,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval_timer = interval(self.incremental_interval);
            interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            info!(
                "Starting incremental update task for tenant: {} (interval: {:?})",
                ctx.tenant_id, self.incremental_interval
            );

            loop {
                interval_timer.tick().await;

                match self.incremental_update(&ctx).await {
                    Ok(_) => {
                        debug!("Incremental update completed successfully");
                    }
                    Err(e) => {
                        error!("Incremental update failed: {:?}", e);
                    }
                }
            }
        })
    }
}

impl ModelUpdater {
    /// Rebuild complete similarity matrices every 24 hours
    /// Scheduled during low-traffic periods
    /// Logs completion with metrics
    pub async fn full_rebuild(&self, ctx: &TenantContext) -> Result<()> {
        info!("Starting full rebuild for tenant: {}", ctx.tenant_id);
        let start_time = std::time::Instant::now();

        // Rebuild all user preference vectors
        let users_updated = match self.rebuild_all_user_vectors(ctx).await {
            Ok(count) => {
                info!("Rebuilt {} user preference vectors", count);
                count
            }
            Err(e) => {
                error!("Failed to rebuild user vectors: {:?}", e);
                0
            }
        };

        // Rebuild all entity feature vectors
        let entities_updated = match self.rebuild_all_entity_vectors(ctx).await {
            Ok(count) => {
                info!("Rebuilt {} entity feature vectors", count);
                count
            }
            Err(e) => {
                error!("Failed to rebuild entity vectors: {:?}", e);
                0
            }
        };

        // Rebuild HNSW indices for optimal performance
        match self.rebuild_vector_indices(ctx).await {
            Ok(_) => {
                info!("Successfully rebuilt vector indices");
            }
            Err(e) => {
                error!("Failed to rebuild vector indices: {:?}", e);
            }
        }

        // Clear all caches to force fresh computations
        match self.clear_all_caches(ctx).await {
            Ok(count) => {
                info!("Cleared {} cache entries", count);
            }
            Err(e) => {
                warn!("Failed to clear caches: {:?}", e);
            }
        }

        let duration = start_time.elapsed();
        let duration_ms = duration.as_millis() as u64;

        info!(
            "Completed full rebuild for tenant: {} in {:?} (users: {}, entities: {})",
            ctx.tenant_id, duration, users_updated, entities_updated
        );

        // Trigger model_updated webhook
        if self.webhook_delivery.is_some() {
            let event = WebhookEvent::model_updated(
                ctx.tenant_id.clone(),
                users_updated,
                entities_updated,
                duration_ms,
            );

            if let Some(webhook) = &self.webhook_delivery {
                webhook.clone().dispatch_async(event);
            }
        }

        Ok(())
    }

    /// Rebuild all user preference vectors from scratch
    async fn rebuild_all_user_vectors(&self, ctx: &TenantContext) -> Result<usize> {
        let all_users = self.vector_store.get_all_user_ids(ctx).await?;
        let mut updated_count = 0;

        for user_id in all_users {
            match self
                .vector_store
                .recompute_user_preference_vector(ctx, &user_id)
                .await
            {
                Ok(_) => {
                    updated_count += 1;
                }
                Err(e) => {
                    warn!(
                        "Failed to rebuild preference vector for user {}: {:?}",
                        user_id, e
                    );
                }
            }

            // Add small delay to avoid overwhelming the database
            if updated_count % 100 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        Ok(updated_count)
    }

    /// Rebuild all entity feature vectors from scratch
    async fn rebuild_all_entity_vectors(&self, ctx: &TenantContext) -> Result<usize> {
        let all_entities = self.vector_store.get_all_entity_ids(ctx).await?;
        let mut updated_count = 0;

        for entity_id in all_entities {
            match self
                .vector_store
                .recompute_entity_feature_vector(ctx, &entity_id)
                .await
            {
                Ok(_) => {
                    updated_count += 1;
                }
                Err(e) => {
                    warn!(
                        "Failed to rebuild feature vector for entity {}: {:?}",
                        entity_id, e
                    );
                }
            }

            // Add small delay to avoid overwhelming the database
            if updated_count % 100 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        Ok(updated_count)
    }

    /// Rebuild vector indices for optimal query performance
    async fn rebuild_vector_indices(&self, _ctx: &TenantContext) -> Result<()> {
        // Rebuild HNSW index for entity feature vectors
        self.vector_store.rebuild_entity_vector_index(None).await?;

        // Rebuild HNSW index for user preference vectors
        self.vector_store.rebuild_user_vector_index(None).await?;

        Ok(())
    }

    /// Clear all caches for the tenant
    async fn clear_all_caches(&self, ctx: &TenantContext) -> Result<usize> {
        let pattern = format!("*:{}:*", ctx.tenant_id);
        let count =
            self.cache.delete_pattern(&pattern).await.map_err(|e| {
                recommendation_models::RecommendationError::CacheError(e.to_string())
            })?;
        Ok(count)
    }
}

impl ModelUpdater {
    /// Start full rebuild background task
    /// Runs every configured interval (default: 24 hours)
    /// Scheduled during low-traffic periods (e.g., 3 AM)
    pub fn start_full_rebuild(self: Arc<Self>, ctx: TenantContext) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval_timer = interval(self.full_rebuild_interval);
            interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            info!(
                "Starting full rebuild task for tenant: {} (interval: {:?})",
                ctx.tenant_id, self.full_rebuild_interval
            );

            // Wait for initial delay to schedule during low-traffic period
            // Calculate time until next 3 AM
            let initial_delay = self.calculate_delay_until_low_traffic();
            if initial_delay > Duration::from_secs(0) {
                info!(
                    "Waiting {:?} until low-traffic period for first full rebuild",
                    initial_delay
                );
                sleep(initial_delay).await;
            }

            loop {
                match self.full_rebuild(&ctx).await {
                    Ok(_) => {
                        info!("Full rebuild completed successfully");
                    }
                    Err(e) => {
                        error!("Full rebuild failed: {:?}", e);
                    }
                }

                interval_timer.tick().await;
            }
        })
    }

    /// Calculate delay until next low-traffic period (3 AM)
    fn calculate_delay_until_low_traffic(&self) -> Duration {
        use chrono::{Local, Timelike};

        let now = Local::now();
        let target_hour = 3; // 3 AM

        let mut target = now
            .date_naive()
            .and_hms_opt(target_hour, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();

        // If we're past 3 AM today, schedule for tomorrow
        if now.hour() >= target_hour {
            target += chrono::Duration::days(1);
        }

        let delay = target.signed_duration_since(now);
        Duration::from_secs(delay.num_seconds().max(0) as u64)
    }
}

impl ModelUpdater {
    /// Calculate trending entities every 1 hour
    /// Updates Redis cache with trending results
    pub async fn update_trending(&self, ctx: &TenantContext) -> Result<()> {
        info!(
            "Starting trending calculation for tenant: {}",
            ctx.tenant_id
        );
        let start_time = std::time::Instant::now();

        // Get all entity types in the system
        let entity_types = self.vector_store.get_all_entity_types(ctx).await?;

        let mut total_trending = 0;
        let mut trending_by_type: Vec<(String, usize)> = Vec::new();

        for entity_type in entity_types {
            match self.calculate_trending_for_type(ctx, &entity_type).await {
                Ok(count) => {
                    debug!(
                        "Calculated {} trending entities for type: {}",
                        count, entity_type
                    );
                    total_trending += count;
                    trending_by_type.push((entity_type.clone(), count));
                }
                Err(e) => {
                    error!(
                        "Failed to calculate trending for type {}: {:?}",
                        entity_type, e
                    );
                }
            }
        }

        // Also calculate overall trending (all entity types)
        match self.calculate_trending_for_type(ctx, "all").await {
            Ok(count) => {
                debug!("Calculated {} overall trending entities", count);
                total_trending += count;
            }
            Err(e) => {
                error!("Failed to calculate overall trending: {:?}", e);
            }
        }

        let duration = start_time.elapsed();
        info!(
            "Completed trending calculation for tenant: {} in {:?} ({} trending entities)",
            ctx.tenant_id, duration, total_trending
        );

        // Trigger trending_changed webhook for each entity type
        if self.webhook_delivery.is_some() && !trending_by_type.is_empty() {
            for (entity_type, count) in trending_by_type {
                let event =
                    WebhookEvent::trending_changed(ctx.tenant_id.clone(), entity_type, count);

                if let Some(webhook) = &self.webhook_delivery {
                    webhook.clone().dispatch_async(event);
                }
            }
        }

        Ok(())
    }

    /// Calculate trending entities for a specific entity type
    async fn calculate_trending_for_type(
        &self,
        ctx: &TenantContext,
        entity_type: &str,
    ) -> Result<usize> {
        // Get trending entities from last 7 days
        let entity_type_filter = if entity_type == "all" {
            None
        } else {
            Some(entity_type)
        };

        let trending_stats = self
            .vector_store
            .get_trending_entity_stats(ctx, entity_type_filter, 100)
            .await?;

        if trending_stats.is_empty() {
            debug!("No trending entities found for type: {}", entity_type);
            return Ok(0);
        }

        // Normalize scores to [0, 1] range
        let max_score = trending_stats
            .iter()
            .map(|(_, _, score)| *score)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);

        let normalized_trending: Vec<(String, String, f32)> = trending_stats
            .into_iter()
            .map(|(entity_id, entity_type, score)| {
                let normalized_score = if max_score > 0.0 {
                    score / max_score
                } else {
                    0.0
                };
                (entity_id, entity_type, normalized_score)
            })
            .collect();

        let count = normalized_trending.len();

        // Cache trending results for different counts (10, 20, 50, 100)
        for cache_count in [10, 20, 50, 100] {
            let cache_key = format!("trending:{}:{}:{}", ctx.tenant_id, entity_type, cache_count);
            let trending_subset: Vec<_> = normalized_trending
                .iter()
                .take(cache_count)
                .cloned()
                .collect();

            // Serialize to JSON for caching
            if let Ok(json) = serde_json::to_string(&trending_subset) {
                let _ = self
                    .cache
                    .set_string(&cache_key, &json, Duration::from_secs(3600))
                    .await;
            }
        }

        Ok(count)
    }
}

impl ModelUpdater {
    /// Start trending calculation background task
    /// Runs every configured interval (default: 1 hour)
    pub fn start_trending_updates(
        self: Arc<Self>,
        ctx: TenantContext,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval_timer = interval(self.trending_interval);
            interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            info!(
                "Starting trending update task for tenant: {} (interval: {:?})",
                ctx.tenant_id, self.trending_interval
            );

            loop {
                interval_timer.tick().await;

                match self.update_trending(&ctx).await {
                    Ok(_) => {
                        debug!("Trending update completed successfully");
                    }
                    Err(e) => {
                        error!("Trending update failed: {:?}", e);
                    }
                }
            }
        })
    }

    /// Start all background tasks for a tenant
    /// Returns a TaskScheduler that manages all spawned tasks
    pub fn start_all_tasks(self: Arc<Self>, ctx: TenantContext) -> TaskScheduler {
        let mut scheduler = TaskScheduler::new();

        // Start incremental updates
        let incremental_handle = self.clone().start_incremental_updates(ctx.clone());
        scheduler.handles.push(incremental_handle);

        // Start full rebuild
        let rebuild_handle = self.clone().start_full_rebuild(ctx.clone());
        scheduler.handles.push(rebuild_handle);

        // Start trending updates
        let trending_handle = self.clone().start_trending_updates(ctx.clone());
        scheduler.handles.push(trending_handle);

        info!("Started all background tasks for tenant: {}", ctx.tenant_id);

        scheduler
    }
}
