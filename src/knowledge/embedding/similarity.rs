//! Similarity metrics for vector comparison.

use crate::knowledge::embedding::vector::DenseVector;
use serde::{Deserialize, Serialize};

/// An enumeration of available similarity metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimilarityMetricType {
    /// Cosine similarity (dot product for normalized vectors)
    Cosine,
    /// Dot product without normalization
    DotProduct,
    /// Euclidean distance (lower is more similar, inverted to similarity range)
    EuclideanDistance,
}

/// Trait for computing similarity between two vectors.
pub trait SimilarityMetric: Send + Sync {
    /// Compute similarity between two vectors.
    /// Should return a value where higher means more similar.
    fn similarity(&self, a: &DenseVector, b: &DenseVector) -> f32;

    /// Get the metric type.
    fn metric_type(&self) -> SimilarityMetricType;
}

/// Cosine similarity metric (works best with normalized vectors).
#[derive(Debug, Clone)]
pub struct CosineSimilarity;

impl SimilarityMetric for CosineSimilarity {
    fn similarity(&self, a: &DenseVector, b: &DenseVector) -> f32 {
        a.cosine_similarity(b)
    }

    fn metric_type(&self) -> SimilarityMetricType {
        SimilarityMetricType::Cosine
    }
}

/// Dot product metric (can be used with or without normalized vectors).
#[derive(Debug, Clone)]
pub struct DotProductSimilarity;

impl SimilarityMetric for DotProductSimilarity {
    fn similarity(&self, a: &DenseVector, b: &DenseVector) -> f32 {
        a.dot_product(b)
    }

    fn metric_type(&self) -> SimilarityMetricType {
        SimilarityMetricType::DotProduct
    }
}

/// Euclidean distance metric (inverted so higher similarity means closer proximity).
#[derive(Debug, Clone)]
pub struct EuclideanDistanceSimilarity {
    /// Temperature parameter for softening the distance (higher = softer transition)
    temperature: f32,
}

impl EuclideanDistanceSimilarity {
    /// Create a new Euclidean distance similarity metric.
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }

    /// Create with default temperature (1.0).
    pub fn default_temp() -> Self {
        Self { temperature: 1.0 }
    }
}

impl SimilarityMetric for EuclideanDistanceSimilarity {
    fn similarity(&self, a: &DenseVector, b: &DenseVector) -> f32 {
        let distance = a.euclidean_distance(b);
        if distance.is_infinite() {
            0.0
        } else {
            // Invert and apply temperature: exp(-distance / temperature)
            (-distance / self.temperature).exp()
        }
    }

    fn metric_type(&self) -> SimilarityMetricType {
        SimilarityMetricType::EuclideanDistance
    }
}

/// A similarity engine that can compute similarities using different metrics.
pub struct SimilarityEngine {
    metric: Box<dyn SimilarityMetric>,
}

impl SimilarityEngine {
    /// Create a new similarity engine with the specified metric.
    pub fn new(metric: Box<dyn SimilarityMetric>) -> Self {
        Self { metric }
    }

    /// Create a cosine similarity engine.
    pub fn cosine() -> Self {
        Self::new(Box::new(CosineSimilarity))
    }

    /// Create a dot product similarity engine.
    pub fn dot_product() -> Self {
        Self::new(Box::new(DotProductSimilarity))
    }

    /// Create an Euclidean distance similarity engine with default temperature.
    pub fn euclidean() -> Self {
        Self::new(Box::new(EuclideanDistanceSimilarity::default_temp()))
    }

    /// Compute similarity between two vectors.
    pub fn similarity(&self, a: &DenseVector, b: &DenseVector) -> f32 {
        self.metric.similarity(a, b)
    }

    /// Get the current metric type.
    pub fn metric_type(&self) -> SimilarityMetricType {
        self.metric.metric_type()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let v1 = DenseVector::new(vec![1.0, 0.0]).normalize();
        let v2 = DenseVector::new(vec![1.0, 0.0]).normalize();
        let metric = CosineSimilarity;
        let sim = metric.similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_similarity() {
        let v1 = DenseVector::new(vec![1.0, 2.0, 3.0]);
        let v2 = DenseVector::new(vec![4.0, 5.0, 6.0]);
        let metric = DotProductSimilarity;
        assert_eq!(metric.similarity(&v1, &v2), 32.0);
    }

    #[test]
    fn test_euclidean_distance_similarity() {
        let v1 = DenseVector::new(vec![0.0, 0.0]);
        let v2 = DenseVector::new(vec![0.0, 0.0]);
        let metric = EuclideanDistanceSimilarity::default_temp();
        let sim = metric.similarity(&v1, &v2);
        assert!((sim - 1.0).abs() < 1e-6); // Same point = distance 0 = similarity 1
    }

    #[test]
    fn test_similarity_engine() {
        let engine = SimilarityEngine::cosine();
        let v1 = DenseVector::new(vec![1.0, 0.0]).normalize();
        let v2 = DenseVector::new(vec![0.0, 1.0]).normalize();
        let sim = engine.similarity(&v1, &v2);
        assert!(sim.abs() < 1e-6);
    }
}
