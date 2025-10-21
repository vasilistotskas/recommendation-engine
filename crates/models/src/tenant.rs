use serde::{Deserialize, Serialize};

/// Context for multi-tenant operations
/// Provides tenant isolation across all data operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantContext {
    pub tenant_id: String,
}

impl TenantContext {
    /// Create a new tenant context
    pub fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
        }
    }

    /// Get the default tenant context
    pub fn default_tenant() -> Self {
        Self {
            tenant_id: "default".to_string(),
        }
    }

    /// Check if this is the default tenant
    pub fn is_default(&self) -> bool {
        self.tenant_id == "default"
    }
}

impl Default for TenantContext {
    fn default() -> Self {
        Self::default_tenant()
    }
}

impl From<String> for TenantContext {
    fn from(tenant_id: String) -> Self {
        Self::new(tenant_id)
    }
}

impl From<&str> for TenantContext {
    fn from(tenant_id: &str) -> Self {
        Self::new(tenant_id)
    }
}

impl From<Option<String>> for TenantContext {
    fn from(tenant_id: Option<String>) -> Self {
        tenant_id.map(Self::new).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_context_new() {
        let ctx = TenantContext::new("tenant_123");
        assert_eq!(ctx.tenant_id, "tenant_123");
    }

    #[test]
    fn test_tenant_context_default() {
        let ctx = TenantContext::default();
        assert_eq!(ctx.tenant_id, "default");
        assert!(ctx.is_default());
    }

    #[test]
    fn test_tenant_context_default_tenant() {
        let ctx = TenantContext::default_tenant();
        assert_eq!(ctx.tenant_id, "default");
        assert!(ctx.is_default());
    }

    #[test]
    fn test_tenant_context_is_default() {
        let default_ctx = TenantContext::new("default");
        let custom_ctx = TenantContext::new("tenant_a");

        assert!(default_ctx.is_default());
        assert!(!custom_ctx.is_default());
    }

    #[test]
    fn test_tenant_context_from_string() {
        let ctx: TenantContext = "tenant_456".to_string().into();
        assert_eq!(ctx.tenant_id, "tenant_456");
    }

    #[test]
    fn test_tenant_context_from_str() {
        let ctx: TenantContext = "tenant_789".into();
        assert_eq!(ctx.tenant_id, "tenant_789");
    }

    #[test]
    fn test_tenant_context_from_option_some() {
        let ctx: TenantContext = Some("tenant_abc".to_string()).into();
        assert_eq!(ctx.tenant_id, "tenant_abc");
    }

    #[test]
    fn test_tenant_context_from_option_none() {
        let ctx: TenantContext = None.into();
        assert_eq!(ctx.tenant_id, "default");
        assert!(ctx.is_default());
    }

    #[test]
    fn test_tenant_context_equality() {
        let ctx1 = TenantContext::new("tenant_a");
        let ctx2 = TenantContext::new("tenant_a");
        let ctx3 = TenantContext::new("tenant_b");

        assert_eq!(ctx1, ctx2);
        assert_ne!(ctx1, ctx3);
    }

    #[test]
    fn test_tenant_context_clone() {
        let ctx = TenantContext::new("tenant_clone");
        let cloned = ctx.clone();

        assert_eq!(ctx, cloned);
        assert_eq!(ctx.tenant_id, cloned.tenant_id);
    }

    #[test]
    fn test_tenant_context_serialization() {
        let ctx = TenantContext::new("tenant_serialize");
        let json = serde_json::to_string(&ctx).unwrap();

        assert!(json.contains("tenant_serialize"));
        assert!(json.contains("tenant_id"));
    }

    #[test]
    fn test_tenant_context_deserialization() {
        let json = r#"{"tenant_id":"tenant_deserialize"}"#;
        let ctx: TenantContext = serde_json::from_str(json).unwrap();

        assert_eq!(ctx.tenant_id, "tenant_deserialize");
    }

    #[test]
    fn test_tenant_context_round_trip() {
        let original = TenantContext::new("tenant_roundtrip");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TenantContext = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }
}
