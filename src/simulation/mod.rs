#![allow(clippy::type_complexity)]

use crate::components::{Color, Energy, MovementType, Position, Size, Velocity};
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

pub(crate) mod food;
pub(crate) mod rng;

use food::{FoodField, FoodFieldConfig};
pub(crate) use rng::FastRng;
use rng::{
    generate_particle_matrix, mix_seed, BLOOM_SALT, CULL_SALT, DEFAULT_SEED, OFFSPRING_SALT,
};

/// Max simultaneous visual effects. Effects are short-lived, so the live set
/// stays well under this; new ones past the cap are dropped (purely cosmetic).
const EFFECT_CAP: usize = 400;

/// A transient visual flourish (predation flash, bloom burst, cull shockwave) —
/// an expanding glowing ring the renderer draws. Pure cosmetic side-data derived
/// from events that already happen deterministically; never read back into the
/// simulation, so it can't affect determinism.
#[derive(Clone)]
struct Effect {
    x: f32,
    y: f32,
    /// Final ring radius in world units.
    base_radius: f32,
    age: u32,
    max_age: u32,
    /// 0 = predation flash, 1 = bloom/seed burst, 2 = cull shockwave.
    kind: f32,
}

/// Stable numeric id for a movement style, packed into the render buffer so the
/// shader can vary a creature's look by behaviour (e.g. mark predators).
fn movement_style_id(style: &MovementType) -> f32 {
    match style {
        MovementType::Random => 0.0,
        MovementType::Flocking => 1.0,
        MovementType::Solitary => 2.0,
        MovementType::Predatory => 3.0,
        MovementType::Grazing => 4.0,
    }
}

// Simulation state
pub struct EntityUpdate {
    pub entity: Entity,
    pub pos: Position,
    pub energy: Energy,
    pub size: Size,
    pub velocity: Velocity,
    /// Energy this creature grazed from the food field this tick (used by the
    /// serial apply phase to deplete the patches it fed on).
    pub grazed: f32,
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
    /// Drifting, regenerating food patches — the spatial primary-production field.
    food_field: FoodField,
    /// Slow-moving "crowding pressure": a low-pass filter of the population
    /// density that the density-dependent death rate reads instead of the live
    /// density. Because it *lags* the population, mortality arrives late — the
    /// crowd overshoots its carrying capacity, the accumulated pressure then
    /// crashes it back, and the cycle repeats. This delayed density dependence is
    /// the engine of the boom/bust waves. Updated once per tick in the serial
    /// phase; read immutably by the parallel compute.
    crowding_pressure: f32,
    /// Transient visual effects (flashes/rings) drawn by the renderer. Cosmetic
    /// side-data only; aged each tick and never read back into the sim.
    effects: Vec<Effect>,

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
    crowding_pressure: f32,
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

        let food_field = FoodField::new(
            seed,
            world_size,
            Self::food_field_config(&config, world_size),
        );

