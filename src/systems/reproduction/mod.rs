use crate::components::Position;
use crate::config::SimulationConfig;
use crate::genes::Genes;
use rand::Rng;

/// Reproduction system - handles entity reproduction and population control
pub struct ReproductionSystem;

impl super::System for ReproductionSystem {
    fn run(&self, ctx: &mut super::EntityContext) {
        // Reproduction is throttled by LOCAL crowding (neighbours within sense range),
        // not the global population. A lineage that reaches an open patch reproduces
        // freely and blooms into it; a crowded one stalls. This is what makes new
        // populations visibly bloom and spread rather than the whole field sitting
        // at a uniform global equilibrium.
        let local_crowding = (ctx.nearby_entities.len() as f32 / 20.0).min(1.0);
        // Negative frequency-dependent selection: weight the crowding by how many
        // neighbours share this creature's hue. A creature surrounded by its own
        // colour is throttled harder (its niche is saturated), a rare colour breeds
        // freely — so several colour lineages coexist instead of one taking over.
        let same_hue_fraction = self.same_hue_fraction(ctx);
        let hue_pressure = same_hue_fraction * ctx.config.reproduction.hue_crowding_factor;
        let effective_crowding = (local_crowding * (1.0 + hue_pressure)).min(1.0);
        ctx.should_reproduce = self.check_reproduction(
            ctx.new_energy,
            ctx.energy_max,
            ctx.genes,
            effective_crowding,
            ctx.config,
            &mut ctx.rng,
        );
        // Death scales with the LAGGED crowding pressure (a slow low-pass of the
        // global density), not the instantaneous density — so mortality arrives
        // late and the population overshoots then crashes, the boom/bust cycle.
        // The live density gates it off near the safety floor so a deep bust can't
        // spiral to extinction. A system-wide culling pressure, deliberately unlike
        // reproduction's local throttle above.
        if self.check_death(
            ctx.crowding_pressure,
            ctx.population_density,
            ctx.config,
            &mut ctx.rng,
        ) {
            ctx.new_energy = 0.0; // Kill the entity
        }
        if ctx.should_reproduce {
            // Offspring is spawned in apply_entity_updates; deduct the parent's cost here.
            ctx.new_energy *= ctx.config.reproduction.reproduction_energy_cost;
        }
    }
}

impl ReproductionSystem {
    /// Fraction of nearby creatures whose hue is within ~one colour sector of this
    /// creature's (hue is circular in `0..1`). Drives the frequency-dependent
    /// reproduction throttle that keeps multiple colour lineages coexisting.
    fn same_hue_fraction(&self, ctx: &super::EntityContext) -> f32 {
        if ctx.nearby_entities.is_empty() {
            return 0.0;
        }
        let h = ctx.genes.appearance.hue;
        let mut same = 0usize;
        for e in ctx.nearby_entities {
            if let Some(n) = ctx.cache.get(e) {
                let mut d = (n.genes.appearance.hue - h).abs();
                d = d.min(1.0 - d); // wrap-around distance on the colour wheel
                if d < 1.0 / 12.0 {
                    same += 1;
                }
            }
        }
        same as f32 / ctx.nearby_entities.len() as f32
    }

    pub fn check_reproduction(
        &self,
        energy: f32,
        max_energy: f32,
        genes: &Genes,
        crowding: f32,
        config: &SimulationConfig,
        rng: &mut impl Rng,
    ) -> bool {
        // Local crowding scales the base reproduction rate down, but the
        // multiplier is floored at `min_reproduction_chance` so a saturated patch
        // never suppresses reproduction *completely*. The floor is on the crowding
        // multiplier, not the final probability — and with default params
        // (factor 0.8, crowding ≤ 1) the multiplier stays ≥ 0.2, so the floor only
        // binds when `population_density_factor` is pushed high.
        let crowding_factor = (1.0 - crowding * config.reproduction.population_density_factor)
            .max(config.reproduction.min_reproduction_chance);
        let reproduction_chance = genes.reproduction_rate() * crowding_factor;

        energy > max_energy * config.reproduction.reproduction_energy_threshold
            && rng.random::<f32>() < reproduction_chance
    }

    pub fn create_offspring(
        &self,
        parent_genes: &Genes,
        parent_energy_max: f32,
        parent_pos: &Position,
        config: &SimulationConfig,
        rng: &mut impl Rng,
    ) -> crate::systems::CreatureBundle {
        let child_genes = parent_genes.mutate(rng);
        let child_energy = parent_energy_max * config.reproduction.child_energy_factor;

        // Use uniform distribution in a circle for child positioning
        let (dx, dy) = loop {
            let dx = rng.random_range(
                -config.reproduction.child_spawn_radius..config.reproduction.child_spawn_radius,
            );
            let dy = rng.random_range(
                -config.reproduction.child_spawn_radius..config.reproduction.child_spawn_radius,
            );
            let distance_sq = dx * dx + dy * dy;
            if distance_sq
                <= config.reproduction.child_spawn_radius * config.reproduction.child_spawn_radius
            {
                break (dx, dy);
            }
        };

        crate::systems::creature_bundle(
            Position {
                x: parent_pos.x + dx,
                y: parent_pos.y + dy,
            },
            child_energy,
            parent_energy_max,
            child_genes,
            15.0,
            config.physics.min_entity_radius,
        )
    }

    /// Density-dependent death. The *rate* is driven by `pressure` (the lagged
    /// crowding signal — this is what makes mortality lag the population and
    /// produces the boom/bust overshoot), but it is gated to zero once the *live*
    /// density falls to the `death_floor_density` safety floor: below the floor no
    /// density-death happens at all, so a deep bust dives toward the floor but can
    /// never spiral through it to extinction. The gate ramps in smoothly over a
    /// short band above the floor (a smoothstep) so the cycle stays gentle.
    pub fn check_death(
        &self,
        pressure: f32,
        live_density: f32,
        config: &SimulationConfig,
        rng: &mut impl Rng,
    ) -> bool {
        let floor = config.reproduction.death_floor_density;
        let band = (floor).max(0.001); // ramp over one floor-width above the floor
        let gate = ((live_density - floor) / band).clamp(0.0, 1.0);
        let gate = gate * gate * (3.0 - 2.0 * gate); // smoothstep
        let death_chance = pressure * config.reproduction.death_chance_factor * gate;
        rng.random::<f32>() < death_chance
    }
}

#[cfg(test)]
mod tests;
