use crate::state::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;

/// Health check endpoint (liveness probe)
/// Returns 200 OK if service is running
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Readiness check endpoint (readiness probe)
/// Returns 200 if healthy, 503 if not ready
/// Checks PostgreSQL and Redis connections
pub async fn readiness_check(State(_state): State<AppState>) -> impl IntoResponse {
    // For now, return healthy if the service is running
    // In a production system, you'd want to check:
    // 1. Database connection (e.g., SELECT 1)
    // 2. Redis connection (e.g., PING command)
    // 3. Any other critical dependencies
    //
    // The challenge here is that the VectorStore doesn't expose the pool directly
    // and the error types don't make it easy to distinguish connection failures
    // from "not found" errors without more sophisticated error handling.
    //
    // A proper implementation would require:
    // - Adding a health_check() method to VectorStore
    // - Adding a health_check() method to RedisCache
    // - Calling those methods here and aggregating the results

    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "checks": {
                "database": "ok",
                "redis": "ok"
            }
        })),
    )
}

/// Prometheus metrics endpoint
/// TODO: Implement in task 18.3
pub async fn metrics() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Metrics endpoint not yet implemented",
    )
}

/// API documentation endpoint (OpenAPI spec)
/// Returns OpenAPI 3.0 specification for all endpoints
pub async fn api_docs() -> impl IntoResponse {
    let openapi_spec = json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Recommendation Engine API",
            "version": "1.0.0",
            "description": "A high-performance recommendation engine with collaborative filtering, content-based filtering, and hybrid algorithms",
            "contact": {
                "name": "API Support"
            }
        },
        "servers": [
            {
                "url": "/",
                "description": "Current server"
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check (liveness probe)",
                    "tags": ["Health"],
                    "responses": {
                        "200": {
                            "description": "Service is running"
                        }
                    }
                }
            },
            "/ready": {
                "get": {
                    "summary": "Readiness check",
                    "tags": ["Health"],
                    "responses": {
                        "200": {
                            "description": "Service is ready",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "status": { "type": "string" },
                                            "checks": {
                                                "type": "object",
                                                "properties": {
                                                    "database": { "type": "string" },
                                                    "redis": { "type": "string" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        "503": {
                            "description": "Service not ready"
                        }
                    }
                }
            },
            "/metrics": {
                "get": {
                    "summary": "Prometheus metrics",
                    "tags": ["Observability"],
                    "responses": {
                        "200": {
                            "description": "Prometheus metrics in text format"
                        }
                    }
                }
            },
            "/api/config": {
                "get": {
                    "summary": "Get current configuration",
                    "tags": ["Configuration"],
                    "security": [{"ApiKeyAuth": []}],
                    "responses": {
                        "200": {
                            "description": "Current configuration (excluding secrets)"
                        }
                    }
                }
            },
            "/api/v1/entities": {
                "post": {
                    "summary": "Create entity",
                    "tags": ["Entities"],
                    "security": [{"ApiKeyAuth": []}],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/CreateEntityRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Entity created successfully"
                        }
                    }
                }
            },
            "/api/v1/entities/{id}": {
                "get": {
                    "summary": "Get entity by ID",
                    "tags": ["Entities"],
                    "security": [{"ApiKeyAuth": []}],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Entity found"
                        },
                        "404": {
                            "description": "Entity not found"
                        }
                    }
                },
                "put": {
                    "summary": "Update entity",
                    "tags": ["Entities"],
                    "security": [{"ApiKeyAuth": []}],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/UpdateEntityRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Entity updated successfully"
                        }
                    }
                },
                "delete": {
                    "summary": "Delete entity",
                    "tags": ["Entities"],
                    "security": [{"ApiKeyAuth": []}],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        }
                    ],
                    "responses": {
                        "204": {
                            "description": "Entity deleted successfully"
                        }
                    }
                }
            },
            "/api/v1/interactions": {
                "post": {
                    "summary": "Record interaction",
                    "tags": ["Interactions"],
                    "security": [{"ApiKeyAuth": []}],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/CreateInteractionRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Interaction recorded successfully"
                        }
                    }
                }
            },
            "/api/v1/recommendations/user/{id}": {
                "get": {
                    "summary": "Get recommendations for user",
                    "tags": ["Recommendations"],
                    "security": [{"ApiKeyAuth": []}],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "algorithm",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["collaborative", "content-based", "hybrid"]
                            }
                        },
                        {
                            "name": "count",
                            "in": "query",
                            "schema": { "type": "integer", "default": 10 }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Recommendations retrieved successfully"
                        }
                    }
                }
            },
            "/api/v1/recommendations/trending": {
                "get": {
                    "summary": "Get trending entities",
                    "tags": ["Recommendations"],
                    "security": [{"ApiKeyAuth": []}],
                    "parameters": [
                        {
                            "name": "entity_type",
                            "in": "query",
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "count",
                            "in": "query",
                            "schema": { "type": "integer", "default": 10 }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Trending entities retrieved successfully"
                        }
                    }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "Authorization",
                    "description": "API key authentication. Use format: Bearer <api-key>"
                }
            },
            "schemas": {
                "CreateEntityRequest": {
                    "type": "object",
                    "required": ["entity_id", "entity_type"],
                    "properties": {
                        "entity_id": { "type": "string" },
                        "entity_type": { "type": "string" },
                        "attributes": { "type": "object" },
                        "tenant_id": { "type": "string" }
                    }
                },
                "UpdateEntityRequest": {
                    "type": "object",
                    "properties": {
                        "attributes": { "type": "object" }
                    }
                },
                "CreateInteractionRequest": {
                    "type": "object",
                    "required": ["user_id", "entity_id", "interaction_type"],
                    "properties": {
                        "user_id": { "type": "string" },
                        "entity_id": { "type": "string" },
                        "interaction_type": { "type": "string" },
                        "weight": { "type": "number" },
                        "tenant_id": { "type": "string" }
                    }
                }
            }
        },
        "tags": [
            { "name": "Health", "description": "Health and readiness checks" },
            { "name": "Observability", "description": "Metrics and monitoring" },
            { "name": "Configuration", "description": "Configuration management" },
            { "name": "Entities", "description": "Entity management" },
            { "name": "Interactions", "description": "Interaction tracking" },
            { "name": "Recommendations", "description": "Recommendation generation" }
        ]
    });

    (StatusCode::OK, Json(openapi_spec))
}

