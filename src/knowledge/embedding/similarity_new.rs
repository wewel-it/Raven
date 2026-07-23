//! Similarity metrics for embeddings.
//!
//! This module provides various similarity metrics for comparing embeddings.

use crate::knowledge::embedding::vector::DenseVector;

/// Similarity result.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Similarity {
    pub score: f32,
}

impl Similarity {
    /// Create a new similarity score.
    pub fn new(score: f32) -> Self {
        Self {
            score: score.clamp(-1.0, 1.0),
        }
    }

    /// Get the similarity score.
    pub fn score(&self) -> f32 {
        self.score
    }

    /// Check if similarity is above threshold.
    pub fn above_threshold(&self, threshold: f32) -> bool {
        self.score > threshold
    }
}

/// Cosine similarity.
pub struct CosineSimilarity;

impl CosineSimilarity {
    /// Calculate cosine similarity between two vectors.
    pub fn similarity(a: &DenseVector, b: &DenseVector) -> Result<Similarity, String> {
        if a.dimension() != b.dimension() {
            return Err(format!(
                "Dimension mismatch: {} vs {}",
                a.dimension(),
                b.dimension()
            ));
        }

        let dot_product: f32 = a
            .data()
            .iter()
            .zip(b.data().iter())
            .map(|(x, y)| x * y)
            .sum();

        let magnitude_a = a.l2_norm();
        let magnitude_b = b.l2_norm();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return Ok(Similarity::new(0.0));
        }

        let similarity = dot_product / (magnitude_a * magnitude_b);
        Ok(Similarity::new(similarity))
    }
}

/// Dot product similarity.
pub struct DotProductSimilarity;

impl DotProductSimilarity {
    /// Calculate dot product similarity between two vectors.
    pub fn similarity(a: &DenseVector, b: &DenseVector) -> Result<Similarity, String> {
        if a.dimension() != b.dimension() {
            return Err(format!(
                "Dimension mismatch: {} vs {}",
                a.dimension(),
                b.dimension()
            ));
        }

        let dot_product: f32 = a
            .data()
            .iter()
            .zip(b.data().iter())
            .map(|(x, y)| x * y)
            .sum();

        // Normalize to [-1, 1] range
        let normalized = (dot_product / a.dimension() as f32).clamp(-1.0, 1.0);
        Ok(Similarity::new(normalized))
    }
}

/// Euclidean distance similarity.
pub struct EuclideanDistanceSimilarity;

impl EuclideanDistanceSimilarity {
    /// Calculate Euclidean distance similarity (1 / (1 + distance)).
    pub fn similarity(a: &DenseVector, b: &DenseVector) -> Result<Similarity, String> {
        if a.dimension() != b.dimension() {
            return Err(format!(
                "Dimension mismatch: {} vs {}",
                a.dimension(),
                b.dimension()
            ));
        }

        let sum_of_squares: f32 = a
            .data()
            .iter()
            .zip(b.data().iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum();

        let distance = sum_of_squares.sqrt();
        let similarity = 1.0 / (1.0 + distance);
        Ok(Similarity::new(similarity))
    }
}

/// Manhattan distance similarity.
pub struct ManhattanDistanceSimilarity;

impl ManhattanDistanceSimilarity {
    /// Calculate Manhattan distance similarity.
    pub fn similarity(a: &DenseVector, b: &DenseVector) -> Result<Similarity, String> {
        if a.dimension() != b.dimension() {
            return Err(format!(
                "Dimension mismatch: {} vs {}",
                a.dimension(),
                b.dimension()
            ));
        }

        let distance: f32 = a
            .data()
            .iter()
            .zip(b.data().iter())
            .map(|(x, y)| (x - y).abs())
            .sum();

        let similarity = 1.0 / (1.0 + distance);
        Ok(Similarity::new(similarity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vector(values: Vec<f32>) -> DenseVector {
        DenseVector::new(values)
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let a = create_test_vector(vec![1.0, 0.0, 0.0]);
        let b = create_test_vector(vec![1.0, 0.0, 0.0]);
        let sim = CosineSimilarity::similarity(&a, &b).unwrap();
        assert!((sim.score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = create_test_vector(vec![1.0, 0.0, 0.0]);
        let b = create_test_vector(vec![0.0, 1.0, 0.0]);
        let sim = CosineSimilarity::similarity(&a, &b).unwrap();
        assert!(sim.score.abs() < 0.001);
    }

    #[test]
    fn test_euclidean_identical() {
        let a = create_test_vector(vec![1.0, 2.0, 3.0]);
        let b = create_test_vector(vec![1.0, 2.0, 3.0]);
        let sim = EuclideanDistanceSimilarity::similarity(&a, &b).unwrap();
        assert!((sim.score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_similarity_threshold() {
        let sim = Similarity::new(0.8);
        assert!(sim.above_threshold(0.7));
        assert!(!sim.above_threshold(0.9));
    }
}
