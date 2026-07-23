# Configuration Guide

## Service Configuration

### Basic Configuration

```rust
use raven_agent::knowledge::embedding::{EmbeddingService, EmbeddingServiceConfig, BackendConfig};

let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),
    batch_size: 32,
    cache_enabled: true,
    normalize: true,
};

let service = EmbeddingService::new(config).await?;
```

### Configuration Options

#### Backend Selection

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `backend.provider` | "bge-small", "bge-base", "e5-small", "e5-base", "nomic", "qwen", "local-hash" | "bge-small" | Embedding model provider |
| `backend.model_path` | File path | None | Path to custom model (reserved for future use) |

#### Processing Options

| Option | Type | Default | Range | Description |
|--------|------|---------|-------|-------------|
| `batch_size` | usize | 32 | 1-1024 | Batch size for processing |
| `cache_enabled` | bool | true | - | Enable embedding caching |
| `normalize` | bool | true | - | Normalize embeddings to unit length |

### Using Different Backends

```rust
// BGE Small (fast, 384-dim)
let config = EmbeddingServiceConfig::with_backend("bge-small");

// BGE Base (high quality, 768-dim)
let config = EmbeddingServiceConfig::with_backend("bge-base");

// E5 Small (query-aware, 384-dim)
let config = EmbeddingServiceConfig::with_backend("e5-small");

// E5 Base (query-aware, 768-dim)
let config = EmbeddingServiceConfig::with_backend("e5-base");

// Nomic (long context, 768-dim)
let config = EmbeddingServiceConfig::with_backend("nomic");

// Qwen (multilingual, 1024-dim)
let config = EmbeddingServiceConfig::with_backend("qwen");

// Local Hash (development/fallback, 256-dim)
let config = EmbeddingServiceConfig::with_backend("local-hash");
```

### Batch Size Selection

Choose batch size based on available memory:

```rust
// High memory (>16GB) - for production high-throughput
let config = EmbeddingServiceConfig {
    batch_size: 128,
    ..Default::default()
};

// Standard memory (8-16GB) - balanced
let config = EmbeddingServiceConfig {
    batch_size: 64,
    ..Default::default()
};

// Default (4-8GB) - safe for most systems
let config = EmbeddingServiceConfig {
    batch_size: 32,
    ..Default::default()
};

// Low memory (<4GB) - constrained environments
let config = EmbeddingServiceConfig {
    batch_size: 16,
    ..Default::default()
};
```

### Cache Configuration

```rust
// Enable caching (production)
let config = EmbeddingServiceConfig {
    cache_enabled: true,
    ..Default::default()
};

// Disable caching (development/testing)
let config = EmbeddingServiceConfig {
    cache_enabled: false,
    ..Default::default()
};

// Programmatic control
let service = EmbeddingService::new(config).await?;
service.clear_cache()?;
let cache_size = service.cache_size()?;
```

### Normalization Options

```rust
// Enable normalization (recommended for similarity)
let config = EmbeddingServiceConfig {
    normalize: true,
    ..Default::default()
};

// Disable normalization (for raw embedding values)
let config = EmbeddingServiceConfig {
    normalize: false,
    ..Default::default()
};
```

## Backend Capabilities

### Feature Matrix

| Feature | BGE-S | BGE-B | E5-S | E5-B | Nomic | Qwen | Local |
|---------|-------|-------|------|------|-------|------|-------|
| Dimension | 384 | 768 | 384 | 768 | 768 | 1024 | 256 |
| Batch Support | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| GPU Support | - | - | - | - | - | ✓ | - |
| CPU Support | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Query-Aware | - | - | ✓ | ✓ | - | - | - |
| Long Context | - | - | - | - | ✓ | - | - |
| Multilingual | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Cache Enabled | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Preset Configurations

### Development Configuration

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("local-hash"),
    batch_size: 16,
    cache_enabled: true,
    normalize: true,
};
```

### Production Configuration (General Search)

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-base"),
    batch_size: 64,
    cache_enabled: true,
    normalize: true,
};
```

### Production Configuration (Semantic Search)

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("e5-base"),
    batch_size: 64,
    cache_enabled: true,
    normalize: true,
};
```

### Production Configuration (Multilingual)

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("qwen"),
    batch_size: 32,  // Larger model needs smaller batch
    cache_enabled: true,
    normalize: true,
};
```

