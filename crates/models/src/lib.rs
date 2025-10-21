pub mod entity;
pub mod error;
pub mod feature_extractor;
pub mod interaction;
pub mod interaction_type_registry;
pub mod recommendation;
pub mod tenant;
pub mod user_profile;

pub use entity::{AttributeValue, Entity};
pub use error::{ErrorDetail, ErrorResponse, RecommendationError, Result};
pub use feature_extractor::{
    DefaultFeatureExtractor, FeatureExtractor, MinMaxNormalizer, OneHotEncoder, TfIdfEncoder,
};
pub use interaction::{Interaction, InteractionType};
pub use interaction_type_registry::{
    ListInteractionTypesResponse, RegisterInteractionTypeRequest, RegisteredInteractionType,
    UpdateInteractionTypeRequest,
};
pub use recommendation::{Algorithm, RecommendationRequest, RecommendationResponse, ScoredEntity};
pub use tenant::TenantContext;
pub use user_profile::UserProfile;
