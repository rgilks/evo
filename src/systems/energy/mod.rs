use crate::components::Size;
use crate::config::SimulationConfig;
use crate::genes::Genes;

/// Energy system - handles energy consumption and metabolism
pub struct EnergySystem;

impl super::System for EnergySystem {
    fn run(&self, ctx: &mut super::EntityContext) {
        self.update_energy(
            &mut ctx.new_energy,
            ctx.size,
            ctx.genes,
            ctx.config,
            ctx.population_density,
        );
    }
}

impl EnergySystem {
    pub fn update_energy(
        &self,
        new_energy: &mut f32,
        size: &Size,
        genes: &Genes,
        config: &SimulationConfig,
        population_density: f32,
    ) {
        // Primary production: graze the ambient food field. The field is finite, so
        // the per-capita share shrinks as the population grows — this is what gives
        // the ecosystem a carrying capacity instead of decaying to a few survivors.
        *new_energy += config.energy.ambient_energy_gain * (1.0 - population_density).max(0.0);

        // Metabolism: larger entities cost more to maintain.
        let size_energy_cost = size.radius * config.energy.size_energy_cost_factor;
        *new_energy -= (genes.energy_loss_rate() + size_energy_cost) / genes.energy_efficiency();
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
