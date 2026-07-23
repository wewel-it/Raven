# Performance Tuning Guide

## Performance Characteristics

### Embedding Generation Speed

| Backend | Single (ms) | Batch 32 (ms/each) | Throughput (docs/sec) |
|---------|-------------|-------------------|----------------------|
| Local Hash | 0.1 | 0.03 | ~33,000 |
| BGE-Small | 1.0 | 0.5 | ~2,000 |
| BGE-Base | 1.5 | 0.8 | ~1,250 |
| E5-Small | 1.0 | 0.5 | ~2,000 |
| E5-Base | 1.5 | 0.8 | ~1,250 |
| Nomic | 2.0 | 1.0 | ~1,000 |
| Qwen | 3.0 | 1.5 | ~667 |

### Memory Usage

| Backend | Model Size | Per-Embedding | Batch 32 |
|---------|-----------|----------------|----------|
| Local Hash | Minimal | 1 KB | 32 KB |
| BGE-Small | Light | 1.5 KB | 48 KB |
| BGE-Base | Medium | 3 KB | 96 KB |
| Nomic | Medium | 3 KB | 96 KB |
| Qwen | Heavy | 4 KB | 128 KB |

*Note: These are approximate values for DenseVector storage only*

## Optimization Strategies

### 1. Batch Processing Optimization

**Problem**: Processing texts one-by-one

```rust
// Inefficient
for text in texts {
    let emb = service.embed(text).await?;
}
```

**Solution**: Use batch processing

```rust
// Efficient
let embeddings = service.embed_batch(&texts).await?;
```

**Impact**: 10-100x faster depending on batch size

### 2. Batch Size Tuning

**Finding optimal batch size**:

```rust
async fn benchmark_batch_sizes() -> Result<(), Box<dyn std::error::Error>> {
    let texts = vec!["sample text"; 1000];
    
    for batch_size in &[8, 16, 32, 64, 128] {
        let config = EmbeddingServiceConfig {
            batch_size: *batch_size,
            ..Default::default()
        };
        
        let service = EmbeddingService::new(config).await?;
        
        let start = std::time::Instant::now();
        let _ = service.embed_batch(&texts).await?;
        let duration = start.elapsed();
        
        let throughput = texts.len() as f64 / duration.as_secs_f64();
        println!("Batch {}: {:.0} docs/sec", batch_size, throughput);
    }
    
    Ok(())
}
```

**Guidelines**:
- **Small batches (8-16)**: Low latency, lower throughput
- **Medium batches (32-64)**: Balanced throughput and latency
- **Large batches (128-256)**: High throughput, higher latency

### 3. Backend Selection Impact

**Throughput by backend**:

```
Local Hash  ████████████████████████████████ 30,000 docs/sec
BGE-Small   ██████ 2,000 docs/sec
BGE-Base    ████ 1,300 docs/sec
Qwen        ██ 700 docs/sec
```

**Decision guide**:
- **Max throughput**: Use Local Hash (for dev/testing)
- **Balanced**: Use BGE-Small (general purpose)
- **Quality**: Use BGE-Base or E5-Base
- **Multilingual**: Use Qwen (if quality matters more than speed)

### 4. Cache Utilization

**Measuring cache effectiveness**:

```rust
let service = EmbeddingService::new(config).await?;

// Embed with repeated texts
let texts = vec!["common", "common", "unique1", "unique2", "common"];
let _ = service.embed_batch(&texts).await?;

let metrics = service.metrics();
println!("Cache hit ratio: {:.2}%", metrics.cache_hit_ratio * 100.0);
```

**Optimization strategies**:
- **High repetition**: Keep cache_enabled=true
- **One-time texts**: Can disable cache (saves memory)
- **Memory limited**: Clear cache periodically

```rust
// Monitor cache
let size = service.cache_size()?;
if size > 100_000 {
    service.clear_cache()?;  // Clear if too large
}
```

## Latency Optimization

### For Real-Time Applications

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),  // Fastest model
    batch_size: 8,   // Small batch for low latency
    cache_enabled: true,
    normalize: true,
};

let service = EmbeddingService::new(config).await?;

// Target: <5ms per query
let start = std::time::Instant::now();
let embedding = service.embed(query).await?;
let latency = start.elapsed().as_millis();

println!("Query latency: {}ms", latency);
assert!(latency < 5);
```

### Latency Targets

| Requirement | Batch Size | Backend | Target |
|-------------|-----------|---------|--------|
| <1ms | 1 | Local Hash | Achievable |
| <5ms | 8 | BGE-Small | Achievable |
| <10ms | 16 | BGE-Base | Achievable |
| <100ms | 32+ | Any | Achievable |

## Throughput Optimization

### For Batch Processing

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-base"),  // Best quality/speed
    batch_size: 128,  // Large batch for throughput
    cache_enabled: true,
    normalize: true,
};

let service = EmbeddingService::new(config).await?;

// Target: 1000+ docs/sec
let start = std::time::Instant::now();
let embeddings = service.embed_batch(&large_document_list).await?;
let duration = start.elapsed();

let throughput = embeddings.len() as f64 / duration.as_secs_f64();
println!("Throughput: {:.0} docs/sec", throughput);
```

