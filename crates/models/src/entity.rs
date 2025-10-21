use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_id: String,
    pub entity_type: String,
    pub attributes: HashMap<String, AttributeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_vector: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "name".to_string(),
            AttributeValue::String("Test Product".to_string()),
        );
        attributes.insert("price".to_string(), AttributeValue::Number(99.99));
        attributes.insert("in_stock".to_string(), AttributeValue::Boolean(true));
        attributes.insert(
            "tags".to_string(),
            AttributeValue::Array(vec!["new".to_string(), "sale".to_string()]),
        );

        let entity = Entity {
            entity_id: "prod_123".to_string(),
            entity_type: "product".to_string(),
            attributes,
            feature_vector: Some(vec![0.1, 0.2, 0.3]),
            tenant_id: Some("tenant_a".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(entity.entity_id, "prod_123");
        assert_eq!(entity.entity_type, "product");
        assert_eq!(entity.tenant_id, Some("tenant_a".to_string()));
        assert_eq!(entity.feature_vector, Some(vec![0.1, 0.2, 0.3]));
    }

    #[test]
    fn test_entity_serialization() {
        let mut attributes = HashMap::new();
        attributes.insert(
            "name".to_string(),
            AttributeValue::String("Laptop".to_string()),
        );
        attributes.insert("price".to_string(), AttributeValue::Number(1299.99));

        let entity = Entity {
            entity_id: "prod_456".to_string(),
            entity_type: "product".to_string(),
            attributes,
            feature_vector: None,
            tenant_id: Some("tenant_b".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("prod_456"));
        assert!(json.contains("Laptop"));
        assert!(json.contains("1299.99"));
        assert!(json.contains("tenant_b"));
    }

    #[test]
    fn test_entity_deserialization() {
        let json = r#"{
            "entity_id": "prod_789",
            "entity_type": "product",
            "attributes": {
                "name": "Phone",
                "price": 699.99,
                "in_stock": true,
                "tags": ["electronics", "mobile"]
            },
            "tenant_id": "tenant_c",
            "created_at": "2025-10-20T10:00:00Z",
            "updated_at": "2025-10-20T10:00:00Z"
        }"#;

        let entity: Entity = serde_json::from_str(json).unwrap();
        assert_eq!(entity.entity_id, "prod_789");
        assert_eq!(entity.entity_type, "product");
        assert_eq!(entity.tenant_id, Some("tenant_c".to_string()));
        assert!(entity.attributes.contains_key("name"));
    }

    #[test]
    fn test_attribute_value_variants() {
        let string_val = AttributeValue::String("test".to_string());
        let number_val = AttributeValue::Number(42.5);
        let bool_val = AttributeValue::Boolean(true);
        let array_val = AttributeValue::Array(vec!["a".to_string(), "b".to_string()]);

        // Test serialization
        assert!(serde_json::to_string(&string_val).unwrap().contains("test"));
        assert!(serde_json::to_string(&number_val).unwrap().contains("42.5"));
        assert!(serde_json::to_string(&bool_val).unwrap().contains("true"));
        assert!(serde_json::to_string(&array_val).unwrap().contains("a"));
    }

    #[test]
    fn test_entity_without_tenant() {
        let entity = Entity {
            entity_id: "prod_999".to_string(),
            entity_type: "product".to_string(),
            attributes: HashMap::new(),
            feature_vector: None,
            tenant_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&entity).unwrap();
        // tenant_id should be omitted when None
        assert!(!json.contains("tenant_id"));
    }
}
