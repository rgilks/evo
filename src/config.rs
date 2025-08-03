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
    pub velocity_bounce_factor: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            entity_scale: 1.0,
            max_population: 8000,
            initial_entities: 4000,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = SimulationConfig::default();

        // Test default values
        assert_eq!(config.entity_scale, 1.0);
        assert_eq!(config.max_population, 8000);
        assert_eq!(config.initial_entities, 4000);
        assert_eq!(config.max_velocity, 2.0);
        assert_eq!(config.max_entity_radius, 20.0);
        assert_eq!(config.min_entity_radius, 1.0);
        assert_eq!(config.spawn_radius_factor, 0.2);
        assert_eq!(config.grid_cell_size, 25.0);
        assert_eq!(config.boundary_margin, 5.0);
        assert_eq!(config.interaction_radius_offset, 15.0);
        assert_eq!(config.reproduction_energy_threshold, 0.8);
        assert_eq!(config.reproduction_energy_cost, 0.7);
        assert_eq!(config.child_energy_factor, 0.4);
        assert_eq!(config.child_spawn_radius, 15.0);
        assert_eq!(config.size_energy_cost_factor, 0.15);
        assert_eq!(config.movement_energy_cost, 0.1);
        assert_eq!(config.population_density_factor, 0.8);
        assert_eq!(config.min_reproduction_chance, 0.05);
        assert_eq!(config.death_chance_factor, 0.1);
        assert_eq!(config.velocity_bounce_factor, 0.8);
    }

    #[test]
    fn test_config_serialization() {
        let config = SimulationConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SimulationConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.entity_scale, deserialized.entity_scale);
        assert_eq!(config.max_population, deserialized.max_population);
        assert_eq!(config.initial_entities, deserialized.initial_entities);
        assert_eq!(config.max_velocity, deserialized.max_velocity);
        assert_eq!(config.max_entity_radius, deserialized.max_entity_radius);
        assert_eq!(config.min_entity_radius, deserialized.min_entity_radius);
        assert_eq!(config.spawn_radius_factor, deserialized.spawn_radius_factor);
        assert_eq!(config.grid_cell_size, deserialized.grid_cell_size);
        assert_eq!(config.boundary_margin, deserialized.boundary_margin);
        assert_eq!(
            config.interaction_radius_offset,
            deserialized.interaction_radius_offset
        );
        assert_eq!(
            config.reproduction_energy_threshold,
            deserialized.reproduction_energy_threshold
        );
        assert_eq!(
            config.reproduction_energy_cost,
            deserialized.reproduction_energy_cost
        );
        assert_eq!(config.child_energy_factor, deserialized.child_energy_factor);
        assert_eq!(config.child_spawn_radius, deserialized.child_spawn_radius);
        assert_eq!(
            config.size_energy_cost_factor,
            deserialized.size_energy_cost_factor
        );
        assert_eq!(
            config.movement_energy_cost,
            deserialized.movement_energy_cost
        );
        assert_eq!(
            config.population_density_factor,
            deserialized.population_density_factor
        );
        assert_eq!(
            config.min_reproduction_chance,
            deserialized.min_reproduction_chance
        );
        assert_eq!(config.death_chance_factor, deserialized.death_chance_factor);
        assert_eq!(
            config.velocity_bounce_factor,
            deserialized.velocity_bounce_factor
        );
    }

    #[test]
    fn test_config_clone() {
        let config = SimulationConfig::default();
        let cloned = config.clone();

        assert_eq!(config.entity_scale, cloned.entity_scale);
        assert_eq!(config.max_population, cloned.max_population);
        assert_eq!(config.initial_entities, cloned.initial_entities);
        assert_eq!(config.max_velocity, cloned.max_velocity);
        assert_eq!(config.max_entity_radius, cloned.max_entity_radius);
        assert_eq!(config.min_entity_radius, cloned.min_entity_radius);
        assert_eq!(config.spawn_radius_factor, cloned.spawn_radius_factor);
        assert_eq!(config.grid_cell_size, cloned.grid_cell_size);
        assert_eq!(config.boundary_margin, cloned.boundary_margin);
        assert_eq!(
            config.interaction_radius_offset,
            cloned.interaction_radius_offset
        );
        assert_eq!(
            config.reproduction_energy_threshold,
            cloned.reproduction_energy_threshold
        );
        assert_eq!(
            config.reproduction_energy_cost,
            cloned.reproduction_energy_cost
        );
        assert_eq!(config.child_energy_factor, cloned.child_energy_factor);
        assert_eq!(config.child_spawn_radius, cloned.child_spawn_radius);
        assert_eq!(
            config.size_energy_cost_factor,
            cloned.size_energy_cost_factor
        );
        assert_eq!(config.movement_energy_cost, cloned.movement_energy_cost);
        assert_eq!(
            config.population_density_factor,
            cloned.population_density_factor
        );
        assert_eq!(
            config.min_reproduction_chance,
            cloned.min_reproduction_chance
        );
        assert_eq!(config.death_chance_factor, cloned.death_chance_factor);
        assert_eq!(config.velocity_bounce_factor, cloned.velocity_bounce_factor);
    }

    #[test]
    fn test_save_and_load_config() {
        let config = SimulationConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Test save
        let save_result = config.save_to_file(path);
        assert!(save_result.is_ok());

        // Test load
        let load_result = SimulationConfig::load_from_file(path);
        assert!(load_result.is_ok());

        let loaded_config = load_result.unwrap();
        assert_eq!(config.entity_scale, loaded_config.entity_scale);
        assert_eq!(config.max_population, loaded_config.max_population);
        assert_eq!(config.initial_entities, loaded_config.initial_entities);
        assert_eq!(config.max_velocity, loaded_config.max_velocity);
        assert_eq!(config.max_entity_radius, loaded_config.max_entity_radius);
        assert_eq!(config.min_entity_radius, loaded_config.min_entity_radius);
        assert_eq!(
            config.spawn_radius_factor,
            loaded_config.spawn_radius_factor
        );
        assert_eq!(config.grid_cell_size, loaded_config.grid_cell_size);
        assert_eq!(config.boundary_margin, loaded_config.boundary_margin);
        assert_eq!(
            config.interaction_radius_offset,
            loaded_config.interaction_radius_offset
        );
        assert_eq!(
            config.reproduction_energy_threshold,
            loaded_config.reproduction_energy_threshold
        );
        assert_eq!(
            config.reproduction_energy_cost,
            loaded_config.reproduction_energy_cost
        );
        assert_eq!(
            config.child_energy_factor,
            loaded_config.child_energy_factor
        );
        assert_eq!(config.child_spawn_radius, loaded_config.child_spawn_radius);
        assert_eq!(
            config.size_energy_cost_factor,
            loaded_config.size_energy_cost_factor
        );
        assert_eq!(
            config.movement_energy_cost,
            loaded_config.movement_energy_cost
        );
        assert_eq!(
            config.population_density_factor,
            loaded_config.population_density_factor
        );
        assert_eq!(
            config.min_reproduction_chance,
            loaded_config.min_reproduction_chance
        );
        assert_eq!(
            config.death_chance_factor,
            loaded_config.death_chance_factor
        );
        assert_eq!(
            config.velocity_bounce_factor,
            loaded_config.velocity_bounce_factor
        );
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = SimulationConfig::load_from_file("nonexistent_file.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_json() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Write invalid JSON
        fs::write(path, "invalid json content").unwrap();

        let result = SimulationConfig::load_from_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_default_config_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let result = SimulationConfig::create_default_config_file(path);
        assert!(result.is_ok());

        // Verify the file was created and contains valid JSON
        let load_result = SimulationConfig::load_from_file(path);
        assert!(load_result.is_ok());
    }

    #[test]
    fn test_custom_config_values() {
        let mut config = SimulationConfig::default();

        // Modify some values
        config.entity_scale = 1.0;
        config.max_population = 1000;
        config.initial_entities = 200;
        config.max_velocity = 3.0;
        config.max_entity_radius = 25.0;
        config.min_entity_radius = 2.0;
        config.spawn_radius_factor = 0.3;
        config.grid_cell_size = 30.0;
        config.boundary_margin = 10.0;
        config.interaction_radius_offset = 20.0;
        config.reproduction_energy_threshold = 0.9;
        config.reproduction_energy_cost = 0.8;
        config.child_energy_factor = 0.5;
        config.child_spawn_radius = 20.0;
        config.size_energy_cost_factor = 0.2;
        config.movement_energy_cost = 0.15;
        config.population_density_factor = 0.9;
        config.min_reproduction_chance = 0.1;
        config.death_chance_factor = 0.2;
        config.velocity_bounce_factor = 0.9;

        // Test that values were set correctly
        assert_eq!(config.entity_scale, 1.0);
        assert_eq!(config.max_population, 1000);
        assert_eq!(config.initial_entities, 200);
        assert_eq!(config.max_velocity, 3.0);
        assert_eq!(config.max_entity_radius, 25.0);
        assert_eq!(config.min_entity_radius, 2.0);
        assert_eq!(config.spawn_radius_factor, 0.3);
        assert_eq!(config.grid_cell_size, 30.0);
        assert_eq!(config.boundary_margin, 10.0);
        assert_eq!(config.interaction_radius_offset, 20.0);
        assert_eq!(config.reproduction_energy_threshold, 0.9);
        assert_eq!(config.reproduction_energy_cost, 0.8);
        assert_eq!(config.child_energy_factor, 0.5);
        assert_eq!(config.child_spawn_radius, 20.0);
        assert_eq!(config.size_energy_cost_factor, 0.2);
        assert_eq!(config.movement_energy_cost, 0.15);
        assert_eq!(config.population_density_factor, 0.9);
        assert_eq!(config.min_reproduction_chance, 0.1);
        assert_eq!(config.death_chance_factor, 0.2);
        assert_eq!(config.velocity_bounce_factor, 0.9);
    }

    #[test]
    fn test_config_debug_format() {
        let config = SimulationConfig::default();
        let debug_str = format!("{:?}", config);

        // Should contain some key fields
        assert!(debug_str.contains("entity_scale"));
        assert!(debug_str.contains("max_population"));
        assert!(debug_str.contains("initial_entities"));
    }
}
