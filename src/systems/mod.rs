pub mod energy;
pub mod interaction;
pub mod movement;
pub mod reproduction;

pub use energy::*;
pub use interaction::*;
pub use movement::*;
pub use reproduction::*;

use crate::components::{Color, Energy, MovementStyle, Position, Size, Velocity};
use crate::config::SimulationConfig;
use crate::genes::Genes;
use crate::simulation::food::FoodField;
use crate::simulation::FastRng;
use hecs::Entity;
use std::collections::HashMap;

/// A read-only snapshot of a neighbour's hot fields, captured once per tick so
/// the per-entity compute reads contiguous cached data instead of doing
/// scattered `world.get` lookups. Keyed by entity in [`NeighborCache`].
pub struct NeighborSnapshot {
    pub pos: Position,
    pub genes: Genes,
    pub energy: Energy,
    pub size: Size,
    pub velocity: Velocity,
}

/// Per-tick neighbour data, rebuilt each tick alongside the spatial grid.
pub type NeighborCache = HashMap<Entity, NeighborSnapshot>;

/// Per-entity working state shared by every system during the compute phase.
/// Systems read the immutable inputs and mutate the `new_*` fields in turn; the
/// orchestrator then reads the result into an `EntityUpdate`.
pub struct EntityContext<'a> {
    pub genes: &'a Genes,
    pub pos: &'a Position,
    pub size: &'a Size,
    pub nearby_entities: &'a [Entity],
    pub cache: &'a NeighborCache,
    /// Global particle-life interaction matrix, indexed `[self_sector][other_sector]`.
    pub particle_matrix: &'a [[f32; 6]; 6],
    /// Read-only snapshot of the food field for this tick — drives both the
    /// primary-production gain (energy) and the food-seeking force (movement).
    pub food_field: &'a FoodField,
    pub config: &'a SimulationConfig,
    pub world_size: f32,
    pub population_density: f32,
    /// Slow-moving (lagged) crowding pressure — the death rate reads this instead
    /// of `population_density` so mortality lags the population (boom/bust). See
    /// `Simulation::crowding_pressure`.
    pub crowding_pressure: f32,
    pub energy_max: f32,

    pub new_pos: Position,
    pub new_velocity: Velocity,
    pub new_energy: f32,
    /// Energy grazed from the food field this tick; the serial apply phase uses
    /// it to deplete the patches the creature fed on.
    pub grazed: f32,
    pub should_reproduce: bool,
    pub eaten_entity: Option<Entity>,
    /// Per-entity, per-tick RNG (seeded from the world seed + entity id + tick),
    /// so randomness is reproducible and independent of thread scheduling.
    pub rng: FastRng,
}

/// Uniform interface for the per-entity systems. Each system reads from and
/// mutates the shared [`EntityContext`]; the orchestrator runs them in order.
pub trait System {
    fn run(&self, ctx: &mut EntityContext);
}

/// The standard creature component bundle. Centralizes the archetype so the
/// initial spawn and reproduction cannot drift out of sync.
pub type CreatureBundle = (
    Position,
    Energy,
    Size,
    Genes,
    Color,
    Velocity,
    MovementStyle,
);

/// Energy needed per unit of body radius; radius scales with current energy and
/// the size-factor gene.
const ENERGY_PER_RADIUS_UNIT: f32 = 15.0;

/// Body radius derived from current energy and the size-factor gene, clamped to
/// `[min, max]`. The single source of truth for sizing — used by both the
/// creature bundle (spawn/reproduction) and the per-tick size update.
pub fn derive_radius(energy: f32, size_factor: f32, min: f32, max: f32) -> f32 {
    (energy / ENERGY_PER_RADIUS_UNIT * size_factor).clamp(min, max)
}

/// Build a creature from its genes and energy, deriving size, color, and
/// movement style. `max_radius` / `min_radius` bound the size clamp (offspring
/// and the initial population use different upper clamps).
pub fn creature_bundle(
    pos: Position,
    energy_current: f32,
    energy_max: f32,
    genes: Genes,
    max_radius: f32,
    min_radius: f32,
) -> CreatureBundle {
    let radius = derive_radius(energy_current, genes.size_factor(), min_radius, max_radius);
    let color = genes.get_color();
    let movement_style = genes.behavior.movement_style.clone();
    (
        pos,
        Energy {
            current: energy_current,
            max: energy_max,
        },
        Size { radius },
        genes,
        color,
        Velocity { x: 0.0, y: 0.0 },
        movement_style,
    )
}
