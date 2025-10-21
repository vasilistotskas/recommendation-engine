use crate::{AttributeValue, RecommendationError};
use std::collections::HashMap;

/// Trait for extracting feature vectors from entity attributes
pub trait FeatureExtractor: Send + Sync {
    /// Extract a feature vector from entity attributes
    fn extract_features(
        &self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> Result<Vec<f32>, RecommendationError>;

    /// Get the expected dimension of the feature vector
    fn feature_dimension(&self) -> usize;
}

/// Default implementation of feature extraction
/// Handles categorical, numerical, text, and nested attributes
#[derive(Debug, Clone)]
pub struct DefaultFeatureExtractor {
    dimension: usize,
}

impl DefaultFeatureExtractor {
    /// Create a new feature extractor with specified dimension
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Extract features from a single attribute value
    fn extract_attribute_features(
        &self,
        value: &AttributeValue,
        features: &mut Vec<f32>,
    ) -> Result<(), RecommendationError> {
        match value {
            AttributeValue::Number(n) => {
                // Min-max normalization to [0, 1] range
                // For now, we assume values are already in reasonable range
                // In production, you'd track min/max per attribute
                let normalized = n.clamp(0.0, 1.0);
                features.push(normalized as f32);
            }
            AttributeValue::Boolean(b) => {
                // Boolean as 0.0 or 1.0
                features.push(if *b { 1.0 } else { 0.0 });
            }
            AttributeValue::String(s) => {
                // Simple hash-based encoding for strings
                // In production, use proper one-hot encoding or embeddings
                let hash = self.hash_string(s);
                features.push(hash);
            }
            AttributeValue::Array(arr) => {
                // For arrays, compute average hash
                if arr.is_empty() {
                    features.push(0.0);
                } else {
                    let sum: f32 = arr.iter().map(|s| self.hash_string(s)).sum();
                    features.push(sum / arr.len() as f32);
                }
            }
        }
        Ok(())
    }

    /// Simple string hashing for categorical values
    /// Returns a value in [0, 1] range
    fn hash_string(&self, s: &str) -> f32 {
        let mut hash: u32 = 0;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        // Normalize to [0, 1]
        (hash % 1000) as f32 / 1000.0
    }

    /// Extract features from nested attributes (up to 3 levels)
    fn extract_nested_features(
        &self,
        attributes: &HashMap<String, AttributeValue>,
        features: &mut Vec<f32>,
        depth: usize,
    ) -> Result<(), RecommendationError> {
        if depth > 3 {
            return Ok(());
        }

        for (_key, value) in attributes.iter() {
            self.extract_attribute_features(value, features)?;
        }

        Ok(())
    }

    /// Normalize feature vector to unit length (L2 normalization)
    fn normalize_vector(&self, features: &mut [f32]) {
        let magnitude: f32 = features.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for feature in features.iter_mut() {
                *feature /= magnitude;
            }
        }
    }

    /// Pad or truncate vector to target dimension
    fn resize_vector(&self, features: &mut Vec<f32>) {
        if features.len() < self.dimension {
            // Pad with zeros
            features.resize(self.dimension, 0.0);
        } else if features.len() > self.dimension {
            // Truncate
            features.truncate(self.dimension);
        }
    }
}

impl FeatureExtractor for DefaultFeatureExtractor {
    fn extract_features(
        &self,
        attributes: &HashMap<String, AttributeValue>,
    ) -> Result<Vec<f32>, RecommendationError> {
        let mut features = Vec::new();

        // Extract features from all attributes
        self.extract_nested_features(attributes, &mut features, 0)?;

        // Resize to target dimension
        self.resize_vector(&mut features);

        // Normalize to unit length
        self.normalize_vector(&mut features);

        Ok(features)
    }

    fn feature_dimension(&self) -> usize {
        self.dimension
    }
}

/// One-hot encoder for categorical attributes
#[derive(Debug, Clone)]
pub struct OneHotEncoder {
    vocabulary: HashMap<String, usize>,
    dimension: usize,
}

impl OneHotEncoder {
    /// Create a new one-hot encoder with vocabulary
    pub fn new(vocabulary: Vec<String>) -> Self {
        let dimension = vocabulary.len();
        let vocabulary = vocabulary
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();

        Self {
            vocabulary,
            dimension,
        }
    }

    /// Encode a categorical value
    pub fn encode(&self, value: &str) -> Vec<f32> {
        let mut encoded = vec![0.0; self.dimension];
        if let Some(&index) = self.vocabulary.get(value) {
            encoded[index] = 1.0;
        }
        encoded
    }

