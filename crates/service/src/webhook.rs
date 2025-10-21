use chrono::{DateTime, Utc};
use recommendation_models::{RecommendationError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Webhook event types that can be triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// Triggered after model updates complete
    ModelUpdated,
    /// Triggered when trending entities change
    TrendingChanged,
    /// Triggered when error threshold is exceeded
    ErrorThresholdExceeded,
}

impl std::fmt::Display for WebhookEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookEventType::ModelUpdated => write!(f, "model_updated"),
            WebhookEventType::TrendingChanged => write!(f, "trending_changed"),
            WebhookEventType::ErrorThresholdExceeded => write!(f, "error_threshold_exceeded"),
        }
    }
}

/// Webhook event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Event type
    pub event_type: WebhookEventType,
    /// Tenant ID
    pub tenant_id: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event-specific data
    pub data: HashMap<String, serde_json::Value>,
}

impl WebhookEvent {
    /// Create a new webhook event
    pub fn new(
        event_type: WebhookEventType,
        tenant_id: String,
        data: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            event_type,
            tenant_id,
            timestamp: Utc::now(),
            data,
        }
    }

    /// Create a model_updated event
    pub fn model_updated(
        tenant_id: String,
        users_updated: usize,
        entities_updated: usize,
        duration_ms: u64,
    ) -> Self {
        let mut data = HashMap::new();
        data.insert(
            "users_updated".to_string(),
            serde_json::json!(users_updated),
        );
        data.insert(
            "entities_updated".to_string(),
            serde_json::json!(entities_updated),
        );
        data.insert("duration_ms".to_string(), serde_json::json!(duration_ms));

        Self::new(WebhookEventType::ModelUpdated, tenant_id, data)
    }

    /// Create a trending_changed event
    pub fn trending_changed(tenant_id: String, entity_type: String, trending_count: usize) -> Self {
        let mut data = HashMap::new();
        data.insert("entity_type".to_string(), serde_json::json!(entity_type));
        data.insert(
            "trending_count".to_string(),
            serde_json::json!(trending_count),
        );

        Self::new(WebhookEventType::TrendingChanged, tenant_id, data)
    }

    /// Create an error_threshold_exceeded event
    pub fn error_threshold_exceeded(
        tenant_id: String,
        error_type: String,
        error_count: usize,
        threshold: usize,
    ) -> Self {
        let mut data = HashMap::new();
        data.insert("error_type".to_string(), serde_json::json!(error_type));
        data.insert("error_count".to_string(), serde_json::json!(error_count));
        data.insert("threshold".to_string(), serde_json::json!(threshold));

        Self::new(WebhookEventType::ErrorThresholdExceeded, tenant_id, data)
    }
}

/// Webhook delivery service
pub struct WebhookDelivery {
    client: Client,
    webhook_urls: Vec<String>,
    secret_key: String,
    max_retries: u32,
    retry_delay_ms: u64,
}

impl WebhookDelivery {
    /// Create a new WebhookDelivery instance
    pub fn new(
        webhook_urls: Vec<String>,
        secret_key: String,
        max_retries: u32,
        retry_delay_ms: u64,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        info!(
            "Initialized WebhookDelivery with {} URLs, max_retries: {}, retry_delay: {}ms",
            webhook_urls.len(),
            max_retries,
            retry_delay_ms
        );

        Self {
            client,
            webhook_urls,
            secret_key,
            max_retries,
            retry_delay_ms,
        }
    }

    /// Create a WebhookDelivery with default settings
    /// - max_retries: 3
    /// - retry_delay: 1000ms (exponential backoff)
    pub fn with_defaults(webhook_urls: Vec<String>, secret_key: String) -> Self {
        Self::new(webhook_urls, secret_key, 3, 1000)
    }

    /// Generate HMAC-SHA256 signature for webhook payload
    pub fn generate_signature(&self, payload: &str) -> String {
        use sha2::{Digest, Sha256};

        let mut mac = Sha256::new();
        mac.update(self.secret_key.as_bytes());
        mac.update(payload.as_bytes());
        let result = mac.finalize();

        format!("sha256={}", hex::encode(result))
    }

    /// Verify webhook signature
    pub fn verify_signature(&self, payload: &str, signature: &str) -> bool {
        let expected_signature = self.generate_signature(payload);
        expected_signature == signature
    }

