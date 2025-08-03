use crate::components::{Position, Size};
use crate::gpu_spatial_system::GpuSpatialSystem;
use crate::spatial_system::SpatialSystem;
use hecs::Entity;
use std::time::Instant;
use wgpu::{Device, Queue};

/// Benchmark results for spatial query performance
#[derive(Debug)]
pub struct SpatialBenchmarkResults {
    pub entity_count: usize,
    pub query_count: usize,
    pub cpu_time_ms: f64,
    pub gpu_time_ms: f64,
    pub gpu_speedup: f64,
    pub cpu_queries_per_second: f64,
    pub gpu_queries_per_second: f64,
}

impl SpatialBenchmarkResults {
    pub fn print_summary(&self) {
        println!("=== Spatial Query Performance Benchmark ===");
        println!("Entity Count: {}", self.entity_count);
        println!("Query Count: {}", self.query_count);
        println!("CPU Time: {:.2}ms", self.cpu_time_ms);
        println!("GPU Time: {:.2}ms", self.gpu_time_ms);
        println!("GPU Speedup: {:.2}x", self.gpu_speedup);
        println!("CPU Queries/sec: {:.0}", self.cpu_queries_per_second);
        println!("GPU Queries/sec: {:.0}", self.gpu_queries_per_second);
        println!("==========================================");
    }
}

/// Run a comprehensive benchmark comparing CPU vs GPU spatial queries
pub fn run_spatial_benchmark(
    world_size: f32,
    entity_counts: &[usize],
    query_radius: f32,
) -> Vec<SpatialBenchmarkResults> {
    let mut results = Vec::new();

    for &entity_count in entity_counts {
        println!("Benchmarking with {} entities...", entity_count);

        // Generate test entities
        let entities = generate_test_entities(entity_count, world_size);

        // Initialize CPU spatial system
        let mut cpu_spatial = SpatialSystem::new(world_size, entity_count);
        for (entity, pos, _) in &entities {
            cpu_spatial.insert(*entity, pos.x, pos.y);
        }

        // Generate test queries
        let queries = generate_test_queries(1000, world_size);

        // Benchmark CPU queries
        let cpu_start = Instant::now();
        let mut cpu_results = Vec::new();
        for (x, y) in &queries {
            let nearby = cpu_spatial.get_nearby_entities(*x, *y, query_radius);
            cpu_results.push(nearby.len());
        }
        let cpu_time = cpu_start.elapsed().as_secs_f64() * 1000.0;

        // Initialize GPU and benchmark GPU queries
        let gpu_time = if let Ok(gpu_result) =
            benchmark_gpu_queries(&entities, &queries, world_size, entity_count, query_radius)
        {
            gpu_result
        } else {
            println!("⚠️  GPU benchmark failed, using CPU time as fallback");
            cpu_time
        };

        let result = SpatialBenchmarkResults {
            entity_count,
            query_count: queries.len(),
            cpu_time_ms: cpu_time,
            gpu_time_ms: gpu_time,
            gpu_speedup: cpu_time / gpu_time,
            cpu_queries_per_second: (queries.len() as f64) / (cpu_time / 1000.0),
            gpu_queries_per_second: (queries.len() as f64) / (gpu_time / 1000.0),
        };

        result.print_summary();
        results.push(result);
    }

    results
}

