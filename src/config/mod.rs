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
    /// Fraction of primary production a *predator* (Predatory movement style) can
    /// graze from the food field, in `0..1`. Carnivores live off prey, not the
    /// field: at `0.25` a predator grazes a quarter of what a grazer would, so a
    /// predator lineage only thrives where prey is abundant and **starves when it
    /// has eaten the prey down** — the decoupling that drives predator/prey
    /// boom/bust. Crucially this only throttles predators: every prey lineage
    /// keeps the full food floor, so the *prey* population can never go extinct.
    #[serde(default = "default_predator_graze_fraction")]
    pub predator_graze_fraction: f32,
    /// Extra metabolic upkeep paid only by predators (added to `loss_rate` before
    /// the efficiency divide). A hungry-predator tax: it makes predator numbers
    /// recede quickly once prey thins, so the boom is followed by a real bust
    /// rather than a high predator plateau.
    #[serde(default = "default_predator_upkeep")]
    pub predator_upkeep: f32,
}

fn default_predator_graze_fraction() -> f32 {
    0.6
}
fn default_predator_upkeep() -> f32 {
    0.0
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
    /// How far grazing can strip a patch, as a fraction of its capacity (0..1). A
    /// *low* floor lets a crowded patch be eaten down to almost nothing, so the
    /// patch food crashes under heavy grazing and recovers only as the crowd
    /// starves and disperses — the renewable-resource cycle that drives the whole
    /// population's boom/bust. The inexhaustible uniform `base` (not the patches)
    /// is the real extinction floor, so the patches are free to swing hard.
    #[serde(default = "default_graze_floor")]
    pub graze_floor: f32,
}

fn default_graze_floor() -> f32 {
    0.2
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
    /// Speed at which the lagged "crowding pressure" tracks the live population
    /// density, per tick (0..1). The density-dependent death rate is driven by
    /// this slow-moving pressure rather than the instantaneous density, so
    /// mortality *lags* the population: the crowd overshoots its carrying
    /// capacity before mortality catches up, then the accumulated pressure pulls
    /// it back under, and the cycle repeats. This delayed density dependence is
    /// what turns a flat equilibrium into visible boom/bust waves. Smaller = a
    /// longer, deeper cycle; `1.0` collapses back to the old instantaneous death.
    /// The food floor still guarantees recovery from any trough, so the cycle is
    /// bounded — it can never spiral to extinction.
    #[serde(default = "default_crowding_pressure_rate")]
    pub crowding_pressure_rate: f32,
    /// Population density below which density-dependent mortality is switched off
    /// entirely (and above which it ramps in smoothly over a short band). This is
    /// the **hard safety floor**: the boom/bust trough can dive toward it for
    /// drama, but mortality can never push the population *through* it, so a deep
    /// bust can't spiral to extinction. Set as a fraction of the population cap.
    #[serde(default = "default_death_floor_density")]
    pub death_floor_density: f32,
    /// Negative frequency-dependent selection on hue: how much *same-hue* local
    /// crowding throttles reproduction, beyond plain crowding. A creature ringed
    /// by its own colour is suppressed (the niche is full of its kind), while a
    /// rare colour breeds freely — so no single hue can take the whole world and
    /// several colour lineages coexist. `0` disables it. This is the engine of
    /// visible speciation: distinct colour clusters that wax and wane instead of
    /// collapsing to one hue.
    #[serde(default = "default_hue_crowding_factor")]
    pub hue_crowding_factor: f32,
}

fn default_hue_crowding_factor() -> f32 {
    1.2
}
fn default_crowding_pressure_rate() -> f32 {
    0.006
}
fn default_death_floor_density() -> f32 {
    0.03
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
                max_velocity: 6.0,
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
                // A richer field than the bare-sustenance 0.9: it lifts the
                // carrying capacity so even the weakest seeds stay comfortably
                // above the safety floor while the boom/bust waves play out.
                ambient_energy_gain: 1.3,
                predator_graze_fraction: default_predator_graze_fraction(),
                predator_upkeep: default_predator_upkeep(),
            },
            reproduction: ReproductionConfig {
                reproduction_energy_threshold: 0.6,
                reproduction_energy_cost: 0.7,
                child_energy_factor: 0.4,
                child_spawn_radius: 15.0,
                population_density_factor: 0.8,
                min_reproduction_chance: 0.05,
                death_chance_factor: 0.04,
                crowding_pressure_rate: default_crowding_pressure_rate(),
                death_floor_density: default_death_floor_density(),
                hue_crowding_factor: default_hue_crowding_factor(),
            },
            food: FoodConfig::default(),
        }
    }
}

impl SimulationConfig {
    /// The curated config the web frontend ships. `web/js/app.js` passes an empty
    /// config string to [`WebSimulation`], which selects this; the live panel then
    /// tweaks individual parameters on top of it.
    ///
    /// It is deliberately *separate* from [`Default`]: `Default` is tuned for the
    /// headless dynamics tests, whereas this is tuned for the **look on the site
    /// over a long watch** — a full, lively population that swings in visible
    /// boom/bust waves for ten-plus minutes without starving out or pinning at the
    /// cap. The deltas below are the entire difference from `Default`; keep the
    /// slider start values in `web/index.html` in sync with them. Guarded by
    /// `test_browser_default_sustains_long_run` so the shipped balance can't
    /// silently regress into a die-off again.
    pub fn browser_default() -> Self {
        let mut c = Self::default();
        // Predators reach a touch further for prey, for a livelier food web.
        c.physics.interaction_radius_offset = 8.0;
        // Energy economy: a slightly leaner field and a higher size cost than the
        // test default, so body size still has a real price, balanced to a healthy
        // carrying capacity rather than the old starvation tuning.
        c.energy.size_energy_cost_factor = 0.22;
        c.energy.ambient_energy_gain = 1.1;
        // Breed a little more eagerly and cull a little harder than the test
        // default — together with the food dynamics below this sharpens the
        // boom/bust swing while staying comfortably bounded.
        c.reproduction.reproduction_energy_threshold = 0.45;
        c.reproduction.death_chance_factor = 0.06;
        // Patches regrow a touch slower and are grazed down faster and further than
        // the test default, so a crowd visibly eats a patch out and must move on.
        c.food.regen_rate = 0.04;
        c.food.graze_rate = 0.025;
        c.food.graze_floor = 0.12;
        c
    }
}

impl Default for FoodConfig {
    fn default() -> Self {
        // Production is split between a thin inexhaustible uniform base and a
        // handful of broad, drifting patches. The base is the ecosystem's
        // extinction floor — it keeps between-patch creatures alive so the
        // population can never die out — while the patches are a *renewable
        // resource* that the population grazes down and that regrows slowly. That
        // graze-down/regrow loop, with a low `graze_floor` and a slow `regen_rate`
        // against a heavier `graze_rate`, is the engine of the population's
        // boom/bust: the crowd eats the patches out, starves back, and recovers as
        // the patches refill. Tuned across many seeds to stay strictly bounded —
        // never extinct, never pinned at the cap (see the dynamics tests).
        Self {
            patch_count: 7,
            patch_radius_frac: 0.11,
            drift_speed: 0.0002,
            regen_rate: 0.05,
            graze_rate: 0.008,
            seek_strength: 1.0,
            patch_fraction: 0.35,
            graze_floor: default_graze_floor(),
        }
    }
}

#[cfg(test)]
mod tests;
