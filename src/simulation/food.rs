//! Deterministic drifting food field.
//!
//! Primary production is no longer spread uniformly across the world. It is
//! split into a thin uniform *base* everywhere plus a small set of drifting
//! [`FoodPatch`]es that wander, regenerate, and deplete as creatures graze them.
//! This gives the field *places worth moving to* — persistent gathering spots
//! that drift slowly, get grazed down when crowded, and regrow — so the
//! population visibly migrates and aggregates instead of sitting in a uniform
//! glow, while the base keeps between-patch creatures alive so the ecosystem
//! stays stable.
//!
//! Carrying capacity is preserved by *concentrating* the same production in
//! space, not removing it: the base plus the spatial average of the patches sum
//! to the old uniform `ambient_energy_gain` (see `Simulation::food_field_config`).
//!
//! Everything here is a pure function of the run seed + tick (via
//! [`mix_seed`](super::rng::mix_seed) / [`FastRng`]), so a run still reproduces
//! bit-for-bit. The field is updated once per tick in the serial phase and then
//! read immutably by the parallel per-entity compute.

use super::rng::{mix_seed, FastRng};
use rand::{Rng, SeedableRng};

/// Salt for the food-field RNG stream, distinct from every other per-tick stream.
const FOOD_SALT: u64 = 0xF00D_5EED_1234_ABCD;

/// Grazing never strips a patch below this fraction of its capacity, so a crowded
/// patch visibly dips (the "graze it down then move on" pressure) but always
/// keeps a gradient for creatures to climb — the gathering spot persists instead
/// of vanishing under a large crowd.
const GRAZE_FLOOR: f32 = 0.3;

/// A single circular nourishment patch sitting on top of the uniform base.
/// `intensity` is the patch's current bonus richness at its centre (energy per
/// tick a creature grazes there, above the base); it regrows toward `capacity`
/// and drops when grazed.
#[derive(Clone, Debug)]
pub struct FoodPatch {
    pub x: f32,
    pub y: f32,
    /// Falloff radius — the bonus drops smoothly to zero at this distance.
    pub radius: f32,
    /// Current bonus richness at the centre, in [0, `capacity`].
    pub intensity: f32,
    /// Maximum bonus richness this patch regrows toward.
    pub capacity: f32,
    /// Per-patch drift heading (radians); slowly re-rolled so patches wander.
    heading: f32,
}

/// The whole food field: a uniform base plus a fixed-size set of patches, with
/// the tuning that drives their drift, regrowth, and grazing depletion.
#[derive(Clone, Debug)]
pub struct FoodField {
    patches: Vec<FoodPatch>,
    cfg: FoodFieldConfig,
    half_world: f32,
}

/// Tuning for the food field, derived from the live config in
/// [`Simulation::food_field_config`](crate::simulation::Simulation).
#[derive(Clone, Debug)]
pub struct FoodFieldConfig {
    pub patch_count: usize,
    pub patch_radius: f32,
    pub drift_speed: f32,
    /// Regrowth per tick as a *fraction of capacity* (scale-invariant): a richer
    /// patch refills proportionally faster.
    pub regen_rate: f32,
    pub graze_rate: f32,
    /// Uniform production everywhere (energy/tick), keeping between-patch
    /// creatures alive so the population stays stable.
    pub base: f32,
    /// Bonus production at the centre of a full patch (energy/tick), on top of
    /// the base. Patch count × radius × this are balanced so the world-average
    /// production matches the old uniform field.
    pub patch_peak: f32,
}

impl FoodField {
    /// Build the initial field from the seed: patches scattered uniformly across
    /// the playable area, each starting at full intensity.
    pub fn new(seed: u64, world_size: f32, cfg: FoodFieldConfig) -> Self {
        let half_world = world_size / 2.0;
        let mut rng = FastRng::seed_from_u64(mix_seed(seed, FOOD_SALT, 0));
        // Keep patches inside the interior the creatures actually roam (the edge
        // repulsion holds them off the rim), so food never strands at the wall.
        let spread = half_world * 0.7;
        let patches = (0..cfg.patch_count)
            .map(|_| {
                let angle = rng.random_range(0.0..std::f32::consts::TAU);
                let dist = spread * rng.random::<f32>().sqrt();
                FoodPatch {
                    x: dist * angle.cos(),
                    y: dist * angle.sin(),
                    radius: cfg.patch_radius,
                    intensity: cfg.patch_peak,
                    capacity: cfg.patch_peak,
                    heading: rng.random_range(0.0..std::f32::consts::TAU),
                }
            })
            .collect();
        Self {
            patches,
            cfg,
            half_world,
        }
    }

