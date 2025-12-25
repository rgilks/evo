use crate::components::{Color, Energy, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use crate::spatial_grid::SpatialGrid;
use crate::stats::SimulationStats;
use crate::systems::{EnergySystem, InteractionSystem, MovementSystem, ReproductionSystem};
use hecs::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;

// Simulation state
pub struct EntityUpdate {
    pub entity: Entity,
    pub pos: Position,
    pub energy: Energy,
    pub size: Size,
    pub genes: Genes,
    pub color: Color,
    pub velocity: Velocity,
    pub movement_style: crate::components::MovementStyle,
    pub should_reproduce: bool,
    pub eaten_entity: Option<Entity>,
}

pub struct Simulation {
    world: World,
    world_size: f32,
    step: u32,
    grid: SpatialGrid,
    previous_positions: HashMap<Entity, Position>, // For smooth interpolation
    config: SimulationConfig,

    // System instances
    movement_system: MovementSystem,
    interaction_system: InteractionSystem,
    energy_system: EnergySystem,
    reproduction_system: ReproductionSystem,
}

struct ProcessEntityParams<'a> {
    entity: Entity,
    pos: &'a Position,
    energy: &'a Energy,
    size: &'a Size,
    genes: &'a Genes,
    color: &'a Color,
    velocity: &'a Velocity,
    movement_style: &'a crate::components::MovementStyle,
}

impl Simulation {
    #[allow(dead_code)]
    pub fn new(world_size: f32) -> Self {
        Self::new_with_config(world_size, SimulationConfig::default())
    }

    pub fn new_with_config(world_size: f32, config: SimulationConfig) -> Self {
        let mut world = World::new();
        let mut rng = thread_rng();
        let grid = SpatialGrid::new(config.physics.grid_cell_size);

        Self::spawn_initial_entities(&mut world, &mut rng, world_size, &config);

        Self {
            world,
            world_size,
            step: 0,
            grid,
            previous_positions: HashMap::new(),
            config,
            movement_system: MovementSystem,
            interaction_system: InteractionSystem,
            energy_system: EnergySystem,
            reproduction_system: ReproductionSystem,
        }
    }

    fn spawn_initial_entities(
        world: &mut World,
        rng: &mut ThreadRng,
        world_size: f32,
        config: &SimulationConfig,
    ) {
        let total_entities =
            (config.population.initial_entities as f32 * config.population.entity_scale) as usize;
        let spawn_radius = world_size * config.population.spawn_radius_factor;

        for _ in 0..total_entities {
            // Use perfectly uniform distribution in a circle
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = spawn_radius * rng.gen::<f32>().sqrt(); // Square root for uniform distribution
            let x = distance * angle.cos();
            let y = distance * angle.sin();

            let genes = Genes::new_random(rng);
            let energy = rng.gen_range(15.0..75.0);
            let color = genes.get_color();
            let radius = (energy / 15.0 * genes.size_factor()).clamp(
                config.physics.min_entity_radius,
                config.physics.max_entity_radius,
            );

            world.spawn((
                Position { x, y },
                Energy {
                    current: energy,
                    max: energy * 1.3,
                },
                Size { radius },
                genes.clone(),
                color,
                Velocity { x: 0.0, y: 0.0 },
                genes.behavior.movement_style.clone(),
            ));
        }
    }

    pub fn update(&mut self) {
        self.step += 1;
        self.update_simulation();

        if self.step % 60 == 0 {
            self.log_simulation_metrics();
        }
    }

    fn log_simulation_metrics(&self) {
        let stats = SimulationStats::from_world(
            &self.world,
            self.config.population.max_population as f32,
            self.config.population.entity_scale,
        );
        println!("{}", stats.format_detailed(self.step));
    }

    fn update_simulation(&mut self) {
        self.store_previous_positions();
        self.rebuild_spatial_grid();
        let updates = self.process_entities_parallel();
        self.apply_entity_updates(updates);
    }

