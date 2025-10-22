use axum::{extract::Request, response::Response};
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

// ============================================================================
// Metrics Middleware
// ============================================================================

/// Layer that tracks HTTP request metrics
#[derive(Clone)]
pub struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware { inner }
    }
}

/// Middleware that tracks HTTP request metrics
#[derive(Clone)]
pub struct MetricsMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for MetricsMiddleware<S>
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
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        // Normalize path to remove IDs for better metrics grouping
        let normalized_path = normalize_path(&path);

        let future = self.inner.call(req);

        Box::pin(async move {
            let result = future.await;
            let duration = start.elapsed();

            // Record metrics
            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();

                    // Increment request counter
                    metrics::counter!("http_requests_total",
                        "method" => method.clone(),
                        "path" => normalized_path.clone(),
                        "status" => status.to_string(),
                    )
                    .increment(1);

                    // Record request duration
                    metrics::histogram!("http_request_duration_seconds",
                        "method" => method.clone(),
                        "path" => normalized_path.clone(),
                        "status" => status.to_string(),
                    )
                    .record(duration.as_secs_f64());

                    // Track error rates
                    if status >= 400 {
                        metrics::counter!("http_requests_errors_total",
                            "method" => method,
                            "path" => normalized_path,
                            "status" => status.to_string(),
                        )
                        .increment(1);
                    }
                }
                Err(_) => {
                    // Record internal errors
                    metrics::counter!("http_requests_total",
                        "method" => method.clone(),
                        "path" => normalized_path.clone(),
                        "status" => "500",
                    )
                    .increment(1);

                    metrics::counter!("http_requests_errors_total",
                        "method" => method,
                        "path" => normalized_path,
                        "status" => "500",
                    )
                    .increment(1);
                }
            }

            result
        })
    }
}

/// Normalize API path for metrics grouping
/// Replaces entity IDs and UUIDs with placeholders
fn normalize_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let mut normalized = Vec::new();

    for part in parts.iter() {
        if part.is_empty() {
            continue;
        }

        // Replace UUIDs and entity IDs with placeholders
        if is_uuid_or_id(part) {
            normalized.push("{id}");
        } else {
            normalized.push(part);
        }
    }

    format!("/{}", normalized.join("/"))
}

/// Check if a path segment looks like a UUID or entity ID
fn is_uuid_or_id(s: &str) -> bool {
    // Check if it's a UUID
    if s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4 {
        return true;
    }

    // Check if it's all digits (numeric ID)
    if s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // Check if it looks like an entity ID (e.g., "product-123", "user_456")
    if s.contains('-') || s.contains('_') {
        let parts: Vec<&str> = s.split(['-', '_']).collect();
        if parts.len() >= 2 && parts.iter().any(|p| p.chars().all(|c| c.is_ascii_digit())) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path("/api/v1/entities/123"),
            "/api/v1/entities/{id}"
        );
        assert_eq!(
            normalize_path("/api/v1/users/user-456"),
            "/api/v1/users/{id}"
        );
        assert_eq!(
            normalize_path("/api/v1/entities/550e8400-e29b-41d4-a716-446655440000"),
            "/api/v1/entities/{id}"
        );
        assert_eq!(normalize_path("/health"), "/health");
        assert_eq!(
            normalize_path("/api/v1/recommendations/trending"),
            "/api/v1/recommendations/trending"
        );
    }

    #[test]
    fn test_is_uuid_or_id() {
        assert!(is_uuid_or_id("123"));
        assert!(is_uuid_or_id("550e8400-e29b-41d4-a716-446655440000"));
        assert!(is_uuid_or_id("user-123"));
        assert!(is_uuid_or_id("product_456"));
        assert!(!is_uuid_or_id("health"));
        assert!(!is_uuid_or_id("trending"));
        assert!(!is_uuid_or_id("api"));
    }
}
