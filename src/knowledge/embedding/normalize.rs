//! Vector normalization utilities.
//!
//! This module provides functions for normalizing embedding vectors.

use crate::knowledge::embedding::vector::DenseVector;

/// Vector normalizer.
pub struct Normalizer;

impl Normalizer {
    /// Normalize a vector to unit length (L2 normalization).
    pub fn normalize_l2(vector: &mut DenseVector) {
        vector.normalize_inplace();
    }

    /// Normalize a vector to unit length (return new vector).
    pub fn normalize_l2_new(vector: &DenseVector) -> DenseVector {
        let mut v = vector.clone();
        v.normalize_inplace();
        v
    }

    /// L1 normalization (Manhattan distance).
    pub fn normalize_l1(values: &mut [f32]) {
        let sum: f32 = values.iter().map(|v| v.abs()).sum();
        if sum > 0.0 {
            for v in values.iter_mut() {
                *v /= sum;
            }
        }
    }

    /// Min-Max normalization to [0, 1] range.
    pub fn normalize_minmax(values: &mut [f32]) {
        if values.is_empty() {
            return;
        }

        let min = values.iter().copied().fold(f32::INFINITY, f32::min);
        let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        if (max - min).abs() < 1e-10 {
            // All values are the same
            for v in values.iter_mut() {
                *v = 0.5;
            }
        } else {
            let range = max - min;
            for v in values.iter_mut() {
                *v = (*v - min) / range;
            }
        }
    }

    /// Z-score normalization (standardization).
    pub fn normalize_zscore(values: &mut [f32]) {
        if values.len() < 2 {
            return;
        }

        let mean: f32 = values.iter().sum::<f32>() / values.len() as f32;
        let variance: f32 =
            values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;
        let std_dev = variance.sqrt();

        if std_dev > 0.0 {
            for v in values.iter_mut() {
                *v = (*v - mean) / std_dev;
            }
        }
    }

    /// Clip values to [-1, 1] range.
    pub fn clip_values(values: &mut [f32]) {
        for v in values.iter_mut() {
            *v = v.clamp(-1.0, 1.0);
        }
    }

    /// Check if a vector is normalized (approximately).
    pub fn is_normalized(vector: &DenseVector, tolerance: f32) -> bool {
        let magnitude = vector.l2_norm();
        (magnitude - 1.0).abs() < tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_l2() {
        let vec = vec![3.0, 4.0];
        let dense = DenseVector::new(vec);
        let normalized = Normalizer::normalize_l2_new(&dense);
        let magnitude = normalized.l2_norm();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_l1() {
        let mut values = vec![1.0, 2.0, 3.0];
        Normalizer::normalize_l1(&mut values);
        let sum: f32 = values.iter().sum();
        assert!((sum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_minmax() {
        let mut values = vec![0.0, 5.0, 10.0];
        Normalizer::normalize_minmax(&mut values);
        assert!(values.iter().all(|v| *v >= 0.0 && *v <= 1.0));
        assert_eq!(values[0], 0.0);
        assert_eq!(values[2], 1.0);
    }

    #[test]
    fn test_is_normalized() {
        let vec = DenseVector::new(vec![1.0, 0.0, 0.0]);
        assert!(Normalizer::is_normalized(&vec, 0.01));
    }
}
