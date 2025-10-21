use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendationRequest {
    pub user_id: Option<String>,
    pub entity_id: Option<String>,
    pub algorithm: Algorithm,
    #[serde(default = "default_count")]
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<HashMap<String, String>>,
}

fn default_count() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    Collaborative,
    ContentBased,
    Hybrid {
        collaborative_weight: f32,
        content_weight: f32,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecommendationResponse {
    pub recommendations: Vec<ScoredEntity>,
    pub algorithm: String,
    pub cold_start: bool,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoredEntity {
    pub entity_id: String,
    pub entity_type: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_request_creation() {
        let request = RecommendationRequest {
            user_id: Some("user_123".to_string()),
            entity_id: None,
            algorithm: Algorithm::Collaborative,
            count: 10,
            filters: None,
        };

        assert_eq!(request.user_id, Some("user_123".to_string()));
        assert_eq!(request.count, 10);
        assert!(request.filters.is_none());
    }

    #[test]
    fn test_recommendation_request_deserialization() {
        let json = r#"{
            "user_id": "user_456",
            "algorithm": "collaborative"
        }"#;

        let request: RecommendationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.user_id, Some("user_456".to_string()));
        assert_eq!(request.count, 10); // Default value
    }

    #[test]
    fn test_recommendation_request_with_filters() {
        let mut filters = HashMap::new();
        filters.insert("category".to_string(), "electronics".to_string());
        filters.insert("price_max".to_string(), "1000".to_string());

        let request = RecommendationRequest {
            user_id: Some("user_789".to_string()),
            entity_id: None,
            algorithm: Algorithm::ContentBased,
            count: 20,
            filters: Some(filters),
        };

        assert!(request.filters.is_some());
        let f = request.filters.unwrap();
        assert_eq!(f.get("category"), Some(&"electronics".to_string()));
    }

    #[test]
    fn test_algorithm_collaborative() {
        let algo = Algorithm::Collaborative;
        let json = serde_json::to_string(&algo).unwrap();
        assert_eq!(json, r#""collaborative""#);
    }

    #[test]
    fn test_algorithm_content_based() {
        let algo = Algorithm::ContentBased;
        let json = serde_json::to_string(&algo).unwrap();
        assert_eq!(json, r#""content_based""#);
    }

    #[test]
    fn test_algorithm_hybrid() {
        let algo = Algorithm::Hybrid {
            collaborative_weight: 0.7,
            content_weight: 0.3,
        };
        let json = serde_json::to_string(&algo).unwrap();
        assert!(json.contains("collaborative_weight"));
        assert!(json.contains("0.7"));
        assert!(json.contains("0.3"));
    }

    #[test]
    fn test_algorithm_deserialization() {
        let json = r#"{"hybrid":{"collaborative_weight":0.6,"content_weight":0.4}}"#;
        let algo: Algorithm = serde_json::from_str(json).unwrap();

        match algo {
            Algorithm::Hybrid {
                collaborative_weight,
                content_weight,
            } => {
                assert_eq!(collaborative_weight, 0.6);
                assert_eq!(content_weight, 0.4);
            }
            _ => panic!("Expected Hybrid algorithm"),
        }
    }

    #[test]
    fn test_scored_entity_creation() {
        let entity = ScoredEntity {
            entity_id: "prod_123".to_string(),
            entity_type: "product".to_string(),
            score: 0.95,
            reason: Some("High similarity".to_string()),
        };

        assert_eq!(entity.entity_id, "prod_123");
        assert_eq!(entity.score, 0.95);
        assert_eq!(entity.reason, Some("High similarity".to_string()));
    }

    #[test]
    fn test_scored_entity_serialization() {
        let entity = ScoredEntity {
            entity_id: "prod_456".to_string(),
            entity_type: "product".to_string(),
            score: 0.87,
            reason: None,
        };

        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("prod_456"));
        assert!(json.contains("0.87"));
        assert!(!json.contains("reason")); // Should be omitted when None
    }

    #[test]
    fn test_recommendation_response_creation() {
        let entities = vec![
            ScoredEntity {
                entity_id: "prod_1".to_string(),
                entity_type: "product".to_string(),
                score: 0.95,
                reason: None,
            },
            ScoredEntity {
                entity_id: "prod_2".to_string(),
                entity_type: "product".to_string(),
                score: 0.87,
                reason: None,
            },
        ];

        let response = RecommendationResponse {
            recommendations: entities,
            algorithm: "hybrid".to_string(),
            cold_start: false,
            generated_at: Utc::now(),
        };

        assert_eq!(response.recommendations.len(), 2);
        assert_eq!(response.algorithm, "hybrid");
        assert!(!response.cold_start);
    }

    #[test]
    fn test_recommendation_response_serialization() {
        let response = RecommendationResponse {
            recommendations: vec![ScoredEntity {
                entity_id: "prod_789".to_string(),
                entity_type: "product".to_string(),
                score: 0.92,
                reason: Some("Popular item".to_string()),
            }],
            algorithm: "collaborative".to_string(),
            cold_start: true,
            generated_at: Utc::now(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("prod_789"));
        assert!(json.contains("0.92"));
        assert!(json.contains("collaborative"));
        assert!(json.contains("true")); // cold_start
        assert!(json.contains("Popular item"));
    }

    #[test]
    fn test_recommendation_request_entity_based() {
        let request = RecommendationRequest {
            user_id: None,
            entity_id: Some("prod_999".to_string()),
            algorithm: Algorithm::ContentBased,
            count: 5,
            filters: None,
        };

        assert!(request.user_id.is_none());
        assert_eq!(request.entity_id, Some("prod_999".to_string()));
    }

    #[test]
    fn test_default_count_function() {
        assert_eq!(default_count(), 10);
    }
}
