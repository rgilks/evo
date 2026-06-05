use crate::components::Position;
use crate::config::SimulationConfig;
use crate::genes::Genes;
use rand::prelude::*;

/// Reproduction system - handles entity reproduction and population control
pub struct ReproductionSystem;

impl super::System for ReproductionSystem {
    fn run(&self, ctx: &mut super::EntityContext) {
        let global_density = ctx.population_density;
        // Reproduction is throttled by LOCAL crowding (neighbours within sense range),
        // not the global population. A lineage that reaches an open patch reproduces
        // freely and blooms into it; a crowded one stalls. This is what makes new
        // populations visibly bloom and spread rather than the whole field sitting
        // at a uniform global equilibrium.
        let local_crowding = (ctx.nearby_entities.len() as f32 / 20.0).min(1.0);
        ctx.should_reproduce = self.check_reproduction(
            ctx.new_energy,
            ctx.energy_max,
            ctx.genes,
            local_crowding,
            ctx.config,
            &mut ctx.rng,
        );
        // Death scales with GLOBAL density — a system-wide culling pressure,
        // deliberately unlike reproduction's local throttle above.
        if self.check_death(global_density, ctx.config, &mut ctx.rng) {
            ctx.new_energy = 0.0; // Kill the entity
        }
        if ctx.should_reproduce {
            // Offspring is spawned in apply_entity_updates; deduct the parent's cost here.
            ctx.new_energy *= ctx.config.reproduction.reproduction_energy_cost;
        }
    }
}

impl ReproductionSystem {
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
            && rng.gen::<f32>() < reproduction_chance
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
            let dx = rng.gen_range(
                -config.reproduction.child_spawn_radius..config.reproduction.child_spawn_radius,
            );
            let dy = rng.gen_range(
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

    pub fn check_death(
        &self,
        population_density: f32,
        config: &SimulationConfig,
        rng: &mut impl Rng,
    ) -> bool {
        let death_chance = population_density * config.reproduction.death_chance_factor;
        rng.gen::<f32>() < death_chance
    }
}

#[cfg(test)]
mod tests;