### Production Configuration (Long Documents)

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("nomic"),
    batch_size: 32,  // Smaller batches for long sequences
    cache_enabled: true,
    normalize: true,
};
```

## Backend Configuration

### Backend Config Structure

```rust
pub struct BackendConfig {
    pub provider: String,           // Backend identifier
    pub model_path: Option<String>, // Reserved for future
}

impl BackendConfig {
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model_path: None,
        }
    }

    pub fn with_model_path(mut self, path: impl Into<String>) -> Self {
        self.model_path = Some(path.into());
        self
    }
}
```

### Example Backend Configuration

```rust
// Standard backend
let config = BackendConfig::new("bge-small");

// With model path (reserved for future use)
let config = BackendConfig::new("bge-base")
    .with_model_path("/path/to/model");
```

## Runtime Configuration

### Service Inspection

```rust
let service = EmbeddingService::new(config).await?;

// Get backend info
let info = service.backend_info();
println!("Model: {}", info.model_name);
println!("Dimension: {}", info.dimension);
println!("Batch support: {}", info.supports_batch);

// Get metrics
let metrics = service.metrics();
println!("Total embeddings: {}", metrics.total_embeddings);
println!("Average time: {:.2}ms", metrics.average_embedding_time_ms);

// Check cache
let cache_size = service.cache_size()?;
println!("Cache entries: {}", cache_size);
```

### Dynamic Configuration

```rust
// Clear cache dynamically
service.clear_cache()?;

// Get current cache size
let size = service.cache_size()?;
println!("Cache size: {}", size);

// Get snapshot of metrics
let snapshot = service.metrics();
println!("{}", snapshot.to_string());
```

## Common Configuration Scenarios

### Scenario 1: High-Volume Document Processing

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-base"),
    batch_size: 128,  // Large batches
    cache_enabled: true,
    normalize: true,
};

let service = EmbeddingService::new(config).await?;

// Process large number of documents
let documents = vec![ /* ... */ ];
let embeddings = service.embed_batch(&documents).await?;
```

### Scenario 2: Real-time Query Processing

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("e5-small"),  // Fast model
    batch_size: 8,   // Small batches for latency
    cache_enabled: true,
    normalize: true,
};

let service = EmbeddingService::new(config).await?;

// Process single query or small batch
let embedding = service.embed(query_text).await?;
```

### Scenario 3: Memory-Constrained Environment

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),  // Smaller model
    batch_size: 8,   // Smaller batch
    cache_enabled: true,  // Still use cache
    normalize: true,
};

let service = EmbeddingService::new(config).await?;
```

### Scenario 4: Development/Testing

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("local-hash"),  // Fast for testing
    batch_size: 32,
    cache_enabled: false,  // No cache overhead
    normalize: false,  // Raw values for testing
};

let service = EmbeddingService::new(config).await?;
```

## Troubleshooting Configuration Issues

### Issue: Out of Memory

**Solution**: Reduce batch_size and use smaller model

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),
    batch_size: 16,  // Reduced from 32
    cache_enabled: true,
    normalize: true,
};
```

### Issue: High Latency

**Solution**: Use faster backend and optimize batch size

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),  // Instead of bge-base
    batch_size: 64,  // Larger batch for throughput
    cache_enabled: true,
    normalize: true,
};
```

### Issue: Cache Not Working

**Diagnosis**:
```rust
let service = EmbeddingService::new(config).await?;
let metrics = service.metrics();
println!("Cache hit ratio: {:.2}%", metrics.cache_hit_ratio * 100.0);
```

**Solution**: Ensure identical text strings and cache_enabled = true

### Issue: Inconsistent Results

**Solution**: Ensure normalize = true and same backend

```rust
let config = EmbeddingServiceConfig {
    normalize: true,  // Must normalize for consistency
    ..Default::default()
};
```

## Configuration Checklist

- [ ] Selected appropriate backend for use case
- [ ] Set batch_size based on available memory
- [ ] Enabled/disabled cache as needed
- [ ] Set normalize=true for similarity matching
- [ ] Tested configuration with sample data
- [ ] Monitored metrics for optimization
- [ ] Set up error handling
- [ ] Documented configuration decisions
