use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::state::AppState;

/// Build the complete API router with all endpoints
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Health and readiness endpoints
        .route("/health", get(crate::handlers::health::health_check))
        .route("/ready", get(crate::handlers::health::readiness_check))
        // Entity routes
        .nest("/api/v1/entities", entity_routes())
        // Interaction routes
        .nest("/api/v1/interactions", interaction_routes())
        // Interaction type registry routes
        .nest("/api/v1/interaction-types", interaction_type_routes())
        // Recommendation routes
        .nest("/api/v1/recommendations", recommendation_routes())
        // Export routes
        .nest("/api/v1/export", export_routes())
        // Metrics endpoint (will be implemented in task 18)
        .route("/metrics", get(crate::handlers::health::metrics))
        // API docs endpoint (will be implemented in task 18)
        .route("/api/docs", get(crate::handlers::health::api_docs))
        // Config endpoint (will be implemented in task 18)
        .route("/api/config", get(crate::handlers::health::config))
        .with_state(state)
}

/// Entity management routes
fn entity_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(crate::handlers::entity::create_entity))
        .route("/{id}", get(crate::handlers::entity::get_entity))
        .route("/{id}", put(crate::handlers::entity::update_entity))
        .route("/{id}", delete(crate::handlers::entity::delete_entity))
        .route("/bulk", post(crate::handlers::entity::bulk_import_entities))
}

/// Interaction tracking routes
fn interaction_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(crate::handlers::interaction::create_interaction))
        .route("/user/{id}", get(crate::handlers::interaction::get_user_interactions))
        .route("/bulk", post(crate::handlers::interaction::bulk_import_interactions))
}

/// Recommendation routes
fn recommendation_routes() -> Router<AppState> {
    Router::new()
        .route("/user/{id}", get(crate::handlers::recommendation::get_user_recommendations))
        .route("/entity/{id}", get(crate::handlers::recommendation::get_similar_entities))
        .route("/trending", get(crate::handlers::recommendation::get_trending_entities))
}

/// Export routes
fn export_routes() -> Router<AppState> {
    Router::new()
        .route("/entities", get(crate::handlers::export::export_entities))
        .route("/interactions", get(crate::handlers::export::export_interactions))
        .route("/users", get(crate::handlers::export::export_users))
}

/// Interaction type registry routes
fn interaction_type_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(crate::handlers::interaction_type::register_interaction_type))
        .route("/", get(crate::handlers::interaction_type::list_interaction_types))
        .route("/{type}", get(crate::handlers::interaction_type::get_interaction_type))
        .route("/{type}", put(crate::handlers::interaction_type::update_interaction_type))
        .route("/{type}", delete(crate::handlers::interaction_type::delete_interaction_type))
}
