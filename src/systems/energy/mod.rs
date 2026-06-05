use crate::components::Size;
use crate::config::SimulationConfig;
use crate::genes::Genes;

/// Energy system - handles energy consumption and metabolism
pub struct EnergySystem;

impl super::System for EnergySystem {
    fn run(&self, ctx: &mut super::EntityContext) {
        // Primary production now comes from the local food field: graze a share
        // proportional to how close this creature is to the drifting patches,
        // rather than a flat per-capita amount. The total production across the
        // world is balanced to the old uniform field (see `food_field_config`),
        // so the carrying capacity is preserved — the food is concentrated in
        // space, not removed. `(1 - density)` still makes the field finite.
        let local_gain = ctx.food_field.gain_at(ctx.new_pos.x, ctx.new_pos.y)
            * (1.0 - ctx.population_density).max(0.0);
        ctx.grazed = local_gain;
        ctx.new_energy += local_gain;
        self.apply_metabolism(&mut ctx.new_energy, ctx.size, ctx.genes, ctx.config);
    }
}

impl EnergySystem {
    /// Metabolic upkeep: every creature pays a base loss plus a size-dependent
    /// maintenance cost each tick, scaled down by its efficiency gene.
    fn apply_metabolism(
        &self,
        new_energy: &mut f32,
        size: &Size,
        genes: &Genes,
        config: &SimulationConfig,
    ) {
        let size_energy_cost = size.radius * config.energy.size_energy_cost_factor;
        *new_energy -= (genes.energy_loss_rate() + size_energy_cost) / genes.energy_efficiency();
    }

    /// Direct primary-production + metabolism step, used by tests that exercise
    /// the energy balance without a full simulation. Mirrors `run` with a flat
    /// food gain in place of the spatial field.
    #[cfg(test)]
    pub fn update_energy(
        &self,
        new_energy: &mut f32,
        size: &Size,
        genes: &Genes,
        config: &SimulationConfig,
        population_density: f32,
    ) {
        *new_energy += config.energy.ambient_energy_gain * (1.0 - population_density).max(0.0);
        self.apply_metabolism(new_energy, size, genes, config);
    }

    pub fn calculate_new_size(&self, energy: f32, genes: &Genes, config: &SimulationConfig) -> f32 {
        super::derive_radius(
            energy,
            genes.size_factor(),
            config.physics.min_entity_radius,
            config.physics.max_entity_radius,
        )
    }
}

#[cfg(test)]
mod tests;
