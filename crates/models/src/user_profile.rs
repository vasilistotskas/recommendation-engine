use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub preference_vector: Vec<f32>,
    pub interaction_count: i32,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub tenant_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_profile_creation() {
        let profile = UserProfile {
            user_id: "user_123".to_string(),
            preference_vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            interaction_count: 42,
            last_interaction_at: Some(Utc::now()),
            tenant_id: Some("tenant_a".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(profile.user_id, "user_123");
        assert_eq!(profile.preference_vector.len(), 5);
        assert_eq!(profile.interaction_count, 42);
        assert_eq!(profile.tenant_id, Some("tenant_a".to_string()));
        assert!(profile.last_interaction_at.is_some());
    }

    #[test]
    fn test_user_profile_without_interactions() {
        let profile = UserProfile {
            user_id: "user_456".to_string(),
            preference_vector: vec![],
            interaction_count: 0,
            last_interaction_at: None,
            tenant_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(profile.interaction_count, 0);
        assert!(profile.last_interaction_at.is_none());
        assert!(profile.preference_vector.is_empty());
        assert!(profile.tenant_id.is_none());
    }

    #[test]
    fn test_user_profile_clone() {
        let profile = UserProfile {
            user_id: "user_789".to_string(),
            preference_vector: vec![1.0, 2.0, 3.0],
            interaction_count: 10,
            last_interaction_at: Some(Utc::now()),
            tenant_id: Some("tenant_b".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let cloned = profile.clone();
        assert_eq!(cloned.user_id, profile.user_id);
        assert_eq!(cloned.preference_vector, profile.preference_vector);
        assert_eq!(cloned.interaction_count, profile.interaction_count);
        assert_eq!(cloned.tenant_id, profile.tenant_id);
    }

    #[test]
    fn test_user_profile_with_large_vector() {
        let large_vector: Vec<f32> = (0..512).map(|i| i as f32 / 512.0).collect();
        
        let profile = UserProfile {
            user_id: "user_999".to_string(),
            preference_vector: large_vector.clone(),
            interaction_count: 1000,
            last_interaction_at: Some(Utc::now()),
            tenant_id: Some("tenant_c".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(profile.preference_vector.len(), 512);
        assert_eq!(profile.preference_vector, large_vector);
    }

    #[test]
    fn test_user_profile_multi_tenant() {
        let profile_a = UserProfile {
            user_id: "user_shared".to_string(),
            preference_vector: vec![0.1, 0.2],
            interaction_count: 5,
            last_interaction_at: None,
            tenant_id: Some("tenant_a".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let profile_b = UserProfile {
            user_id: "user_shared".to_string(),
            preference_vector: vec![0.9, 0.8],
            interaction_count: 15,
            last_interaction_at: None,
            tenant_id: Some("tenant_b".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Same user_id but different tenants should have different profiles
        assert_eq!(profile_a.user_id, profile_b.user_id);
        assert_ne!(profile_a.tenant_id, profile_b.tenant_id);
        assert_ne!(profile_a.preference_vector, profile_b.preference_vector);
        assert_ne!(profile_a.interaction_count, profile_b.interaction_count);
    }
}
