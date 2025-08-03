use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evo::config::SimulationConfig;
use evo::profiler::PerformanceAnalyzer;
use evo::simulation::Simulation;
use tracing_subscriber;

fn setup_logging() {
    tracing_subscriber::fmt().with_env_filter("info").init();
}

fn benchmark_simulation_update(c: &mut Criterion) {
    setup_logging();

    let mut group = c.benchmark_group("simulation_update");

    // Test different world sizes
    for world_size in [300.0, 600.0, 1200.0] {
        group.bench_function(&format!("world_size_{}", world_size as i32), |b| {
            let mut sim = Simulation::new(world_size);
            b.iter(|| {
                black_box(sim.update());
            });
        });
    }

    group.finish();
}

fn benchmark_spatial_grid(c: &mut Criterion) {
    setup_logging();

    let mut group = c.benchmark_group("spatial_grid");

    // Test different entity counts
    for entity_count in [100, 500, 1000, 2000] {
        group.bench_function(&format!("entity_count_{}", entity_count), |b| {
            let mut config = SimulationConfig::default();
            config.initial_entities = entity_count;
            let mut sim = Simulation::new_with_config(600.0, config);

            b.iter(|| {
                black_box(sim.update());
            });
        });
    }

    group.finish();
}

fn benchmark_entity_processing(c: &mut Criterion) {
    setup_logging();

    let mut group = c.benchmark_group("entity_processing");

    // Test different entity densities
    for entity_scale in [0.5, 1.0, 2.0, 4.0] {
        group.bench_function(&format!("entity_scale_{}", entity_scale), |b| {
            let mut config = SimulationConfig::default();
            config.entity_scale = entity_scale;
            let mut sim = Simulation::new_with_config(600.0, config);

            b.iter(|| {
                black_box(sim.update());
            });
        });
    }

    group.finish();
}

fn benchmark_movement_system(c: &mut Criterion) {
    setup_logging();

    let mut group = c.benchmark_group("movement_system");

    // Test different grid cell sizes
    for cell_size in [10.0, 25.0, 50.0, 100.0] {
        group.bench_function(&format!("cell_size_{}", cell_size as i32), |b| {
            let mut config = SimulationConfig::default();
            config.grid_cell_size = cell_size;
            let mut sim = Simulation::new_with_config(600.0, config);

            b.iter(|| {
                black_box(sim.update());
            });
        });
    }

    group.finish();
}

fn benchmark_interaction_system(c: &mut Criterion) {
    setup_logging();

    let mut group = c.benchmark_group("interaction_system");

    // Test different sense radii
    for sense_radius in [10.0, 25.0, 50.0, 100.0] {
        group.bench_function(&format!("sense_radius_{}", sense_radius as i32), |b| {
            let mut config = SimulationConfig::default();
            // We'll need to modify the genes to test different sense radii
            let mut sim = Simulation::new_with_config(600.0, config);

            b.iter(|| {
                black_box(sim.update());
            });
        });
    }

    group.finish();
}

fn benchmark_profiling_overhead(c: &mut Criterion) {
    setup_logging();

    let mut group = c.benchmark_group("profiling_overhead");

    // Test with and without profiling
    group.bench_function("without_profiling", |b| {
        let mut sim = Simulation::new(600.0);
        b.iter(|| {
            black_box(sim.update());
        });
    });

    group.bench_function("with_profiling", |b| {
        let mut sim = Simulation::new(600.0);
        let mut analyzer = PerformanceAnalyzer::new(true, 100);

        b.iter(|| {
            analyzer.profiler().start_timer("simulation_update");
            black_box(sim.update());
            analyzer.profiler().stop_timer("simulation_update");
            analyzer.step();
        });
    });

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    setup_logging();

    let mut group = c.benchmark_group("memory_usage");

    // Test memory usage with different entity counts
    for entity_count in [100, 500, 1000, 2000, 5000] {
        group.bench_function(&format!("memory_entities_{}", entity_count), |b| {
            b.iter(|| {
                let mut config = SimulationConfig::default();
                config.initial_entities = entity_count;
                let mut sim = Simulation::new_with_config(600.0, config);

                // Run a few steps to stabilize memory usage
                for _ in 0..10 {
                    sim.update();
                }

                black_box(sim);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_simulation_update,
    benchmark_spatial_grid,
    benchmark_entity_processing,
    benchmark_movement_system,
    benchmark_interaction_system,
    benchmark_profiling_overhead,
    benchmark_memory_usage
);
criterion_main!(benches);
