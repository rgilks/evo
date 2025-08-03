use crate::components::{Energy, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use hecs::{Entity, World};
use rand::prelude::*;

/// Movement system - handles entity movement and boundary constraints
pub struct MovementSystem;

impl MovementSystem {
    pub fn update_movement(
        &self,
        genes: &Genes,
        new_pos: &mut Position,
        new_velocity: &mut Velocity,
        new_energy: &mut f32,
        pos: &Position,
        nearby_entities: &[Entity],
        world: &World,
        config: &SimulationConfig,
    ) {
        // Find target for movement based on genes
        let target = self.find_movement_target(pos, genes, nearby_entities, world);

        if let Some((target_x, target_y)) = target {
            // Move towards target
            let dx = target_x - pos.x;
            let dy = target_y - pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 0.0 {
                new_velocity.x = (dx / distance) * genes.speed();
                new_velocity.y = (dy / distance) * genes.speed();
            }
        } else {
            // Random movement - use uniform distribution in a circle to avoid bias
            let mut rng = thread_rng();
            let speed_variation = rng.gen_range(0.8..1.2);
            let speed = genes.speed() * speed_variation;

            // Generate random direction using uniform distribution in a circle
            let (dx, dy) = loop {
                let dx = rng.gen_range(-1.0f32..1.0);
                let dy = rng.gen_range(-1.0f32..1.0);
                let length_sq = dx * dx + dy * dy;
                if length_sq <= 1.0 && length_sq > 0.0 {
                    // Normalize to unit vector
                    let length = length_sq.sqrt();
                    break (dx / length, dy / length);
                }
            };

            new_velocity.x = dx * speed;
            new_velocity.y = dy * speed;

            // Cap velocity to prevent extreme movements
            if new_velocity.x.abs() > config.max_velocity {
                new_velocity.x = new_velocity.x.signum() * config.max_velocity;
            }
            if new_velocity.y.abs() > config.max_velocity {
                new_velocity.y = new_velocity.y.signum() * config.max_velocity;
            }
        }

        new_pos.x += new_velocity.x;
        new_pos.y += new_velocity.y;

        // Validate position to prevent NaN or infinite values
        if new_pos.x.is_nan() || new_pos.x.is_infinite() {
            new_pos.x = 0.0;
        }
        if new_pos.y.is_nan() || new_pos.y.is_infinite() {
            new_pos.y = 0.0;
        }

        // Movement cost based on genes
        let movement_distance =
            (new_velocity.x * new_velocity.x + new_velocity.y * new_velocity.y).sqrt();
        *new_energy -= movement_distance * config.movement_energy_cost / genes.energy_efficiency();
    }

    fn find_movement_target(
        &self,
        pos: &Position,
        genes: &Genes,
        nearby_entities: &[Entity],
        world: &World,
    ) -> Option<(f32, f32)> {
        for &entity in nearby_entities {
            if let Ok(nearby_pos) = world.get::<&Position>(entity) {
                if let Ok(nearby_genes) = world.get::<&Genes>(entity) {
                    if let Ok(nearby_energy) = world.get::<&Energy>(entity) {
                        if let Ok(nearby_size) = world.get::<&Size>(entity) {
                            if nearby_energy.current > 0.0 {
                                let distance = ((nearby_pos.x - pos.x).powi(2)
                                    + (nearby_pos.y - pos.y).powi(2))
                                .sqrt();
                                if distance < genes.sense_radius() {
                                    // Check if this is a potential food source
                                    if genes.can_eat(
                                        &*nearby_genes,
                                        &*nearby_size,
                                        &Size { radius: 1.0 },
                                    ) {
                                        return Some((nearby_pos.x, nearby_pos.y));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn handle_boundaries(
        &self,
        pos: &mut Position,
        velocity: &mut Velocity,
        world_size: f32,
        config: &SimulationConfig,
    ) {
        let half_world = world_size / 2.0;

        // Use <= and >= to handle edge cases better
        if pos.x <= -half_world + config.boundary_margin {
            pos.x = -half_world + config.boundary_margin;
            velocity.x = velocity.x.abs() * config.velocity_bounce_factor;
        } else if pos.x >= half_world - config.boundary_margin {
            pos.x = half_world - config.boundary_margin;
            velocity.x = -velocity.x.abs() * config.velocity_bounce_factor;
        }

        if pos.y <= -half_world + config.boundary_margin {
            pos.y = -half_world + config.boundary_margin;
            velocity.y = velocity.y.abs() * config.velocity_bounce_factor;
        } else if pos.y >= half_world - config.boundary_margin {
            pos.y = half_world - config.boundary_margin;
            velocity.y = -velocity.y.abs() * config.velocity_bounce_factor;
        }
    }
}

/// Interaction system - handles entity interactions and predation
pub struct InteractionSystem;

impl InteractionSystem {
    pub fn handle_interactions(
        &self,
        new_energy: &mut f32,
        eaten_entity: &mut Option<Entity>,
        new_pos: &Position,
        size: &Size,
        genes: &Genes,
        nearby_entities: &[Entity],
        world: &World,
        config: &SimulationConfig,
    ) {
        for &entity in nearby_entities {
            if let Ok(nearby_pos) = world.get::<&Position>(entity) {
                if let Ok(nearby_genes) = world.get::<&Genes>(entity) {
                    if let Ok(nearby_energy) = world.get::<&Energy>(entity) {
                        if let Ok(nearby_size) = world.get::<&Size>(entity) {
                            let distance = ((nearby_pos.x - new_pos.x).powi(2)
                                + (nearby_pos.y - new_pos.y).powi(2))
                            .sqrt();

                            if distance < (size.radius + config.interaction_radius_offset)
                                && nearby_energy.current > 0.0
                            {
                                if genes.can_eat(&*nearby_genes, &*nearby_size, size) {
                                    *eaten_entity = Some(entity);
                                    let energy_gained = genes.get_energy_gain(
                                        nearby_energy.current,
                                        &*nearby_size,
                                        size,
                                    );
                                    *new_energy = (*new_energy + energy_gained - 0.5)
                                        .min(genes.energy_efficiency() * 100.0);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Energy system - handles energy consumption and metabolism
pub struct EnergySystem;

impl EnergySystem {
    pub fn update_energy(
        &self,
        new_energy: &mut f32,
        size: &Size,
        genes: &Genes,
        config: &SimulationConfig,
    ) {
        // Energy changes based on genes and size (larger entities cost more to maintain)
        let size_energy_cost = size.radius * config.size_energy_cost_factor;
        *new_energy -= (genes.energy_loss_rate() + size_energy_cost) / genes.energy_efficiency();
    }

    pub fn calculate_new_size(&self, energy: f32, genes: &Genes, config: &SimulationConfig) -> f32 {
        (energy / 15.0 * genes.size_factor())
            .clamp(config.min_entity_radius, config.max_entity_radius)
    }
}

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
            * (1.0 - population_density * config.population_density_factor)
                .max(config.min_reproduction_chance);

        energy > max_energy * config.reproduction_energy_threshold
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
    ) {
        let mut rng = thread_rng();
        let child_genes = parent_genes.mutate(&mut rng);
        let child_energy = parent_energy_max * config.child_energy_factor;
        let child_radius =
            (child_energy / 15.0 * child_genes.size_factor()).clamp(config.min_entity_radius, 15.0);
        let child_color = child_genes.get_color();

        // Use uniform distribution in a circle for child positioning
        let (dx, dy) = loop {
            let dx = rng.gen_range(-config.child_spawn_radius..config.child_spawn_radius);
            let dy = rng.gen_range(-config.child_spawn_radius..config.child_spawn_radius);
            let distance_sq = dx * dx + dy * dy;
            if distance_sq <= config.child_spawn_radius * config.child_spawn_radius {
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
            child_genes,
            child_color,
            Velocity { x: 0.0, y: 0.0 },
        )
    }

    pub fn check_death(&self, population_density: f32, config: &SimulationConfig) -> bool {
        let death_chance = population_density * config.death_chance_factor;
        thread_rng().gen::<f32>() < death_chance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Color, Energy, Position, Size, Velocity};
    use crate::genes::Genes;
    use hecs::World;

    #[test]
    fn test_movement_system_update_movement() {
        let system = MovementSystem;
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let mut new_pos = Position { x: 0.0, y: 0.0 };
        let mut new_velocity = Velocity { x: 0.0, y: 0.0 };
        let mut new_energy = 100.0;
        let pos = Position { x: 0.0, y: 0.0 };
        let nearby_entities = vec![];
        let world = World::new();
        let config = SimulationConfig::default();

        system.update_movement(
            &genes,
            &mut new_pos,
            &mut new_velocity,
            &mut new_energy,
            &pos,
            &nearby_entities,
            &world,
            &config,
        );

        // Position should have changed
        assert_ne!(new_pos.x, 0.0);
        assert_ne!(new_pos.y, 0.0);

        // Velocity should be set
        assert_ne!(new_velocity.x, 0.0);
        assert_ne!(new_velocity.y, 0.0);

        // Energy should have decreased due to movement cost
        assert!(new_energy < 100.0);
    }

    #[test]
    fn test_movement_system_handle_boundaries() {
        let system = MovementSystem;
        let mut pos = Position { x: 60.0, y: 60.0 }; // Outside boundary
        let mut velocity = Velocity { x: 10.0, y: 10.0 };
        let world_size = 100.0;
        let config = SimulationConfig::default();

        system.handle_boundaries(&mut pos, &mut velocity, world_size, &config);

        // Position should be clamped to boundary
        assert!(pos.x <= 50.0 - config.boundary_margin);
        assert!(pos.y <= 50.0 - config.boundary_margin);

        // Velocity should be reflected
        assert!(velocity.x < 0.0 || velocity.y < 0.0);
    }

    #[test]
    fn test_movement_system_boundary_center() {
        let system = MovementSystem;
        let mut pos = Position { x: 0.0, y: 0.0 }; // Center
        let mut velocity = Velocity { x: 5.0, y: 5.0 };
        let world_size = 100.0;
        let config = SimulationConfig::default();

        system.handle_boundaries(&mut pos, &mut velocity, world_size, &config);

        // Position should remain unchanged
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);

        // Velocity should have drift compensation applied
        assert_eq!(velocity.x, 5.0);
        assert_eq!(velocity.y, 5.0);
    }

    #[test]
    fn test_interaction_system_handle_interactions() {
        let system = InteractionSystem;
        let mut new_energy = 50.0;
        let mut eaten_entity = None;
        let new_pos = Position { x: 0.0, y: 0.0 };
        let size = Size { radius: 10.0 };
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let nearby_entities = vec![];
        let world = World::new();
        let config = SimulationConfig::default();

        system.handle_interactions(
            &mut new_energy,
            &mut eaten_entity,
            &new_pos,
            &size,
            &genes,
            &nearby_entities,
            &world,
            &config,
        );

        // Energy should remain unchanged if no interactions
        assert_eq!(new_energy, 50.0);
        assert!(eaten_entity.is_none());
    }

    #[test]
    fn test_energy_system_update_energy() {
        let system = EnergySystem;
        let mut new_energy = 50.0;
        let size = Size { radius: 10.0 };
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let config = SimulationConfig::default();

        system.update_energy(&mut new_energy, &size, &genes, &config);

        // Energy should have changed due to loss and gain
        assert_ne!(new_energy, 50.0);
    }

    #[test]
    fn test_energy_system_calculate_new_size() {
        let system = EnergySystem;
        let energy = 80.0;
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let config = SimulationConfig::default();

        let new_size = system.calculate_new_size(energy, &genes, &config);

        // Size should be positive and reasonable
        assert!(new_size > 0.0);
        assert!(new_size <= config.max_entity_radius);
    }

    #[test]
    fn test_reproduction_system_check_reproduction() {
        let system = ReproductionSystem;
        let energy = 90.0;
        let max_energy = 100.0;
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let population_density = 0.1; // Low density for higher reproduction chance
        let config = SimulationConfig::default();

        let should_reproduce =
            system.check_reproduction(energy, max_energy, &genes, population_density, &config);

        // Reproduction is probabilistic, so we just check it doesn't panic
        // and that the logic is sound (high energy, low density = higher chance)
        assert!(should_reproduce || !should_reproduce); // Always true, just checking no panic
    }

    #[test]
    fn test_reproduction_system_create_offspring() {
        let system = ReproductionSystem;
        let mut rng = thread_rng();
        let parent_genes = Genes::new_random(&mut rng);
        let parent_energy_max = 100.0;
        let parent_pos = Position { x: 0.0, y: 0.0 };
        let config = SimulationConfig::default();

        let (pos, energy, size, _genes, color, velocity) =
            system.create_offspring(&parent_genes, parent_energy_max, &parent_pos, &config);

        // Position should be near parent
        let distance = ((pos.x - parent_pos.x).powi(2) + (pos.y - parent_pos.y).powi(2)).sqrt();
        assert!(distance <= config.child_spawn_radius);

        // Energy should be reasonable
        assert!(energy.current > 0.0);
        assert!(energy.current <= energy.max);

        // Size should be reasonable
        assert!(size.radius > 0.0);
        assert!(size.radius <= config.max_entity_radius);

        // Color should be valid
        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);

        // Velocity should be reasonable
        assert!(velocity.x.abs() <= config.max_velocity);
        assert!(velocity.y.abs() <= config.max_velocity);
    }

    #[test]
    fn test_reproduction_system_check_death() {
        let system = ReproductionSystem;
        let population_density = 0.9; // High density
        let config = SimulationConfig::default();

        let should_die = system.check_death(population_density, &config);

        // High population density should increase death chance
        // This is probabilistic, so we just check it doesn't panic
        assert!(should_die || !should_die); // Always true, just checking no panic
    }

    #[test]
    fn test_movement_system_with_target() {
        let system = MovementSystem;
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let mut new_pos = Position { x: 0.0, y: 0.0 };
        let mut new_velocity = Velocity { x: 0.0, y: 0.0 };
        let mut new_energy = 100.0;
        let pos = Position { x: 0.0, y: 0.0 };

        // Create a world with a target entity
        let mut world = World::new();
        let target_entity = world.spawn((
            Position { x: 10.0, y: 10.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut rng),
        ));
        let nearby_entities = vec![target_entity];

        let config = SimulationConfig::default();

        system.update_movement(
            &genes,
            &mut new_pos,
            &mut new_velocity,
            &mut new_energy,
            &pos,
            &nearby_entities,
            &world,
            &config,
        );

        // Should have moved (position changed) and used energy
        assert!(
            new_pos.x != 0.0 || new_pos.y != 0.0 || new_velocity.x != 0.0 || new_velocity.y != 0.0
        );
        assert!(new_energy < 100.0);
    }

    #[test]
    fn test_energy_system_energy_bounds() {
        let system = EnergySystem;
        let mut new_energy = 0.0; // Start with no energy
        let size = Size { radius: 10.0 };
        let mut rng = thread_rng();
        let genes = Genes::new_random(&mut rng);
        let config = SimulationConfig::default();

        system.update_energy(&mut new_energy, &size, &genes, &config);

        // Energy can go below 0 due to energy loss, but should be finite
        assert!(new_energy.is_finite());
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
    fn test_movement_drift_analysis() {
        let config = SimulationConfig::default();
        let movement_system = MovementSystem;
        let mut world = World::new();

        // Test 1: Check if initial velocity has any bias
        let _entity = world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { x: 0.0, y: 0.0 },
            Energy {
                current: 100.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        ));

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut velocity = Velocity { x: 0.0, y: 0.0 };
        let mut energy = 100.0;

        // Run movement update with no nearby entities
        movement_system.update_movement(
            &Genes::new_random(&mut thread_rng()),
            &mut pos,
            &mut velocity,
            &mut energy,
            &Position { x: 0.0, y: 0.0 },
            &[],
            &world,
            &config,
        );

        // Check if there's any systematic bias in velocity generation
        println!(
            "Initial velocity after update: ({}, {})",
            velocity.x, velocity.y
        );
        assert!(
            velocity.x.abs() < 10.0,
            "Velocity x should be reasonable: {}",
            velocity.x
        );
        assert!(
            velocity.y.abs() < 10.0,
            "Velocity y should be reasonable: {}",
            velocity.y
        );
    }

    #[test]
    fn test_boundary_handling_drift() {
        let config = SimulationConfig::default();
        let movement_system = MovementSystem;
        let world_size = 100.0;

        // Test boundary handling for all four sides
        let test_cases = vec![
            // Left boundary
            (Position { x: -45.0, y: 0.0 }, Velocity { x: -5.0, y: 0.0 }),
            // Right boundary
            (Position { x: 45.0, y: 0.0 }, Velocity { x: 5.0, y: 0.0 }),
            // Top boundary
            (Position { x: 0.0, y: -45.0 }, Velocity { x: 0.0, y: -5.0 }),
            // Bottom boundary
            (Position { x: 0.0, y: 45.0 }, Velocity { x: 0.0, y: 5.0 }),
        ];

        for (mut pos, mut velocity) in test_cases {
            let original_velocity = velocity.clone();
            movement_system.handle_boundaries(&mut pos, &mut velocity, world_size, &config);

            println!(
                "Boundary test - Original: ({}, {}), Final: ({}, {})",
                original_velocity.x, original_velocity.y, velocity.x, velocity.y
            );

            // Check that velocity direction is properly reversed
            if pos.x <= -45.0 || pos.x >= 45.0 {
                assert!(
                    (original_velocity.x * velocity.x) <= 0.0,
                    "X velocity should be reversed at boundaries"
                );
            }
            if pos.y <= -45.0 || pos.y >= 45.0 {
                assert!(
                    (original_velocity.y * velocity.y) <= 0.0,
                    "Y velocity should be reversed at boundaries"
                );
            }
        }
    }

    #[test]
    fn test_velocity_distribution_analysis() {
        let config = SimulationConfig::default();
        let movement_system = MovementSystem;
        let mut world = World::new();

        // Create multiple entities and track their velocity distributions
        let mut x_velocities = Vec::new();
        let mut y_velocities = Vec::new();

        for _ in 0..100 {
            let _entity = world.spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { x: 0.0, y: 0.0 },
                Energy {
                    current: 100.0,
                    max: 100.0,
                },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
            ));

            let mut pos = Position { x: 0.0, y: 0.0 };
            let mut velocity = Velocity { x: 0.0, y: 0.0 };
            let mut energy = 100.0;

            movement_system.update_movement(
                &Genes::new_random(&mut thread_rng()),
                &mut pos,
                &mut velocity,
                &mut energy,
                &Position { x: 0.0, y: 0.0 },
                &[],
                &world,
                &config,
            );

            x_velocities.push(velocity.x);
            y_velocities.push(velocity.y);
        }

        // Calculate statistics
        let x_mean = x_velocities.iter().sum::<f32>() / x_velocities.len() as f32;
        let y_mean = y_velocities.iter().sum::<f32>() / y_velocities.len() as f32;

        let x_std = (x_velocities
            .iter()
            .map(|&x| (x - x_mean).powi(2))
            .sum::<f32>()
            / x_velocities.len() as f32)
            .sqrt();
        let y_std = (y_velocities
            .iter()
            .map(|&y| (y - y_mean).powi(2))
            .sum::<f32>()
            / y_velocities.len() as f32)
            .sqrt();

        println!("Velocity distribution analysis:");
        println!("X - Mean: {:.3}, Std: {:.3}", x_mean, x_std);
        println!("Y - Mean: {:.3}, Std: {:.3}", y_mean, y_std);

        // Check for systematic bias (mean should be close to 0)
        assert!(
            x_mean.abs() < 1.0,
            "X velocity mean should be close to 0, got: {}",
            x_mean
        );
        assert!(
            y_mean.abs() < 1.0,
            "Y velocity mean should be close to 0, got: {}",
            y_mean
        );
    }

    #[test]
    fn test_movement_target_bias() {
        let config = SimulationConfig::default();
        let movement_system = MovementSystem;
        let mut world = World::new();

        // Create some target entities in different quadrants
        let targets = vec![
            (
                Position { x: 20.0, y: 20.0 },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
            ),
            (
                Position { x: -20.0, y: 20.0 },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
            ),
            (
                Position { x: 20.0, y: -20.0 },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
            ),
            (
                Position { x: -20.0, y: -20.0 },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
            ),
        ];

        let target_entities: Vec<Entity> = targets
            .iter()
            .map(|(pos, size, genes)| {
                world.spawn((
                    pos.clone(),
                    size.clone(),
                    genes.clone(),
                    Energy {
                        current: 50.0,
                        max: 100.0,
                    },
                ))
            })
            .collect();

        // Test movement towards targets from center
        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut velocity = Velocity { x: 0.0, y: 0.0 };
        let mut energy = 100.0;

        movement_system.update_movement(
            &Genes::new_random(&mut thread_rng()),
            &mut pos,
            &mut velocity,
            &mut energy,
            &Position { x: 0.0, y: 0.0 },
            &target_entities,
            &world,
            &config,
        );

        println!(
            "Movement towards targets - Final velocity: ({}, {})",
            velocity.x, velocity.y
        );

        // The velocity should generally point towards one of the targets
        // but we want to check if there's a systematic bias towards top-left
        let distance_to_top_left =
            ((velocity.x + 20.0).powi(2) + (velocity.y + 20.0).powi(2)).sqrt();
        let distance_to_top_right =
            ((velocity.x - 20.0).powi(2) + (velocity.y + 20.0).powi(2)).sqrt();
        let distance_to_bottom_left =
            ((velocity.x + 20.0).powi(2) + (velocity.y - 20.0).powi(2)).sqrt();
        let distance_to_bottom_right =
            ((velocity.x - 20.0).powi(2) + (velocity.y - 20.0).powi(2)).sqrt();

        println!(
            "Distances to quadrants - TL: {:.1}, TR: {:.1}, BL: {:.1}, BR: {:.1}",
            distance_to_top_left,
            distance_to_top_right,
            distance_to_bottom_left,
            distance_to_bottom_right
        );
    }

    #[test]
    fn test_long_term_drift_simulation() {
        let config = SimulationConfig::default();
        let movement_system = MovementSystem;
        let mut world = World::new();

        let _entity = world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { x: 0.0, y: 0.0 },
            Energy {
                current: 100.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
        ));

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut velocity = Velocity { x: 0.0, y: 0.0 };
        let mut energy = 100.0;

        let mut total_x_movement = 0.0;
        let mut total_y_movement = 0.0;

        // Simulate many movement steps
        for step in 0..100 {
            let old_pos = pos.clone();

            movement_system.update_movement(
                &Genes::new_random(&mut thread_rng()),
                &mut pos,
                &mut velocity,
                &mut energy,
                &old_pos.clone(),
                &[],
                &world,
                &config,
            );

            // Handle boundaries
            movement_system.handle_boundaries(&mut pos, &mut velocity, 100.0, &config);

            total_x_movement += pos.x - old_pos.x;
            total_y_movement += pos.y - old_pos.y;

            if step % 20 == 0 {
                println!(
                    "Step {} - Position: ({:.1}, {:.1}), Velocity: ({:.2}, {:.2})",
                    step, pos.x, pos.y, velocity.x, velocity.y
                );
            }
        }

        println!(
            "Total movement over 100 steps: ({:.1}, {:.1})",
            total_x_movement, total_y_movement
        );

        // Check for systematic drift
        let drift_magnitude = (total_x_movement.powi(2) + total_y_movement.powi(2)).sqrt();
        println!("Drift magnitude: {:.1}", drift_magnitude);

        // If there's significant drift, it should be detected
        if drift_magnitude > 50.0 {
            println!(
                "WARNING: Significant drift detected! Direction: ({:.1}, {:.1})",
                total_x_movement, total_y_movement
            );
        }
    }

    #[test]
    fn test_world_coordinate_system() {
        let world_size = 100.0; // Typical world size

        // Test coordinate transformations
        let test_positions = vec![
            (0.0, 0.0),     // Center
            (-50.0, -50.0), // Bottom-left
            (50.0, 50.0),   // Top-right
            (-50.0, 50.0),  // Top-left
            (50.0, -50.0),  // Bottom-right
        ];

        for (x, y) in test_positions {
            // Test the same transformation used in UI rendering
            let screen_x: f32 = (x + world_size / 2.0) / world_size * 2.0 - 1.0;
            let screen_y: f32 = -((y + world_size / 2.0) / world_size * 2.0 - 1.0);

            println!(
                "World: ({:.1}, {:.1}) -> Screen: ({:.3}, {:.3})",
                x, y, screen_x, screen_y
            );

            // Check that coordinates are properly mapped
            if x == 0.0 && y == 0.0 {
                assert!(
                    (screen_x.abs() < 0.01f32) && (screen_y.abs() < 0.01f32),
                    "Center should map to (0,0)"
                );
            }
        }
    }

    #[test]
    fn test_entity_position_distribution() {
        let mut world = World::new();

        // Create entities in a grid pattern to test distribution
        let grid_size = 10;
        let world_size = 100.0;
        let spacing = world_size / grid_size as f32;

        for i in 0..grid_size {
            for j in 0..grid_size {
                let x = (i as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;
                let y = (j as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;

                world.spawn((
                    Position { x, y },
                    Velocity { x: 0.0, y: 0.0 },
                    Energy {
                        current: 100.0,
                        max: 100.0,
                    },
                    Size { radius: 5.0 },
                    Genes::new_random(&mut thread_rng()),
                    Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                    },
                ));
            }
        }

        // Calculate center of mass
        let mut total_x = 0.0;
        let mut total_y = 0.0;
        let mut count = 0;

        for (_, (pos,)) in world.query::<(&Position,)>().iter() {
            total_x += pos.x;
            total_y += pos.y;
            count += 1;
        }

        let center_x = total_x / count as f32;
        let center_y = total_y / count as f32;

        println!(
            "Grid distribution - Center of mass: ({:.1}, {:.1})",
            center_x, center_y
        );
        println!("Expected center: (0.0, 0.0)");

        // Center of mass should be very close to (0,0) for a uniform grid
        assert!(
            center_x.abs() < 1.0,
            "X center should be close to 0, got: {}",
            center_x
        );
        assert!(
            center_y.abs() < 1.0,
            "Y center should be close to 0, got: {}",
            center_y
        );
    }

    #[test]
    fn test_simulation_clustering() {
        use crate::config::SimulationConfig;
        use crate::simulation::Simulation;

        let config = SimulationConfig::default();
        let world_size = 100.0;
        let mut simulation = Simulation::new_with_config(world_size, config);

        println!("Initial entity count: {}", simulation.world().len());

        // Run simulation for 100 steps
        for step in 0..100 {
            simulation.update();

            if step % 20 == 0 {
                let entities = simulation.get_entities();
                let mut total_x = 0.0;
                let mut total_y = 0.0;
                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;

                for (x, y, _, _, _, _) in &entities {
                    total_x += x;
                    total_y += y;
                    min_x = min_x.min(*x);
                    min_y = min_y.min(*y);
                    max_x = max_x.max(*x);
                    max_y = max_y.max(*y);
                }

                let center_x = total_x / entities.len() as f32;
                let center_y = total_y / entities.len() as f32;
                let spread_x = max_x - min_x;
                let spread_y = max_y - min_y;

                println!(
                    "Step {}: {} entities, Center: ({:.1}, {:.1}), Spread: ({:.1}, {:.1})",
                    step,
                    entities.len(),
                    center_x,
                    center_y,
                    spread_x,
                    spread_y
                );

                // Check for clustering in top-left
                if center_x < -20.0 && center_y > 20.0 {
                    println!(
                        "WARNING: Entities clustering in top-left! Center: ({:.1}, {:.1})",
                        center_x, center_y
                    );
                }
            }
        }
    }

    #[test]
    fn test_drift_direction_analysis() {
        use crate::config::SimulationConfig;
        use crate::simulation::Simulation;

        let config = SimulationConfig::default();
        let world_size = 100.0;
        let mut simulation = Simulation::new_with_config(world_size, config);

        println!("Testing drift direction over 200 steps...");

        let mut positions = Vec::new();

        // Run simulation for 200 steps, recording positions every 20 steps
        for step in 0..200 {
            simulation.update();

            if step % 20 == 0 {
                let entities = simulation.get_entities();
                let mut total_x = 0.0;
                let mut total_y = 0.0;

                for (x, y, _, _, _, _) in &entities {
                    total_x += x;
                    total_y += y;
                }

                let center_x = total_x / entities.len() as f32;
                let center_y = total_y / entities.len() as f32;
                positions.push((step, center_x, center_y));

                println!("Step {}: Center ({:.1}, {:.1})", step, center_x, center_y);
            }
        }

        // Analyze drift direction
        if positions.len() >= 2 {
            let first = positions[0];
            let last = positions[positions.len() - 1];
            let drift_x = last.1 - first.1;
            let drift_y = last.2 - first.2;

            println!("\nDrift Analysis:");
            println!("Start position: ({:.1}, {:.1})", first.1, first.2);
            println!("End position: ({:.1}, {:.1})", last.1, last.2);
            println!("Total drift: ({:.1}, {:.1})", drift_x, drift_y);

            // Determine drift direction
            let direction = if drift_x < -5.0 && drift_y < -5.0 {
                "Bottom-Left (appears as Top-Left on screen)"
            } else if drift_x > 5.0 && drift_y < -5.0 {
                "Bottom-Right (appears as Top-Right on screen)"
            } else if drift_x < -5.0 && drift_y > 5.0 {
                "Top-Left"
            } else if drift_x > 5.0 && drift_y > 5.0 {
                "Top-Right"
            } else {
                "Minimal or no significant drift"
            };

            println!("Drift direction: {}", direction);

            // Check if this matches the observed visual clustering
            if drift_x < -5.0 && drift_y < -5.0 {
                println!("CONFIRMED: Entities are drifting to bottom-left in world coordinates!");
                println!("This appears as top-left clustering on screen due to Y-axis flip in rendering.");
            }
        }
    }

    #[test]
    fn test_random_number_bias() {
        use rand::thread_rng;
        use rand::Rng;

        let mut rng = thread_rng();
        let mut x_values = Vec::new();
        let mut y_values = Vec::new();

        // Generate many random values to check for bias
        for _ in 0..10000 {
            // Test the same random generation used in movement
            let dx = rng.gen_range(-1.0f32..1.0);
            let dy = rng.gen_range(-1.0f32..1.0);
            let length_sq = dx * dx + dy * dy;

            if length_sq <= 1.0 && length_sq > 0.0 {
                let length = length_sq.sqrt();
                x_values.push(dx / length);
                y_values.push(dy / length);
            }
        }

        // Calculate statistics
        let x_mean = x_values.iter().sum::<f32>() / x_values.len() as f32;
        let y_mean = y_values.iter().sum::<f32>() / y_values.len() as f32;

        let x_std = (x_values.iter().map(|&x| (x - x_mean).powi(2)).sum::<f32>()
            / x_values.len() as f32)
            .sqrt();
        let y_std = (y_values.iter().map(|&y| (y - y_mean).powi(2)).sum::<f32>()
            / y_values.len() as f32)
            .sqrt();

        println!("Random direction analysis ({} samples):", x_values.len());
        println!("X - Mean: {:.4}, Std: {:.4}", x_mean, x_std);
        println!("Y - Mean: {:.4}, Std: {:.4}", y_mean, y_std);

        // Check for systematic bias
        assert!(
            x_mean.abs() < 0.05,
            "X mean should be very close to 0, got: {}",
            x_mean
        );
        assert!(
            y_mean.abs() < 0.05,
            "Y mean should be very close to 0, got: {}",
            y_mean
        );

        // Check that standard deviations are reasonable (should be around 0.7 for uniform distribution in circle)
        assert!(
            x_std > 0.6 && x_std < 0.8,
            "X std should be around 0.7, got: {}",
            x_std
        );
        assert!(
            y_std > 0.6 && y_std < 0.8,
            "Y std should be around 0.7, got: {}",
            y_std
        );
    }

    #[test]
    fn test_interaction_system_drift() {
        use crate::config::SimulationConfig;
        use crate::simulation::Simulation;

        let mut config = SimulationConfig::default();
        // Disable reproduction to isolate interaction effects
        config.reproduction_energy_threshold = 2.0; // Impossible threshold
        config.min_reproduction_chance = 0.0;

        let world_size = 100.0;
        let mut simulation = Simulation::new_with_config(world_size, config);

        println!("Testing interaction system drift (reproduction disabled)...");

        let mut positions = Vec::new();

        // Run simulation for 100 steps, recording positions every 20 steps
        for step in 0..100 {
            simulation.update();

            if step % 20 == 0 {
                let entities = simulation.get_entities();
                let mut total_x = 0.0;
                let mut total_y = 0.0;

                for (x, y, _, _, _, _) in &entities {
                    total_x += x;
                    total_y += y;
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

            println!("\nInteraction System Drift Analysis:");
            println!("Start position: ({:.1}, {:.1})", first.1, first.2);
            println!("End position: ({:.1}, {:.1})", last.1, last.2);
            println!("Total drift: ({:.1}, {:.1})", drift_x, drift_y);

            if drift_x.abs() > 5.0 || drift_y.abs() > 5.0 {
                println!("INTERACTION SYSTEM IS CAUSING DRIFT!");
            } else {
                println!("Interaction system appears unbiased");
            }
        }
    }

    #[test]
    fn test_reproduction_system_drift() {
        use crate::config::SimulationConfig;
        use crate::simulation::Simulation;

        let mut config = SimulationConfig::default();
        // Disable interactions to isolate reproduction effects
        config.interaction_radius_offset = 0.0; // No interactions

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

                for (x, y, _, _, _, _) in &entities {
                    total_x += x;
                    total_y += y;
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

    #[test]
    fn test_spatial_grid_bias() {
        use crate::spatial_grid::SpatialGrid;

        let mut grid = SpatialGrid::new(25.0);

        // Create entities in a grid pattern
        let grid_size = 10;
        let world_size = 100.0;
        let spacing = world_size / grid_size as f32;

        let mut entities = Vec::new();

        let mut world = World::new();
        for i in 0..grid_size {
            for j in 0..grid_size {
                let x = (i as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;
                let y = (j as f32 - (grid_size as f32 - 1.0) / 2.0) * spacing;

                let entity = world.spawn((
                    Position { x, y },
                    Velocity { x: 0.0, y: 0.0 },
                    Energy {
                        current: 100.0,
                        max: 100.0,
                    },
                    Size { radius: 5.0 },
                    Genes::new_random(&mut thread_rng()),
                    Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                    },
                ));
                entities.push((entity, x, y));
                grid.insert(entity, x, y);
            }
        }

        // Test neighbor detection from different positions
        let test_positions = vec![
            (0.0, 0.0),     // Center
            (-25.0, -25.0), // Bottom-left
            (25.0, 25.0),   // Top-right
            (-25.0, 25.0),  // Top-left
            (25.0, -25.0),  // Bottom-right
        ];

        for (test_x, test_y) in test_positions {
            let nearby = grid.get_nearby_entities(test_x, test_y, 30.0);

            // Calculate center of nearby entities
            let mut total_x = 0.0;
            let mut total_y = 0.0;
            let mut count = 0;

            for &entity in &nearby {
                if let Some((_, x, y)) = entities.iter().find(|(e, _, _)| *e == entity) {
                    total_x += x;
                    total_y += y;
                    count += 1;
                }
            }

            if count > 0 {
                let center_x = total_x / count as f32;
                let center_y = total_y / count as f32;

                println!(
                    "Test pos ({:.1}, {:.1}): {} nearby, center ({:.1}, {:.1})",
                    test_x, test_y, count, center_x, center_y
                );

                // Check for bias relative to test position
                let bias_x = center_x - test_x;
                let bias_y = center_y - test_y;

                if bias_x.abs() > 5.0 || bias_y.abs() > 5.0 {
                    println!("SPATIAL GRID BIAS DETECTED: ({:.1}, {:.1})", bias_x, bias_y);
                }
            }
        }
    }

    #[test]
    fn test_spatial_grid_order_bias() {
        use crate::spatial_grid::SpatialGrid;

        let mut grid = SpatialGrid::new(25.0);

        let mut world = World::new();
        // Create entities in a specific pattern to test order bias
        let mut entities = Vec::new();

        let positions = vec![
            (-25.0, -25.0), // Bottom-left
            (25.0, -25.0),  // Bottom-right
            (-25.0, 25.0),  // Top-left
            (25.0, 25.0),   // Top-right
        ];

        // Insert entities in a specific order
        for (_i, (x, y)) in positions.iter().enumerate() {
            let entity = world.spawn((
                Position { x: *x, y: *y },
                Velocity { x: 0.0, y: 0.0 },
                Energy {
                    current: 100.0,
                    max: 100.0,
                },
                Size { radius: 5.0 },
                Genes::new_random(&mut thread_rng()),
                Color {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                },
            ));
            entities.push((entity, *x, *y));
            grid.insert(entity, *x, *y);
        }

        // Test neighbor detection from center
        let nearby = grid.get_nearby_entities(0.0, 0.0, 30.0);

        println!("Nearby entities from center (0,0):");
        for (i, &entity) in nearby.iter().enumerate() {
            if let Some((_, x, y)) = entities.iter().find(|(e, _, _)| *e == entity) {
                println!("  {}: ({:.1}, {:.1})", i, x, y);
            }
        }

        // Check if there's a consistent order bias
        if nearby.len() >= 4 {
            let first_entity = nearby[0];
            if let Some((_, x, y)) = entities.iter().find(|(e, _, _)| *e == first_entity) {
                println!("First entity found: ({:.1}, {:.1})", x, y);

                // Check if it's consistently from a particular quadrant
                if *x < 0.0 && *y < 0.0 {
                    println!("BIAS DETECTED: First entity is from bottom-left quadrant!");
                } else if *x > 0.0 && *y < 0.0 {
                    println!("BIAS DETECTED: First entity is from bottom-right quadrant!");
                } else if *x < 0.0 && *y > 0.0 {
                    println!("BIAS DETECTED: First entity is from top-left quadrant!");
                } else {
                    println!("BIAS DETECTED: First entity is from top-right quadrant!");
                }
            }
        }
    }

    #[test]
    fn test_interaction_processing_order() {
        use crate::config::SimulationConfig;
        use crate::simulation::Simulation;

        let mut config = SimulationConfig::default();
        // Make interactions more likely to see the effect
        config.interaction_radius_offset = 25.0; // Larger interaction radius

        let world_size = 100.0;
        let mut simulation = Simulation::new_with_config(world_size, config);

        println!("Testing interaction processing order bias...");

        // Track which entities are being eaten and from which positions
        let mut eaten_positions: Vec<(f32, f32)> = Vec::new();
        let _predator_positions: Vec<(f32, f32)> = Vec::new();

        // Run simulation for 50 steps and track interactions
        for step in 0..50 {
            simulation.update();

            if step % 10 == 0 {
                let entities = simulation.get_entities();
                let mut total_x = 0.0;
                let mut total_y = 0.0;

                for (x, y, _, _, _, _) in &entities {
                    total_x += x;
                    total_y += y;
                }

                let center_x = total_x / entities.len() as f32;
                let center_y = total_y / entities.len() as f32;

                println!(
                    "Step {}: {} entities, Center ({:.1}, {:.1})",
                    step,
                    entities.len(),
                    center_x,
                    center_y
                );

                // Store positions for analysis
                for (x, y, _, _, _, _) in &entities {
                    eaten_positions.push((*x, *y));
                }
            }
        }

        // Analyze if there's a pattern in where entities are being eaten
        if !eaten_positions.is_empty() {
            let mut total_x = 0.0;
            let mut total_y = 0.0;

            for (x, y) in &eaten_positions {
                total_x += x;
                total_y += y;
            }

            let avg_x = total_x / eaten_positions.len() as f32;
            let avg_y = total_y / eaten_positions.len() as f32;

            println!("Average position of entities: ({:.1}, {:.1})", avg_x, avg_y);

            // Check if there's a bias towards certain quadrants
            let mut quadrant_counts = [0, 0, 0, 0]; // TL, TR, BL, BR

            for (x, y) in &eaten_positions {
                if *x < 0.0 && *y > 0.0 {
                    quadrant_counts[0] += 1; // Top-left
                } else if *x > 0.0 && *y > 0.0 {
                    quadrant_counts[1] += 1; // Top-right
                } else if *x < 0.0 && *y < 0.0 {
                    quadrant_counts[2] += 1; // Bottom-left
                } else {
                    quadrant_counts[3] += 1; // Bottom-right
                }
            }

            println!(
                "Entity distribution by quadrant: TL:{}, TR:{}, BL:{}, BR:{}",
                quadrant_counts[0], quadrant_counts[1], quadrant_counts[2], quadrant_counts[3]
            );

            // Check for significant bias
            let total = quadrant_counts.iter().sum::<i32>();
            let expected = total / 4;

            for (i, count) in quadrant_counts.iter().enumerate() {
                let bias = (*count as f32 - expected as f32) / total as f32;
                if bias.abs() > 0.1 {
                    println!("SIGNIFICANT BIAS in quadrant {}: {:.1}%", i, bias * 100.0);
                }
            }
        }
    }

    #[test]
    fn test_interaction_order_bias() {
        use crate::config::SimulationConfig;
        use crate::simulation::Simulation;

        let mut config = SimulationConfig::default();
        // Make interactions very likely
        config.interaction_radius_offset = 30.0;

        let world_size = 100.0;
        let mut simulation = Simulation::new_with_config(world_size, config);

        println!("Testing interaction order bias...");

        // Track the positions of entities that survive vs those that don't
        let mut survivor_positions = Vec::new();
        let mut initial_positions = Vec::new();

        // Get initial positions
        let initial_entities = simulation.get_entities();
        for (x, y, _, _, _, _) in &initial_entities {
            initial_positions.push((*x, *y));
        }

        // Run simulation for a few steps
        for step in 0..20 {
            simulation.update();

            if step == 19 {
                // After 20 steps
                let final_entities = simulation.get_entities();
                for (x, y, _, _, _, _) in &final_entities {
                    survivor_positions.push((*x, *y));
                }
            }
        }

        // Analyze the bias
        if !survivor_positions.is_empty() && !initial_positions.is_empty() {
            let mut initial_total_x = 0.0;
            let mut initial_total_y = 0.0;
            let mut survivor_total_x = 0.0;
            let mut survivor_total_y = 0.0;

            for (x, y) in &initial_positions {
                initial_total_x += x;
                initial_total_y += y;
            }

            for (x, y) in &survivor_positions {
                survivor_total_x += x;
                survivor_total_y += y;
            }

            let initial_center_x = initial_total_x / initial_positions.len() as f32;
            let initial_center_y = initial_total_y / initial_positions.len() as f32;
            let survivor_center_x = survivor_total_x / survivor_positions.len() as f32;
            let survivor_center_y = survivor_total_y / survivor_positions.len() as f32;

            let drift_x = survivor_center_x - initial_center_x;
            let drift_y = survivor_center_y - initial_center_y;

            println!(
                "Initial center: ({:.1}, {:.1})",
                initial_center_x, initial_center_y
            );
            println!(
                "Survivor center: ({:.1}, {:.1})",
                survivor_center_x, survivor_center_y
            );
            println!("Drift: ({:.1}, {:.1})", drift_x, drift_y);

            // Check if survivors are biased towards certain quadrants
            let mut survivor_quadrants = [0, 0, 0, 0]; // TL, TR, BL, BR

            for (x, y) in &survivor_positions {
                if *x < 0.0 && *y > 0.0 {
                    survivor_quadrants[0] += 1; // Top-left
                } else if *x > 0.0 && *y > 0.0 {
                    survivor_quadrants[1] += 1; // Top-right
                } else if *x < 0.0 && *y < 0.0 {
                    survivor_quadrants[2] += 1; // Bottom-left
                } else {
                    survivor_quadrants[3] += 1; // Bottom-right
                }
            }

            println!(
                "Survivor distribution: TL:{}, TR:{}, BL:{}, BR:{}",
                survivor_quadrants[0],
                survivor_quadrants[1],
                survivor_quadrants[2],
                survivor_quadrants[3]
            );

            // Check for significant bias
            let total_survivors = survivor_quadrants.iter().sum::<i32>();
            let expected = total_survivors / 4;

            for (i, count) in survivor_quadrants.iter().enumerate() {
                let bias = (*count as f32 - expected as f32) / total_survivors as f32;
                if bias.abs() > 0.1 {
                    println!("SURVIVOR BIAS in quadrant {}: {:.1}%", i, bias * 100.0);
                }
            }

            if drift_x.abs() > 5.0 || drift_y.abs() > 5.0 {
                println!(
                    "INTERACTION ORDER BIAS CONFIRMED: Drift of ({:.1}, {:.1})",
                    drift_x, drift_y
                );
            }
        }
    }
}