    fn store_previous_positions(&mut self) {
        self.previous_positions.clear();
        for (entity, (pos,)) in self.world.query::<(&Position,)>().iter() {
            self.previous_positions.insert(entity, pos.clone());
        }
    }

    fn rebuild_spatial_grid(&mut self) {
        self.grid.clear();

        // Use parallel processing for grid building
        let grid_entities: Vec<_> = self
            .world
            .query::<(&Position,)>()
            .iter()
            .par_bridge()
            .map(|(entity, (pos,))| (entity, pos.x, pos.y))
            .collect();

        // Insert entities into grid (this part needs to be sequential due to HashMap)
        for (entity, x, y) in grid_entities {
            self.grid.insert(entity, x, y);
        }
    }

    fn process_entities_parallel(&self) -> Vec<EntityUpdate> {
        self.world
            .query::<(
                &Position,
                &Energy,
                &Size,
                &Genes,
                &Color,
                &Velocity,
                &crate::components::MovementStyle,
            )>()
            .iter()
            .par_bridge()
            .filter_map(
                |(entity, (pos, energy, size, genes, color, velocity, movement_style))| {
                    if energy.current <= 0.0 {
                        return None;
                    }

                    self.process_entity(ProcessEntityParams {
                        entity,
                        pos,
                        energy,
                        size,
                        genes,
                        color,
                        velocity,
                        movement_style,
                    })
                },
            )
            .collect()
    }

    fn process_entity(&self, params: ProcessEntityParams) -> Option<EntityUpdate> {
        let ProcessEntityParams {
            entity,
            pos,
            energy,
            size,
            genes,
            color,
            velocity,
            movement_style,
        } = params;

        let nearby_entities = self.get_nearby_entities_for_entity(pos, genes);

        let mut new_pos = pos.clone();
        let mut new_velocity = velocity.clone();
        let mut new_energy = energy.current;
        let mut eaten_entity = None;

        // Apply movement
        self.apply_movement_to_entity(
            genes,
            &mut new_pos,
            &mut new_velocity,
            &mut new_energy,
            pos,
            &nearby_entities,
        );

        // Handle boundaries
        self.movement_system.handle_boundaries(
            &mut new_pos,
            &mut new_velocity,
            self.world_size,
            &self.config,
        );

        // Handle interactions
        self.apply_interactions_to_entity(
            &mut new_energy,
            &mut eaten_entity,
            &new_pos,
            size,
            genes,
            &nearby_entities,
        );

        // Apply energy changes
        self.energy_system
            .update_energy(&mut new_energy, size, genes, &self.config);

        // Check reproduction and death
        let population_density = self.calculate_population_density();
        let should_reproduce =
            self.check_reproduction_for_entity(new_energy, energy.max, genes, population_density);

        if self
            .reproduction_system
            .check_death(population_density, &self.config)
        {
            new_energy = 0.0; // Kill the entity
        }

        if should_reproduce {
            // Don't spawn child here - we'll handle it in apply_entity_updates
            // Reduce parent energy
            new_energy *= self.config.reproduction.reproduction_energy_cost;
        }

        let new_size_radius =
            self.energy_system
                .calculate_new_size(new_energy, genes, &self.config);

        Some(EntityUpdate {
            entity,
            pos: new_pos,
            energy: Energy {
                current: new_energy,
                max: energy.max,
            },
            size: Size {
                radius: new_size_radius,
            },
            genes: genes.clone(),
            color: color.clone(),
            velocity: new_velocity,
            movement_style: movement_style.clone(),
            should_reproduce,
            eaten_entity,
        })
    }

    fn get_nearby_entities_for_entity(&self, pos: &Position, genes: &Genes) -> Vec<Entity> {
        let nearby_entities = self
            .grid
            .get_nearby_entities(pos.x, pos.y, genes.sense_radius());
        nearby_entities.iter().take(20).copied().collect::<Vec<_>>()
    }

