//! Vector representation and operations.
//!
//! This module provides efficient vector operations including normalization,
//! serialization, and fundamental similarity computations.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A dense vector representation using f32 for efficient computation.
///
/// Vectors are typically normalized to unit length for cosine similarity.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DenseVector {
    data: Vec<f32>,
}

impl DenseVector {
    /// Create a new dense vector from a slice of f32 values.
    pub fn new(data: Vec<f32>) -> Self {
        Self { data }
    }

    /// Create a zero vector of the given dimension.
    pub fn zeros(dim: usize) -> Self {
        Self {
            data: vec![0.0; dim],
        }
    }

    /// Get the dimension of the vector.
    pub fn dimension(&self) -> usize {
        self.data.len()
    }

    /// Get a reference to the underlying data.
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get a mutable reference to the underlying data.
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Compute the L2 norm (Euclidean norm) of the vector.
    pub fn l2_norm(&self) -> f32 {
        let sum: f32 = self.data.iter().map(|x| x * x).sum();
        sum.sqrt()
    }

    /// Compute the squared L2 norm (useful for optimization).
    pub fn l2_norm_squared(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum()
    }

    /// Normalize the vector to unit length (L2 normalization).
    ///
    /// If the vector is zero, returns a zero vector.
    pub fn normalize(&self) -> Self {
        let norm = self.l2_norm();
        if norm == 0.0 || norm.is_nan() {
            return Self::zeros(self.dimension());
        }
        let normalized = self.data.iter().map(|x| x / norm).collect();
        Self { data: normalized }
    }

    /// Normalize the vector in-place to unit length.
    pub fn normalize_inplace(&mut self) {
        let norm = self.l2_norm();
        if norm > 0.0 && !norm.is_nan() {
            for x in &mut self.data {
                *x /= norm;
            }
        }
    }

    /// Compute the dot product with another vector.
    ///
    /// Returns 0.0 if dimensions don't match.
    pub fn dot_product(&self, other: &DenseVector) -> f32 {
        if self.dimension() != other.dimension() {
            return 0.0;
        }
        self.data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| a * b)
            .sum()
    }

    /// Compute the cosine similarity with another vector (both assumed normalized).
    ///
    /// For normalized vectors, this is equivalent to dot product.
    pub fn cosine_similarity(&self, other: &DenseVector) -> f32 {
        self.dot_product(other)
    }

    /// Compute the Euclidean distance to another vector.
    ///
    /// Returns f32::INFINITY if dimensions don't match.
    pub fn euclidean_distance(&self, other: &DenseVector) -> f32 {
        if self.dimension() != other.dimension() {
            return f32::INFINITY;
        }
        let sum: f32 = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| (a - b) * (a - b))
            .sum();
        sum.sqrt()
    }

    /// Compute the squared Euclidean distance (useful for optimization).
    pub fn euclidean_distance_squared(&self, other: &DenseVector) -> f32 {
        if self.dimension() != other.dimension() {
            return f32::INFINITY;
        }
        self.data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| (a - b) * (a - b))
            .sum()
    }

    /// Element-wise addition.
    pub fn add(&self, other: &DenseVector) -> Option<Self> {
        if self.dimension() != other.dimension() {
            return None;
        }
        let result = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| a + b)
            .collect();
        Some(Self { data: result })
    }

    /// Element-wise subtraction.
    pub fn subtract(&self, other: &DenseVector) -> Option<Self> {
        if self.dimension() != other.dimension() {
            return None;
        }
        let result = self
            .data
            .iter()
            .zip(&other.data)
            .map(|(a, b)| a - b)
            .collect();
        Some(Self { data: result })
    }

    /// Scalar multiplication.
    pub fn scale(&self, scalar: f32) -> Self {
        let result = self.data.iter().map(|x| x * scalar).collect();
        Self { data: result }
    }

    /// Check if the vector contains any NaN values.
    pub fn has_nan(&self) -> bool {
        self.data.iter().any(|x| x.is_nan())
    }

    /// Check if the vector contains any infinity values.
    pub fn has_infinity(&self) -> bool {
        self.data.iter().any(|x| x.is_infinite())
    }

    /// Get the maximum absolute value in the vector.
    pub fn max_abs(&self) -> f32 {
        self.data
            .iter()
            .map(|x| x.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get the minimum value in the vector.
    pub fn min(&self) -> f32 {
        self.data
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get the maximum value in the vector.
    pub fn max(&self) -> f32 {
        self.data
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get the mean value of the vector.
    pub fn mean(&self) -> f32 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f32>() / self.data.len() as f32
    }

    /// Get the sum of all values in the vector.
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }
}

impl fmt::Display for DenseVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DenseVector(dim={})", self.dimension())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_norm() {
        let v = DenseVector::new(vec![3.0, 4.0]);
        assert_eq!(v.l2_norm(), 5.0);
    }

    #[test]
    fn test_normalize() {
        let v = DenseVector::new(vec![3.0, 4.0]);
        let normalized = v.normalize();
        assert!((normalized.l2_norm() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product() {
        let v1 = DenseVector::new(vec![1.0, 2.0, 3.0]);
        let v2 = DenseVector::new(vec![4.0, 5.0, 6.0]);
        assert_eq!(v1.dot_product(&v2), 32.0);
    }

    #[test]
    fn test_euclidean_distance() {
        let v1 = DenseVector::new(vec![0.0, 0.0]);
        let v2 = DenseVector::new(vec![3.0, 4.0]);
        assert_eq!(v1.euclidean_distance(&v2), 5.0);
    }

    #[test]
    fn test_add() {
        let v1 = DenseVector::new(vec![1.0, 2.0]);
        let v2 = DenseVector::new(vec![3.0, 4.0]);
        let result = v1.add(&v2).unwrap();
        assert_eq!(result.data(), &[4.0, 6.0]);
    }

    #[test]
    fn test_scale() {
        let v = DenseVector::new(vec![1.0, 2.0, 3.0]);
        let scaled = v.scale(2.0);
        assert_eq!(scaled.data(), &[2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_dimension_mismatch() {
        let v1 = DenseVector::new(vec![1.0, 2.0]);
        let v2 = DenseVector::new(vec![3.0, 4.0, 5.0]);
        assert_eq!(v1.dot_product(&v2), 0.0);
        assert_eq!(v1.euclidean_distance(&v2), f32::INFINITY);
        assert!(v1.add(&v2).is_none());
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = DenseVector::new(vec![1.0, 0.0]).normalize();
        let v2 = DenseVector::new(vec![1.0, 0.0]).normalize();
        let sim = v1.cosine_similarity(&v2);
        assert!((sim - 1.0).abs() < 1e-6);
    }
}
