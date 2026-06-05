#![allow(clippy::type_complexity)]

use crate::components::{Color, Energy, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use crate::spatial_grid::SpatialGrid;
use crate::stats::SimulationStats;
use crate::systems::{
    creature_bundle, EnergySystem, EntityContext, InteractionSystem, MovementSystem, NeighborCache,
    NeighborSnapshot, ReproductionSystem, System,
};
use hecs::*;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

pub(crate) mod rng;

pub(crate) use rng::FastRng;
use rng::{
    generate_particle_matrix, mix_seed, BLOOM_SALT, CULL_SALT, DEFAULT_SEED, OFFSPRING_SALT,
};

// Simulation state
pub struct EntityUpdate {
    pub entity: Entity,
    pub pos: Position,
    pub energy: Energy,
    pub size: Size,
    pub velocity: Velocity,
    pub should_reproduce: bool,
    pub eaten_entity: Option<Entity>,
}

pub struct Simulation {
    world: World,
    world_size: f32,
    step: u32,
    grid: SpatialGrid,
    neighbor_cache: NeighborCache,
    previous_positions: HashMap<Entity, Position>, // For smooth interpolation
    config: SimulationConfig,
    seed: u64,
    /// Global particle-life interaction matrix (see `generate_particle_matrix`).
    particle_matrix: [[f32; 6]; 6],

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
        Self::new_with_config_seeded(world_size, config, DEFAULT_SEED)
    }

    pub fn new_with_config_seeded(world_size: f32, config: SimulationConfig, seed: u64) -> Self {
        let mut world = World::new();
        let grid = SpatialGrid::new(config.physics.grid_cell_size);

        Self::spawn_initial_entities(&mut world, seed, world_size, &config);

        Self {
            world,
            world_size,
            step: 0,
            grid,
            neighbor_cache: NeighborCache::new(),
            previous_positions: HashMap::new(),
            config,
            seed,
            particle_matrix: generate_particle_matrix(seed),
            movement_system: MovementSystem,
            interaction_system: InteractionSystem,
            energy_system: EnergySystem,
            reproduction_system: ReproductionSystem,
        }
    }

    fn spawn_initial_entities(
        world: &mut World,
        seed: u64,
        world_size: f32,
        config: &SimulationConfig,
    ) {
        let total_entities =
            (config.population.initial_entities as f32 * config.population.entity_scale) as usize;
        let spawn_radius = world_size * config.population.spawn_radius_factor;

        for i in 0..total_entities {
            // Each initial entity gets its own deterministic RNG stream.
            let mut rng = FastRng::seed_from_u64(mix_seed(seed, i as u64, 0));

            // Use perfectly uniform distribution in a circle
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = spawn_radius * rng.gen::<f32>().sqrt(); // Square root for uniform distribution
            let x = distance * angle.cos();
            let y = distance * angle.sin();

            let genes = Genes::new_random(&mut rng);
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

    /// Instantly remove roughly `fraction` of the population (the user "cull"
    /// action). Not part of the deterministic tick — it's an external
    /// perturbation, and the ecosystem re-settles afterwards.
    pub fn cull(&mut self, fraction: f32) {
        let frac = fraction.clamp(0.0, 1.0);
        if frac <= 0.0 {
            return;
        }
        let mut rng = FastRng::seed_from_u64(mix_seed(self.seed, CULL_SALT, self.step as u64));
        let doomed: Vec<Entity> = self
            .world
            .query::<&Position>()
            .iter()
            .map(|(entity, _)| entity)
            .filter(|_| rng.gen::<f32>() < frac)
            .collect();
        for entity in doomed {
            let _ = self.world.despawn(entity);
        }
    }

    /// Instantly spawn a burst of `count` fresh random creatures near the centre
    /// (the user "bloom" action), capped so it cannot exceed the population limit.
    pub fn bloom(&mut self, count: u32) {
        let cap = (self.config.population.max_population as f32
            * self.config.population.entity_scale) as usize;
        let room = cap.saturating_sub(self.world.len() as usize);
        let n = (count as usize).min(room);
        let spawn_radius = self.world_size * self.config.population.spawn_radius_factor;
        for i in 0..n {
            let mut rng = FastRng::seed_from_u64(mix_seed(
                self.seed,
                BLOOM_SALT ^ i as u64,
                self.step as u64,
            ));
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let distance = spawn_radius * rng.gen::<f32>().sqrt();
            let genes = Genes::new_random(&mut rng);
            let energy = rng.gen_range(15.0..75.0);
            self.world.spawn(creature_bundle(
                Position {
                    x: distance * angle.cos(),
                    y: distance * angle.sin(),
                },
                energy,
                energy * 1.3,
                genes,
                self.config.physics.max_entity_radius,
                self.config.physics.min_entity_radius,
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

    /// Rebuild the spatial grid and the neighbour cache for this tick. Both are
    /// derived from the same world snapshot in one serial pass, so neighbour
    /// reads during the compute phase hit contiguous cached data instead of
    /// scattered `world.get` lookups.
    fn rebuild_spatial_grid(&mut self) {
        self.grid.clear();
        self.neighbor_cache.clear();

        for (entity, (pos, genes, energy, size, velocity)) in self
            .world
            .query::<(&Position, &Genes, &Energy, &Size, &Velocity)>()
            .iter()
        {
            self.grid.insert(entity, pos.x, pos.y);
            self.neighbor_cache.insert(
                entity,
                NeighborSnapshot {
                    pos: pos.clone(),
                    genes: genes.clone(),
                    energy: energy.clone(),
                    size: size.clone(),
                    velocity: velocity.clone(),
                },
            );
        }
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
            cache: &self.neighbor_cache,
            particle_matrix: &self.particle_matrix,
            config: &self.config,
            world_size: self.world_size,
            population_density,
            energy_max: energy.max,
            new_pos: pos.clone(),
            new_velocity: velocity.clone(),
            new_energy: energy.current,
            should_reproduce: false,
            eaten_entity: None,
            rng: FastRng::seed_from_u64(mix_seed(
                self.seed,
                entity.to_bits().get(),
                self.step as u64,
            )),
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
            velocity: ctx.new_velocity,
            should_reproduce: ctx.should_reproduce,
            eaten_entity: ctx.eaten_entity,
        })
    }

    fn get_nearby_entities_for_entity(&self, pos: &Position, genes: &Genes) -> Vec<Entity> {
        let radius = genes.sense_radius();
        let r2 = radius * radius;
        let candidates = self.grid.get_nearby_entities(pos.x, pos.y, radius);

        // Deterministic nearest-N selection: rank candidates by (distance, id).
        // Unlike a random subset this is order-independent (so the sim is
        // reproducible), and "nearest" removes the directional bias a fixed-order
        // truncation would introduce.
        let mut scored: Vec<(f32, u64, Entity)> = candidates
            .iter()
            .filter_map(|&e| {
                self.neighbor_cache.get(&e).and_then(|n| {
                    let dx = n.pos.x - pos.x;
                    let dy = n.pos.y - pos.y;
                    let d2 = dx * dx + dy * dy;
                    (d2 > 0.001 && d2 <= r2).then_some((d2, e.to_bits().get(), e))
                })
            })
            .collect();
        scored.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        scored.into_iter().take(20).map(|(_, _, e)| e).collect()
    }

    fn calculate_population_density(&self) -> f32 {
        self.world.len() as f32
            / (self.config.population.max_population as f32 * self.config.population.entity_scale)
    }

    fn apply_entity_updates(&mut self, mut updates: Vec<EntityUpdate>) {
        let max_population = (self.config.population.max_population as f32
            * self.config.population.entity_scale) as usize;
        // Soft population cap: as before, every reproducing parent is tested
        // against the same start-of-tick population baseline.
        let baseline = self.world.len() as usize;

        // Canonical order so the structural mutations below (despawn/spawn) run
        // deterministically — this keeps entity-id assignment, and thus the
        // per-entity RNG keyed by id, reproducible across runs.
        updates.sort_by_key(|u| u.entity.to_bits());

        // Entities eaten by a predator this tick are removed. Collect them first
        // so they are neither updated in place nor allowed to reproduce.
        let eaten: HashSet<Entity> = updates.iter().filter_map(|u| u.eaten_entity).collect();

        // Collect deaths and queue offspring (read-only over self and the world).
        let mut dead: Vec<Entity> = Vec::new();
        let mut offspring = Vec::new();
        for update in &updates {
            if update.energy.current <= 0.0 {
                dead.push(update.entity);
            } else if update.should_reproduce
                && !eaten.contains(&update.entity)
                && baseline < max_population
            {
                // Genes never change for a living entity, so its current genes in
                // the world equal what it carried through the compute phase. Fetch
                // them here rather than cloning genes into every EntityUpdate —
                // only the <1% that reproduce each tick ever need them.
                if let Ok(parent_genes) = self.world.get::<&Genes>(update.entity) {
                    let mut rng = FastRng::seed_from_u64(mix_seed(
                        self.seed ^ OFFSPRING_SALT,
                        update.entity.to_bits().get(),
                        self.step as u64,
                    ));
                    offspring.push(self.reproduction_system.create_offspring(
                        &parent_genes,
                        update.energy.max,
                        &update.pos,
                        &self.config,
                        &mut rng,
                    ));
                }
            }
        }

        // Apply each survivor's new state in place — no despawn/respawn churn.
        // Genes, color, and movement style never change for an existing entity, so
        // only the mutable components are written. Eaten entities are skipped here
        // and despawned below.
        let updated: HashMap<Entity, &EntityUpdate> = updates
            .iter()
            .filter(|u| u.energy.current > 0.0 && !eaten.contains(&u.entity))
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

        // Births, deaths, and predation removals are the only structural
        // mutations (serial — hecs). Removal order is canonical so id recycling
        // stays deterministic.
        for entity in dead {
            let _ = self.world.despawn(entity);
        }
        let mut eaten_removal: Vec<Entity> = eaten.iter().copied().collect();
        eaten_removal.sort_by_key(|e| e.to_bits());
        for entity in eaten_removal {
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

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_size(&self) -> f32 {
        self.world_size
    }

    pub fn step(&self) -> u32 {
        self.step
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }

    /// Mutable access to the live config, so a single tunable can be changed in
    /// place without cloning the whole struct. `Simulation` owns the only copy.
    pub fn config_mut(&mut self) -> &mut SimulationConfig {
        &mut self.config
    }
}

#[cfg(test)]
mod tests;