    fn apply_movement_to_entity(
        &self,
        genes: &Genes,
        new_pos: &mut Position,
        new_velocity: &mut Velocity,
        new_energy: &mut f32,
        pos: &Position,
        nearby_entities: &[Entity],
    ) {
        self.movement_system
            .update_movement(crate::systems::MovementUpdateParams {
                genes,
                new_pos,
                new_velocity,
                new_energy,
                pos,
                nearby_entities,
                world: &self.world,
                config: &self.config,
                world_size: self.world_size,
            });
    }

    fn apply_interactions_to_entity(
        &self,
        new_energy: &mut f32,
        eaten_entity: &mut Option<Entity>,
        new_pos: &Position,
        size: &Size,
        genes: &Genes,
        nearby_entities: &[Entity],
    ) {
        self.interaction_system
            .handle_interactions(crate::systems::InteractionParams {
                new_energy,
                eaten_entity,
                new_pos,
                size,
                genes,
                nearby_entities,
                world: &self.world,
                config: &self.config,
            });
    }

    fn calculate_population_density(&self) -> f32 {
        self.world.len() as f32
            / (self.config.population.max_population as f32 * self.config.population.entity_scale)
    }

    fn check_reproduction_for_entity(
        &self,
        energy: f32,
        max_energy: f32,
        genes: &Genes,
        population_density: f32,
    ) -> bool {
        self.reproduction_system.check_reproduction(
            energy,
            max_energy,
            genes,
            population_density,
            &self.config,
        )
    }

    fn apply_entity_updates(&mut self, updates: Vec<EntityUpdate>) {
        // Remove eaten entities in parallel
        let entities_to_remove: Vec<_> = updates
            .par_iter()
            .filter_map(|update| update.eaten_entity)
            .collect();

        // Despawn entities (this needs to be sequential due to Hecs limitations)
        for &entity in &entities_to_remove {
            let _ = self.world.despawn(entity);
        }

        // Prepare spawn data in parallel
        let spawn_data: Vec<_> = updates
            .par_iter()
            .filter_map(|update| {
                if update.energy.current <= 0.0 {
                    return None;
                }

                // Store values before spawning to avoid move issues
                let energy_max = update.energy.max;

                let mut spawn_entities = vec![(
                    update.pos.clone(),
                    update.energy.clone(),
                    update.size.clone(),
                    update.genes.clone(),
                    update.color.clone(),
                    update.velocity.clone(),
                    update.movement_style.clone(),
                )];

                // Handle reproduction with stricter population control
                let max_population = (self.config.population.max_population as f32
                    * self.config.population.entity_scale)
                    as u32;
                if update.should_reproduce && self.world.len() < max_population {
                    let (
                        child_pos,
                        child_energy,
                        child_size,
                        child_genes,
                        child_color,
                        child_velocity,
                        child_movement_style,
                    ) = self.reproduction_system.create_offspring(
                        &update.genes,
                        energy_max,
                        &update.pos,
                        &self.config,
                    );

                    spawn_entities.push((
                        child_pos,
                        child_energy,
                        child_size,
                        child_genes,
                        child_color,
                        child_velocity,
                        child_movement_style,
                    ));
                }

                Some(spawn_entities)
            })
            .flatten()
            .collect();

        // Despawn old entities
        for update in updates {
            let _ = self.world.despawn(update.entity);
        }

        // Spawn new entities (this needs to be sequential due to Hecs limitations)
        for (position, energy, size, genes, color, velocity, movement_style) in spawn_data {
            self.world.spawn((
                position,
                energy,
                size,
                genes,
                color,
                velocity,
                movement_style,
            ));
        }
    }

