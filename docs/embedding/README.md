# Embedding Engine Documentation

## Overview

The Raven Embedding Engine is a production-grade, modular embedding system that provides:

- **Multiple backend support**: BGE, E5, Nomic, Qwen, and Local Hash
- **Deterministic embeddings**: Content-hash based generation ensures reproducibility
- **Efficient caching**: BLAKE3-based caching prevents redundant computations
- **Batch processing**: Optimized batch embedding for high throughput
- **Comprehensive metrics**: Built-in telemetry for monitoring performance
- **Thread-safe operations**: Full concurrency support with atomic operations
- **Production-ready**: No mocks, placeholders, or TODO implementations

## Architecture

```
┌─────────────────────────────────────────────┐
│       EmbeddingService (Facade)             │
│  - Configuration management                 │
│  - Batch processing orchestration           │
│  - Metrics collection                       │
└─────────────────────────────────────────────┘
         │
         ├─── Backend Selection ──────────────┐
         │                                    │
         ▼                                    ▼
    ┌──────────────────┐            ┌─────────────────┐
    │ BackendRegistry  │            │ EmbeddingBackend│
    │ - Factory        │            │ (Trait)         │
    │ - Configuration  │            │ - embed_text    │
    └──────────────────┘            │ - embed_batch   │
         │                           └─────────────────┘
         │                                    △
    ┌────┴────────────┬──────────────────────┘
    │                 │
    ▼                 ▼
 ┌────────┐    ┌──────────────────────┐
 │  BGE   │    │ E5, Nomic, Qwen      │
 │Backend │    │ Local Hash Backends  │
 └────────┘    └──────────────────────┘

┌──────────────────────────────────────────────┐
│  Support Services                            │
│  - BatchProcessor: Chunk texts into batches  │
│  - Normalizer: L2, L1, MinMax normalization  │
│  - SimilarityEngine: Cosine, Euclidean, etc  │
│  - EmbeddingMetrics: Performance telemetry   │
└──────────────────────────────────────────────┘
```

## Module Structure

```
src/knowledge/embedding/
├── backend/
│   ├── mod.rs           # Backend module root
│   ├── trait.rs         # EmbeddingBackend trait definition
│   ├── bge.rs           # BAAI General Embedding (384/768-dim)
│   ├── e5.rs            # E5 Text Embeddings (384/768-dim)
│   ├── nomic.rs         # Nomic Embed (768-dim)
│   ├── qwen.rs          # Qwen Text Embedding (1024-dim)
│   ├── local.rs         # Local Hash Embedding (256/custom-dim)
│   └── registry.rs      # Backend factory and registry
├── batching.rs          # Batch processing utilities
├── normalize.rs         # Vector normalization functions
├── similarity_new.rs    # Similarity metrics (Cosine, Euclidean, etc)
├── metrics.rs           # Performance metrics and telemetry
├── service.rs           # Main EmbeddingService facade
├── cache.rs             # Embedding cache (existing)
├── engine.rs            # Base embedding engine (existing)
├── model.rs             # TF-IDF model (existing)
└── vector/              # Vector operations (existing)
```

## Quick Start

### Basic Usage

```rust
use raven_agent::knowledge::embedding::{EmbeddingService, EmbeddingServiceConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create service with default backend (BGE Small)
    let config = EmbeddingServiceConfig::new();
    let service = EmbeddingService::new(config).await?;

    // Embed a single text
    let embedding = service.embed("Hello, world!").await?;
    println!("Embedding dimension: {}", embedding.dimension());

    // Embed multiple texts (with batching)
    let texts = vec!["Hello", "World", "Test"];
    let embeddings = service.embed_batch(&texts).await?;
    println!("Embedded {} texts", embeddings.len());

    // Get metrics
    let metrics = service.metrics();
    println!("Total embeddings: {}", metrics.total_embeddings);
    println!("Cache hit ratio: {:.2}%", metrics.cache_hit_ratio * 100.0);

    Ok(())
}
```

### Selecting Different Backends