/// Run a large-scale benchmark to test GPU performance with many entities
pub fn run_large_scale_benchmark(world_size: f32) -> Result<(), String> {
    println!("🚀 Running large-scale GPU spatial benchmark...");

    // Test with much larger entity counts
    let entity_counts = vec![10000, 25000, 50000, 100000];

    for entity_count in entity_counts {
        println!("Testing with {} entities...", entity_count);

        // Generate test entities
        let entities = generate_test_entities(entity_count, world_size);

        // Initialize CPU spatial system
        let mut cpu_spatial = SpatialSystem::new(world_size, entity_count);
        for (entity, pos, _) in &entities {
            cpu_spatial.insert(*entity, pos.x, pos.y);
        }

        // Generate fewer test queries for large-scale test
        let queries = generate_test_queries(100, world_size);

        // Benchmark CPU queries
        let cpu_start = Instant::now();
        for (x, y) in &queries {
            let _nearby = cpu_spatial.get_nearby_entities(*x, *y, 50.0);
        }
        let cpu_time = cpu_start.elapsed().as_secs_f64() * 1000.0;

        // Initialize GPU and benchmark GPU queries
        let gpu_time = if let Ok(gpu_result) =
            benchmark_gpu_queries(&entities, &queries, world_size, entity_count, 50.0)
        {
            gpu_result
        } else {
            println!("⚠️  GPU benchmark failed for {} entities", entity_count);
            continue;
        };

        let speedup = cpu_time / gpu_time;
        println!(
            "  {} entities: CPU {:.2}ms, GPU {:.2}ms, Speedup: {:.2}x",
            entity_count, cpu_time, gpu_time, speedup
        );

        if speedup > 1.0 {
            println!("  🎉 GPU is faster for {} entities!", entity_count);
        }
    }

    Ok(())
}

/// Benchmark GPU queries with a fresh device/queue
fn benchmark_gpu_queries(
    entities: &[(Entity, Position, Size)],
    queries: &[(f32, f32)],
    world_size: f32,
    entity_count: usize,
    query_radius: f32,
) -> Result<f64, String> {
    // Initialize GPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok_or("Failed to find an appropriate adapter")?;

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .map_err(|e| format!("Failed to create device: {:?}", e))?;

    // Initialize GPU spatial system
    let mut gpu_spatial = GpuSpatialSystem::new(device, queue, world_size, entity_count as u32);
    gpu_spatial.update_entities(entities);

    // Benchmark GPU queries
    let gpu_start = Instant::now();
    let mut gpu_results = Vec::new();
    for (x, y) in queries {
        let nearby = gpu_spatial.query_radius(*x, *y, query_radius);
        gpu_results.push(nearby.len());
    }
    let gpu_time = gpu_start.elapsed().as_secs_f64() * 1000.0;

    Ok(gpu_time)
}

/// Generate test entities with random positions
fn generate_test_entities(count: usize, world_size: f32) -> Vec<(Entity, Position, Size)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..count)
        .filter_map(|i| {
            // Try to create entity with different bit patterns
            let entity_bits = (i + 1) as u64;
            let entity = Entity::from_bits(entity_bits)?;

            let pos = Position {
                x: rng.gen_range(0.0..world_size),
                y: rng.gen_range(0.0..world_size),
            };

            let size = Size {
                radius: rng.gen_range(2.0..8.0),
            };

            Some((entity, pos, size))
        })
        .collect()
}

/// Generate test query points
fn generate_test_queries(count: usize, world_size: f32) -> Vec<(f32, f32)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    (0..count)
        .map(|_| {
            (
                rng.gen_range(0.0..world_size),
                rng.gen_range(0.0..world_size),
            )
        })
        .collect()
}

/// Run a quick performance test to verify GPU spatial system is working
pub fn quick_gpu_test(device: Device, queue: Queue, world_size: f32) -> Result<(), String> {
    println!("🧪 Running quick GPU spatial system test...");

    let entity_count = 1000;
    let entities = generate_test_entities(entity_count, world_size);

    let mut gpu_spatial = GpuSpatialSystem::new(device, queue, world_size, entity_count as u32);
    gpu_spatial.update_entities(&entities);

    // Test a few queries
    let test_queries = vec![
        (world_size / 2.0, world_size / 2.0),
        (100.0, 100.0),
        (500.0, 500.0),
    ];

    for (i, (x, y)) in test_queries.iter().enumerate() {
        let nearby = gpu_spatial.query_radius(*x, *y, 50.0);
        println!(
            "  Query {}: Found {} entities near ({:.1}, {:.1})",
            i + 1,
            nearby.len(),
            x,
            y
        );
    }

    println!("✅ GPU spatial system test completed successfully!");
    Ok(())
}