### Throughput Targets

| Volume | Backend | Batch | Expected |
|--------|---------|-------|----------|
| <1K | Any | 32 | 1,000+ docs/sec |
| 1K-100K | BGE-Base | 64 | 1,200+ docs/sec |
| 100K+ | BGE-Base | 128 | 1,500+ docs/sec |

## Memory Optimization

### For Memory-Constrained Environments

```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),  // Smaller model
    batch_size: 16,  // Smaller batches
    cache_enabled: true,  // But keep cache
    normalize: true,
};

let service = EmbeddingService::new(config).await?;

// Monitor memory usage
let start_metrics = service.metrics();
let _ = service.embed_batch(&texts).await?;
let end_metrics = service.metrics();

println!("Memory usage: {:?}", end_metrics);
```

### Memory Reduction Strategies

1. **Use smaller models**: BGE-Small instead of BGE-Base
2. **Reduce batch size**: Trade throughput for memory
3. **Disable cache** (if not needed): `cache_enabled: false`
4. **Use Local Hash** for testing/development

## Concurrent Processing

### Handling Multiple Queries

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = EmbeddingServiceConfig::new();
    let service = Arc::new(EmbeddingService::new(config).await?);
    
    let queries = vec!["query1", "query2", "query3"];
    let tasks: Vec<_> = queries.into_iter().map(|q| {
        let s = service.clone();
        tokio::spawn(async move {
            s.embed(q).await
        })
    }).collect();
    
    // All run concurrently
    let results = futures::future::join_all(tasks).await;
    Ok(())
}
```

## Profiling and Monitoring

### Using Metrics

```rust
use std::time::Instant;

let service = EmbeddingService::new(config).await?;

// Warm up
let _ = service.embed("warmup").await?;

// Measure
let start = Instant::now();
let embeddings = service.embed_batch(&texts).await?;
let duration = start.elapsed();

// Analyze
let metrics = service.metrics();
println!("=== Performance Metrics ===");
println!("Duration: {:?}", duration);
println!("Throughput: {:.0} docs/sec", 
         texts.len() as f64 / duration.as_secs_f64());
println!("Avg embedding time: {:.2}ms", 
         metrics.average_embedding_time_ms);
println!("Cache hit ratio: {:.2}%", 
         metrics.cache_hit_ratio * 100.0);
println!("Total embeddings: {}", metrics.total_embeddings);
```

### Benchmarking Example

```rust
#[tokio::main]
async fn benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let config = EmbeddingServiceConfig::with_backend("bge-base");
    let service = EmbeddingService::new(config).await?;
    
    // Generate test data
    let texts: Vec<String> = (0..10_000)
        .map(|i| format!("Document {} with some content", i))
        .collect();
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    
    // Benchmark batch embedding
    let start = std::time::Instant::now();
    let _ = service.embed_batch(&text_refs).await?;
    let duration = start.elapsed();
    
    let metrics = service.metrics();
    println!("Benchmark Results:");
    println!("  Documents: {}", text_refs.len());
    println!("  Time: {:.2}s", duration.as_secs_f64());
    println!("  Throughput: {:.0} docs/sec", 
             text_refs.len() as f64 / duration.as_secs_f64());
    println!("  Avg per doc: {:.2}ms",
             metrics.average_embedding_time_ms);
    
    Ok(())
}
```

## Common Performance Issues

### Issue 1: Low Cache Hit Ratio

**Symptom**: Cache hit ratio < 20%

**Causes**:
- Too many unique texts
- Cache cleared too frequently

**Solutions**:
```rust
// Check if caching helps
let before = service.cache_size()?;
let _ = service.embed_batch(&texts).await?;
let metrics = service.metrics();

if metrics.cache_hit_ratio < 0.2 {
    // Consider disabling cache to save memory
    let config = EmbeddingServiceConfig {
        cache_enabled: false,
        ..Default::default()
    };
}
```

### Issue 2: High Latency

**Symptom**: Embedding takes >100ms per query

**Causes**:
- Large batch size
- Too many documents in batch
- Wrong backend chosen

**Solutions**:
```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),  // Faster
    batch_size: 8,  // Smaller
    cache_enabled: true,
    normalize: true,
};
```

### Issue 3: OOM Errors

**Symptom**: Memory allocation failure

**Causes**:
- Batch size too large
- Model too large for available RAM

**Solutions**:
```rust
let config = EmbeddingServiceConfig {
    backend: BackendConfig::new("bge-small"),  // Smaller model
    batch_size: 16,  // Smaller batch
    cache_enabled: false,  // Disable cache
    normalize: true,
};
```

## Performance Checklist

- [ ] Profiled with actual data
- [ ] Batch size optimized for use case
- [ ] Backend selected for requirements
- [ ] Cache effectiveness monitored
- [ ] Latency targets verified
- [ ] Throughput meets requirements
- [ ] Memory usage acceptable
- [ ] Error handling in place
- [ ] Metrics enabled for monitoring
- [ ] Concurrent operation tested
