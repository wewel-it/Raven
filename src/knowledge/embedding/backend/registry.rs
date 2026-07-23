//! Backend registry for dynamic backend selection.
//!
//! This module provides a registry for selecting embedding backends
//! based on configuration.

use super::bge::BgeBackend;
use super::e5::E5Backend;
use super::local::LocalHashBackend;
use super::nomic::NomicBackend;
use super::qwen::QwenBackend;
use super::trait_impl::{BackendError, BackendResult, EmbeddingBackend};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for backend selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Backend provider name (e.g., "bge-small", "e5-base", etc.)
    pub provider: String,
    /// Optional model path (for future use with local model files)
    pub model_path: Option<String>,
}

impl BackendConfig {
    /// Create a new backend configuration.
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model_path: None,
        }
    }

    /// Set model path.
    pub fn with_model_path(mut self, path: impl Into<String>) -> Self {
        self.model_path = Some(path.into());
        self
    }
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            provider: "bge-small".to_string(),
            model_path: None,
        }
    }
}

/// Backend registry for creating backends from configuration.
pub struct BackendRegistry;

impl BackendRegistry {
    /// Create a backend from configuration.
    pub fn create(config: &BackendConfig) -> BackendResult<Arc<dyn EmbeddingBackend>> {
        let provider = config.provider.to_lowercase();

        match provider.as_str() {
            "bge-small" => Ok(Arc::new(BgeBackend::small())),
            "bge-base" => Ok(Arc::new(BgeBackend::base())),
            "e5-small" => Ok(Arc::new(E5Backend::small())),
            "e5-base" => Ok(Arc::new(E5Backend::base())),
            "nomic" | "nomic-embed" => Ok(Arc::new(NomicBackend::new())),
            "qwen" | "qwen-embed" => Ok(Arc::new(QwenBackend::new())),
            "local-hash" | "local" | "hash" => Ok(Arc::new(LocalHashBackend::new())),
            _ => Err(BackendError::ConfigurationError(format!(
                "Unknown backend provider: {}",
                config.provider
            ))),
        }
    }

    /// List available backend providers.
    pub fn available_providers() -> Vec<&'static str> {
        vec![
            "bge-small",
            "bge-base",
            "e5-small",
            "e5-base",
            "nomic",
            "qwen",
            "local-hash",
        ]
    }

    /// Get information about a provider.
    pub fn provider_info(provider: &str) -> BackendResult<ProviderInfo> {
        let provider = provider.to_lowercase();

        let info = match provider.as_str() {
            "bge-small" => ProviderInfo {
                name: "bge-small".to_string(),
                dimension: 384,
                description: "BAAI General Embedding Small (384-dim)".to_string(),
                gpu_support: false,
                cpu_support: true,
            },
            "bge-base" => ProviderInfo {
                name: "bge-base".to_string(),
                dimension: 768,
                description: "BAAI General Embedding Base (768-dim)".to_string(),
                gpu_support: false,
                cpu_support: true,
            },
            "e5-small" => ProviderInfo {
                name: "e5-small".to_string(),
                dimension: 384,
                description: "E5 Small (384-dim)".to_string(),
                gpu_support: false,
                cpu_support: true,
            },
            "e5-base" => ProviderInfo {
                name: "e5-base".to_string(),
                dimension: 768,
                description: "E5 Base (768-dim)".to_string(),
                gpu_support: false,
                cpu_support: true,
            },
            "nomic" | "nomic-embed" => ProviderInfo {
                name: "nomic".to_string(),
                dimension: 768,
                description: "Nomic Embed (768-dim, long context)".to_string(),
                gpu_support: false,
                cpu_support: true,
            },
            "qwen" | "qwen-embed" => ProviderInfo {
                name: "qwen".to_string(),
                dimension: 1024,
                description: "Qwen Text Embedding (1024-dim, multilingual)".to_string(),
                gpu_support: true,
                cpu_support: true,
            },
            "local-hash" | "local" | "hash" => ProviderInfo {
                name: "local-hash".to_string(),
                dimension: 256,
                description: "Local Hash Embedding (256-dim fallback)".to_string(),
                gpu_support: false,
                cpu_support: true,
            },
            _ => {
                return Err(BackendError::ConfigurationError(format!(
                    "Unknown provider: {}",
                    provider
                )))
            }
        };

        Ok(info)
    }
}

/// Information about a backend provider.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub dimension: usize,
    pub description: String,
    pub gpu_support: bool,
    pub cpu_support: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_bge_small() {
        let config = BackendConfig::new("bge-small");
        let backend = BackendRegistry::create(&config);
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.embedding_dimension(), 384);
        assert_eq!(backend.model_name(), "bge-small");
    }

    #[test]
    fn test_create_e5_base() {
        let config = BackendConfig::new("e5-base");
        let backend = BackendRegistry::create(&config);
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.embedding_dimension(), 768);
        assert_eq!(backend.model_name(), "e5-base");
    }

    #[test]
    fn test_create_nomic() {
        let config = BackendConfig::new("nomic");
        let backend = BackendRegistry::create(&config);
        assert!(backend.is_ok());
    }

    #[test]
    fn test_create_qwen() {
        let config = BackendConfig::new("qwen");
        let backend = BackendRegistry::create(&config);
        assert!(backend.is_ok());
        let backend = backend.unwrap();
        assert_eq!(backend.embedding_dimension(), 1024);
    }

    #[test]
    fn test_create_local_hash() {
        let config = BackendConfig::new("local-hash");
        let backend = BackendRegistry::create(&config);
        assert!(backend.is_ok());
    }

    #[test]
    fn test_available_providers() {
        let providers = BackendRegistry::available_providers();
        assert!(providers.len() >= 7);
        assert!(providers.contains(&"bge-small"));
        assert!(providers.contains(&"qwen"));
    }

    #[test]
    fn test_provider_info() {
        let info = BackendRegistry::provider_info("bge-small").unwrap();
        assert_eq!(info.dimension, 384);
        assert!(info.cpu_support);
    }

    #[test]
    fn test_invalid_provider() {
        let config = BackendConfig::new("invalid-provider");
        let result = BackendRegistry::create(&config);
        assert!(result.is_err());
    }
}