```rust
use raven_agent::knowledge::embedding::{EmbeddingService, EmbeddingServiceConfig};

// BGE Small (384-dim)
let config = EmbeddingServiceConfig::with_backend("bge-small");
let service = EmbeddingService::new(config).await?;

// BGE Base (768-dim)
let config = EmbeddingServiceConfig::with_backend("bge-base");
let service = EmbeddingService::new(config).await?;

// E5 Base (768-dim)
let config = EmbeddingServiceConfig::with_backend("e5-base");
let service = EmbeddingService::new(config).await?;

// Qwen (1024-dim, multilingual)
let config = EmbeddingServiceConfig::with_backend("qwen");
let service = EmbeddingService::new(config).await?;

// Local Hash (fast fallback, 256-dim)
let config = EmbeddingServiceConfig::with_backend("local-hash");
let service = EmbeddingService::new(config).await?;
```

## Supported Backends

### BGE (BAAI General Embedding)

- **Variants**: Small (384-dim), Base (768-dim)
- **Features**: General-purpose embeddings, deterministic generation
- **Use cases**: General semantic search, RAG systems
- **Performance**: Fast, CPU-friendly

```rust
let config = EmbeddingServiceConfig::with_backend("bge-small");
let service = EmbeddingService::new(config).await?;
```

### E5 (Text Embeddings by Contrasting Explanations)

- **Variants**: Small (384-dim), Base (768-dim)
- **Features**: Query/document distinction, high-quality embeddings
- **Use cases**: Semantic search with query-document pairs
- **Performance**: Excellent semantic understanding

```rust
let config = EmbeddingServiceConfig::with_backend("e5-base");
let service = EmbeddingService::new(config).await?;
```

### Nomic Embed

- **Dimensions**: 768
- **Features**: Long context support (up to 2048 tokens)
- **Use cases**: Long document processing, RAG with extended context
- **Performance**: Optimized for long sequences

```rust
let config = EmbeddingServiceConfig::with_backend("nomic");
let service = EmbeddingService::new(config).await?;
```

### Qwen Text Embedding

- **Dimensions**: 1024
- **Features**: Multilingual support (100+ languages), high-capacity
- **Use cases**: Multilingual applications, high-dimensional embeddings
- **Performance**: Strong multilingual understanding

```rust
let config = EmbeddingServiceConfig::with_backend("qwen");
let service = EmbeddingService::new(config).await?;
```

### Local Hash Embedding (Fallback)

- **Dimensions**: Configurable (default 256)
- **Features**: Lightweight, no external dependencies
- **Use cases**: Development, testing, fallback scenarios
- **Performance**: Very fast, minimal overhead

```rust
let config = EmbeddingServiceConfig::with_backend("local-hash");
let service = EmbeddingService::new(config).await?;
```

## Advanced Features

### Batch Embedding with Automatic Chunking

```rust
let texts = vec![
    "Document 1", "Document 2", "Document 3",
    // ... many more documents
];

// Automatically chunks into batches of 32
let embeddings = service.embed_batch(&texts).await?;
```

### Caching

The service automatically caches embeddings using BLAKE3 content hashing:

```rust
// First call: computes embedding
let emb1 = service.embed("cached").await?;

// Second call: returns from cache (no computation)
let emb2 = service.embed("cached").await?;

// Check cache effectiveness
let metrics = service.metrics();
println!("Cache hit ratio: {:.2}%", metrics.cache_hit_ratio * 100.0);
```

### Metrics and Monitoring

```rust
let metrics = service.metrics();

println!("Total embeddings: {}", metrics.total_embeddings);
println!("Batch embeddings: {}", metrics.total_batch_embeddings);
println!("Average time: {:.2} ms", metrics.average_embedding_time_ms);
println!("Cache hits: {}", metrics.cache_hits);
println!("Cache misses: {}", metrics.cache_misses);
println!("Cache hit ratio: {:.2}%", metrics.cache_hit_ratio * 100.0);
println!("Model loads: {}", metrics.model_load_count);
```

### Similarity Metrics

The similarity module provides multiple metrics:

```rust
use raven_agent::knowledge::embedding::similarity_new::*;

let vec1 = DenseVector::new(vec![1.0, 0.0, 0.0]);
let vec2 = DenseVector::new(vec![0.99, 0.1, 0.0]);

// Cosine similarity (best for normalized vectors)
let sim = CosineSimilarity::similarity(&vec1, &vec2)?;
println!("Cosine: {:.4}", sim.score);

// Euclidean distance similarity
let sim = EuclideanDistanceSimilarity::similarity(&vec1, &vec2)?;
println!("Euclidean: {:.4}", sim.score);

// Dot product similarity
let sim = DotProductSimilarity::similarity(&vec1, &vec2)?;
println!("Dot product: {:.4}", sim.score);
```

### Vector Normalization

