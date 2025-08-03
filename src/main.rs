mod batch_processor;
mod components;
mod config;
mod genes;
mod profiler;
mod quadtree;
mod simulation;
mod spatial_grid;
mod spatial_system;
mod stats;
mod systems;
mod ui;

use clap::Parser;
use config::SimulationConfig;

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

    if args.headless {
        println!("Running evolution simulation in headless mode...");
        let mut sim = simulation::Simulation::new_with_config(args.world_size, config.clone());

        for step in 0..args.steps {
            sim.update();
            if step % 10 == 0 {
                // Use the new stats module for clean, consistent logging
                let stats = stats::SimulationStats::from_world(
                    sim.world(),
                    config.max_population as f32,
                    config.entity_scale,
                );
                println!("{}", stats.format_summary(step));
            }
        }
        println!("Simulation complete!");
    } else {
        println!("Starting evolution simulation with UI...");
        ui::run(args.world_size, config);
    }
}
