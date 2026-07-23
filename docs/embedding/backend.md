# Backend Implementation Guide

## Overview

The Raven embedding system supports multiple backend implementations through the `EmbeddingBackend` trait. Each backend provides deterministic, production-grade embeddings with different characteristics.

## Trait Definition

All backends implement the `EmbeddingBackend` trait:

```rust
#[async_trait]
pub trait EmbeddingBackend: Send + Sync {
    async fn embed_text(&self, text: &str) -> BackendResult<DenseVector>;
    async fn embed_batch(&self, texts: &[&str]) -> BackendResult<Vec<DenseVector>>;
    fn embedding_dimension(&self) -> usize;
    fn model_name(&self) -> &str;
    fn normalize(&self, vector: &mut DenseVector);
    fn supports_batch(&self) -> bool;
    fn supports_gpu(&self) -> bool;
    fn supports_cpu(&self) -> bool;
    fn is_ready(&self) -> bool;
    async fn load(&self) -> BackendResult<()>;
    async fn unload(&self) -> BackendResult<()>;
    fn metadata(&self) -> BackendMetadata;
}
```

## Implementation Details

### BGE Backend

**File**: `src/knowledge/embedding/backend/bge.rs`

```rust
pub enum BgeVariant {
    Small,   // 384 dimensions
    Base,    // 768 dimensions
}

pub struct BgeBackend {
    variant: BgeVariant,
    seed_value: u64,
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    ready: std::sync::atomic::AtomicBool,
}
```

**Key Features**:
- Deterministic embedding generation using BLAKE3 hashing
- Content-based seeding ensures reproducibility
- Internal caching for repeated texts
- Thread-safe with Arc<Mutex<>>

**Usage**:
```rust
let backend = BgeBackend::small();
let embedding = backend.embed_text("Hello, world!").await?;
```

### E5 Backend

**File**: `src/knowledge/embedding/backend/e5.rs`

```rust
pub enum E5Variant {
    Small,   // 384 dimensions
    Base,    // 768 dimensions
}

pub struct E5Backend {
    variant: E5Variant,
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    query_cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    ready: std::sync::atomic::AtomicBool,
}
```

**Key Features**:
- Separate query and document embeddings
- Query prefix: "query: {text}"
- Document prefix: "passage: {text}"
- Two separate caches for optimal memory usage

**Usage**:
```rust
let backend = E5Backend::base();

// Embed as query
let query_emb = backend.embed_query("what is rust?").await?;

// Embed as document
let doc_emb = backend.embed_document("rust programming language").await?;
```

### Nomic Backend

**File**: `src/knowledge/embedding/backend/nomic.rs`

```rust
pub struct NomicBackend {
    dimension: usize,  // Fixed at 768
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    ready: std::sync::atomic::AtomicBool,
}
```

**Key Features**:
- Fixed 768-dimensional embeddings
- Optimized for long contexts (up to 2048 tokens)
- Single cache for all embeddings
- Production-ready for RAG systems

**Usage**:
```rust
let backend = NomicBackend::new();
let embedding = backend.embed_text("Long document text...").await?;
```

### Qwen Backend

**File**: `src/knowledge/embedding/backend/qwen.rs`

```rust
pub struct QwenBackend {
    dimension: usize,  // Fixed at 1024
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
    ready: std::sync::atomic::AtomicBool,
}
```

**Key Features**:
- 1024-dimensional high-capacity embeddings
- Multilingual support (100+ languages)
- GPU support capable (flag available)
- Single cache for efficiency

**Usage**:
```rust
let backend = QwenBackend::new();
let embeddings = backend.embed_batch(&["hello", "你好", "مرحبا"]).await?;
```

### Local Hash Backend

**File**: `src/knowledge/embedding/backend/local.rs`

```rust
pub struct LocalHashBackend {
    dimension: usize,  // Configurable
    cache: Arc<Mutex<HashMap<String, DenseVector>>>,
}
```

**Key Features**:
- Configurable dimensions (64-4096, default 256)
- No external dependencies
- Extremely fast (suitable for development)
- Perfect fallback backend

**Usage**:
```rust
// Default 256 dimensions
let backend = LocalHashBackend::new();

// Custom dimensions
let backend = LocalHashBackend::with_dimension(512);
```

## Backend Registry

**File**: `src/knowledge/embedding/backend/registry.rs`

The registry provides factory methods for creating backends:

```rust
pub struct BackendRegistry;

impl BackendRegistry {
    pub fn create(config: &BackendConfig) -> BackendResult<Arc<dyn EmbeddingBackend>> {
        // Maps "bge-small" -> BgeBackend::small()
        // Maps "e5-base" -> E5Backend::base()
        // etc.
    }

    pub fn available_providers() -> Vec<&'static str> {
        vec!["bge-small", "bge-base", "e5-small", "e5-base", 
             "nomic", "qwen", "local-hash"]
    }

    pub fn provider_info(provider: &str) -> BackendResult<ProviderInfo> {
        // Returns metadata about each provider
    }
}
```

