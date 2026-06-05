use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub entity_scale: f32,
    pub max_population: u32,
    pub initial_entities: usize,
    pub spawn_radius_factor: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub max_velocity: f32,
    pub max_entity_radius: f32,
    pub min_entity_radius: f32,
    pub grid_cell_size: f32,
    pub boundary_margin: f32,
    pub interaction_radius_offset: f32,
    pub velocity_bounce_factor: f32,
    pub edge_repulsion_strength: f32,
    pub particle_force_scale: f32,
    pub particle_friction: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnergyConfig {
    pub size_energy_cost_factor: f32,
    pub movement_energy_cost: f32,
    /// Primary production: energy each creature grazes from the food field per
    /// tick, scaled by `(1 - population_density)` so the field is finite. This is
    /// the ecosystem's only energy *input* (eating just transfers it), so it sets
    /// the carrying capacity — without it the closed system decays to a few
    /// survivors. Production is no longer uniform: it is concentrated into the
    /// drifting [`FoodConfig`] patches, so the per-tick gain depends on how close
    /// a creature is to food. `ambient_energy_gain` is the live "Food" slider that
    /// scales the whole field's richness up or down.
    pub ambient_energy_gain: f32,
}

/// The drifting food field (see `simulation::food`). Production is split into a
/// thin uniform *base* everywhere plus drifting *patches* that wander, regrow,
/// and deplete when grazed, so the world grows gathering spots worth migrating
/// to while staying alive between them. The carrying capacity is preserved by
/// concentrating the *same* total production in space (base + patches average to
/// `EnergyConfig::ambient_energy_gain` across the world) rather than removing it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodConfig {
    /// Number of patches in the field.
    pub patch_count: usize,
    /// Falloff radius of each patch as a *fraction of world size*, so the food
    /// structure is the same relative scale at any window resolution.
    pub patch_radius_frac: f32,
    /// How fast a patch drifts across the world per tick, as a fraction of world
    /// size — scale-invariant, so patches wander at the same relative pace.
    pub drift_speed: f32,
    /// How fast a patch regrows toward its capacity per tick.
    pub regen_rate: f32,
    /// How strongly grazing depletes a patch (per unit of energy taken).
    pub graze_rate: f32,
    /// Gentle attraction pulling creatures up the local food gradient, so they
    /// migrate to and gather at patches. Modulated per-creature by genes.
    pub seek_strength: f32,
    /// Fraction of total production routed into the drifting patches (0..1); the
    /// remainder is the thin uniform base. Higher = more dramatic gathering spots
    /// but a barer field between them; the base keeps the between-patch creatures
    /// alive so the population stays stable instead of collapsing onto the patches.
    pub patch_fraction: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReproductionConfig {
    pub reproduction_energy_threshold: f32,
    pub reproduction_energy_cost: f32,
    pub child_energy_factor: f32,
    pub child_spawn_radius: f32,
    pub population_density_factor: f32,
    pub min_reproduction_chance: f32,
    pub death_chance_factor: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub population: PopulationConfig,
    pub physics: PhysicsConfig,
    pub energy: EnergyConfig,
    pub reproduction: ReproductionConfig,
    #[serde(default)]
    pub food: FoodConfig,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            population: PopulationConfig {
                entity_scale: 0.5,
                max_population: 10000,
                initial_entities: 2500,
                spawn_radius_factor: 0.2,
            },
            physics: PhysicsConfig {
                max_velocity: 2.0,
                max_entity_radius: 20.0,
                min_entity_radius: 1.0,
                grid_cell_size: 25.0,
                boundary_margin: 5.0,
                interaction_radius_offset: 6.0,
                velocity_bounce_factor: 0.8,
                edge_repulsion_strength: 0.3,
                particle_force_scale: 0.15,
                particle_friction: 0.95,
            },
            energy: EnergyConfig {
                size_energy_cost_factor: 0.15,
                movement_energy_cost: 0.1,
                ambient_energy_gain: 0.9,
            },
            reproduction: ReproductionConfig {
                reproduction_energy_threshold: 0.6,
                reproduction_energy_cost: 0.7,
                child_energy_factor: 0.4,
                child_spawn_radius: 15.0,
                population_density_factor: 0.8,
                min_reproduction_chance: 0.05,
                death_chance_factor: 0.04,
            },
            food: FoodConfig::default(),
        }
    }
}

impl Default for FoodConfig {
    fn default() -> Self {
        // 65% of production stays as a thin uniform base and 35% is concentrated
        // into ten broad, overlapping, drifting patches (see `patch_fraction`).
        // The base keeps between-patch creatures alive so the population is stable
        // across seeds, while the patches and the food-seeking force make
        // creatures visibly migrate to and gather at the brighter cores. Tuned via
        // the `sweep_food_tuning` harness against many seeds to keep the ecosystem
        // healthy (see `test_population_sustains_via_primary_production`).
        Self {
            patch_count: 7,
            patch_radius_frac: 0.11,
            drift_speed: 0.0002,
            regen_rate: 0.05,
            graze_rate: 0.008,
            seek_strength: 1.0,
            patch_fraction: 0.35,
        }
    }
}

#[cfg(test)]
mod tests;
