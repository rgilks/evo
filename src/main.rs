mod simulation;
mod ui;
mod test_graphics;

use clap::Parser;

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

    /// Run graphics test (simple circle)
    #[arg(long)]
    test_graphics: bool,
}

fn main() {
    let args = Args::parse();

    if args.test_graphics {
        println!("Running graphics test...");
        test_graphics::test_graphics();
    } else if args.headless {
        println!("Running evolution simulation in headless mode...");
        let mut sim = simulation::Simulation::new(args.world_size);

        let mut last_stats = (0, 0, 0); // (resources, herbivores, predators)

        for step in 0..args.steps {
            sim.update();
            if step % 10 == 0 {
                let entities = sim.get_entities();
                let resources = entities
                    .iter()
                    .filter(|(_, _, _, _, g, _)| *g > 0.7)
                    .count();
                let predators = entities
                    .iter()
                    .filter(|(_, _, _, r, _, _)| *r > 0.7)
                    .count();
                let herbivores = entities
                    .len()
                    .saturating_sub(resources)
                    .saturating_sub(predators);

                let current_stats = (resources, herbivores, predators);
                let population_change = if step > 0 {
                    let total_change = (resources + herbivores + predators) as i32
                        - (last_stats.0 + last_stats.1 + last_stats.2) as i32;
                    if total_change > 0 {
                        format!("+{}", total_change)
                    } else {
                        format!("{}", total_change)
                    }
                } else {
                    "".to_string()
                };

                println!(
                    "Step {}: {} entities (R:{} H:{} P:{}) {}",
                    step,
                    entities.len(),
                    resources,
                    herbivores,
                    predators,
                    population_change
                );

                last_stats = current_stats;
            }
        }
        println!("Simulation complete!");
    } else {
        println!("Starting evolution simulation with UI...");
        ui::run(args.world_size);
    }
}
