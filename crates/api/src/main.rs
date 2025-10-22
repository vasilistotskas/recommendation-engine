use anyhow::Result;
use recommendation_api::{
    metrics_middleware::MetricsLayer,
    middleware::{AuthLayer, RateLimitLayer, RequestIdLayer},
    routes::build_router,
    state::AppState,
};
use recommendation_engine::{
    CollaborativeConfig, CollaborativeFilteringEngine, ContentBasedConfig,
    ContentBasedFilteringEngine, HybridConfig, HybridEngine,
};
use recommendation_service::{EntityService, InteractionService, RecommendationService};
use recommendation_storage::{Database, DatabaseConfig, RedisCache, RedisCacheConfig, VectorStore};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,recommendation_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Initialize Prometheus metrics exporter
    let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    tracing::info!("Prometheus metrics exporter initialized");
    tracing::info!("Starting Recommendation Engine API");

    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::warn!("DATABASE_URL not set, using default");
        "postgresql://localhost:5432/recommendations".to_string()
    });

    let db_config = DatabaseConfig {
        url: database_url,
        max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5),
        acquire_timeout_secs: 3,
        idle_timeout_secs: 600,
        max_lifetime_secs: 1800,
    };

    let database = Database::new(db_config).await?;
    tracing::info!("Database connection established");

    // Initialize Redis cache
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| {
        tracing::warn!("REDIS_URL not set, using default");
        "redis://localhost:6379".to_string()
    });

    let redis_config = RedisCacheConfig {
        url: redis_url,
        pool_size: std::env::var("REDIS_POOL_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
        connection_timeout: std::time::Duration::from_secs(5),
        max_retry_attempts: 3,
        retry_backoff_ms: 100,
    };

    let redis_cache = Arc::new(RedisCache::new(redis_config).await?);
    tracing::info!("Redis cache connection established");

    // Initialize services
    let vector_store = Arc::new(VectorStore::new(database.pool().clone()));
    let entity_service = Arc::new(EntityService::new(Arc::clone(&vector_store)));
    let interaction_service = Arc::new(InteractionService::new(Arc::clone(&vector_store)));

    // Initialize recommendation engines
    let collaborative_config = CollaborativeConfig {
        k_neighbors: std::env::var("COLLABORATIVE_K_NEIGHBORS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50),
        min_similarity: std::env::var("COLLABORATIVE_MIN_SIMILARITY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.1),
        default_count: 10,
    };

    let collaborative_engine = Arc::new(CollaborativeFilteringEngine::new(
        Arc::clone(&vector_store),
        Arc::clone(&redis_cache),
        collaborative_config,
    ));

    let content_based_config = ContentBasedConfig {
        similarity_threshold: std::env::var("SIMILARITY_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.5),
        default_count: 10,
    };

    let content_based_engine = Arc::new(ContentBasedFilteringEngine::new(
        Arc::clone(&vector_store),
        Arc::clone(&redis_cache),
        content_based_config,
    ));

    // Configure hybrid engine with weights from environment or defaults
    let hybrid_config = HybridConfig {
        collaborative_weight: std::env::var("COLLABORATIVE_WEIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.6),
        content_weight: std::env::var("CONTENT_BASED_WEIGHT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.4),
        enable_diversity: true,
        min_categories: 3,
        default_count: 10,
    };

    let hybrid_engine = Arc::new(HybridEngine::new(
        Arc::clone(&collaborative_engine),
        Arc::clone(&content_based_engine),
        Arc::clone(&redis_cache),
        hybrid_config,
    )?);

    // Initialize recommendation service
    let recommendation_service = Arc::new(RecommendationService::new(
        Arc::clone(&collaborative_engine),
        Arc::clone(&content_based_engine),
        Arc::clone(&hybrid_engine),
        Arc::clone(&vector_store),
        Arc::clone(&redis_cache),
    ));

    // Initialize interaction type service
    let interaction_type_service = Arc::new(recommendation_service::InteractionTypeService::new(
        Arc::clone(&vector_store),
    ));

    // Get default tenant ID from environment
    let default_tenant_id =
        std::env::var("DEFAULT_TENANT_ID").unwrap_or_else(|_| "default".to_string());

    // Create application state
    let app_state = AppState::new(
        entity_service,
        interaction_service,
        interaction_type_service,
        recommendation_service,
        Arc::clone(&vector_store),
        Arc::clone(&redis_cache),
        default_tenant_id,
        metrics_handle,
    );

    // Get API key from environment
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| {
        tracing::warn!("API_KEY not set, using default (insecure for production!)");
        "dev-api-key-change-in-production".to_string()
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Configure tracing
    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    // Build application router with all endpoints
    // Layers are applied bottom-to-top, so the order is:
    // 1. Compression (innermost)
    // 2. CORS
    // 3. Tracing
    // 4. Metrics (track after tracing for accurate timing)
    // 5. Authentication
    // 6. Rate Limiting (optional, can be disabled for performance testing)
    // 7. Request ID (outermost)

    // Check if rate limiting should be disabled (for performance testing)
    let disable_rate_limit = std::env::var("DISABLE_RATE_LIMIT")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    // Clone app_state for shutdown signal handler (before moving it)
    let shutdown_state = app_state.clone();

    let mut app = build_router(app_state)
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(trace)
        .layer(MetricsLayer)
        .layer(AuthLayer::new(api_key));

    if !disable_rate_limit {
        app = app.layer(RateLimitLayer::from_env());
        tracing::info!("Rate limiting enabled");
    } else {
        tracing::warn!("Rate limiting DISABLED - only use for performance testing!");
    }

    let app = app.layer(RequestIdLayer);

    // Get server configuration
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on {}:{}", host, port);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Create graceful shutdown signal handler
    let shutdown_signal = async move {
        // Wait for SIGTERM (Kubernetes shutdown signal)
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Received shutdown signal, starting graceful shutdown...");

        // Mark service as shutting down - this will make readiness probe return 503
        shutdown_state.set_shutting_down();

        // Get shutdown timeout from environment
        let shutdown_timeout_secs = std::env::var("SHUTDOWN_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        tracing::info!(
            "Waiting up to {} seconds for in-flight requests to complete",
            shutdown_timeout_secs
        );

        // Wait for in-flight requests to complete
        tokio::time::sleep(std::time::Duration::from_secs(shutdown_timeout_secs)).await;

        tracing::info!("Graceful shutdown complete");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    Ok(())
}