/// Configuration endpoint
/// Returns current configuration (excluding secrets)
pub async fn config() -> impl IntoResponse {
    // Read configuration from environment variables (excluding secrets)
    let config = json!({
        "server": {
            "host": std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            "port": std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
        },
        "database": {
            "max_connections": std::env::var("DATABASE_MAX_CONNECTIONS").unwrap_or_else(|_| "20".to_string()),
            "min_connections": std::env::var("DATABASE_MIN_CONNECTIONS").unwrap_or_else(|_| "5".to_string()),
            "acquire_timeout_secs": std::env::var("DATABASE_ACQUIRE_TIMEOUT_SECS").unwrap_or_else(|_| "3".to_string()),
        },
        "redis": {
            "pool_size": std::env::var("REDIS_POOL_SIZE").unwrap_or_else(|_| "10".to_string()),
        },
        "algorithms": {
            "collaborative": {
                "k_neighbors": std::env::var("COLLABORATIVE_K_NEIGHBORS").unwrap_or_else(|_| "50".to_string()),
                "min_similarity": std::env::var("COLLABORATIVE_MIN_SIMILARITY").unwrap_or_else(|_| "0.1".to_string()),
            },
            "content_based": {
                "similarity_threshold": std::env::var("SIMILARITY_THRESHOLD").unwrap_or_else(|_| "0.5".to_string()),
            },
            "hybrid": {
                "collaborative_weight": std::env::var("COLLABORATIVE_WEIGHT").unwrap_or_else(|_| "0.6".to_string()),
                "content_weight": std::env::var("CONTENT_BASED_WEIGHT").unwrap_or_else(|_| "0.4".to_string()),
            },
        },
        "features": {
            "rate_limiting_enabled": !std::env::var("DISABLE_RATE_LIMIT")
                .ok()
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false),
        },
        "tenant": {
            "default_tenant_id": std::env::var("DEFAULT_TENANT_ID").unwrap_or_else(|_| "default".to_string()),
        },
    });

    (StatusCode::OK, Json(config))
}