    /// Advance the field one tick: each patch wanders slowly and continuously and
    /// regrows toward capacity in place. Patches are *persistent* gathering spots
    /// that migrate gradually (the drift) and pulse richer/poorer (regrowth vs the
    /// grazing depletion applied in the apply phase) — so creatures can actually
    /// pile up on a spot, follow it as it drifts, and move on when it is grazed
    /// down. Deterministic in (seed, step).
    pub fn update(&mut self, seed: u64, step: u32) {
        let bound = self.half_world * 0.82;
        for (i, p) in self.patches.iter_mut().enumerate() {
            // Gentle wandering: nudge the heading a little each tick, then step.
            let mut hrng =
                FastRng::seed_from_u64(mix_seed(seed ^ FOOD_SALT, i as u64, step as u64));
            p.heading += hrng.random_range(-0.18..0.18);
            p.x += p.heading.cos() * self.cfg.drift_speed;
            p.y += p.heading.sin() * self.cfg.drift_speed;
            // Reflect off the interior bound so patches turn back rather than
            // piling onto the edge.
            if p.x.abs() > bound {
                p.x = p.x.clamp(-bound, bound);
                p.heading = std::f32::consts::PI - p.heading;
            }
            if p.y.abs() > bound {
                p.y = p.y.clamp(-bound, bound);
                p.heading = -p.heading;
            }

            // Regrow toward capacity in place — a depleted spot recovers over time
            // rather than teleporting away, so gathering can persist and migrate.
            // Regrowth is a fraction of capacity, so it scales with patch richness
            // (and thus with world size): a richer patch refills proportionally
            // faster, so a big-world crowd can't strip patches to nothing forever.
            p.intensity = (p.intensity + p.capacity * self.cfg.regen_rate).min(p.capacity);
        }
    }

    /// Total production a creature at `(x, y)` grazes this tick: the uniform base
    /// plus every patch's bonus, each falling off smoothly (squared) to zero at
    /// its radius and scaled by the patch's current intensity. This replaces the
    /// old flat `ambient_energy_gain` — the same total, concentrated in space.
    pub fn gain_at(&self, x: f32, y: f32) -> f32 {
        let mut gain = self.cfg.base;
        for p in &self.patches {
            let dx = x - p.x;
            let dy = y - p.y;
            let d2 = dx * dx + dy * dy;
            let r2 = p.radius * p.radius;
            if d2 < r2 {
                // Smooth quadratic falloff: 1 at the centre, 0 at the rim.
                let t = 1.0 - d2 / r2;
                gain += p.intensity * t * t;
            }
        }
        gain
    }

    /// The bonus part of the gain only (patches, not the base). Used by the
    /// food-seeking force so creatures climb toward patches; the flat base has no
    /// gradient and contributes nothing to seeking.
    pub fn patch_gain_at(&self, x: f32, y: f32) -> f32 {
        let mut gain = 0.0;
        for p in &self.patches {
            let dx = x - p.x;
            let dy = y - p.y;
            let d2 = dx * dx + dy * dy;
            let r2 = p.radius * p.radius;
            if d2 < r2 {
                let t = 1.0 - d2 / r2;
                gain += p.intensity * t * t;
            }
        }
        gain
    }

    /// Deplete patches near `(x, y)` to reflect a creature grazing there. Called
    /// once per fed creature in the serial apply phase, after the compute, so the
    /// field the parallel compute read stays fixed for the whole tick. Only the
    /// patch bonus depletes — the uniform base is inexhaustible.
    pub fn graze(&mut self, x: f32, y: f32, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        for p in &mut self.patches {
            let dx = x - p.x;
            let dy = y - p.y;
            let d2 = dx * dx + dy * dy;
            let r2 = p.radius * p.radius;
            if d2 < r2 {
                let t = 1.0 - d2 / r2;
                let floor = p.capacity * GRAZE_FLOOR;
                p.intensity = (p.intensity - amount * self.cfg.graze_rate * t * t).max(floor);
            }
        }
    }

    pub fn patches(&self) -> &[FoodPatch] {
        &self.patches
    }
}

#[cfg(test)]
mod tests;
