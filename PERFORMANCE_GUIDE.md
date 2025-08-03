# Performance Optimization Guide

This guide provides comprehensive information on profiling and optimizing the evolution simulation application.

## Quick Start

### 1. Run Basic Profiling

```bash
# Run the profiling script
./scripts/profile.sh

# Or run individual profiling tools
cargo bench --bench simulation_benchmark
cargo run --release -- --headless --steps 1000
```

### 2. Install Profiling Tools

```bash
# Install flamegraph for CPU profiling
cargo install flamegraph

# Install valgrind for memory profiling (macOS: brew install valgrind)
# Linux: sudo apt-get install valgrind
```

## Profiling Tools

### 1. Built-in Profiler

The application includes a built-in profiler that tracks:

- Execution time of different systems
- Entity processing performance
- Spatial grid operations
- Memory usage patterns

**Usage:**

```rust
// Profiling is automatically enabled in release mode
cargo run --release -- --headless --steps 1000
```

### 2. Criterion Benchmarks

Comprehensive benchmarks for different components:

```bash
cargo bench --bench simulation_benchmark
```

### 3. Flamegraph (CPU Profiling)

Generate CPU usage flamegraphs:

```bash
flamegraph --output flamegraph.svg -- cargo run --release -- --headless --steps 1000
```

### 4. Valgrind (Memory Profiling)

Memory leak detection and analysis:

```bash
valgrind --tool=massif --massif-out-file=massif.out cargo run --release -- --headless --steps 100
ms_print massif.out > memory_profile.txt
```

## Common Bottlenecks and Solutions

### 1. Spatial Grid Performance

**Symptoms:**

- Slow neighbor finding
- High CPU usage in grid operations
- Poor scaling with entity count

**Solutions:**

```rust
// Optimize cell size based on entity density
config.grid_cell_size = (world_size / (entity_count as f32).sqrt()).max(10.0);

// Use spatial partitioning for large worlds
// Implement quadtree for better performance
```

### 2. Entity Processing Bottlenecks

**Symptoms:**

- Slow simulation updates
- High CPU usage in entity processing
- Poor parallelization

**Solutions:**

```rust
// Optimize parallel processing
rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .build_global()
    .unwrap();

// Reduce nearby entity checks
let nearby_limit = 20; // Limit nearby entity processing
```

### 3. Memory Usage Issues

**Symptoms:**

- High memory consumption
- Memory leaks
- Poor performance with large entity counts

**Solutions:**

```rust
// Implement object pooling
// Use more efficient data structures
// Reduce entity count or world size
```

### 4. Rendering Performance

**Symptoms:**

- Low frame rates
- GPU bottleneck
- UI lag

**Solutions:**

```rust
// Optimize vertex buffer updates
// Use instanced rendering
// Reduce entity rendering frequency
```

## Performance Configuration

### Optimal Settings by Use Case

#### High Performance (Large Scale)

```json
{
  "initial_entities": 1000,
  "entity_scale": 1.0,
  "grid_cell_size": 50.0,
  "max_population": 2000,
  "world_size": 1200.0
}
```

#### Balanced Performance

```json
{
  "initial_entities": 500,
  "entity_scale": 1.0,
  "grid_cell_size": 25.0,
  "max_population": 1000,
  "world_size": 600.0
}
```

#### High Quality (Small Scale)

```json
{
  "initial_entities": 200,
  "entity_scale": 1.0,
  "grid_cell_size": 15.0,
  "max_population": 500,
  "world_size": 400.0
}
```

## Optimization Techniques

### 1. Algorithmic Optimizations

#### Spatial Partitioning

- Use quadtree for large worlds
- Implement hierarchical spatial structures
- Optimize neighbor search algorithms

#### Entity Processing

- Batch entity updates
- Use efficient data structures
- Implement culling for off-screen entities

### 2. Parallelization

#### Rayon Thread Pool

```rust
// Configure thread pool for optimal performance
rayon::ThreadPoolBuilder::new()
    .num_threads(num_cpus::get())
    .stack_size(32 * 1024 * 1024) // 32MB stack
    .build_global()
    .unwrap();
```