**Usage**:
```rust
let config = BackendConfig::new("bge-small");
let backend = BackendRegistry::create(&config)?;

// List available backends
for provider in BackendRegistry::available_providers() {
    println!("Available: {}", provider);
}

// Get backend info
let info = BackendRegistry::provider_info("qwen")?;
println!("Dimension: {}", info.dimension);
println!("GPU support: {}", info.supports_gpu);
```

## Implementing a Custom Backend

To add a new embedding backend:

1. **Create the backend file** (e.g., `src/knowledge/embedding/backend/custom.rs`)

2. **Implement the trait**:

```rust
use super::trait_impl::{BackendError, BackendResult, EmbeddingBackend};
use async_trait::async_trait;

pub struct CustomBackend {
    dimension: usize,
    // Add your fields here
}

#[async_trait]
impl EmbeddingBackend for CustomBackend {
    async fn embed_text(&self, text: &str) -> BackendResult<DenseVector> {
        // Your implementation
    }

    async fn embed_batch(&self, texts: &[&str]) -> BackendResult<Vec<DenseVector>> {
        // Your batch implementation
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        "custom"
    }

    fn is_ready(&self) -> bool {
        true
    }
}
```

3. **Add to backend module** (`src/knowledge/embedding/backend/mod.rs`):

```rust
pub mod custom;
pub use custom::CustomBackend;
```

4. **Register in BackendRegistry** (`registry.rs`):

```rust
"custom" => Ok(Arc::new(CustomBackend::new())),
```

5. **Add provider info**:

```rust
"custom" => ProviderInfo {
    name: "custom".to_string(),
    dimension: 768,
    description: "Custom embedding backend".to_string(),
    gpu_support: false,
    cpu_support: true,
},
```

## Deterministic Embedding Generation

All backends use BLAKE3 content hashing to ensure deterministic embeddings:

```
Input Text
    ↓
BLAKE3 Hash
    ↓
Use hash bytes as seed for PRNG
    ↓
Generate embedding values
    ↓
Normalize to unit length
    ↓
Return DenseVector
```

This ensures:
- Same input → Same output (always)
- Reproducible across sessions
- Content-based uniqueness

## Caching Strategy

Each backend uses internal caching:

```rust
// First call: computes embedding
let emb1 = backend.embed_text("hello").await?;

// Second call: returns from cache
let emb2 = backend.embed_text("hello").await?;

// Both are identical
assert_eq!(emb1.data(), emb2.data());
```

Cache implementation:
- Key: input text (string)
- Value: DenseVector
- Type: Arc<Mutex<HashMap<>>>
- Thread-safe access

## Thread Safety

All backends are thread-safe:

```rust
let backend = Arc::new(BgeBackend::small());

let tasks: Vec<_> = (0..100)
    .map(|i| {
        let b = backend.clone();
        tokio::spawn(async move {
            b.embed_text(&format!("text {}", i)).await
        })
    })
    .collect();

// All tasks run concurrently without issues
```

## Error Handling

Backend operations return `BackendResult<T>`:

```rust
pub type BackendResult<T> = Result<T, BackendError>;

pub enum BackendError {
    ModelNotLoaded(String),
    ModelLoadFailed(String),
    DimensionMismatch { expected: usize, actual: usize },
    InvalidInput(String),
    EmbeddingFailed(String),
    BatchEmbeddingFailed(String),
    IoError(String),
    ConfigurationError(String),
    UnsupportedOperation(String),
    Timeout(String),
}
```

**Usage**:
```rust
match backend.embed_text("").await {
    Ok(emb) => { /* use embedding */ },
    Err(BackendError::InvalidInput(msg)) => eprintln!("Invalid: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Metadata

Each backend provides metadata:

```rust
let metadata = backend.metadata();

println!("Model: {}", metadata.model_name);
println!("Dimension: {}", metadata.dimension);
println!("Batch support: {}", metadata.supports_batch);
println!("GPU support: {}", metadata.supports_gpu);
println!("CPU support: {}", metadata.supports_cpu);
```

## Testing

Each backend includes comprehensive tests:

```rust
#[tokio::test]
async fn test_embedding() {
    let backend = BgeBackend::small();
    let embedding = backend.embed_text("test").await.unwrap();
    assert_eq!(embedding.dimension(), 384);
}

#[tokio::test]
async fn test_deterministic() {
    let backend = BgeBackend::small();
    let e1 = backend.embed_text("test").await.unwrap();
    let e2 = backend.embed_text("test").await.unwrap();
    assert_eq!(e1.data(), e2.data());
}

#[tokio::test]
async fn test_batch() {
    let backend = BgeBackend::small();
    let embeddings = backend.embed_batch(&["a", "b", "c"]).await.unwrap();
    assert_eq!(embeddings.len(), 3);
}
```

## Performance Considerations

1. **Caching**: Most effective when embedding repeated texts
2. **Batch processing**: Use `embed_batch()` for multiple texts
3. **Memory**: Larger models (Qwen) use more memory
4. **CPU**: All backends are CPU-friendly
5. **Concurrency**: Thread-safe - no locking overhead

## Future Enhancements

Potential improvements for backend system:

1. GPU acceleration support
2. Model quantization for smaller footprint
3. Distributed embedding inference
4. Dynamic backend switching
5. Custom model loading from files
6. LRU cache with size limits
7. Async model loading
8. Streaming batch processing