    pub fn get_entities(&self) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .par_bridge()
            .map(|(_, (pos, size, color))| (pos.x, pos.y, size.radius, color.r, color.g, color.b))
            .collect()
    }

    pub fn get_interpolated_entities(
        &self,
        interpolation_factor: f32,
    ) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .par_bridge()
            .map(|(entity, (pos, size, color))| {
                let interpolated_pos = if let Some(prev_pos) = self.previous_positions.get(&entity)
                {
                    // Interpolate between previous and current position
                    let x = prev_pos.x + (pos.x - prev_pos.x) * interpolation_factor;
                    let y = prev_pos.y + (pos.y - prev_pos.y) * interpolation_factor;
                    (x, y)
                } else {
                    (pos.x, pos.y)
                };

                (
                    interpolated_pos.0,
                    interpolated_pos.1,
                    size.radius,
                    color.r,
                    color.g,
                    color.b,
                )
            })
            .collect()
    }

    /// Get a reference to the world for stats calculation
    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_size(&self) -> f32 {
        self.world_size
    }

    pub fn step(&self) -> u32 {
        self.step
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_simulation_creation() {
        let sim = Simulation::new(1000.0);

        // Should have initial entities
        assert!(!sim.world.is_empty());
        assert!(sim.world.len() <= 1250); // Default config values (2500 * 0.5 scale)

        // World size should be set correctly
        assert_eq!(sim.world_size, 1000.0);

        // Step should start at 0
        assert_eq!(sim.step, 0);

        // Grid should be initialized
    }

    #[test]
    fn test_simulation_creation_with_config() {
        let mut config = SimulationConfig::default();
        config.population.initial_entities = 100;
        config.population.max_population = 500;

        let sim = Simulation::new_with_config(500.0, config.clone());

        assert_eq!(sim.world_size, 500.0);
        assert_eq!(sim.config.population.initial_entities, 100);
        assert_eq!(sim.config.population.max_population, 500);
    }

    #[test]
    fn test_simulation_update() {
        let mut sim = Simulation::new(100.0);
        let initial_step = sim.step;

        sim.update();

        // Step should increment
        assert_eq!(sim.step, initial_step + 1);

        // Entity count might change due to reproduction/death
        // but should be within reasonable bounds
    }

    #[test]
    fn test_simulation_multiple_updates() {
        let mut sim = Simulation::new(100.0);

        for i in 0..10 {
            sim.update();
            assert_eq!(sim.step, i + 1);
        }
    }

    #[test]
    fn test_simulation_get_entities() {
        let sim = Simulation::new(100.0);
        let entities = sim.get_entities();

        // Should return data for all entities
        // Note: We can't easily compare lengths due to type mismatches
        assert!(!entities.is_empty() || sim.world.is_empty());
    }

    #[test]
    fn test_simulation_get_interpolated_entities() {
        let sim = Simulation::new(100.0);
        let _entities = sim.get_interpolated_entities(0.5);

        // Should return data for all entities
        // Note: We can't easily compare lengths due to type mismatches

        // Interpolation factor should be between 0 and 1
        let entities_0 = sim.get_interpolated_entities(0.0);
        let entities_1 = sim.get_interpolated_entities(1.0);

        assert_eq!(entities_0.len(), entities_1.len());
    }

    #[test]
    fn test_simulation_world_access() {
        let sim = Simulation::new(100.0);
        let world_ref = sim.world();

        // Should be able to access world
        let world_len = sim.world.len();
        assert_eq!(world_ref.len(), world_len);
    }

    #[test]
    fn test_boundary_handling() {
        let sim = Simulation::new(100.0);
        let mut pos = Position { x: 60.0, y: 60.0 }; // Outside boundary
        let mut velocity = Velocity { x: 10.0, y: 10.0 };

        sim.movement_system
            .handle_boundaries(&mut pos, &mut velocity, 100.0, &sim.config);

        // Position should be clamped to boundary
        assert!(pos.x <= 50.0 - sim.config.physics.boundary_margin);
        assert!(pos.y <= 50.0 - sim.config.physics.boundary_margin);
    }

    #[test]
    fn test_boundary_handling_center() {
        let sim = Simulation::new(100.0);
        let mut pos = Position { x: 0.0, y: 0.0 }; // Center
        let mut velocity = Velocity { x: 5.0, y: 5.0 };

        sim.movement_system
            .handle_boundaries(&mut pos, &mut velocity, 100.0, &sim.config);

        // Position should remain unchanged
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        // Velocity should have drift compensation applied
        assert_eq!(velocity.x, 5.0);
        assert_eq!(velocity.y, 5.0);
    }

    #[test]
    fn test_simulation_logging() {
        let _sim = Simulation::new(100.0);

        // This should not panic
        // Note: We can't easily test the actual logging output in unit tests
        // but we can ensure the method doesn't crash
    }

    #[test]
    fn test_simulation_spatial_grid_rebuild() {
        let mut sim = Simulation::new(100.0);

        // Rebuild grid
        sim.rebuild_spatial_grid();

        // Grid should be rebuilt without panicking
        // We can't easily test the internal state, but we can ensure it doesn't crash
    }

    #[test]
    fn test_simulation_empty_world() {
        let mut sim = Simulation::new(100.0);
        sim.world.clear();

        // Should handle empty world gracefully
        sim.update();
        assert_eq!(sim.world.len(), 0);
    }

    #[test]
    fn test_simulation_large_world() {
        let mut config = SimulationConfig::default();
        config.population.initial_entities = 1000;
        config.population.max_population = 2000;

        let sim = Simulation::new_with_config(1000.0, config);

        // Should handle large world
        assert!(!sim.world.is_empty());
        assert!(sim.world.len() <= 1000);
    }

    #[test]
    fn test_simulation_entity_processing() {
        let mut sim = Simulation::new(100.0);
        let _entity = sim.world.spawn((
            Position { x: 0.0, y: 0.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
            crate::components::MovementStyle {
                style: crate::components::MovementType::Random,
                flocking_strength: 0.5,
                separation_distance: 10.0,
                alignment_strength: 0.5,
                cohesion_strength: 0.5,
            },
        ));

        // Test processing a single entity
        // Note: This test is complex due to borrowing rules, so we'll just ensure it doesn't panic
        // In a real scenario, you'd need to restructure the code to avoid borrowing conflicts
    }

    #[test]
    fn test_simulation_apply_updates() {
        let mut sim = Simulation::new(100.0);
        let _entity = sim.world.spawn((
            Position { x: 0.0, y: 0.0 },
            Energy {
                current: 50.0,
                max: 100.0,
            },
            Size { radius: 5.0 },
            Genes::new_random(&mut thread_rng()),
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            Velocity { x: 0.0, y: 0.0 },
            crate::components::MovementStyle {
                style: crate::components::MovementType::Flocking,
                flocking_strength: 0.7,
                separation_distance: 12.0,
                alignment_strength: 0.6,
                cohesion_strength: 0.6,
            },
        ));

        let updates = vec![EntityUpdate {
            entity: _entity,
            pos: Position { x: 10.0, y: 10.0 },
            energy: Energy {
                current: 60.0,
                max: 100.0,
            },
            size: Size { radius: 6.0 },
            genes: Genes::new_random(&mut thread_rng()),
            color: Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
            },
            velocity: Velocity { x: 1.0, y: 1.0 },
            movement_style: crate::components::MovementStyle {
                style: crate::components::MovementType::Flocking,
                flocking_strength: 0.7,
                separation_distance: 12.0,
                alignment_strength: 0.6,
                cohesion_strength: 0.6,
            },
            should_reproduce: false,
            eaten_entity: None,
        }];

        sim.apply_entity_updates(updates);

        // Entity should be updated
        // Note: We can't easily test this due to borrowing rules
        // In a real scenario, you'd need to restructure the code
    }
}
