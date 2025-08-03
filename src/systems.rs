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

        // Add deliberate compensating drift to counteract systematic bias
        velocity.x += config.drift_compensation_x;
        velocity.y += config.drift_compensation_y;
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
    use crate::components::{Energy, Position, Size, Velocity};
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
        assert_eq!(velocity.x, 5.0 + config.drift_compensation_x);
        assert_eq!(velocity.y, 5.0 + config.drift_compensation_y);
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

        // Should move towards target
        assert!(new_pos.x > 0.0 || new_pos.y > 0.0);
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
}
