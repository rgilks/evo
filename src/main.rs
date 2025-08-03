mod batch_processor;
mod components;
mod config;
mod genes;
mod memory_mapped_storage;
mod profiler;
mod quadtree;
mod simulation;
mod spatial_grid;
mod spatial_hash;
mod spatial_system;
mod stats;
mod systems;
mod ui;

// GPU acceleration modules
mod gpu_spatial_system;
// mod gpu_movement_system;
// mod hybrid_simulation;

// GPU test module
mod gpu_test;

// GPU simulation module
mod gpu_simulation;

// GPU spatial benchmark module
mod gpu_spatial_benchmark;

use clap::Parser;
use config::SimulationConfig;
// use hybrid_simulation::HybridSimulation;
use pollster;

// fn initialize_gpu() -> (wgpu::Device, wgpu::Queue) {
//     let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//         backends: wgpu::Backends::all(),
//         dx12_shader_compiler: Default::default(),
//         flags: wgpu::InstanceFlags::default(),
//         gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
//     });
//
//     let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
//         power_preference: wgpu::PowerPreference::default(),
//         dx12_shader_compiler: Default::default(),
//         force_fallback_adapter: false,
//     })).expect("Failed to find an appropriate adapter");
//
//     pollster::block_on(adapter.request_device(
//         &wgpu::DeviceDescriptor {
//             required_features: wgpu::Features::empty(),
//             required_limits: wgpu::Limits::default(),
//             label: None,
//         },
//         None,
//     )).expect("Failed to create device")
// }

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in headless mode (no UI)
    #[arg(long)]
    headless: bool,

    /// Number of simulation steps to run in headless mode
    #[arg(short, long, default_value_t = 1000)]
    steps: u32,

    /// World size
    #[arg(short, long, default_value_t = 600.0)]
    world_size: f32,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Create a default configuration file
    #[arg(long)]
    create_config: Option<String>,

    /// Use GPU acceleration (experimental)
    #[arg(long)]
    gpu: bool,

    /// Test GPU functionality
    #[arg(long)]
    test_gpu: bool,

    /// Run GPU spatial benchmark
    #[arg(long)]
    benchmark_gpu: bool,
}

fn main() {
    let args = Args::parse();

    // Handle config file creation
    if let Some(config_path) = args.create_config {
        match SimulationConfig::create_default_config_file(&config_path) {
            Ok(_) => println!("Default configuration file created at: {}", config_path),
            Err(e) => eprintln!("Failed to create config file: {}", e),
        }
        return;
    }

    // Load configuration
    let config = if let Some(config_path) = args.config {
        match SimulationConfig::load_from_file(&config_path) {
            Ok(config) => {
                println!("Loaded configuration from: {}", config_path);
                config
            }
            Err(e) => {
                eprintln!("Failed to load config file: {}. Using defaults.", e);
                SimulationConfig::default()
            }
        }
    } else {
        SimulationConfig::default()
    };

    if args.test_gpu {
        println!("🧪 Testing GPU functionality...");
        match gpu_test::test_gpu_initialization() {
            Ok(_) => {
                println!("✅ GPU initialization test passed!");
                match gpu_test::test_gpu_operations() {
                    Ok(_) => println!("✅ All GPU tests passed!"),
                    Err(e) => eprintln!("❌ GPU operations test failed: {}", e),
                }
            }
            Err(e) => eprintln!("❌ GPU initialization test failed: {}", e),
        }
    } else if args.benchmark_gpu {
        println!("🏁 Running GPU spatial benchmark...");

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
        .expect("Failed to find an appropriate adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .expect("Failed to create device");

        println!(
            "✅ GPU initialized: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

        // Run quick test first
        match gpu_spatial_benchmark::quick_gpu_test(device, queue, args.world_size) {
            Ok(_) => {
                // Re-initialize GPU for the main benchmark
                let adapter =
                    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        compatible_surface: None,
                        force_fallback_adapter: false,
                    }))
                    .expect("Failed to find an appropriate adapter");

                let (device, queue) = pollster::block_on(adapter.request_device(
                    &wgpu::DeviceDescriptor {
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None,
                ))
                .expect("Failed to create device");

                // Run comprehensive benchmark
                let entity_counts = vec![100, 500, 1000, 5000, 10000];
                let results = gpu_spatial_benchmark::run_spatial_benchmark(
                    args.world_size,
                    &entity_counts,
                    50.0,
                );

                println!("\n📊 Final Benchmark Summary:");
                for result in results {
                    println!(
                        "  {} entities: {:.2}x speedup (GPU: {:.0} q/s vs CPU: {:.0} q/s)",
                        result.entity_count,
                        result.gpu_speedup,
                        result.gpu_queries_per_second,
                        result.cpu_queries_per_second
                    );
                }

                // Run large-scale benchmark
                println!("\n🚀 Testing large-scale performance...");
                if let Err(e) = gpu_spatial_benchmark::run_large_scale_benchmark(args.world_size) {
                    eprintln!("❌ Large-scale benchmark failed: {}", e);
                }
            }
            Err(e) => eprintln!("❌ GPU spatial test failed: {}", e),
        }
    } else if args.headless {
        println!("Running evolution simulation in headless mode...");

        if args.gpu {
            println!("🚀 Using GPU acceleration...");

            // Initialize GPU
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                dx12_shader_compiler: Default::default(),
                flags: wgpu::InstanceFlags::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            });

            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: None,
                    force_fallback_adapter: false,
                }))
                .expect("Failed to find an appropriate adapter");

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            ))
            .expect("Failed to create device");

            println!(
                "✅ GPU initialized: {} ({:?})",
                adapter.get_info().name,
                adapter.get_info().backend
            );

            // Use GPU simulation
            let mut sim =
                gpu_simulation::GpuSimulation::new(args.world_size, config.clone(), device, queue);

            for step in 0..args.steps {
                sim.update();
                if step % 10 == 0 {
                    let stats = stats::SimulationStats::from_world(
                        sim.world(),
                        config.max_population as f32,
                        config.entity_scale,
                    );
                    println!("{}", stats.format_summary(step));
                }
            }
        } else {
            // Use regular CPU simulation
            let mut sim = simulation::Simulation::new_with_config(args.world_size, config.clone());

            for step in 0..args.steps {
                sim.update();
                if step % 10 == 0 {
                    let stats = stats::SimulationStats::from_world(
                        sim.world(),
                        config.max_population as f32,
                        config.entity_scale,
                    );
                    println!("{}", stats.format_summary(step));
                }
            }
        }
        println!("Simulation complete!");
    } else {
        println!("Starting evolution simulation with UI...");
        ui::run(args.world_size, config);
    }
}
