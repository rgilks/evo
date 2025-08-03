mod simulation;
mod ui;

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
}

fn main() {
    let args = Args::parse();

    if args.headless {
        println!("Running evolution simulation in headless mode...");
        let mut sim = simulation::Simulation::new(args.world_size);

        let mut last_stats = (0, 0, 0); // (passive, neutral, aggressive)

        for step in 0..args.steps {
            sim.update();
            if step % 10 == 0 {
                let entities = sim.get_entities();
                // Count entities by color ranges for more detailed color analysis
                let red_dominant = entities
                    .iter()
                    .filter(|(_, _, _, r, g, b)| *r > 0.6 && *r > *g && *r > *b)
                    .count();
                let green_dominant = entities
                    .iter()
                    .filter(|(_, _, _, r, g, b)| *g > 0.6 && *g > *r && *g > *b)
                    .count();
                let blue_dominant = entities
                    .iter()
                    .filter(|(_, _, _, r, g, b)| *b > 0.6 && *b > *r && *b > *g)
                    .count();
                let purple_colors = entities
                    .iter()
                    .filter(|(_, _, _, r, g, b)| {
                        // Purple: high red and blue, low green
                        *r > 0.5 && *b > 0.5 && *g < 0.4
                    })
                    .count();
                let mixed_colors = entities
                    .iter()
                    .filter(|(_, _, _, r, g, b)| {
                        let max = (*r).max(*g).max(*b);
                        // Not dominant in any color and not purple
                        (max <= 0.6 || ((*r - *g).abs() < 0.2 && (*g - *b).abs() < 0.2))
                            && !(*r > 0.5 && *b > 0.5 && *g < 0.4)
                    })
                    .count();

                let current_stats = (red_dominant, green_dominant, blue_dominant);
                let population_change = if step > 0 {
                    let total_change = (red_dominant
                        + green_dominant
                        + blue_dominant
                        + purple_colors
                        + mixed_colors) as i32
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
                    "Step {}: {} entities (Red:{} Green:{} Blue:{} Purple:{} Mixed:{}) {}",
                    step,
                    entities.len(),
                    red_dominant,
                    green_dominant,
                    blue_dominant,
                    purple_colors,
                    mixed_colors,
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
