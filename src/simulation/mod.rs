#![allow(clippy::type_complexity)]

use crate::components::{Color, Energy, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use crate::spatial_grid::SpatialGrid;
use crate::stats::SimulationStats;
use crate::systems::{
    creature_bundle, EnergySystem, EntityContext, InteractionSystem, MovementSystem,
    ReproductionSystem, System,
};
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
    pub velocity: Velocity,
    pub should_reproduce: bool,
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
    velocity: &'a Velocity,
    population_density: f32,
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

            world.spawn(creature_bundle(
                Position { x, y },
                energy,
                energy * 1.3,
                genes,
                config.physics.max_entity_radius,
                config.physics.min_entity_radius,
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

        // Parallel inserts directly into DashMap (thread-safe)
        self.world
            .query::<(&Position,)>()
            .iter()
            .par_bridge()
            .for_each(|(entity, (pos,))| {
                self.grid.insert(entity, pos.x, pos.y);
            });
    }

    fn process_entities_parallel(&self) -> Vec<EntityUpdate> {
        // Population density is constant across the tick (the world is not mutated
        // during the compute phase), so compute it once rather than per entity.
        let population_density = self.calculate_population_density();
        self.world
            .query::<(&Position, &Energy, &Size, &Genes, &Velocity)>()
            .iter()
            .par_bridge()
            .filter_map(|(entity, (pos, energy, size, genes, velocity))| {
                if energy.current <= 0.0 {
                    return None;
                }

                self.process_entity(ProcessEntityParams {
                    entity,
                    pos,
                    energy,
                    size,
                    genes,
                    velocity,
                    population_density,
                })
            })
            .collect()
    }

    fn process_entity(&self, params: ProcessEntityParams) -> Option<EntityUpdate> {
        let ProcessEntityParams {
            entity,
            pos,
            energy,
            size,
            genes,
            velocity,
            population_density,
        } = params;

        let nearby_entities = self.get_nearby_entities_for_entity(pos, genes);

        let mut ctx = EntityContext {
            genes,
            pos,
            size,
            nearby_entities: &nearby_entities,
            world: &self.world,
            config: &self.config,
            world_size: self.world_size,
            population_density,
            energy_max: energy.max,
            new_pos: pos.clone(),
            new_velocity: velocity.clone(),
            new_energy: energy.current,
            should_reproduce: false,
        };

        // Systems run in order over the shared context (movement → interaction →
        // energy → reproduction); size is derived from the final energy.
        self.movement_system.run(&mut ctx);
        self.interaction_system.run(&mut ctx);
        self.energy_system.run(&mut ctx);
        self.reproduction_system.run(&mut ctx);

        let new_size_radius =
            self.energy_system
                .calculate_new_size(ctx.new_energy, genes, &self.config);

        Some(EntityUpdate {
            entity,
            pos: ctx.new_pos,
            energy: Energy {
                current: ctx.new_energy,
                max: energy.max,
            },
            size: Size {
                radius: new_size_radius,
            },
            genes: genes.clone(),
            velocity: ctx.new_velocity,
            should_reproduce: ctx.should_reproduce,
        })
    }

    fn get_nearby_entities_for_entity(&self, pos: &Position, genes: &Genes) -> Vec<Entity> {
        let nearby_entities = self
            .grid
            .get_nearby_entities(pos.x, pos.y, genes.sense_radius());
        nearby_entities.iter().take(20).copied().collect::<Vec<_>>()
    }

    fn calculate_population_density(&self) -> f32 {
        self.world.len() as f32
            / (self.config.population.max_population as f32 * self.config.population.entity_scale)
    }

    fn apply_entity_updates(&mut self, updates: Vec<EntityUpdate>) {
        let max_population = (self.config.population.max_population as f32
            * self.config.population.entity_scale) as usize;
        // Soft population cap: as before, every reproducing parent is tested
        // against the same start-of-tick population baseline.
        let baseline = self.world.len() as usize;

        // Collect deaths and queue offspring (read-only over self and the world).
        let mut dead: Vec<Entity> = Vec::new();
        let mut offspring = Vec::new();
        for update in &updates {
            if update.energy.current <= 0.0 {
                dead.push(update.entity);
            } else if update.should_reproduce && baseline < max_population {
                offspring.push(self.reproduction_system.create_offspring(
                    &update.genes,
                    update.energy.max,
                    &update.pos,
                    &self.config,
                ));
            }
        }

        // Apply each survivor's new state in place — no despawn/respawn churn.
        // Genes, color, and movement style never change for an existing entity, so
        // only the mutable components are written. Predation transferred energy
        // during the compute phase; eaten prey is not removed (see BACKLOG.md).
        let updated: HashMap<Entity, &EntityUpdate> = updates
            .iter()
            .filter(|u| u.energy.current > 0.0)
            .map(|u| (u.entity, u))
            .collect();
        for (entity, (pos, velocity, energy, size)) in
            self.world
                .query_mut::<(&mut Position, &mut Velocity, &mut Energy, &mut Size)>()
        {
            if let Some(u) = updated.get(&entity) {
                pos.clone_from(&u.pos);
                velocity.clone_from(&u.velocity);
                energy.clone_from(&u.energy);
                size.clone_from(&u.size);
            }
        }

        // Births and deaths are the only structural mutations (serial — hecs).
        for entity in dead {
            let _ = self.world.despawn(entity);
        }
        for bundle in offspring {
            self.world.spawn(bundle);
        }
    }

    pub fn get_entities(&self) -> Vec<(f32, f32, f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(&Position, &Size, &Color)>()
            .iter()
            .par_bridge()
            .map(|(entity, (pos, size, color))| {
                let prev_pos = self.previous_positions.get(&entity).unwrap_or(pos);
                (
                    prev_pos.x,
                    prev_pos.y,
                    pos.x,
                    pos.y,
                    size.radius,
                    color.r,
                    color.g,
                    color.b,
                )
            })
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

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_size(&self) -> f32 {
        self.world_size
    }

    pub fn step(&self) -> u32 {
        self.step
    }

    pub fn update_config(&mut self, config: SimulationConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests;
