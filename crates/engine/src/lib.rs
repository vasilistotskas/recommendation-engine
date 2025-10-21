pub mod collaborative;
pub mod content_based;
pub mod feature_extractor;
pub mod hybrid;

pub use collaborative::{CollaborativeConfig, CollaborativeFilteringEngine};
pub use content_based::{ContentBasedConfig, ContentBasedFilteringEngine};
pub use feature_extractor::FeatureExtractor;
pub use hybrid::{HybridConfig, HybridEngine};
