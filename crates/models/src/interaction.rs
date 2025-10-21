use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub user_id: String,
    pub entity_id: String,
    pub interaction_type: InteractionType,
    pub weight: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionType {
    View,
    AddToCart,
    Purchase,
    Like,
    Rating(f32),
    Custom(String),
}

impl InteractionType {
    pub fn default_weight(&self) -> f32 {
        match self {
            InteractionType::View => 1.0,
            InteractionType::AddToCart => 3.0,
            InteractionType::Purchase => 5.0,
            InteractionType::Like => 2.0,
            InteractionType::Rating(rating) => *rating,
            InteractionType::Custom(_) => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_creation() {
        let interaction = Interaction {
            id: Some(1),
            user_id: "user_123".to_string(),
            entity_id: "prod_456".to_string(),
            interaction_type: InteractionType::Purchase,
            weight: 5.0,
            metadata: None,
            tenant_id: Some("tenant_a".to_string()),
            timestamp: Utc::now(),
        };

        assert_eq!(interaction.user_id, "user_123");
        assert_eq!(interaction.entity_id, "prod_456");
        assert_eq!(interaction.weight, 5.0);
        assert_eq!(interaction.tenant_id, Some("tenant_a".to_string()));
    }

    #[test]
    fn test_interaction_type_default_weights() {
        assert_eq!(InteractionType::View.default_weight(), 1.0);
        assert_eq!(InteractionType::AddToCart.default_weight(), 3.0);
        assert_eq!(InteractionType::Purchase.default_weight(), 5.0);
        assert_eq!(InteractionType::Like.default_weight(), 2.0);
        assert_eq!(InteractionType::Rating(4.5).default_weight(), 4.5);
        assert_eq!(InteractionType::Custom("share".to_string()).default_weight(), 1.0);
    }

    #[test]
    fn test_interaction_serialization() {
        let interaction = Interaction {
            id: None,
            user_id: "user_789".to_string(),
            entity_id: "prod_101".to_string(),
            interaction_type: InteractionType::Like,
            weight: 2.0,
            metadata: Some(HashMap::from([
                ("source".to_string(), "mobile_app".to_string()),
            ])),
            tenant_id: Some("tenant_b".to_string()),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&interaction).unwrap();
        assert!(json.contains("user_789"));
        assert!(json.contains("prod_101"));
        assert!(json.contains("like"));
        assert!(json.contains("mobile_app"));
        assert!(json.contains("tenant_b"));
    }

    #[test]
    fn test_interaction_deserialization() {
        let json = r#"{
            "user_id": "user_456",
            "entity_id": "prod_789",
            "interaction_type": "purchase",
            "weight": 5.0,
            "tenant_id": "tenant_c",
            "timestamp": "2025-10-20T10:00:00Z"
        }"#;

        let interaction: Interaction = serde_json::from_str(json).unwrap();
        assert_eq!(interaction.user_id, "user_456");
        assert_eq!(interaction.entity_id, "prod_789");
        assert_eq!(interaction.weight, 5.0);
        assert_eq!(interaction.tenant_id, Some("tenant_c".to_string()));
    }

    #[test]
    fn test_interaction_type_rating_serialization() {
        let interaction = Interaction {
            id: None,
            user_id: "user_111".to_string(),
            entity_id: "prod_222".to_string(),
            interaction_type: InteractionType::Rating(4.5),
            weight: 4.5,
            metadata: None,
            tenant_id: None,
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&interaction).unwrap();
        assert!(json.contains("4.5"));
    }

    #[test]
    fn test_interaction_type_custom_serialization() {
        let interaction = Interaction {
            id: None,
            user_id: "user_333".to_string(),
            entity_id: "prod_444".to_string(),
            interaction_type: InteractionType::Custom("share".to_string()),
            weight: 3.0,
            metadata: None,
            tenant_id: None,
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&interaction).unwrap();
        assert!(json.contains("share"));
    }

    #[test]
    fn test_interaction_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("device".to_string(), "ios".to_string());
        metadata.insert("version".to_string(), "1.2.3".to_string());

        let interaction = Interaction {
            id: Some(42),
            user_id: "user_555".to_string(),
            entity_id: "prod_666".to_string(),
            interaction_type: InteractionType::View,
            weight: 1.0,
            metadata: Some(metadata),
            tenant_id: Some("tenant_d".to_string()),
            timestamp: Utc::now(),
        };

        assert!(interaction.metadata.is_some());
        let meta = interaction.metadata.unwrap();
        assert_eq!(meta.get("device"), Some(&"ios".to_string()));
        assert_eq!(meta.get("version"), Some(&"1.2.3".to_string()));
    }
}