    /// Encode multiple categorical values (multi-hot encoding)
    pub fn encode_multi(&self, values: &[String]) -> Vec<f32> {
        let mut encoded = vec![0.0; self.dimension];
        for value in values {
            if let Some(&index) = self.vocabulary.get(value) {
                encoded[index] = 1.0;
            }
        }
        encoded
    }
}

/// Min-max normalizer for numerical attributes
#[derive(Debug, Clone)]
pub struct MinMaxNormalizer {
    min: f64,
    max: f64,
}

impl MinMaxNormalizer {
    /// Create a new min-max normalizer
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// Normalize a value to [0, 1] range
    pub fn normalize(&self, value: f64) -> f32 {
        if self.max == self.min {
            return 0.5;
        }
        let normalized = (value - self.min) / (self.max - self.min);
        normalized.clamp(0.0, 1.0) as f32
    }
}

/// TF-IDF encoder for text attributes
#[derive(Debug, Clone)]
pub struct TfIdfEncoder {
    vocabulary: HashMap<String, usize>,
    idf: Vec<f32>,
    dimension: usize,
}

impl TfIdfEncoder {
    /// Create a new TF-IDF encoder
    pub fn new(vocabulary: Vec<String>, idf: Vec<f32>) -> Self {
        let dimension = vocabulary.len();
        let vocabulary = vocabulary
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, i))
            .collect();

        Self {
            vocabulary,
            idf,
            dimension,
        }
    }

    /// Encode text using TF-IDF
    pub fn encode(&self, text: &str) -> Vec<f32> {
        let mut encoded = vec![0.0; self.dimension];
        
        // Simple tokenization (split by whitespace and lowercase)
        let tokens: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if tokens.is_empty() {
            return encoded;
        }

        // Count term frequencies
        let mut tf: HashMap<String, f32> = HashMap::new();
        for token in &tokens {
            *tf.entry(token.clone()).or_insert(0.0) += 1.0;
        }

        // Normalize TF by document length
        let doc_length = tokens.len() as f32;
        for count in tf.values_mut() {
            *count /= doc_length;
        }

        // Compute TF-IDF
        for (term, tf_value) in tf {
            if let Some(&index) = self.vocabulary.get(&term)
                && index < self.idf.len()
            {
                encoded[index] = tf_value * self.idf[index];
            }
        }

        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_feature_extractor() {
        let extractor = DefaultFeatureExtractor::new(512);
        
        let mut attributes = HashMap::new();
        attributes.insert("price".to_string(), AttributeValue::Number(99.99));
        attributes.insert("category".to_string(), AttributeValue::String("electronics".to_string()));
        attributes.insert("in_stock".to_string(), AttributeValue::Boolean(true));
        attributes.insert("tags".to_string(), AttributeValue::Array(vec!["new".to_string(), "sale".to_string()]));

        let features = extractor.extract_features(&attributes).unwrap();
        
        assert_eq!(features.len(), 512);
        // Check that vector is normalized (magnitude should be ~1.0)
        let magnitude: f32 = features.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_one_hot_encoder() {
        let encoder = OneHotEncoder::new(vec![
            "red".to_string(),
            "green".to_string(),
            "blue".to_string(),
        ]);

        let encoded = encoder.encode("green");
        assert_eq!(encoded, vec![0.0, 1.0, 0.0]);

        let multi = encoder.encode_multi(&["red".to_string(), "blue".to_string()]);
        assert_eq!(multi, vec![1.0, 0.0, 1.0]);
    }

    #[test]
    fn test_min_max_normalizer() {
        let normalizer = MinMaxNormalizer::new(0.0, 100.0);
        
        assert_eq!(normalizer.normalize(0.0), 0.0);
        assert_eq!(normalizer.normalize(50.0), 0.5);
        assert_eq!(normalizer.normalize(100.0), 1.0);
        assert_eq!(normalizer.normalize(150.0), 1.0); // Clamped
    }

    #[test]
    fn test_tfidf_encoder() {
        let vocabulary = vec!["hello".to_string(), "world".to_string(), "rust".to_string()];
        let idf = vec![1.5, 2.0, 1.0];
        let encoder = TfIdfEncoder::new(vocabulary, idf);

        let encoded = encoder.encode("hello world hello");
        assert!(encoded[0] > 0.0); // "hello" appears
        assert!(encoded[1] > 0.0); // "world" appears
        assert_eq!(encoded[2], 0.0); // "rust" doesn't appear
    }
}
