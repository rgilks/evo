use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub entity_scale: f32,
    pub max_population: u32,
    pub initial_entities: usize,
    pub max_velocity: f32,
    pub max_entity_radius: f32,
    pub min_entity_radius: f32,
    pub spawn_radius_factor: f32,
    pub grid_cell_size: f32,
    pub boundary_margin: f32,
    pub interaction_radius_offset: f32,
    pub reproduction_energy_threshold: f32,
    pub reproduction_energy_cost: f32,
    pub child_energy_factor: f32,
    pub child_spawn_radius: f32,
    pub size_energy_cost_factor: f32,
    pub movement_energy_cost: f32,
    pub population_density_factor: f32,
    pub min_reproduction_chance: f32,
    pub death_chance_factor: f32,
    pub drift_compensation_x: f32,
    pub drift_compensation_y: f32,
    pub velocity_bounce_factor: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            entity_scale: 0.5,
            max_population: 2000,
            initial_entities: 500,
            max_velocity: 2.0,
            max_entity_radius: 20.0,
            min_entity_radius: 1.0,
            spawn_radius_factor: 0.2,
            grid_cell_size: 25.0,
            boundary_margin: 5.0,
            interaction_radius_offset: 15.0,
            reproduction_energy_threshold: 0.8,
            reproduction_energy_cost: 0.7,
            child_energy_factor: 0.4,
            child_spawn_radius: 15.0,
            size_energy_cost_factor: 0.15,
            movement_energy_cost: 0.1,
            population_density_factor: 0.8,
            min_reproduction_chance: 0.05,
            death_chance_factor: 0.1,
            drift_compensation_x: 0.5,
            drift_compensation_y: 0.4,
            velocity_bounce_factor: 0.8,
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
