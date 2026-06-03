use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConfig {
    pub entity_scale: f32,
    pub max_population: u32,
    pub initial_entities: usize,
    pub spawn_radius_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub max_velocity: f32,
    pub max_entity_radius: f32,
    pub min_entity_radius: f32,
    pub grid_cell_size: f32,
    pub boundary_margin: f32,
    pub interaction_radius_offset: f32,
    pub velocity_bounce_factor: f32,
    pub center_pressure_strength: f32,
    pub particle_force_scale: f32,
    pub particle_friction: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConfig {
    pub size_energy_cost_factor: f32,
    pub movement_energy_cost: f32,
    /// Primary production: energy each creature grazes from the ambient food field
    /// per tick, scaled by `(1 - population_density)` so the field is finite. This
    /// is the ecosystem's only energy *input* (eating just transfers it), so it
    /// sets the carrying capacity — without it the closed system decays to a few
    /// survivors.
    pub ambient_energy_gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproductionConfig {
    pub reproduction_energy_threshold: f32,
    pub reproduction_energy_cost: f32,
    pub child_energy_factor: f32,
    pub child_spawn_radius: f32,
    pub population_density_factor: f32,
    pub min_reproduction_chance: f32,
    pub death_chance_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub population: PopulationConfig,
    pub physics: PhysicsConfig,
    pub energy: EnergyConfig,
    pub reproduction: ReproductionConfig,
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
                center_pressure_strength: 0.3,
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
        }
    }
}

impl SimulationConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: SimulationConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn create_default_config_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let default_config = SimulationConfig::default();
        default_config.save_to_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