    /// Send webhook event to a single URL with retry logic
    async fn send_to_url(&self, url: &str, event: &WebhookEvent, attempt: u32) -> Result<()> {
        let payload = serde_json::to_string(event).map_err(|e| {
            error!("Failed to serialize webhook event: {:?}", e);
            RecommendationError::InternalError
        })?;

        let signature = self.generate_signature(&payload);

        debug!(
            "Sending webhook to {} (attempt {}/{}): event_type={}, tenant_id={}",
            url,
            attempt + 1,
            self.max_retries + 1,
            event.event_type,
            event.tenant_id
        );

        let response = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", signature)
            .header("X-Webhook-Event", event.event_type.to_string())
            .header("X-Webhook-Timestamp", event.timestamp.to_rfc3339())
            .body(payload)
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!(
                        "Successfully delivered webhook to {}: status={}",
                        url,
                        resp.status()
                    );
                    Ok(())
                } else {
                    let status = resp.status();
                    let error_body = resp.text().await.unwrap_or_default();
                    warn!(
                        "Webhook delivery failed to {}: status={}, body={}",
                        url, status, error_body
                    );
                    Err(RecommendationError::InternalError)
                }
            }
            Err(e) => {
                warn!("Failed to send webhook to {}: {:?}", url, e);
                Err(RecommendationError::InternalError)
            }
        }
    }

    /// Send webhook event to a URL with exponential backoff retry
    async fn send_with_retry(&self, url: &str, event: &WebhookEvent) -> Result<()> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.send_to_url(url, event, attempt).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        // Exponential backoff: delay * 2^attempt
                        let delay_ms = self.retry_delay_ms * 2_u64.pow(attempt);
                        debug!(
                            "Retrying webhook to {} after {}ms (attempt {}/{})",
                            url,
                            delay_ms,
                            attempt + 1,
                            self.max_retries
                        );
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }

        error!(
            "Failed to deliver webhook to {} after {} attempts",
            url,
            self.max_retries + 1
        );

        Err(last_error.unwrap_or(RecommendationError::InternalError))
    }

    /// Dispatch webhook event to all configured URLs
    pub async fn dispatch(&self, event: WebhookEvent) -> Vec<Result<()>> {
        if self.webhook_urls.is_empty() {
            debug!("No webhook URLs configured, skipping dispatch");
            return vec![];
        }

        info!(
            "Dispatching webhook event: type={}, tenant_id={}, timestamp={}, urls={}",
            event.event_type,
            event.tenant_id,
            event.timestamp.to_rfc3339(),
            self.webhook_urls.len()
        );

        let mut results = Vec::new();

        for url in &self.webhook_urls {
            info!(
                "Attempting webhook delivery: url={}, event_type={}, tenant_id={}",
                url, event.event_type, event.tenant_id
            );

            let result = self.send_with_retry(url, &event).await;

            // Log delivery result
            match &result {
                Ok(_) => {
                    info!(
                        "Webhook delivery successful: url={}, event_type={}, tenant_id={}",
                        url, event.event_type, event.tenant_id
                    );
                }
                Err(e) => {
                    error!(
                        "Webhook delivery failed: url={}, event_type={}, tenant_id={}, error={:?}",
                        url, event.event_type, event.tenant_id, e
                    );
                }
            }

            results.push(result);
        }

        results
    }

    /// Dispatch webhook event asynchronously (fire and forget)
    pub fn dispatch_async(self: Arc<Self>, event: WebhookEvent) {
        let event_type = event.event_type.clone();
        let tenant_id = event.tenant_id.clone();

        tokio::spawn(async move {
            info!(
                "Starting async webhook dispatch: event_type={}, tenant_id={}",
                event_type, tenant_id
            );

            let results = self.dispatch(event).await;

            let success_count = results.iter().filter(|r| r.is_ok()).count();
            let failure_count = results.len() - success_count;

            if failure_count > 0 {
                warn!(
                    "Webhook dispatch completed with failures: event_type={}, tenant_id={}, success={}, failed={}",
                    event_type, tenant_id, success_count, failure_count
                );
            } else {
                info!(
                    "Webhook dispatch completed successfully: event_type={}, tenant_id={}, delivered to {} URLs",
                    event_type, tenant_id, success_count
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_creation() {
        let event = WebhookEvent::model_updated("tenant_1".to_string(), 100, 50, 1500);

        assert_eq!(event.event_type, WebhookEventType::ModelUpdated);
        assert_eq!(event.tenant_id, "tenant_1");
        assert_eq!(
            event.data.get("users_updated").unwrap(),
            &serde_json::json!(100)
        );
        assert_eq!(
            event.data.get("entities_updated").unwrap(),
            &serde_json::json!(50)
        );
        assert_eq!(
            event.data.get("duration_ms").unwrap(),
            &serde_json::json!(1500)
        );
    }

    #[test]
    fn test_signature_generation() {
        let delivery = WebhookDelivery::with_defaults(
            vec!["http://example.com/webhook".to_string()],
            "test_secret".to_string(),
        );

        let payload = r#"{"test":"data"}"#;
        let signature = delivery.generate_signature(payload);

        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71); // "sha256=" + 64 hex chars
    }

    #[test]
    fn test_signature_verification() {
        let delivery = WebhookDelivery::with_defaults(
            vec!["http://example.com/webhook".to_string()],
            "test_secret".to_string(),
        );

        let payload = r#"{"test":"data"}"#;
        let signature = delivery.generate_signature(payload);

        assert!(delivery.verify_signature(payload, &signature));
        assert!(!delivery.verify_signature(payload, "invalid_signature"));
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(WebhookEventType::ModelUpdated.to_string(), "model_updated");
        assert_eq!(
            WebhookEventType::TrendingChanged.to_string(),
            "trending_changed"
        );
        assert_eq!(
            WebhookEventType::ErrorThresholdExceeded.to_string(),
            "error_threshold_exceeded"
        );
    }
}
