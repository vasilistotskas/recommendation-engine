pub mod entity;
pub mod interaction;
pub mod user_profile;
pub mod recommendation;
pub mod error;
pub mod tenant;
pub mod feature_extractor;
pub mod interaction_type_registry;

pub use entity::{Entity, AttributeValue};
pub use interaction::{Interaction, InteractionType};
pub use user_profile::UserProfile;
pub use recommendation::{RecommendationRequest, RecommendationResponse, ScoredEntity, Algorithm};
pub use error::{RecommendationError, ErrorResponse, ErrorDetail, Result};
pub use tenant::TenantContext;
pub use feature_extractor::{
    FeatureExtractor, DefaultFeatureExtractor, OneHotEncoder, MinMaxNormalizer, TfIdfEncoder,
};
pub use interaction_type_registry::{
    RegisteredInteractionType, RegisterInteractionTypeRequest, UpdateInteractionTypeRequest,
    ListInteractionTypesResponse,
};