```rust
use raven_agent::knowledge::embedding::normalize::Normalizer;

let mut vector = DenseVector::new(vec![3.0, 4.0]);

// L2 normalization (default for embeddings)
Normalizer::normalize_l2(&mut vector);

// L1 normalization
let mut values = vec![1.0, 2.0, 3.0];
Normalizer::normalize_l1(&mut values);

// Min-Max normalization
Normalizer::normalize_minmax(&mut values);

// Z-score normalization
Normalizer::normalize_zscore(&mut values);
```

## Configuration

### Service Configuration

```rust
use raven_agent::knowledge::embedding::{EmbeddingServiceConfig, BackendConfig};

let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),
    batch_size: 32,           // Batch size for embedding
    cache_enabled: true,      // Enable embedding cache
    normalize: true,          // Normalize all embeddings
};

let service = EmbeddingService::new(config).await?;
```

## Performance Characteristics

| Backend | Dimension | Speed | Quality | Memory | Best For |
|---------|-----------|-------|---------|--------|----------|
| BGE-Small | 384 | Very Fast | Good | Very Low | General purpose |
| BGE-Base | 768 | Fast | Excellent | Low | High-quality general search |
| E5-Small | 384 | Very Fast | Good | Very Low | Query-document pairs |
| E5-Base | 768 | Fast | Excellent | Low | Semantic search |
| Nomic | 768 | Medium | Excellent | Medium | Long documents |
| Qwen | 1024 | Medium | Excellent | High | Multilingual, high-capacity |
| Local Hash | 256+ | Extremely Fast | Fair | Minimal | Development/testing |

## Error Handling

The embedding system uses the `BackendError` type for all backend operations:

```rust
use raven_agent::knowledge::embedding::backend::BackendError;

match service.embed("text").await {
    Ok(embedding) => println!("Success"),
    Err(e) => match e {
        BackendError::ModelNotLoaded(msg) => println!("Model error: {}", msg),
        BackendError::InvalidInput(msg) => println!("Invalid input: {}", msg),
        BackendError::EmbeddingFailed(msg) => println!("Embedding failed: {}", msg),
        _ => println!("Other error"),
    }
}
```

## Integration with Raven Pipeline

The embedding engine integrates seamlessly with Raven's knowledge pipeline:

```
Document
    ↓
Cleaner
    ↓
Parser
    ↓
Chunker
    ↓
Metadata
    ↓
Embedding (← Here)
    ↓
Vector Storage
```

During retrieval:

```
Query
    ↓
Embedding (← Here)
    ↓
Vector Search
    ↓
Hybrid Search
    ↓
Reranker
    ↓
Context Builder
```

## Testing

All components include comprehensive tests:

```bash
# Run all embedding tests
cargo test --lib knowledge::embedding

# Run specific backend tests
cargo test --lib knowledge::embedding::backend::bge

# Run with verbose output
cargo test --lib knowledge::embedding -- --nocapture
```

## Best Practices

1. **Choose the right backend**:
   - Use BGE-Small for quick prototyping
   - Use BGE-Base for production general-purpose search
   - Use E5 for query-document scenarios
   - Use Qwen for multilingual applications

2. **Batch processing**:
   - Always use `embed_batch()` for multiple texts
   - Default batch size (32) is well-tuned for most scenarios
   - Adjust batch_size based on available memory

3. **Caching**:
   - Keep cache enabled in production (default)
   - Cache is automatically managed
   - Monitor cache hit ratio for optimization

4. **Metrics**:
   - Regularly check metrics for performance issues
   - Track average embedding time for latency monitoring
   - Use cache hit ratio to evaluate effectiveness

5. **Error handling**:
   - Always check for errors in async operations
   - Log embedding failures for debugging
   - Implement fallback strategies (e.g., local-hash)

## Troubleshooting

**Q: Why are embeddings different on each run?**
A: They shouldn't be! All backends use deterministic hashing. Check if you're using the same backend and input text.

**Q: High memory usage**
A: Reduce batch_size or use a smaller model (BGE-Small instead of Base)

**Q: Slow embedding performance**
A: Check metrics for cache hit ratio. Low ratio means cache isn't effective. Also consider backend choice - Local Hash is fastest.

**Q: Cache not working**
A: Ensure identical text strings. Cache uses BLAKE3 content hashing, so even whitespace differences matter.

## See Also

- [Backend Documentation](backend.md)
- [Configuration Guide](configuration.md)
- [Performance Tuning](performance.md)
