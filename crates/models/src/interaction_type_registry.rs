use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Registered custom interaction type with weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredInteractionType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub tenant_id: String,
    pub interaction_type: String,
    pub weight: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to register a new interaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterInteractionTypeRequest {
    pub interaction_type: String,
    pub weight: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// Request to update an existing interaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInteractionTypeRequest {
    pub weight: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response containing list of registered interaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInteractionTypesResponse {
    pub interaction_types: Vec<RegisteredInteractionType>,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registered_interaction_type_creation() {
        let interaction_type = RegisteredInteractionType {
            id: Some(1),
            tenant_id: "tenant_a".to_string(),
            interaction_type: "share".to_string(),
            weight: 3.5,
            description: Some("User shared entity".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(interaction_type.interaction_type, "share");
        assert_eq!(interaction_type.weight, 3.5);
        assert_eq!(interaction_type.tenant_id, "tenant_a");
    }

    #[test]
    fn test_register_interaction_type_request_serialization() {
        let request = RegisterInteractionTypeRequest {
            interaction_type: "bookmark".to_string(),
            weight: 2.5,
            description: Some("User bookmarked entity".to_string()),
            tenant_id: Some("tenant_b".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("bookmark"));
        assert!(json.contains("2.5"));
        assert!(json.contains("tenant_b"));
    }

    #[test]
    fn test_register_interaction_type_request_deserialization() {
        let json = r#"{
            "interaction_type": "download",
            "weight": 4.0,
            "description": "User downloaded entity"
        }"#;

        let request: RegisterInteractionTypeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.interaction_type, "download");
        assert_eq!(request.weight, 4.0);
        assert_eq!(request.description, Some("User downloaded entity".to_string()));
        assert_eq!(request.tenant_id, None);
    }

    #[test]
    fn test_update_interaction_type_request() {
        let request = UpdateInteractionTypeRequest {
            weight: 5.5,
            description: Some("Updated description".to_string()),
        };

        assert_eq!(request.weight, 5.5);
        assert_eq!(request.description, Some("Updated description".to_string()));
    }

    #[test]
    fn test_list_interaction_types_response() {
        let response = ListInteractionTypesResponse {
            interaction_types: vec![
                RegisteredInteractionType {
                    id: Some(1),
                    tenant_id: "default".to_string(),
                    interaction_type: "view".to_string(),
                    weight: 1.0,
                    description: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            ],
            total: 1,
        };

        assert_eq!(response.total, 1);
        assert_eq!(response.interaction_types.len(), 1);
    }
}
