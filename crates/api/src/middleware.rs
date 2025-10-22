use axum::{extract::Request, http::HeaderValue, response::Response};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tower::{Layer, Service};
use uuid::Uuid;

/// Request ID header name
pub const X_REQUEST_ID: &str = "x-request-id";

/// Layer that adds a unique request ID to each request
#[derive(Clone)]
pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdMiddleware { inner }
    }
}

/// Middleware that adds a unique request ID to each request
#[derive(Clone)]
pub struct RequestIdMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for RequestIdMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        // Generate or extract request ID
        let request_id = req
            .headers()
            .get(X_REQUEST_ID)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Store request ID in extensions for handlers to access
        req.extensions_mut().insert(RequestId(request_id.clone()));

        // Add request ID to tracing span
        let span = tracing::info_span!(
            "request",
            request_id = %request_id,
            method = %req.method(),
            uri = %req.uri(),
        );

        let future = self.inner.call(req);

        Box::pin(async move {
            let _enter = span.enter();
            let mut response = future.await?;

            // Add request ID to response headers
            if let Ok(header_value) = HeaderValue::from_str(&request_id) {
                response.headers_mut().insert(X_REQUEST_ID, header_value);
            }

            Ok(response)
        })
    }
}

/// Request ID extractor for handlers
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

// ============================================================================
// Authentication Middleware
// ============================================================================

/// Authorization header name
pub const AUTHORIZATION: &str = "authorization";

/// Layer that validates API keys
#[derive(Clone)]
pub struct AuthLayer {
    api_key: String,
}

impl AuthLayer {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            api_key: self.api_key.clone(),
        }
    }
}

/// Middleware that validates API keys
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    api_key: String,
}

impl<S, B> Service<Request<B>> for AuthMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // Skip authentication for health check and metrics endpoints
        let path = req.uri().path();
        if path == "/health" || path == "/ready" || path == "/metrics" {
            let future = self.inner.call(req);
            return Box::pin(future);
        }

        // Extract API key from Authorization header
        let auth_header = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let expected_key = self.api_key.clone();
        let is_valid = match auth_header {
            Some(header) => {
                // Support both "Bearer <key>" and just "<key>" formats
                let key = header
                    .strip_prefix("Bearer ")
                    .or_else(|| header.strip_prefix("bearer "))
                    .unwrap_or(header);
                key == expected_key
            }
            None => false,
        };

        if !is_valid {
            tracing::warn!("Unauthorized request to {}", path);
            return Box::pin(async move {
                let response = Response::builder()
                    .status(401)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"error":{"code":401,"message":"Unauthorized: Invalid or missing API key"}}"#,
                    ))
                    .unwrap();
                Ok(response)
            });
        }

        let future = self.inner.call(req);
        Box::pin(future)
    }
}

// ============================================================================
// Rate Limiting Middleware
// ============================================================================

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Rate limiter configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 1000,
            window: Duration::from_secs(60),
        }
    }
}

/// Rate limiter state for a single client
#[derive(Debug)]
struct RateLimitState {
    requests: Vec<Instant>,
}

impl RateLimitState {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    fn check_and_update(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let window_start = now - config.window;

        // Remove old requests outside the window
        self.requests.retain(|&time| time > window_start);

        // Check if limit exceeded
        if self.requests.len() >= config.max_requests {
            return false;
        }

        // Add current request
        self.requests.push(now);
        true
    }
}

/// Layer that implements rate limiting
#[derive(Clone)]
pub struct RateLimitLayer {
    config: RateLimitConfig,
    state: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn from_env() -> Self {
        let max_requests = std::env::var("RATE_LIMIT_MAX_REQUESTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        let window_secs = std::env::var("RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        Self::new(RateLimitConfig {
            max_requests,
            window: Duration::from_secs(window_secs),
        })
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            config: self.config.clone(),
            state: self.state.clone(),
        }
    }
}

/// Middleware that implements rate limiting
#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    config: RateLimitConfig,
    state: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

impl<S, B> Service<Request<B>> for RateLimitMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        // Skip rate limiting for health check and metrics endpoints
        let path = req.uri().path();
        if path == "/health" || path == "/ready" || path == "/metrics" {
            let future = self.inner.call(req);
            return Box::pin(future);
        }

        // Check for rate limit bypass header (for performance testing)
        let bypass_rate_limit = req
            .headers()
            .get("x-bypass-rate-limit")
            .and_then(|v| v.to_str().ok())
            .map(|s| s == "true")
            .unwrap_or(false);

        if bypass_rate_limit {
            let future = self.inner.call(req);
            return Box::pin(future);
        }

        // Determine client identifier (API key or IP address)
        let client_id = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| {
                // Fallback to IP address if no API key
                req.headers()
                    .get("x-forwarded-for")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Check rate limit
        let mut state = self.state.lock().unwrap();
        let client_state = state
            .entry(client_id.clone())
            .or_insert_with(RateLimitState::new);

        let allowed = client_state.check_and_update(&self.config);
        drop(state); // Release lock

        if !allowed {
            tracing::warn!("Rate limit exceeded for client: {}", client_id);
            return Box::pin(async move {
                let response = Response::builder()
                    .status(429)
                    .header("content-type", "application/json")
                    .header("retry-after", "60")
                    .body(axum::body::Body::from(
                        r#"{"error":{"code":429,"message":"Too Many Requests: Rate limit exceeded"}}"#,
                    ))
                    .unwrap();
                Ok(response)
            });
        }

        let future = self.inner.call(req);
        Box::pin(future)
    }
}