        // Start the pressure at the initial density so the very first ticks aren't
        // an artificial mortality holiday (computed before `config` is moved in).
        let crowding_pressure = Self::initial_density(&config);

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
            food_field,
            crowding_pressure,
            effects: Vec::new(),
            movement_system: MovementSystem,
            interaction_system: InteractionSystem,
            energy_system: EnergySystem,
            reproduction_system: ReproductionSystem,
        }
    }

    /// Derive the food field's tuning from the live config so the world-average
    /// production matches the old uniform `ambient_energy_gain` — the carrying
    /// capacity is preserved, the food is merely concentrated in space. The
    /// `patch_fraction` of production goes into the patches and the rest into a
    /// uniform `base`. A quadratic-falloff disc integrates to `π r² / 3`, so the
    /// per-patch peak that makes the patches' spatial average equal
    /// `ambient · patch_fraction` is `ambient · patch_fraction · world_area /
    /// (patch_count · π r² / 3)`. The live "Food" slider scales `ambient`, hence
    /// the whole field.
    fn food_field_config(config: &SimulationConfig, world_size: f32) -> FoodFieldConfig {
        let f = &config.food;
        let ambient = config.energy.ambient_energy_gain;
        let frac = f.patch_fraction.clamp(0.0, 1.0);
        let world_area = world_size * world_size;
        // Patch radius and drift are world-relative so the food structure looks the
        // same at any window resolution.
        let r = f.patch_radius_frac * world_size;
        let per_patch_integral = std::f32::consts::PI * r * r / 3.0;
        let denom = (f.patch_count.max(1) as f32) * per_patch_integral;
        let patch_peak = ambient * frac * world_area / denom;
        FoodFieldConfig {
            patch_count: f.patch_count,
            patch_radius: r,
            drift_speed: f.drift_speed * world_size,
            regen_rate: f.regen_rate,
            graze_rate: f.graze_rate,
            graze_floor: f.graze_floor.clamp(0.0, 1.0),
            base: ambient * (1.0 - frac),
            patch_peak,
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
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = spawn_radius * rng.random::<f32>().sqrt(); // Square root for uniform distribution
            let x = distance * angle.cos();
            let y = distance * angle.sin();

            let genes = Genes::new_random(&mut rng);
            let energy = rng.random_range(15.0..75.0);

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
            .query::<Entity>()
            .iter()
            .filter(|_| rng.random::<f32>() < frac)
            .collect();
        for entity in doomed {
            let _ = self.world.despawn(entity);
        }
        // A cool shockwave ripple radiating from the centre.
        self.add_effect(0.0, 0.0, self.world_size * 0.6, 45, 2.0);
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
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = spawn_radius * rng.random::<f32>().sqrt();
            let genes = Genes::new_random(&mut rng);
            let energy = rng.random_range(15.0..75.0);
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
        // A bright burst of life at the centre.
        self.add_effect(0.0, 0.0, self.world_size * 0.14, 30, 1.0);
    }

    /// Spawn a tight burst of `count` fresh creatures around world `(x, y)` — the
    /// user clicking the canvas to seed life. Like [`bloom`](Self::bloom) but
    /// centred on the cursor with a small spread, clamped to the world and the
    /// population cap.
    pub fn bloom_at(&mut self, x: f32, y: f32, count: u32) {
        let cap = (self.config.population.max_population as f32
            * self.config.population.entity_scale) as usize;
        let room = cap.saturating_sub(self.world.len() as usize);
        let n = (count as usize).min(room);
        let spread = self.world_size * 0.045;
        let half = self.world_size / 2.0;
        for i in 0..n {
            let mut rng = FastRng::seed_from_u64(mix_seed(
                self.seed,
                BLOOM_SALT ^ (i as u64).wrapping_mul(0x9E37_79B9),
                self.step as u64,
            ));
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = spread * rng.random::<f32>().sqrt();
            let genes = Genes::new_random(&mut rng);
            let energy = rng.random_range(15.0..75.0);
            self.world.spawn(creature_bundle(
                Position {
                    x: (x + distance * angle.cos()).clamp(-half, half),
                    y: (y + distance * angle.sin()).clamp(-half, half),
                },
                energy,
                energy * 1.3,
                genes,
                self.config.physics.max_entity_radius,
                self.config.physics.min_entity_radius,
            ));
        }
        // A seed-of-life burst at the cursor, even if the cap left no room.
        self.add_effect(x, y, self.world_size * 0.08, 26, 1.0);
    }

    /// Drop a patch of food at world `(x, y)` — the user clicking to feed the
    /// world. Creatures migrate to it and graze it down until it disappears.
    pub fn drop_food(&mut self, x: f32, y: f32) {
        self.food_field.drop_food(x, y);
        // A soft green nourishment ring as a drop cue.
        self.add_effect(x, y, self.world_size * 0.05, 18, 1.0);
    }

    /// Queue a transient visual effect (cosmetic ring/flash). Dropped silently
    /// once the active set hits `EFFECT_CAP`.
    fn add_effect(&mut self, x: f32, y: f32, base_radius: f32, max_age: u32, kind: f32) {
        if self.effects.len() >= EFFECT_CAP {
            return;
        }
        self.effects.push(Effect {
            x,
            y,
            base_radius,
            age: 0,
            max_age,
            kind,
        });
    }

    /// Advance every effect by one tick and drop the expired ones.
    fn age_effects(&mut self) {
        for e in &mut self.effects {
            e.age += 1;
        }
        self.effects.retain(|e| e.age < e.max_age);
    }

    pub fn update(&mut self) {
        self.step += 1;
        self.update_simulation();

        // Periodic console metrics for the browser run. Skipped under test so the
        // long headless dynamics probes aren't drowned in per-tick output (the log
        // is pure output — gating it changes no simulation state).
        #[cfg(not(test))]
        if self.step.is_multiple_of(60) {
            self.log_simulation_metrics();
        }
    }

    #[cfg_attr(test, allow(dead_code))]
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
        // Advance the food field serially before the compute, so the patches are
        // fixed for the whole tick and the parallel per-entity reads see a stable
        // snapshot (determinism + the read→compute→apply invariant).
        self.food_field.update(self.seed, self.step);
        // Relax the lagged crowding pressure toward the live density. Reading this
        // (rather than the instantaneous density) in the death rate makes mortality
        // lag the population, producing the boom/bust overshoot. Updated here in the
        // serial phase so the parallel compute sees one fixed value for the tick.
        let density = self.calculate_population_density();
        let rate = self
            .config
            .reproduction
            .crowding_pressure_rate
            .clamp(0.0, 1.0);
        self.crowding_pressure += (density - self.crowding_pressure) * rate;
        self.rebuild_spatial_grid();
        let updates = self.process_entities_parallel();
        self.apply_entity_updates(updates);
        self.age_effects();
    }

    /// The initial population density (used to seed the crowding pressure).
    fn initial_density(config: &SimulationConfig) -> f32 {
        let initial = config.population.initial_entities as f32 * config.population.entity_scale;
        let cap = config.population.max_population as f32 * config.population.entity_scale;
        if cap > 0.0 {
            initial / cap
        } else {
            0.0
        }
    }

    fn store_previous_positions(&mut self) {
        self.previous_positions.clear();
        for (entity, pos) in self.world.query::<(Entity, &Position)>().iter() {
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

        for (entity, pos, genes, energy, size, velocity) in self
            .world
            .query::<(Entity, &Position, &Genes, &Energy, &Size, &Velocity)>()
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
        let crowding_pressure = self.crowding_pressure;
        self.world
            .query::<(Entity, &Position, &Energy, &Size, &Genes, &Velocity)>()
            .iter()
            .par_bridge()
            .filter_map(|(entity, pos, energy, size, genes, velocity)| {
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
                    crowding_pressure,
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
            crowding_pressure,
        } = params;

        let nearby_entities = self.get_nearby_entities_for_entity(pos, genes);

        let mut ctx = EntityContext {
            genes,
            pos,
            size,
            nearby_entities: &nearby_entities,
            cache: &self.neighbor_cache,
            particle_matrix: &self.particle_matrix,
            food_field: &self.food_field,
            config: &self.config,
            world_size: self.world_size,
            population_density,
            crowding_pressure,
            energy_max: energy.max,
            new_pos: pos.clone(),
            new_velocity: velocity.clone(),
            new_energy: energy.current,
            grazed: 0.0,
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
            grazed: ctx.grazed,
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
        for (entity, pos, velocity, energy, size) in
            self.world
                .query_mut::<(Entity, &mut Position, &mut Velocity, &mut Energy, &mut Size)>()
        {
            if let Some(u) = updated.get(&entity) {
                pos.clone_from(&u.pos);
                velocity.clone_from(&u.velocity);
                energy.clone_from(&u.energy);
                size.clone_from(&u.size);
            }
        }

        // Deplete the food patches each surviving creature grazed this tick.
        // `updates` is sorted by entity id, so this runs in a canonical order and
        // stays deterministic. Grazing here (after the compute) keeps the field
        // fixed for the parallel read and only mutates it in this serial phase.
        for update in &updates {
            if update.grazed > 0.0 && update.energy.current > 0.0 && !eaten.contains(&update.entity)
            {
                self.food_field
                    .graze(update.pos.x, update.pos.y, update.grazed);
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

        // A faint, occasional spark at a few kill sites — strided so they spread
        // out and hard-capped per tick, so even a churning food web shimmers
        // quietly instead of strobing yellow. Purely cosmetic.
        let mut pred_seen = 0u32;
        let mut pred_flashes = 0u32;
        for update in &updates {
            if update.eaten_entity.is_some() && update.energy.current > 0.0 {
                if pred_seen.is_multiple_of(5) && pred_flashes < 3 {
                    self.add_effect(update.pos.x, update.pos.y, self.world_size * 0.008, 12, 0.0);
                    pred_flashes += 1;
                }
                pred_seen += 1;
            }
        }
    }

    /// Per-entity render data:
    /// `(prev_x, prev_y, x, y, radius, r, g, b, health, style_id, speed_norm, sense_norm)`.
    /// `health` is the energy fraction (0..1) — the renderer dims the starving
    /// and brightens the thriving. `style_id` is the movement type (0..4) so the
    /// renderer can give each behaviour its own body-plan. `speed_norm` and
    /// `sense_norm` are the speed and sense-radius genes normalised to 0..1, so
    /// the shader can shape morphology (elongation, sensory halo) by genotype —
    /// making evolution of body plans visible.
    #[allow(clippy::type_complexity)]
    pub fn get_entities(
        &self,
    ) -> Vec<(f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32)> {
        self.world
            .query::<(Entity, &Position, &Size, &Color, &Energy, &Genes)>()
            .iter()
            .par_bridge()
            .map(|(entity, pos, size, color, energy, genes)| {
                let prev_pos = self.previous_positions.get(&entity).unwrap_or(pos);
                let health = if energy.max > 0.0 {
                    (energy.current / energy.max).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let style_id = movement_style_id(&genes.behavior.movement_style.style);
                // Normalise to the gene ranges declared in genes/mod.rs (speed
                // 0.1..6.5, sense 2..180) so the shader gets stable 0..1 traits.
                let speed_norm = ((genes.speed() - 0.1) / 6.4).clamp(0.0, 1.0);
                let sense_norm = ((genes.sense_radius() - 2.0) / 178.0).clamp(0.0, 1.0);
                (
                    prev_pos.x,
                    prev_pos.y,
                    pos.x,
                    pos.y,
                    size.radius,
                    color.r,
                    color.g,
                    color.b,
                    health,
                    style_id,
                    speed_norm,
                    sense_norm,
                )
            })
            .collect()
    }

    /// Render data for the food patches: `(x, y, radius, intensity_fraction)`
    /// per patch, where the fraction is the patch's current intensity over its
    /// capacity (0..1) so the renderer can dim a depleted patch.
    pub fn get_food_patches(&self) -> Vec<(f32, f32, f32, f32)> {
        self.food_field
            .patches()
            .iter()
            .map(|p| {
                let frac = if p.capacity > 0.0 {
                    (p.intensity / p.capacity).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (p.x, p.y, p.radius, frac)
            })
            .collect()
    }

    /// Render data for the transient visual effects:
    /// `(x, y, base_radius, life, life_step, kind)` per effect, where `life` is
    /// `age/max_age` (0..1) and `life_step` is `1/max_age` so the renderer can
    /// interpolate the ring animation smoothly between ticks.
    pub fn get_effects(&self) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        self.effects
            .iter()
            .map(|e| {
                let (life, step) = if e.max_age > 0 {
                    (e.age as f32 / e.max_age as f32, 1.0 / e.max_age as f32)
                } else {
                    (1.0, 0.0)
                };
                (e.x, e.y, e.base_radius, life, step, e.kind)
            })
            .collect()
    }

    /// Centroid and a robust focus radius of the live population, in world units,
    /// so the UI can frame the swarm. The radius is ~2.4× the RMS distance from
    /// the centroid (a handful of stragglers can't blow it up), clamped to a sane
    /// band of the world size.
    pub fn view_focus(&self) -> (f32, f32, f32) {
        let positions: Vec<(f32, f32)> = self
            .world
            .query::<&Position>()
            .iter()
            .map(|p| (p.x, p.y))
            .collect();
        let n = positions.len();
        if n == 0 {
            return (0.0, 0.0, self.world_size * 0.25);
        }
        let inv = 1.0 / n as f32;
        let (sx, sy) = positions
            .iter()
            .fold((0.0f32, 0.0f32), |(ax, ay), &(x, y)| (ax + x, ay + y));
        let (cx, cy) = (sx * inv, sy * inv);
        let var = positions
            .iter()
            .fold(0.0f32, |a, &(x, y)| a + (x - cx).powi(2) + (y - cy).powi(2))
            * inv;
        let radius = (var.sqrt() * 2.4).clamp(self.world_size * 0.06, self.world_size * 0.5);
        (cx, cy, radius)
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
