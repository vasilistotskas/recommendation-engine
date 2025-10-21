use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};
use crate::config::{AlgorithmConfig, CacheConfig, ColdStartConfig};
use crate::error::{ConfigError, Result};

/// Per-tenant configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub tenant_id: String,
    
    /// Algorithm configuration overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithms: Option<AlgorithmConfig>,
    
    /// Cache configuration overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheConfig>,
    
    /// Cold start configuration overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_start: Option<ColdStartConfig>,
    
    /// Custom interaction type weights
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_weights: Option<HashMap<String, f32>>,
}

impl TenantConfig {
    /// Create a new tenant configuration
    pub fn new(tenant_id: String) -> Self {
        Self {
            tenant_id,
            algorithms: None,
            cache: None,
            cold_start: None,
            interaction_weights: None,
        }
    }

    /// Validate tenant configuration
    pub fn validate(&self) -> Result<()> {
        // Validate algorithm overrides if present
        if let Some(ref algo) = self.algorithms {
            let weight_sum = algo.collaborative_weight + algo.content_based_weight;
            if (weight_sum - 1.0).abs() > 0.001 {
                return Err(ConfigError::ValidationError(
                    format!(
                        "Tenant {} algorithm weights must sum to 1.0, got {}",
                        self.tenant_id, weight_sum
                    ),
                ));
            }
        }

        // Validate interaction weights if present
        if let Some(ref weights) = self.interaction_weights {
            for (interaction_type, weight) in weights {
                if *weight < 0.0 {
                    return Err(ConfigError::ValidationError(
                        format!(
                            "Tenant {} interaction weight for '{}' must be non-negative, got {}",
                            self.tenant_id, interaction_type, weight
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get interaction weight for a specific type, or None if not overridden
    pub fn get_interaction_weight(&self, interaction_type: &str) -> Option<f32> {
        self.interaction_weights
            .as_ref()
            .and_then(|weights| weights.get(interaction_type).copied())
    }
}

/// Manager for tenant-specific configurations
pub struct TenantConfigManager {
    /// Map of tenant_id to tenant configuration
    tenants: Arc<RwLock<HashMap<String, TenantConfig>>>,
}

impl TenantConfigManager {
    /// Create a new tenant configuration manager
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load tenant configurations from a file
    pub fn load_from_file(&self, path: &str) -> Result<()> {
        info!("Loading tenant configurations from: {}", path);

        let content = std::fs::read_to_string(path)
            .map_err(ConfigError::IoError)?;

        let tenant_configs: HashMap<String, TenantConfig> = if path.ends_with(".json") {
            serde_json::from_str(&content)
                .map_err(ConfigError::SerializationError)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::LoadError(format!("YAML parse error: {}", e)))?
        } else if path.ends_with(".toml") {
            toml::from_str(&content)
                .map_err(|e| ConfigError::LoadError(format!("TOML parse error: {}", e)))?
        } else {
            return Err(ConfigError::LoadError(
                format!("Unsupported file format: {}", path)
            ));
        };

        // Validate all tenant configurations
        for (tenant_id, config) in &tenant_configs {
            config.validate()
                .map_err(|e| ConfigError::ValidationError(
                    format!("Invalid configuration for tenant {}: {}", tenant_id, e)
                ))?;
        }

        // Store configurations
        let mut tenants = self.tenants.write().unwrap();
        *tenants = tenant_configs;

        info!("Loaded {} tenant configurations", tenants.len());

        Ok(())
    }

    /// Register a tenant configuration
    pub fn register_tenant(&self, config: TenantConfig) -> Result<()> {
        debug!("Registering tenant configuration: {}", config.tenant_id);

        // Validate configuration
        config.validate()?;

        let mut tenants = self.tenants.write().unwrap();
        tenants.insert(config.tenant_id.clone(), config);

        Ok(())
    }

    /// Get tenant configuration
    pub fn get_tenant(&self, tenant_id: &str) -> Option<TenantConfig> {
        let tenants = self.tenants.read().unwrap();
        tenants.get(tenant_id).cloned()
    }

    /// Check if a tenant exists
    pub fn has_tenant(&self, tenant_id: &str) -> bool {
        let tenants = self.tenants.read().unwrap();
        tenants.contains_key(tenant_id)
    }

    /// Remove a tenant configuration
    pub fn remove_tenant(&self, tenant_id: &str) -> bool {
        let mut tenants = self.tenants.write().unwrap();
        tenants.remove(tenant_id).is_some()
    }

    /// Get all tenant IDs
    pub fn list_tenants(&self) -> Vec<String> {
        let tenants = self.tenants.read().unwrap();
        tenants.keys().cloned().collect()
    }

    /// Get count of registered tenants
    pub fn tenant_count(&self) -> usize {
        let tenants = self.tenants.read().unwrap();
        tenants.len()
    }

    /// Clear all tenant configurations
    pub fn clear(&self) {
        let mut tenants = self.tenants.write().unwrap();
        tenants.clear();
    }

    /// Get algorithm configuration for a tenant, falling back to default
    pub fn get_algorithm_config(
        &self,
        tenant_id: &str,
        default: &AlgorithmConfig,
    ) -> AlgorithmConfig {
        self.get_tenant(tenant_id)
            .and_then(|t| t.algorithms)
            .unwrap_or_else(|| default.clone())
    }

    /// Get cache configuration for a tenant, falling back to default
    pub fn get_cache_config(
        &self,
        tenant_id: &str,
        default: &CacheConfig,
    ) -> CacheConfig {
        self.get_tenant(tenant_id)
            .and_then(|t| t.cache)
            .unwrap_or_else(|| default.clone())
    }

    /// Get cold start configuration for a tenant, falling back to default
    pub fn get_cold_start_config(
        &self,
        tenant_id: &str,
        default: &ColdStartConfig,
    ) -> ColdStartConfig {
        self.get_tenant(tenant_id)
            .and_then(|t| t.cold_start)
            .unwrap_or_else(|| default.clone())
    }

    /// Get interaction weight for a tenant and interaction type
    /// Returns None if no override is configured
    pub fn get_interaction_weight(
        &self,
        tenant_id: &str,
        interaction_type: &str,
    ) -> Option<f32> {
        self.get_tenant(tenant_id)
            .and_then(|t| t.get_interaction_weight(interaction_type))
    }
}

impl Default for TenantConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_config_creation() {
        let config = TenantConfig::new("tenant_a".to_string());
        assert_eq!(config.tenant_id, "tenant_a");
        assert!(config.algorithms.is_none());
        assert!(config.cache.is_none());
    }

    #[test]
    fn test_tenant_config_validation() {
        let mut config = TenantConfig::new("tenant_a".to_string());
        
        // Valid configuration
        config.algorithms = Some(AlgorithmConfig {
            collaborative_weight: 0.7,
            content_based_weight: 0.3,
            ..Default::default()
        });
        assert!(config.validate().is_ok());

        // Invalid weights
        config.algorithms = Some(AlgorithmConfig {
            collaborative_weight: 0.7,
            content_based_weight: 0.5,
            ..Default::default()
        });
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tenant_config_interaction_weights() {
        let mut config = TenantConfig::new("tenant_a".to_string());
        
        let mut weights = HashMap::new();
        weights.insert("view".to_string(), 1.0);
        weights.insert("purchase".to_string(), 5.0);
        config.interaction_weights = Some(weights);

        assert_eq!(config.get_interaction_weight("view"), Some(1.0));
        assert_eq!(config.get_interaction_weight("purchase"), Some(5.0));
        assert_eq!(config.get_interaction_weight("like"), None);
    }

    #[test]
    fn test_tenant_config_manager() {
        let manager = TenantConfigManager::new();
        
        let config = TenantConfig::new("tenant_a".to_string());
        manager.register_tenant(config).unwrap();

        assert!(manager.has_tenant("tenant_a"));
        assert!(!manager.has_tenant("tenant_b"));
        assert_eq!(manager.tenant_count(), 1);

        let retrieved = manager.get_tenant("tenant_a").unwrap();
        assert_eq!(retrieved.tenant_id, "tenant_a");
    }

    #[test]
    fn test_tenant_config_manager_remove() {
        let manager = TenantConfigManager::new();
        
        let config = TenantConfig::new("tenant_a".to_string());
        manager.register_tenant(config).unwrap();

        assert!(manager.has_tenant("tenant_a"));
        assert!(manager.remove_tenant("tenant_a"));
        assert!(!manager.has_tenant("tenant_a"));
        assert!(!manager.remove_tenant("tenant_a"));
    }

    #[test]
    fn test_tenant_config_manager_list() {
        let manager = TenantConfigManager::new();
        
        manager.register_tenant(TenantConfig::new("tenant_a".to_string())).unwrap();
        manager.register_tenant(TenantConfig::new("tenant_b".to_string())).unwrap();
        manager.register_tenant(TenantConfig::new("tenant_c".to_string())).unwrap();

        let tenants = manager.list_tenants();
        assert_eq!(tenants.len(), 3);
        assert!(tenants.contains(&"tenant_a".to_string()));
        assert!(tenants.contains(&"tenant_b".to_string()));
        assert!(tenants.contains(&"tenant_c".to_string()));
    }

    #[test]
    fn test_tenant_config_manager_clear() {
        let manager = TenantConfigManager::new();
        
        manager.register_tenant(TenantConfig::new("tenant_a".to_string())).unwrap();
        manager.register_tenant(TenantConfig::new("tenant_b".to_string())).unwrap();

        assert_eq!(manager.tenant_count(), 2);
        manager.clear();
        assert_eq!(manager.tenant_count(), 0);
    }

    #[test]
    fn test_tenant_config_manager_fallback() {
        let manager = TenantConfigManager::new();
        let default_algo = AlgorithmConfig::default();

        // No tenant registered - should return default
        let algo = manager.get_algorithm_config("tenant_a", &default_algo);
        assert_eq!(algo.collaborative_weight, default_algo.collaborative_weight);

        // Register tenant with override
        let mut config = TenantConfig::new("tenant_a".to_string());
        config.algorithms = Some(AlgorithmConfig {
            collaborative_weight: 0.8,
            content_based_weight: 0.2,
            ..Default::default()
        });
        manager.register_tenant(config).unwrap();

        // Should return tenant override
        let algo = manager.get_algorithm_config("tenant_a", &default_algo);
        assert_eq!(algo.collaborative_weight, 0.8);
    }

    #[test]
    fn test_tenant_config_manager_interaction_weight() {
        let manager = TenantConfigManager::new();

        let mut config = TenantConfig::new("tenant_a".to_string());
        let mut weights = HashMap::new();
        weights.insert("view".to_string(), 2.0);
        config.interaction_weights = Some(weights);
        manager.register_tenant(config).unwrap();

        assert_eq!(manager.get_interaction_weight("tenant_a", "view"), Some(2.0));
        assert_eq!(manager.get_interaction_weight("tenant_a", "like"), None);
        assert_eq!(manager.get_interaction_weight("tenant_b", "view"), None);
    }
}
