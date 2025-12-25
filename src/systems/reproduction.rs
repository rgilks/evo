use crate::components::{Energy, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use rand::prelude::*;

/// Reproduction system - handles entity reproduction and population control
pub struct ReproductionSystem;

impl ReproductionSystem {
    pub fn check_reproduction(
        &self,
        energy: f32,
        max_energy: f32,
        genes: &Genes,
        population_density: f32,
        config: &SimulationConfig,
    ) -> bool {
        let reproduction_chance = genes.reproduction_rate()
            * (1.0 - population_density * config.reproduction.population_density_factor)
                .max(config.reproduction.min_reproduction_chance);

        energy > max_energy * config.reproduction.reproduction_energy_threshold
            && thread_rng().gen::<f32>() < reproduction_chance
    }

    pub fn create_offspring(
        &self,
        parent_genes: &Genes,
        parent_energy_max: f32,
        parent_pos: &Position,
        config: &SimulationConfig,
    ) -> (
        Position,
        Energy,
        Size,
        Genes,
        crate::components::Color,
        Velocity,
        crate::components::MovementStyle,
    ) {
        let mut rng = thread_rng();
        let child_genes = parent_genes.mutate(&mut rng);
        let child_energy = parent_energy_max * config.reproduction.child_energy_factor;
        let child_radius = (child_energy / 15.0 * child_genes.size_factor())
            .clamp(config.physics.min_entity_radius, 15.0);
        let child_color = child_genes.get_color();

        // Use uniform distribution in a circle for child positioning
        let (dx, dy) = loop {
            let dx = rng.gen_range(
                -config.reproduction.child_spawn_radius..config.reproduction.child_spawn_radius,
            );
            let dy = rng.gen_range(
                -config.reproduction.child_spawn_radius..config.reproduction.child_spawn_radius,
            );
            let distance_sq = dx * dx + dy * dy;
            if distance_sq
                <= config.reproduction.child_spawn_radius * config.reproduction.child_spawn_radius
            {
                break (dx, dy);
            }
        };

        (
            Position {
                x: parent_pos.x + dx,
                y: parent_pos.y + dy,
            },
            Energy {
                current: child_energy,
                max: parent_energy_max,
            },
            Size {
                radius: child_radius,
            },
            child_genes.clone(),
            child_color,
            Velocity { x: 0.0, y: 0.0 },
            child_genes.behavior.movement_style.clone(),
        )
    }

    pub fn check_death(&self, population_density: f32, config: &SimulationConfig) -> bool {
        let death_chance = population_density * config.reproduction.death_chance_factor;
        thread_rng().gen::<f32>() < death_chance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Position;
    use crate::config::SimulationConfig;
    use crate::genes::Genes;
    use rand::thread_rng;

    #[test]
    fn test_reproduction_system_check_reproduction() {
        let system = ReproductionSystem;
        let energy = 90.0;
        let max_energy = 100.0;
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let population_density = 0.1; // Low density for higher reproduction chance
        let config = SimulationConfig::default();

        let _should_reproduce =
            system.check_reproduction(energy, max_energy, &genes, population_density, &config);
    }

    #[test]
    fn test_reproduction_system_create_offspring() {
        let system = ReproductionSystem;
        let mut rng = thread_rng();
        let parent_genes = Genes::new_random(&mut rng);
        let parent_energy_max = 100.0;
        let parent_pos = Position { x: 0.0, y: 0.0 };
        let config = SimulationConfig::default();

        let (pos, energy, size, _genes, color, velocity, _movement_style) =
            system.create_offspring(&parent_genes, parent_energy_max, &parent_pos, &config);

        // Position should be near parent
        let distance = ((pos.x - parent_pos.x).powi(2) + (pos.y - parent_pos.y).powi(2)).sqrt();
        assert!(distance <= config.reproduction.child_spawn_radius);

        // Energy should be reasonable
        assert!(energy.current > 0.0);
        assert!(energy.current <= energy.max);

        // Size should be reasonable
        assert!(size.radius > 0.0);
        assert!(size.radius <= config.physics.max_entity_radius);

        // Color should be valid
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);

        // Velocity should be reasonable
        assert!(velocity.x.abs() <= config.physics.max_velocity);
        assert!(velocity.y.abs() <= config.physics.max_velocity);
    }

    #[test]
    fn test_reproduction_system_check_death() {
        let system = ReproductionSystem;
        let population_density = 0.9; // High density
        let config = SimulationConfig::default();

        let _should_die = system.check_death(population_density, &config);
    }

    #[test]
    fn test_reproduction_system_low_energy() {
        let system = ReproductionSystem;
        let energy = 10.0;
        let max_energy = 100.0;
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let population_density = 0.1; // Low density
        let config = SimulationConfig::default();

        let should_reproduce =
            system.check_reproduction(energy, max_energy, &genes, population_density, &config);

        // Should not reproduce with low energy
        assert!(!should_reproduce);
    }

    #[test]
    fn test_reproduction_system_drift() {
        use crate::config::SimulationConfig;
        use crate::simulation::Simulation;

        let mut config = SimulationConfig::default();
        // Disable interactions to isolate reproduction effects
        config.physics.interaction_radius_offset = 0.0; // No interactions

        let world_size = 100.0;
        let mut simulation = Simulation::new_with_config(world_size, config);

        println!("Testing reproduction system drift (interactions disabled)...");

        let mut positions = Vec::new();

        // Run simulation for 100 steps, recording positions every 20 steps
        for step in 0..100 {
            simulation.update();

            if step % 20 == 0 {
                let entities = simulation.get_entities();
                let mut total_x = 0.0;
                let mut total_y = 0.0;

                for (_px, _py, cx, cy, _, _, _, _) in &entities {
                    total_x += cx;
                    total_y += cy;
                }

                let center_x = total_x / entities.len() as f32;
                let center_y = total_y / entities.len() as f32;
                positions.push((step, center_x, center_y));

                println!(
                    "Step {}: {} entities, Center ({:.1}, {:.1})",
                    step,
                    entities.len(),
                    center_x,
                    center_y
                );
            }
        }

        // Analyze drift direction
        if positions.len() >= 2 {
            let first = positions[0];
            let last = positions[positions.len() - 1];
            let drift_x = last.1 - first.1;
            let drift_y = last.2 - first.2;

            println!("\nReproduction System Drift Analysis:");
            println!("Start position: ({:.1}, {:.1})", first.1, first.2);
            println!("End position: ({:.1}, {:.1})", last.1, last.2);
            println!("Total drift: ({:.1}, {:.1})", drift_x, drift_y);

            if drift_x.abs() > 5.0 || drift_y.abs() > 5.0 {
                println!("REPRODUCTION SYSTEM IS CAUSING DRIFT!");
            } else {
                println!("Reproduction system appears unbiased");
            }
        }
    }
}