#### Parallel Processing Patterns

- Process entities in parallel
- Parallel spatial grid updates
- Concurrent system execution

### 3. Memory Optimizations

#### Data Structure Optimization

```rust
// Use more efficient data structures
use std::collections::HashMap;
use dashmap::DashMap; // For concurrent access

// Object pooling for frequently allocated objects
use object_pool::Pool;
```

#### Memory Layout

- Structure of Arrays (SoA) vs Array of Structures (AoS)
- Cache-friendly data access patterns
- Minimize memory allocations

### 4. Rendering Optimizations

#### GPU Optimizations

```rust
// Use instanced rendering for entities
// Implement frustum culling
// Optimize shader performance
```

#### UI Optimizations

- Reduce update frequency
- Implement level-of-detail rendering
- Use efficient UI frameworks

## Monitoring and Analysis

### 1. Real-time Monitoring

```rust
// Enable real-time performance monitoring
let mut analyzer = PerformanceAnalyzer::new(true, 60);
analyzer.profiler().start_timer("frame_time");
// ... frame processing ...
analyzer.profiler().stop_timer("frame_time");
```

### 2. Performance Metrics

Track these key metrics:

- Frame time (target: <16ms for 60 FPS)
- Entity processing time
- Memory usage
- CPU utilization
- GPU utilization

### 3. Profiling Output Analysis

```bash
# Analyze profiling results
grep "Slow operation" profiling_output/*.log
grep "Performance" profiling_output/*.log
```

## Troubleshooting

### Common Issues

#### 1. High CPU Usage

**Cause:** Inefficient algorithms or poor parallelization
**Solution:** Profile and optimize slow operations

#### 2. Memory Leaks

**Cause:** Unmanaged allocations or circular references
**Solution:** Use memory profilers and fix leaks

#### 3. Poor Scaling

**Cause:** Algorithmic complexity or synchronization overhead
**Solution:** Optimize algorithms and reduce contention

#### 4. Low Frame Rates

**Cause:** Rendering bottlenecks or inefficient updates
**Solution:** Optimize rendering pipeline and reduce update frequency

### Debugging Tools

#### 1. Built-in Debugging

```rust
// Enable debug logging
RUST_LOG=debug cargo run --release

// Enable performance logging
RUST_LOG=info cargo run --release
```

#### 2. External Tools

- **perf** (Linux): CPU profiling
- **Instruments** (macOS): Performance analysis
- **VTune** (Intel): Advanced profiling

## Best Practices

### 1. Development Workflow

1. Profile early and often
2. Set performance benchmarks
3. Monitor performance regressions
4. Use automated performance testing

### 2. Optimization Strategy

1. Measure first (identify bottlenecks)
2. Optimize the slowest operations
3. Re-measure and iterate
4. Consider algorithmic improvements

### 3. Code Quality

1. Write efficient algorithms
2. Use appropriate data structures
3. Minimize allocations
4. Profile in release mode

## Advanced Topics

### 1. SIMD Optimizations

```rust
// Use SIMD for vector operations
use std::simd::*;

// Optimize mathematical operations
let positions: [f32; 4] = [x1, y1, x2, y2];
let velocities: [f32; 4] = [vx1, vy1, vx2, vy2];
```

### 2. Lock-free Programming

```rust
// Use lock-free data structures for concurrent access
use crossbeam::queue::ArrayQueue;
use parking_lot::RwLock;
```

### 3. Custom Allocators

```rust
// Implement custom allocators for specific use cases
use std::alloc::{GlobalAlloc, Layout};
```

## Resources

### Documentation

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rayon Documentation](https://docs.rs/rayon/)
- [Criterion.rs Guide](https://bheisler.github.io/criterion.rs/book/)

### Tools

- [flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [criterion](https://github.com/bheisler/criterion.rs)
- [valgrind](https://valgrind.org/)

### Community

- [Rust Performance Working Group](https://github.com/rust-lang/wg-performance)
- [Rust Forum - Performance](https://users.rust-lang.org/c/performance)
